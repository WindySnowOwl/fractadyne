//! The self-test `view-rotation` (design/view-rotation.md): a view turned by θ, the picture turning
//! counterclockwise on the screen.
//!
//! The oracle is exact: a square view turned a quarter turn maps every texel to the numbers an
//! unturned texel maps to (`rotation_cos_sin` is exact there), so its lanes are the unturned lanes
//! turned, bit for bit — the counts and the distance as they are, the normal turned with them (it
//! is written in screen axes). On every arithmetic path: direct, df32 perturbation, floatexp with
//! BLA. At 30° and 45°, where nothing is exact, each sampled pixel's count against the CPU's at the
//! plane point the view's own mapping (`Viewport::pixel_to_complex`) puts under it: the GPU's
//! rotation and the CPU's are the same one. And the relief light stays fixed to the screen: a view
//! turned a quarter turn with the light on draws the unturned picture turned.

use super::SelfCheck;
use crate::FractalKind;
use fractadyne_core::Viewport;

/// The square frames' side.
const N: u32 = 160;

/// The lanes (count, normal x, normal y, distance) of `vp`, one sample a pixel; `tiled`, in tiles
/// of at most 48 px. With the tiles there were and how many iterated chunked (the control that the
/// path asked for is the path taken).
#[inline(never)]
fn lanes_counted(app: &mut crate::FractadyneApp, device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue, vp: &Viewport, tiled: bool) -> Option<(Vec<f32>, u32, u32)> {
    let mut r = Box::new(app.lanes_request_for(vp, false, None));
    (r.width, r.height, r.ss) = (N, N, 1);
    if tiled {
        r.tile_px_max = Some(48);
    }
    fractadyne_gpu::render_lanes(device, queue, &r, crate::tunables::CORRECT_WORK_BUDGET).ok().map(|x| (x.pixels, x.tiles_total, x.tiles_chunked))
}

/// The lanes of `vp` in one tile (see [`lanes_counted`]).
fn lanes(app: &mut crate::FractadyneApp, device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue, vp: &Viewport) -> Option<Vec<f32>> {
    lanes_counted(app, device, queue, vp, false).map(|x| x.0)
}

/// The colored frame of `vp` (RGBA, linear), one sample a pixel.
#[inline(never)]
fn colors(app: &mut crate::FractadyneApp, device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue, vp: &Viewport) -> Option<Vec<f32>> {
    let mut r = Box::new(app.current_export_request_for(vp, false));
    (r.width, r.height, r.ss) = (N, N, 1);
    let (progress, cancel) = (std::sync::atomic::AtomicU32::new(0), std::sync::atomic::AtomicBool::new(false));
    fractadyne_gpu::render_export(device, queue, &r, &progress, &cancel).ok().map(|x| x.pixels)
}

/// The texel `(x, y)` of a square frame turned by `quarters` quarter turns counterclockwise holds
/// the unturned frame's texel at the returned position (screen y down).
fn source_of(x: usize, y: usize, quarters: i32) -> (usize, usize) {
    let n = N as usize - 1;
    match quarters.rem_euclid(4) {
        0 => (x, y),
        1 => (n - y, x),
        2 => (n - x, n - y),
        _ => (y, n - x),
    }
}

/// A normal `(nx, ny)` (+y up) turned by `quarters` quarter turns counterclockwise, exactly.
fn turned(nx: f32, ny: f32, quarters: i32) -> (f32, f32) {
    match quarters.rem_euclid(4) {
        0 => (nx, ny),
        1 => (-ny, nx),
        2 => (-nx, -ny),
        _ => (ny, -nx),
    }
}

impl crate::FractadyneApp {
    /// A square view of `(re, im)` at `mag`, turned `deg`.
    fn vr_view(&self, re: &str, im: &str, mag: f64, deg: f64) -> Viewport {
        let at = |s: &str| fractadyne_core::parse_bf_prec(s, 512).expect("the view's coordinate");
        let mut vp = Viewport::new(N as f64, N as f64);
        vp.set_center_mag(at(re), at(im), mag);
        vp.rotation = deg;
        vp
    }

