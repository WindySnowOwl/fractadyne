use super::*;

fn ctx() -> AnchorContext {
    AnchorContext {
        script_digest: script_digest("format_version = 2\n[[keyframe]]\nt = 0\n"),
        fps: 30.0,
        frames: 181,
        base_iter: 500_000,
        base_auto: true,
        total: 6.0,
    }
}

/// A distribution over `[lo, hi]` with awkward f32 knots (a measurement's kind), never decreasing.
fn distribution(lo: f32, hi: f32) -> fractadyne_core::coloring::Quantiles {
    let n = fractadyne_core::coloring::EQ_QUANTILES;
    let mut q = [0.0f32; fractadyne_core::coloring::EQ_QUANTILES + 1];
    for (i, k) in q.iter_mut().enumerate() {
        let s = i as f64 / n as f64;
        *k = (lo as f64 + (hi as f64 - lo as f64) * s.powf(2.7)) as f32;
    }
    q[n] = hi;
    q
}

/// Ranges with awkward f32 values — the kind a measurement produces — at keyframe frames, each
/// with its distribution.
fn anchors(c: &AnchorContext) -> Vec<Anchor> {
    [(0u64, (0.1f32, 1234.567f32)), (90, (3.3333333, 98765.43)), (180, (f32::MIN_POSITIVE, 1.0e30))]
        .into_iter()
        .map(|(frame, range)| Anchor { frame, t: frame_time(frame, c.fps, c.total), range, eq: Some(distribution(range.0, range.1)) })
        .collect()
}

/// ⭐The property the file exists for: what one machine measured, another applies BIT FOR BIT.
#[test]
fn anchors_round_trip_exactly() {
    let c = ctx();
    let a = anchors(&c);
    let text = encode(&c, "0.3.0-beta.18", "gabc1234", &a);
    let d = decode(&text, &c).expect("own file must read back");
    assert_eq!(d.anchors.len(), a.len());
    for (got, want) in d.anchors.iter().zip(&a) {
        assert_eq!(got.frame, want.frame);
        assert_eq!(got.t.to_bits(), want.t.to_bits());
        assert_eq!(got.range.0.to_bits(), want.range.0.to_bits(), "lo drifted at frame {}", want.frame);
        assert_eq!(got.range.1.to_bits(), want.range.1.to_bits(), "hi drifted at frame {}", want.frame);
        let (g, w) = (got.eq.expect("version 2 carries the distribution"), want.eq.unwrap());
        assert!(g.iter().zip(&w).all(|(a, b)| a.to_bits() == b.to_bits()), "the distribution drifted at frame {}", want.frame);
    }
    assert_eq!((d.app_version.as_str(), d.git.as_str()), ("0.3.0-beta.18", "gabc1234"));
    assert!(text.contains("version = 2"), "written as version 2");
}

/// A file of ranges alone (version 1, as builds before Equalize colors reached tours wrote it) still
/// reads, without distributions; and anchors without them are written so.
#[test]
fn a_ranges_only_file_reads_without_distributions() {
    let c = ctx();
    let bare: Vec<Anchor> = anchors(&c).into_iter().map(|a| Anchor { eq: None, ..a }).collect();
    let text = encode(&c, "v", "g", &bare);
    assert!(text.contains("version = 1") && !text.contains("quantiles"), "{text}");
    let d = decode(&text, &c).expect("a version-1 file reads");
    assert_eq!(d.anchors, bare);
}

