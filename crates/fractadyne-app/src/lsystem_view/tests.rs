use super::*;

#[test]
fn the_status_readouts_keep_their_width() {
    let (o1, s1) = status_readouts(7, Some(12));
    let (o2, s2) = status_readouts(4096, Some(999_999_999));
    assert_eq!(o1.chars().count(), o2.chars().count());
    assert_eq!(s1.chars().count(), s2.chars().count());
    let (_, s3) = status_readouts(7, None);
    assert_eq!(s1.chars().count(), s3.chars().count(), "a walk not yet landed keeps the width");
}

#[test]
fn nesting_counts_the_deepest_bracket() {
    let s = LSystem::parse("angle 20\naxiom X\nX = F[+X[-X]]F[-X]+X\n").unwrap();
    assert_eq!(nesting(&s.axiom), 0);
    assert_eq!(nesting(s.rule(b'X').unwrap()), 2);
}

#[test]
fn the_default_system_is_in_the_library() {
    let st = LSystemState::default();
    assert_eq!(st.system.name, DEFAULT_SYSTEM);
    assert!(st.tables.grows());
}

#[test]
fn walk_values_stay_in_the_palette_range() {
    // Every colouring writes a value in [0, 1] (the shader reads < 0 as "no line").
    // In f64 at 100 px a unit, and deep at 2^200 px a unit (the dragon at order ~400, 2^400
    // segments: an index past f64's integers).
    let st = LSystemState::default();
    let zero = || BigFloat::from_f64(0.0, 128);
    for (upp_log2, deep) in [(0.01f64.log2(), false), (-200.0, true)] {
        let order = st.order_for(upp_log2);
        assert_eq!(st.needs_deep(upp_log2, order), deep, "at 2^{upp_log2} px a unit");
        let mut big = None;
        for colouring in Colouring::ALL {
            let key = WalkKey {
                tables: 1,
                order,
                centre: [zero(), zero()],
                upp_log2,
                size: [400, 300],
                colouring,
                margin: 1.0,
                deep,
            };
            let WalkOut { segments: segs, stats, big: used, .. } = walk_segments(&st.drawn_system(), &st.tables, big.clone(), None, &key, 4.0);
            assert_eq!(used.is_some(), deep);
            big = used;
            assert!(stats.segments > 0 && segs.len() as u64 == stats.segments, "{colouring:?} deep {deep}: {stats:?}");
            for s in &segs {
                assert!((0.0..=1.0).contains(&s.value), "{colouring:?}: {}", s.value);
            }
        }
    }
}

#[test]
fn a_stochastic_plant_reseeds_and_zooms_deep() {
    let mut st = LSystemState::default();
    st.set_system(library::find("Stochastic plant").unwrap().system().unwrap());
    assert!(st.tables.grows());
    let (id, n) = (st.tables_id, st.tables.axiom_entry(6).n);
    st.set_seed(st.system.seed + 1);
    assert_ne!(st.tables_id, id, "a new seed rebuilds the tables");
    assert_ne!(st.tables.axiom_entry(6).n, n, "another seed, another plant");
    // At its base, 2^60 px a unit: deep, and the stem is there.
    let upp_log2 = -60.0;
    let order = st.order_for(upp_log2);
    assert!(st.needs_deep(upp_log2, order));
    let zero = || BigFloat::from_f64(0.0, 128);
    let key = WalkKey {
        tables: 1,
        order,
        centre: [zero(), zero()],
        upp_log2,
        size: [400, 300],
        colouring: Colouring::Depth,
        margin: 1.0,
        deep: true,
    };
    let out = walk_segments(&st.drawn_system(), &st.tables, None, None, &key, 4.0);
    assert!(out.big.is_some());
    assert!(out.stats.segments > 0 && !out.stats.stopped, "{:?}", out.stats);
}

#[test]
fn a_parametric_system_is_built_once_at_its_order_and_drawn() {
    let mut st = LSystemState::default();
    st.set_system(library::find("Branching pattern (ABOP 1.39)").unwrap().system().unwrap());
    // Its own order at any zoom, and never deep.
    assert_eq!(st.order_for(-5.0), 13);
    assert_eq!(st.order_for(-300.0), 13);
    assert!(!st.needs_deep(-300.0, 13));
    let zero = || BigFloat::from_f64(0.0, 128);
    let key = WalkKey {
        tables: 1,
        order: 13,
        centre: [zero(), zero()],
        upp_log2: -8.0,
        size: [400, 300],
        colouring: Colouring::Depth,
        margin: 1.0,
        deep: false,
    };
    let out = walk_segments(&st.drawn_system(), &st.tables, None, None, &key, 1.0);
    let x = out.ex.expect("its word");
    assert_eq!(x.order, 13);
    assert!(out.stats.segments > 0, "{:?}", out.stats);
    assert!(out.segments.iter().all(|s| (0.0..=1.0).contains(&s.value)));
    // The word handed back is drawn again, not rebuilt.
    let again = walk_segments(&st.drawn_system(), &st.tables, None, Some(x.clone()), &key, 1.0);
    assert!(Arc::ptr_eq(&again.ex.unwrap(), &x));
    assert_eq!(again.stats, out.stats);
}

