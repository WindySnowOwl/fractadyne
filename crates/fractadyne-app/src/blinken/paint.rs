//! The panel's period parts, drawn with egui's painter (design/blinkenlights.md, "The look"): lamps
//! with an incandescent glow, phosphor screens, meter faces, arcs, engraved legends.
//!
//! egui has no blur, so every soft edge here is a mesh whose vertex colours fade to transparent: a
//! glow is a disc whose rim colour is the lamp's and whose outer ring is clear. Colours are built
//! with `from_rgba_unmultiplied` and scaled with [`fade`], so a half-lit lamp is half as opaque, not
//! a darker colour on top of the panel.

use egui::epaint::{Mesh, PathShape, PathStroke};
use egui::{Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Shape, Stroke, vec2};
use std::f32::consts::TAU;

/// The panel's palette: a slate hammertone case, engraved cream, lamp and phosphor colours.
pub(crate) mod ink {
    use egui::Color32;
    /// The hammertone's base, before its dimples are lit ([`super::hammertone`]).
    pub(crate) const HAMMER: Color32 = Color32::from_rgb(0x34, 0x39, 0x3f);
    pub(crate) const BEZEL: Color32 = Color32::from_rgb(0x3a, 0x3f, 0x45);
    pub(crate) const ENGRAVE: Color32 = Color32::from_rgb(0xe4, 0xdb, 0xc4);
    /// Every legend printed on the case (titles, lamp labels, the model tag). The hammertone is
    /// busier than the flat anodizing it replaced, so this stays at least 4.5:1 (WCAG AA for small
    /// text) against the case's brightest texels — checked by a unit test, not by eye.
    pub(crate) const ENGRAVE_DIM: Color32 = Color32::from_rgb(0xcc, 0xc2, 0xa8);
    pub(crate) const AMBER: Color32 = Color32::from_rgb(0xff, 0xa5, 0x3a);
    pub(crate) const RED: Color32 = Color32::from_rgb(0xff, 0x4a, 0x2a);
    pub(crate) const LAMP_GREEN: Color32 = Color32::from_rgb(0x8c, 0xff, 0x70);
    pub(crate) const LAMP_WHITE: Color32 = Color32::from_rgb(0xff, 0xf3, 0xd6);
    pub(crate) const LAMP_BLUE: Color32 = Color32::from_rgb(0x9a, 0xd8, 0xff);
    pub(crate) const PHOSPHOR: Color32 = Color32::from_rgb(0x7d, 0xff, 0xa0);
    pub(crate) const PHOSPHOR_TEXT: Color32 = Color32::from_rgb(0xa0, 0xff, 0xbe);
    pub(crate) const SCREEN_HI: Color32 = Color32::from_rgb(0x0f, 0x3a, 0x20);
    pub(crate) const SCREEN_LO: Color32 = Color32::from_rgb(0x03, 0x14, 0x0b);
    pub(crate) const CREAM: Color32 = Color32::from_rgb(0xef, 0xe5, 0xcb);
    pub(crate) const CREAM_LO: Color32 = Color32::from_rgb(0xdd, 0xd0, 0xad);
    pub(crate) const INK: Color32 = Color32::from_rgb(0x1b, 0x1a, 0x17);
    pub(crate) const METER_RED: Color32 = Color32::from_rgb(0xc4, 0x36, 0x1c);
    pub(crate) const NIXIE: Color32 = Color32::from_rgb(0xff, 0x9c, 0x4a);
    pub(crate) const NIXIE_GLOW: Color32 = Color32::from_rgb(0xff, 0x6a, 0x1a);
    pub(crate) const MAKER_RED: Color32 = Color32::from_rgb(0xd8, 0x40, 0x2a);
    pub(crate) const FACE_DARK: Color32 = Color32::from_rgb(0x12, 0x13, 0x15);
    /// Supersampling passes on the scope: lavender, so they never pass for moving frames' green.
    pub(crate) const SSAA: Color32 = Color32::from_rgb(0xc8, 0x9c, 0xff);
}

