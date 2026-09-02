use super::{BenchDepth, STD_FRAMES};

/// Frames per regime for a preset, computed exactly the way `step_standard_bench` computes
/// the magnification it renders at - via `regime_index`, which asks `RenderMode::select`
/// rather than restating any threshold.
fn regime_frames(depth: BenchDepth) -> [u32; 3] {
    let z = depth.zoom_log10();
    let frames = STD_FRAMES.max(2) as i32;
    let mut c = [0u32; 3];
    for i in 0..frames {
        let frac = i as f64 / (frames - 1) as f64;
        let log2mag = frac * z * std::f64::consts::LOG2_10;
        c[super::regime_index(log2mag.exp2())] += 1;
    }
    c
}

/// No preset may report `floatexp 0`. Both did, for a long time: measured at 720p,
/// Standard (1 -> 1e12x) was 20/40/0 and Ultra (1 -> 1e28x) was 9/51/0 - Ultra's endpoint
/// WAS the threshold and computing it through `exp2` landed a hair below, so neither
/// rendered a single floatexp frame. BLA, which `bla_eligible` gates on floatexp, had
/// therefore never executed in a benchmark whose settings block prints "BLA on". This is
/// the guard against that returning, by a re-sized dive or by a moved threshold.
#[test]
fn every_preset_crosses_into_floatexp() {
    for depth in BenchDepth::ALL {
        let c = regime_frames(depth);
        assert!(
            c.iter().all(|&n| n >= 5),
            "{}: split {c:?} - every regime needs real frames, not a token one",
            depth.label()
        );
    }
    // A ladder, not two names for the same dive: deeper preset, more of the deep path.
    assert!(
        regime_frames(BenchDepth::Ultra)[2] > regime_frames(BenchDepth::Standard)[2],
        "ultra must spend more frames in floatexp than standard"
    );
}

/// A centre is only usable as deep as its digit count supports, and `begin_standard_bench`
/// falls back to (-0.5, 0) when a centre fails to parse - which would dive somewhere else
/// entirely and still print a complete, plausible report. Sub-pixel placement at 1eN needs
/// about N + log10(width in px) digits, so require N + 4.
#[test]
fn every_preset_centre_parses_and_is_precise_enough_for_its_depth() {
    for depth in BenchDepth::ALL {
        let (cx, cy, site) = depth.center();
        assert!(!site.is_empty(), "{}: unnamed dive site", depth.label());
        for coord in [cx, cy] {
            assert!(
                fractadyne_core::parse_bf(coord).is_some(),
                "{site}: centre does not parse: {coord}"
            );
            let digits = coord.chars().filter(|c| c.is_ascii_digit()).count() as f64;
            assert!(
                digits > depth.zoom_log10() + 4.0,
                "{site}: {digits} digits is too coarse for a 1e{} dive",
                depth.zoom_log10()
            );
        }
    }
}

/// Every token the help text and the CLI advertise must actually select something, and the
/// labels must stay distinct - a picker with two identical entries is unusable.
#[test]
fn depth_tokens_round_trip_and_labels_are_distinct() {
    for (tok, want) in [
        ("standard", BenchDepth::Standard),
        ("1e32", BenchDepth::Standard),
        ("ultra", BenchDepth::Ultra),
        ("all", BenchDepth::Ultra),
        ("all-regimes", BenchDepth::Ultra),
        ("1e48", BenchDepth::Ultra),
    ] {
        assert_eq!(BenchDepth::from_token(tok), Some(want), "token {tok:?}");
    }
    // Tokens for endpoints no dive lands on any more must FAIL, not quietly resolve to
    // the nearest preset: `--depth` is fatal on an unreadable value, so an old command
    // line stops rather than reporting a different workload under a familiar name.
    for gone in ["1e12", "12", "1e28", "28", "nonsense"] {
        assert_eq!(BenchDepth::from_token(gone), None, "token {gone:?} should be refused");
    }
    let mut labels: Vec<&str> = BenchDepth::ALL.iter().map(|d| d.label()).collect();
    labels.sort_unstable();
    let n = labels.len();
    labels.dedup();
    assert_eq!(labels.len(), n, "two depth presets share a label");
}