    /// The group's checks (see the module).
    pub(super) fn view_rotation_checks(&mut self, device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue) -> Vec<SelfCheck> {
        let saved = (
            (self.fractal, self.julia_mode, self.coloring.color_method, self.coloring.use_custom_palette),
            (self.effects.light, self.effects.light_angle, self.effects.de),
            (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.render_cfg.series_approx, self.render_cfg.use_bla),
        );
        self.fractal = FractalKind::Mandelbrot;
        self.julia_mode = false;
        self.coloring.color_method = crate::ColorMethod::Smooth;
        self.coloring.use_custom_palette = false;
        (self.effects.light, self.effects.de) = (false, false);
        (self.render_cfg.auto_iter, self.render_cfg.series_approx, self.render_cfg.use_bla) = (false, true, true);
        let mut checks = Vec::new();
        let sh = ("-0.7436438870371587047521915061147707", "0.131825904205311970493132056385139");
        let c07 = ("-1.178853950372678747911373866849720956148855", "0.1853420232408490265512092752061929308714979");
        // (name, center, magnification, iterations, in 48-px tiles, through a custom formula). The
        // four copies of the shader's prologue: `iterate_at` (the one-tile direct and df32 rows),
        // `fs_iterate_chunk` (the direct row past 400,000 iterations: `ChunkPricer` opens at
        // 400,000, so every tile chunks), `chunk_fe_at` (floatexp, whose step-bounded tiles always
        // chunk) and the custom module's own `chunk_fe_at`.
        let views: [(&str, (&str, &str), f64, u32, bool, bool); 5] = [
            ("direct, the seahorse at 2e3x", sh, 2e3, 2_000, false, false),
            ("df32, the seahorse at 1e6x", sh, 1e6, 4_000, false, false),
            ("direct chunked in tiles, the seahorse at 2e3x", sh, 2e3, 500_000, true, false),
            ("floatexp with BLA, corpus 07 at 1e30x", c07, 1e30, 25_000, false, false),
            ("a custom formula's floatexp, corpus 07 at 1e30x", c07, 1e30, 25_000, false, true),
        ];
        let saved_custom = self.custom.clone();
        for (name, (re, im), mag, iter, tiled, custom) in views {
            self.render_cfg.max_iter = iter;
            if custom {
                let Ok(f) = crate::custom_formula::CustomFormula::compile("z = z^2 + c", &[]) else {
                    checks.push(SelfCheck {
                        category: "consistency",
                        name: format!("a view turned by quarter turns is the unturned view's lanes turned, bit for bit ({name})"),
                        params: "z = z^2 + c".into(),
                        result: "the formula did not compile".into(),
                        threshold: "compiles",
                        pass: false,
                    });
                    continue;
                };
                self.custom = Some(std::sync::Arc::new(f));
                self.fractal = FractalKind::Custom;
            }
            let counted = lanes_counted(self, device, queue, &self.vr_view(re, im, mag, 0.0), tiled);
            let tiles = counted.as_ref().map_or((0, 0), |c| (c.1, c.2));
            let base = counted.map(|c| c.0);
            let mut results = Vec::new();
            for (deg, q) in [(90.0, 1), (180.0, 2), (-90.0, -1)] {
                let got = lanes_counted(self, device, queue, &self.vr_view(re, im, mag, deg), tiled).map(|c| c.0);
                let differ = match (&base, &got) {
                    (Some(b), Some(g)) if b.len() == g.len() => {
                        let mut bad = 0usize;
                        for y in 0..N as usize {
                            for x in 0..N as usize {
                                let (sx, sy) = source_of(x, y, q);
                                let (t, s) = (&g[(y * N as usize + x) * 4..][..4], &b[(sy * N as usize + sx) * 4..][..4]);
                                let (nx, ny) = turned(s[1], s[2], q);
                                let same = t[0].to_bits() == s[0].to_bits() && t[3].to_bits() == s[3].to_bits() && t[1] == nx && t[2] == ny;
                                bad += (!same) as usize;
                            }
                        }
                        Some(bad)
                    }
                    _ => None,
                };
                results.push((deg, differ));
            }
            let escaped = base.as_ref().map_or(0, |b| b.chunks_exact(4).filter(|t| t[0] >= 0.0).count());
            // The control: the path the row names is the path taken.
            let path_ok = match (tiled, mag > 1e20) {
                (true, _) => tiles.0 > 1 && tiles.1 == tiles.0,
                (false, true) => tiles.1 == tiles.0 && tiles.0 >= 1,
                (false, false) => tiles == (1, 0),
            };
            if custom {
                (self.custom, self.fractal) = (saved_custom.clone(), FractalKind::Mandelbrot);
            }
            checks.push(SelfCheck {
                category: "consistency",
                name: format!("a view turned by quarter turns is the unturned view's lanes turned, bit for bit ({name})"),
                params: format!("{N}x{N}, {iter} iterations; turned 90, 180 and -90 degrees; the count, the normal (turned with it) and the distance of every texel"),
                result: format!("(the turn, texels that differ): {results:?}; {escaped} escaped texels; (tiles, chunked) {tiles:?}"),
                threshold: "none differ at any turn; at least a tenth escaped; the row's path taken (one tile unchunked, or every tile chunked)",
                pass: results.iter().all(|(_, d)| *d == Some(0)) && escaped * 10 > (N * N) as usize && path_ok,
            });
        }
        // At 30 and 45 degrees: the counts at sampled pixels against the CPU's at the view's own
        // mapping of the pixel's center, with 0 as the control. On the overview: at the seahorse at
        // 2e3x a fifth of the pixels miss the CPU unturned too (the direct path's f32 orbit near the
        // boundary), which would hide a turn's error.
        self.render_cfg.max_iter = 2_000;
        for deg in [0.0, 30.0, 45.0] {
            let vp = self.vr_view("-0.6", "0.0", 1.0, deg);
            let got = lanes(self, device, queue, &vp);
            let agree = got.as_ref().map(|g| {
                let (mut read, mut bad) = (0usize, 0usize);
                for y in (3..N as usize).step_by(8) {
                    for x in (5..N as usize).step_by(8) {
                        let (cx, cy) = vp.complex_at_pixel_f64(x as f64 + 0.5, y as f64 + 0.5);
                        let pts = fractadyne_core::orbit_points((0.0, 0.0), (cx, cy), fractadyne_core::formula::MANDELBROT, 2_000, 256.0 * 256.0);
                        let (zx, zy) = *pts.last().unwrap();
                        let m2 = zx * zx + zy * zy;
                        let cpu = (m2 > 256.0 * 256.0).then(|| (pts.len() - 1) as f64 + 1.0 - (m2.ln() * 0.5 / 2f64.ln()).ln() / 2f64.ln());
                        let gpu = g[(y * N as usize + x) * 4];
                        read += 1;
                        let ok = match cpu {
                            Some(v) => gpu >= 0.0 && (f64::from(gpu) - v).abs() < 0.01,
                            None => gpu < 0.0,
                        };
                        bad += (!ok) as usize;
                    }
                }
                (read, bad)
            });
            checks.push(SelfCheck {
                category: "consistency",
                name: format!("a view turned {deg} degrees counts as the CPU at the view's own mapping"),
                params: format!("the overview (-0.6, 0) at 1x, {N}x{N}, 2000 iterations, every 8th pixel: the CPU's orbit (f64) at `complex_at_pixel_f64` of the pixel's center (0 degrees: the unturned control)"),
                result: format!("(pixels read, pixels off by 0.01 or in status): {agree:?}"),
                threshold: "at most 2% off",
                pass: agree.is_some_and(|(r, b)| r > 300 && b * 50 <= r),
            });
        }
        // The relief light stays fixed to the screen: lit, a quarter turn draws the unturned picture
        // turned (within the palette's anti-aliasing, whose footprint is the screen's neighbors).
        (self.effects.light, self.effects.de) = (true, true);
        self.effects.light_angle = 126.0;
        let lit0 = colors(self, device, queue, &self.vr_view(sh.0, sh.1, 2e3, 0.0));
        // The light turned with the picture: the same light on the turned surface.
        self.effects.light_angle = 216.0;
        let lit90 = colors(self, device, queue, &self.vr_view(sh.0, sh.1, 2e3, 90.0));
        let light = match (&lit0, &lit90) {
            (Some(a), Some(b)) if a.len() == b.len() => {
                let mut sum = 0.0f64;
                for y in 0..N as usize {
                    for x in 0..N as usize {
                        let (sx, sy) = source_of(x, y, 1);
                        for k in 0..3 {
                            sum += f64::from((b[(y * N as usize + x) * 4 + k] - a[(sy * N as usize + sx) * 4 + k]).abs()) * 255.0;
                        }
                    }
                }
                Some(sum / f64::from(3 * N * N))
            }
            _ => None,
        };
        checks.push(SelfCheck {
            category: "consistency",
            name: "the relief light is fixed to the screen: a turned view and light draw the picture turned".into(),
            params: format!("the seahorse at 2e3x, {N}x{N}: unturned with the light at 126 degrees, against turned 90 with the light at 216 (turned with it), the first turned"),
            result: format!("the mean channel difference in levels: {light:?}"),
            threshold: "at most 1 level",
            pass: light.is_some_and(|d| d <= 1.0),
        });
        checks.push(self.vr_file_check());
        (
            (self.fractal, self.julia_mode, self.coloring.color_method, self.coloring.use_custom_palette),
            (self.effects.light, self.effects.light_angle, self.effects.de),
            (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.render_cfg.series_approx, self.render_cfg.use_bla),
        ) = saved;
        checks
    }

