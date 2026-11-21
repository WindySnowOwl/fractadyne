// Smooth, written out as a coloring hook.
//
// Fractadyne calls custom_color once for every sample of the picture and shows what it returns
// (a display color, each channel 0 to 1). `l` holds the sample's lanes: l.count (the smooth
// iteration count), l.normal_x and l.normal_y (the slope's direction), l.de_log2 (log2 of the
// distance to the set in pixels), l.stripe, l.triangle, l.trap and l.angle (the orbit
// statistics), l.inside (in the set), l.coord (the view's method's palette position) and
// l.color (the color the view's method gives it).
//
// palette(t) is the view's palette at position t (it wraps every 1), palette_map(v) maps a value
// as the view's method maps its own (its rate, Offset and range map: for an iteration count), and
// palette_offset() is the Offset alone, for a value you map at your own rate (a statistic, a
// direction: palette(v * 2.0 + palette_offset())). p1() to p4() are the sliders.
// Save the file and the view recolors.

fn custom_color(l: Lanes) -> vec3<f32> {
    if (l.inside) {
        return l.color; // the view's own inside coloring
    }
    return palette(palette_map(l.count));
}
