//! TEMPORARY diagnostic (not committed): build a reference orbit exactly as the app does and
//! profile the dynamic range of the STORED (df32) samples — hunting f32 underflow of near-zero
//! orbit passes (the ~1e142x uniform-exterior investigation).

#[test]
fn probe_orbit_profile() {
    let Ok(spec) = std::env::var("PROBE_ORBIT") else {
        return; // only runs when pointed at a location
    };
    // spec: "label|cx|cy|iters|prec"
    let parts: Vec<&str> = spec.split('|').collect();
    let (label, cx, cy, iters, prec) = (
        parts[0],
        parts[1],
        parts[2],
        parts[3].parse::<u32>().unwrap(),
        parts[4].parse::<usize>().unwrap(),
    );
    let cx = fractadyne_core::parse_bf(cx).expect("cx");
    let cy = fractadyne_core::parse_bf(cy).expect("cy");
    let zero = fractadyne_core::BigFloat::from_f64(0.0, prec);
    let (orbit, len) =
        fractadyne_core::reference_orbit(&zero, &zero, &cx, &cy, 0, iters, prec);

    let mut ext_markers = 0u32;
    for s in orbit.iter().take(len as usize) {
        if s[0].is_nan() { ext_markers += 1; }
    }
    println!("[probe-ext] {label}: NaN-marked extended samples = {ext_markers}");
    let mut min_nz = f64::MAX;
    let mut min_iter = 0usize;
    let (mut zeros, mut sub38, mut sub20, mut sub10) = (0u32, 0u32, 0u32, 0u32);
    let mut first_dips: Vec<(usize, f64)> = Vec::new();
    for (i, s) in orbit.iter().enumerate().take(len as usize) {
        let x = s[0] as f64 + s[2] as f64;
        let y = s[1] as f64 + s[3] as f64;
        let m = (x * x + y * y).sqrt();
        if m == 0.0 {
            zeros += 1;
            if first_dips.len() < 8 {
                first_dips.push((i, 0.0));
            }
            continue;
        }
        if m < 1.2e-38 {
            sub38 += 1;
        }
        if m < 1e-20 {
            sub20 += 1;
        }
        if m < 1e-10 {
            sub10 += 1;
            if first_dips.len() < 8 {
                first_dips.push((i, m));
            }
        }
        if m < min_nz {
            min_nz = m;
            min_iter = i;
        }
    }
    println!(
        "[probe] {label}: len={len} zeros={zeros} sub38={sub38} sub20={sub20} sub10={sub10} \
         min_nonzero={min_nz:.3e}@{min_iter} dips={first_dips:?}"
    );

    // TRUE dip magnitudes: the f64-stage samples (before the df32 split flushes them). Reveals the
    // real |Z| at the periodic near-zero passes that the stored orbit records as exactly 0.
    let f64_orbit = fractadyne_core::reference_orbit_f64(&cx, &cy, iters, prec);
    let mut true_min = f64::MAX;
    let mut true_min_i = 0usize;
    let mut dip_mags: Vec<(usize, f64)> = Vec::new();
    for (i, (x, y)) in f64_orbit.iter().enumerate() {
        let m = (x * x + y * y).sqrt();
        if m < 1e-50 && dip_mags.len() < 12 {
            dip_mags.push((i, m));
        }
        if m < true_min && i > 0 {
            true_min = m;
            true_min_i = i;
        }
    }
    println!("[probe-f64] {label}: true_min={true_min:.3e}@{true_min_i} first_true_dips={dip_mags:?}");
}