/// The panel's typefaces, registered in `theme.rs` (design/blinkenlights.md, "Type").
pub(crate) mod face {
    use egui::{FontFamily, FontId};
    /// Routed Gothic: engraved legends (a tracing of K&E Leroy lettering).
    pub(crate) const ENGRAVE: &str = "blinken-engrave";
    /// Routed Gothic Narrow: captions on the phosphor screens and the lamp labels.
    pub(crate) const ENGRAVE_NARROW: &str = "blinken-engrave-narrow";
    /// Barlow Condensed SemiBold: numerals printed on dial faces.
    pub(crate) const DIAL: &str = "blinken-dial";
    /// Barlow Condensed Bold Italic: the maker's mark.
    pub(crate) const MAKER: &str = "blinken-maker";
    /// Share Tech Mono: the nixie tubes.
    pub(crate) const NIXIE: &str = "blinken-nixie";

    pub(crate) fn id(name: &'static str, size: f32) -> FontId {
        FontId::new(size, FontFamily::Name(name.into()))
    }
    pub(crate) fn engrave(size: f32) -> FontId {
        id(ENGRAVE, size)
    }
    pub(crate) fn narrow(size: f32) -> FontId {
        id(ENGRAVE_NARROW, size)
    }
    pub(crate) fn dial(size: f32) -> FontId {
        id(DIAL, size)
    }
    pub(crate) fn maker(size: f32) -> FontId {
        id(MAKER, size)
    }
    pub(crate) fn nixie(size: f32) -> FontId {
        id(NIXIE, size)
    }
}

/// The hammertone tile's side, texels; one texel is one panel unit at scale 1.
pub(crate) const HAMMER_TILE: usize = 512;
/// The dents' mean radius, texels, and how many times over their discs cover the tile.
const HAMMER_DENT_R: f32 = 6.5;
const HAMMER_COVER: f32 = 1.6;
/// Height to slope: how strongly a dent's walls tilt toward the light.
const HAMMER_RELIEF: f32 = 0.45;
/// The light's share of brightness, and the metal flake's glint.
const HAMMER_GAIN: f32 = 0.5;
const HAMMER_SHEEN: f32 = 20.0;

/// A seamless square tile of hammertone paint, `n` texels a side: the hammered, faintly metallic
/// finish of a period instrument case. Small round dents of varied size, each a cap with a defined
/// rim (`(1 − d²/r²)²` inside its radius), are pressed into a flat height field round the tile's
/// edges so opposite edges meet; the result is lit from the
/// top left, with a glint where a dent's wall faces the light, and a fine speckle of metal flake.
/// Tuned in a prototype against the panel's legends (2026-10-20; soft Gaussian dents read as smoke
/// at 1:1, not as hammered paint): every legend on the case keeps
/// at least 4.5:1 against the tile's brightest texels once its halo ([`draw_galley`]) darkens
/// them, and the tile's seam is no rougher than its interior.
pub(crate) fn hammertone(n: usize, seed: u64) -> egui::ColorImage {
    let mut rng = seed ^ 0x9e37_79b9_7f4a_7c15;
    let mut next = move || {
        rng ^= rng >> 12;
        rng ^= rng << 25;
        rng ^= rng >> 27;
        (rng.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 11) as f32 / (1u64 << 53) as f32
    };
    let nf = n as f32;
    let mut height = vec![0.0f32; n * n];
    let count = (HAMMER_COVER * nf * nf / (std::f32::consts::PI * HAMMER_DENT_R * HAMMER_DENT_R)) as usize;
    for _ in 0..count {
        let (cx, cy) = (next() * nf, next() * nf);
        let r = HAMMER_DENT_R * (0.7 + 0.6 * next());
        let depth = r * (0.7 + 0.6 * next());
        let reach = r as i64 + 1;
        let inv = 1.0 / (r * r);
        for yi in cy as i64 - reach..=cy as i64 + reach {
            let dy = yi as f32 + 0.5 - cy;
            let row = yi.rem_euclid(n as i64) as usize * n;
            for xi in cx as i64 - reach..=cx as i64 + reach {
                let dx = xi as f32 + 0.5 - cx;
                let cap = (1.0 - (dx * dx + dy * dy) * inv).max(0.0);
                height[row + xi.rem_euclid(n as i64) as usize] -= depth * cap * cap;
            }
        }
    }
    let h = |x: usize, y: usize| height[(y % n) * n + (x % n)];
    // Lit from the top left (egui's y runs down), with the viewer straight on.
    let unit = |x: f32, y: f32, z: f32| {
        let l = (x * x + y * y + z * z).sqrt();
        (x / l, y / l, z / l)
    };
    let light = unit(-0.55, -0.65, 0.52);
    let half = unit(light.0, light.1, light.2 + 1.0);
    let base = ink::HAMMER.to_srgba_unmultiplied();
    let mut pixels = Vec::with_capacity(n * n);
    for y in 0..n {
        for x in 0..n {
            let gx = (h(x + 1, y) - h(x + n - 1, y)) * 0.5 * HAMMER_RELIEF;
            let gy = (h(x, y + 1) - h(x, y + n - 1)) * 0.5 * HAMMER_RELIEF;
            let nrm = unit(-gx, -gy, 1.0);
            let diffuse = (nrm.0 * light.0 + nrm.1 * light.1 + nrm.2 * light.2).max(0.0);
            let glint = (nrm.0 * half.0 + nrm.1 * half.1 + nrm.2 * half.2).max(0.0).powi(28);
            let flake = (next() - 0.5) * 0.05;
            // A flat texel's diffuse is `light.2`: the base colour, as specified.
            let k = 1.0 + HAMMER_GAIN * (diffuse - light.2) + flake;
            let c = |v: u8, tint: f32| (v as f32 * k + HAMMER_SHEEN * glint * tint).round().clamp(0.0, 255.0) as u8;
            pixels.push(Color32::from_rgb(c(base[0], 0.95), c(base[1], 1.0), c(base[2], 1.08)));
        }
    }
    egui::ColorImage { size: [n, n], pixels }
}