/// A damaged distribution is refused, never repaired into one nobody measured.
#[test]
fn a_damaged_distribution_is_refused() {
    let c = ctx();
    let good = encode(&c, "v", "g", &anchors(&c));
    let line = good.lines().find(|l| l.starts_with("quantiles = ")).expect("a distribution line").to_string();
    let words: Vec<&str> = line["quantiles = \"".len()..line.len() - 1].split(' ').collect();
    let with = |w: Vec<String>| good.replacen(&line, &format!("quantiles = \"{}\"", w.join(" ")), 1);
    let own = || words.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let mut short = own();
    short.pop();
    let mut long = own();
    long.push("7f000000".into());
    let mut swapped = own();
    swapped.swap(10, 200);
    let mut nan = own();
    nan[5] = "7fc00000".into();
    let mut word = own();
    word[3] = "zz".into();
    let bad = [
        with(short),
        with(long),
        with(swapped),
        with(nan),
        with(word),
        good.replacen(&format!("{line}\n"), "", 1),        // a version-2 anchor without it
        good.replace("version = 2", "version = 1"),          // distributions in a version-1 file
        good.replace("version = 2", "version = 3"),
    ];
    for (i, text) in bad.iter().enumerate() {
        assert_ne!(text, &good, "case {i}: the damage did not apply");
        assert!(decode(text, &c).is_err(), "case {i} was accepted");
    }
}

/// An all-interior tour measures nothing — and that, too, is a file the reader accepts.
#[test]
fn an_empty_anchor_list_round_trips() {
    let c = ctx();
    let d = decode(&encode(&c, "v", "g", &[]), &c).expect("empty is valid");
    assert!(d.anchors.is_empty());
}

/// The same script copied between machines with different line endings is the same script.
#[test]
fn the_script_digest_ignores_line_endings_and_nothing_else() {
    let lf = "format_version = 2\nname = \"x\"\n";
    assert_eq!(script_digest(lf), script_digest("format_version = 2\r\nname = \"x\"\r\n"));
    assert_eq!(script_digest(lf), script_digest("format_version = 2\rname = \"x\"\r"));
    assert_ne!(script_digest(lf), script_digest("format_version = 2\nname = \"y\"\n"));
}

/// Every way the file can describe a DIFFERENT render is refused, naming what differed.
#[test]
fn a_file_for_another_render_is_refused_with_the_reason() {
    let c = ctx();
    let text = encode(&c, "v", "g", &anchors(&c));
    let cases: Vec<(AnchorContext, &str)> = vec![
        (AnchorContext { script_digest: c.script_digest ^ 1, ..c.clone() }, "different script"),
        (AnchorContext { fps: 24.0, ..c.clone() }, "fps"),
        (AnchorContext { frames: 145, ..c.clone() }, "frames"),
        (AnchorContext { base_iter: 2000, ..c.clone() }, "iteration base"),
        (AnchorContext { base_auto: false, ..c.clone() }, "iteration base"),
    ];
    for (want, needle) in cases {
        let e = decode(&text, &want).expect_err("must refuse");
        assert!(e.contains(needle), "refusal for {needle} said: {e}");
    }
}

/// Hand-edited or damaged content is refused, never "repaired" into a palette nobody measured.
#[test]
fn damaged_content_is_refused() {
    let c = ctx();
    let good = encode(&c, "v", "g", &anchors(&c));
    let bad = [
        good.replace("format = \"fractadyne-norm-anchors\"", "format = \"other\""),
        good.replace("version = 2", "version = 3"),
        good.replace("frame = 90", "frame = 9000"),       // past the end
        good.replace("frame = 90", "frame = 0"),          // not increasing
        good.replace("frame = 90", "frame = 91"),         // time no longer matches
        good.replace("lo = 0.10000000149011612", "lo = 0.1"), // not an f32 value
        good.replace("hi = 1234.5670166015625", "hi = 0.0"),  // lo > hi
        good.replace("base_auto = true", "base_auto = true\nsurprise = 1"), // unknown key
        "not toml at all [[[".to_string(),
    ];
    for (i, text) in bad.iter().enumerate() {
        assert_ne!(text, &good, "case {i}: the damage did not apply");
        assert!(decode(text, &c).is_err(), "case {i} was accepted:\n{text}");
    }
}

/// Times come from the frame index by the renderer's own formula, clamped at the tour's end.
#[test]
fn frame_time_matches_the_renderer_and_clamps() {
    assert_eq!(frame_time(0, 30.0, 6.0), 0.0);
    assert_eq!(frame_time(90, 30.0, 6.0), 3.0);
    assert_eq!(frame_time(500, 30.0, 6.0), 6.0);
    assert_eq!(frame_time(7, 30.0, 0.0), 0.0); // a zero-length tour is one still frame
}
