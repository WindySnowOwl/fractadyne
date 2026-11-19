//! `crit_span`: where a Julia view's critical orbit goes in the reference buffer, and when it
//! does not go at all (the rule before: rebase onto Z_0 only).
use super::{crit_span, CRIT_INDEX_MAX};

const NO_LIMIT: u64 = u64::MAX;

#[test]
fn a_julia_view_puts_its_critical_orbit_right_after_the_reference() {
    assert_eq!(crit_span(1, 340, 0, 305, NO_LIMIT), [340, 305]);
}

#[test]
fn a_mandelbrot_view_has_none_whatever_it_carries() {
    // The shader's `crit_on` reads `julia` too, but the words must say so on their own.
    assert_eq!(crit_span(0, 340, 0, 305, NO_LIMIT), [0, 0]);
}

#[test]
fn no_critical_orbit_means_the_rule_before() {
    assert_eq!(crit_span(1, 340, 0, 0, NO_LIMIT), [0, 0]);
}

#[test]
fn a_buffer_with_trees_after_the_orbit_takes_none() {
    // A BLA or u-space tree sits at `orbit.len()`: the critical orbit would overlap what the
    // shader reads there. Neither exists for a Julia view, so this is the guard, not a case.
    assert_eq!(crit_span(1, 340, 4 * 100, 305, NO_LIMIT), [0, 0]);
}

#[test]
fn every_index_fits_a_resumable_pass_state() {
    // `ref_n` travels between passes in an f32 lane: exact below 2^24.
    let at = (CRIT_INDEX_MAX - 1000) as usize;
    assert_eq!(crit_span(1, at, 0, 999, NO_LIMIT), [at as u32, 999]);
    assert_eq!(crit_span(1, at, 0, 1000, NO_LIMIT), [0, 0]);
}

#[test]
fn a_buffer_that_cannot_hold_it_takes_none() {
    assert_eq!(crit_span(1, 340, 0, 305, 645), [340, 305]);
    assert_eq!(crit_span(1, 340, 0, 305, 644), [0, 0]);
}
