use super::*;

/// The segments an SVG document's paths draw, in order, with their colours: what a reader of the
/// file gets back.
fn read_back(svg: &str) -> Vec<([f64; 2], [f64; 2], String)> {
    let mut out = Vec::new();
    for line in svg.lines().filter(|l| l.starts_with("<path ")) {
        let stroke = line.split("stroke=\"").nth(1).unwrap().split('"').next().unwrap().to_string();
        let d = line.split(" d=\"").nth(1).unwrap().split('"').next().unwrap();
        let mut pts = Vec::new();
        for part in d.split(['M', 'L']).filter(|t| !t.trim().is_empty()) {
            let v: Vec<f64> = part.split_whitespace().map(|n| n.parse().unwrap()).collect();
            pts.push([v[0], v[1]]);
        }
        for w in pts.windows(2) {
            out.push((w[0], w[1], stroke.clone()));
        }
    }
    out
}

fn seg(a: [f32; 2], b: [f32; 2], value: f32) -> SegmentInstance {
    SegmentInstance { a, b, value }
}

/// Two colours by palette value: below ½ red, else blue.
fn two(v: f32) -> [u8; 3] {
    if v < 0.5 {
        [255, 0, 0]
    } else {
        [0, 0, 255]
    }
}

#[test]
fn the_file_reads_back_as_the_segment_list() {
    // A chain of three that meet (one path), a colour change mid-chain (a second path), and a jump
    // (a third) — in the view's y-up pixels about its centre.
    let segs = [
        seg([-50.0, 0.0], [-25.0, 10.5], 0.1),
        seg([-25.0, 10.5], [0.0, 0.0], 0.2),
        seg([0.0, 0.0], [25.25, -10.0], 0.7),
        seg([30.0, 30.0], [40.125, 30.0], 0.8),
    ];
    let svg = document(&Picture {
        size: [200, 100],
        width: 1.5,
        segments: &segs,
        polygons: &[],
        colour: &two,
        background: [5, 5, 8],
        title: "test",
    });
    assert_eq!(svg.matches("<path ").count(), 3, "{svg}");
    let back = read_back(&svg);
    assert_eq!(back.len(), segs.len());
    for (g, (a, b, stroke)) in segs.iter().zip(&back) {
        // SVG's y-down, from the corner, to 0.01 px.
        let want_a = [f64::from(g.a[0]) + 100.0, 50.0 - f64::from(g.a[1])];
        let want_b = [f64::from(g.b[0]) + 100.0, 50.0 - f64::from(g.b[1])];
        for (p, q) in [(a, want_a), (b, want_b)] {
            assert!((p[0] - q[0]).abs() <= 0.005 && (p[1] - q[1]).abs() <= 0.005, "{p:?} vs {q:?}");
        }
        assert_eq!(*stroke, hex(two(g.value)));
    }
    assert!(svg.contains(r#"stroke-width="1.5""#));
    assert!(svg.contains(r##"fill="#050508""##), "the background");
}

#[test]
fn filled_shapes_go_under_the_lines() {
    let poly = vec![(vec![[-10.0, -10.0], [10.0, -10.0], [0.0, 10.0]], 0.9)];
    let svg = document(&Picture {
        size: [40, 40],
        width: 1.0,
        segments: &[seg([-20.0, 0.0], [20.0, 0.0], 0.0)],
        polygons: &poly,
        colour: &two,
        background: [0, 0, 0],
        title: "a & b <c>",
    });
    let (pg, pa) = (svg.find("<polygon").unwrap(), svg.find("<path").unwrap());
    assert!(pg < pa, "the shapes first");
    assert!(svg.contains(r##"<polygon points="10,30 30,30 20,10" fill="#0000ff"/>"##), "{svg}");
    assert!(svg.contains("<title>a &amp; b &lt;c&gt;</title>"));
}

#[test]
fn numbers_are_written_short_and_exact_to_a_hundredth() {
    assert_eq!(num(1234), "12.34");
    assert_eq!(num(700), "7");
    assert_eq!(num(-5), "-0.05");
    assert_eq!(num(-150), "-1.5");
    assert_eq!(num(0), "0");
}
