use super::progressive_frame_order;

#[test]
fn progressive_order_is_an_exact_permutation() {
    // The property `--resume` and mp4 assembly rest on: every frame exactly once.
    for (first, last, kfs) in [
        (0u64, 0u64, vec![]),
        (0, 8, vec![]),
        (0, 232, vec![0, 45, 135, 232]),
        (17, 41, vec![3, 20, 20, 99]), // out-of-range + duplicate seeds must not corrupt it
    ] {
        let order = progressive_frame_order(first, last, &kfs);
        assert_eq!(order.len() as u64, last - first + 1, "wrong count for kfs={kfs:?}");
        let mut sorted = order.clone();
        sorted.sort_unstable();
        assert_eq!(
            sorted,
            (first..=last).collect::<Vec<_>>(),
            "not a permutation for [{first},{last}] kfs={kfs:?}"
        );
    }
}

#[test]
fn progressive_order_bisects_coarse_to_fine() {
    // No keyframes: endpoints, then the classic ½ → ¼,¾ → ⅛… refinement.
    assert_eq!(progressive_frame_order(0, 8, &[]), vec![0, 8, 4, 2, 6, 1, 3, 5, 7]);
    // Keyframe seeds lead, in time order, before any bisection.
    let order = progressive_frame_order(0, 100, &[60, 30]);
    assert_eq!(&order[..4], &[0, 30, 60, 100]);
}

/// Deterministic xorshift64*. A fixed seed keeps any failure reproducible, and rolling it
/// by hand keeps the workspace free of dev-dependencies (it has none, on purpose).
pub(super) struct Rng(pub u64);
impl Rng {
    pub(super) fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// Uniform-enough in `0..n` for a property sweep; `n == 0` yields 0.
    pub(super) fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            0
        } else {
            self.next() % n
        }
    }
}

#[test]
fn progressive_order_is_a_permutation_for_any_input() {
    // The test above pins four shapes; this quantifies over the input space, which is where
    // `--resume` and mp4 assembly actually live. A duplicated frame silently re-renders (at
    // 4K that is hours), and a missing one leaves a hole `--resume` cannot know about and the
    // encoder turns into a visible jump.
    let mut r = Rng(0x9E37_79B9_7F4A_7C15);
    for case in 0..4000u32 {
        let lo = r.below(500);
        let span = r.below(400);
        // Every 7th case passes the endpoints BACKWARDS: the function swaps them and callers
        // rely on that, so the property has to hold for reversed input too.
        let (a, b) = if case % 7 == 0 { (lo + span, lo) } else { (lo, lo + span) };
        let n_kf = r.below(9);
        // Drawn from a wider span than the range, so most cases carry out-of-range seeds.
        let kfs: Vec<u64> = (0..n_kf).map(|_| r.below(1000)).collect();
        let order = progressive_frame_order(a, b, &kfs);
        let (first, last) = (a.min(b), a.max(b));
        assert_eq!(order.len() as u64, last - first + 1, "count: [{a},{b}] kfs={kfs:?}");
        let mut sorted = order.clone();
        sorted.sort_unstable();
        assert_eq!(
            sorted,
            (first..=last).collect::<Vec<_>>(),
            "not a permutation: [{a},{b}] kfs={kfs:?}"
        );
    }
}

#[test]
fn progressive_order_depends_only_on_the_seed_set() {
    // Scripts list keyframes in authoring order, with repeats, and `--segment` hands over
    // slices whose keyframes fall outside the range. Order must depend only on the SET of
    // in-range seeds (it is collected through a BTreeSet). If it depended on spelling, two
    // runs of the same tour could disagree on frame order and `--resume` would "fill" frames
    // that were never missing while leaving the real gaps.
    let mut r = Rng(0xDEAD_BEEF_CAFE_1234);
    for _ in 0..1500 {
        let first = r.below(50);
        let last = first + r.below(200);
        let n = r.below(7);
        let base: Vec<u64> = (0..n).map(|_| first + r.below(220)).collect();
        let canonical = progressive_frame_order(first, last, &base);
        let mut noisy: Vec<u64> = base.iter().rev().copied().collect();
        noisy.extend(base.iter().copied()); // reversed AND duplicated — same set
        assert_eq!(
            progressive_frame_order(first, last, &noisy),
            canonical,
            "order changed with keyframe spelling: [{first},{last}] {base:?}"
        );
    }
}

#[test]
fn progressive_order_leads_with_every_in_range_seed() {
    // The early-look contract, and the whole reason the mode exists: keyframes plus the two
    // endpoints render BEFORE any bisection, in time order, so a deep frame can be inspected
    // minutes into a render instead of at the end.
    let mut r = Rng(0x0123_4567_89AB_CDEF);
    for _ in 0..1500 {
        let first = r.below(40);
        let last = first + 1 + r.below(200);
        let n = r.below(6);
        let kfs: Vec<u64> = (0..n).map(|_| first + r.below(260)).collect();
        let mut seeds: Vec<u64> =
            kfs.iter().copied().filter(|f| (first..=last).contains(f)).collect();
        seeds.push(first);
        seeds.push(last);
        seeds.sort_unstable();
        seeds.dedup();
        let order = progressive_frame_order(first, last, &kfs);
        assert_eq!(
            &order[..seeds.len()],
            &seeds[..],
            "seeds did not lead: [{first},{last}] kfs={kfs:?}"
        );
    }
}

