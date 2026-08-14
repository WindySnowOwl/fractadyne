//! `--gputest` — verify the shader's df32/floatexp primitives against CPU oracles, per machine.
//!
//! The WGSL error-free transforms assume round-to-nearest with a FUSED fma; everything deep is
//! built on them, and until now they were verified only end-to-end by goldens blessed on one
//! RTX 3080. This harness runs `fs_gputest` (which calls the SAME shader functions the renderer
//! uses) on hash-derived inputs and checks every op family against an f64 or exact-EFT oracle —
//! so a driver/hardware where the assumption fails (fast-math fma contraction, flushed
//! denormals, wrong rounding) is caught as "two_prod residual wrong on THIS machine", not as an
//! unexplained black frame from a stranger's GPU. Row 9 (Julia-form accumulation, per-pixel
//! identity only in z₀) is the regression canary for the 2026-08-13 Julia precision class.
//!
//! Input construction uses integer hashing + bitcast only (no floating arithmetic), so this
//! module reproduces the shader's inputs bit-exactly.

use eframe::wgpu;

/// Mirrors WGSL `gt_hash` (wrapping u32 arithmetic — identical on both sides).
fn gt_hash(x0: u32) -> u32 {
    let mut x = x0.wrapping_mul(747796405).wrapping_add(2891336453);
    x = ((x >> ((x >> 28) + 4)) ^ x).wrapping_mul(277803737);
    (x >> 22) ^ x
}
/// Mirrors WGSL `gt_f32`: sign|exponent|mantissa from hash bits, exponent in [emin, emin+espan).
fn gt_f32(seed: u32, emin: i32, espan: u32) -> f32 {
    let mant = gt_hash(seed) & 0x007F_FFFF;
    let eb = (127 + emin + (gt_hash(seed ^ 0x9E37_79B9) % espan) as i32) as u32;
    let sign = (gt_hash(seed ^ 0x85EB_CA6B) & 1) << 31;
    f32::from_bits(sign | (eb << 23) | mant)
}
/// Mirrors WGSL `gt_df`: a NORMALIZED pair, `lo`'s exponent derived from `hi`'s actual exponent
/// so `|lo| <= ulp(hi)/2` (integer/bit math only — see the WGSL for why no EFT is used here).
fn gt_df(seed: u32, emin: i32, espan: u32) -> (f32, f32) {
    let hi = gt_f32(seed, emin, espan);
    let he = ((hi.to_bits() >> 23) & 0xFF) as i32 - 127;
    (hi, gt_f32(seed ^ 0xDEAD_BEEF, he - 25, 1))
}
fn df64(d: (f32, f32)) -> f64 {
    d.0 as f64 + d.1 as f64
}

struct OpCheck {
    name: &'static str,
    max_err: f64,
    tol: f64,
    fails: u32,
    detail: String,
}

