//! The wave traps' phase in a tour (user 2026-11-25: "is there a way to render the sin phase
//! shift?"): a keyframe's `trap_phase` and `interior_trap_phase`, unwrapped degrees, interpolated
//! along a glide and put on the traps wrapped.

use super::{parse_tour_text, TrapPhases};
use crate::TrapPlace;

fn tour(body: &str) -> Result<super::Playback, String> {
    parse_tour_text(&format!("format_version = 2\nname = \"t\"\n{body}"))
}

/// One full turn while the camera stays put: two keyframes at the same place, 0 then 360 over a
/// linear glide. A hold keeps its keyframe's phase; a keyframe that states none inherits it.
#[test]
fn a_glide_turns_the_phase_and_a_hold_keeps_it() {
    let p = tour(
        "[[keyframe]]\nt = 0.0\nre = \"-0.75\"\nim = \"0.1\"\nzoom = 4\ntrap_phase = 0\ninterior_trap_phase = 90\nhold = 1.0\n\
         [[keyframe]]\nt = 5.0\ntrap_phase = 360\nease = \"linear\"\nhold = 1.0\n\
         [[keyframe]]\nt = 7.0\nzoom = 8\n",
    )
    .expect("a phase tour parses");
    let at = |t: f64| p.sample(t).phases;
    assert_eq!(at(0.5), TrapPhases { trap: Some(0.0), inside: Some(90.0) }, "the first hold");
    // A quarter of the way along the 4 s glide: a quarter turn; the inside's, stated once, held.
    let q = at(2.0);
    assert!((q.trap.unwrap() - 90.0).abs() < 1e-9, "{q:?}");
    assert_eq!(q.inside, Some(90.0));
    assert!((at(3.0).trap.unwrap() - 180.0).abs() < 1e-9);
    assert_eq!(at(5.5).trap, Some(360.0), "the second hold, unwrapped");
    assert_eq!(at(7.0).trap, Some(360.0), "inherited by a keyframe that states none");
    // The camera did not move during the turn.
    let (a, b) = (p.sample(1.5), p.sample(4.0));
    assert_eq!((a.cx, a.cy, a.logmag), (b.cx, b.cy, b.logmag));
}

/// A tour that states no phase leaves the coloring's alone.
#[test]
fn a_tour_without_a_phase_states_none() {
    let p = tour("[[keyframe]]\nt = 0.0\nzoom = 2\n[[keyframe]]\nt = 2.0\nzoom = 4\n").unwrap();
    assert_eq!(p.sample(1.0).phases, TrapPhases::default());
    let (mut trap, mut inside) = (TrapPlace { phase: 33.0, ..TrapPlace::DEFAULT }, TrapPlace::DEFAULT);
    p.sample(1.0).phases.apply(&mut trap, &mut inside);
    assert_eq!((trap.phase, inside.phase), (33.0, 0.0));
}

/// Put on the traps wrapped into [-180, 180), keeping the fraction of a phase many turns long.
#[test]
fn the_phase_is_put_on_the_traps_wrapped() {
    let (mut trap, mut inside) = (TrapPlace::DEFAULT, TrapPlace::DEFAULT);
    TrapPhases { trap: Some(450.0), inside: Some(-190.0) }.apply(&mut trap, &mut inside);
    assert_eq!((trap.phase, inside.phase), (90.0, 170.0));
    TrapPhases { trap: Some(360.0 * 900.0 + 12.5), inside: Some(180.0) }.apply(&mut trap, &mut inside);
    assert_eq!((trap.phase, inside.phase), (12.5, -180.0));
}

/// A phase that is not a number, or more than a thousand turns, is refused with its keyframe.
#[test]
fn a_wild_phase_is_refused() {
    for (key, v) in [("trap_phase", "nan"), ("trap_phase", "inf"), ("interior_trap_phase", "4e5")] {
        let e = match tour(&format!("[[keyframe]]\nt = 0.0\n{key} = {v}\n")) {
            Ok(_) => panic!("{key} = {v} should be refused"),
            Err(e) => e,
        };
        assert!(e.contains(key) && e.contains("#1"), "{e}");
    }
}