/// `c` with its alpha scaled by `k` (0..1), unmultiplied, so a dimmer lamp is more transparent.
pub(crate) fn fade(c: Color32, k: f32) -> Color32 {
    let [r, g, b, a] = c.to_srgba_unmultiplied();
    Color32::from_rgba_unmultiplied(r, g, b, (a as f32 * k.clamp(0.0, 1.0)).round() as u8)
}

/// Linear mix of two colours (`t` = 0 gives `a`), alpha included.
pub(crate) fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let (a, b) = (a.to_srgba_unmultiplied(), b.to_srgba_unmultiplied());
    let m = |i: usize| (a[i] as f32 + (b[i] as f32 - a[i] as f32) * t).round() as u8;
    Color32::from_rgba_unmultiplied(m(0), m(1), m(2), m(3))
}

/// Segments for a circle of radius `r` on screen: enough that the polygon never shows.
fn segments(r: f32) -> usize {
    ((r * 0.9).ceil() as usize).clamp(12, 96)
}

/// A filled disc whose colour runs from `inner` at the centre to `outer` at radius `r`.
pub(crate) fn radial_disc(p: &Painter, c: Pos2, r: f32, inner: Color32, outer: Color32) {
    let n = segments(r);
    let mut m = Mesh::default();
    m.colored_vertex(c, inner);
    for i in 0..n {
        let a = i as f32 / n as f32 * TAU;
        m.colored_vertex(c + vec2(a.cos(), a.sin()) * r, outer);
    }
    for i in 0..n as u32 {
        m.add_triangle(0, 1 + i, 1 + (i + 1) % n as u32);
    }
    p.add(Shape::mesh(m));
}

/// A ring from radius `r0` to `r1`, colour `c0` at its inner edge fading to `c1` at its outer one:
/// the halo of a glow, or the afterglow band of a phosphor arc.
pub(crate) fn radial_ring(p: &Painter, c: Pos2, r0: f32, r1: f32, c0: Color32, c1: Color32) {
    let n = segments(r1);
    let mut m = Mesh::default();
    for i in 0..n {
        let a = i as f32 / n as f32 * TAU;
        let d = vec2(a.cos(), a.sin());
        m.colored_vertex(c + d * r0, c0);
        m.colored_vertex(c + d * r1, c1);
    }
    let n = n as u32;
    for i in 0..n {
        let (a, b) = (2 * i, 2 * ((i + 1) % n));
        m.add_triangle(a, a + 1, b);
        m.add_triangle(b, a + 1, b + 1);
    }
    p.add(Shape::mesh(m));
}