/// One op row's verdict against its oracle. `err` is relative to the oracle magnitude (or the
/// input magnitude for cancellation-prone adds); EFT rows demand exactness and use fails only.
fn check_rows(w: u32, h: u32, px: &[f32]) -> Vec<OpCheck> {
    let texel = |ix: u32, op: u32| -> [f32; 4] {
        let base = ((op * w + ix) * 4) as usize;
        [px[base], px[base + 1], px[base + 2], px[base + 3]]
    };
    let mut out = Vec::new();
    let _ = h;

    // Rows 0/1 — the error-free transforms — and row 10, two_sum again but with bitcast armor
    // (see the WGSL `gt_opaque`): if plain two_sum fails while the armored one is exact, the
    // machine's compiler is REASSOCIATING float expressions (folding the residual away); if
    // both fail, the rounding itself is off. EXACT by theorem under round-to-nearest + real
    // fma: s must be fl(a+b) / fl(a·b) and e the exact residual (always representable in f32).
    for (op, name) in [
        (0u32, "two_sum (exact EFT)"),
        (1u32, "two_prod (exact EFT — fma canary)"),
        (10u32, "two_sum, bitcast-armored (reassociation discriminator)"),
        (11u32, "quick_two_sum (exact EFT — df_mul/df_div depend on it)"),
    ] {
        let mut fails = 0u32;
        let mut detail = String::new();
        for ix in 0..w {
            let s = ix + 100003 * op;
            let (mut a, mut b) = (gt_f32(s * 2 + 1, -8, 16), gt_f32(s * 2 + 2, -8, 16));
            if op == 11 && a.abs() < b.abs() {
                std::mem::swap(&mut a, &mut b); // quick_two_sum requires |a| >= |b|
            }
            let got = texel(ix, op);
            let (es, ee) = if op == 1 {
                let fp = a * b;
                (fp, (a as f64) * (b as f64) - fp as f64)
            } else {
                let fs = a + b;
                (fs, (a as f64 + b as f64) - fs as f64)
            };
            if got[0] != es || got[1] as f64 != ee {
                fails += 1;
                if detail.is_empty() {
                    detail = format!(
                        "first: a={a:e} b={b:e} got=({:e},{:e}) want=({es:e},{ee:e})",
                        got[0], got[1]
                    );
                }
            }
        }
        out.push(OpCheck { name, max_err: 0.0, tol: 0.0, fails, detail });
    }

    // Generic relative-error rows.
    struct Rel {
        op: u32,
        name: &'static str,
        tol: f64,
    }
    for r in [
        Rel { op: 2, name: "df_add", tol: 2f64.powi(-42) },
        Rel { op: 3, name: "df_mul", tol: 2f64.powi(-42) },
        Rel { op: 4, name: "df_div", tol: 2f64.powi(-40) },
        Rel { op: 5, name: "c_sqr (complex square)", tol: 2f64.powi(-40) },
        Rel { op: 6, name: "fe_mul (floatexp)", tol: 2f64.powi(-38) },
        Rel { op: 7, name: "fe_add (floatexp align/renorm)", tol: 2f64.powi(-38) },
        Rel { op: 8, name: "64-step df32 accumulation (Mandelbrot form)", tol: 2f64.powi(-33) },
        Rel { op: 9, name: "64-step df32 accumulation (Julia form, z0-carried)", tol: 2f64.powi(-33) },
    ] {
        let mut max_err = 0f64;
        let mut fails = 0u32;
        let mut detail = String::new();
        for ix in 0..w {
            let s = ix + 100003 * r.op;
            let ar = gt_df(s * 8 + 3, -4, 8);
            let ai = gt_df(s * 8 + 4, -4, 8);
            let br = gt_df(s * 8 + 5, -4, 8);
            let bi = gt_df(s * 8 + 6, -4, 8);
            let got = texel(ix, r.op);
            // (oracle value(s), got value(s), magnitude scale for the relative error)
            let (checks, scale): (Vec<(f64, f64)>, f64) = match r.op {
                2 => {
                    let v = df64(ar) + df64(br);
                    (vec![(v, got[0] as f64 + got[1] as f64)], df64(ar).abs() + df64(br).abs())
                }
                3 => {
                    let v = df64(ar) * df64(br);
                    (vec![(v, got[0] as f64 + got[1] as f64)], v.abs())
                }
                4 => {
                    let v = df64(ar) / df64(br);
                    (vec![(v, got[0] as f64 + got[1] as f64)], v.abs())
                }
                5 => {
                    let (re, im) = (df64(ar), df64(ai));
                    let (vr, vi) = (re * re - im * im, 2.0 * re * im);
                    let mag = re * re + im * im;
                    (
                        vec![
                            (vr, got[0] as f64 + got[1] as f64),
                            (vi, got[2] as f64 + got[3] as f64),
                        ],
                        mag.max(1e-30),
                    )
                }
                6 => {
                    let ea = (gt_hash(s ^ 17) % 200) as i32 - 100;
                    let eb2 = (gt_hash(s ^ 23) % 200) as i32 - 100;
                    let (are, aim) = (df64(ar), df64(ai));
                    let (bre, bim) = (df64(br), df64(bi));
                    let sc = 2f64.powi(ea + eb2);
                    let vr = (are * bre - aim * bim) * sc;
                    let vi = (are * bim + aim * bre) * sc;
                    let e_out = got[2] as i32;
                    let gre = (got[0] as f64 + got[1] as f64) * 2f64.powi(e_out);
                    let gim = got[3] as f64 * 2f64.powi(e_out);
                    let mag = (are.abs() + aim.abs()) * (bre.abs() + bim.abs()) * sc;
                    // im has only the hi limb in the output — looser check for it.
                    (
                        vec![(vr, gre), (vi * 2f64.powi(-24), gim * 2f64.powi(-24))],
                        mag.max(1e-300),
                    )
                }
                7 => {
                    let ea = (gt_hash(s ^ 31) % 80) as i32 - 40;
                    let (are, aim) = (df64(ar), df64(ai));
                    let (bre, bim) = (df64(br), df64(bi));
                    let sa = 2f64.powi(ea);
                    let vr = are * sa + bre;
                    let vi = aim * sa + bim;
                    let e_out = got[2] as i32;
                    let gre = (got[0] as f64 + got[1] as f64) * 2f64.powi(e_out);
                    let gim = got[3] as f64 * 2f64.powi(e_out);
                    let mag = are.abs() * sa + bre.abs() + aim.abs() * sa + bim.abs();
                    (
                        vec![(vr, gre), (vi * 2f64.powi(-24), gim * 2f64.powi(-24))],
                        mag.max(1e-300),
                    )
                }
                8 | 9 => {
                    // Mirror the contracted recurrence z ← z²·2⁻¹ + c in f64.
                    let cr8 = gt_df(s * 8 + 3, -3, 3);
                    let ci8 = gt_df(s * 8 + 4, -3, 3);
                    let (cre, cim, mut zr, mut zi) = if r.op == 8 {
                        (df64(cr8), df64(ci8), 0f64, 0f64)
                    } else {
                        (-0.7436f32 as f64, 0.1318f32 as f64, df64(cr8), df64(ci8))
                    };
                    for _ in 0..64 {
                        let (qr, qi) = (zr * zr - zi * zi, 2.0 * zr * zi);
                        zr = qr * 0.5 + cre;
                        zi = qi * 0.5 + cim;
                    }
                    let mag = zr.abs().max(zi.abs()).max(1e-6);
                    (
                        vec![
                            (zr, got[0] as f64 + got[1] as f64),
                            (zi, got[2] as f64 + got[3] as f64),
                        ],
                        mag,
                    )
                }
                _ => (vec![], 1.0),
            };
            for (want, got_v) in checks {
                let err = (got_v - want).abs() / scale.max(1e-300);
                if err > max_err {
                    max_err = err;
                    if err > r.tol {
                        detail = format!("worst ix={ix}: want {want:e} got {got_v:e}");
                    }
                }
                if err > r.tol {
                    fails += 1;
                }
            }
        }
        out.push(OpCheck { name: r.name, max_err, tol: r.tol, fails, detail });
    }
    out
}

