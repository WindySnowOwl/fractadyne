//! The palette browser's thumbnails: the view's current iteration texture colored through each
//! palette in turn, into one atlas texture that egui draws from.
//!
//! ⭐**Nothing is iterated.** Coloring is its own pass over the iteration texture the view already
//! holds, so a thumbnail costs one small color pass, however deep the view: fifty palettes are
//! fifty draws of `fs_color` into cells of one texture, in one submission. Each thumbnail uses the
//! coloring the view was last drawn with (`ViewResources::last_cu`) — method, cycle, offset,
//! normalization, lighting — with only the palette swapped, so it shows what choosing that
//! palette would show.
//!
//! ⚠**Color space.** `fs_color` writes display values into a non-sRGB target, which is what the
//! screen shows. egui samples a registered texture as LINEAR and encodes it for display, so the
//! atlas is registered through an sRGB VIEW of the same texture: sampling decodes the display values
//! to linear and egui's encode puts them back, unchanged. Registered through the plain view, every
//! thumbnail would come out pale.

use crate::{make_color_bg, make_lut_buffer, write_lut, ColorUniforms, Renderer};
use egui_wgpu::wgpu;

/// One palette for [`render_palette_atlas`]: its baked table, as `Lut::entries`, and whether to
/// interpolate between entries (`Lut::smooth`).
pub struct AtlasPalette {
    pub lut: Vec<[f32; 4]>,
    pub smooth: bool,
}

/// A rendered atlas. Hold it to keep drawing it; hand it back to [`render_palette_atlas`] to redraw
/// in place, or to [`free_palette_atlas`] when done.
pub struct PaletteAtlas {
    texture: wgpu::Texture,
    /// What egui draws it by.
    pub id: egui::TextureId,
    /// One thumbnail's size in texels.
    pub cell: [u32; 2],
    pub cols: u32,
    /// How many thumbnails it holds, in the order the palettes were given.
    pub count: usize,
}

impl PaletteAtlas {
    /// Where thumbnail `k` sits in the atlas, in uv (0..1).
    pub fn uv(&self, k: usize) -> egui::Rect {
        let size = self.texture.size();
        let (col, row) = ((k as u32) % self.cols, (k as u32) / self.cols);
        let (w, h) = (size.width as f32, size.height as f32);
        egui::Rect::from_min_max(
            egui::pos2((col * self.cell[0]) as f32 / w, (row * self.cell[1]) as f32 / h),
            egui::pos2(((col + 1) * self.cell[0]) as f32 / w, ((row + 1) * self.cell[1]) as f32 / h),
        )
    }
}

/// Why an atlas could not be drawn.
#[derive(Debug, PartialEq, Eq)]
pub enum AtlasError {
    /// The renderer is not installed, or the view has not drawn a frame yet.
    NoView,
    /// The atlas would be larger than this device's largest texture.
    TooLarge,
    /// No palettes, or a zero-sized cell.
    Empty,
}

