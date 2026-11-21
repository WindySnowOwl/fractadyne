// Transfer curves: the smooth count bent by a curve before the palette, then optionally cut
// into bands. A curve flattening the high counts spreads the palette over the outer bands; a
// steep one gives the deep ones more of it.
//
// @param p1 "Curve: 0 linear, 1 sqrt, 2 cube root, 3 log, 4 atan" 0 4 1
// @param p2 "Bands (0 = smooth)" 0 32 0

fn custom_color(l: Lanes) -> vec3<f32> {
    if (l.inside) {
        return l.color;
    }
    let n = max(l.count, 0.0);
    let curve = i32(round(p1()));
    var v = n;
    if (curve == 1) {
        v = sqrt(n);
    } else if (curve == 2) {
        v = pow(n, 1.0 / 3.0);
    } else if (curve == 3) {
        v = log(n + 1.0);
    } else if (curve == 4) {
        v = atan(n / 64.0) * 64.0;
    }
    var t = palette_map(v);
    let bands = round(p2());
    if (bands >= 1.0) {
        t = floor(t * bands) / bands;
    }
    return palette(t);
}
