use super::*;

/// The outline's segments' endpoints, for checking shapes.
fn points(o: &Outline) -> Vec<[f64; 2]> {
    o.text()
        .split(['M', 'L'])
        .filter_map(|t| {
            let v: Vec<f64> = t.split_whitespace().filter_map(|n| n.parse().ok()).collect();
            (v.len() == 2).then(|| [v[0], v[1]])
        })
        .collect()
}

#[test]
fn a_path_with_curves_and_arcs_flattens_into_the_unit_circle() {
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
        <path d="M 50 90 C 10 60, 0 30, 25 15 Q 40 5, 50 25 Q 60 5, 75 15 C 100 30, 90 60, 50 90 Z"/>
        <path d="M 30 50 A 20 20 0 0 1 70 50"/>
    </svg>"#;
    let o = outline_from_svg(svg).unwrap();
    assert!(o.segment_count() > 20 && o.segment_count() < 600, "{} segments", o.segment_count());
    let r = points(&o).iter().map(|p| p[0].hypot(p[1])).fold(0.0f64, f64::max);
    assert!((r - 1.0).abs() < 1e-4, "farthest {r}");
}

#[test]
fn transforms_compose_down_the_tree_and_y_is_flipped_up() {
    // A unit square in the corner, moved by a group's translate and its own scale: the outline is the
    // same square whatever the transforms (normalized), but a lone segment to the right of a point
    // below it must stay to the right and come out ABOVE after the flip.
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg">
        <g transform="translate(100 0)"><g transform="scale(2)">
            <line x1="0" y1="10" x2="10" y2="10"/>
            <line x1="0" y1="0" x2="0" y2="0.5"/>
        </g></g>
    </svg>"#;
    let o = outline_from_svg(svg).unwrap();
    let p = points(&o);
    // The long line (y = 10 in the file, the lowest on the page) is the bottom one after the flip.
    let ys: Vec<f64> = p.iter().map(|q| q[1]).collect();
    let (lo, hi) = (ys.iter().cloned().fold(f64::MAX, f64::min), ys.iter().cloned().fold(f64::MIN, f64::max));
    let long: Vec<&[f64; 2]> = p.iter().filter(|q| (q[1] - lo).abs() < 1e-6).collect();
    assert_eq!(long.len(), 2, "the long line's two ends at the bottom: {p:?}");
    assert!(hi > lo);
}

#[test]
fn basic_shapes_are_read_and_defs_text_and_use_are_not() {
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg">
        <defs><path d="M 0 0 L 1000 1000"/></defs>
        <text x="0" y="0">ignored</text>
        <use href="#nothing"/>
        <rect x="0" y="0" width="10" height="10"/>
        <circle cx="5" cy="5" r="3"/>
        <ellipse cx="5" cy="5" rx="4" ry="2"/>
        <polygon points="1,1 9,1 5,8"/>
        <polyline points="0,0 10,10"/>
    </svg>"##;
    let o = outline_from_svg(svg).unwrap();
    // The defs path would have stretched the frame to 1000: the square's corner must sit near radius 1.
    let r = points(&o).iter().map(|p| p[0].hypot(p[1])).fold(0.0f64, f64::max);
    assert!((r - 1.0).abs() < 1e-4, "{r}");
    assert!(o.segment_count() > 4 + 3 + 2 + 1, "{}", o.segment_count());
}

#[test]
fn untrusted_files_are_refused_in_words() {
    let cases: [(&[u8], &str); 9] = [
        (b"", "not an SVG drawing"),
        (b"\xff\xfe<svg/>", "not UTF-8"),
        (b"<html><body/></html>", "its root is not <svg>"),
        (b"<svg xmlns='http://www.w3.org/2000/svg'><text>hi</text></svg>", "no paths or shapes"),
        (
            b"<?xml version='1.0'?><!DOCTYPE svg [<!ENTITY a 'aaaaaaaaaa'><!ENTITY b '&a;&a;&a;&a;&a;'>]><svg>&b;</svg>",
            "not an SVG drawing",
        ),
        (b"<svg xmlns='http://www.w3.org/2000/svg'><path d='M 0 0 L 1e309 0'/></svg>", "cannot read"),
        // A finite transform of a finite point that overflows: the drawing's own check.
        (b"<svg xmlns='http://www.w3.org/2000/svg'><path d='M 0 0 L 10 0' transform='scale(1e308)'/></svg>", "not a number"),
        (b"<svg xmlns='http://www.w3.org/2000/svg'><path d='M 0 0 L 5 5' transform='scale(1e309)'/></svg>", "cannot read"),
        (b"<svg xmlns='http://www.w3.org/2000/svg'><rect width='50%' height='10'/></svg>", "no paths or shapes"),
    ];
    for (svg, want) in cases {
        let e = outline_from_svg(svg).map(|_| ()).unwrap_err();
        assert!(e.contains(want), "{:?}: {e}", String::from_utf8_lossy(svg));
    }
    let big = vec![b' '; MAX_SVG_BYTES + 1];
    assert!(outline_from_svg(&big).unwrap_err().contains("at most"));
}

