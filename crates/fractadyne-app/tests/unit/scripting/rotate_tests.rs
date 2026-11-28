//! The view's rotation in a tour (design/view-rotation.md): a keyframe's `rotate`, unwrapped
//! degrees, interpolated along a glide, inherited forward from 0, and put on a viewport wrapped.

use super::parse_tour_text;

fn tour(body: &str) -> Result<super::Playback, String> {
    parse_tour_text(&format!("format_version = 2\nname = \"t\"\n{body}"))
}

/// One full turn while the camera stays put: 0 then 360 over a linear glide. A hold keeps its
/// keyframe's angle; a keyframe that states none inherits it; a viewport takes it wrapped.
#[test]
fn a_glide_turns_the_view_and_a_hold_keeps_it() {
    let p = tour(
        "[[keyframe]]\nt = 0.0\nre = \"-0.75\"\nim = \"0.1\"\nzoom = 4\nrotate = 0\nhold = 1.0\n\
         [[keyframe]]\nt = 5.0\nrotate = 360\nease = \"linear\"\nhold = 1.0\n\
         [[keyframe]]\nt = 7.0\nzoom = 8\n",
    )
    .expect("a turning tour parses");
    let at = |t: f64| p.sample(t).rotate;
    assert_eq!(at(0.5), 0.0, "the first hold");
    assert!((at(2.0) - 90.0).abs() < 1e-9, "a quarter of the glide, a quarter turn: {}", at(2.0));
    assert!((at(3.0) - 180.0).abs() < 1e-9);
    assert_eq!(at(5.5), 360.0, "the second hold, unwrapped");
    assert_eq!(at(7.0), 360.0, "inherited by a keyframe that states none");
    let mut vp = fractadyne_core::Viewport::new(100.0, 100.0);
    p.sample(2.0).turn(&mut vp);
    assert!((vp.rotation - 90.0).abs() < 1e-9);
    p.sample(4.0).turn(&mut vp);
    assert!((vp.rotation - -90.0).abs() < 1e-9, "270 is put on the view as -90: {}", vp.rotation);
    p.sample(5.5).turn(&mut vp);
    assert_eq!(vp.rotation.to_bits(), 0f64.to_bits(), "a whole turn is unturned, +0");
    // The camera did not move during the turn.
    let (a, b) = (p.sample(1.5), p.sample(4.0));
    assert_eq!((a.cx, a.cy, a.logmag), (b.cx, b.cy, b.logmag));
}

/// A tour that never states `rotate` plays unturned, whatever the view was turned to before.
#[test]
fn a_tour_without_rotate_plays_unturned() {
    let p = tour("[[keyframe]]\nt = 0.0\nzoom = 2\n[[keyframe]]\nt = 2.0\nzoom = 4\n").unwrap();
    assert_eq!(p.sample(1.0).rotate, 0.0);
    let mut vp = fractadyne_core::Viewport::new(100.0, 100.0);
    vp.rotation = 33.0;
    p.sample(1.0).turn(&mut vp);
    assert_eq!(vp.rotation.to_bits(), 0f64.to_bits());
}

/// An angle that is not a number, or more than a thousand turns, is refused with its keyframe.
#[test]
fn a_wild_rotate_is_refused() {
    for v in ["nan", "inf", "4e5"] {
        let e = match tour(&format!("[[keyframe]]\nt = 0.0\nrotate = {v}\n")) {
            Ok(_) => panic!("rotate = {v} should be refused"),
            Err(e) => e,
        };
        assert!(e.contains("rotate") && e.contains("#1"), "{e}");
    }
}
