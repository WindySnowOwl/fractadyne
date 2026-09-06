//! Tests for the screen eyedropper: the byte order, and the state machine that decides when a
//! click means "take this colour".
//!
//! ⚠The FFI itself cannot be tested here — it needs a desktop, a cursor and another window to
//! point at. What CAN be tested is everything that decides what the FFI's answers mean, which is
//! where both of the plausible bugs live: reading the colour channels backwards, and treating the
//! click that STARTED the pick as the click that ends it.

use super::{colorref_rgb, step, Pick, PickStep};

/// ⭐⭐**A `COLORREF` is `0x00bbggrr`.** Reading it as RGB swaps red and blue and still yields a
/// perfectly plausible colour — the failure has no symptom except that everything picked comes
/// back the wrong hue. The `.ugr` importer had the same trap and it took a pixel-level check to
/// find, so this pins the two channels that swap, by name, with a control between them.
#[test]
fn colorref_is_bgr_not_rgb() {
    // Pure red is 0x000000ff, because red is the LOW byte.
    assert_eq!(colorref_rgb(0x0000_00ff), Some([1.0, 0.0, 0.0]), "0x0000ff must be RED");
    // Pure blue is 0x00ff0000, the high byte. An RGB reading swaps these two and passes any test
    // that only checks one of them.
    assert_eq!(colorref_rgb(0x00ff_0000), Some([0.0, 0.0, 1.0]), "0xff0000 must be BLUE");
    // Green sits in the middle either way, so it is the control that proves the test would still
    // notice a channel rotation rather than only a swap.
    assert_eq!(colorref_rgb(0x0000_ff00), Some([0.0, 1.0, 0.0]), "0x00ff00 must be GREEN");
    assert_eq!(colorref_rgb(0), Some([0.0, 0.0, 0.0]));
    // An asymmetric value, so a transposition anywhere is visible.
    let c = colorref_rgb(0x0011_2233).unwrap();
    let b = |v: f32| (v * 255.0 + 0.5) as u8;
    assert_eq!([b(c[0]), b(c[1]), b(c[2])], [0x33, 0x22, 0x11], "0x00112233 unpacks to r33 g22 b11");
}

/// ⚠`CLR_INVALID` is how `GetPixel` reports failure, and it is otherwise a legal-looking value —
/// white with the top byte set. Taking it as a colour would silently paint the stop white every
/// time a sample failed, which reads as "the dropper picked the wrong thing" rather than as an
/// error.
#[test]
fn an_invalid_sample_is_not_a_colour() {
    assert_eq!(colorref_rgb(0xffff_ffff), None);
    // ⭐But 0x00ffffff — the same white with a clear top byte — IS a colour, and must survive.
    assert_eq!(colorref_rgb(0x00ff_ffff), Some([1.0, 1.0, 1.0]));
}

/// ⭐⭐**The bug `armed` exists to prevent.** The click that starts the pick is still physically
/// down when the first poll runs, so "commit while the button is down" takes the colour of the
/// dropper button itself — instantly, before the user has moved anywhere.
#[test]
fn the_starting_click_cannot_be_the_picking_click() {
    let start = Pick::default();
    assert!(!start.armed);
    // Frame 1: the button that opened the pick is still held. Nothing may be taken.
    let s = step(start, true, false, Some([1.0, 0.0, 0.0]));
    let PickStep::Continue(p) = s else { panic!("a held starting button took a colour: {s:?}") };
    assert!(!p.armed, "still not armed while the starting press is down");
    // Frame 2: released. Now the gesture is live.
    let PickStep::Continue(p) = step(p, false, false, Some([0.0, 1.0, 0.0])) else {
        panic!("release should continue")
    };
    assert!(p.armed);
    // Frame 3: a fresh press takes the colour.
    assert_eq!(step(p, true, false, Some([0.0, 0.0, 1.0])), PickStep::Take([0.0, 0.0, 1.0]));
}

/// The colour taken is the one under the cursor AT THE CLICK, not the last preview — between two
/// frames the pointer can move a long way, and the user is looking at where it is now.
#[test]
fn the_click_takes_the_current_sample_not_the_stale_preview() {
    let armed = Pick { armed: true, preview: Some([1.0, 0.0, 0.0]) };
    assert_eq!(step(armed, true, false, Some([0.0, 0.0, 1.0])), PickStep::Take([0.0, 0.0, 1.0]));
}

