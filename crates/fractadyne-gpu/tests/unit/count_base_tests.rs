//! The count base (coloring plan D2): the rule that picks a frame's base, the palette map rewritten
//! about it, and the counter slot the shader reports a base set too high in.

use super::{
    count_base_for, count_rebased_map, escape_range_of, live_count_base, COUNTER_SLOTS, COUNT_BASE_MIN, COUNT_BASE_STEP, CTR_BELOW_BASE,
    CTR_ESC_MAX, CTR_ESC_MIN, CTR_TRAP_HIST_IN, LIVE_COUNT_BASE_MARGIN, TRAP_HIST_BUCKETS,
};
use fractadyne_core::coloring::Transfer;

/// No base below 2^21 or with nothing escaped; past it, a multiple of 2^16 at least one step and
/// less than two below the frame's lowest count, so the stored counts start in [2^16, 2^17).
#[test]
fn the_base_sits_one_to_two_steps_below_the_lowest_count() {
    assert_eq!(count_base_for(None), 0);
    assert_eq!(count_base_for(Some(f64::NAN)), 0);
    assert_eq!(count_base_for(Some(f64::INFINITY)), 0);
    assert_eq!(count_base_for(Some(f64::from(COUNT_BASE_MIN) - 0.5)), 0);
    assert_eq!(count_base_for(Some(154_303.34)), 0);
    // The 9.3e78 minibrot's edge at 2.8e80 (2026-11-27): its top count.
    assert_eq!(count_base_for(Some(9_745_195.0)), 147 * COUNT_BASE_STEP);
    let step = f64::from(COUNT_BASE_STEP);
    let mut m = f64::from(COUNT_BASE_MIN);
    while m < 1.0e8 {
        let b = count_base_for(Some(m));
        assert_eq!(b % COUNT_BASE_STEP, 0, "{m}");
        let room = m - f64::from(b);
        assert!((step..2.0 * step).contains(&room), "{m}: base {b}, room {room}");
        m = m * 1.37 + 0.731;
    }
}

/// The palette coordinate before any remap, `T(max(v − lo, 0))·cycle + offset` for a curve or an
/// equalized map, `v·cycle + offset` for a linear one (`coord_under`), in f64.
fn coord(mode: u32, equalized: bool, cycle: f32, offset: f32, lo: f32, v: f64) -> f64 {
    if mode != 0 || equalized {
        let t = Transfer::from_gpu_id(mode).expect("a curve");
        t.apply(v - f64::from(lo)) * f64::from(cycle) + f64::from(offset)
    } else {
        v * f64::from(cycle) + f64::from(offset)
    }
}

/// Each count stored less the base colors as the count itself does under the map rewritten about
/// the base: the same coordinate, modulo the wrap's period of 2 (Repeat, Mirror) or held to 0..1
/// (Clamp: a linear map's offset is not reduced, so far outside 0..1 it rounds, where both hold the
/// end), to the f32 the rewritten words are rounded to. Base 0 returns the words to the bit.
#[test]
fn the_rebased_map_colors_every_count_as_before() {
    let cases: [(u32, bool, f32, f32, f32); 7] = [
        (0, false, 0.0202, 0.1, 0.0),       // the fixed rate, the default palette offset
        (0, false, 0.000_423, -3_812.4, 0.0), // a normalized linear map: offset 0.1 − lo·cycle
        (0, false, 4.24e-5, -381.5, 0.0),    // ...whose counts land in 0..1 at first (Clamp)
        (0, true, 0.000_423, 0.1, 9_000_123.0),
        (1, false, 0.31, 0.25, 9_000_123.0),
        (2, false, 0.031, -0.4, 9_000_123.0),
        (5, true, 1.7, 0.9, 9_000_123.0),
    ];
    for &(mode, eq, cycle, offset, lo) in &cases {
        for wrap in 0..3u32 {
            assert_eq!(count_rebased_map(mode, eq, cycle, offset, lo, wrap, 0), (offset, lo), "base 0 keeps the words");
            let base = count_base_for(Some(9_000_000.0));
            let (o2, lo2) = count_rebased_map(mode, eq, cycle, offset, lo, wrap, base);
            if wrap <= 1 && mode == 0 && !eq {
                assert!((0.0..2.0).contains(&o2), "a wrapped offset is reduced: {o2}");
            }
            for k in 0..200 {
                let v = 9_000_000.0 + f64::from(k) * 1_377.731 + 0.123;
                let want = coord(mode, eq, cycle, offset, lo, v);
                let got = coord(mode, eq, cycle, o2, lo2, v - f64::from(base));
                let d = got - want;
                let d = if wrap <= 1 && mode == 0 && !eq {
                    d - 2.0 * (d / 2.0).round()
                } else if wrap == 2 {
                    got.clamp(0.0, 1.0) - want.clamp(0.0, 1.0)
                } else {
                    d
                };
                assert!(d.abs() < 2.0e-6, "mode {mode} eq {eq} wrap {wrap} v {v}: {got} for {want}");
            }
        }
    }
}

