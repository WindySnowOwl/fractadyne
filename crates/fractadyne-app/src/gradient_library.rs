//! Tests for the saved-gradient library's on-disk round trip.
//!
//! ⭐⭐**The failure this exists to prevent is silent.** A gradient library that stores STOPS
//! looks perfectly correct — it reloads, it renders, the colours are right — and has quietly
//! dropped every midpoint, blend curve and hue sweep, which are exactly the properties that make a
//! gradient worth saving. It is the same trap `custom_segments` was introduced for, one layer out,
//! and nothing about the UI would reveal it: you would only find out by saving a curved gradient,
//! reloading it a week later, and wondering why it had gone flat.

use super::{live_saved_index, segments_to_gradient, GradientFile, SavedGradient};
use fractadyne_color::segment::{Blend, Gradient, Segment, Space, LUT_SIZE};

/// A gradient that uses every property a stop list cannot hold.
fn rich() -> Gradient {
    let c = |r: f32, g: f32, b: f32| [r, g, b, 1.0];
    Gradient {
        name: "rich".into(),
        segments: vec![
            // An off-centre midpoint.
            Segment {
                left: 0.0,
                mid: 0.08,
                right: 0.3,
                left_color: c(0.0, 0.0, 0.0),
                right_color: c(0.9, 0.1, 0.2),
                blend: Blend::Linear,
                space: Space::Rgb,
            },
            // A non-linear blend.
            Segment {
                left: 0.3,
                mid: 0.5,
                right: 0.7,
                left_color: c(0.9, 0.1, 0.2),
                right_color: c(0.2, 0.6, 0.9),
                blend: Blend::SphereIncreasing,
                space: Space::Rgb,
            },
            // A hue sweep the long way round.
            Segment {
                left: 0.7,
                mid: 0.85,
                right: 1.0,
                left_color: c(0.2, 0.6, 0.9),
                right_color: c(1.0, 1.0, 1.0),
                blend: Blend::Sine,
                space: Space::HsvCw,
            },
        ],
    }
}

fn as_saved(g: &Gradient, name: &str) -> SavedGradient {
    SavedGradient {
        name: name.to_string(),
        segment: g
            .segments
            .iter()
            .map(|s| fractadyne_state::PaletteSegment {
                left: s.left,
                mid: s.mid,
                right: s.right,
                left_color: s.left_color,
                right_color: s.right_color,
                blend: s.blend.as_u8(),
                space: s.space.as_u8(),
                blend_params: s.blend.params(),
            })
            .collect(),
    }
}

/// ⭐⭐**Saving and reloading must be BIT-IDENTICAL through the bake**, not merely similar.
/// Comparing the baked LUT rather than the segment fields is deliberate: it is what the GPU
/// actually fetches, so it catches a field that survives the file but is then misread on the way
/// back — a `blend`/`space` number reinterpreted, say, which comparing the numbers to themselves
/// would never notice.
#[test]
fn a_saved_gradient_round_trips_through_toml_without_flattening() {
    let before = rich();
    let file = GradientFile { gradient: vec![as_saved(&before, "rich")] };
    let text = toml::to_string_pretty(&file).expect("serialize");
    let back: GradientFile = toml::from_str(&text).expect("deserialize");
    assert_eq!(back.gradient.len(), 1);
    assert_eq!(back.gradient[0].name, "rich");

    let after = segments_to_gradient(&back.gradient[0].name, &back.gradient[0].segment);
    assert_eq!(
        before.bake(LUT_SIZE),
        after.bake(LUT_SIZE),
        "a saved gradient came back rendering differently"
    );

    // And the properties are individually intact — so a failure says WHICH one was lost rather
    // than only that something was.
    assert_eq!(after.segments.len(), 3);
    assert!((after.segments[0].mid - 0.08).abs() < 1e-6, "midpoint lost");
    assert_eq!(after.segments[1].blend, Blend::SphereIncreasing, "blend curve lost");
    assert_eq!(after.segments[2].space, Space::HsvCw, "hue sweep lost");
    assert!(!after.is_stop_expressible(), "the whole point is that stops could not hold this");
}

