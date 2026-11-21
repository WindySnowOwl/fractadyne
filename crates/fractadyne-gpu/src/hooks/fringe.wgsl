// The fringe, written out as a coloring hook: dark lines along the set on a light field whose
// colors turn once around every feature. The palette gives the hue, from the slope's direction,
// and the distance to the set gives the lightness.
//
// @param p1 "Line width (pixels)" 0.25 4 1
// @param p2 "Sweeps (palettes a turn)" 1 4 1

fn custom_color(l: Lanes) -> vec3<f32> {
    if (l.inside || l.de_log2 > 1.0e20) {
        return l.color; // in the set, or no distance here
    }
    // The slope's direction in turns, 0 to 1.
    let turn = fract(atan2(l.normal_y, l.normal_x) * 0.15915494309 + 1.0);
    // Dark at the set, light from p1 pixels away.
    let d = exp2(min(l.de_log2, 60.0));
    let lightness = 0.12 + 0.78 * sqrt(clamp(d / p1(), 0.0, 1.0));
    // The palette's color at that lightness (OKLab), its hue kept: the palette swept a whole
    // number of times a turn, so the turn closes on itself, from the view's Offset.
    return with_lightness(palette(turn * round(p2()) + palette_offset()), lightness);
}