#[test]
fn deep_nesting_does_not_overflow_the_stack() {
    let mut svg = String::from("<svg xmlns='http://www.w3.org/2000/svg'>");
    for _ in 0..20_000 {
        svg.push_str("<g>");
    }
    svg.push_str("<line x1='0' y1='0' x2='1' y2='1'/>");
    for _ in 0..20_000 {
        svg.push_str("</g>");
    }
    svg.push_str("</svg>");
    // XML that deep may be refused by the parser itself; either way, no overflow and no panic.
    let _ = outline_from_svg(svg.as_bytes());
}

#[test]
fn a_drawing_with_too_many_lines_is_simplified_to_fit() {
    // 300 circles: some 15,000 segments at the default tolerance; coarser until it fits.
    let mut svg = String::from("<svg xmlns='http://www.w3.org/2000/svg'>");
    for i in 0..300 {
        svg.push_str(&format!("<circle cx='{}' cy='{}' r='3'/>", (i % 20) * 10, (i / 20) * 10));
    }
    svg.push_str("</svg>");
    let o = outline_from_svg(svg.as_bytes()).unwrap();
    assert!(o.segment_count() <= MAX_SEGMENTS && o.segment_count() > 1000, "{}", o.segment_count());
}

/// A view's keys as the reader sees them: `key=value` lines.
fn reader(lines: &str) -> impl Fn(&str) -> Option<String> + '_ {
    move |k: &str| lines.lines().find_map(|l| l.strip_prefix(k).and_then(|r| r.strip_prefix('=')).map(str::to_string))
}

/// The orbit trap method with `shape`, and an outline.
fn spec_with(shape: &str) -> fractadyne_state::ColoringSpec {
    fractadyne_state::ColoringSpec {
        color_method: "trap".into(),
        trap_type: shape.into(),
        trap_outline: "M -1 0 L 1 0 L 0 1".into(),
        trap_outline_name: "tri.svg".into(),
        ..Default::default()
    }
}

#[test]
fn a_view_carries_the_outline_only_when_a_trap_it_uses_is_one() {
    let lines = outline_view_lines(&spec_with("outline"));
    assert_eq!(lines, "trap_outline=M -1 0 L 1 0 L 0 1\ntrap_outline_name=tri.svg\n");
    // Another shape: nothing, so every other view is as before.
    assert_eq!(outline_view_lines(&spec_with("star")), "");
    // Another method: the trap is not drawn, nothing.
    assert_eq!(outline_view_lines(&fractadyne_state::ColoringSpec { color_method: "smooth".into(), ..spec_with("outline") }), "");
    // The inside's own trap is the outline: carried.
    let inside = fractadyne_state::ColoringSpec { interior: "trap".into(), interior_trap: "outline".into(), ..spec_with("star") };
    assert!(outline_view_lines(&inside).starts_with("trap_outline=M -1 0"), "{}", outline_view_lines(&inside));
    // A name with a line break (a Linux file name) stays one line.
    let odd = fractadyne_state::ColoringSpec { trap_outline_name: "a\nb.svg".into(), ..spec_with("outline") };
    assert!(outline_view_lines(&odd).ends_with("trap_outline_name=a b.svg\n"), "{:?}", outline_view_lines(&odd));
}

#[test]
fn a_view_s_outline_reads_back_and_a_bad_one_is_said() {
    // Written, then read onto a session with another outline: the view's.
    let lines = outline_view_lines(&spec_with("outline"));
    let mut s = fractadyne_state::ColoringSpec { trap_outline: "M 0 0 L 0.5 0".into(), ..spec_with("outline") };
    let mut problems = Vec::new();
    apply_outline_view(&mut s, &reader(&lines), true, &mut problems);
    assert_eq!((s.trap_outline.as_str(), s.trap_outline_name.as_str(), problems.len()), ("M -1 0 L 1 0 L 0 1", "tri.svg", 0));
    // UNTRUSTED text that is not an outline: said, and the trap is left without one.
    let mut s = spec_with("outline");
    apply_outline_view(&mut s, &reader("trap_outline=M 0 0 L 9 9\n"), true, &mut problems);
    assert_eq!((s.trap_outline.as_str(), problems.len()), ("", 1), "{problems:?}");
    // The view's trap is the outline and it states its coloring but carries none: said, cleared.
    let mut s = spec_with("outline");
    apply_outline_view(&mut s, &reader("cycle=1\n"), true, &mut problems);
    assert_eq!((s.trap_outline.as_str(), problems.len()), ("", 2), "{problems:?}");
    // A hand-written location (no stated coloring) keeps the session's.
    let mut s = spec_with("outline");
    apply_outline_view(&mut s, &reader("center_re=0\n"), false, &mut problems);
    assert_eq!(s.trap_outline, "M -1 0 L 1 0 L 0 1");
    // A view whose traps are not the outline leaves the session's alone, even past a bad one.
    let mut s = spec_with("star");
    apply_outline_view(&mut s, &reader("trap_outline=garbage\n"), true, &mut problems);
    assert_eq!((s.trap_outline.as_str(), problems.len()), ("M -1 0 L 1 0 L 0 1", 3));
}
