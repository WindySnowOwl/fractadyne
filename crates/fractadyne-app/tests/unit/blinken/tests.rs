use super::*;

#[test]
fn the_panel_snaps_to_the_corner_nearest_where_it_was_dropped() {
    let view = Rect::from_min_size(Pos2::new(0.0, 0.0), vec2(1000.0, 800.0));
    let at = |x: f32, y: f32| Rect::from_min_size(Pos2::new(x, y), vec2(300.0, 200.0));
    assert_eq!(Corner::nearest(view, at(10.0, 10.0)), Corner::TopLeft);
    assert_eq!(Corner::nearest(view, at(690.0, 10.0)), Corner::TopRight);
    assert_eq!(Corner::nearest(view, at(10.0, 590.0)), Corner::BottomLeft);
    assert_eq!(Corner::nearest(view, at(690.0, 590.0)), Corner::BottomRight);
    // And a corner places the panel inside the view, a margin in.
    let p = Corner::BottomRight.place(view, vec2(300.0, 200.0), 10.0);
    assert_eq!(p, Pos2::new(690.0, 590.0));
}

#[test]
fn sizes_and_corners_round_trip_through_their_saved_names() {
    for size in [Size::Off, Size::Strip, Size::Panel, Size::Full] {
        assert_eq!(Size::from_name(size.name()), Some(size));
    }
    for c in [Corner::TopLeft, Corner::TopRight, Corner::BottomLeft, Corner::BottomRight] {
        assert_eq!(Corner::from_name(c.name()), c);
    }
    // Unknown or empty: off, top-left.
    assert_eq!(Overlay::from_session("", "", 0.95).size, Size::Off);
    assert_eq!(Overlay::from_session("bogus", "?", f32::NAN).corner, Corner::TopLeft);
    assert_eq!(Overlay::from_session("full", "br", 2.0).opacity, 1.0);
}

#[test]
fn b_cycles_off_strip_panel_full_and_back() {
    let mut o = Overlay::from_session("", "", 0.95);
    let mut seen = vec![o.size];
    for _ in 0..4 {
        o.cycle();
        seen.push(o.size);
    }
    assert_eq!(seen, vec![Size::Off, Size::Strip, Size::Panel, Size::Full, Size::Off]);
}

#[test]
fn the_clock_reads_yyyy_mm_dd_hh_mm_ss_tenths() {
    let t = clock_text();
    let b = t.as_bytes();
    assert_eq!(t.len(), 21, "{t}");
    for (i, ch) in [(4, b'-'), (7, b'-'), (10, b'_'), (13, b':'), (16, b':'), (19, b'.')] {
        assert_eq!(b[i], ch, "{t}");
    }
    assert!(t.chars().filter(|c| c.is_ascii_digit()).count() == 15, "{t}");
}

#[test]
fn utc_from_the_system_clock_is_a_real_date() {
    let (y, mo, d, h, mi, s, _) = utc_now();
    assert!((2020..2200).contains(&y));
    assert!((1..=12).contains(&mo) && (1..=31).contains(&d) && h < 24 && mi < 60 && s < 60);
}

#[test]
fn the_panel_is_taller_at_each_size_and_off_takes_no_room() {
    let w = panel::DESIGN_W;
    assert_eq!(panel::height(Size::Off, w), 0.0);
    assert!(panel::height(Size::Strip, w) < panel::height(Size::Panel, w));
    assert!(panel::height(Size::Panel, w) < panel::height(Size::Full, w));
}
