//! The kept colored picture's signature (`color_inputs_sig`): every input the color pass makes its
//! picture from must move it, or a kept picture goes stale on screen.

use super::*;
use std::sync::Arc;

fn base() -> (ColorUniforms, Vec<[f32; 4]>, InteriorShade) {
    let mut cu: ColorUniforms = bytemuck::Zeroable::zeroed();
    cu.lut_len = 4;
    cu.cycle = 1.0;
    let lut = vec![[0.1, 0.2, 0.3, 1.0], [0.4, 0.5, 0.6, 1.0], [0.7, 0.8, 0.9, 1.0], [1.0, 1.0, 1.0, 1.0]];
    (cu, lut, InteriorShade::default())
}

fn sig(cu: &ColorUniforms, lut: &[[f32; 4]], smooth: bool, interior: &InteriorShade, writes: u64, size: [u32; 2]) -> u64 {
    color_inputs_sig(cu, lut, smooth, interior, writes, size)
}

#[test]
fn the_same_inputs_give_the_same_signature() {
    let (cu, lut, int) = base();
    assert_eq!(sig(&cu, &lut, true, &int, 7, [640, 480]), sig(&cu, &lut, true, &int, 7, [640, 480]));
}

#[test]
fn a_uniform_moves_it() {
    let (cu, lut, int) = base();
    let a = sig(&cu, &lut, true, &int, 7, [640, 480]);
    let mut moved = cu;
    moved.offset = 0.25; // palette cycling's phase
    assert_ne!(a, sig(&moved, &lut, true, &int, 7, [640, 480]));
    let mut moved = cu;
    moved.de_phase = 0.5; // the glow's sweep
    assert_ne!(a, sig(&moved, &lut, true, &int, 7, [640, 480]));
}

#[test]
fn one_palette_entry_moves_it() {
    let (cu, lut, int) = base();
    let a = sig(&cu, &lut, true, &int, 7, [640, 480]);
    let mut other = lut.clone();
    other[2][1] += 1.0e-6;
    assert_ne!(a, sig(&cu, &other, true, &int, 7, [640, 480]));
    // …and its interpolation.
    assert_ne!(a, sig(&cu, &lut, false, &int, 7, [640, 480]));
}

#[test]
fn the_interior_palette_moves_it() {
    let (cu, lut, int) = base();
    let a = sig(&cu, &lut, true, &int, 7, [640, 480]);
    let with = InteriorShade { lut: Some(Arc::new(vec![[0.5, 0.5, 0.5, 1.0]])), ..InteriorShade::default() };
    let b = sig(&cu, &lut, true, &with, 7, [640, 480]);
    assert_ne!(a, b);
    let changed = InteriorShade { lut: Some(Arc::new(vec![[0.5, 0.5, 0.6, 1.0]])), ..InteriorShade::default() };
    assert_ne!(b, sig(&cu, &lut, true, &changed, 7, [640, 480]));
    let unsmooth = InteriorShade { lut_smooth: !with.lut_smooth, ..with.clone() };
    assert_ne!(b, sig(&cu, &lut, true, &unsmooth, 7, [640, 480]));
}

#[test]
fn a_pass_that_wrote_the_g_buffer_moves_it() {
    let (cu, lut, int) = base();
    assert_ne!(sig(&cu, &lut, true, &int, 7, [640, 480]), sig(&cu, &lut, true, &int, 8, [640, 480]));
}

#[test]
fn the_size_it_is_drawn_at_moves_it() {
    let (cu, lut, int) = base();
    let a = sig(&cu, &lut, true, &int, 7, [640, 480]);
    assert_ne!(a, sig(&cu, &lut, true, &int, 7, [641, 480]));
    assert_ne!(a, sig(&cu, &lut, true, &int, 7, [640, 481]));
}
