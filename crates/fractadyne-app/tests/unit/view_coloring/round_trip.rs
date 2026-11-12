//! A view's coloring through its own writer and reader (`view_coloring`): every setting comes
//! back, a default coloring writes nothing, a view that states its coloring resets what it leaves
//! out, and nothing the picture does not depend on is touched. The app's side (its fields to and
//! from `ViewColoring`, and a saved view rendering the same) is the selftest `view-coloring`.

use super::{TwoColor, ViewColoring, KEYS};
use crate::{ColorMethod, TrapType};

/// Read `text` the way the view reader does and apply it to `onto`.
fn load(text: &str, onto: &ViewColoring, states: bool) -> (ViewColoring, Vec<&'static str>) {
    let pairs: Vec<(String, String)> = crate::export::view_field_pairs(text)
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let get = |k: &str| pairs.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone());
    let mut out = onto.clone();
    let mut clamped = Vec::new();
    // The reader's rules are the table's (`coloring_fields::apply_view`), on the spec.
    let mut spec = out.to_spec(&fractadyne_state::ColoringSpec::default());
    crate::coloring_fields::apply_view(&mut spec, &get, states, &mut clamped);
    out = ViewColoring::from_spec(&spec);
    (out, clamped)
}

/// A session with every setting away from its default and from every variant below, so a value
/// that did not travel shows.
fn scrambled() -> ViewColoring {
    ViewColoring {
        method: ColorMethod::Distance,
        stripe_freq: 17.0,
        stripe_tail: true,
        stripe_tail_len: 900,
        trap: TrapType::Circle,
        light: true,
        light_angle: -1.25,
        light_height: 3.5,
        glow: true,
        glow_strength: 0.15,
        glow_width: 3.25,
        two_color: TwoColor::Binary,
        lo: [0.9, 0.1, 0.1],
        hi: [0.1, 0.9, 0.1],
        normalize_live: false,
        normalize_fit: true,
        log_scale: true,
    }
}

/// The colorings a view has to carry: every method, each with its settings away from the default,
/// the light, the glow, both two-color palettes, the normalization switches, and all at once.
fn variants() -> Vec<ViewColoring> {
    let d = ViewColoring::default();
    let mut v: Vec<ViewColoring> = ColorMethod::ALL
        .into_iter()
        .map(|m| ViewColoring { method: m, ..d.clone() })
        .collect();
    let stripe = ViewColoring { method: ColorMethod::Stripe, ..d.clone() };
    v.push(ViewColoring { stripe_freq: 9.5, ..stripe.clone() });
    v.push(ViewColoring { stripe_tail: true, ..stripe.clone() });
    v.push(ViewColoring { stripe_freq: 1.0, stripe_tail: true, stripe_tail_len: 4096, ..stripe.clone() });
    for t in TrapType::ALL {
        v.push(ViewColoring { method: ColorMethod::OrbitTrap, trap: t, ..d.clone() });
    }
    v.push(ViewColoring { light: true, ..d.clone() });
    v.push(ViewColoring { light: true, light_angle: 0.3, light_height: 0.2, ..d.clone() });
    v.push(ViewColoring { glow: true, ..d.clone() });
    v.push(ViewColoring { glow: true, glow_strength: 1.0, glow_width: 0.15, ..d.clone() });
    for tc in TwoColor::ON {
        v.push(ViewColoring { two_color: tc, ..d.clone() });
        v.push(ViewColoring { two_color: tc, lo: [1.0, 0.5, 0.0], hi: [0.0, 0.25, 0.75], ..d.clone() });
    }
    v.push(ViewColoring { normalize_live: false, ..d.clone() });
    v.push(ViewColoring { normalize_fit: true, log_scale: true, ..d.clone() });
    v.push(ViewColoring { method: ColorMethod::OrbitTrap, normalize_fit: true, log_scale: true, ..d.clone() });
    v.push(ViewColoring {
        method: ColorMethod::Stripe,
        stripe_freq: 3.0,
        stripe_tail: true,
        stripe_tail_len: 64,
        light: true,
        light_angle: 1.0,
        light_height: 0.8,
        glow: true,
        glow_strength: 0.4,
        glow_width: 0.7,
        two_color: TwoColor::Duotone,
        lo: [0.2, 0.3, 0.4],
        hi: [0.9, 0.8, 0.7],
        ..d.clone()
    });
    v
}

#[test]
fn the_default_coloring_writes_nothing() {
    // An ordinary view is written exactly as before the keys existed.
    assert_eq!(ViewColoring::default().lines(), "");
}

#[test]
fn every_coloring_comes_back_onto_a_different_session() {
    for v in variants() {
        let text = v.lines();
        let (got, clamped) = load(&text, &scrambled(), true);
        assert!(clamped.is_empty(), "{text:?} clamped {clamped:?}");
        // ⭐The writer emits exactly the settings the picture depends on, so equal lines mean every
        // one of them came back, the defaults included (a left-out key reset).
        assert_eq!(got.lines(), text, "a view written as {text:?} came back different");
        assert_eq!(
            (got.method, got.light, got.glow, got.two_color),
            (v.method, v.light, v.glow, v.two_color),
            "{text:?}"
        );
    }
}

