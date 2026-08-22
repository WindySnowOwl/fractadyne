//! Offscreen high-resolution export: render a whole frame (perturbation, all modes) to a
//! texture and read it back to the CPU. `render_export` (colored image), `render_iter` (raw
//! iteration data as EXR), and `color_iter_buffer` (color a CPU-side iteration buffer — the
//! glitch-correction path). Uses the shared pipeline/uniform scaffolding from the crate root.

use crate::{
    color_bind_group_layout, fullscreen_pipeline, iter_bind_group_layout, make_color_bg,
    make_iter_bg, make_iter_texture, make_orbit_buffer, method_needs_aux, shader_module,
    ColorUniforms, IterUniforms, RefOffset, Vignette, ITER_FORMAT,
};
use egui_wgpu::wgpu;
use std::sync::Arc;

const EXPORT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;

/// Failure modes of the offscreen GPU render / readback path. `Canceled`'s `Display` is exactly
/// `"canceled"` (load-bearing: callers may still string-compare it) and `Readback` folds the mpsc
/// / `map_async` readback failures — neither of which any caller distinguishes further.
#[derive(Debug, thiserror::Error)]
pub enum GpuError {
    /// The caller's cancel flag was set mid-render (a tiled export aborts between tiles).
    #[error("canceled")]
    Canceled,
    /// A GPU buffer readback failed — the mpsc channel dropped or `map_async` reported an error.
    #[error("GPU readback failed: {0}")]
    Readback(String),
    /// The reference orbit + BLA (one storage-buffer binding) exceed this GPU's binding-size limit.
    /// Guards a pathological export (an interior reference at a very high iteration count) from
    /// panicking in `create_bind_group`; the caller shows this and the render is skipped.
    #[error("reference orbit too large for this GPU: {bytes} B exceeds the {limit} B storage-buffer binding limit (reduce iterations)")]
    OrbitTooLarge { bytes: u64, limit: u64 },
}

/// Guard the reference-orbit storage binding against the GPU's max binding size. The orbit and its
/// BLA tree share one storage buffer — `(orbit_len + bla_len) * 16` bytes — and an interior reference
/// at a very high `max_iter` can exceed `max_storage_buffer_binding_size` (128 MB default ⇒ ~932k
/// orbit samples once the BLA is included), which panics in `create_bind_group`. Returning an error
/// lets the export fail cleanly instead of crashing the app.
fn check_orbit_binding(device: &wgpu::Device, orbit_len: usize, bla_len: usize) -> Result<(), GpuError> {
    let bytes = (orbit_len + bla_len).max(1) as u64 * 16;
    let limit = device.limits().max_storage_buffer_binding_size as u64;
    if bytes > limit {
        return Err(GpuError::OrbitTooLarge { bytes, limit });
    }
    Ok(())
}

/// Everything needed to render one frame offscreen at an arbitrary resolution.
/// Mirrors the live view's parameters (the app fills it from the current view).
#[derive(Clone)]
pub struct ExportRequest {
    pub width: u32,
    pub height: u32,
    pub ss: u32,
    /// Complex span *mantissa* (`span · 2^-delta_exp`, O(1)) — see [`fractadyne_core::GpuScale`].
    pub span_mantissa: fractadyne_core::SpanMantissa,
    pub center: [f32; 4],
    pub ref_offset: RefOffset,
    pub delta_exp: i32,
    /// Series-approximation skip (0 = none) + order-3 coeffs (complex df32 mantissa × 2^exp).
    pub sa_skip: u32,
    /// 1 = flag Pauldelbrot-glitched pixels with a `-2` sentinel in the iteration texture's `r`
    /// channel (multi-reference correction passes read this back). 0 for normal rendering.
    pub glitch_on: u32,
    /// Spotlight vignette (guided-tour exports); `on == 0` disables it.
    pub vignette: Vignette,
    pub sa_a: [f32; 4],
    pub sa_a_exp: i32,
    pub sa_b: [f32; 4],
    pub sa_b_exp: i32,
    pub sa_c: [f32; 4],
    pub sa_c_exp: i32,
    pub julia_c: [f32; 4],
    pub orbit: Arc<Vec<[f32; 4]>>,
    pub orbit_len: u32,
    /// BLA tree (flattened) appended after the orbit; `bla_on = 1` enables the traversal.
    pub bla: Arc<Vec<[f32; 4]>>,
    pub bla_on: u32,
    pub max_iter: u32,
    pub mode: u32,
    pub formula: u32,
    pub julia: u32,
    pub cycle: f32,
    pub offset: f32,
    /// Palette-range mapping: 0 = linear (`cycle`/`offset` alone), 1 = log about `norm_lo`.
    pub norm_mode: u32,
    pub norm_lo: f32,
    pub stop_count: u32,
    pub stops: [[f32; 4]; 8],
    pub light: u32,
    pub light_angle: f32,
    pub light_height: f32,
    pub de_on: u32,
    pub de_strength: f32,
    pub de_width: f32,
    pub de_phase: f32,
    pub color_method: u32,
    pub stripe_freq: f32,
    pub trap_type: u32,
    pub aa_filter: u32,
    pub interior_col: [f32; 4],
    /// Per-tile nominal-work cap (`tile²·ss²·max_iter`) overriding the default `TILE_WORK_BUDGET`.
    /// `None` keeps the export default; a smaller value forces smaller tiles so no single GPU
    /// submission runs long enough to trip the OS watchdog. The offline tour path sets this
    /// (a shallow keyframe asking millions of iterations would otherwise issue one multi-second
    /// dispatch and lose the device — the live path bounds this via `fe_budget`, the tour didn't).
    pub work_budget: Option<u64>,
}

/// The actual `(width, height)` an export produced after clamping to the GPU's
/// max texture dimension, plus the effective supersampling used.
pub struct ExportResult {
    pub width: u32,
    pub height: u32,
    pub ss: u32,
    /// Linear RGBA, row-major, `width*height*4` floats.
    pub pixels: Vec<f32>,
    /// Pure-GPU iterate-pass time summed across tiles (ms; 0.0 when TIMESTAMP_QUERY is
    /// unavailable). Unconditional since D3.1 — the export throughput metric.
    pub iterate_ms: f64,
    /// Pure-GPU color-pass time summed across tiles (ms; 0.0 when unavailable).
    pub color_ms: f64,
    /// Shader event counters for the whole render (D3.3): indices `CTR_*` in the crate
    /// root — rebases, extended-sample decodes, glitch flags, BLA skips, max-iter
    /// exhaustions. All zero when the readback failed. **u64**: the GPU-side slots are u32
    /// atomics, but a deep multi-tile export accumulates billions of events — `render_export`
    /// zeroes + reads them PER TILE and sums into these u64s so the whole-render total does
    /// not wrap (a single tile stays well under u32 by the tile work budget).
    pub counters: [u64; crate::COUNTER_SLOTS],
    /// Longest single GPU submission this render made, wall-clock ms (chunked paths measure per
    /// iteration window; unchunked tiled paths report the whole tile incl. readback as an upper
    /// bound; 0.0 = not measured on this path). The TDR-forensics figure: a render that stayed
    /// bounded shows a few hundred ms here no matter how long it ran in total.
    pub max_dispatch_ms: f64,
}

/// Render `req` offscreen and read the colored image back to the CPU. Synchronous
/// (blocks until the GPU finishes). Single-texture for now, so the result is clamped
/// to the device's max 2D texture dimension (e.g. 8192); supersampling is reduced if
/// `resolution × ss` would exceed it.
/// Wall-priced tile cap for the NEXT export tile (design/mode2-chunking.md §12, offline
/// flavour). The static tile bound prices work NOMINALLY (`tile²·ss²·iter`), and nominal is not
/// real: at a wrap-storm view (BLA skips nothing, rebases every few hundred iterations) a
/// nominally-budgeted tile runs seconds — crash-1787194989 was 64²-px thumbnail tiles at
/// 6.55e10 nominal racing the live walk. Every tile's WALL time is already observable (the loop
/// polls Wait per tile), so the next tile halves when this one ran hot and doubles back when
/// clearly cheap — bounded oscillation-free by the [16, static-bound] clamp. The FIRST tile of a
/// render is still nominal-priced (no observation yet): that single opening overshoot is the
/// recorded residual, and it is one tile, not a sustained sequence.
pub(crate) fn export_tile_cap(cap: u32, wall_ms: f64, ceiling: u32) -> u32 {
    /// Past this, halve: comfortably under the ~900 ms band even if the NEXT (halved) tile is
    /// as mispriced as this one was.
    const HOT_MS: f64 = 500.0;
    /// Under this, double: a clearly-cheap region should not pay 4× the tile count forever.
    const CHEAP_MS: f64 = 100.0;
    let next = if !wall_ms.is_finite() || wall_ms < 0.0 {
        cap
    } else if wall_ms > HOT_MS {
        cap / 2
    } else if wall_ms < CHEAP_MS {
        cap.saturating_mul(2)
    } else {
        cap
    };
    next.clamp(16, ceiling.max(16))
}

