use super::*;

const H: f64 = 1.2; // the default light height (Relief)
const ANGLE: f64 = 2.2; // the default light angle

/// A slope of 1 everywhere is the classic relief, (n·l + h) / (1 + h), and the slope from the
/// distance only changes how steep the surface is: 45 degrees `steep_px` from the set.
#[test]
fn a_slope_of_one_is_the_classic_relief() {
    for h in [0.1f64, 0.5, 1.2, 4.0] {
        for dn in [-1.0f64, -0.4, 0.0, 0.3, 1.0] {
            let classic = ((dn + h) / (1.0 + h)).clamp(0.0, 1.0);
            assert_eq!(relief_diffuse(dn, relief_slope(3.0, 0.0), h), classic, "h {h}, n.l {dn}");
        }
    }
    assert_eq!(relief_slope(8.0, 8.0), 1.0);
    assert_eq!(relief_slope(16.0, 8.0), 0.5);
    assert_eq!(relief_slope(1.0e-40, 8.0), RELIEF_SLOPE_MAX);
    assert_eq!(relief_slope(f64::INFINITY, 8.0), 0.0, "no distance: flat");
}

/// From the distance, a face turned to the light is lit fully at any slope, the far background is
/// evenly lit (the slope goes to 0), and near the set the shade is the light's horizontal part
/// alone, so the edges facing away go dark.
#[test]
fn the_slope_from_the_distance_lights_the_background_evenly() {
    for d in [0.5, 2.0, 8.0, 64.0, 4096.0] {
        assert!((relief_diffuse(1.0, relief_slope(d, 8.0), H) - 1.0).abs() < 1e-12, "a face to the light at {d} px");
    }
    let far = relief_diffuse(-1.0, relief_slope(1.0e6, 8.0), H);
    assert!(far > 0.99, "a face away from the light far from the set: {far}");
    let near = relief_diffuse(-0.5, relief_slope(1.0e-3, 8.0), H);
    assert!(near < 1.0e-3, "a face away from the light at the set: {near}");
    let classic_far = relief_diffuse(-1.0, relief_slope(1.0e6, 0.0), H);
    assert!(classic_far < 0.1, "the classic relief shades the background as steeply: {classic_far}");
}

/// The highlight peaks where the surface tilts halfway toward the light, `relief_highlight_at` from
/// the set on a slope facing the light, and is 1 there; a flat surface has only the lobe's tail
/// (`shininess` 64: about 2% at the default light).
#[test]
fn the_highlight_sits_where_the_surface_tilts_halfway_to_the_light() {
    let facing = [ANGLE.cos(), ANGLE.sin()];
    for (steep, shin) in [(8.0, 64.0), (4.0, 24.0), (32.0, 128.0)] {
        let want = relief_highlight_at(steep, H);
        let at = |d: f64| relief_highlight(facing, relief_slope(d, steep), ANGLE, H, shin);
        // The peak over a fine log scan of distances.
        let (mut best_d, mut best) = (0.0, -1.0);
        for i in 0..4000 {
            let d = 0.1 * (1.0e4f64).powf(i as f64 / 4000.0);
            let s = at(d);
            if s > best {
                (best_d, best) = (d, s);
            }
        }
        assert!((best - 1.0).abs() < 1e-3, "steep {steep}: the peak is {best}");
        assert!((best_d / want - 1.0).abs() < 0.01, "steep {steep}: the peak at {best_d} px, want {want}");
    }
    let flat = relief_highlight(facing, 0.0, ANGLE, H, 64.0);
    assert!(flat > 0.005 && flat < 0.05, "the flat background's sheen at shininess 64: {flat}");
    let away = relief_highlight([-facing[0], -facing[1]], relief_slope(relief_highlight_at(8.0, H), 8.0), ANGLE, H, 64.0);
    assert!(away < 1.0e-6, "a slope facing away has none: {away}");
    // A surface of one slope at 45 degrees never tilts halfway to the default light: no highlight
    // worth the name anywhere.
    let best45 = (0..360)
        .map(|deg| {
            let a = (deg as f64).to_radians();
            relief_highlight([a.cos(), a.sin()], 1.0, ANGLE, H, 64.0)
        })
        .fold(0.0, f64::max);
    assert!(best45 < 0.01, "45 degrees everywhere: at most {best45}");
}