/// Grade one device's results and print the verdict table. Returns failing op-family count.
fn report(w: u32, h: u32, px: &[f32]) -> usize {
    println!("  {:<52} {:>12} {:>10} {:>6}", "op", "max rel err", "tolerance", "");
    let checks = check_rows(w, h, px);
    let mut failed = 0usize;
    for c in &checks {
        let ok = c.fails == 0;
        if !ok {
            failed += 1;
        }
        let verdict = if ok { "PASS" } else { "FAIL" };
        if c.tol == 0.0 {
            println!(
                "  {:<52} {:>12} {:>10} {:>6}",
                c.name,
                if ok { "exact".to_string() } else { format!("{} wrong", c.fails) },
                "exact",
                verdict
            );
        } else {
            println!("  {:<52} {:>12.2e} {:>10.1e} {:>6}", c.name, c.max_err, c.tol, verdict);
        }
        if !ok && !c.detail.is_empty() {
            println!("      {}", c.detail);
        }
    }
    if failed == 0 {
        println!("  All {} op families within tolerance.", checks.len());
    } else {
        println!(
            "  {failed} op FAMILY(IES) FAILED — the shader's extended-precision arithmetic does \
             not hold on this stack."
        );
    }
    failed
}

/// Block on a wgpu native future. Adapter/device requests resolve immediately on native, so a
/// busy poll with a no-op waker is sufficient (and keeps the crate free of an async runtime).
fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    let mut fut = std::pin::pin!(fut);
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    loop {
        match fut.as_mut().poll(&mut cx) {
            std::task::Poll::Ready(v) => return v,
            std::task::Poll::Pending => std::thread::yield_now(),
        }
    }
}