/// One-time gate for the export tile trace: `FRACTADYNE_TRACE` containing `tile` prints one
/// stderr line per export tile (origin, size, wall, cap) — the offline counterpart of the app's
/// live `tile` trace category, so a tile-cost curve can be captured without a debugger.
pub(crate) fn tile_trace_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| {
        std::env::var("FRACTADYNE_TRACE")
            .map(|v| v.split(',').any(|c| c.trim() == "tile"))
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tile_cap {
    use super::export_tile_cap;

    #[test]
    fn a_hot_tile_halves_the_next_and_a_cheap_one_doubles_it() {
        assert_eq!(export_tile_cap(64, 800.0, 2048), 32);
        assert_eq!(export_tile_cap(64, 50.0, 2048), 128);
        assert_eq!(export_tile_cap(64, 250.0, 2048), 64, "the band between holds");
    }

    #[test]
    fn the_floor_and_the_static_ceiling_both_hold() {
        assert_eq!(export_tile_cap(16, 5000.0, 2048), 16, "never below 16");
        assert_eq!(export_tile_cap(70, 10.0, 70), 70, "never above the nominal bound");
        assert_eq!(export_tile_cap(2048, 10.0, 2048), 2048);
    }

    #[test]
    fn garbage_walls_change_nothing()  {
        assert_eq!(export_tile_cap(64, f64::NAN, 2048), 64);
        assert_eq!(export_tile_cap(64, -1.0, 2048), 64);
    }
}

/// Iteration-window pricing for the per-tile chunked iterate — the actuator the area cap cannot
/// be (crash-1787292746): a dwell-bound tile is LATENCY-bound, so its wall is
/// `max_dwell / serial_chain_rate` regardless of tile area (measured: a 20x20 tile cost the same
/// 0.5 s as the 50x50 beside it). Only splitting the ITERATION range bounds the dispatch. Windows
/// are wall-priced like the area cap (halve hot, double cheap), and every tile OPENS at a window
/// bounded by the worst serial cost observed so far this render (`worst_ms_per_iter`, a
/// high-water mark seeded from a pessimistic 1M iters/s serial floor) — so the first tile of a
/// dwell cliff can overshoot the target band at most by the ratio of the true serial rate to the
/// worst observed one, instead of arbitrarily (the "first tile is nominal-priced" hole, closed).
const CHUNK_HOT_MS: f64 = 400.0;
/// Under this, the next window doubles — a settled-cheap tile sweeps its range in a few passes.
const CHUNK_CHEAP_MS: f64 = 100.0;
/// Opening-window serial-rate floor, iters/ms (1M iters/s). Pessimistic on purpose: on hardware
/// where the true serial chain rate is lower, the FIRST hot chunk halves the window and raises
/// the high-water mark, so subsequent openings tighten; the exposure is one bounded overshoot.
const CHUNK_SERIAL_FLOOR_IPMS: f64 = 1000.0;
/// Window floor: keeps the pass count bounded (a 4M ask is at most ~244 passes) — per-pass fixed
/// overhead is ~1 ms, so the floor bounds chunking overhead well under the serial work it prices.
const CHUNK_MIN_ITERS: u32 = 16_384;

/// Cross-tile pricing state for the chunked iterate. One per render; both tiled loops carry it.
pub(crate) struct ChunkPricer {
    /// High-water mark of observed `wall_ms / window_iters` — an upper bound on the serial cost
    /// per iteration. Only ever rises: a cheap (BLA-skipping, parallel-bound) chunk must never
    /// re-widen the openings that protect the next dwell-bound tile.
    worst_ms_per_iter: f64,
}

impl ChunkPricer {
    pub(crate) fn new() -> Self {
        Self { worst_ms_per_iter: 1.0 / CHUNK_SERIAL_FLOOR_IPMS }
    }
    /// Opening window for a tile: the largest window that stays under `CHUNK_HOT_MS` even if some
    /// pixel runs it fully serial at the worst rate seen so far.
    pub(crate) fn open(&self, max_iter: u32) -> u32 {
        let w = (CHUNK_HOT_MS / self.worst_ms_per_iter) as u32;
        w.max(CHUNK_MIN_ITERS).min(max_iter.max(1))
    }
    /// Record a chunk's measured wall. Garbage walls (NaN/negative) are ignored.
    pub(crate) fn observe(&mut self, window: u32, wall_ms: f64) {
        if window > 0 && wall_ms.is_finite() && wall_ms > 0.0 {
            self.worst_ms_per_iter = self.worst_ms_per_iter.max(wall_ms / window as f64);
        }
    }
    /// Next window within the same tile: halve hot, double cheap, hold the band between. Doubling
    /// past the opening bound is allowed on purpose — within one tile the pixels are the same, so
    /// a cheap chunk is direct evidence the survivors are skipping, not grinding.
    pub(crate) fn next(&self, window: u32, wall_ms: f64, max_iter: u32) -> u32 {
        let next = if !wall_ms.is_finite() || wall_ms < 0.0 {
            window
        } else if wall_ms > CHUNK_HOT_MS {
            window / 2
        } else if wall_ms < CHUNK_CHEAP_MS {
            window.saturating_mul(2)
        } else {
            window
        };
        next.max(CHUNK_MIN_ITERS.min(max_iter.max(1))).min(max_iter.max(1))
    }
}

#[cfg(test)]
mod chunk_pricer {
    use super::*;

    #[test]
    fn openings_are_bounded_by_the_serial_floor_and_tighten_on_worse_evidence() {
        let mut p = ChunkPricer::new();
        assert_eq!(p.open(4_000_000), 400_000, "1M it/s floor x 400 ms target");
        p.observe(400_000, 800.0); // twice as slow as assumed
        assert_eq!(p.open(4_000_000), 200_000);
        p.observe(400_000, 8.0); // a cheap chunk must never re-widen the opening
        assert_eq!(p.open(4_000_000), 200_000);
        assert_eq!(p.open(50_000), 50_000, "never past the ask");
    }

    #[test]
    fn windows_halve_hot_double_cheap_and_hold_the_band() {
        let p = ChunkPricer::new();
        assert_eq!(p.next(400_000, 800.0, 4_000_000), 200_000);
        assert_eq!(p.next(400_000, 20.0, 4_000_000), 800_000);
        assert_eq!(p.next(400_000, 250.0, 4_000_000), 400_000);
        assert_eq!(p.next(20_000, 5000.0, 4_000_000), CHUNK_MIN_ITERS, "floor holds");
        assert_eq!(p.next(3_000_000, 20.0, 4_000_000), 4_000_000, "ask caps growth");
        assert_eq!(p.next(400_000, f64::NAN, 4_000_000), 400_000);
    }
}

/// Per-render plumbing for the chunked per-tile iterate: the resumable chunk pipeline, the
/// state->G-buffer resolve pipeline, and one max-tile-sized pair of ping-pong state texture sets
/// shared by every tile (chunk 0 of each tile initializes from scratch, so no cross-tile state
/// survives; passes are scissored to the tile's sample rect so out-of-tile texels neither burn
/// iterations nor pollute the per-tile event counters).
struct TileChunker {
    chunk_pipeline: wgpu::RenderPipeline,
    resolve_pipeline: wgpu::RenderPipeline,
    state: [Vec<wgpu::TextureView>; 2],
    state_bg: [wgpu::BindGroup; 2],
}

impl TileChunker {
    /// `max_size` is the largest sample grid any tile can ask for (static tile bound x ss).
    fn new(
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        iter_bgl: &wgpu::BindGroupLayout,
        fe: bool,
        max_size: [u32; 2],
    ) -> Self {
        let targets: usize = if fe { 4 } else { 3 };
        let state_bgl = crate::state_bind_group_layout_n(device, targets as u32);
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("export.tilechunk_layout"),
            bind_group_layouts: &[iter_bgl, &state_bgl],
            push_constant_ranges: &[],
        });
        let chunk_formats = [ITER_FORMAT; 4];
        let chunk_pipeline = fullscreen_pipeline(
            device,
            shader,
            &layout,
            if fe { "fs_iterate_chunk_fe" } else { "fs_iterate_chunk" },
            &chunk_formats[..targets],
            "export.tilechunk_pipeline",
        );
        let resolve_pipeline = fullscreen_pipeline(
            device, shader, &layout, "fs_resolve", &[ITER_FORMAT, ITER_FORMAT],
            "export.tilechunk_resolve",
        );
        let state = [
            crate::make_state_textures(device, max_size, targets),
            crate::make_state_textures(device, max_size, targets),
        ];
        let state_bg = [
            crate::make_state_bg(device, &state_bgl, &state[0]),
            crate::make_state_bg(device, &state_bgl, &state[1]),
        ];
        Self { chunk_pipeline, resolve_pipeline, state, state_bg }
    }

    /// Run one tile's iterate as bounded chunk passes over `[0, max_iter)`, one submission each
    /// (polled to completion, so every dispatch the watchdog sees is one priced window). The
    /// caller's `iu` must already describe the tile (res/px_offset/step); its iteration range is
    /// overwritten per pass and left at the final range, exactly as `render_iter_chunked` leaves
    /// it for the resolve. Returns `(passes, read_set, max_chunk_ms)`; the caller resolves from
    /// `state_bg[read_set]`. `wall_sum_ms` collects the summed chunk walls (the iterate-time
    /// figure for chunked tiles — GPU timestamps would cost a readback per pass for <1% accuracy).
    #[allow(clippy::too_many_arguments)]
    fn run_tile(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        iter_bg: &wgpu::BindGroup,
        counters_buf: &wgpu::Buffer,
        iu: &mut IterUniforms,
        iter_uniform: &wgpu::Buffer,
        grid: [u32; 2],
        max_iter: u32,
        pricer: &mut ChunkPricer,
        cancel: Option<&std::sync::atomic::AtomicBool>,
        deadline: Option<std::time::Instant>,
        wall_sum_ms: &mut f64,
    ) -> Result<(u32, usize, f64), GpuError> {
        let mut window = pricer.open(max_iter);
        let mut read_set = 0usize;
        let mut passes = 0u32;
        let mut max_chunk_ms = 0.0f64;
        let mut s = 0u32;
        while s < max_iter {
            if cancel.is_some_and(|c| c.load(std::sync::atomic::Ordering::Relaxed)) {
                return Err(GpuError::Canceled);
            }
            if passes > 0 && deadline.is_some_and(|d| std::time::Instant::now() >= d) {
                // Between chunks, like the tile loop between tiles; never before the first pass
                // (a tile that starts must settle, or the caller would merge an unresolved tile).
                return Err(GpuError::Canceled);
            }
            let e = s.saturating_add(window).min(max_iter);
            iu.start_iter = s;
            iu.end_iter = e;
            queue.write_buffer(iter_uniform, 0, bytemuck::bytes_of(iu));
            let write_set = 1 - read_set;
            let t = std::time::Instant::now();
            let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("export.tilechunk_enc"),
            });
            if passes == 0 {
                // The tile's counter epoch starts with its first chunk; later passes accumulate.
                enc.clear_buffer(counters_buf, 0, None);
            }
            {
                let attach = |v| Some(wgpu::RenderPassColorAttachment {
                    view: v,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                });
                let attachments: Vec<_> = self.state[write_set].iter().map(|v| attach(v)).collect();
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("export.tilechunk_pass"),
                    color_attachments: &attachments,
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&self.chunk_pipeline);
                pass.set_bind_group(0, iter_bg, &[]);
                pass.set_bind_group(1, &self.state_bg[read_set], &[]);
                pass.set_scissor_rect(0, 0, grid[0], grid[1]);
                pass.draw(0..3, 0..1);
            }
            queue.submit(std::iter::once(enc.finish()));
            let _ = device.poll(wgpu::Maintain::Wait);
            let wall = t.elapsed().as_secs_f64() * 1000.0;
            if tile_trace_on() {
                eprintln!("[fd-export] chunk [{s},{e}) wall={wall:.1}ms");
            }
            *wall_sum_ms += wall;
            max_chunk_ms = max_chunk_ms.max(wall);
            pricer.observe(e - s, wall);
            window = pricer.next(window, wall, max_iter);
            read_set = write_set;
            passes += 1;
            s = e;
        }
        Ok((passes, read_set, max_chunk_ms))
    }
}

