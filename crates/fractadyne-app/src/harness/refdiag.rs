//! `--refdiag`: reference orbit lengths sampled across a view - do long (interior) references exist here? Dispatched from `cli::run_headless`.

pub(crate) fn run(args: &[String]) -> bool {
    use fractadyne_core as fc;
    let val = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1));
    let two = |name: &str| args.iter().position(|a| a == name).and_then(|i| Some((args.get(i + 1)?, args.get(i + 2)?)));
    let (cx, cy) = two("--center")
        .map(|(x, y)| crate::arg_center("--center", x, y))
        .unwrap_or((fc::BigFloat::from_f64(-0.5, 64), fc::BigFloat::from_f64(0.0, 64)));
    let log2mag = val("--zoom-log2")
        .map(|s| crate::arg_parse::<f64>("--zoom-log2", s, "a number"))
        .unwrap_or(133.0);
    let max_iter = val("--iter")
        .map(|s| crate::arg_parse::<u32>("--iter", s, "a whole number"))
        .unwrap_or(60_000);
    let p = fc::precision_for_octaves(log2mag.ceil().max(0.0) as u64);
    let mut vp = fc::Viewport::new(1000.0, 1000.0);
    vp.set_center_log2mag(cx, cy, log2mag);
    println!("refdiag @ 1e{:.1}x, precision {p} bits, max_iter {max_iter}", log2mag / std::f64::consts::LOG2_10);
    // Sample an 11x11 grid across the view; report each point's reference orbit length.
    let n = 11usize;
    let mut lens: Vec<u32> = Vec::new();
    let zero = fc::BigFloat::from_f64(0.0, p);
    for j in 0..n {
        for i in 0..n {
            let px = (i as f64 / (n as f64 - 1.0)) * 1000.0;
            let py = (j as f64 / (n as f64 - 1.0)) * 1000.0;
            let (rx, ry) = vp.pixel_to_complex(px, py);
            let (_orbit, len) = fc::reference_orbit(&zero, &zero, &rx, &ry, 0, max_iter, p);
            lens.push(len);
        }
    }
    lens.sort_unstable();
    let total = lens.len();
    let interior = lens.iter().filter(|&&l| l >= max_iter).count();
    let median = lens[total / 2];
    let maxl = *lens.last().unwrap();
    let minl = *lens.first().unwrap();
    // Histogram buckets by orbit length.
    let buckets = [1000u32, 4000, 8000, 16000, 32000, u32::MAX];
    let mut hist = [0usize; 6];
    for &l in &lens {
        for (bi, &b) in buckets.iter().enumerate() {
            if l < b { hist[bi] += 1; break; }
        }
    }
    println!("  {total} points: min={minl} median={median} max={maxl} interior(>=max_iter)={interior}");
    println!("  orbit-length histogram: <1k={} <4k={} <8k={} <16k={} <32k={} >=32k={}",
        hist[0], hist[1], hist[2], hist[3], hist[4], hist[5]);
    let best = fc::best_reference(&[
        {let (x,_)=vp.pixel_to_complex(500.0,500.0); x},
        {let (_,y)=vp.pixel_to_complex(500.0,500.0); y},
    ], [vp.complex_span_fe().0, vp.complex_span_fe().1], 0, false, [0.0,0.0], max_iter, p);
    let (_bo, blen) = fc::reference_orbit(&zero, &zero, &best[0], &best[1], 0, max_iter, p);
    println!("  best_reference orbit length = {blen}  (if << max_iter, pixels rebase past it)");
    true
}
