//! SVG export of an L-system view (design/lsystems.md §7, §9.5): what the view shows, as vectors —
//! its lines as paths and its filled shapes as polygons, in the colours on screen — for print or a
//! plotter. The walk is the screen's (culled to the view, subtrees under a pixel as their chords),
//! so the file holds what the pixels show and is bounded by them, at any zoom.

use fractadyne_gpu::lsystem::SegmentInstance;
use std::fmt::Write;

/// What to write.
pub(crate) struct Picture<'a> {
    /// The view, pixels.
    pub size: [u32; 2],
    /// The line width, pixels.
    pub width: f32,
    /// The lines, in the walk's pixels (from the view's centre, y up), in curve order.
    pub segments: &'a [SegmentInstance],
    /// The filled shapes (clipped to the view), and their palette values.
    pub polygons: &'a [(Vec<[f32; 2]>, f32)],
    /// A palette value's colour.
    pub colour: &'a dyn Fn(f32) -> [u8; 3],
    pub background: [u8; 3],
    pub title: &'a str,
}

/// A coordinate to 0.01 px: the view's y-up, centred pixels to SVG's y-down from the corner.
fn coord(p: [f32; 2], size: [u32; 2]) -> (i64, i64) {
    let x = f64::from(p[0]) + 0.5 * f64::from(size[0]);
    let y = 0.5 * f64::from(size[1]) - f64::from(p[1]);
    ((x * 100.0).round() as i64, (y * 100.0).round() as i64)
}

/// A hundredth-pixel count as text: `1234` → `12.34`, `-5` → `-0.05`, `700` → `7`.
fn num(v: i64) -> String {
    let (sign, a) = if v < 0 { ("-", -v) } else { ("", v) };
    match a % 100 {
        0 => format!("{sign}{}", a / 100),
        f if f % 10 == 0 => format!("{sign}{}.{}", a / 100, f / 10),
        f => format!("{sign}{}.{f:02}", a / 100),
    }
}

fn hex(c: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// The SVG document. Consecutive segments that meet, in the same colour, are one path — a curve
/// coloured by position along it changes colour every few segments, not every one — so a drawing
/// of a million segments is not a million elements.
pub(crate) fn document(p: &Picture) -> String {
    let [w, h] = p.size;
    let mut s = String::new();
    let _ = writeln!(s, r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    let _ = writeln!(s, r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#);
    let _ = writeln!(s, "<title>{}</title>", escape(p.title));
    let _ = writeln!(s, r#"<rect width="{w}" height="{h}" fill="{}"/>"#, hex(p.background));
    // The filled shapes under the lines, as the screen draws them.
    if !p.polygons.is_empty() {
        let _ = writeln!(s, r#"<g stroke="none">"#);
        for (pts, value) in p.polygons {
            let points: Vec<String> = pts
                .iter()
                .map(|&q| {
                    let (x, y) = coord(q, p.size);
                    format!("{},{}", num(x), num(y))
                })
                .collect();
            let _ = writeln!(s, r#"<polygon points="{}" fill="{}"/>"#, points.join(" "), hex((p.colour)(*value)));
        }
        let _ = writeln!(s, "</g>");
    }
    let _ = writeln!(
        s,
        r#"<g fill="none" stroke-width="{}" stroke-linecap="round" stroke-linejoin="round">"#,
        num((f64::from(p.width) * 100.0).round() as i64)
    );
    // The open path: its colour, where it ends, and its `d`.
    let mut open: Option<([u8; 3], (i64, i64), String)> = None;
    let close = |s: &mut String, o: Option<([u8; 3], (i64, i64), String)>| {
        if let Some((c, _, d)) = o {
            let _ = writeln!(s, r#"<path stroke="{}" d="{d}"/>"#, hex(c));
        }
    };
    for g in p.segments {
        let (a, b) = (coord(g.a, p.size), coord(g.b, p.size));
        let c = (p.colour)(g.value);
        match &mut open {
            Some((oc, end, d)) if *oc == c && *end == a => {
                let _ = write!(d, " L{} {}", num(b.0), num(b.1));
                *end = b;
            }
            _ => {
                close(&mut s, open.take());
                open = Some((c, b, format!("M{} {} L{} {}", num(a.0), num(a.1), num(b.0), num(b.1))));
            }
        }
    }
    close(&mut s, open.take());
    let _ = writeln!(s, "</g>");
    let _ = writeln!(s, "</svg>");
    s
}

#[cfg(test)]
mod tests;