/// A rectangle filled with a vertical gradient, `top` to `bottom`.
pub(crate) fn vgradient(p: &Painter, r: Rect, top: Color32, bottom: Color32) {
    let mut m = Mesh::default();
    m.colored_vertex(r.left_top(), top);
    m.colored_vertex(r.right_top(), top);
    m.colored_vertex(r.right_bottom(), bottom);
    m.colored_vertex(r.left_bottom(), bottom);
    m.add_triangle(0, 1, 2);
    m.add_triangle(0, 2, 3);
    p.add(Shape::mesh(m));
}

/// An incandescent panel lamp's bezel, radius `r`: it does not change while the lamp burns, so
/// it is in the panel's static layer.
pub(crate) fn lamp_bezel(p: &Painter, c: Pos2, r: f32) {
    p.circle_filled(c, r + 3.0, ink::BEZEL);
    p.circle_filled(c, r + 2.0, Color32::from_rgb(0x12, 0x13, 0x15));
}

/// The lamp's glass at brightness `lit` (0..1), in the panel's dynamic layer: a bead that warms
/// from a dark tint of `hue` to the full colour, with a halo past its rim as it brightens.
pub(crate) fn lamp_glass(p: &Painter, c: Pos2, r: f32, hue: Color32, lit: f32) {
    let lit = lit.clamp(0.0, 1.0);
    if lit > 0.02 {
        radial_ring(p, c, r + 1.5, r + 1.5 + 9.0 * lit, fade(hue, 0.55 * lit), fade(hue, 0.0));
    }
    let dark = mix(Color32::from_rgb(0x2b, 0x2b, 0x2b), hue, 0.18);
    let body = mix(dark, hue, lit);
    let core = mix(mix(dark, hue, 0.35), mix(hue, Color32::WHITE, 0.35), lit);
    radial_disc(p, c + vec2(-r * 0.2, -r * 0.25), r, core, body);
}

/// A green-phosphor screen face: dark at the rim, a faint glow at the centre.
pub(crate) fn phosphor_disc(p: &Painter, c: Pos2, r: f32) {
    radial_disc(p, c, r, ink::SCREEN_HI, ink::SCREEN_LO);
}

/// A rectangular phosphor screen with rounded corners.
pub(crate) fn phosphor_rect(p: &Painter, r: Rect) {
    p.rect_filled(r, 6.0, ink::SCREEN_LO);
    radial_disc(p, r.center(), r.width().min(r.height()) * 0.7, fade(ink::SCREEN_HI, 0.8), fade(ink::SCREEN_HI, 0.0));
}

/// An instrument's bezel: the dark well, a hairline shadow and a raised lip.
pub(crate) fn bezel(p: &Painter, r: Rect) {
    p.rect_filled(r.expand(3.0), 8.0, Color32::from_rgb(0x1a, 0x1c, 0x1f));
    p.rect_filled(r.expand(2.0), 7.0, ink::BEZEL);
    p.rect_filled(r, 6.0, Color32::from_rgb(0x0b, 0x0c, 0x0d));
}

/// Points along an arc of radius `r` from angle `a0` to `a1` (radians, clockwise from +x on
/// screen, as egui's y runs down).
pub(crate) fn arc_points(c: Pos2, r: f32, a0: f32, a1: f32) -> Vec<Pos2> {
    let n = (((a1 - a0).abs() * r / 3.0).ceil() as usize).clamp(2, 256);
    (0..=n).map(|i| {
        let a = a0 + (a1 - a0) * i as f32 / n as f32;
        c + vec2(a.cos(), a.sin()) * r
    }).collect()
}