pub fn render_export(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    req: &ExportRequest,
    progress: &std::sync::atomic::AtomicU32,
    cancel: &std::sync::atomic::AtomicBool,
) -> Result<ExportResult, GpuError> {
    render_export_impl(device, queue, req, progress, cancel, true)
}

/// [`render_export`] with the chunked per-tile iterate disabled — the single-dispatch control
/// for the selftest's bit-identity gate. Not for production use: this is exactly the unbounded
/// dispatch shape that lost the device (crash-1787292746).
#[doc(hidden)]
pub fn render_export_unchunked(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    req: &ExportRequest,
    progress: &std::sync::atomic::AtomicU32,
    cancel: &std::sync::atomic::AtomicBool,
) -> Result<ExportResult, GpuError> {
    render_export_impl(device, queue, req, progress, cancel, false)
}

fn render_export_impl(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    req: &ExportRequest,
    progress: &std::sync::atomic::AtomicU32,
    cancel: &std::sync::atomic::AtomicBool,
    allow_chunking: bool,
) -> Result<ExportResult, GpuError> {
    use std::sync::atomic::Ordering::Relaxed;
    let max_dim = device.limits().max_texture_dimension_2d;
    let max_buf = device.limits().max_buffer_size;
    let w = req.width.max(1);
    let h = req.height.max(1);
    let ss = req.ss.max(1);
    let full_iw = w as f32 * ss as f32; // full iteration resolution (across all tiles)
    let full_ih = h as f32 * ss as f32;
    // Tile size (output px): keep tile×ss within the texture cap and the per-tile
    // readback buffer (tile²·16 B) within the buffer-size limit. Tiling lets exports
    // exceed the single-texture/buffer limits without crashing. Also bound each tile by
    // *iteration work* (tile²·ss²·iter) so a single GPU submission can't run long enough
    // to trip the OS watchdog (TDR ≈ 2 s → device-lost) or monopolize the shared device
    // and freeze the live UI — the render stays responsive across many short tiles.
    const TILE_WORK_BUDGET: u64 = 20_000_000_000;
    // A caller may cap per-tile work below the default (the tour path does). A smaller budget also
    // lowers the tile-size floor so the cap is actually honoured — at extreme `max_iter` a 64²
    // floor tile could still exceed a small budget, so drop the floor to 16² when one is set.
    let budget = req.work_budget.unwrap_or(TILE_WORK_BUDGET);
    let by_tex = (max_dim / ss).max(1);
    let by_buf = (((max_buf / 16) as f64).sqrt() as u32).max(256);
    let work_per_px = (ss as u64 * ss as u64) * (req.max_iter.max(1) as u64);
    // ⚠The 64² efficiency floor may NEVER override the work budget upward — it did, and at an
    // extreme ask the floor tile was 3.3× the budget: a bookmark thumbnail (which had inherited
    // the export dialog's ss=2 on top of an explicit 4,000,000) dispatched 64²-px tiles of
    // 6.55e10 nominal each at a view where nominal ≈ real, racing the live session's settle walk
    // on the same queue — device lost (crash-1787194989, beta.109). The floor drops to 16²
    // whenever the budget asks for less than 64²; readback-overhead efficiency is a luxury the
    // watchdog budget outranks. (Nominal pricing itself is still the residual here: a wall-
    // adaptive tile loop — the §12 design, offline flavour — is the recorded follow-up.)
    let by_work_raw = ((budget / work_per_px.max(1)) as f64).sqrt() as u32;
    let by_work = by_work_raw.max(if by_work_raw < 64 { 16 } else { 64 });
    let tile = by_tex.min(by_buf).min(by_work).clamp(1, 2048);

    let shader = shader_module(device);
    let iter_bgl = iter_bind_group_layout(device);
    let color_bgl = color_bind_group_layout(device);
    let iter_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("export.iter_layout"),
        bind_group_layouts: &[&iter_bgl],
        push_constant_ranges: &[],
    });
    let color_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("export.color_layout"),
        bind_group_layouts: &[&color_bgl],
        push_constant_ranges: &[],
    });
    let iter_pipeline = fullscreen_pipeline(
        device, &shader, &iter_layout, "fs_iterate", &[ITER_FORMAT, ITER_FORMAT],
        "export.iter_pipeline",
    );
    let color_pipeline = fullscreen_pipeline(
        device, &shader, &color_layout, "fs_color", &[EXPORT_FORMAT], "export.color_pipeline",
    );

    // Chunked per-tile iterate (crash-1787292746, the export-TDR fix): when the request fits the
    // resumable chunk shaders' scope, every tile's iterate runs as wall-priced iteration windows
    // (see `ChunkPricer`) instead of one unbounded dispatch — a dwell-bound tile is LATENCY-bound
    // (wall = max dwell / serial chain rate, independent of area), so the area cap below cannot
    // price it. Out of scope (aux coloring, non-holomorphic formulas, or a device without the
    // state-attachment width) keeps the single-dispatch path unchanged. Glitch detection IS in
    // scope since beta.124 (`ST_GLITCHED`).
    let fe = req.mode == 2;
    let chunk_scope = allow_chunking
        && (req.mode == 1 || req.mode == 0 || req.mode == 2)
        && req.formula <= 3
        && !method_needs_aux(req.color_method)
        && device.limits().max_color_attachment_bytes_per_sample >= if fe { 64 } else { 48 };
    // ⚠Built LAZILY, on the first tile that would actually be SPLIT. Two render pipelines over a
    // 121 KB shader plus the state textures is real setup cost, and the corrector calls into
    // these loops once per reference pass — eagerly building it there cost ~7 s on a 60,000-
    // iteration frame that then ran one window per tile anyway (measured: bench scene 04 went
    // 14.3 s → 21.7 s). When the whole ask fits one priced window, chunking is a no-op by
    // construction, so paying for it buys nothing.
    let mut chunker: Option<TileChunker> = None;
    let mut pricer = ChunkPricer::new();
    let mut max_dispatch_ms = 0.0f64;

    let uniform = |label, size| {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    };
    let iter_uniform = uniform("export.iter_uniform", std::mem::size_of::<IterUniforms>() as u64);
    let color_uniform = uniform("export.color_uniform", std::mem::size_of::<ColorUniforms>() as u64);

    check_orbit_binding(device, req.orbit.len(), req.bla.len())?;
    let orbit_cap = (req.orbit.len() + req.bla.len()).max(1) as u32;
    let orbit_buf = make_orbit_buffer(device, orbit_cap);
    if !req.orbit.is_empty() {
        queue.write_buffer(&orbit_buf, 0, bytemuck::cast_slice(req.orbit.as_slice()));
    }
    if !req.bla.is_empty() {
        let off = (req.orbit.len() * 16) as u64;
        queue.write_buffer(&orbit_buf, off, bytemuck::cast_slice(req.bla.as_slice()));
    }
    let counters_buf = crate::make_counters_buf(device);
    let counters_read = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("export.counters_read"),
        size: (crate::COUNTER_SLOTS * 4) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let iter_bg = make_iter_bg(device, &iter_bgl, &iter_uniform, &orbit_buf, &counters_buf);

    // Coloring uniform is constant across tiles; step is the full-resolution step.
    let cu = ColorUniforms {
        stop_count: req.stop_count,
        cycle: req.cycle,
        offset: req.offset,
        ss,
        light: req.light,
        light_angle: req.light_angle,
        light_height: req.light_height,
        de_on: req.de_on,
        de_strength: req.de_strength,
        de_width: req.de_width,
        de_phase: req.de_phase,
        color_method: req.color_method,
        aa_filter: req.aa_filter.max(1),
        reproject: 0,
        uv_offset: [0.0, 0.0],
        uv_scale: 1.0,
        vig_on: req.vignette.on,
        vig_dim: req.vignette.dim,
        vig_soft: req.vignette.soft,
        vig_center: req.vignette.center,
        vig_radius: req.vignette.radius,
        _pad_vig: 0.0,
        interior_col: req.interior_col,
        stops: req.stops,
        out_res: [w as f32, h as f32],
        norm_mode: req.norm_mode,
        norm_lo: req.norm_lo,
    };
    queue.write_buffer(&color_uniform, 0, bytemuck::bytes_of(&cu));
    let split = |v: f64| -> (f32, f32) {
        let hi = v as f32;
        (hi, (v - hi as f64) as f32)
    };
    // Step mantissa = span_mantissa (already × 2^-delta_exp) / texdim — O(1), no overflow.
    let (sxh, sxl) = split(req.span_mantissa.x / (w as f64 * ss as f64));
    let (syh, syl) = split(req.span_mantissa.y / (h as f64 * ss as f64));

    let bpp = 16u32; // Rgba32Float
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let mut pixels = vec![0.0_f32; (w as usize) * (h as usize) * 4];

    // Wall-adaptive cap (see `export_tile_cap`): starts at the nominal bound, then prices every
    // subsequent tile from the last one's measured wall time. Progress is by PIXELS because the
    // tile count is no longer fixed.
    let mut cap = tile;
    let total_px = (w as u64).saturating_mul(h as u64).max(1);
    let mut done_px = 0u64;
    let (mut sum_iterate_ms, mut sum_color_ms) = (0.0f64, 0.0f64);
    // Event counters summed across tiles in u64 (each tile is zeroed + read below, so the
    // per-tile u32 never wraps, and the whole-render total can exceed 2^32).
    let mut ctr_sum = [0u64; crate::COUNTER_SLOTS];

    let mut ty0 = 0u32;
    while ty0 < h {
        let th = cap.min(h - ty0);
        let mut tx0 = 0u32;
        while tx0 < w {
            if cancel.load(Relaxed) {
                return Err(GpuError::Canceled);
            }
            let t_tile = std::time::Instant::now();
            let tw = cap.min(w - tx0).min(th.max(16));
            let iw = tw * ss;
            let ih = th * ss;

            let mut iu = IterUniforms {
                step: [sxh, sxl, syh, syl],
                ref_offset: req.ref_offset.to_array(),
                center: req.center,
                julia_c: req.julia_c,
                res: [full_iw, full_ih],
                px_offset: [(tx0 * ss) as f32, (ty0 * ss) as f32],
                max_iter: req.max_iter,
                orbit_len: req.orbit_len,
                mode: req.mode,
                formula: req.formula,
                julia: req.julia,
                delta_exp: req.delta_exp,
                color_method: req.color_method,
                stripe_freq: req.stripe_freq,
                trap_type: req.trap_type,
                aux_on: method_needs_aux(req.color_method) as u32,
                sa_skip: req.sa_skip,
                glitch_on: req.glitch_on,
                sa_a: req.sa_a,
                sa_b: req.sa_b,
                sa_c: req.sa_c,
                sa_a_exp: req.sa_a_exp,
                sa_b_exp: req.sa_b_exp,
                sa_c_exp: req.sa_c_exp,
                bla_on: req.bla_on,
                start_iter: 0,
                end_iter: 0,
                _pad_ir: [0; 2],
            };
            queue.write_buffer(&iter_uniform, 0, bytemuck::bytes_of(&iu));

            let iter_view = make_iter_texture(device, [iw, ih]);
            let aux_view = make_iter_texture(device, [iw, ih]);
            let color_tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("export.color_tex"),
                size: wgpu::Extent3d { width: tw, height: th, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: EXPORT_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let color_view = color_tex.create_view(&wgpu::TextureViewDescriptor::default());
            let color_bg =
                make_color_bg(device, &color_bgl, &color_uniform, &iter_view, &aux_view);

            let unpadded_bpr = tw * bpp;
            let padded_bpr = unpadded_bpr.div_ceil(align) * align;
            let out_buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("export.readback"),
                size: (padded_bpr * th) as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            // Per-pass GPU timestamps — unconditional since D3.1 (the tile already blocks on
            // poll(Wait), so the extra 32-byte map is marginal). Four queries: iterate
            // begin/end (0,1) and color begin/end (2,3), resolved and read after the poll.
            let ts = if device.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
                let set = device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("export.timestamps"),
                    ty: wgpu::QueryType::Timestamp,
                    count: 4,
                });
                let resolve = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("export.ts_resolve"),
                    size: 256, // >= 4*8 bytes and a multiple of QUERY_RESOLVE_BUFFER_ALIGNMENT
                    usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });
                let read = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("export.ts_read"),
                    size: 32,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                Some((set, resolve, read))
            } else {
                None
            };

            // Chunk only once a window is actually SHORTER than the ask — i.e. once the priced
            // opening would split this tile. Below that the chunked and unchunked paths issue the
            // same single dispatch, so the setup would be pure cost (see the `chunker` note).
            if chunk_scope && chunker.is_none() && pricer.open(req.max_iter) < req.max_iter {
                chunker = Some(TileChunker::new(device, &shader, &iter_bgl, fe, [tile * ss, tile * ss]));
            }
            // Chunked tiles iterate BEFORE the main encoder: each window is its own polled
            // submission (that is the whole point), and the first window clears the counters.
            let chunked = match chunker.as_ref() {
                Some(ch) => {
                    let (passes, read_set, max_chunk) = ch.run_tile(
                        device, queue, &iter_bg, &counters_buf, &mut iu, &iter_uniform,
                        [iw, ih], req.max_iter, &mut pricer, Some(cancel), None,
                        &mut sum_iterate_ms,
                    )?;
                    max_dispatch_ms = max_dispatch_ms.max(max_chunk);
                    Some((passes, read_set))
                }
                None => None,
            };
            let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("export.encoder"),
            });
            if chunked.is_none() {
                // Zero the event counters before THIS tile's iterate pass, so each tile's counts
                // are read back independently and summed in u64 (no cross-tile u32 wrap). On a
                // chunked tile the first window's encoder already did this.
                enc.clear_buffer(&counters_buf, 0, None);
            }
            {
                let attach = |v| Some(wgpu::RenderPassColorAttachment {
                    view: v,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                });
                // Chunked: settle the ping-pong state into the tile's G-buffer (`fs_resolve`),
                // so the color pass below is oblivious to how the iterate was dispatched.
                // Unchunked: the classic single-dispatch iterate. Either way the pass writes
                // iter/aux and the iterate timestamps bracket it.
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("export.iter_pass"),
                    color_attachments: &[attach(&iter_view), attach(&aux_view)],
                    depth_stencil_attachment: None,
                    timestamp_writes: ts.as_ref().map(|(set, _, _)| {
                        wgpu::RenderPassTimestampWrites {
                            query_set: set,
                            beginning_of_pass_write_index: Some(0),
                            end_of_pass_write_index: Some(1),
                        }
                    }),
                    occlusion_query_set: None,
                });
                match (chunker.as_ref(), chunked) {
                    (Some(ch), Some((_, read_set))) => {
                        pass.set_pipeline(&ch.resolve_pipeline);
                        pass.set_bind_group(0, &iter_bg, &[]);
                        pass.set_bind_group(1, &ch.state_bg[read_set], &[]);
                        pass.draw(0..3, 0..1);
                    }
                    _ => {
                        pass.set_pipeline(&iter_pipeline);
                        pass.set_bind_group(0, &iter_bg, &[]);
                        pass.draw(0..3, 0..1);
                    }
                }
            }
            {
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("export.color_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &color_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: ts.as_ref().map(|(set, _, _)| {
                        wgpu::RenderPassTimestampWrites {
                            query_set: set,
                            beginning_of_pass_write_index: Some(2),
                            end_of_pass_write_index: Some(3),
                        }
                    }),
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&color_pipeline);
                pass.set_bind_group(0, &color_bg, &[]);
                pass.draw(0..3, 0..1);
            }
            if let Some((set, resolve, read)) = &ts {
                enc.resolve_query_set(set, 0..4, resolve, 0);
                enc.copy_buffer_to_buffer(resolve, 0, read, 0, 32);
            }
            enc.copy_buffer_to_buffer(
                &counters_buf, 0, &counters_read, 0, (crate::COUNTER_SLOTS * 4) as u64,
            );
            enc.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &color_tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &out_buf,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(padded_bpr),
                        rows_per_image: Some(th),
                    },
                },
                wgpu::Extent3d { width: tw, height: th, depth_or_array_layers: 1 },
            );
            queue.submit(std::iter::once(enc.finish()));

            let slice = out_buf.slice(..);
            let (tx, rx) = std::sync::mpsc::channel();
            slice.map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });
            let ts_rx = ts.as_ref().map(|(_, _, read)| {
                let (ttx, trx) = std::sync::mpsc::channel();
                read.slice(..).map_async(wgpu::MapMode::Read, move |r| {
                    let _ = ttx.send(r);
                });
                trx
            });
            let (ctx_, ctr_rx) = std::sync::mpsc::channel();
            counters_read.slice(..).map_async(wgpu::MapMode::Read, move |r| {
                let _ = ctx_.send(r);
            });
            let _ = device.poll(wgpu::Maintain::Wait);
            rx.recv()
                .map_err(|e| GpuError::Readback(e.to_string()))?
                .map_err(|e| GpuError::Readback(e.to_string()))?;

            // This tile's event counts → the u64 running totals.
            if ctr_rx.recv().map(|r| r.is_ok()).unwrap_or(false) {
                let mapped = counters_read.slice(..).get_mapped_range();
                let tile_ctr: &[u32] = bytemuck::cast_slice(&mapped[..crate::COUNTER_SLOTS * 4]);
                for (sum, &c) in ctr_sum.iter_mut().zip(tile_ctr) {
                    *sum += c as u64;
                }
                drop(mapped);
                counters_read.unmap();
            }

            // Resolved timestamps → pure-GPU iterate/color ms (ticks × the queue's timestamp
            // period), accumulated into the active `timing::capture` scope.
            if let (Some((_, _, read)), Some(ts_rx)) = (&ts, &ts_rx) {
                if ts_rx.recv().map(|r| r.is_ok()).unwrap_or(false) {
                    let mapped = read.slice(..).get_mapped_range();
                    let t: &[u64] = bytemuck::cast_slice(&mapped[..32]);
                    let period = queue.get_timestamp_period() as f64; // ns per tick
                    let iterate_ms = t[1].saturating_sub(t[0]) as f64 * period / 1.0e6;
                    let color_ms = t[3].saturating_sub(t[2]) as f64 * period / 1.0e6;
                    drop(mapped);
                    read.unmap();
                    sum_iterate_ms += iterate_ms;
                    sum_color_ms += color_ms;
                    crate::timing::accumulate(iterate_ms, color_ms);
                }
            }

            let data = slice.get_mapped_range();
            let row_floats = (tw * 4) as usize;
            for r in 0..th {
                let src = (r * padded_bpr) as usize;
                let src_row: &[f32] =
                    bytemuck::cast_slice(&data[src..src + unpadded_bpr as usize]);
                let dst = (((ty0 + r) * w + tx0) * 4) as usize;
                pixels[dst..dst + row_floats].copy_from_slice(src_row);
            }
            drop(data);
            out_buf.unmap();

            done_px += (tw as u64) * (th as u64);
            progress.store(((done_px.saturating_mul(1000)) / total_px) as u32, Relaxed);
            let wall_ms = t_tile.elapsed().as_secs_f64() * 1000.0;
            let chunk_passes = chunked.map_or(0, |(p, _)| p);
            if chunked.is_none() {
                max_dispatch_ms = max_dispatch_ms.max(wall_ms);
                // An UNCHUNKED tile is still evidence: it ran the full ask in `wall_ms`, so it
                // prices the serial chain exactly as a window would. Feeding it in is what keeps
                // the lazy trigger honest — a render whose ask fits one opening window but whose
                // tiles turn out slow anyway will raise the high-water mark here, shrink the
                // opening below the ask, and start chunking from the next tile on. (The wall
                // includes readback and the color pass, so it over-prices slightly — the safe
                // direction: chunking engages sooner, never later.)
                pricer.observe(req.max_iter, wall_ms);
            }
            if tile_trace_on() {
                eprintln!(
                    "[fd-export] tile {tx0},{ty0} {tw}x{th} ss={ss} wall={wall_ms:.1}ms \
cap={cap} chunks={chunk_passes}"
                );
            }
            if chunk_passes <= 1 {
                cap = export_tile_cap(cap, wall_ms, tile);
            }
            // else: serial regime — the wall is dwell-chain time, which shrinking the AREA cannot
            // reduce (it only multiplies how many chains the frame pays; the same wrong-actuator
            // retreat as the home-from-deep device loss). The chunk windows already bound every
            // dispatch, so the area cap holds its value here.
            tx0 += tw;
        }
        ty0 += th;
    }

    Ok(ExportResult {
        width: w,
        height: h,
        ss,
        pixels,
        iterate_ms: sum_iterate_ms,
        color_ms: sum_color_ms,
        counters: ctr_sum,
        max_dispatch_ms,
    })
}

