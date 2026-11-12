use super::*;

/// ⛔Coloring plan A4: turning on Stripe's "Tail only" over a settled view left the frame exactly
/// as it was (`--uitest` stripe-tail-redraws: the toggled frame matched the tail-off frame pixel
/// for pixel), because the key a frame re-iterates on had no tail fields. The tail and its length
/// change what the iteration accumulates, so each must change the key.
#[test]
fn the_stripe_tail_changes_the_iteration_key() {
    let params = |tail: bool, len: u32| MandelbrotParams {
        color_method: fractadyne_core::coloring::ColorMethod::Stripe.gpu_id(),
        stripe_tail: tail,
        stripe_tail_len: len,
        ..MandelbrotParams::default()
    };
    let key = |p: MandelbrotParams| p.iter_key([64, 64], 0, 0);
    assert!(key(params(false, 16)) == key(params(false, 16)), "the same settings must give the same key");
    assert!(key(params(false, 16)) != key(params(true, 16)), "turning the tail on must re-iterate");
    assert!(key(params(true, 16)) != key(params(true, 4)), "a new tail length must re-iterate");
}

/// B2: the key holds what the iteration accumulates, not the method on screen. Smooth and
/// Distance accumulate nothing, so switching between them only recolors; a statistic's parameter
/// (stripe frequency and tail, trap shape) counts only while that statistic is accumulated.
#[test]
fn the_key_holds_what_is_accumulated_not_what_is_shown() {
    use fractadyne_core::coloring::{ColorMethod, Stat, StatSet, TrapType};
    let params = |m: ColorMethod, orbit_stats: StatSet, freq: f32, trap: TrapType| MandelbrotParams {
        color_method: m.gpu_id(),
        orbit_stats,
        stripe_freq: freq,
        trap_type: trap.gpu_id(),
        ..MandelbrotParams::default()
    };
    let key = |p: MandelbrotParams| p.iter_key([64, 64], 0, 0);
    let (none, point) = (StatSet::NONE, TrapType::Point);
    assert!(key(params(ColorMethod::Smooth, none, 1.0, point)) == key(params(ColorMethod::Distance, none, 1.0, point)));
    // A method that reads a statistic still re-iterates: it accumulates it.
    for m in [ColorMethod::Stripe, ColorMethod::TriangleIneq, ColorMethod::OrbitTrap, ColorMethod::Decomposition] {
        assert!(key(params(ColorMethod::Smooth, none, 1.0, point)) != key(params(m, none, 1.0, point)), "{m:?}");
        // …unless the Smooth frame accumulated it already.
        assert!(key(params(ColorMethod::Smooth, m.orbit_stats(), 1.0, point)) == key(params(m, none, 1.0, point)), "{m:?}");
    }
    // Parameters of a statistic not accumulated change nothing; accumulated, they re-iterate.
    let smooth = |freq, trap| key(params(ColorMethod::Smooth, none, freq, trap));
    assert!(smooth(1.0, point) == smooth(5.0, TrapType::Cross));
    let stripe = |freq| key(params(ColorMethod::Smooth, StatSet::of(Stat::Stripe), freq, point));
    assert!(stripe(1.0) != stripe(5.0));
    let trap = |t| key(params(ColorMethod::Smooth, StatSet::of(Stat::Trap), 1.0, t));
    assert!(trap(TrapType::Point) != trap(TrapType::Cross));
    let tail = |on: bool| {
        key(MandelbrotParams { stripe_tail: on, stripe_tail_len: 16, ..params(ColorMethod::Smooth, none, 1.0, point) })
    };
    assert!(tail(false) == tail(true), "the stripe tail counts only with the stripe accumulated");
}
