//! Does this device's shader compiler FUSE `two_prod`'s product into the add that consumes it?
//!
//! With the add transforms' residuals dropped (WGSL `EFT_ADD_RESIDUALS`), `df_mul`'s last step is
//! `p.x + e` with `p.x = a.x * b.x`. A compiler free to contract may turn that into
//! `fma(a.x, b.x, e)`, which counts the product's own error twice, and may do so in some shaders and
//! not others: the RX 6800 XT's Vulkan/GL compiler did (TODO PR-C2: 75 of 256 probe inputs, seven
//! self-test checks that need two code paths to agree failed), NVIDIA's does not where it matters
//! for the goldens. So each device is asked once, when the app creates it: one pass of
//! `fs_fusion_probe` computes `df_mul` on 256 hashed pairs with no barrier, and any result that
//! differs from the code as written compiles the WGSL `PRODUCT_BARRIER` into that device's iterate
//! pipelines ([`crate::raster_pipeline_c`]). A device never asked (a harness's own) gets no barrier:
//! the code as it was before the probe existed.

use crate::{GpuError, ITER_FORMAT};
use egui_wgpu::wgpu;

/// Inputs per probe pass.
const N: u32 = 256;

/// Verdicts by device, decided once each ([`detect_product_fusion`]). A handful of devices live per
/// process, so a list is enough.
static VERDICTS: std::sync::Mutex<Vec<(u64, bool)>> = std::sync::Mutex::new(Vec::new());

fn device_key(device: &wgpu::Device) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    device.hash(&mut h);
    h.finish()
}

/// Whether `device`'s iterate pipelines carry the product barrier: what [`detect_product_fusion`]
/// found for it, or `false` if it was never asked.
pub fn product_barrier(device: &wgpu::Device) -> bool {
    let key = device_key(device);
    VERDICTS.lock().map(|v| v.iter().any(|&(k, on)| k == key && on)).unwrap_or(false)
}

/// What the probe found on one device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FusionVerdict {
    /// Inputs whose result differed from the code as written.
    pub differ: u32,
    /// Inputs compared.
    pub of: u32,
}

impl FusionVerdict {
    /// The barrier goes in when any input differs.
    pub fn barrier(self) -> bool {
        self.differ > 0
    }
}

/// Ask `device` once and record the answer for [`product_barrier`]. Pipelines built on the device
/// before this call carry no barrier, so the app calls it as soon as a device exists.
pub fn detect_product_fusion(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<FusionVerdict, GpuError> {
    let got = run_probe(device, queue)?;
    let differ = (0..N).filter(|&ix| got[ix as usize].to_bits() != as_written(ix).to_bits()).count() as u32;
    let verdict = FusionVerdict { differ, of: N };
    let key = device_key(device);
    if let Ok(mut v) = VERDICTS.lock() {
        v.retain(|&(k, _)| k != key);
        v.push((key, verdict.barrier()));
    }
    Ok(verdict)
}

/// One pass of `fs_fusion_probe` over N texels: each texel's `df_mul` hi limb.
fn run_probe(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<Vec<f32>, GpuError> {
    let shader = crate::shader_module(device);
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("fusion.layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });
    // The renderer's arithmetic (add residuals dropped) without the barrier: the question is what
    // the compiler does when nothing stops it.
    let consts = std::collections::HashMap::from([
        ("EFT_ADD_RESIDUALS".to_string(), 0.0),
        ("PRODUCT_BARRIER".to_string(), 0.0),
    ]);
    let pipeline =
        crate::fullscreen_pipeline_c(device, &shader, &layout, "fs_fusion_probe", &[ITER_FORMAT], "fusion.probe", &consts);
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("fusion.out"),
        size: wgpu::Extent3d { width: N, height: 1, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: ITER_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let bpr = (N * 16).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let out = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("fusion.readback"),
        size: u64::from(bpr),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("fusion.enc") });
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fusion.pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&pipeline);
        pass.draw(0..3, 0..1);
    }
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: &tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo {
            buffer: &out,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(bpr), rows_per_image: Some(1) },
        },
        wgpu::Extent3d { width: N, height: 1, depth_or_array_layers: 1 },
    );
    queue.submit(std::iter::once(enc.finish()));
    let slice = out.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    crate::await_readback(device, &rx, None, None).into_result()?;
    let data = slice.get_mapped_range();
    let row: &[f32] = bytemuck::cast_slice(&data[..(N * 16) as usize]);
    let hi: Vec<f32> = row.chunks(4).map(|t| t[0]).collect();
    drop(data);
    out.unmap();
    Ok(hi)
}

// ---- The CPU side: the probe's inputs and `df_mul` as written, bit for bit -------------------

/// Mirrors WGSL `gt_hash`.
fn gt_hash(x0: u32) -> u32 {
    let mut x = x0.wrapping_mul(747796405).wrapping_add(2891336453);
    x = ((x >> ((x >> 28) + 4)) ^ x).wrapping_mul(277803737);
    (x >> 22) ^ x
}

/// Mirrors WGSL `gt_f32`.
fn gt_f32(seed: u32, emin: i32, espan: u32) -> f32 {
    let mant = gt_hash(seed) & 0x007F_FFFF;
    let eb = (127 + emin + (gt_hash(seed ^ 0x9E37_79B9) % espan) as i32) as u32;
    let sign = (gt_hash(seed ^ 0x85EB_CA6B) & 1) << 31;
    f32::from_bits(sign | (eb << 23) | mant)
}

/// Mirrors WGSL `gt_df`.
fn gt_df(seed: u32, emin: i32, espan: u32) -> (f32, f32) {
    let hi = gt_f32(seed, emin, espan);
    let he = ((hi.to_bits() >> 23) & 0xFF) as i32 - 127;
    (hi, gt_f32(seed ^ 0xDEAD_BEEF, he - 25, 1))
}

/// `df_mul`'s hi limb for probe input `ix`, evaluated as the WGSL is written with the add residuals
/// dropped: `p + (pe + (a.x*b.y + a.y*b.x))`, every operation rounded on its own (Rust neither
/// reassociates nor contracts).
pub(crate) fn as_written(ix: u32) -> f32 {
    let s = ix + 100003 * 3;
    let (a, b) = (gt_df(s * 8 + 3, -4, 8), gt_df(s * 8 + 5, -4, 8));
    let p = a.0 * b.0;
    let pe = a.0.mul_add(b.0, -p);
    p + (pe + (a.0 * b.1 + a.1 * b.0))
}

#[cfg(test)]
#[path = "../tests/unit/fusion/fusion_tests.rs"]
mod fusion_tests;