// ---- Tour scripts are UNTRUSTED input: they are the artifact people share on a forum ----

/// A hostile `zoom` must be REFUSED, not turned into an allocation. Every one of these
/// reaches `precision_for_octaves`, which sizes the BigFloat precision that every centre in
/// the tour is parsed at; before the bound in `parse_zoom_log10`, the infinite cases
/// saturated the octave count and asked astro-float for a usize::MAX-bit number, killing the
/// process during load. Regression pins for that class.
#[test]
fn hostile_zoom_is_refused_not_allocated() {
    for z in [
        "1e1e999",             // exponent itself overflows f64 -> inf log10
        "1e999999999999",      // finite but absurd: ~3.3e12 octaves ~ 415 GB per centre
        "1E1E999",             // same, upper case
        "-1e1e999",            // negative mantissa (already refused, kept as a pin)
        "1e-1e999",            // -inf log10
        "inf",
        "NaN",
    ] {
        assert!(
            super::parse_zoom_log10(z).is_err(),
            "hostile zoom {z:?} accepted — this sizes a BigFloat allocation"
        );
    }
    // Depths that are real must still pass: the deepest verified corpus location, the
    // extreme-zoom battery, a zoomed-OUT view, and the documented ceiling itself.
    for z in ["4.6e1105", "1e21000", "6.5e94", "0.5", "1e1000000"] {
        assert!(super::parse_zoom_log10(z).is_ok(), "legitimate zoom {z:?} refused");
    }
    // Pin the MECHANISM the bound exists to stop, without allocating it: an unbounded log10
    // flows into this exact arithmetic, and a float→int cast SATURATES rather than wrapping,
    // so the octave count lands at u64::MAX and the requested precision at usize::MAX bits.
    // That is the allocation the loader used to attempt on a hostile tour.
    let octaves = (f64::INFINITY / std::f64::consts::LN_2).max(0.0).ceil() as u64;
    assert_eq!(octaves, u64::MAX, "cast no longer saturates — revisit the bound's rationale");
    assert_eq!(fractadyne_core::precision_for_octaves(octaves), usize::MAX);
}

/// Random and mutated tour text must never panic the parser — only `Ok` or a `String` error.
/// Structured formats fail differently from flat ones: the mutation pass below starts from a
/// VALID script, so it reaches the resolve step (cross-references between keyframes,
/// locations, palettes and segments) that purely random bytes never get past.
#[test]
fn fuzz_parse_tour_text_panic_free() {
    let mut s = 0xda3e_39cb_94b3_c83fu64;
    let mut next = || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        s
    };
    // Tokens drawn from the real grammar, so random input lands on plausible structures far
    // more often than uniform bytes would.
    let toks: [&str; 26] = [
        "[[keyframe]]", "[[location]]", "[[palette]]", "[[segment]]", "[[annotation]]",
        "[render]", "[playback]", "format_version", "t", "zoom", "re", "im", "id", "hold",
        "ease", "location", "palette", "max_iter", "=", "\n", "\"", "0", "-1", "1e999",
        "1e1e999", "\u{0}",
    ];
    for _ in 0..4_000 {
        let n = (next() % 24) as usize;
        let mut buf = String::new();
        for _ in 0..n {
            buf.push_str(toks[(next() as usize) % toks.len()]);
            if next() % 3 == 0 {
                buf.push('\n');
            }
        }
        let _ = super::parse_tour_text(&buf);
    }
    // Byte-level mutation of a VALID script — reaches deserialize + resolve.
    let seed = "format_version = 2\n\
                [[location]]\nid = \"a\"\nre = \"-0.75\"\nim = \"0.1\"\nzoom = 1000.0\n\
                [[keyframe]]\nid = \"k0\"\nt = 0\nlocation = \"a\"\n\
                palette = \"Ember\"\nmax_iter = 500\nhold = 1\n\
                [[keyframe]]\nid = \"k1\"\nt = 5\nre = \"-0.75\"\nim = \"0.1\"\n\
                zoom = \"1e30\"\n\
                [[segment]]\nid = \"s\"\ntitle = \"S\"\nt = 0\n";
    if let Err(e) = super::parse_tour_text(seed) {
        panic!("seed script must be valid, else the mutation pass never reaches resolve: {e}");
    }
    let base = seed.as_bytes();
    for _ in 0..4_000 {
        let mut b = base.to_vec();
        for _ in 0..1 + next() % 4 {
            if b.is_empty() {
                break; // a truncation to nothing ends this sample (and guards the modulo)
            }
            let at = (next() as usize) % b.len();
            match next() % 3 {
                0 => b[at] = (next() % 256) as u8,
                1 => b[at] = toks[(next() as usize) % toks.len()].as_bytes()[0],
                _ => b.truncate(at),
            }
        }
        if let Ok(text) = std::str::from_utf8(&b) {
            let _ = super::parse_tour_text(text);
        }
    }
}