    /// A turned view's file (format 7), through the real writer and reader: its rotation and the
    /// Julia pane's go out and come back; an unturned view writes no key and not format 7; a file
    /// without the key opens unturned; a value that does not read is unturned and reported.
    #[inline(never)]
    fn vr_file_check(&mut self) -> SelfCheck {
        let saved = (self.dual, self.julia_mode, self.viewport.clone(), self.julia_viewport.clone());
        self.fractal = FractalKind::Mandelbrot;
        (self.dual, self.julia_mode) = (true, false);
        self.viewport.rotation = 30.5;
        self.julia_viewport.rotation = -120.0;
        let turned = self.view_metadata();
        (self.viewport.rotation, self.julia_viewport.rotation, self.dual) = (0.0, 0.0, false);
        let rt = self.load_view_metadata(&turned);
        let back = (self.viewport.rotation, self.julia_viewport.rotation);
        let turned_ok = rt.note().is_none()
            && turned.contains("\nrotation=30.5\n")
            && turned.contains("\njulia_view_rotation=-120.0\n")
            && crate::meta_get(&turned, "format_version") == "7"
            && back == (30.5, -120.0);
        // Unturned: no key and not format 7; and a file without the key unturns the view.
        (self.viewport.rotation, self.julia_viewport.rotation, self.dual) = (0.0, 0.0, false);
        let plain = self.view_metadata();
        let plain_ok = !plain.contains("rotation") && crate::meta_get(&plain, "format_version") != "7";
        self.viewport.rotation = 45.0;
        let rp = self.load_view_metadata(&plain);
        let missing_ok = rp.note().is_none() && self.viewport.rotation.to_bits() == 0f64.to_bits();
        // Not a number: unturned, and said.
        self.viewport.rotation = 45.0;
        let rb = self.load_view_metadata(&turned.replace("\nrotation=30.5\n", "\nrotation=sideways\n"));
        let bad_ok = self.viewport.rotation == 0.0 && rb.clamped.contains(&"rotation");
        (self.dual, self.julia_mode, self.viewport, self.julia_viewport) = saved;
        SelfCheck {
            category: "View format",
            name: "a turned view's file carries its rotation".into(),
            params: "a dual view turned 30.5, its Julia pane -120 → scramble → load; an unturned view; a file without the key; rotation=sideways".into(),
            result: format!("turned round trip {turned_ok} (came back {back:?}), unturned writes none {plain_ok}, missing key unturns {missing_ok}, unreadable unturns and says so {bad_ok}"),
            threshold: "format 7 only when turned; both angles restored; a missing or unreadable key opens unturned",
            pass: turned_ok && plain_ok && missing_ok && bad_ok,
        }
    }
}