/// Tiled variant of [`render_iter`]: renders the raw iteration texture in **bounded per-tile
/// dispatches** (each ≈ `work_budget` nominal steps) and reassembles the full `width×height`
/// RGBA32F buffer. The point is that no single GPU submission runs long enough to trip the OS
/// watchdog or to hang uninterruptibly on the deep-interior "dark dendrite core" pixels that no
/// acceleration (SA/BLA) can skip — the pathology that made multi-reference glitch correction run
/// for hours. `deadline` is checked between tiles: once it passes, the render returns
/// `GpuError::Canceled`, so the correction loop can keep whatever it has already merged and stop
/// instead of blocking. `ss = 1` (the correction path never supersamples); emits the iteration
/// texture (not a colored image) so the caller can find glitched pixels (`smooth_iter < -1.5`).
///
/// ⭐`roi` (region of interest) is a `width*height` mask of the pixels the caller will actually
/// READ; tiles containing none of them are skipped entirely and their pixels are left ZERO. This
/// exists because the multi-reference corrector re-rendered the WHOLE FRAME once per extra
/// reference to repair a handful of pixels — measured at 1.3e6×, passes 5..64 each re-iterated
/// all 2,073,600 pixels to resolve one or two, and correction ate 10.2 s of an 11.1 s render.
/// Restricting each pass to the tiles that still contain glitched pixels changes NO output (the
/// caller adopts only the pixels it asked for) and removes almost all of that cost. `None`
/// renders the full frame.
///
/// ⚠A skipped tile's pixels read back as 0.0, which is a *valid-looking* escape value — so a
/// caller passing `roi` must never read outside its own mask.
#[allow(clippy::too_many_arguments)]
pub fn render_iter_tiled(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    req: &ExportRequest,
    work_budget: u64,
    deadline: Option<std::time::Instant>,
    roi: Option<&[bool]>,
) -> Result<ExportResult, GpuError> {
    let max_dim = device.limits().max_texture_dimension_2d;
    let max_buf = device.limits().max_buffer_size;
    let w = req.width.max(1);
    let h = req.height.max(1);

    // Tile size (output px): bound by the texture cap, the per-tile readback buffer
    // (tile²·16 B ≤ max_buf), and — the whole point — the ITERATION WORK per dispatch.
    // work_per_px = max_iter (ss = 1). A `work_budget` smaller than the export path's keeps
    // tiles small enough that even a tile full of ~50×-cost, BLA-unskippable dark-core pixels
    // finishes well inside the TDR window, so the loop stays interruptible between tiles.
    let by_tex = max_dim.max(1);
    let by_buf = (((max_buf / 16) as f64).sqrt() as u32).max(256);
    let work_per_px = req.max_iter.max(1) as u64;
    let by_work = (((work_budget / work_per_px.max(1)) as f64).sqrt() as u32).max(16);
    // ⛔**MEASURED-FALSE, 2026-08-21: do not shrink the tile when an `roi` is set.** The idea was
    // that a big tile defeats ROI twice over — it catches a scattered glitch almost everywhere
    // (10/11 tiles wanted at 816 px) and allocates ~32 MB of textures + readback to repair a
    // handful of pixels. Tried at 128 px: skipping improved (58/155 tiles) and output was
    // byte-identical, but correction got SLOWER — 138 → 161 ms per pass, 9.0 → 10.6 s total.
    // The extra submissions cost more than the allocation they save, so per-tile FIXED overhead
    // dominates, not bytes. The way out is fewer, smaller dispatches — i.e. the gather pass in
    // the TODO — not a different tile size.
    let tile = by_tex.min(by_buf).min(by_work).clamp(1, 2048);

    let t_setup = std::time::Instant::now();
    let shader = shader_module(device);
    let iter_bgl = iter_bind_group_layout(device);
    let iter_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("itertiled.layout"),
        bind_group_layouts: &[&iter_bgl],
        push_constant_ranges: &[],
    });
    let iter_pipeline = fullscreen_pipeline(
        device, &shader, &iter_layout, "fs_iterate", &[ITER_FORMAT, ITER_FORMAT],
        "itertiled.pipeline",
    );
    let setup_ms = t_setup.elapsed().as_secs_f64() * 1000.0;

    // Chunked per-tile iterate, same rule and reason as `render_export`. ⭐Glitch detection is
    // IN scope since beta.124 (`ST_GLITCHED`), which is what matters here: this is the multi-
    // reference corrector's own base pass, whose BLA-less dark-core tiles are the most
    // latency-bound dispatches the app issues, and its 120 s deadline is only checked BETWEEN
    // tiles — so before chunking, one such tile could still overrun the watchdog inside it.
    let chunk_scope = (req.mode == 1 || req.mode == 0 || req.mode == 2)
        && req.formula <= 3
        && device.limits().max_color_attachment_bytes_per_sample
            >= if req.mode == 2 { 64 } else { 48 };
    // Lazy for the same reason as `render_export` — and it matters most HERE, since the
    // multi-reference corrector calls this once per pass (up to 64).
    let mut chunker: Option<TileChunker> = None;
    let mut pricer = ChunkPricer::new();
    let mut max_dispatch_ms = 0.0f64;
    let mut chunk_wall_sink = 0.0f64; // iterate_ms stays 0.0 on this path (see the result)
    // ROI effectiveness, reported under the `tile` trace: skipping depends entirely on whether
    // the wanted pixels CLUSTER, and tile size falls out of the work budget, so "how much did
    // this actually save" is not something to reason about — measure it.
    let (mut roi_skipped, mut roi_total) = (0u32, 0u32);

    let iter_uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("itertiled.uniform"),
        size: std::mem::size_of::<IterUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    check_orbit_binding(device, req.orbit.len(), req.bla.len())?;
    let orbit_buf = make_orbit_buffer(device, (req.orbit.len() + req.bla.len()).max(1) as u32);
    if !req.orbit.is_empty() {
        queue.write_buffer(&orbit_buf, 0, bytemuck::cast_slice(req.orbit.as_slice()));
    }
    if !req.bla.is_empty() {
        let off = (req.orbit.len() * 16) as u64;
        queue.write_buffer(&orbit_buf, off, bytemuck::cast_slice(req.bla.as_slice()));
    }
    let counters_buf = crate::make_counters_buf(device);
    let counters_read = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("itertiled.counters_read"),
        size: (crate::COUNTER_SLOTS * 4) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let iter_bg = make_iter_bg(device, &iter_bgl, &iter_uniform, &orbit_buf, &counters_buf);

    let split = |v: f64| -> (f32, f32) {
        let hi = v as f32;
        (hi, (v - hi as f64) as f32)
    };
    // Step mantissa is per full-resolution pixel (ss = 1); px_offset places each tile in the
    // full frame, exactly as `render_export` does — so no per-tile reference arithmetic.
    let (sxh, sxl) = split(req.span_mantissa.x / w as f64);
    let (syh, syl) = split(req.span_mantissa.y / h as f64);

    let bpp = 16u32;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let mut pixels = vec![0.0_f32; (w as usize) * (h as usize) * 4];
    let mut ctr_sum = [0u64; crate::COUNTER_SLOTS];

    // Wall-adaptive cap, same rule as `render_export` (see `export_tile_cap`): the correction
    // loop's budget is nominal too, and its dark-core tiles are exactly where nominal != real.
    let mut cap = tile;
    let mut ty0 = 0u32;
    while ty0 < h {
        let th = cap.min(h - ty0);
        let mut tx0 = 0u32;
        while tx0 < w {
            let t_tile = std::time::Instant::now();
            if deadline.is_some_and(|d| std::time::Instant::now() >= d) {
                return Err(GpuError::Canceled);
            }
            let tw = cap.min(w - tx0).min(th.max(16));
            roi_total += 1;
            // Region of interest: skip a tile the caller will not read from. The scan is O(tile
            // area) of plain bools against a GPU dispatch over the same area, so it pays for
            // itself the first time it says "no".
            if let Some(mask) = roi {
                let wanted = (ty0..ty0 + th).any(|y| {
                    let row = (y as usize) * (w as usize);
                    mask[row + tx0 as usize..row + (tx0 + tw) as usize].iter().any(|&m| m)
                });
                if !wanted {
                    roi_skipped += 1;
                    tx0 += tw;
                    continue;
                }
            }
            let mut iu = IterUniforms {
                step: [sxh, sxl, syh, syl],
                ref_offset: req.ref_offset.to_array(),
                center: req.center,
                julia_c: req.julia_c,
                res: [w as f32, h as f32],
                px_offset: [tx0 as f32, ty0 as f32],
                max_iter: req.max_iter,
                orbit_len: req.orbit_len,
                mode: req.mode,
                formula: req.formula,
                julia: req.julia,
                delta_exp: req.delta_exp,
                color_method: 0,
                stripe_freq: 1.0,
                trap_type: 0,
                aux_on: 0,
                sa_skip: req.sa_skip,
                glitch_on: req.glitch_on,
                sa_a: req.sa_a,
                sa_b: req.sa_b,
                sa_c: req.sa_c,
                sa_a_exp: req.sa_a_exp,
                sa_b_exp: req.sa_b_exp,
                sa_c_exp: req.sa_c_exp,
                bla_on: req.bla_on,
                start_iter: 0,
                end_iter: 0,
                _pad_ir: [0; 2],
            };
            queue.write_buffer(&iter_uniform, 0, bytemuck::bytes_of(&iu));

            let mk = |label| {
                device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d { width: tw, height: th, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: ITER_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                })
            };
            let main_tex = mk("itertiled.main");
            let aux_tex = mk("itertiled.aux");
            let main_view = main_tex.create_view(&wgpu::TextureViewDescriptor::default());
            let aux_view = aux_tex.create_view(&wgpu::TextureViewDescriptor::default());

            let unpadded_bpr = tw * bpp;
            let padded_bpr = unpadded_bpr.div_ceil(align) * align;
            let out_buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("itertiled.readback"),
                size: (padded_bpr * th) as u64,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            // Same lazy trigger as `render_export`.
            if chunk_scope && chunker.is_none() && pricer.open(req.max_iter) < req.max_iter {
                chunker =
                    Some(TileChunker::new(device, &shader, &iter_bgl, req.mode == 2, [tile, tile]));
            }
            // Chunked tiles iterate BEFORE the main encoder, one polled submission per window;
            // the first window clears the counters. The deadline is honoured between windows.
            let chunked = match chunker.as_ref() {
                Some(ch) => {
                    let (passes, read_set, max_chunk) = ch.run_tile(
                        device, queue, &iter_bg, &counters_buf, &mut iu, &iter_uniform,
                        [tw, th], req.max_iter, &mut pricer, None, deadline,
                        &mut chunk_wall_sink,
                    )?;
                    max_dispatch_ms = max_dispatch_ms.max(max_chunk);
                    Some((passes, read_set))
                }
                None => None,
            };
            let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("itertiled.enc"),
            });
            if chunked.is_none() {
                // Zero counters before THIS tile so per-tile u32 counts can't wrap; summed in
                // u64. On a chunked tile the first window's encoder already did this.
                enc.clear_buffer(&counters_buf, 0, None);
            }
            {
                let attach = |v| Some(wgpu::RenderPassColorAttachment {
                    view: v,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                });
                let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("itertiled.iter_pass"),
                    color_attachments: &[attach(&main_view), attach(&aux_view)],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                match (chunker.as_ref(), chunked) {
                    (Some(ch), Some((_, read_set))) => {
                        pass.set_pipeline(&ch.resolve_pipeline);
                        pass.set_bind_group(0, &iter_bg, &[]);
                        pass.set_bind_group(1, &ch.state_bg[read_set], &[]);
                        pass.draw(0..3, 0..1);
                    }
                    _ => {
                        pass.set_pipeline(&iter_pipeline);
                        pass.set_bind_group(0, &iter_bg, &[]);
                        pass.draw(0..3, 0..1);
                    }
                }
            }
            enc.copy_buffer_to_buffer(
                &counters_buf, 0, &counters_read, 0, (crate::COUNTER_SLOTS * 4) as u64,
            );
            enc.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &main_tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &out_buf,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(padded_bpr),
                        rows_per_image: Some(th),
                    },
                },
                wgpu::Extent3d { width: tw, height: th, depth_or_array_layers: 1 },
            );
            queue.submit(std::iter::once(enc.finish()));

            let slice = out_buf.slice(..);
            let (tx, rx) = std::sync::mpsc::channel();
            slice.map_async(wgpu::MapMode::Read, move |r| {
                let _ = tx.send(r);
            });
            let (ctx_, ctr_rx) = std::sync::mpsc::channel();
            counters_read.slice(..).map_async(wgpu::MapMode::Read, move |r| {
                let _ = ctx_.send(r);
            });
            let _ = device.poll(wgpu::Maintain::Wait);
            rx.recv()
                .map_err(|e| GpuError::Readback(e.to_string()))?
                .map_err(|e| GpuError::Readback(e.to_string()))?;
            if ctr_rx.recv().map(|r| r.is_ok()).unwrap_or(false) {
                let mapped = counters_read.slice(..).get_mapped_range();
                let tile_ctr: &[u32] = bytemuck::cast_slice(&mapped[..crate::COUNTER_SLOTS * 4]);
                for (sum, &c) in ctr_sum.iter_mut().zip(tile_ctr) {
                    *sum += c as u64;
                }
                drop(mapped);
                counters_read.unmap();
            }
            let data = slice.get_mapped_range();
            let row_floats = (tw * 4) as usize;
            for r in 0..th {
                let src = (r * padded_bpr) as usize;
                let src_row: &[f32] =
                    bytemuck::cast_slice(&data[src..src + unpadded_bpr as usize]);
                let dst = (((ty0 + r) * w + tx0) * 4) as usize;
                pixels[dst..dst + row_floats].copy_from_slice(src_row);
            }
            drop(data);
            out_buf.unmap();
            let wall_ms = t_tile.elapsed().as_secs_f64() * 1000.0;
            let chunk_passes = chunked.map_or(0, |(p, _)| p);
            if chunked.is_none() {
                max_dispatch_ms = max_dispatch_ms.max(wall_ms);
                pricer.observe(req.max_iter, wall_ms); // see `render_export` — keeps lazy honest
            }
            if tile_trace_on() {
                eprintln!(
                    "[fd-export] itile {tx0},{ty0} {tw}x{th} wall={wall_ms:.1}ms cap={cap} \
chunks={chunk_passes}"
                );
            }
            if chunk_passes <= 1 {
                cap = export_tile_cap(cap, wall_ms, tile);
            }
            // else: serial regime — hold the area cap (see `render_export`; same reasoning).
            tx0 += tw;
        }
        ty0 += th;
    }
    if roi.is_some() && tile_trace_on() {
        eprintln!(
            "[fd-export] roi: {roi_skipped}/{roi_total} tiles skipped (tile={tile}px) setup={setup_ms:.0}ms total={:.0}ms",
            t_setup.elapsed().as_secs_f64() * 1000.0
        );
    }
    Ok(ExportResult {
        width: w,
        height: h,
        ss: 1,
        pixels,
        iterate_ms: 0.0, // per-tile GPU timestamps omitted here; the loop tracks wall-clock
        color_ms: 0.0,
        counters: ctr_sum,
        max_dispatch_ms,
    })
}

