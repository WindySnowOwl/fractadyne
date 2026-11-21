// Smooth, written out as a coloring hook.
//
// Fractadyne calls custom_color once for every sample of the picture and shows what it returns
// (a display color, each channel 0 to 1). `l` holds the sample's lanes: l.count (the smooth
// iteration count), l.normal_x and l.normal_y (the slope's direction), l.de_log2 (log2 of the
// distance to the set in pixels), l.stripe, l.triangle, l.trap and l.angle (the orbit
// statistics), l.inside (in the set), l.coord (the view's method's palette position) and
// l.color (the color the view's method gives it).
//
// palette(t) is the view's palette at position t (it wraps every 1), palette_map(v) applies the
// view's Cycle, Offset and range map to a value, and p1() to p4() are the sliders.
// Save the file and the view recolors.

fn custom_color(l: Lanes) -> vec3<f32> {
    if (l.inside) {
        return l.color; // the view's own inside coloring
    }
    return palette(palette_map(l.count));
}