/// Draw `palettes` over view `view_id`'s current iteration texture into an atlas of `cols` columns
/// of `cell`-sized thumbnails. `reuse` is the previous atlas: redrawn in place (keeping its egui id)
/// when the size matches, freed otherwise.
pub fn render_palette_atlas(
    rs: &egui_wgpu::RenderState,
    view_id: u32,
    palettes: &[AtlasPalette],
    cell: [u32; 2],
    cols: u32,
    reuse: Option<PaletteAtlas>,
) -> Result<PaletteAtlas, AtlasError> {
    if palettes.is_empty() || cell[0] == 0 || cell[1] == 0 || cols == 0 {
        if let Some(old) = reuse {
            free_palette_atlas(rs, old);
        }
        return Err(AtlasError::Empty);
    }
    // What the draw needs from the live renderer, cloned out so the lock is not held while encoding.
    let found = {
        let guard = rs.renderer.read();
        guard.callback_resources.get::<Renderer>().and_then(|r| {
            let v = r.views.get(&view_id)?;
            let cu = v.last_cu?;
            v.rendered.then(|| {
                (
                    r.color_pipeline.clone(),
                    r.color_bgl.clone(),
                    v.tex_view.clone(),
                    v.aux_view.clone(),
                    v.last_ss.max(1),
                    cu,
                    r.target_format,
                )
            })
        })
    };
    let Some((pipeline, bgl, tex_view, aux_view, ss, cu, format)) = found else {
        if let Some(old) = reuse {
            free_palette_atlas(rs, old);
        }
        return Err(AtlasError::NoView);
    };
    let device = &rs.device;
    let rows = (palettes.len() as u32).div_ceil(cols);
    let size = [cols * cell[0], rows * cell[1]];
    let max = device.limits().max_texture_dimension_2d;
    if size[0] > max || size[1] > max {
        if let Some(old) = reuse {
            free_palette_atlas(rs, old);
        }
        return Err(AtlasError::TooLarge);
    }
    let srgb = format.add_srgb_suffix();
    let (texture, old_id) = match reuse {
        Some(old) if old.texture.size().width == size[0] && old.texture.size().height == size[1] => {
            (old.texture, Some(old.id))
        }
        other => {
            if let Some(old) = other {
                free_palette_atlas(rs, old);
            }
            let view_formats: &[wgpu::TextureFormat] = if srgb != format { &[srgb] } else { &[] };
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("fractadyne.palette_atlas"),
                size: wgpu::Extent3d { width: size[0], height: size[1], depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats,
            });
            (texture, None)
        }
    };

    // One uniform and one LUT buffer per palette: every draw reads its own, since writes to a
    // single buffer between draws of one submission would all land before any of them ran.
    let bind_groups: Vec<wgpu::BindGroup> = palettes
        .iter()
        .map(|p| {
            let lut_buf = make_lut_buffer(device);
            let (lut_len, lut_smooth) = write_lut(&rs.queue, &lut_buf, &p.lut, p.smooth);
            let thumb = ColorUniforms {
                lut_len,
                lut_smooth,
                ss,
                // The texture as it is, not a reprojection of a frozen frame, and nothing laid over
                // it: no spotlight, no dissolve from another palette map.
                reproject: 0,
                uv_offset: [0.0, 0.0],
                uv_scale: 1.0,
                vig_on: 0,
                out_res: [cell[0] as f32, cell[1] as f32],
                fade_weight: 0.0,
                ..cu
            };
            let uniform = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("fractadyne.palette_atlas_uniform"),
                size: std::mem::size_of::<ColorUniforms>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            rs.queue.write_buffer(&uniform, 0, bytemuck::bytes_of(&thumb));
            make_color_bg(device, &bgl, &uniform, &lut_buf, &tex_view, &aux_view)
        })
        .collect();

    let target = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("fractadyne.palette_atlas"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fractadyne.palette_atlas"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&pipeline);
        for (k, bg) in bind_groups.iter().enumerate() {
            let (col, row) = ((k as u32) % cols, (k as u32) / cols);
            let (x, y) = (col * cell[0], row * cell[1]);
            pass.set_viewport(x as f32, y as f32, cell[0] as f32, cell[1] as f32, 0.0, 1.0);
            pass.set_scissor_rect(x, y, cell[0], cell[1]);
            pass.set_bind_group(0, bg, &[]);
            pass.draw(0..3, 0..1);
        }
    }
    rs.queue.submit(Some(encoder.finish()));

    let sampled = texture.create_view(&wgpu::TextureViewDescriptor { format: Some(srgb), ..Default::default() });
    let mut renderer = rs.renderer.write();
    let id = match old_id {
        Some(id) => {
            renderer.update_egui_texture_from_wgpu_texture(device, &sampled, wgpu::FilterMode::Linear, id);
            id
        }
        None => renderer.register_native_texture(device, &sampled, wgpu::FilterMode::Linear),
    };
    Ok(PaletteAtlas { texture, id, cell, cols, count: palettes.len() })
}

/// Let egui forget an atlas; the texture goes with it.
pub fn free_palette_atlas(rs: &egui_wgpu::RenderState, atlas: PaletteAtlas) {
    rs.renderer.write().free_texture(&atlas.id);
}

/// How many times view `view_id`'s iteration texture has been written: a thumbnail drawn at one
/// count is out of date at another. `None` before the view's first frame.
pub fn view_writes(rs: &egui_wgpu::RenderState, view_id: u32) -> Option<u64> {
    let guard = rs.renderer.read();
    let v = guard.callback_resources.get::<Renderer>()?.views.get(&view_id)?;
    v.rendered.then_some(v.writes)
}