/// Copy the event-counter atomics to a MAP_READ staging buffer and read them back
/// (blocking; every caller is already synchronous), widened to u64. Zeros on any failure.
/// Used by the single-pass `render_iter` (one texture, no cross-tile accumulation, so the
/// u32 slots can't wrap); the tiled `render_export` sums per tile inline instead.
fn read_counters(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    counters_buf: &wgpu::Buffer,
    counters_read: &wgpu::Buffer,
) -> [u64; crate::COUNTER_SLOTS] {
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("export.counters_copy"),
    });
    enc.copy_buffer_to_buffer(counters_buf, 0, counters_read, 0, (crate::COUNTER_SLOTS * 4) as u64);
    queue.submit(std::iter::once(enc.finish()));
    let slice = counters_read.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = device.poll(wgpu::Maintain::Wait);
    let mut out = [0u64; crate::COUNTER_SLOTS];
    if rx.recv().map(|r| r.is_ok()).unwrap_or(false) {
        let mapped = slice.get_mapped_range();
        let raw: &[u32] = bytemuck::cast_slice(&mapped[..crate::COUNTER_SLOTS * 4]);
        for (o, &c) in out.iter_mut().zip(raw) {
            *o = c as u64;
        }
        drop(mapped);
        counters_read.unmap();
    }
    out
}

