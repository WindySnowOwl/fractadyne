use super::frame::{distance, signature, signature_distance};

fn noise(len: usize, mut seed: u32) -> Vec<u8> {
    (0..len)
        .map(|_| {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            (seed >> 24) as u8
        })
        .collect()
}

/// ⭐**The palette check's shortcut is only as good as its bound.** Two signatures are never
/// farther apart than the frames they came from, so a pair the signatures call different is
/// different, at the self-test's frame size and at odd ones.
#[test]
fn a_signature_distance_never_exceeds_the_frames() {
    for (len, runs, seed) in [(480 * 270 * 4, 4800, 1), (4200, 7, 2), (64, 64, 3), (480 * 270 * 4, 4800, 4)] {
        let (a, b) = (noise(len, seed), noise(len, seed + 100));
        let (sa, sb) = (signature(&a, runs), signature(&b, runs));
        assert_eq!(sa.len(), runs, "{len} bytes in {runs} runs");
        let (s, d) = (signature_distance(&sa, &sb), distance(&a, &b));
        assert!(s <= d + 1e-9, "{len}/{runs}: signature {s} > frames {d}");
    }
}

/// The bound is tight where it should be (a uniform shift moves every run's mean by as much as
/// its bytes) and blind inside a run (a run's bytes reordered keep its mean): the check on
/// signatures can only be stricter than the one on frames, never looser.
#[test]
fn a_signature_is_exact_for_a_shift_and_blind_inside_a_run() {
    let runs = 4800;
    let a: Vec<u8> = noise(480 * 270 * 4, 9).iter().map(|v| v / 2).collect();
    let up: Vec<u8> = a.iter().map(|v| v + 10).collect();
    let d = signature_distance(&signature(&a, runs), &signature(&up, runs));
    assert!((d - 10.0).abs() < 1e-3 && (distance(&a, &up) - 10.0).abs() < 1e-9, "a shift of 10 reads {d}");
    let run = a.len() / runs;
    let mut shuffled = a.clone();
    for c in shuffled.chunks_exact_mut(run) {
        c.reverse();
    }
    assert!(distance(&a, &shuffled) > 1.0, "the reordered frame differs");
    assert!(signature_distance(&signature(&a, runs), &signature(&shuffled, runs)) < 1e-3, "its signature does not");
    assert_eq!(signature_distance(&signature(&a, runs), &signature(&a, runs)), 0.0);
}

/// A frame shorter than the run count is its own signature, byte for byte.
#[test]
fn a_frame_shorter_than_the_runs_is_its_own_signature() {
    let a = noise(10, 5);
    assert_eq!(signature(&a, 100), a.iter().map(|&v| v as f32).collect::<Vec<_>>());
}
