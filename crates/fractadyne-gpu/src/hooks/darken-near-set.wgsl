// Darken near the set: the view's own colors, darkened within a few pixels of the set, so the
// filaments stand out. It works on top of any method, because it starts from l.color.
//
// @param p1 "Width (pixels)" 0.25 8 2
// @param p2 "Strength" 0 1 0.8

fn custom_color(l: Lanes) -> vec3<f32> {
    if (l.inside || l.de_log2 > 1.0e20) {
        return l.color;
    }
    let d = exp2(min(l.de_log2, 60.0));
    let t = clamp(d / p1(), 0.0, 1.0);
    return l.color * mix(1.0 - p2(), 1.0, t);
}