/// Render only the **iteration pass** for a single (clamped) tile and read back the raw
/// iteration texture — RGBA32F per pixel: `(smooth_iter, normal.x, normal.y, DE_log2)`,
/// with `smooth_iter < 0` marking interior. For validation (`--selftest`): comparing raw
/// dwell across render modes is far more sensitive than comparing final colors. Always
/// `ss = 1`; size is clamped to the device's max 2D texture dimension.
pub fn render_iter(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    req: &ExportRequest,
) -> Result<ExportResult, GpuError> {
    let max_dim = device.limits().max_texture_dimension_2d;
    let w = req.width.clamp(1, max_dim);
    let h = req.height.clamp(1, max_dim);

    let shader = shader_module(device);
    let iter_bgl = iter_bind_group_layout(device);
    let iter_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("selftest.iter_layout"),
        bind_group_layouts: &[&iter_bgl],
        push_constant_ranges: &[],
    });
    let iter_pipeline = fullscreen_pipeline(
        device, &shader, &iter_layout, "fs_iterate", &[ITER_FORMAT, ITER_FORMAT],
        "selftest.iter_pipeline",
    );

    let iter_uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("selftest.iter_uniform"),
        size: std::mem::size_of::<IterUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    check_orbit_binding(device, req.orbit.len(), req.bla.len())?;
    let orbit_buf = make_orbit_buffer(device, (req.orbit.len() + req.bla.len()).max(1) as u32);
    if !req.orbit.is_empty() {
        queue.write_buffer(&orbit_buf, 0, bytemuck::cast_slice(req.orbit.as_slice()));
    }
    if !req.bla.is_empty() {
        let off = (req.orbit.len() * 16) as u64;
        queue.write_buffer(&orbit_buf, off, bytemuck::cast_slice(req.bla.as_slice()));
    }
    let counters_buf = crate::make_counters_buf(device);
    let counters_read = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("selftest.counters_read"),
        size: (crate::COUNTER_SLOTS * 4) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let iter_bg = make_iter_bg(device, &iter_bgl, &iter_uniform, &orbit_buf, &counters_buf);

    let split = |v: f64| -> (f32, f32) {
        let hi = v as f32;
        (hi, (v - hi as f64) as f32)
    };
    let (sxh, sxl) = split(req.span_mantissa.x / w as f64);
    let (syh, syl) = split(req.span_mantissa.y / h as f64);
    let iu = IterUniforms {
        step: [sxh, sxl, syh, syl],
        ref_offset: req.ref_offset.to_array(),
        center: req.center,
        julia_c: req.julia_c,
        res: [w as f32, h as f32],
        px_offset: [0.0, 0.0],
        max_iter: req.max_iter,
        orbit_len: req.orbit_len,
        mode: req.mode,
        formula: req.formula,
        julia: req.julia,
        delta_exp: req.delta_exp,
        color_method: 0,
        stripe_freq: 1.0,
        trap_type: 0,
        aux_on: 0,
        sa_skip: req.sa_skip,
        glitch_on: req.glitch_on,
        sa_a: req.sa_a,
        sa_b: req.sa_b,
        sa_c: req.sa_c,
        sa_a_exp: req.sa_a_exp,
        sa_b_exp: req.sa_b_exp,
        sa_c_exp: req.sa_c_exp,
        bla_on: req.bla_on,
        start_iter: 0,
        end_iter: 0,
        _pad_ir: [0; 2],
    };
    queue.write_buffer(&iter_uniform, 0, bytemuck::bytes_of(&iu));

    let tex = |label| {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: ITER_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
            .create_view(&wgpu::TextureViewDescriptor::default())
    };
    let aux_view = tex("selftest.iter_aux");
    let main_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("selftest.iter_main"),
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: ITER_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let main_copy_view = main_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let bpp = 16u32;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let unpadded_bpr = w * bpp;
    let padded_bpr = unpadded_bpr.div_ceil(align) * align;
    let out_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("selftest.readback"),
        size: (padded_bpr * h) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Iterate-pass timestamps (D3.1/F14): render_iter is the primitive under the
    // multi-reference glitch loop — the app's worst historical time sink ran on a path
    // with zero instrumentation.
    let ts = if device.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
        let set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("selftest.timestamps"),
            ty: wgpu::QueryType::Timestamp,
            count: 2,
        });
        let resolve = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("selftest.ts_resolve"),
            size: 256,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let read = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("selftest.ts_read"),
            size: 16,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Some((set, resolve, read))
    } else {
        None
    };
    let mut enc =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("selftest.enc") });
    {
        let attach = |v| Some(wgpu::RenderPassColorAttachment {
            view: v,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        });
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("selftest.iter_pass"),
            color_attachments: &[attach(&main_copy_view), attach(&aux_view)],
            depth_stencil_attachment: None,
            timestamp_writes: ts.as_ref().map(|(set, _, _)| wgpu::RenderPassTimestampWrites {
                query_set: set,
                beginning_of_pass_write_index: Some(0),
                end_of_pass_write_index: Some(1),
            }),
            occlusion_query_set: None,
        });
        pass.set_pipeline(&iter_pipeline);
        pass.set_bind_group(0, &iter_bg, &[]);
        pass.draw(0..3, 0..1);
    }
    if let Some((set, resolve, read)) = &ts {
        enc.resolve_query_set(set, 0..2, resolve, 0);
        enc.copy_buffer_to_buffer(resolve, 0, read, 0, 16);
    }
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &main_tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &out_buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    queue.submit(std::iter::once(enc.finish()));

    let slice = out_buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let ts_rx = ts.as_ref().map(|(_, _, read)| {
        let (ttx, trx) = std::sync::mpsc::channel();
        read.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ = ttx.send(r);
        });
        trx
    });
    let _ = device.poll(wgpu::Maintain::Wait);
    rx.recv().map_err(|e| GpuError::Readback(e.to_string()))?.map_err(|e| GpuError::Readback(e.to_string()))?;
    let mut iterate_ms = 0.0f64;
    if let (Some((_, _, read)), Some(ts_rx)) = (&ts, &ts_rx) {
        if ts_rx.recv().map(|r| r.is_ok()).unwrap_or(false) {
            let mapped = read.slice(..).get_mapped_range();
            let t: &[u64] = bytemuck::cast_slice(&mapped[..16]);
            iterate_ms = t[1].saturating_sub(t[0]) as f64
                * queue.get_timestamp_period() as f64
                / 1.0e6;
            drop(mapped);
            read.unmap();
        }
    }

    let data = slice.get_mapped_range();
    let mut pixels = vec![0.0_f32; (w as usize) * (h as usize) * 4];
    let row_floats = (w * 4) as usize;
    for r in 0..h {
        let src = (r * padded_bpr) as usize;
        let src_row: &[f32] = bytemuck::cast_slice(&data[src..src + unpadded_bpr as usize]);
        let dst = (r as usize) * row_floats;
        pixels[dst..dst + row_floats].copy_from_slice(src_row);
    }
    drop(data);
    out_buf.unmap();

    let counters = read_counters(device, queue, &counters_buf, &counters_read);
    Ok(ExportResult {
        width: w, height: h, ss: 1, pixels, iterate_ms, color_ms: 0.0, counters,
        max_dispatch_ms: 0.0,
    })
}