#[test]
fn what_the_picture_does_not_depend_on_is_left_alone() {
    // A default Smooth view, opened on a session tuned for everything else.
    let s = scrambled();
    let (got, _) = load("", &s, true);
    let d = ViewColoring::default();
    // What the view states, by leaving it out: Smooth, no light, no glow, no two-color palette,
    // and Smooth's normalization at its defaults.
    assert_eq!(got.method, ColorMethod::Smooth);
    assert!(!got.light && !got.glow);
    assert_eq!(got.two_color, TwoColor::Off);
    assert_eq!((got.normalize_live, got.normalize_fit, got.log_scale), (d.normalize_live, d.normalize_fit, d.log_scale));
    // What it does not depend on keeps this session's values.
    assert_eq!((got.stripe_freq, got.stripe_tail, got.stripe_tail_len), (17.0, true, 900));
    assert_eq!(got.trap, TrapType::Circle);
    assert_eq!((got.light_angle, got.light_height), (-1.25, 3.5));
    assert_eq!((got.glow_strength, got.glow_width), (0.15, 3.25));
    assert_eq!((got.lo, got.hi), (s.lo, s.hi));
}

#[test]
fn a_location_that_states_no_coloring_changes_nothing() {
    let s = scrambled();
    assert_eq!(load("", &s, false).0, s);
}

#[test]
fn a_left_out_setting_resets_only_under_the_parent_the_file_names() {
    let (s, d) = (scrambled(), ViewColoring::default());
    let (got, _) = load("method=stripe\n", &s, true);
    assert_eq!((got.stripe_freq, got.stripe_tail), (d.stripe_freq, false));
    assert_eq!(got.stripe_tail_len, 900, "the tail's length is the tail's, and the tail is off");
    assert_eq!(got.trap, TrapType::Circle, "the trap shape is not the stripe method's");
    assert!(!got.normalize_live, "normalization's guard is Smooth's");

    let (got, _) = load("method=stripe\nstripe_tail=1\n", &s, true);
    assert_eq!((got.stripe_tail, got.stripe_tail_len), (true, d.stripe_tail_len));

    let (got, _) = load("method=trap\n", &s, true);
    assert_eq!(got.trap, d.trap);
    assert_eq!((got.normalize_fit, got.log_scale), (false, false), "the trap fits its own range");

    let (got, _) = load("light=1\nglow=1\npalette_mode=duotone\n", &s, true);
    assert_eq!((got.light_angle, got.light_height), (d.light_angle, d.light_height));
    assert_eq!((got.glow_strength, got.glow_width), (d.glow_strength, d.glow_width));
    assert_eq!((got.two_color, got.lo, got.hi), (TwoColor::Duotone, d.lo, d.hi));
}

#[test]
fn out_of_range_values_are_clamped_and_named() {
    let text = "method=stripe\nstripe_freq=99\nstripe_tail=1\nstripe_tail_len=2\n\
                light=1\nlight_height=0\nlight_angle=inf\nglow=1\nglow_strength=5\nglow_width=0\n\
                palette_mode=binary\npalette_lo=2,-1,0.5\n";
    let (got, clamped) = load(text, &scrambled(), true);
    assert_eq!(got.stripe_freq, 24.0);
    assert_eq!(got.stripe_tail_len, 4);
    assert_eq!(got.light_height, 0.2);
    assert_eq!(got.light_angle, -1.25, "a value that is not finite keeps the session's");
    assert_eq!((got.glow_strength, got.glow_width), (1.0, 0.15));
    assert_eq!(got.lo, [1.0, 0.0, 0.5]);
    for name in ["stripe density", "tail length", "relief", "light angle", "glow", "band width", "palette colors"] {
        assert!(clamped.contains(&name), "{name:?} not reported in {clamped:?}");
    }
}

#[test]
fn an_unknown_method_keeps_the_session_and_says_where() {
    let s = scrambled();
    let (got, clamped) = load("method=angle\nstripe_freq=9\n", &s, true);
    assert_eq!(got.method, s.method, "never a silent Smooth");
    assert_eq!(got.stripe_freq, 9.0, "what the file does say still applies");
    assert_eq!((got.trap, got.normalize_live, got.normalize_fit), (s.trap, s.normalize_live, s.normalize_fit));
    assert!(clamped.is_empty());
    // The reader reports it with its line.
    let (report, _) = crate::export::inspect_view_text("center_re=0\ncenter_im=0\ncycle=0.2\nmethod=angle\n");
    let said = report.problems.join("; ");
    assert!(
        said.contains("line 4") && said.contains("method needs one of smooth, stripe, triangle, trap, distance, decomposition"),
        "{said}"
    );
}

#[test]
fn every_key_is_known_typed_and_written() {
    for k in KEYS {
        assert!(crate::export::KNOWN_VIEW_KEYS.contains(k), "{k} is written but the reader would call it unknown");
        let (report, _) = crate::export::inspect_view_text(&format!("center_re=0\ncenter_im=0\n{k}=@\n"));
        assert!(
            report.problems.iter().any(|p| p.contains(&format!("{k} needs"))),
            "an unreadable {k} is not reported: {:?}",
            report.problems
        );
    }
    // And the list is what the writer writes: every key appears in some coloring, none outside it.
    let mut written: Vec<String> = variants()
        .iter()
        .flat_map(|v| v.lines().lines().map(|l| l.split_once('=').unwrap().0.to_string()).collect::<Vec<_>>())
        .collect();
    written.sort();
    written.dedup();
    let mut keys: Vec<String> = KEYS.iter().map(|k| k.to_string()).collect();
    keys.sort();
    assert_eq!(written, keys);
}

#[test]
fn the_lines_read_as_a_person_would_write_them() {
    let v = ViewColoring {
        method: ColorMethod::Stripe,
        stripe_freq: 9.5,
        stripe_tail: true,
        stripe_tail_len: 64,
        light: true,
        light_angle: 0.5,
        two_color: TwoColor::Binary,
        hi: [1.0, 0.75, 0.0],
        ..ViewColoring::default()
    };
    assert_eq!(
        v.lines(),
        "method=stripe\nstripe_freq=9.5\nstripe_tail=1\nstripe_tail_len=64\nlight=1\nlight_angle=0.5\n\
         palette_mode=binary\npalette_hi=1,0.75,0\n"
    );
}
