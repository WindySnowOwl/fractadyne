use super::*;

/// A heart, as the trap-shape study drew it (160 points on the classic parametric heart).
fn heart() -> Vec<[f64; 2]> {
    let n = 160;
    (0..=n)
        .map(|i| {
            let t = std::f64::consts::TAU * i as f64 / n as f64;
            let x = 16.0 * t.sin().powi(3);
            let y = 13.0 * t.cos() - 5.0 * (2.0 * t).cos() - 2.0 * (3.0 * t).cos() - (4.0 * t).cos();
            [x, y]
        })
        .collect()
}

/// A deterministic spread of points over and past the grid's domain.
fn probes() -> Vec<[f64; 2]> {
    let mut s = 0x2545_f491_4f6c_dd1du64;
    let mut next = || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        (s >> 11) as f64 / (1u64 << 53) as f64
    };
    (0..4000).map(|_| [next() * 5.0 - 2.5, next() * 5.0 - 2.5]).collect()
}

#[test]
fn the_grid_finds_the_exact_distance_inside_its_domain() {
    let o = Outline::from_paths(&[heart()]).unwrap();
    let mut checked = 0;
    for p in probes() {
        if p[0].abs() <= DOMAIN && p[1].abs() <= DOMAIN {
            let (g, b) = (o.dist(p[0], p[1]), o.dist_brute(p[0], p[1]));
            assert!((g - b).abs() <= 1e-12, "at {p:?}: grid {g}, every segment {b}");
            checked += 1;
        }
    }
    assert!(checked > 2000, "{checked} probes inside the domain");
}

#[test]
fn outside_the_domain_the_distance_is_an_upper_bound_close_to_the_exact_one() {
    let o = Outline::from_paths(&[heart()]).unwrap();
    for p in probes() {
        if p[0].abs() > DOMAIN || p[1].abs() > DOMAIN {
            let (g, b) = (o.dist(p[0], p[1]), o.dist_brute(p[0], p[1]));
            assert!(g >= b - 1e-12, "at {p:?}: {g} below the exact {b}");
            assert!(g <= b * 1.5, "at {p:?}: {g} far above the exact {b}");
        }
    }
}

#[test]
fn the_outline_fits_the_unit_circle_centered_on_its_box() {
    let o = Outline::from_paths(&[heart()]).unwrap();
    let (mut x0, mut y0, mut x1, mut y1, mut r) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN, 0.0f64);
    for s in &o.segs {
        for p in [[s[0], s[1]], [s[2], s[3]]] {
            (x0, y0, x1, y1) = (x0.min(p[0]), y0.min(p[1]), x1.max(p[0]), y1.max(p[1]));
            r = r.max(p[0].hypot(p[1]));
        }
    }
    assert!(((x0 + x1) / 2.0).abs() < 1e-4 && ((y0 + y1) / 2.0).abs() < 1e-4, "box center ({}, {})", (x0 + x1) / 2.0, (y0 + y1) / 2.0);
    assert!((r - 1.0).abs() < 1e-4, "farthest point {r}");
}

#[test]
fn the_text_round_trips_to_the_same_outline() {
    let o = Outline::from_paths(&[heart(), vec![[0.0, 0.0], [3.0, 1.0]]]).unwrap();
    let back = Outline::from_text(o.text()).unwrap();
    assert_eq!(back.text(), o.text());
    assert_eq!(back.segs, o.segs);
    assert_eq!(back.key(), o.key());
    assert!(o.text().starts_with("M ") && o.text().matches(" M ").count() == 1, "{}", &o.text()[..60]);
}

#[test]
fn untrusted_text_is_refused_in_words() {
    let bad = [
        ("", OutlineError::Empty),
        ("M 0 0", OutlineError::Empty),
        ("M 0 0 L 0 0", OutlineError::Empty),
        ("L 0 0", OutlineError::Malformed("a line before the first M".into())),
        ("M 0 0 L 2 0", OutlineError::Malformed("2 is outside the unit frame".into())),
        ("M 0 0 L NaN 0", OutlineError::Malformed("NaN is outside the unit frame".into())),
        ("M 0 0 L x 0", OutlineError::Malformed("\"x\" is not a number".into())),
        ("M 0 0 C 1 1", OutlineError::Malformed("\"C\" is neither M nor L".into())),
        ("M 0 0 L 0.5", OutlineError::Malformed("a coordinate is missing after segment 0".into())),
    ];
    for (text, want) in bad {
        assert_eq!(Outline::from_text(text).unwrap_err(), want, "{text:?}");
    }
    let long = "M 0 0".repeat(MAX_TEXT / 5 + 1);
    assert!(matches!(Outline::from_text(&long), Err(OutlineError::TooLong(_))));
    let mut many = String::from("M -1 0");
    for i in 0..=MAX_SEGMENTS {
        many.push_str(&format!(" L {} {}", i as f64 / MAX_SEGMENTS as f64 - 0.5, (i % 2) as f64 * 0.1));
    }
    assert!(matches!(Outline::from_text(&many), Err(OutlineError::TooManySegments(_))));
    for s in ["M 1 0 L 1e400 0", "M 0 0 L inf 0"] {
        assert!(Outline::from_text(s).is_err(), "{s}");
    }
}

#[test]
fn a_ring_s_center_cell_keeps_every_segment_within_the_cap() {
    // A circle flattened as the importer would (sagitta 0.002 of the unit radius): about 50 segments,
    // all at one distance from the center cell, which must then list them all.
    let n = 50;
    let ring: Vec<[f64; 2]> = (0..=n).map(|i| {
        let t = std::f64::consts::TAU * i as f64 / n as f64;
        [t.cos(), t.sin()]
    }).collect();
    let o = Outline::from_paths(&[ring]).unwrap();
    let (cx, cy) = cell_of(0.01, 0.01);
    assert_eq!(o.cells[cy * GRID + cx].1 as usize, n, "the center cell's list");
    assert!(o.longest_cell() <= MAX_CELL);
    for p in [[0.01, 0.01], [-0.02, 0.015], [0.0, 0.0]] {
        assert!((o.dist(p[0], p[1]) - o.dist_brute(p[0], p[1])).abs() < 1e-12);
    }
}

#[test]
fn the_gpu_data_holds_the_same_cells_indices_and_segments() {
    let o = Outline::from_paths(&[heart()]).unwrap();
    let (v, l) = o.gpu_data();
    assert_eq!(l.len as usize, v.len());
    assert_eq!(l.idx as usize, GRID * GRID / 2);
    assert_eq!((l.segs - l.idx) as usize, o.idx.len().div_ceil(4));
    assert_eq!((l.len - l.segs) as usize, o.segs.len());
    // Every cell, back from the packed form, and its first index and segment.
    for (c, &(start, count)) in o.cells.iter().enumerate() {
        let q = v[c / 2];
        let (s, n) = if c % 2 == 0 { (q[0], q[1]) } else { (q[2], q[3]) };
        assert_eq!((s as u32, n as u32), (start, count), "cell {c}");
        if count > 0 {
            let k = v[l.idx as usize + start as usize / 4][start as usize % 4] as usize;
            assert_eq!(k, o.idx[start as usize] as usize);
            assert_eq!(v[l.segs as usize + k][0], o.segs[k][0] as f32);
        }
    }
}