/// `render_iter`, but with **iteration-range tiling**: the `0..max_iter` loop is split across
/// `ceil(max_iter / chunk_iters)` bounded GPU submissions, carrying per-pixel state (z, dz, iter,
/// status) between passes in ping-pong textures — so an arbitrarily high iteration count can never
/// run as a single watchdog-tripping dispatch. Output is bit-identical to `render_iter` for the
/// supported scope (the resumable shader replicates the direct branch's arithmetic and order
/// exactly, and the state carries full df32 precision). Scope: DIRECT (`mode == 1`), df32
/// perturbation (`mode == 0`) and floatexp perturbation (`mode == 2`), holomorphic formulas 0..=3,
/// aux coloring off (glitch detection is supported since beta.124 — a glitched pixel settles as
/// `ST_GLITCHED` and resolves to the same `-2` sentinel the single pass emits); anything else
/// falls back to plain `render_iter`.
/// Always `ss = 1`, like `render_iter`. `iterate_ms` is not measured on this path (0.0).
///
/// Mode 2 runs the four-target `fs_iterate_chunk_fe` entry point instead — floatexp state does not
/// fit in three (see `chunking_mode2_available`) — and needs 64 bytes/sample rather than 48.
pub fn render_iter_chunked(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    req: &ExportRequest,
    chunk_iters: u32,
) -> Result<ExportResult, GpuError> {
    // The chunk pass writes Rgba32Float state attachments — three (48 bytes/sample) for direct and
    // mode 0, four (64) for mode 2. A device that granted less can't run it: fall back to the
    // single-pass render (the caller's TDR exposure is then what it always was; the app requests
    // min(adapter, 64) at device creation). Scope: direct (1), df32 perturbation (0) and floatexp
    // perturbation (2) with aux off, holomorphic formulas 0..3 — the same scope the chunk shaders
    // are written to, which is narrower than `fs_iterate`'s.
    let fe = req.mode == 2; // floatexp: the four-target entry point
    let mode_ok = req.mode == 1 || req.mode == 0 || req.mode == 2;
    let attach_need = if fe { 64 } else { 48 };
    if !mode_ok
        || req.formula > 3
        || chunk_iters == 0
        || device.limits().max_color_attachment_bytes_per_sample < attach_need
    {
        return render_iter(device, queue, req);
    }
    let max_dim = device.limits().max_texture_dimension_2d;
    let w = req.width.clamp(1, max_dim);
    let h = req.height.clamp(1, max_dim);

    let shader = shader_module(device);
    let iter_bgl = iter_bind_group_layout(device);
    let targets: usize = if fe { 4 } else { 3 };
    let state_bgl = crate::state_bind_group_layout_n(device, targets as u32);
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("chunked.layout"),
        bind_group_layouts: &[&iter_bgl, &state_bgl],
        push_constant_ranges: &[],
    });
    let chunk_formats = [ITER_FORMAT; 4];
    let chunk_pipeline = fullscreen_pipeline(
        device,
        &shader,
        &layout,
        if fe { "fs_iterate_chunk_fe" } else { "fs_iterate_chunk" },
        &chunk_formats[..targets],
        "chunked.chunk_pipeline",
    );
    let resolve_pipeline = fullscreen_pipeline(
        device, &shader, &layout, "fs_resolve", &[ITER_FORMAT, ITER_FORMAT],
        "chunked.resolve_pipeline",
    );

    let iter_uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("chunked.iter_uniform"),
        size: std::mem::size_of::<IterUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    // Modes 0/2 iterate against the reference orbit; direct mode binds a 1-slot placeholder. The
    // flattened BLA tree is appended after the orbit exactly as `render_iter` lays it out.
    // ⚠It must be uploaded even though mode 0 never traverses it: mode 2's chunk pass rebuilds its
    // BLA table every pass, and a chunk pass that ran with `bla_on = 0` and an empty tree is the
    // beta.101 e100 pathology verbatim — 0.04 Gsteps/s against the base pass's 174 in the same
    // frame, which reads as "chunking is slow" rather than "chunking is broken".
    check_orbit_binding(device, req.orbit.len(), req.bla.len())?;
    let orbit_buf = make_orbit_buffer(device, (req.orbit.len() + req.bla.len()).max(1) as u32);
    if !req.orbit.is_empty() {
        queue.write_buffer(&orbit_buf, 0, bytemuck::cast_slice(req.orbit.as_slice()));
    }
    if !req.bla.is_empty() {
        let off = (req.orbit.len() * std::mem::size_of::<[f32; 4]>()) as u64;
        queue.write_buffer(&orbit_buf, off, bytemuck::cast_slice(req.bla.as_slice()));
    }
    let counters_buf = crate::make_counters_buf(device);
    let counters_read = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("chunked.counters_read"),
        size: (crate::COUNTER_SLOTS * 4) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let iter_bg = make_iter_bg(device, &iter_bgl, &iter_uniform, &orbit_buf, &counters_buf);

    // Ping-pong state: pass k reads set (k % 2) and writes set ((k+1) % 2).
    let state = [
        crate::make_state_textures(device, [w, h], targets),
        crate::make_state_textures(device, [w, h], targets),
    ];
    let state_bg = [
        crate::make_state_bg(device, &state_bgl, &state[0]),
        crate::make_state_bg(device, &state_bgl, &state[1]),
    ];

    let split = |v: f64| -> (f32, f32) {
        let hi = v as f32;
        (hi, (v - hi as f64) as f32)
    };
    let (sxh, sxl) = split(req.span_mantissa.x / w as f64);
    let (syh, syl) = split(req.span_mantissa.y / h as f64);
    let mut iu = IterUniforms {
        step: [sxh, sxl, syh, syl],
        ref_offset: req.ref_offset.to_array(),
        center: req.center,
        julia_c: req.julia_c,
        res: [w as f32, h as f32],
        px_offset: [0.0, 0.0],
        max_iter: req.max_iter,
        orbit_len: req.orbit_len, // mode 0 iterates + rebases against the reference
        mode: req.mode,
        formula: req.formula,
        julia: req.julia,
        delta_exp: req.delta_exp,
        color_method: 0,
        stripe_freq: 1.0,
        trap_type: 0,
        aux_on: 0,
        sa_skip: req.sa_skip, // mode 0's SA seeding is replicated in the chunk init
        glitch_on: 0,
        sa_a: req.sa_a,
        sa_b: req.sa_b,
        sa_c: req.sa_c,
        sa_a_exp: req.sa_a_exp,
        sa_b_exp: req.sa_b_exp,
        sa_c_exp: req.sa_c_exp,
        // ⚠Passed through, not zeroed. It is inert for direct and mode 0 (neither branch reads it —
        // BLA lives in the mode-2 loop), and load-bearing for mode 2: see the orbit upload above.
        bla_on: req.bla_on,
        start_iter: 0,
        end_iter: 0,
        _pad_ir: [0; 2],
    };

    // One bounded submission per iteration range; poll-wait between them so each stays a short,
    // watchdog-friendly unit of GPU work (the entire point of this path).
    let chunks = req.max_iter.div_ceil(chunk_iters.max(1)).max(1);
    let mut read_set = 0usize;
    for k in 0..chunks {
        iu.start_iter = k * chunk_iters;
        iu.end_iter = iu.start_iter.saturating_add(chunk_iters).min(req.max_iter);
        queue.write_buffer(&iter_uniform, 0, bytemuck::bytes_of(&iu));
        let write_set = 1 - read_set;
        let mut enc = device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("chunked.enc") });
        {
            let attach = |v| Some(wgpu::RenderPassColorAttachment {
                view: v,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            });
            let attachments: Vec<_> = state[write_set].iter().map(|v| attach(v)).collect();
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("chunked.chunk_pass"),
                color_attachments: &attachments,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&chunk_pipeline);
            pass.set_bind_group(0, &iter_bg, &[]);
            pass.set_bind_group(1, &state_bg[read_set], &[]);
            pass.draw(0..3, 0..1);
        }
        queue.submit(std::iter::once(enc.finish()));
        let _ = device.poll(wgpu::Maintain::Wait);
        read_set = write_set;
    }

    // Resolve the settled state into the normal iteration G-buffer and read it back.
    let mk_out = |label: &str| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: ITER_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    };
    let main_tex = mk_out("chunked.iter_main");
    let main_view = main_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let aux_view = mk_out("chunked.iter_aux").create_view(&wgpu::TextureViewDescriptor::default());

    let bpp = 16u32;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let unpadded_bpr = w * bpp;
    let padded_bpr = unpadded_bpr.div_ceil(align) * align;
    let out_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("chunked.readback"),
        size: (padded_bpr * h) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut enc = device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("chunked.resolve_enc") });
    {
        let attach = |v| Some(wgpu::RenderPassColorAttachment {
            view: v,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        });
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("chunked.resolve_pass"),
            color_attachments: &[attach(&main_view), attach(&aux_view)],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&resolve_pipeline);
        pass.set_bind_group(0, &iter_bg, &[]);
        pass.set_bind_group(1, &state_bg[read_set], &[]);
        pass.draw(0..3, 0..1);
    }
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &main_tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &out_buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    queue.submit(std::iter::once(enc.finish()));

    let slice = out_buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = device.poll(wgpu::Maintain::Wait);
    rx.recv()
        .map_err(|e| GpuError::Readback(e.to_string()))?
        .map_err(|e| GpuError::Readback(e.to_string()))?;
    let data = slice.get_mapped_range();
    let mut pixels = vec![0.0_f32; (w as usize) * (h as usize) * 4];
    let row_floats = (w * 4) as usize;
    for r in 0..h {
        let src = (r * padded_bpr) as usize;
        let src_row: &[f32] = bytemuck::cast_slice(&data[src..src + unpadded_bpr as usize]);
        let dst = (r as usize) * row_floats;
        pixels[dst..dst + row_floats].copy_from_slice(src_row);
    }
    drop(data);
    out_buf.unmap();

    let counters = read_counters(device, queue, &counters_buf, &counters_read);
    Ok(ExportResult {
        width: w, height: h, ss: 1, pixels, iterate_ms: 0.0, color_ms: 0.0, counters,
        max_dispatch_ms: 0.0,
    })
}