/// The SVG of a view holds its walk: a path per run of joined lines, a polygon per filled shape.
#[test]
fn an_svg_holds_the_walk_of_its_view() {
    let mut st = LSystemState::default();
    st.set_system(library::find("Mango leaf").unwrap().system().unwrap());
    let t = st.tables.clone();
    let order = st.order_for(0.0);
    let zero = || BigFloat::from_f64(0.0, 128);
    let key = WalkKey {
        tables: 1,
        order,
        centre: [zero(), BigFloat::from_f64(20.0, 128)],
        upp_log2: -1.0,
        size: [300, 200],
        colouring: Colouring::Depth,
        margin: 1.0,
        deep: false,
    };
    let out = walk_segments(&st.drawn_system(), &t, None, None, &key, st.depth_scale(order));
    assert!(!out.outlines.is_empty() && !out.segments.is_empty(), "{:?}", out.stats);
    let doc = svg::document(&svg::Picture {
        size: key.size,
        width: 1.5,
        segments: &out.segments,
        polygons: &out.outlines,
        colour: &|v| [(v * 255.0) as u8, 0, 0],
        background: [0, 0, 0],
        title: "Mango leaf",
    });
    assert_eq!(doc.matches("<polygon ").count(), out.outlines.len());
    let lines: usize = doc.lines().filter(|l| l.starts_with("<path ")).map(|l| l.matches(" L").count()).sum();
    assert_eq!(lines, out.segments.len(), "every segment a line of some path");
}

#[test]
fn a_centre_difference_divides_at_any_scale() {
    let p = 4096;
    let d = fractadyne_core::bf_sub(&BigFloat::from_f64(1.0, p), &BigFloat::from_f64(1.0 - 2f64.powi(-40), p), p);
    assert_eq!(ratio_f64(&d, -40.0 - 3.0), 8.0);
    // 2^-1100 / 2^-1102: both outside f64, the ratio inside.
    let mut tiny = BigFloat::from_f64(-1.0, p);
    tiny.set_exponent(tiny.exponent().unwrap() - 1100);
    assert_eq!(ratio_f64(&tiny, -1102.0), -4.0);
    assert_eq!(ratio_f64(&BigFloat::from_f64(0.0, p), -5.0), 0.0);
}

#[test]
fn a_filled_shape_becomes_triangles_inside_the_view() {
    // A filled snowflake framed: its triangles cover about the snowflake's area, and every corner
    // is within the clip rectangle around the view.
    let mut st = LSystemState::default();
    st.set_system(LSystem::parse("angle 60\naxiom {F++F++F}\nF = F-F++F-F\n").unwrap());
    let t = st.tables.clone();
    let b = lsystem::bounds(&t, 4, 1 << 20).unwrap();
    let upp = ((b[2] - b[0]) / 300.0).max((b[3] - b[1]) / 300.0);
    let centre = [BigFloat::from_f64(0.5 * (b[0] + b[2]), 128), BigFloat::from_f64(0.5 * (b[1] + b[3]), 128)];
    let key = WalkKey { tables: 1, order: 4, centre, upp_log2: upp.log2(), size: [320, 320], colouring: Colouring::Plain, margin: 1.0, deep: false };
    let out = walk_segments(&st.drawn_system(), &t, None, None, &key, 1.0);
    assert!(out.segments.is_empty(), "inside braces the turtle draws no lines");
    assert!(!out.triangles.is_empty());
    let area: f64 = out
        .triangles
        .iter()
        .map(|tr| {
            let (a, b, c) = (tr.a, tr.b, tr.c);
            0.5 * f64::from(((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])).abs())
        })
        .sum();
    // The order-n snowflake's area is A0·(1 + ⅓·Σ_{k<n} (4/9)^k), A0 its triangle's (side 3^n
    // steps): at order 4, A0·3448/2187. The triangles must cover exactly that.
    let side = (t.step(4)[0].hypot(t.step(4)[1])) * 81.0 / upp;
    let want = 3f64.sqrt() / 4.0 * side * side * 3448.0 / 2187.0;
    assert!((area - want).abs() < 1e-4 * want, "fill {area} px², the snowflake {want} px²");
    for tr in &out.triangles {
        for p in [tr.a, tr.b, tr.c] {
            assert!(p[0].abs() <= 166.0 && p[1].abs() <= 166.0, "{p:?} outside the clip");
        }
    }
}