/// A stroked arc.
pub(crate) fn arc(p: &Painter, c: Pos2, r: f32, a0: f32, a1: f32, stroke: Stroke) {
    p.add(PathShape::line(arc_points(c, r, a0, a1), PathStroke::new(stroke.width, stroke.color)));
}

/// A filled annular sector between radii `r0` and `r1`, angles `a0` to `a1`.
pub(crate) fn sector(p: &Painter, c: Pos2, r0: f32, r1: f32, a0: f32, a1: f32, fill: Color32) {
    let outer = arc_points(c, r1, a0, a1);
    let inner = arc_points(c, r0, a0, a1);
    let mut m = Mesh::default();
    for (o, i) in outer.iter().zip(inner.iter()) {
        m.colored_vertex(*o, fill);
        m.colored_vertex(*i, fill);
    }
    for k in 0..(outer.len() as u32 - 1) {
        let (a, b) = (2 * k, 2 * k + 2);
        m.add_triangle(a, a + 1, b);
        m.add_triangle(b, a + 1, b + 1);
    }
    p.add(Shape::mesh(m));
}

/// The point at angle `a` and radius `r` from `c`.
pub(crate) fn polar(c: Pos2, r: f32, a: f32) -> Pos2 {
    c + vec2(a.cos(), a.sin()) * r
}

/// The alpha of the dark halo round an engraved legend ([`draw_galley`]).
pub(crate) const HALO_ALPHA: u8 = 110;

/// Draws a laid-out galley at `rect.min`; engraved, as paint-filled lettering cut into the case: a
/// dark halo a pixel round it, so a stroke never lands on a bright glint of the hammertone
/// unseparated, then a dark lip above and a faint highlight below. One layout serves every draw.
fn draw_galley(p: &Painter, at: Pos2, galley: std::sync::Arc<egui::Galley>, color: Color32, engrave: bool) {
    if engrave {
        for d in [vec2(-1.0, -1.0), vec2(1.0, -1.0), vec2(-1.0, 1.0), vec2(1.0, 1.0)] {
            p.galley_with_override_text_color(at + d, galley.clone(), Color32::from_black_alpha(HALO_ALPHA));
        }
        p.galley_with_override_text_color(at + vec2(0.0, -1.0), galley.clone(), Color32::from_black_alpha(190));
        p.galley_with_override_text_color(at + vec2(0.0, 1.0), galley.clone(), Color32::from_white_alpha(18));
    }
    p.galley_with_override_text_color(at, galley, color);
}

/// Text with extra space between letters (`TextFormat::extra_letter_spacing`): the panel's
/// tracked small caps, laid out once. Returns the rect it covers.
pub(crate) fn tracked(p: &Painter, pos: Pos2, anchor: Align2, text: &str, font: FontId, color: Color32, spacing: f32, engrave: bool) -> Rect {
    let mut job = egui::text::LayoutJob::default();
    job.append(text, 0.0, egui::TextFormat { font_id: font, color, extra_letter_spacing: spacing, ..Default::default() });
    let galley = p.layout_job(job);
    let rect = anchor.anchor_size(pos, galley.size());
    draw_galley(p, rect.min, galley, color, engrave);
    rect
}

/// Text with every digit in a cell as wide as a 0, so a changing number holds still (Routed
/// Gothic's 1 is narrower than its other digits): one layout, each digit narrower than a 0 its own
/// section, padded half before and half after so it sits centred in its cell.
pub(crate) fn tabular(p: &Painter, pos: Pos2, anchor: Align2, text: &str, font: FontId, color: Color32) -> Rect {
    let galley = p.layout_job(tabular_job(p, text, &font, color));
    let rect = anchor.anchor_size(pos, galley.size());
    p.galley_with_override_text_color(rect.min, galley, color);
    rect
}