/// ⚠A click we could not sample must CANCEL, not fall back to the preview: falling back paints the
/// stop with whatever the cursor last happened to pass over, which is a wrong answer delivered
/// confidently.
#[test]
fn an_unsampleable_click_cancels_rather_than_guessing() {
    let armed = Pick { armed: true, preview: Some([1.0, 0.0, 0.0]) };
    assert_eq!(step(armed, true, false, None), PickStep::Cancel);
}

#[test]
fn escape_always_cancels() {
    for armed in [false, true] {
        for down in [false, true] {
            let p = Pick { armed, preview: Some([0.5; 3]) };
            assert_eq!(step(p, down, true, Some([1.0; 3])), PickStep::Cancel, "armed={armed}");
        }
    }
}

/// The preview keeps the last good colour across a frame that failed to sample, so moving over a
/// surface that cannot be read does not make the swatch flicker to nothing.
#[test]
fn the_preview_holds_through_a_failed_sample() {
    let p = Pick { armed: true, preview: Some([0.25, 0.5, 0.75]) };
    let PickStep::Continue(next) = step(p, false, false, None) else { panic!("should continue") };
    assert_eq!(next.preview, Some([0.25, 0.5, 0.75]));
    // And a good sample replaces it.
    let PickStep::Continue(next) = step(next, false, false, Some([1.0, 1.0, 0.0])) else {
        panic!("should continue")
    };
    assert_eq!(next.preview, Some([1.0, 1.0, 0.0]));
}

/// ⭐⭐**The GDI leak test, which is the one thing about the FFI a machine CAN check.**
/// `GetDC(0)` takes a screen device context that must be released; a missing `ReleaseDC` on any
/// path leaks one per poll, and this is polled every frame of a pick. The symptom would not be a
/// crash — it is `GetDC` beginning to fail once the process hits its GDI object quota, which looks
/// exactly like "the eyedropper stopped working after a while".
///
/// ⚠A SMOKE test, deliberately tolerant: on a session with no desktop (CI, a service) the first
/// sample legitimately fails and there is nothing to check. It only asserts when sampling worked
/// to begin with — and then it asserts that it *still* works after far more polls than a real
/// pick performs.
#[test]
#[cfg(windows)]
fn repeated_sampling_does_not_leak_the_screen_dc() {
    use super::sample_under_cursor;
    let Some(first) = sample_under_cursor() else {
        return; // no desktop to sample; nothing this test can say
    };
    for c in [first] {
        assert!(c.iter().all(|v| (0.0..=1.0).contains(v)), "sample out of range: {c:?}");
    }
    // ⭐**512 is already far past the point a leak would show.** `GetDC(NULL)` hands back a COMMON
    // device context from a cache only a handful deep, so failing to release exhausts it within
    // tens of calls — not at the 10,000-object process quota. The first version looped 20,000
    // times and took **167 seconds**, which measured the same thing and would have made the suite
    // unusable.
    // ⚠That run did measure something worth keeping: a sample costs about **8 ms**, nearly all of
    // it in GetDC/ReleaseDC. Fine for a transient picking mode polled once a frame, and the reason
    // the dropper does not sample continuously outside one.
    for i in 0..512 {
        if let Some(c) = sample_under_cursor() {
            assert!(c.iter().all(|v| (0.0..=1.0).contains(v)), "poll {i} out of range: {c:?}");
        } else {
            panic!("sampling failed at poll {i} after succeeding at poll 0 — a leaked screen DC");
        }
    }
}

/// On Windows the dropper must be offered, and elsewhere it must be refused WITH A REASON — a
/// disabled control with no explanation is worse than an absent one.
#[test]
fn support_and_its_explanation_agree() {
    use super::{supported, unsupported_reason};
    assert_eq!(supported(), cfg!(windows));
    assert_eq!(supported(), unsupported_reason().is_none());
    if let Some(why) = unsupported_reason() {
        assert!(why.len() > 40, "the reason must actually explain, not just say no");
    }
}
