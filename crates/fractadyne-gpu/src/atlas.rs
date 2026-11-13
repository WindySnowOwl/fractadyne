//! The palette browser's thumbnails, and the method sheet's (coloring plan B7): the view's current
//! iteration texture colored through each palette, or each coloring method, in turn, into one atlas
//! texture that egui draws from.
//!
//! ⭐**Nothing is iterated.** Coloring is its own pass over the iteration texture the view already
//! holds, so a thumbnail costs one small color pass, however deep the view: each palette is one
//! draw of `fs_color` into a cell of one texture, all in one submission. Each thumbnail uses the
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

/// What one cell of an atlas shows ([`render_atlas`]): the view's texture through a palette, with
/// the view's own coloring or another method's.
pub struct AtlasCell<'a> {
    pub palette: &'a AtlasPalette,
    /// `None`: the coloring the view was last drawn with, exactly (the palette browser's cells, and
    /// the method sheet's cell for the method in use).
    pub coloring: Option<CellColoring>,
}

/// Another coloring method for one cell, over the same iteration texture (the method sheet, B7).
/// Everything else is the view's: offset, lighting, glow, interior color. ⚠The texture must hold
/// what the method colors by (its orbit statistic, `ColorMethod::reads`), or the cell shows the
/// "no statistics" value: the caller's cell plan decides which cells can be drawn this way.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellColoring {
    pub method: fractadyne_core::coloring::ColorMethod,
    /// The palette rate for this method: the app's, which depends on the method (a bounded value
    /// takes a few sweeps over 0..1, the count a rate per iteration).
    pub cycle: f32,
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
/// of `cell`-sized thumbnails, each with the view's own coloring. `reuse` is the previous atlas:
/// redrawn in place (keeping its egui id) when the size matches, freed otherwise.
pub fn render_palette_atlas(
    rs: &egui_wgpu::RenderState,
    view_id: u32,
    palettes: &[AtlasPalette],
    cell: [u32; 2],
    cols: u32,
    reuse: Option<PaletteAtlas>,
) -> Result<PaletteAtlas, AtlasError> {
    let cells: Vec<AtlasCell> = palettes.iter().map(|p| AtlasCell { palette: p, coloring: None }).collect();
    render_atlas(rs, view_id, &cells, cell, cols, reuse)
}

/// [`render_palette_atlas`] for cells that may each color the texture another way ([`AtlasCell`]).
pub fn render_atlas(
    rs: &egui_wgpu::RenderState,
    view_id: u32,
    palettes: &[AtlasCell],
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
                // COPY_SRC: `read_cell`, the UI test's look at one cell.
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
                view_formats,
            });
            (texture, None)
        }
    };

    // One uniform and one LUT buffer per palette: every draw reads its own, since writes to a
    // single buffer between draws of one submission would all land before any of them ran.
    let bind_groups: Vec<wgpu::BindGroup> = palettes
        .iter()
        .map(|c| {
            let p = c.palette;
            let lut_buf = make_lut_buffer(device);
            let (lut_len, lut_smooth) = write_lut(&rs.queue, &lut_buf, &p.lut, p.smooth);
            let mut thumb = ColorUniforms {
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
                // `display_w` stays the panel's: a thumbnail is the view in miniature, so its
                // distance colors and glow bands sit where the panel shows them.
                ..cu
            };
            if let Some(k) = c.coloring {
                // Another method over the same texture: its own rate, and no range mapping, whose
                // window was measured for the view's method's values (`norm_mode`).
                thumb.color_method = k.method.gpu_id();
                thumb.cycle = k.cycle;
                thumb.norm_mode = 0;
                thumb.norm_lo = 0.0;
            }
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

/// Cell `k`'s texels as the atlas holds them, rows top to bottom, as display-space RGBA8 (a BGRA
/// target's bytes put in that order), read back from the GPU: for the UI test, which holds a method
/// sheet cell against the view's own coloring and against an export's `to_srgb8` bytes. Waits at
/// most five seconds; `None` if the cell does not exist, the target is not an 8-bit RGBA or BGRA
/// format, or the read fails.
pub fn read_cell(rs: &egui_wgpu::RenderState, atlas: &PaletteAtlas, k: usize) -> Option<Vec<u8>> {
    use wgpu::TextureFormat as F;
    if k >= atlas.count {
        return None;
    }
    let bgra = match atlas.texture.format() {
        F::Rgba8Unorm | F::Rgba8UnormSrgb => false,
        F::Bgra8Unorm | F::Bgra8UnormSrgb => true,
        _ => return None,
    };
    let (device, queue) = (&rs.device, &rs.queue);
    let [w, h] = atlas.cell;
    let (col, row) = ((k as u32) % atlas.cols, (k as u32) / atlas.cols);
    let texel = atlas.texture.format().block_copy_size(None)?;
    let unpadded = w * texel;
    let padded = unpadded.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("fractadyne.atlas_cell_read"),
        size: (padded * h) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("fractadyne.atlas_cell_read") });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &atlas.texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x: col * w, y: row * h, z: 0 },
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: Some(h) },
        },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    queue.submit(std::iter::once(enc.finish()));
    let slice = buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    crate::export::await_readback(device, &rx, None, Some(deadline)).into_result().ok()?;
    let data = slice.get_mapped_range();
    let mut out = Vec::with_capacity((unpadded * h) as usize);
    for r in 0..h as usize {
        let src = r * padded as usize;
        out.extend_from_slice(&data[src..src + unpadded as usize]);
    }
    drop(data);
    buf.unmap();
    if bgra {
        for px in out.chunks_exact_mut(4) {
            px.swap(0, 2);
        }
    }
    Some(out)
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