/// ⚠**The control.** The assertion above is only meaningful if flattening would actually be
/// visible — if a stop list happened to render the same, saving stops would have been fine and the
/// test would be theatre. It does not: routing the same gradient through `to_stops`/`from_stops`
/// changes what renders.
#[test]
fn storing_the_same_gradient_as_stops_would_have_changed_the_picture() {
    let before = rich();
    let flattened = Gradient::from_stops("flat", &before.to_stops());
    assert!(flattened.is_stop_expressible());
    assert_ne!(
        before.bake(LUT_SIZE),
        flattened.bake(LUT_SIZE),
        "if these agreed, storing stops would be lossless and this whole design would be moot"
    );
}

/// Saving twice under one name replaces rather than appends — otherwise the picker fills with
/// entries it cannot tell apart, and the user's "update" silently becomes a duplicate.
#[test]
fn the_library_is_keyed_by_name() {
    let mut lib: Vec<SavedGradient> = Vec::new();
    let upsert = |lib: &mut Vec<SavedGradient>, e: SavedGradient| {
        match lib.iter().position(|g| g.name == e.name) {
            Some(i) => lib[i] = e,
            None => lib.push(e),
        }
    };
    upsert(&mut lib, as_saved(&rich(), "one"));
    upsert(&mut lib, as_saved(&Gradient::from_stops("x", &[(0.0, [0.0; 3]), (1.0, [1.0; 3])]), "one"));
    assert_eq!(lib.len(), 1, "the second save under one name must REPLACE");
    assert_eq!(lib[0].segment.len(), 1, "and it must be the second gradient that survived");
    upsert(&mut lib, as_saved(&rich(), "two"));
    assert_eq!(lib.len(), 2, "a different name is a different entry");
}

/// ⭐⭐**The Color ▸ Palette menu's check mark is a claim about the PICTURE, not about a name.**
/// This is the rule that makes it true, and the case that breaks the obvious implementation: a
/// saved gradient that has been loaded and then EDITED still carries its saved name, so a name-only
/// match would keep the tick on it — telling the user the saved gradient is on screen when it is
/// their unsaved edit of it, and implying reopening the entry would change nothing.
#[test]
fn the_menu_tick_follows_the_segments_and_not_the_name() {
    let saved = vec![as_saved(&rich(), "rich"), as_saved(&plain(), "plain")];

    // The live palette IS the second entry.
    let live = saved[1].segment.clone();
    assert_eq!(live_saved_index(&saved, true, &live), Some(1));

    // ⚠**The case with a wrong twin.** One segment edited — the name a UI would still be showing is
    // unchanged, and the gradient is no longer what was saved.
    let mut edited = live.clone();
    edited[0].right_color = [0.1, 0.2, 0.3, 1.0];
    assert_ne!(edited, live, "the guard: this probe must actually differ, or it proves nothing");
    assert_eq!(
        live_saved_index(&saved, true, &edited),
        None,
        "an edited copy still carries the saved NAME — matching on it would tick the wrong row"
    );

    // A preset is never a match, whatever the stale segment list behind it happens to hold.
    assert_eq!(
        live_saved_index(&saved, false, &live),
        None,
        "the tick says which row the palette came from, and a preset came from the preset row"
    );

    // Nothing saved, nothing ticked — the menu simply omits the group.
    assert_eq!(live_saved_index(&[], true, &live), None);
}

/// A second gradient, distinct from [`rich`], for the library-with-two-entries cases.
fn plain() -> Gradient {
    Gradient::from_stops("plain", &[(0.0, [0.0, 0.0, 0.2]), (1.0, [0.8, 0.9, 1.0])])
}