/// The live view's base (stage 2): none below 2^21; placed 2^18 to a step more under the lowest
/// count; kept while the lowest count stays a step to 2^20 above it, whatever small moves the view
/// makes; dropped to 0 when a reading comes within a step of it or under it (a pixel stored below it
/// reads as the base), for the next reading to place it again; no reading keeps it.
#[test]
fn the_live_base_is_placed_kept_and_dropped() {
    assert_eq!(live_count_base(0, None), 0);
    assert_eq!(live_count_base(7 * COUNT_BASE_STEP, None), 7 * COUNT_BASE_STEP);
    assert_eq!(live_count_base(0, Some(f64::NAN)), 0);
    assert_eq!(live_count_base(0, Some(1_000_000.0)), 0);
    // The period-192,920 minibrot's resolved frame at 2.95e88 (2026-11-27).
    let m = 5_178_440.5;
    let b = live_count_base(0, Some(m));
    assert_eq!(b % COUNT_BASE_STEP, 0);
    let room = m - f64::from(b);
    let margin = f64::from(LIVE_COUNT_BASE_MARGIN);
    assert!((margin..margin + f64::from(COUNT_BASE_STEP)).contains(&room), "{room}");
    for d in [-150_000.0, -10_000.0, 0.0, 50_000.0, 600_000.0] {
        assert_eq!(live_count_base(b, Some(m + d)), b, "a move of {d} keeps it");
    }
    let up = m + 2.0e6;
    let b2 = live_count_base(b, Some(up));
    assert!(b2 > b && (margin..margin + f64::from(COUNT_BASE_STEP)).contains(&(up - f64::from(b2))), "far up: {b2}");
    for r in [f64::from(b) + 1000.0, f64::from(b), f64::from(b) - 5.0e5] {
        assert_eq!(live_count_base(b, Some(r)), 0, "a reading at {r} drops it");
    }
    // Under 2^21 with room: kept, the counts still keep their fraction.
    let low = 30 * COUNT_BASE_STEP;
    assert!(low < COUNT_BASE_MIN);
    assert_eq!(live_count_base(low, Some(f64::from(low) + 200_000.0)), low);
}

/// A frame's escape range handed on as the counts themselves: the counters' range plus the base; a
/// sample below the base pulls the bottom down to the base itself; base 0, or no escape, the
/// counters' bits as they are.
#[test]
fn the_escape_range_is_of_the_counts_themselves() {
    let mut slots = vec![0u32; COUNTER_SLOTS];
    slots[CTR_ESC_MIN] = 70_000.5f32.to_bits();
    slots[CTR_ESC_MAX] = 70_150.5f32.to_bits();
    assert_eq!(escape_range_of(&slots, 0), (slots[CTR_ESC_MIN], slots[CTR_ESC_MAX]));
    let (lo, hi) = escape_range_of(&slots, 5_111_808);
    assert_eq!((f32::from_bits(lo), f32::from_bits(hi)), (5_181_808.5, 5_181_958.5));
    slots[CTR_BELOW_BASE] = 3;
    assert_eq!(f32::from_bits(escape_range_of(&slots, 5_111_808).0), 5_111_808.0, "below the base reads as the base");
    slots[CTR_ESC_MIN] = u32::MAX;
    assert_eq!(escape_range_of(&slots, 5_111_808).0, u32::MAX, "no escape stays no escape");
}

/// The slot is the last, after the inside's trap histogram, and the shader's constant says so.
#[test]
fn the_below_base_slot_is_the_last_and_the_shader_agrees() {
    assert_eq!(CTR_BELOW_BASE, CTR_TRAP_HIST_IN + TRAP_HIST_BUCKETS);
    assert_eq!(COUNTER_SLOTS, CTR_BELOW_BASE + 1);
    let wgsl = include_str!("../../src/mandelbrot.wgsl");
    assert!(wgsl.contains(&format!("const CTR_BELOW_BASE: u32 = {CTR_BELOW_BASE}u;")));
    // Every built-in escape stores its count through the base, and nothing else writes one.
    assert_eq!(wgsl.matches("smooth_count(iter, nu)").count(), 6);
    // ...and no code line writes the plain expression any more (comments may name it).
    let code = wgsl.lines().map(|l| l.split("//").next().unwrap_or("")).filter(|l| l.contains("f32(iter) + 1.0 - nu")).count();
    assert_eq!(code, 0, "an escape site writes the count without the base");
}
