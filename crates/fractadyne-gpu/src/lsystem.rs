//! The L-system segment pass (design/lsystems.md §5): the app walks the system for a view
//! (`fractadyne_core::lsystem::walk`) and hands the segments over in an [`LSystemFrame`]; this
//! draws them, a quad each, into the iteration texture, so the colour pass colours them as it
//! colours an escape-time picture.
//!
//! A walk takes time (it runs off the UI thread), so a frame carries the last walk's segments with
//! where its view sits in the current one: while the next walk runs, the last one is drawn moved
//! and scaled under the view, as a pan or a zoom reprojects an escape-time picture.

use egui_wgpu::wgpu;
use std::sync::Arc;

/// One segment as the shader reads it: its ends in pixels of the walked view, from its centre
/// (y up), and its colour value (≥ 0; the palette coordinate before the cycle and offset).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SegmentInstance {
    pub a: [f32; 2],
    pub b: [f32; 2],
    pub value: f32,
}

/// What the app asks of the segment pass on a frame (`MandelbrotParams::lsystem`).
#[derive(Clone, Debug)]
pub struct LSystemFrame {
    /// The last walk's segments.
    pub segments: Arc<Vec<SegmentInstance>>,
    /// Changes whenever `segments` does: the upload's key.
    pub segments_id: u64,
    /// A walked pixel `p` shows at `p * scale + offset` pixels from this view's centre (y up).
    pub scale: f32,
    pub offset: [f32; 2],
    /// The line width, pixels.
    pub width: f32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct LViewU {
    offset: [f32; 2],
    scale: f32,
    half: f32,
    size: [f32; 2],
    ss: f32,
    _pad: f32,
}

const SOURCE: &str = include_str!("lsystem.wgsl");

/// The most segments one frame draws (20 bytes each): the app's walk budget stays under it.
pub const MAX_SEGMENTS: usize = 1 << 23;

pub(crate) struct LSystemRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    group: wgpu::BindGroup,
    instances: wgpu::Buffer,
    capacity: usize,
    count: u32,
    uploaded: Option<u64>,
}

impl LSystemRenderer {
    pub(crate) fn new(device: &wgpu::Device, iter_bgl: &wgpu::BindGroupLayout) -> LSystemRenderer {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("lsystem.wgsl"),
            source: wgpu::ShaderSource::Wgsl(SOURCE.into()),
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lsystem.layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lsystem"),
            bind_group_layouts: &[iter_bgl, &bgl],
            push_constant_ranges: &[],
        });
        let target = Some(wgpu::ColorTargetState { format: crate::ITER_FORMAT, blend: None, write_mask: wgpu::ColorWrites::ALL });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("lsystem.segments"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_segment"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<SegmentInstance>() as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_segment"),
                targets: &[target.clone(), target],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        use wgpu::BufferUsages as U;
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lsystem.view"),
            size: std::mem::size_of::<LViewU>() as u64,
            usage: U::UNIFORM | U::COPY_DST,
            mapped_at_creation: false,
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("lsystem.group"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: uniform.as_entire_binding() }],
        });
        let capacity = 1024;
        let instances = Self::instance_buffer(device, capacity);
        LSystemRenderer { pipeline, uniform, group, instances, capacity, count: 0, uploaded: None }
    }

    fn instance_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lsystem.segments"),
            size: (capacity * std::mem::size_of::<SegmentInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// Upload the frame's segments (when they changed) and its view for a target of `size` texels
    /// at `ss` texels a pixel. Returns the display key: it changes whenever the pass would draw
    /// something different.
    pub(crate) fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, frame: &LSystemFrame, size: [u32; 2], ss: u32) -> u64 {
        if self.uploaded != Some(frame.segments_id) {
            let n = frame.segments.len().min(MAX_SEGMENTS);
            if n > self.capacity {
                self.capacity = n.next_power_of_two();
                self.instances = Self::instance_buffer(device, self.capacity);
            }
            if n > 0 {
                queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(&frame.segments[..n]));
            }
            self.count = n as u32;
            self.uploaded = Some(frame.segments_id);
        }
        let ss = ss.max(1) as f32;
        // At least a texel across, so a hairline still lands on texel centres.
        let half = (0.5 * frame.width * ss).max(0.5);
        let u = LViewU {
            offset: frame.offset,
            scale: frame.scale,
            half,
            size: [size[0] as f32, size[1] as f32],
            ss,
            _pad: 0.0,
        };
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&u));
        let mut h = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        (frame.segments_id, frame.scale.to_bits(), frame.offset.map(f32::to_bits), frame.width.to_bits(), size, ss.to_bits()).hash(&mut h);
        h.finish() | 1
    }

    pub(crate) fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, iter_bg: &'a wgpu::BindGroup) {
        if self.count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, iter_bg, &[]);
        pass.set_bind_group(1, &self.group, &[]);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        pass.draw(0..6, 0..self.count);
    }
}

/// What the pass clears the main target to: "interior" (no line here).
pub(crate) const CLEAR_MAIN: wgpu::Color = wgpu::Color { r: -1.0, g: 0.0, b: 0.0, a: 1.0e30 };
/// … and the aux target: no orbit statistics.
pub(crate) const CLEAR_AUX: wgpu::Color = wgpu::Color { r: 0.0, g: 0.0, b: 1.0e30, a: 0.0 };

pub mod check;

#[cfg(test)]
mod tests;
