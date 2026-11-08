use super::*;

/// ⛔Coloring plan A4: turning on Stripe's "Tail only" over a settled view left the frame exactly
/// as it was (`--uitest` stripe-tail-redraws: the toggled frame matched the tail-off frame pixel
/// for pixel), because the key a frame re-iterates on had no tail fields. The tail and its length
/// change what the iteration accumulates, so each must change the key.
#[test]
fn the_stripe_tail_changes_the_iteration_key() {
    let params = |tail: bool, len: u32| MandelbrotParams {
        color_method: 1,
        stripe_tail: tail,
        stripe_tail_len: len,
        ..MandelbrotParams::default()
    };
    let key = |p: MandelbrotParams| p.iter_key([64, 64], 0, 0);
    assert!(key(params(false, 16)) == key(params(false, 16)), "the same settings must give the same key");
    assert!(key(params(false, 16)) != key(params(true, 16)), "turning the tail on must re-iterate");
    assert!(key(params(true, 16)) != key(params(true, 4)), "a new tail length must re-iterate");
}