/// Color an already-computed iteration buffer (main target, `w*h*4` floats: smooth-iter, normal.x,
/// normal.y, DE) into a linear-RGBA image. The multi-reference glitch corrector merges several
/// `render_iter` passes into one glitch-free buffer, then hands it here to be colored. Supports the
/// non-aux coloring methods (smooth / distance / relief / glow — everything that reads only the
/// main target); aux methods (stripe/TIA/trap/decomposition) need per-orbit statistics the merged
/// buffer doesn't carry, so the caller renders those uncorrected.
pub fn color_iter_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    req: &ExportRequest,
    iter_pixels: &[f32],
) -> Result<ExportResult, GpuError> {
    let w = req.width.max(1);
    let h = req.height.max(1);
    let shader = shader_module(device);
    let color_bgl = color_bind_group_layout(device);
    let color_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("coloriter.layout"),
        bind_group_layouts: &[&color_bgl],
        push_constant_ranges: &[],
    });
    let color_pipeline = fullscreen_pipeline(
        device, &shader, &color_layout, "fs_color", &[EXPORT_FORMAT], "coloriter.pipeline",
    );

    let color_uniform = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("coloriter.uniform"),
        size: std::mem::size_of::<ColorUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let cu = ColorUniforms {
        stop_count: req.stop_count,
        cycle: req.cycle,
        offset: req.offset,
        ss: 1,
        light: req.light,
        light_angle: req.light_angle,
        light_height: req.light_height,
        de_on: req.de_on,
        de_strength: req.de_strength,
        de_width: req.de_width,
        de_phase: req.de_phase,
        color_method: req.color_method,
        aa_filter: 1,
        reproject: 0,
        uv_offset: [0.0, 0.0],
        uv_scale: 1.0,
        vig_on: req.vignette.on,
        vig_dim: req.vignette.dim,
        vig_soft: req.vignette.soft,
        vig_center: req.vignette.center,
        vig_radius: req.vignette.radius,
        _pad_vig: 0.0,
        interior_col: req.interior_col,
        stops: req.stops,
        out_res: [w as f32, h as f32],
        norm_mode: req.norm_mode,
        norm_lo: req.norm_lo,
    };
    queue.write_buffer(&color_uniform, 0, bytemuck::bytes_of(&cu));

    // Upload the merged iteration buffer into a sampled texture; a zeroed aux (unused for the
    // supported non-aux methods) satisfies the color bind group.
    let make_input = |label| device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: ITER_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let write = |tex: &wgpu::Texture, data: &[f32]| {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(data),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 16),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
    };
    let iter_tex = make_input("coloriter.iter");
    write(&iter_tex, iter_pixels);
    let aux_tex = make_input("coloriter.aux");
    write(&aux_tex, &vec![0.0_f32; (w as usize) * (h as usize) * 4]);
    let iter_view = iter_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let aux_view = aux_tex.create_view(&wgpu::TextureViewDescriptor::default());
    let color_bg = make_color_bg(device, &color_bgl, &color_uniform, &iter_view, &aux_view);

    let color_tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("coloriter.out"),
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: EXPORT_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let color_view = color_tex.create_view(&wgpu::TextureViewDescriptor::default());

    let bpp = 16u32;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let unpadded_bpr = w * bpp;
    let padded_bpr = unpadded_bpr.div_ceil(align) * align;
    let out_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("coloriter.readback"),
        size: (padded_bpr * h) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut enc =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("coloriter.enc") });
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("coloriter.pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&color_pipeline);
        pass.set_bind_group(0, &color_bg, &[]);
        pass.draw(0..3, 0..1);
    }
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &color_tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &out_buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(h),
            },
        },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    queue.submit(std::iter::once(enc.finish()));

    let slice = out_buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = device.poll(wgpu::Maintain::Wait);
    rx.recv().map_err(|e| GpuError::Readback(e.to_string()))?.map_err(|e| GpuError::Readback(e.to_string()))?;

    let data = slice.get_mapped_range();
    let mut pixels = vec![0.0_f32; (w as usize) * (h as usize) * 4];
    let row_floats = (w * 4) as usize;
    for r in 0..h {
        let src = (r * padded_bpr) as usize;
        let src_row: &[f32] = bytemuck::cast_slice(&data[src..src + unpadded_bpr as usize]);
        let dst = (r as usize) * row_floats;
        pixels[dst..dst + row_floats].copy_from_slice(src_row);
    }
    drop(data);
    out_buf.unmap();

    Ok(ExportResult {
        width: w,
        height: h,
        ss: 1,
        pixels,
        iterate_ms: 0.0,
        color_ms: 0.0,
        counters: [0u64; crate::COUNTER_SLOTS],
        max_dispatch_ms: 0.0,
    })
}

/// Run the shader's primitive self-test (`fs_gputest`, see the WGSL) and return the raw
/// `(width, height, rgba_f32)` result grid: one op family per row, one hashed input set per
/// column, up to four f32 results per texel. The CPU comparison lives in the app (it needs the
/// f64 / exact-EFT oracles); this just executes the very shader code the renderer uses on THIS
/// device and reads it back.
pub fn gputest(device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(u32, u32, Vec<f32>), GpuError> {
    const W: u32 = 256;
    const H: u32 = 13;
    let shader = shader_module(device);
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("gputest.layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });
    let pipeline =
        fullscreen_pipeline(device, &shader, &layout, "fs_gputest", &[ITER_FORMAT], "gputest.pipeline");
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("gputest.out"),
        size: wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: ITER_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let bpp = 16u32;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let unpadded_bpr = W * bpp;
    let padded_bpr = unpadded_bpr.div_ceil(align) * align;
    let out_buf = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("gputest.readback"),
        size: (padded_bpr * H) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("gputest.enc"),
    });
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("gputest.pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&pipeline);
        pass.draw(0..3, 0..1);
    }
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &out_buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(H),
            },
        },
        wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 },
    );
    queue.submit(std::iter::once(enc.finish()));
    let slice = out_buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let _ = device.poll(wgpu::Maintain::Wait);
    rx.recv()
        .map_err(|e| GpuError::Readback(e.to_string()))?
        .map_err(|e| GpuError::Readback(e.to_string()))?;
    let data = slice.get_mapped_range();
    let mut pixels = vec![0.0_f32; (W * H * 4) as usize];
    for r in 0..H {
        let src = (r * padded_bpr) as usize;
        let row: &[f32] = bytemuck::cast_slice(&data[src..src + unpadded_bpr as usize]);
        pixels[(r * W * 4) as usize..((r + 1) * W * 4) as usize].copy_from_slice(row);
    }
    drop(data);
    out_buf.unmap();
    Ok((W, H, pixels))
}