/// `--gputest`: run the shader's primitive self-test on EVERY backend this machine offers.
///
/// Headless by construction — the test renders to an offscreen texture, so it needs no window and
/// no surface. That matters twice over: it runs on a build server or over SSH, and it can reach
/// backends the windowed path cannot (DX12 surface creation fails under this app's eframe setup,
/// which is why the 2026-08-13 folded-EFT investigation could not get a DX12 data point).
///
/// Sweeping rather than testing one device is the whole point: the arithmetic is compiled by the
/// BACKEND's shader compiler (DX12 → HLSL via FXC/DXC, Vulkan → SPIR-V, GL → GLSL), so "is df32
/// real on this machine" can have a different answer per backend, and the comparison localizes
/// the blame to a specific compiler instead of "the GPU".
///
/// Returns the number of backends that failed (0 = every available backend is sound).
pub(crate) fn run_gputest_sweep() -> usize {
    let candidates: [(wgpu::Backends, &str); 3] = [
        (wgpu::Backends::DX12, "DX12"),
        (wgpu::Backends::VULKAN, "Vulkan"),
        (wgpu::Backends::GL, "OpenGL"),
    ];
    // Which backends exist in THIS BINARY is a build-time fact (wgpu gates each behind a Cargo
    // feature), and it is not obvious from the outside: a backend that was never compiled in
    // looks exactly like a missing GPU at runtime. State it, so a report that omits a backend
    // says which of the two happened.
    let compiled = wgpu::Instance::enabled_backend_features();
    println!(
        "Fractadyne GPU primitive self-test — {} · sweeping every available backend\n\
         Verifies the renderer's own df32/floatexp helpers against CPU oracles. The error-free\n\
         transforms (two_sum/two_prod) must be EXACT: they are what makes df32 more than f32.\n\
         Backends compiled into this binary: {compiled:?}\n",
        crate::version_string()
    );
    let mut ran = 0usize;
    let mut failed = 0usize;
    for (backends, label) in candidates {
        if !compiled.contains(backends) {
            println!("{label}: not compiled into this binary — skipped\n");
            continue;
        }
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });
        let Some(adapter) = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })) else {
            println!("{label}: no adapter — skipped\n");
            continue;
        };
        let info = adapter.get_info();
        // Downlevel (GL) adapters cannot satisfy the default limits; ask for what they have.
        let limits = wgpu::Limits::default().using_resolution(adapter.limits());
        let dev = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("gputest.device"),
                required_features: wgpu::Features::empty(),
                required_limits: limits,
                memory_hints: wgpu::MemoryHints::default(),
            },
            None,
        ));
        let (device, queue) = match dev {
            Ok(d) => d,
            Err(e) => {
                println!("{label}: device request failed ({e}) — skipped\n");
                continue;
            }
        };
        println!("── {label} · {} · driver: {} {}", info.name, info.driver, info.driver_info);
        ran += 1;
        match fractadyne_gpu::gputest(&device, &queue) {
            Ok((w, h, px)) => {
                if report(w, h, &px) > 0 {
                    failed += 1;
                }
            }
            Err(e) => {
                println!("  GPU run failed: {e}");
                failed += 1;
            }
        }
        println!();
    }
    if ran == 0 {
        println!("No usable backend found — nothing tested.");
        return 1;
    }
    if failed == 0 {
        println!("{ran} backend(s) tested, all sound.");
    } else {
        println!(
            "{failed} of {ran} backend(s) FAILED. Include this whole report in a bug report — a\n\
             failing two_sum means the shader compiler is folding the error-free transforms, and\n\
             every extended-precision path silently degrades to plain f32."
        );
    }
    failed
}
