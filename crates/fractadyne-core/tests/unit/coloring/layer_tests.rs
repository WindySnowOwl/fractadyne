use super::*;

/// Every merge mode has its own key and GPU number, and the key reads back.
#[test]
fn every_blend_mode_has_its_own_key_and_number() {
    for (i, m) in BlendMode::ALL.iter().enumerate() {
        assert_eq!(BlendMode::parse_key(m.key()), Some(*m));
        for n in &BlendMode::ALL[i + 1..] {
            assert_ne!(m.key(), n.key());
            assert_ne!(m.gpu_id(), n.gpu_id());
        }
    }
    assert_eq!(BlendMode::default(), BlendMode::HardLight);
    assert_eq!(BlendMode::parse_key("hard light"), None);
}

/// The modes as their names say: the gray's middle leaves a color as it is under hard light, soft
/// light and overlay; white and black are multiply's and screen's identities; multiply never
/// lightens and screen never darkens; hard light is overlay with the roles swapped; every result
/// stays in 0..1.
#[test]
fn each_mode_merges_as_named() {
    let steps: Vec<f32> = (0..=20).map(|i| i as f32 / 20.0).collect();
    for &b in &steps {
        for m in [BlendMode::HardLight, BlendMode::SoftLight, BlendMode::Overlay] {
            assert!((m.apply(b, 0.5) - b).abs() < 1e-6, "{m:?} at the gray's middle moved {b}");
        }
        assert!((BlendMode::Multiply.apply(b, 1.0) - b).abs() < 1e-6);
        assert!((BlendMode::Screen.apply(b, 0.0) - b).abs() < 1e-6);
        for &s in &steps {
            for m in BlendMode::ALL {
                let r = m.apply(b, s);
                assert!((0.0..=1.0).contains(&r), "{m:?}({b}, {s}) = {r}");
            }
            assert!(BlendMode::Multiply.apply(b, s) <= b + 1e-6);
            assert!(BlendMode::Screen.apply(b, s) >= b - 1e-6);
            assert!((BlendMode::HardLight.apply(b, s) - BlendMode::Overlay.apply(s, b)).abs() < 1e-6);
        }
    }
    // Hard light at the gray's ends: black makes black, white makes white.
    assert_eq!(BlendMode::HardLight.apply(0.7, 0.0), 0.0);
    assert!((BlendMode::HardLight.apply(0.3, 1.0) - 1.0).abs() < 1e-6);
}

/// The layer's gray: the statistic's center is the gray's middle, the contrast its slope, clamped.
#[test]
fn the_layer_gray_is_centered_and_clamped() {
    for t in [BandTexture::Stripe, BandTexture::Triangle] {
        assert_eq!(layer_gray(t.center(), t.center(), LAYER_CONTRAST_DEFAULT), 0.5);
    }
    assert!((layer_gray(0.6, 0.5, 4.0) - 0.9).abs() < 1e-6);
    assert_eq!(layer_gray(0.9, 0.5, 4.0), 1.0);
    assert_eq!(layer_gray(0.0, 0.5, 4.0), 0.0);
}