/// [`tabular`] on a plate of `plate` reaching `pad` px past it either side: a readout on a scope's
/// screen that the beam or a bright trace passing under it cannot swallow (a radar's data block).
pub(crate) fn plated(p: &Painter, pos: Pos2, anchor: Align2, text: &str, font: FontId, color: Color32, plate: Color32, pad: f32) -> Rect {
    let galley = p.layout_job(tabular_job(p, text, &font, color));
    let rect = anchor.anchor_size(pos, galley.size());
    p.rect_filled(rect.expand2(vec2(pad, 0.0)), pad, plate);
    p.galley_with_override_text_color(rect.min, galley, color);
    rect
}

fn tabular_job(p: &Painter, text: &str, font: &FontId, color: Color32) -> egui::text::LayoutJob {
    let pad: [f32; 10] = p.fonts(|f| {
        let cell = f.glyph_width(font, '0');
        std::array::from_fn(|d| (cell - f.glyph_width(font, char::from(b'0' + d as u8))).max(0.0))
    });
    let format = |spacing: f32| egui::text::TextFormat { font_id: font.clone(), color, extra_letter_spacing: spacing, ..Default::default() };
    let mut job = egui::text::LayoutJob::default();
    let mut run = String::new();
    for ch in text.chars() {
        let d = ch.to_digit(10).map_or(0.0, |d| pad[d as usize]);
        if d > 0.01 {
            if !run.is_empty() {
                job.append(&std::mem::take(&mut run), 0.0, format(0.0));
            }
            job.append(&ch.to_string(), d * 0.5, format(d * 0.5));
        } else {
            run.push(ch);
        }
    }
    if !run.is_empty() {
        job.append(&run, 0.0, format(0.0));
    }
    job
}

/// A row of characters each centred in a cell of its own width (`cells`), laid out once: the
/// nixie tubes' digits. Returns the galley's rect.
pub(crate) fn celled(p: &Painter, at: Pos2, text: &str, cells: &[f32], font: FontId, color: Color32, glow: Color32, s: f32) -> Rect {
    let widths: Vec<f32> = p.fonts(|f| text.chars().map(|ch| f.glyph_width(&font, ch)).collect());
    let mut job = egui::text::LayoutJob::default();
    let mut carry = 0.0;
    for ((ch, w), cell) in text.chars().zip(widths).zip(cells.iter()) {
        let slack = (cell - w).max(0.0);
        job.append(&ch.to_string(), carry + slack * 0.5, egui::text::TextFormat { font_id: font.clone(), color, ..Default::default() });
        carry = slack * 0.5;
    }
    let galley = p.layout_job(job);
    let rect = Rect::from_min_size(at, galley.size());
    p.galley_with_override_text_color(at + vec2(0.8, 0.4) * s, galley.clone(), glow);
    p.galley_with_override_text_color(at, galley, color);
    rect
}

/// Registers the panel's typefaces as named families, each followed by the proportional list so a
/// glyph it lacks (×, ·, superscript digits) still resolves. Called from `theme::install_fonts`.
pub(crate) fn install_fonts(fonts: &mut egui::FontDefinitions) {
    use std::sync::Arc;
    let fallback: Vec<String> = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    for (family, file, bytes) in [
        (face::ENGRAVE, "RoutedGothic", &include_bytes!("../../assets/fonts/RoutedGothic.ttf")[..]),
        (face::ENGRAVE_NARROW, "RoutedGothicNarrow", &include_bytes!("../../assets/fonts/RoutedGothicNarrow.ttf")[..]),
        (face::DIAL, "BarlowCondensed-SemiBold", &include_bytes!("../../assets/fonts/BarlowCondensed-SemiBold.ttf")[..]),
        (face::MAKER, "BarlowCondensed-BoldItalic", &include_bytes!("../../assets/fonts/BarlowCondensed-BoldItalic.ttf")[..]),
        (face::NIXIE, "ShareTechMono", &include_bytes!("../../assets/fonts/ShareTechMono.ttf")[..]),
    ] {
        fonts.font_data.insert(file.to_owned(), Arc::new(egui::FontData::from_static(bytes)));
        let mut list = vec![file.to_owned()];
        list.extend(fallback.iter().cloned());
        fonts.families.insert(FontFamily::Name(family.into()), list);
    }
}

#[cfg(test)]
#[path = "../../tests/unit/blinken/paint/tests.rs"]
mod tests;
