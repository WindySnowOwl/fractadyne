//! The front panel, drawn (design/blinkenlights.md, part 2): the plate, the nixie row with its
//! clock, the walk dial, the picture-age meter, the tile plane, the GPU scope, the depth dial, the
//! light bank, and the engine room. Every size is in panel units (the mockup's CSS pixels) times
//! `s`, so the panel scales to fit the view.
//!
//! Each part draws in two layers. The STATIC layer (the case, faces, scales, titles, labels,
//! bezels) does not change from frame to frame: it is drawn once, tessellated into meshes and kept
//! ([`Statics`]), and later frames replay the meshes, until something it depends on changes. The
//! DYNAMIC layer (needles, lamps, digits, traces) is drawn every frame, over it. This keeps the
//! panel's cost on the UI thread within its budget (design/blinkenlights.md, "Cost"): egui
//! tessellates every shape every frame, and the static layer's thousands of glyphs were most of it.

use super::events::{PASSES, Pass, RefRole};
use super::model::{Engine, Lamp, Model, Plane, PuckState, STREAM_COLS, STREAM_ROWS, UNTIMED_AFTER_FRAMES, WalkKind};
use super::paint::{self, face, fade, ink, polar};
use egui::epaint::{ClippedShape, Mesh, PathShape, PathStroke};
use egui::{Align2, Color32, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};
use std::f32::consts::{PI, TAU};

/// The panel's width at scale 1.
pub(crate) const DESIGN_W: f32 = 980.0;

/// The panel's sizes, cycled by `B`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum Size {
    #[default]
    Off,
    /// The light bank and the nixie row.
    Strip,
    /// The plate, the nixies, the instruments, the dial and the bank.
    Panel,
    /// The panel and the engine room.
    Full,
}

impl Size {
    pub(crate) fn next(self) -> Size {
        match self {
            Size::Off => Size::Strip,
            Size::Strip => Size::Panel,
            Size::Panel => Size::Full,
            Size::Full => Size::Off,
        }
    }
    pub(crate) fn name(self) -> &'static str {
        match self {
            Size::Off => "off",
            Size::Strip => "strip",
            Size::Panel => "panel",
            Size::Full => "full",
        }
    }
    pub(crate) fn from_name(s: &str) -> Option<Size> {
        [Size::Off, Size::Strip, Size::Panel, Size::Full].into_iter().find(|z| z.name() == s)
    }
}

/// Help-card keys of the instruments and readouts (lamps are in [`LAMP_KEYS`]).
pub(crate) const HELP_KEYS: &[&str] =
    &["walk", "age", "tiles", "scope", "timers", "dial", "bank", "depth", "iter", "ms", "clock", "tach", "engines", "memory", "cores", "pixels"];

/// A lamp on the bank. Its engraved label is [`lamp_label`], in the language in use.
struct LampSpec {
    lamp: Lamp,
    key: &'static str,
    hue: Color32,
    /// A separator goes before this lamp.
    group: bool,
}

const LAMP_SPECS: [LampSpec; 19] = [
    LampSpec { lamp: Lamp::Plan, key: "lamp:plan", hue: ink::AMBER, group: false },
    LampSpec { lamp: Lamp::Dispatch, key: "lamp:dispatch", hue: ink::AMBER, group: false },
    LampSpec { lamp: Lamp::Live, key: "lamp:live", hue: ink::LAMP_GREEN, group: false },
    LampSpec { lamp: Lamp::Held, key: "lamp:held", hue: ink::AMBER, group: false },
    LampSpec { lamp: Lamp::Reproj, key: "lamp:reproj", hue: ink::AMBER, group: false },
    LampSpec { lamp: Lamp::Reading, key: "lamp:reading", hue: ink::LAMP_GREEN, group: false },
    LampSpec { lamp: Lamp::PinPass, key: "lamp:pass", hue: ink::AMBER, group: true },
    LampSpec { lamp: Lamp::Split, key: "lamp:split", hue: ink::AMBER, group: false },
    LampSpec { lamp: Lamp::Adopt, key: "lamp:adopt", hue: ink::LAMP_WHITE, group: false },
    LampSpec { lamp: Lamp::Abandon, key: "lamp:abandon", hue: ink::RED, group: false },
    LampSpec { lamp: Lamp::RefLive, key: "lamp:refLive", hue: ink::LAMP_GREEN, group: true },
    LampSpec { lamp: Lamp::RefAhead, key: "lamp:refAhead", hue: ink::AMBER, group: false },
    LampSpec { lamp: Lamp::RefHold, key: "lamp:refHold", hue: ink::LAMP_BLUE, group: false },
    LampSpec { lamp: Lamp::Series, key: "lamp:series", hue: ink::AMBER, group: false },
    LampSpec { lamp: Lamp::Worker, key: "lamp:worker", hue: ink::LAMP_BLUE, group: true },
    LampSpec { lamp: Lamp::Tile, key: "lamp:tile", hue: ink::AMBER, group: false },
    LampSpec { lamp: Lamp::Ssaa, key: "lamp:ssaa", hue: ink::LAMP_GREEN, group: false },
    LampSpec { lamp: Lamp::Palette, key: "lamp:palette", hue: ink::AMBER, group: false },
    LampSpec { lamp: Lamp::Mode, key: "lamp:mode", hue: ink::RED, group: false },
];

/// A lamp's engraved label, in capitals as the message holds it (the same message titles the lamp's
/// help card).
fn lamp_label(lamp: Lamp) -> String {
    match lamp {
        Lamp::Plan => tr!(live_lamp_plan),
        Lamp::Dispatch => tr!(live_lamp_dispatch),
        Lamp::Live => tr!(live_lamp_live),
        Lamp::Held => tr!(live_lamp_held),
        Lamp::Reproj => tr!(live_lamp_reproj),
        Lamp::Reading => tr!(live_lamp_reading),
        Lamp::PinPass => tr!(live_lamp_pin_pass),
        Lamp::Split => tr!(live_lamp_split),
        Lamp::Adopt => tr!(live_lamp_adopt),
        Lamp::Abandon => tr!(live_lamp_abandon),
        Lamp::RefLive => tr!(live_lamp_ref_live),
        Lamp::RefAhead => tr!(live_lamp_ref_ahead),
        Lamp::RefHold => tr!(live_lamp_ref_hold),
        Lamp::Series => tr!(live_lamp_series),
        Lamp::Worker => tr!(live_lamp_worker),
        Lamp::Tile => tr!(live_lamp_tile),
        Lamp::Ssaa => tr!(live_lamp_ssaa),
        Lamp::Palette => tr!(live_lamp_palette),
        Lamp::Mode => tr!(live_lamp_mode),
    }
}

/// Help-card keys of the lamps, left to right.
pub(crate) const LAMP_KEYS: &[&str] = &[
    "lamp:plan", "lamp:dispatch", "lamp:live", "lamp:held", "lamp:reproj", "lamp:reading", "lamp:pass", "lamp:split", "lamp:adopt", "lamp:abandon",
    "lamp:refLive", "lamp:refAhead", "lamp:refHold", "lamp:series", "lamp:worker", "lamp:tile", "lamp:ssaa", "lamp:palette", "lamp:mode",
];

/// A region of the panel with a help card.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Hit {
    pub(crate) rect: Rect,
    pub(crate) key: &'static str,
}

/// What the panel reads besides the model.
pub(crate) struct Inputs<'a> {
    pub(crate) model: &'a Model,
    pub(crate) engine: &'a Engine,
    /// `yyyy-mm-dd_hh:mm:ss.s` ('_' is the gap between date and time).
    pub(crate) clock: &'a str,
    pub(crate) version: &'a str,
    /// The depth (log2 magnification) where floatexp takes over, and where df32 does.
    pub(crate) fe_from: f32,
    pub(crate) df32_from: f32,
    /// The case's hammertone tile ([`paint::hammertone`]), uploaded once with a repeating sampler.
    pub(crate) hammer: egui::TextureId,
}

/// Which of the two layers a drawing call is making.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Layer {
    /// What does not change from frame to frame: drawn once and replayed.
    Static,
    /// What does: drawn every frame, over the static layer.
    Dynamic,
}

/// What the static layer was drawn for: while all of it stays the same, its shapes are replayed.
#[derive(Clone, Copy, Debug, PartialEq)]
struct StaticKey {
    at: Pos2,
    width: f32,
    s: f32,
    size: Size,
    opacity: f32,
    layer: egui::LayerId,
    /// The depth dial's window (its scale moves in 50-octave steps).
    dial_lo: i32,
    fe_from: f32,
    df32_from: f32,
    cores: usize,
    worker: bool,
    /// Each nixie readout's length and where its separators sit (they set the tubes).
    nixies: [(usize, u64); 4],
    /// The font atlas's size and the display's scale: the kept meshes' glyph coordinates are
    /// normalized to the atlas, so a grown atlas must re-tessellate them.
    atlas: [usize; 2],
    pixels_per_point: f32,
    /// The interface language's generation (`i18n::generation`): the legends are text in the
    /// language in use, so a change of language redraws them.
    language: u64,
}

/// The static layer, tessellated, kept between frames.
#[derive(Default)]
pub(crate) struct Statics {
    key: Option<StaticKey>,
    shapes: Vec<ClippedShape>,
    /// How many times the static layer was drawn (rather than replayed), for the cost line.
    pub(crate) drawn: u64,
}

// Layout at scale 1, in panel units.
const PAD_X: f32 = 18.0;
const PAD_TOP: f32 = 14.0;
const PAD_BOTTOM: f32 = 14.0;
const GAP: f32 = 12.0;
const PLATE_H: f32 = 22.0;
const TITLE_H: f32 = 18.0;
const NIXIE_H: f32 = 30.0;
const ROW1_H: f32 = 172.0;
const DIAL_H: f32 = 96.0;
const LAMP_CELL_W: f32 = 58.0;
const LAMP_ROW_H: f32 = 50.0;
const LAMP_SEP_W: f32 = 14.0;
const ENGINE_H: f32 = 150.0;

/// The light bank's rows at an inner width (lamps wrap).
fn bank_rows(inner_w: f32) -> u32 {
    let mut rows = 1;
    let mut x = 0.0;
    for spec in LAMP_SPECS.iter() {
        let w = LAMP_CELL_W + if spec.group && x > 0.0 { LAMP_SEP_W } else { 0.0 };
        if x + LAMP_CELL_W > inner_w + 0.5 && x > 0.0 {
            rows += 1;
            x = 0.0;
        }
        x += if x == 0.0 { LAMP_CELL_W } else { w };
    }
    rows
}

/// The panel's height at scale 1 for a size and width.
pub(crate) fn height(size: Size, width: f32) -> f32 {
    let inner = width - 2.0 * PAD_X;
    let bank = bank_rows(inner) as f32 * LAMP_ROW_H;
    let nixies = TITLE_H + NIXIE_H;
    let body = match size {
        Size::Off => return 0.0,
        Size::Strip => nixies + GAP + bank,
        Size::Panel => PLATE_H + GAP + nixies + GAP + TITLE_H + ROW1_H + GAP + TITLE_H + DIAL_H + GAP + bank,
        Size::Full => PLATE_H + GAP + nixies + GAP + TITLE_H + ROW1_H + GAP + TITLE_H + DIAL_H + GAP + bank + GAP + TITLE_H + ENGINE_H,
    };
    PAD_TOP + body + PAD_BOTTOM
}

/// A price's colour: green when cheap, amber near the 20 ms target, red past it, brightening.
pub(crate) fn price_color(ms: f32, alpha: f32) -> Color32 {
    target_color(ms, super::model::PIN_TARGET_MS, alpha)
}

/// A pass's colour against the GPU time it was sized for: green when cheap, amber near the
/// target, red past it, brightening.
pub(crate) fn target_color(ms: f32, target_ms: f32, alpha: f32) -> Color32 {
    let p = (ms / target_ms.max(1.0)).clamp(0.0, 1.6);
    let hue = if p < 0.5 { 130.0 - p * 140.0 } else if p < 1.0 { 60.0 - (p - 0.5) * 70.0 } else { 18.0 - (p - 1.0) * 20.0 };
    let light = 0.52 + p.min(1.0) * 0.14;
    fade(hsl(hue, 1.0, light), alpha)
}

/// HSL (hue in degrees) to an opaque colour.
fn hsl(h: f32, s: f32, l: f32) -> Color32 {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = (h.rem_euclid(360.0)) / 60.0;
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let (r, g, b) = match hp as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    let u = |v: f32| ((v + m).clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgb(u(r), u(g), u(b))
}

/// The panel's sections, for its cost line in the log: where its time goes. `static` is the
/// static layer (replayed, or drawn when it changed).
pub(crate) const SECTIONS: [&str; 11] = ["static", "nixies", "radar", "meter", "tiles", "scope", "dial", "bank", "steps", "engines", "rest"];

/// Adds the time since `t` to section `k` and restarts `t`.
fn lap(t: &mut std::time::Instant, cost: &mut [f64; 11], k: usize) {
    let now = std::time::Instant::now();
    cost[k] += (now - *t).as_secs_f64() * 1000.0;
    *t = now;
}

/// The depth dial's window: 200 octaves wide, from 0 while shallow, then stepping in 50s so the
/// view sits in its right half.
fn dial_lo(depth: f64) -> f32 {
    let d = depth.max(0.0) as f32;
    if d < 150.0 { 0.0 } else { ((d - 100.0) / 50.0).floor() * 50.0 }
}

/// The nixie readouts' texts: depth, iterations, frame ms, clock.
fn nixie_texts(inp: &Inputs) -> [String; 4] {
    let snap = &inp.model.snap;
    [
        format!("{:05.1}", snap.depth.clamp(0.0, 9_999_999.0)),
        format!("{:>6}", snap.iterations.min(999_999_999)),
        format!("{:>4.1}", snap.frame_ms.clamp(0.0, 999.9)),
        inp.clock.to_owned(),
    ]
}

/// A readout's length and where its separators sit: what its tubes' layout depends on.
fn tube_pattern(text: &str) -> (usize, u64) {
    let mask = text.chars().take(64).enumerate().fold(0u64, |m, (i, ch)| if matches!(ch, '_' | '-' | ':' | '.') { m | (1 << i) } else { m });
    (text.chars().count(), mask)
}

/// Draws the panel with its top-left at `at`, `width` panel units wide at scale `s`, and returns
/// its rect; `hits` gets every region with a help card, and `cost` each section's ms. The static
/// layer comes from `statics` when nothing it depends on changed.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw(
    p: &Painter,
    at: Pos2,
    width: f32,
    s: f32,
    size: Size,
    opacity: f32,
    inp: &Inputs,
    hits: &mut Vec<Hit>,
    cost: &mut [f64; 11],
    statics: &mut Statics,
) -> Rect {
    let h = height(size, width);
    let rect = Rect::from_min_size(at, vec2(width, h) * s);
    if size == Size::Off {
        return rect;
    }
    let texts = nixie_texts(inp);
    let ctx = p.ctx();
    let key = StaticKey {
        at,
        width,
        s,
        size,
        opacity,
        layer: p.layer_id(),
        dial_lo: dial_lo(inp.model.snap.depth) as i32,
        fe_from: inp.fe_from,
        df32_from: inp.df32_from,
        cores: inp.engine.cores.len(),
        worker: inp.engine.worker,
        nixies: std::array::from_fn(|i| tube_pattern(&texts[i])),
        atlas: ctx.fonts(|f| f.texture_atlas().lock().size()),
        pixels_per_point: ctx.pixels_per_point(),
        language: crate::i18n::generation(),
    };
    let mut t = std::time::Instant::now();
    let layer = p.layer_id();
    if statics.key != Some(key) {
        let start = ctx.graphics(|g| g.get(layer).map_or(0, |l| l.next_idx().0));
        layout(p, Layer::Static, rect, width, s, size, inp, &texts, &mut Vec::new(), &mut [0.0; 11]);
        let shapes: Vec<ClippedShape> = ctx.graphics(|g| g.get(layer).map(|l| l.all_entries().skip(start).cloned().collect()).unwrap_or_default());
        // Drawn (and tessellated by egui) as shapes this frame; kept as meshes for the next ones.
        // The atlas is read after drawing: laying the text out may have grown it.
        statics.shapes = tessellate(ctx, shapes);
        statics.key = Some(StaticKey { atlas: ctx.fonts(|f| f.texture_atlas().lock().size()), ..key });
        statics.drawn += 1;
    } else {
        ctx.graphics_mut(|g| {
            let list = g.entry(layer);
            for cs in &statics.shapes {
                list.add(cs.clip_rect, cs.shape.clone());
            }
        });
    }
    lap(&mut t, cost, 0);
    layout(p, Layer::Dynamic, rect, width, s, size, inp, &texts, hits, cost);
    rect
}

/// Tessellates shapes as egui would this frame, into meshes egui then only has to copy.
fn tessellate(ctx: &egui::Context, shapes: Vec<ClippedShape>) -> Vec<ClippedShape> {
    let options = ctx.tessellation_options(|o| *o);
    let atlas = ctx.fonts(|f| f.texture_atlas());
    let (size, discs) = {
        let a = atlas.lock();
        (a.size(), a.prepared_discs())
    };
    egui::epaint::Tessellator::new(ctx.pixels_per_point(), options, size, discs)
        .tessellate_shapes(shapes)
        .into_iter()
        .filter_map(|cp| match cp.primitive {
            egui::epaint::Primitive::Mesh(m) if !m.is_empty() => Some(ClippedShape { clip_rect: cp.clip_rect, shape: Shape::mesh(m) }),
            _ => None,
        })
        .collect()
}

/// The panel's layout, for one layer.
#[allow(clippy::too_many_arguments)]
fn layout(p: &Painter, layer: Layer, rect: Rect, width: f32, s: f32, size: Size, inp: &Inputs, texts: &[String; 4], hits: &mut Vec<Hit>, cost: &mut [f64; 11]) {
    let mut t = std::time::Instant::now();
    if layer == Layer::Static {
        draw_case(p, rect, s, inp.hammer);
    }
    let inner_l = rect.left() + PAD_X * s;
    let inner_w = (width - 2.0 * PAD_X) * s;
    let mut y = rect.top() + PAD_TOP * s;
    if matches!(size, Size::Panel | Size::Full) {
        if layer == Layer::Static {
            draw_plate(p, Rect::from_min_size(pos2(inner_l, y), vec2(inner_w, PLATE_H * s)), s, inp);
        }
        y += (PLATE_H + GAP) * s;
    }
    lap(&mut t, cost, 10);
    draw_nixie_row(p, layer, Rect::from_min_size(pos2(inner_l, y), vec2(inner_w, (TITLE_H + NIXIE_H) * s)), s, texts, hits);
    y += (TITLE_H + NIXIE_H + GAP) * s;
    lap(&mut t, cost, 1);
    if matches!(size, Size::Panel | Size::Full) {
        let row = Rect::from_min_size(pos2(inner_l, y), vec2(inner_w, (TITLE_H + ROW1_H) * s));
        draw_row1(p, layer, row, s, inp, hits, cost);
        y += (TITLE_H + ROW1_H + GAP) * s;
        t = std::time::Instant::now();
        let dial = Rect::from_min_size(pos2(inner_l, y), vec2(inner_w, (TITLE_H + DIAL_H) * s));
        let face = Rect::from_min_max(pos2(dial.left(), dial.top() + TITLE_H * s), dial.right_bottom());
        if layer == Layer::Static {
            title(p, dial.left_top(), s, &tr!(live_title_dial), dial.width());
        } else {
            hits.push(Hit { rect: dial, key: "dial" });
        }
        draw_dial(p, layer, face, s, inp);
        y += (TITLE_H + DIAL_H + GAP) * s;
        lap(&mut t, cost, 6);
    }
    let bank_h = bank_rows(width - 2.0 * PAD_X) as f32 * LAMP_ROW_H * s;
    let bank = Rect::from_min_size(pos2(inner_l, y), vec2(inner_w, bank_h));
    draw_bank(p, layer, bank, s, inp.model, hits);
    y += bank_h;
    lap(&mut t, cost, 7);
    if size == Size::Full {
        y += GAP * s;
        if layer == Layer::Static {
            let ly = y - GAP * s * 0.5;
            p.line_segment([pos2(inner_l, ly), pos2(inner_l + inner_w, ly)], Stroke::new(1.0_f32, Color32::from_black_alpha(140)));
            p.line_segment([pos2(inner_l, ly + 1.0), pos2(inner_l + inner_w, ly + 1.0)], Stroke::new(1.0_f32, Color32::from_white_alpha(12)));
        }
        let row = Rect::from_min_size(pos2(inner_l, y), vec2(inner_w, (TITLE_H + ENGINE_H) * s));
        draw_engine_room(p, layer, row, s, inp, hits, cost);
        t = std::time::Instant::now();
    }
    lap(&mut t, cost, 10);
}

/// The panel's case: hammertone paint, a light falling off down it, a bevel and four screws.
fn draw_case(p: &Painter, r: Rect, s: f32, hammer: egui::TextureId) {
    p.rect_filled(r.translate(vec2(0.0, 6.0 * s)), 9.0 * s, Color32::from_black_alpha(90));
    // One texel a panel unit, so the dents scale with the panel; the tile repeats across it.
    let tile = paint::HAMMER_TILE as f32 * s;
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(r.width() / tile, r.height() / tile));
    p.add(egui::epaint::RectShape::filled(r, 7.0 * s, Color32::WHITE).with_texture(hammer, uv));
    // The light falls off down the case (clear of the rounded corners).
    let (top, bot, mid) = (r.top() + 7.0 * s, r.bottom() - 7.0 * s, r.top() + r.height() * 0.3);
    paint::vgradient(p, Rect::from_min_max(pos2(r.left() + 1.0, top), pos2(r.right() - 1.0, mid)), Color32::from_white_alpha(8), Color32::TRANSPARENT);
    paint::vgradient(p, Rect::from_min_max(pos2(r.left() + 1.0, mid), pos2(r.right() - 1.0, bot)), Color32::TRANSPARENT, Color32::from_black_alpha(55));
    p.rect_stroke(r, 7.0 * s, Stroke::new(1.0_f32, Color32::from_rgb(0x4a, 0x50, 0x57)), egui::StrokeKind::Inside);
    p.line_segment([r.left_top() + vec2(8.0, 1.5), r.right_top() + vec2(-8.0, 1.5)], Stroke::new(1.0_f32, Color32::from_white_alpha(16)));
    // Three screws: the top-right corner holds the close button (`close_center`, `draw_close`).
    for c in [r.left_top() + vec2(11.0, 11.0) * s, r.left_bottom() + vec2(11.0, -11.0) * s, r.right_bottom() + vec2(-11.0, -11.0) * s] {
        screw(p, c, 5.5 * s);
    }
}

/// Where the close button sits on a case drawn at `r` and scale `s`: in the top-right screw's place,
/// in the case's margin at every size (clear of the plate, the nixies and the bank).
pub(crate) fn close_center(r: Rect, s: f32) -> Pos2 {
    r.right_top() + vec2(-11.0, 11.0) * s
}

/// The close button: a small dark push-button with an engraved ×, red while the pointer is on it
/// (`hot`) and pressed in while held (`down`).
pub(crate) fn draw_close(p: &Painter, c: Pos2, s: f32, hot: bool, down: bool) {
    let r = 6.5 * s;
    p.circle_filled(c + vec2(0.0, 1.0 * s), r, Color32::from_black_alpha(120));
    let c = if down { c + vec2(0.0, 0.6 * s) } else { c };
    let (rim, dome, hi) = if hot {
        (Color32::from_rgb(0x5c, 0x14, 0x0b), Color32::from_rgb(0x9e, 0x24, 0x14), Color32::from_rgb(0xe0, 0x58, 0x3a))
    } else {
        (Color32::from_rgb(0x1e, 0x20, 0x22), Color32::from_rgb(0x34, 0x37, 0x3b), Color32::from_rgb(0x6c, 0x70, 0x76))
    };
    p.circle_filled(c, r, rim);
    paint::radial_disc(p, c + vec2(-0.25, -0.3) * r, r * 0.8, hi, dome);
    p.circle_stroke(c, r, Stroke::new(1.0_f32, Color32::from_rgb(0x2a, 0x2c, 0x2f)));
    let d = 2.4 * s;
    let x = Stroke::new((1.5 * s).max(1.0), if hot { Color32::WHITE } else { Color32::from_rgb(0xd2, 0xd5, 0xd8) });
    p.line_segment([c + vec2(-d, -d), c + vec2(d, d)], x);
    p.line_segment([c + vec2(-d, d), c + vec2(d, -d)], x);
}

fn screw(p: &Painter, c: Pos2, r: f32) {
    p.circle_filled(c + vec2(0.0, 1.0), r, Color32::from_black_alpha(120));
    paint::radial_disc(p, c + vec2(-r * 0.3, -r * 0.4), r, Color32::from_rgb(0xb9, 0xbc, 0xc0), Color32::from_rgb(0x3a, 0x3c, 0x3f));
    let d = vec2(0.82, 0.57) * r * 0.8;
    p.line_segment([c - d, c + d], Stroke::new(1.2_f32, Color32::from_rgb(0x2a, 0x2c, 0x2f)));
}

/// A tracked legend's size as [`paint::tracked`] lays it out, without drawing it.
fn tracked_size(p: &Painter, text: &str, font: egui::FontId, spacing: f32) -> egui::Vec2 {
    let mut job = egui::text::LayoutJob::default();
    job.append(text, 0.0, egui::TextFormat { font_id: font, extra_letter_spacing: spacing, ..Default::default() });
    p.layout_job(job).size()
}

/// The scale a legend is drawn at to fit `room`, `width(k)` being its width at scale `k`: 1 when it
/// fits (every English legend does), else smaller, down to 0.6, so a longer translation shrinks
/// rather than running into the next instrument. egui sets type in whole pixels, so the first guess
/// is measured and stepped down until it fits.
fn fit_scale(room: f32, width: impl Fn(f32) -> f32) -> f32 {
    let w = width(1.0);
    if w <= room {
        return 1.0;
    }
    let mut k = (room / w.max(1.0)).max(0.6);
    for _ in 0..4 {
        if k <= 0.6 || width(k) <= room {
            break;
        }
        k = (k * 0.95).max(0.6);
    }
    k
}

/// An instrument title's type at scale `k` of its own size (`s` the panel's scale).
fn title_type(s: f32, k: f32) -> (egui::FontId, f32) {
    (face::engrave(12.0 * s * k), 1.9 * s * k)
}

/// The scale an instrument title is drawn at to fit `room`.
fn title_scale(p: &Painter, s: f32, text: &str, room: f32) -> f32 {
    fit_scale(room, |k| {
        let (font, track) = title_type(s, k);
        tracked_size(p, text, font, track).x
    })
}

/// An instrument's engraved title: tracked capitals, `room` wide at most. The message holds the
/// capitals (uppercasing a translation gets Turkish i and German ß wrong).
fn title(p: &Painter, at: Pos2, s: f32, text: &str, room: f32) -> Rect {
    let (font, track) = title_type(s, title_scale(p, s, text, room));
    paint::tracked(p, at, Align2::LEFT_TOP, text, font, ink::ENGRAVE_DIM, track, true)
}

/// An instrument title's rect, without drawing it (the dynamic layer's help regions).
fn title_rect(p: &Painter, at: Pos2, s: f32, text: &str, room: f32) -> Rect {
    let (font, track) = title_type(s, title_scale(p, s, text, room));
    Rect::from_min_size(at, tracked_size(p, text, font, track))
}

fn draw_plate(p: &Painter, r: Rect, s: f32, inp: &Inputs) {
    let tag = format!("MODEL FD-B · {}", inp.version.to_uppercase());
    let tr = paint::tracked(p, pos2(r.right() - 4.0 * s, r.center().y), Align2::RIGHT_CENTER, &tag, face::engrave(11.0 * s), ink::ENGRAVE_DIM, 1.8 * s, true);
    // The maker's mark on its own enamelled badge: red on the hammertone itself would read at
    // 1.5–2:1.
    let mark = pos2(tr.left() - 18.0 * s, r.center().y);
    let mr = Align2::RIGHT_CENTER.anchor_size(mark, p.layout_no_wrap("Fractadyne".to_owned(), face::maker(19.0 * s), ink::MAKER_RED).size());
    let badge = mr.expand2(vec2(8.0, 2.0) * s);
    p.rect_filled(badge.translate(vec2(0.0, 1.0 * s)), 4.0 * s, Color32::from_black_alpha(120));
    p.rect_filled(badge, 4.0 * s, ink::FACE_DARK);
    p.rect_stroke(badge, 4.0 * s, Stroke::new(1.0_f32, Color32::from_rgb(0x55, 0x5a, 0x60)), egui::StrokeKind::Inside);
    p.text(mark, Align2::RIGHT_CENTER, "Fractadyne", face::maker(19.0 * s), ink::MAKER_RED);
    // The plate's legend, left, clear of the badge.
    let legend = tr!(live_plate);
    let left = r.left() + 4.0 * s;
    let k = fit_scale(badge.left() - 16.0 * s - left, |k| tracked_size(p, &legend, face::engrave(14.0 * s * k), 4.2 * s * k).x);
    paint::tracked(p, pos2(left, r.center().y), Align2::LEFT_CENTER, &legend, face::engrave(14.0 * s * k), ink::ENGRAVE, 4.2 * s * k, true);
}

/// A nixie readout's tube widths: ' ' an unlit tube, '_' a gap, and - : . small neon separators.
fn tube_widths(text: &str, s: f32) -> Vec<f32> {
    text.chars()
        .map(|ch| match ch {
            '_' => 9.0 * s,
            '-' | ':' | '.' => 8.0 * s,
            _ => 15.0 * s,
        })
        .collect()
}

/// One row of nixie tubes: the case and the tubes' glow in the static layer, the digits in the
/// dynamic one.
fn nixies(p: &Painter, layer: Layer, at: Pos2, s: f32, text: &str) -> Rect {
    let widths = tube_widths(text, s);
    let h = NIXIE_H * s;
    let inner: f32 = widths.iter().sum::<f32>() + 2.0 * s * (widths.len().saturating_sub(1)) as f32;
    let case = Rect::from_min_size(at, vec2(inner + 10.0 * s, h));
    let cy = case.center().y;
    if layer == Layer::Static {
        p.rect_filled(case.expand(2.0 * s), 6.0 * s, ink::BEZEL);
        paint::vgradient(p, case, Color32::from_rgb(0x14, 0x15, 0x16), Color32::from_rgb(0x0a, 0x0b, 0x0b));
        let mut x = case.left() + 5.0 * s;
        for (ch, w) in text.chars().zip(widths.iter()) {
            if !matches!(ch, '_' | '-' | ':' | '.') {
                paint::radial_disc(p, pos2(x + w * 0.5, cy), w * 0.75, fade(ink::NIXIE_GLOW, 0.10), fade(ink::NIXIE_GLOW, 0.0));
            }
            x += w + 2.0 * s;
        }
        return case;
    }
    // The digits, one layout: each centred in its tube (a cell of the tube's width and the gap).
    let font = face::nixie(20.0 * s);
    let shown: String = text.chars().map(|ch| if ch == '_' { ' ' } else { ch }).collect();
    let cells: Vec<f32> = widths.iter().map(|w| w + 2.0 * s).collect();
    let line_h = p.fonts(|f| f.row_height(&font));
    paint::celled(p, pos2(case.left() + 4.0 * s, cy - line_h * 0.5), &shown, &cells, font, ink::NIXIE, fade(ink::NIXIE_GLOW, 0.45), s);
    case
}

fn draw_nixie_row(p: &Painter, layer: Layer, r: Rect, s: f32, texts: &[String; 4], hits: &mut Vec<Hit>) {
    let mut x = r.left();
    // A readout's label sets its cell's width (the row flows), so it needs no room of its own.
    let label = |p: &Painter, at: Pos2, text: &str| {
        if layer == Layer::Static {
            title(p, at, s, text, f32::INFINITY)
        } else {
            title_rect(p, at, s, text, f32::INFINITY)
        }
    };
    for (i, (name, key)) in [(tr!(live_title_depth), "depth"), (tr!(live_title_iterations), "iter"), (tr!(live_title_frame_ms), "ms")].into_iter().enumerate() {
        let lt = label(p, pos2(x, r.top()), name.as_str());
        let case = nixies(p, layer, pos2(x + 2.0 * s, r.top() + TITLE_H * s), s, &texts[i]);
        let cell = lt.union(case);
        if layer == Layer::Dynamic {
            hits.push(Hit { rect: cell, key });
        }
        x = cell.right() + 14.0 * s;
    }
    // The clock, right-aligned.
    let widths = tube_widths(&texts[3], s);
    let clock_w = widths.iter().sum::<f32>() + 2.0 * s * (widths.len().saturating_sub(1)) as f32 + 10.0 * s;
    let cx = (r.right() - clock_w - 2.0 * s).max(x);
    let lt = label(p, pos2(cx, r.top()), tr!(live_title_clock).as_str());
    let case = nixies(p, layer, pos2(cx, r.top() + TITLE_H * s), s, &texts[3]);
    if layer == Layer::Dynamic {
        hits.push(Hit { rect: lt.union(case), key: "clock" });
    }
}

/// The top row: walk dial, age meter, tile plane, GPU scope — one height, one title line.
fn draw_row1(p: &Painter, layer: Layer, r: Rect, s: f32, inp: &Inputs, hits: &mut Vec<Hit>, cost: &mut [f64; 11]) {
    let gap = 16.0 * s;
    let w = r.width();
    let radar_w = (175.0 * s).min(w * 0.19);
    let vu_w = (220.0 * s).min(w * 0.24);
    let tiles_w = (215.0 * s).min(w * 0.23);
    let scope_w = (w - radar_w - vu_w - tiles_w - 3.0 * gap).max(60.0 * s);
    let top = r.top() + TITLE_H * s;
    let h = ROW1_H * s;
    let mut x = r.left();
    let mut cell = |wd: f32| {
        let c = Rect::from_min_size(pos2(x, top), vec2(wd, h));
        x += wd + gap;
        c
    };
    let radar = cell(radar_w);
    let vu = cell(vu_w);
    let tiles = cell(tiles_w);
    let scope = cell(scope_w);
    let faces = [(radar, "walk"), (vu, "age"), (tiles, "tiles"), (scope, "scope")];
    if layer == Layer::Static {
        let names = [tr!(live_title_walk), tr!(live_title_age), tr!(live_title_tiles), tr!(live_title_scope)];
        for (i, ((face_rect, _), name)) in faces.iter().zip(names).enumerate() {
            // The scope's title shares its line with the timer lamps (about 90 units at its right).
            let room = face_rect.width() - if i == 3 { 100.0 * s } else { 0.0 };
            title(p, pos2(face_rect.left(), r.top()), s, &name, room);
            paint::bezel(p, *face_rect);
        }
    } else {
        for (face_rect, key) in faces {
            hits.push(Hit { rect: Rect::from_min_max(pos2(face_rect.left(), r.top()), face_rect.right_bottom()), key });
        }
    }
    let mut t = std::time::Instant::now();
    draw_radar(&p.with_clip_rect(radar), layer, radar, s, inp.model);
    lap(&mut t, cost, 2);
    draw_vu(&p.with_clip_rect(vu), layer, vu, s, inp.model);
    lap(&mut t, cost, 3);
    draw_tiles(&p.with_clip_rect(tiles), layer, tiles, s, inp.model);
    lap(&mut t, cost, 4);
    draw_scope(&p.with_clip_rect(scope), layer, scope, s, inp.model);
    lap(&mut t, cost, 5);
    // The timer lamps, right of the scope's title.
    let mut lx = scope.right() - 4.0 * s;
    let ly = r.top() + TITLE_H * s * 0.45;
    let mut trect = Rect::NOTHING;
    for k in (0..3).rev() {
        let label = format!("T{k}");
        let font = face::narrow(10.0 * s);
        let lr = if layer == Layer::Static {
            paint::tracked(p, pos2(lx, ly), Align2::RIGHT_CENTER, &label, font, ink::ENGRAVE_DIM, 0.0, true)
        } else {
            Align2::RIGHT_CENTER.anchor_size(pos2(lx, ly), p.layout_no_wrap(label, font, ink::ENGRAVE_DIM).size())
        };
        let c = pos2(lr.left() - 9.0 * s, ly);
        if layer == Layer::Static {
            paint::lamp_bezel(p, c, 5.0 * s);
        } else {
            paint::lamp_glass(p, c, 5.0 * s, ink::LAMP_GREEN, 0.15 + 0.85 * inp.model.timers[k]);
        }
        trect = trect.union(lr).union(Rect::from_center_size(c, vec2(14.0, 14.0) * s));
        lx = c.x - 12.0 * s;
    }
    if layer == Layer::Dynamic {
        hits.push(Hit { rect: trect, key: "timers" });
    }
}

/// The walk dial: a radar PPI scope.
fn draw_radar(p: &Painter, layer: Layer, r: Rect, s: f32, m: &Model) {
    let c = r.center();
    let rr = r.width().min(r.height()) * 0.5 - 6.0 * s;
    if layer == Layer::Static {
        paint::phosphor_disc(p, c, rr);
        for f in [0.33, 0.58, 0.86] {
            p.circle_stroke(c, rr * f, Stroke::new(1.0_f32, fade(ink::PHOSPHOR, 0.14)));
        }
        return;
    }
    let Some(w) = &m.walk else { return };
    let text_font = face::narrow(11.0 * s);
    if let Some((done, k)) = w.split {
        // A split refresh: a ring a pass, every band, a share of the tiles.
        for (i, pass) in w.passes.iter().rev().take(k as usize).rev().enumerate() {
            let rad = rr * (0.9 - i as f32 * (0.55 / k as f32));
            let col = match pass.ms {
                Some(ms) => target_color(ms, pass.target_ms, 0.85 * w.fade),
                None => fade(ink::PHOSPHOR, 0.45 * w.fade),
            };
            p.circle_stroke(c, rad, Stroke::new((rr * 0.4 / k as f32).max(2.0 * s), col));
        }
        flash(p, c, rr, m.flash);
        let plate = fade(ink::SCREEN_LO, 0.85);
        paint::plated(p, c + vec2(0.0, -2.0 * s), Align2::CENTER_BOTTOM, &format!("{}×{}", w.res.0, w.res.1), text_font.clone(), ink::PHOSPHOR_TEXT, plate, 3.0 * s);
        paint::plated(p, c + vec2(0.0, 1.0 * s), Align2::CENTER_TOP, &tr!(live_walk_split, done = done, passes = k), text_font, ink::PHOSPHOR_TEXT, plate, 3.0 * s);
        return;
    }
    let ask = w.ask.max(512) as f32;
    let span = (ask / 256.0).log2().max(1.0);
    let b0 = 0.05;
    let ang = |i: f32| -PI / 2.0 + TAU * if i <= 256.0 { i / 256.0 * b0 } else { b0 + (1.0 - b0) * (i / 256.0).log2() / span };
    let mut e = 256.0;
    while e < ask {
        let t = ang(e);
        p.line_segment([polar(c, rr * 0.3, t), polar(c, rr * 0.95, t)], Stroke::new(1.0_f32, fade(ink::PHOSPHOR, 0.22)));
        e *= 2.0;
    }
    for pass in &w.passes {
        let (t0, t1) = (ang(pass.lo as f32), ang((pass.hi as f32).min(ask)));
        if t1 <= t0 {
            continue;
        }
        match (pass.ms, pass.untimed) {
            (Some(ms), _) => paint::sector(p, c, rr * 0.62, rr * 0.86, t0, t1, target_color(ms, pass.target_ms, 0.85 * w.fade)),
            (None, true) => {
                let col = fade(ink::PHOSPHOR, 0.7 * w.fade);
                paint::arc(p, c, rr * 0.86, t0, t1, Stroke::new(1.2_f32, col));
                paint::arc(p, c, rr * 0.62, t0, t1, Stroke::new(1.2_f32, col));
            }
            (None, false) => paint::sector(p, c, rr * 0.62, rr * 0.86, t0, t1, fade(ink::PHOSPHOR, 0.3 * w.fade)),
        }
        p.line_segment([polar(c, rr * 0.62, t1), polar(c, rr * 0.86, t1)], Stroke::new(1.0_f32, fade(ink::SCREEN_LO, 0.9)));
    }
    if !w.ended {
        let t = ang((w.cursor as f32).min(ask));
        // The beam's afterglow: a fan of fading slices behind it.
        for k in 0..8 {
            let a1 = t - k as f32 * 0.06;
            paint::sector(p, c, 0.0, rr, a1 - 0.06, a1, fade(ink::PHOSPHOR, 0.22 * (1.0 - k as f32 / 8.0)));
        }
        p.line_segment([c, polar(c, rr, t)], Stroke::new(3.5 * s, fade(ink::PHOSPHOR, 0.35)));
        p.line_segment([c, polar(c, rr, t)], Stroke::new(1.6 * s, Color32::from_rgb(0xb8, 0xff, 0xcc)));
        if w.verifying {
            let pulse = 0.5 + 0.5 * (m.clock_s * 30.0).sin();
            p.circle_filled(polar(c, rr * 0.95, ang(ask)), 4.0 * s, fade(ink::LAMP_WHITE, 0.4 + 0.6 * pulse));
        }
    }
    flash(p, c, rr, m.flash);
    // A still view's walk names itself above the resolution: the same sweep, a slower budget.
    let tag = match w.kind {
        WalkKind::Pin => None,
        WalkKind::Settle => Some(tr!(live_kind_settle)),
        WalkKind::Ssaa => Some(tr!(live_kind_ssaa)),
    };
    // The readouts sit on plates, so the beam sweeping through the middle never swallows them.
    let plate = fade(ink::SCREEN_LO, 0.85);
    if let Some(tag) = tag {
        let at = c + vec2(0.0, -2.0 * s - text_font.size * 1.15);
        paint::plated(p, at, Align2::CENTER_BOTTOM, &tag, text_font.clone(), fade(ink::PHOSPHOR_TEXT, 0.8), plate, 3.0 * s);
    }
    paint::plated(p, c + vec2(0.0, -2.0 * s), Align2::CENTER_BOTTOM, &format!("{}×{}", w.res.0, w.res.1), text_font.clone(), ink::PHOSPHOR_TEXT, plate, 3.0 * s);
    paint::plated(p, c + vec2(0.0, 1.0 * s), Align2::CENTER_TOP, &format!("{} / {}", group(w.cursor.min(w.ask) as u64), group(w.ask as u64)), text_font, ink::PHOSPHOR_TEXT, plate, 3.0 * s);
}

fn flash(p: &Painter, c: Pos2, r: f32, k: f32) {
    if k > 0.02 {
        p.circle_filled(c, r, fade(Color32::from_rgb(0xe6, 0xff, 0xeb), k * 0.55));
    }
}

/// A count with comma groups (the UI standard).
fn group(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// The picture-age meter: a VU meter in octaves, its recent sawtooth under glass.
fn draw_vu(p: &Painter, layer: Layer, r: Rect, s: f32, m: &Model) {
    let (w, h) = (r.width(), r.height());
    let c = pos2(r.center().x, r.top() + h * 1.02);
    let rr = (h * 0.78).min((w / 2.0 - 16.0 * s) / 0.85f32.sin());
    let (a0, a1) = (-PI / 2.0 - 0.85, -PI / 2.0 + 0.85);
    let ang = |v: f32| a0 + (a1 - a0) * (v / 2.0).clamp(0.0, 1.0);
    let strip = Rect::from_min_size(pos2(r.left() + w * 0.18, r.top() + h * 0.6), vec2(w * 0.64, h * 0.18));
    if layer == Layer::Static {
        paint::vgradient(p, r, ink::CREAM, ink::CREAM_LO);
        paint::arc(p, c, rr, ang(1.0), a1, Stroke::new(6.0 * s, ink::METER_RED));
        paint::arc(p, c, rr - 4.0 * s, a0, a1, Stroke::new(1.4 * s, ink::INK));
        for i in 0..=8 {
            let v = i as f32 * 0.25;
            let major = i % 2 == 0;
            let t = ang(v);
            p.line_segment([polar(c, rr - 4.0 * s, t), polar(c, rr - if major { 14.0 } else { 9.0 } * s, t)], Stroke::new(1.4 * s, ink::INK));
            if major {
                let label = match i {
                    0 => "0",
                    2 => "0.5",
                    4 => "1",
                    6 => "1.5",
                    _ => "2",
                };
                p.text(polar(c, rr + 9.0 * s, t), Align2::CENTER_CENTER, label, face::dial(11.0 * s), ink::INK);
            }
        }
        p.circle_filled(polar(c, rr - 20.0 * s, ang(0.5)), 2.6 * s, Color32::from_rgb(0x2f, 0x6b, 0x2f));
        p.text(pos2(r.center().x, r.top() + h * 0.5), Align2::CENTER_CENTER, tr!(live_age_scale), face::dial(11.0 * s), Color32::from_rgb(0x5a, 0x54, 0x44));
        p.rect_filled(strip, 0.0_f32, Color32::from_rgb(0x06, 0x14, 0x0c));
        dashed(p, pos2(strip.left(), strip.center().y), pos2(strip.right(), strip.center().y), Stroke::new(1.0_f32, fade(ink::RED, 0.6)), 2.0 * s);
        return;
    }
    // The sawtooth under glass.
    let n = m.age_hist.len();
    if n > 1 {
        let full = (super::model::HISTORY_FRAMES - 1) as f32;
        let lead = (super::model::HISTORY_FRAMES - n) as f32;
        let pts: Vec<Pos2> = m
            .age_hist
            .iter()
            .enumerate()
            .map(|(i, v)| pos2(strip.left() + (i as f32 + lead) / full * strip.width(), strip.bottom() - (v / 2.0).clamp(0.0, 1.0) * strip.height()))
            .collect();
        p.add(PathShape::line(pts, PathStroke::new(1.2 * s, ink::PHOSPHOR)));
    }
    // The needle.
    let t = ang(m.age);
    p.line_segment([c, polar(c, rr - 2.0 * s, t)], Stroke::new(1.6 * s, Color32::from_rgb(0x14, 0x13, 0x11)));
    p.circle_filled(c, 7.0 * s, Color32::from_rgb(0x14, 0x13, 0x11));
}

fn dashed(p: &Painter, a: Pos2, b: Pos2, st: Stroke, dash: f32) {
    p.extend(Shape::dashed_line(&[a, b], st, dash, dash));
}

/// The tile plane: what this frame's pass covered.
/// The side of a split refresh's tiles, px: a pass covers every tile whose `(x + y) mod k` is its
/// own (the renderer's interleave).
const SPLIT_TILE_PX: u32 = 16;

/// The tiles instrument's face, behind the plane.
const TILES_FACE: Color32 = Color32::from_rgb(0x0d, 0x0b, 0x08);
/// The plane's levels, as shares of amber mixed into the face: the plane itself, a tile still to
/// come, a tile done, and the tiles of this frame's pass (up to 1 while its glow lasts). Opaque
/// mixes, not amber at an alpha: egui premultiplies a faint colour in linear light, so amber at
/// 0.04 alpha showed as a quarter-bright brown and done tiles at 0.3 nearly matched the pass's
/// own at 0.95 — the last pass of a split read as the whole plane lit (2026-10-20).
const LEVEL_PLANE: f32 = 0.07;
const LEVEL_TO_COME: f32 = 0.13;
const LEVEL_DONE: f32 = 0.38;
const LEVEL_PASS: f32 = 0.7;

/// The plane's colour at a level (a share of amber in the face).
fn level(t: f32) -> Color32 {
    paint::mix(TILES_FACE, ink::AMBER, t)
}

/// Where the picture's plane sits in the tiles instrument `r` (its aspect, scaled to fit above the
/// captions), and the scale from picture px to panel px.
fn plane_rect(r: Rect, s: f32, res: (u32, u32)) -> (Rect, f32) {
    let (rw, rh) = (res.0.max(1) as f32, res.1.max(1) as f32);
    let cap = 30.0 * s;
    let sc = ((r.width() - 12.0 * s) / rw).min((r.height() - cap - 8.0 * s) / rh);
    let (pw, ph) = (rw * sc, rh * sc);
    (Rect::from_min_size(pos2(r.center().x - pw / 2.0, r.top() + 6.0 * s), vec2(pw, ph)), sc)
}

/// The resolution a plane is drawn at.
fn plane_res(m: &Model) -> (u32, u32) {
    match &m.plane {
        Plane::Whole { res, .. } | Plane::Split { res, .. } | Plane::Ssaa { res, .. } => *res,
        Plane::Grid(g) => g.res,
        Plane::Idle(_) => m.snap.res,
    }
}

/// A split refresh's 16-px tiles across a picture: (columns, rows).
fn split_cells(res: (u32, u32)) -> (u32, u32) {
    (res.0.max(1).div_ceil(SPLIT_TILE_PX), res.1.max(1).div_ceil(SPLIT_TILE_PX))
}

/// The smallest cell the plane draws, panel px: below it a grid of cells aliases into stripes and
/// plaids, and its gaps eat the cells (a finished 16-px grid read as a field of dots at real size).
const MIN_CELL_PX: f32 = 5.0;
/// Tiles at least this big on the plane keep a hairline gap between them.
const GAP_CELL_PX: f32 = 6.0;

/// The cells a split refresh is drawn with on a `plane`: its real 16-px tiles where they are big
/// enough to see, else the same interleave at the smallest legible cell — a diagram of the sets,
/// which the captions keep honest ("TILE 16 PX", the tiles a frame).
fn split_display(plane: Rect, res: (u32, u32)) -> (u32, u32) {
    let (cols, rows) = split_cells(res);
    if plane.width() / cols as f32 >= MIN_CELL_PX {
        (cols, rows)
    } else {
        ((plane.width() / MIN_CELL_PX).floor().max(1.0) as u32, (plane.height() / MIN_CELL_PX).floor().max(1.0) as u32)
    }
}

/// The tiles instrument's captions: the line under the plane, then the tile size and the tiles a
/// frame (or a sample). Iteration bands and tile counts are numbers in the language's grouping; a
/// resolution, a grid's shape and a tile's side are data, written as they are.
fn tile_labels(plane: &Plane, res: (u32, u32)) -> (String, String, String) {
    let tile_res = |res: (u32, u32)| tr!(live_tiles_tile_res, res = format!("{}×{}", res.0, res.1));
    match plane {
        Plane::Whole { res, band, settled, .. } => (
            match band {
                Some((lo, hi)) if *settled => tr!(live_tiles_settle_band, lo = *lo, hi = *hi),
                Some((lo, hi)) => tr!(live_tiles_pin_band, lo = *lo, hi = *hi),
                None if *settled => tr!(live_tiles_settled_whole),
                None => tr!(live_tiles_moving_whole),
            },
            tile_res(*res),
            tr!(live_tiles_rate_one),
        ),
        Plane::Split { k, j, .. } => {
            let k = (*k).max(1);
            let (cols, rows) = split_cells(res);
            (
                tr!(live_tiles_split_pass, pass = j + 1, passes = k),
                tr!(live_tiles_tile_px, px = SPLIT_TILE_PX.to_string()),
                tr!(live_tiles_rate_count, count = ((cols * rows) as f32 / k as f32).ceil() as u64),
            )
        }
        Plane::Grid(g) => {
            let done = g.done.iter().filter(|d| **d).count();
            // A tile dearer than the admission goes out in strips: this frame sends part of one.
            let part = g.last.filter(|&(_, _, rows, piece)| !g.complete && piece < rows).map(|(_, _, rows, piece)| piece as f32 / rows as f32);
            let grid = format!("{}×{}", g.cols, g.rows);
            (
                if g.complete {
                    tr!(live_tiles_grid_done, grid = &grid)
                } else if part.is_some() {
                    tr!(live_tiles_grid_strips, grid = &grid, done = done, total = g.done.len())
                } else {
                    tr!(live_tiles_grid_progress, grid = &grid, done = done, total = g.done.len())
                },
                tr!(live_tiles_tile_px, px = g.side.to_string()),
                match part {
                    _ if g.complete => tr!(live_tiles_rate_count, count = 0u32),
                    Some(f) => tr!(live_tiles_rate_part, share = crate::i18n::decimal(f, 2)),
                    None => tr!(live_tiles_rate_one),
                },
            )
        }
        Plane::Ssaa { res, samples, band, .. } => (
            match band {
                // The next sample walks: its pass's band (the samples folded so far stay counted).
                Some((lo, hi)) => tr!(live_tiles_ssaa_band, samples = *samples, lo = *lo, hi = *hi),
                None => tr!(live_tiles_ssaa_whole, sample = *samples),
            },
            tile_res(*res),
            tr!(live_tiles_rate_sample),
        ),
        Plane::Idle(why) => (idle_caption(why), tr!(live_tiles_no_pass), tr!(live_tiles_rate_count, count = 0u32)),
    }
}

/// An idle plane's caption: the model's word for what is on screen (an id, `Plane::Idle`), in the
/// language in use.
fn idle_caption(why: &str) -> String {
    match why {
        "STARTING" => tr!(live_tiles_idle_starting),
        "HELD" => tr!(live_tiles_idle_held),
        "REPROJECTED" => tr!(live_tiles_idle_reprojected),
        "LIVE" => tr!(live_tiles_idle_live),
        "IDLE" => tr!(live_tiles_idle_none),
        other => other.to_owned(),
    }
}

fn draw_tiles(p: &Painter, layer: Layer, r: Rect, s: f32, m: &Model) {
    if layer == Layer::Static {
        p.rect_filled(r, 6.0 * s, TILES_FACE);
        return;
    }
    let res = plane_res(m);
    let (plane, sc) = plane_rect(r, s, res);
    let amber = |a: f32| fade(ink::AMBER, a);
    let pass = |glow: f32| level(LEVEL_PASS + (1.0 - LEVEL_PASS) * glow);
    p.rect_filled(plane, 0.0_f32, level(LEVEL_PLANE));
    // A walk's pass covers the whole picture but iterates only the pixels still running: the plane
    // fills to that share (`running_level`), labelled.
    let running = m.snap.running.filter(|_| matches!(&m.plane, Plane::Whole { band: Some(_), .. } | Plane::Ssaa { band: Some(_), .. }));
    match &m.plane {
        Plane::Whole { glow, .. } if running.is_some() => draw_running(p, plane, s, running.unwrap_or(1.0), pass(*glow)),
        Plane::Ssaa { glow, .. } if running.is_some() => {
            draw_running(p, plane, s, running.unwrap_or(1.0), level(LEVEL_DONE + (1.0 - LEVEL_DONE) * 0.6 * glow))
        }
        Plane::Whole { glow, .. } => {
            p.rect_filled(plane, 0.0_f32, pass(*glow));
        }
        Plane::Split { k, j, glow, .. } => {
            let (cols, rows) = split_display(plane, res);
            let (cw, ch) = (plane.width() / cols as f32, plane.height() / rows as f32);
            let gap = if cw >= GAP_CELL_PX { 0.5 } else { 0.0 };
            let mut mesh = Mesh::default();
            let k = (*k).max(1);
            for y in 0..rows {
                for x in 0..cols {
                    let set = (x + y) % k;
                    let c = if set == *j { pass(*glow) } else if set < *j { level(LEVEL_DONE) } else { level(LEVEL_TO_COME) };
                    let c0 = pos2(plane.left() + x as f32 * cw, plane.top() + y as f32 * ch);
                    let cell = Rect::from_min_size(c0, vec2(cw - gap, ch - gap));
                    mesh.add_colored_rect(cell, c);
                }
            }
            p.add(Shape::mesh(mesh));
        }
        Plane::Grid(g) => {
            let cs = g.side as f32 * sc;
            // A tile just finished flashes toward the pass's level and settles to done.
            draw_grid(p, plane, cs, g, |i| {
                let base = if g.done[i] { LEVEL_DONE } else { LEVEL_TO_COME };
                level(base + (1.0 - base) * g.flash[i].clamp(0.0, 1.0))
            });
            if let Some((idx, sent, rows, _)) = g.last.filter(|_| !g.complete) {
                let t = tile_rect(plane, cs, g.cols, idx);
                if sent < rows {
                    p.rect_filled(Rect::from_min_size(t.min, vec2(t.width(), t.height() * sent as f32 / rows as f32)), 0.0_f32, pass(1.0));
                }
                p.rect_stroke(t, 0.0_f32, Stroke::new(1.2 * s, Color32::from_rgb(0xff, 0xe2, 0xb0)), egui::StrokeKind::Inside);
            }
        }
        Plane::Ssaa { glow, .. } => {
            p.rect_filled(plane, 0.0_f32, level(LEVEL_DONE + (1.0 - LEVEL_DONE) * 0.6 * glow));
        }
        Plane::Idle(_) => {}
    }
    let (l1, tl, tr) = tile_labels(&m.plane, res);
    p.rect_stroke(plane.expand(0.5), 0.0_f32, Stroke::new(1.0_f32, amber(0.3)), egui::StrokeKind::Outside);
    let (line, f) = tile_caption_type(p, r.width(), s, &l1, &tl, &tr);
    p.text(pos2(r.center().x, r.bottom() - 17.0 * s), Align2::CENTER_BOTTOM, &l1, line, Color32::from_rgb(0xa8, 0x9f, 0x88));
    let glow = fade(ink::NIXIE_GLOW, 0.35);
    let lit = Color32::from_rgb(0xff, 0xb5, 0x5c);
    paint::tabular(p, pos2(r.left() + 6.6 * s, r.bottom() - 4.0 * s), Align2::LEFT_BOTTOM, &tl, f.clone(), glow);
    paint::tabular(p, pos2(r.right() - 5.4 * s, r.bottom() - 4.0 * s), Align2::RIGHT_BOTTOM, &tr, f.clone(), glow);
    paint::tabular(p, pos2(r.left() + 6.0 * s, r.bottom() - 4.0 * s), Align2::LEFT_BOTTOM, &tl, f.clone(), lit);
    paint::tabular(p, pos2(r.right() - 6.0 * s, r.bottom() - 4.0 * s), Align2::RIGHT_BOTTOM, &tr, f, lit);
}

/// The tiles instrument's caption type in a window `width` wide: the line under the plane, then
/// the pair at its foot (tile size left, tiles a frame right). They were sized for English; a
/// translation can run longer (Italian's "RIQUADRO 1920×1071" and "1 RIQUADRO/FOTOGRAMMA" met
/// mid-window), so each line shrinks to fit (`fit_scale`), the pair keeping a gap between its two.
fn tile_caption_type(p: &Painter, width: f32, s: f32, line: &str, left: &str, right: &str) -> (egui::FontId, egui::FontId) {
    let room = width - 12.0 * s;
    let wide = |texts: &[&str], size: f32| texts.iter().map(|t| paint::tabular_width(p, t, face::narrow(size))).sum::<f32>();
    let k_line = fit_scale(room, |k| wide(&[line], 10.0 * s * k));
    let k_pair = fit_scale(room - 10.0 * s, |k| wide(&[left, right], 12.0 * s * k));
    (face::narrow(10.0 * s * k_line), face::narrow(12.0 * s * k_pair))
}

/// Where a walk's plane fills to for `share` of its texels still running: a log scale, from 1 in
/// 10,000 at the floor to all of them at the top, so the late passes' last few percent still show.
pub(crate) fn running_level(share: f32) -> f32 {
    ((share.max(1.0e-6).log10() + 4.0) / 4.0).clamp(0.0, 1.0)
}

/// A share as the readout prints it: two significant figures down to a hundredth of a percent.
pub(crate) fn running_text(share: f32) -> String {
    let pc = share * 100.0;
    let percent = if pc >= 10.0 {
        format!("{pc:.0}")
    } else if pc >= 1.0 {
        format!("{pc:.1}")
    } else {
        format!("{pc:.2}")
    };
    tr!(live_tiles_running, percent = percent)
}

/// A walk's pass on the plane: the whole picture lit as done, filled from the bottom to the share
/// of its pixels still iterating (`running_level`) in the pass's colour, with a hairline at the
/// level and the share printed in the corner.
fn draw_running(p: &Painter, plane: Rect, s: f32, share: f32, lit: Color32) {
    p.rect_filled(plane, 0.0_f32, level(LEVEL_DONE));
    let top = plane.bottom() - plane.height() * running_level(share);
    if share > 0.0 {
        p.rect_filled(Rect::from_min_max(pos2(plane.left(), top.min(plane.bottom() - 1.0)), plane.max), 0.0_f32, lit);
        p.line_segment([pos2(plane.left(), top), pos2(plane.right(), top)], Stroke::new(1.0_f32, Color32::from_rgb(0xff, 0xe2, 0xb0)));
    }
    paint::tabular(p, plane.right_top() + vec2(-4.0 * s, 3.0 * s), Align2::RIGHT_TOP, &running_text(share), face::narrow(10.0 * s), Color32::from_rgb(0xff, 0xec, 0xc8));
}

fn tile_rect(plane: Rect, cs: f32, cols: u32, i: u32) -> Rect {
    let (x, y) = ((i % cols) as f32 * cs, (i / cols) as f32 * cs);
    let min = plane.min + vec2(x, y);
    let max = pos2((min.x + cs).min(plane.right()), (min.y + cs).min(plane.bottom()));
    Rect::from_min_max(min, max)
}

fn draw_grid(p: &Painter, plane: Rect, cs: f32, g: &super::model::Grid, color: impl Fn(usize) -> Color32) {
    let mut mesh = Mesh::default();
    // Small tiles meet edge to edge: a gap round a 2-px tile leaves almost nothing of it.
    let gap = if cs >= GAP_CELL_PX { 0.75 } else { 0.0 };
    for i in 0..g.done.len() {
        let t = tile_rect(plane, cs, g.cols, i as u32).shrink(gap);
        if t.width() > 0.0 && t.height() > 0.0 {
            mesh.add_colored_rect(t, color(i));
        }
    }
    p.add(Shape::mesh(mesh));
}

/// The GPU scope: a column a frame, passes stacked by kind, against 16.7 / 33 / 50 ms.
fn draw_scope(p: &Painter, layer: Layer, r: Rect, s: f32, m: &Model) {
    let full = 60.0;
    let y = |ms: f32| r.bottom() - 6.0 * s - (ms / full) * (r.height() - 12.0 * s);
    if layer == Layer::Static {
        paint::phosphor_rect(p, r);
        let grid = Stroke::new(1.0_f32, fade(ink::PHOSPHOR, 0.09));
        for i in 1..10 {
            let x = r.left() + r.width() * i as f32 / 10.0;
            p.line_segment([pos2(x, r.top()), pos2(x, r.bottom())], grid);
        }
        for i in 1..6 {
            let yy = r.top() + r.height() * i as f32 / 6.0;
            p.line_segment([pos2(r.left(), yy), pos2(r.right(), yy)], grid);
        }
        for (ms, col, label) in [(16.7, fade(ink::PHOSPHOR, 0.45), "16.7"), (33.3, fade(ink::AMBER, 0.55), "33"), (50.0, fade(ink::RED, 0.7), "50 ms")] {
            dashed(p, pos2(r.left(), y(ms)), pos2(r.right(), y(ms)), Stroke::new(1.0_f32, col), 3.0 * s);
            p.text(pos2(r.right() - 4.0 * s, y(ms) - 2.0 * s), Align2::RIGHT_BOTTOM, label, face::narrow(10.0 * s), col);
        }
        let f = face::narrow(10.0 * s);
        let mut x = r.left() + 6.0 * s;
        let legend = [
            (tr!(live_kind_moving), fade(ink::PHOSPHOR_TEXT, 0.8)),
            (tr!(live_kind_pin), Color32::from_rgb(0xff, 0xc8, 0x78)),
            (tr!(live_kind_worker), ink::LAMP_BLUE),
            (tr!(live_kind_settle), Color32::from_rgb(0xff, 0xec, 0xc8)),
            (tr!(live_kind_ssaa), ink::SSAA),
        ];
        for (label, col) in legend {
            let lr = p.text(pos2(x, r.top() + 4.0 * s), Align2::LEFT_TOP, label, f.clone(), col);
            x = lr.right() + 8.0 * s;
        }
        return;
    }
    let col_w = 3.0 * s;
    let n = (((r.width() - 8.0 * s) / col_w).floor() as usize).max(1);
    let skip = m.columns.len().saturating_sub(n);
    let shown: Vec<_> = m.columns.iter().skip(skip).collect();
    let now = m.snap.frame;
    let mut mesh = Mesh::default();
    let mut hollow = Vec::new();
    for (i, c) in shown.iter().enumerate() {
        let x = r.left() + 4.0 * s + i as f32 * col_w;
        let age = (shown.len() - 1 - i) as f32 / n as f32;
        let a = 0.25 + 0.75 * (1.0 - age);
        let mut base = r.bottom() - 6.0 * s;
        let mut bar = |ms: f32, color: Color32| {
            if ms <= 0.0 {
                return;
            }
            let hh = (ms / full) * (r.height() - 12.0 * s);
            mesh.add_colored_rect(Rect::from_min_max(pos2(x, base - hh), pos2(x + col_w - 1.0, base)), color);
            base -= hh;
        };
        bar(c.ms[Pass::Moving as usize], Color32::from_rgba_unmultiplied(90, 200, 120, (a * 0.6 * 255.0) as u8));
        bar(c.ms[Pass::Probe as usize], Color32::from_rgba_unmultiplied(90, 200, 120, (a * 0.45 * 255.0) as u8));
        bar(c.ms[Pass::Pin as usize] + c.ms[Pass::Split as usize], price_color(c.ms[Pass::Pin as usize] + c.ms[Pass::Split as usize], a));
        bar(c.ms[Pass::Tile as usize] + c.ms[Pass::Walk as usize], Color32::from_rgba_unmultiplied(255, 236, 200, (a * 0.8 * 255.0) as u8));
        bar(c.ms[Pass::Ssaa as usize], fade(ink::SSAA, a * 0.7));
        bar(c.ms[Pass::Worker as usize], Color32::from_rgba_unmultiplied(154, 216, 255, (a * 0.75 * 255.0) as u8));
        // A pass that ran with no timing back by now: a hollow mark at the baseline.
        let ran_untimed = (0..PASSES).any(|k| c.ran[k] && c.ms[k] <= 0.0);
        if ran_untimed && now.saturating_sub(c.frame) > UNTIMED_AFTER_FRAMES {
            hollow.push(Rect::from_min_size(pos2(x, r.bottom() - 10.0 * s), vec2(col_w - 1.0, 4.0 * s)));
        }
    }
    p.add(Shape::mesh(mesh));
    for h in hollow {
        p.rect_stroke(h, 0.0_f32, Stroke::new(1.0_f32, fade(ink::PHOSPHOR, 0.6)), egui::StrokeKind::Inside);
    }
}

/// The depth dial: a radio tuning dial in log2 magnification, a window 200 octaves wide that
/// follows the view; arithmetic bands, the builders as dots, the view as the red pointer.
fn draw_dial(p: &Painter, layer: Layer, r: Rect, s: f32, inp: &Inputs) {
    let m = inp.model;
    let lo = dial_lo(m.snap.depth);
    let hi = lo + 200.0;
    let (x0, x1) = (r.left() + 14.0 * s, r.right() - 14.0 * s);
    let x = |d: f32| x0 + ((d - lo) / (hi - lo)).clamp(0.0, 1.0) * (x1 - x0);
    let h = r.height();
    if layer == Layer::Static {
        paint::bezel(p, r);
        let mid = r.center().y;
        paint::vgradient(p, Rect::from_min_max(r.left_top(), pos2(r.right(), mid)), Color32::from_rgb(0x2a, 0x1a, 0x07), Color32::from_rgb(0x5a, 0x3a, 0x10));
        paint::vgradient(p, Rect::from_min_max(pos2(r.left(), mid), r.right_bottom()), Color32::from_rgb(0x5a, 0x3a, 0x10), Color32::from_rgb(0x2a, 0x1a, 0x07));
        paint::radial_disc(p, r.center(), r.width() * 0.45, fade(Color32::from_rgb(0xff, 0xbe, 0x5a), 0.16), fade(Color32::from_rgb(0xff, 0xbe, 0x5a), 0.0));
        // DF32 and FLOATEXP are the arithmetic's names; "direct" is a word.
        for (a, b, col, label) in [
            (0.0, inp.df32_from, Color32::from_rgba_unmultiplied(120, 200, 220, 115), tr!(live_dial_direct)),
            (inp.df32_from, inp.fe_from, Color32::from_rgba_unmultiplied(255, 190, 90, 128), "DF32".to_owned()),
            (inp.fe_from, f32::MAX, Color32::from_rgba_unmultiplied(255, 110, 60, 140), "FLOATEXP".to_owned()),
        ] {
            let (a, b) = (a.max(lo), b.min(hi));
            if b <= a {
                continue;
            }
            p.rect_filled(Rect::from_min_max(pos2(x(a), r.top() + h - 9.0 * s), pos2(x(b) - 2.0 * s, r.top() + h - 5.0 * s)), 0.0_f32, col);
            p.text(pos2(x(a) + 2.0 * s, r.top() + h - 11.0 * s), Align2::LEFT_BOTTOM, label, face::dial(10.0 * s), Color32::from_rgba_unmultiplied(255, 225, 170, 166));
        }
        let tick = Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 225, 170, 190));
        let mut d = lo;
        while d <= hi + 0.5 {
            let major = (d as i64) % 50 == 0;
            let xx = x(d);
            p.line_segment([pos2(xx, r.top() + h * 0.42), pos2(xx, r.top() + h * 0.42 + if major { 9.0 } else { 4.0 } * s)], tick);
            if major {
                let label = if d == 0.0 { "1×".to_owned() } else { format!("2^{}", d as i64) };
                p.text(pos2(xx, r.top() + h * 0.36), Align2::CENTER_BOTTOM, &label, face::dial(11.0 * s), Color32::from_rgba_unmultiplied(255, 225, 170, 217));
            }
            d += 10.0;
        }
        if inp.fe_from > lo && inp.fe_from < hi {
            dashed(p, pos2(x(inp.fe_from), r.top() + h * 0.42), pos2(x(inp.fe_from), r.bottom() - 6.0 * s), Stroke::new(1.0_f32, fade(ink::RED, 0.8)), 2.0 * s);
        }
        // The glass's sheen.
        paint::vgradient(p, Rect::from_min_max(r.left_top(), pos2(r.right(), r.top() + h * 0.5)), Color32::from_white_alpha(28), Color32::from_white_alpha(0));
        return;
    }
    // The builders.
    let yy = r.top() + h * 0.66;
    for puck in &m.pucks {
        let px = x(puck.depth);
        let col = match puck.role {
            RefRole::Live => ink::LAMP_GREEN,
            RefRole::Ahead => ink::AMBER,
            RefRole::Hold => ink::LAMP_BLUE,
        };
        match puck.state {
            PuckState::Building => {
                let pulse = 0.5 + 0.5 * (m.clock_s * 40.0).sin();
                p.circle_filled(pos2(px, yy), 3.4 * s, fade(col, 0.5 + 0.5 * pulse));
            }
            PuckState::Built => {
                p.circle_filled(pos2(px, yy), 3.4 * s, col);
            }
            PuckState::Installed => {
                let k = (puck.left_s / super::model::PUCK_AFTERGLOW_S).clamp(0.0, 1.0);
                p.circle_stroke(pos2(px, yy), (3.0 + (1.0 - k) * 13.0) * s, Stroke::new(1.4 * s, fade(col, k)));
            }
            PuckState::Dropped => {
                let k = (puck.left_s / super::model::PUCK_AFTERGLOW_S).clamp(0.0, 1.0);
                let st = Stroke::new(1.6 * s, fade(ink::RED, k));
                let d = 4.0 * s;
                p.line_segment([pos2(px - d, yy - d), pos2(px + d, yy + d)], st);
                p.line_segment([pos2(px + d, yy - d), pos2(px - d, yy + d)], st);
            }
        }
    }
    // The pointer, with its glow.
    let px = x(m.snap.depth.max(0.0) as f32);
    for (w, a) in [(7.0, 0.12), (4.0, 0.25), (2.0, 1.0)] {
        p.line_segment([pos2(px, r.top() + 3.0 * s), pos2(px, r.bottom() - 3.0 * s)], Stroke::new(w * s, fade(Color32::from_rgb(0xff, 0x3b, 0x1f), a)));
    }
}

/// The lamp labels' type at scale 1: Routed Gothic Narrow, its size and tracking.
const LAMP_LABEL_PT: f32 = 10.5;
const LAMP_LABEL_TRACK: f32 = 0.8;

/// A lamp label's lines and the scale its type is drawn at. Measured at scale 1 in the label's own
/// type, so a label breaks the same way at every panel size: one line when it fits the cell with
/// room either side, else two at the space that keeps the longer line shortest; a line still wider
/// than the cell is drawn smaller to fit it. The English labels keep the look they had when their
/// width was reckoned from their letter count: only REF AHEAD breaks (52.3 units at least, at 1 to 4
/// pixels a point, against 50 at most for REF HOLD, the widest kept whole), and none shrinks.
fn lamp_label_lines(p: &Painter, label: &str) -> (Vec<String>, f32) {
    let width = |t: &str, k: f32| tracked_size(p, t, face::narrow(LAMP_LABEL_PT * k), LAMP_LABEL_TRACK * k).x;
    let widest = |lines: &Vec<String>, k: f32| lines.iter().map(|t| width(t.as_str(), k)).fold(0.0, f32::max);
    let words: Vec<&str> = label.split(' ').collect();
    let lines = if words.len() == 1 || width(label, 1.0) <= LAMP_CELL_W - 7.0 {
        vec![label.to_owned()]
    } else {
        (1..words.len())
            .map(|i| vec![words[..i].join(" "), words[i..].join(" ")])
            .min_by(|a, b| widest(a, 1.0).total_cmp(&widest(b, 1.0)))
            .unwrap_or_else(|| vec![label.to_owned()])
    };
    let k = fit_scale(LAMP_CELL_W - 2.0, |k| widest(&lines, k));
    (lines, k)
}

/// The light bank: lamps under engraved labels, wrapping, with separators between groups.
fn draw_bank(p: &Painter, layer: Layer, r: Rect, s: f32, m: &Model, hits: &mut Vec<Hit>) {
    let mut x = r.left();
    let mut y = r.top();
    let cell = LAMP_CELL_W * s;
    for spec in LAMP_SPECS.iter() {
        if spec.group && x > r.left() && x + LAMP_SEP_W * s + cell <= r.right() + 0.5 {
            if layer == Layer::Static {
                let sx = x + LAMP_SEP_W * s * 0.5;
                p.line_segment([pos2(sx, y + 4.0 * s), pos2(sx, y + LAMP_ROW_H * s - 10.0 * s)], Stroke::new(1.0_f32, Color32::from_rgb(0x4a, 0x4f, 0x55)));
            }
            x += LAMP_SEP_W * s;
        }
        if x + cell > r.right() + 0.5 && x > r.left() {
            x = r.left();
            y += LAMP_ROW_H * s;
        }
        let c = pos2(x + cell * 0.5, y + 12.0 * s);
        if layer == Layer::Static {
            paint::lamp_bezel(p, c, 7.5 * s);
            let (lines, k) = lamp_label_lines(p, &lamp_label(spec.lamp));
            let font = face::narrow(LAMP_LABEL_PT * s * k);
            for (i, line) in lines.iter().enumerate() {
                let at = pos2(c.x, y + (24.0 + 11.0 * i as f32) * s);
                paint::tracked(p, at, Align2::CENTER_TOP, line, font.clone(), ink::ENGRAVE_DIM, LAMP_LABEL_TRACK * s * k, true);
            }
        } else {
            paint::lamp_glass(p, c, 7.5 * s, spec.hue, m.lamp(spec.lamp));
            hits.push(Hit { rect: Rect::from_min_size(pos2(x, y), vec2(cell, LAMP_ROW_H * s)), key: spec.key });
        }
        x += cell;
    }
    if layer == Layer::Dynamic {
        hits.push(Hit { rect: r, key: "bank" });
    }
}

fn draw_engine_room(p: &Painter, layer: Layer, r: Rect, s: f32, inp: &Inputs, hits: &mut Vec<Hit>, cost: &mut [f64; 11]) {
    let gap = 14.0 * s;
    let n = 5.0;
    let w = (r.width() - gap * (n - 1.0)) / n;
    let top = r.top() + TITLE_H * s;
    let h = ENGINE_H * s;
    let keys = ["tach", "engines", "memory", "cores", "pixels"];
    for (i, key) in keys.iter().enumerate() {
        let x = r.left() + i as f32 * (w + gap);
        let face_rect = Rect::from_min_size(pos2(x, top), vec2(w, h));
        if layer == Layer::Static {
            let name = match i {
                0 => tr!(live_title_steps),
                1 => tr!(live_title_engines),
                2 => tr!(live_title_memory),
                3 => tr!(live_title_cores),
                _ => tr!(live_title_pixels),
            };
            title(p, pos2(x, r.top()), s, &name, w);
            paint::bezel(p, face_rect);
        } else {
            hits.push(Hit { rect: Rect::from_min_max(pos2(x, r.top()), face_rect.right_bottom()), key });
        }
        let cp = p.with_clip_rect(face_rect);
        let mut t = std::time::Instant::now();
        match i {
            0 => draw_tach(&cp, layer, face_rect, s, inp.engine),
            1 => draw_engines(&cp, layer, face_rect, s, inp),
            2 => draw_tanks(&cp, layer, face_rect, s, inp.engine),
            3 => draw_cores(&cp, layer, face_rect, s, inp),
            _ => draw_pixels(&cp, layer, face_rect, s, inp),
        }
        lap(&mut t, cost, if i == 0 { 8 } else { 9 });
    }
}

/// Steps per second on a log dial (1e9..1e13): white asked, amber executed; the odometer below.
fn draw_tach(p: &Painter, layer: Layer, r: Rect, s: f32, e: &Engine) {
    let c = r.center() + vec2(0.0, -6.0 * s);
    let rr = r.width().min(r.height() - 22.0 * s) * 0.5 - 10.0 * s;
    let (a0, a1) = (PI * 0.75, PI * 2.25);
    let ang = |v: f64| a0 + (a1 - a0) * (((v.max(1.0).log10() - 9.0) / 4.0).clamp(0.0, 1.0)) as f32;
    let dw = 10.0 * s;
    let ox = r.center().x - dw * 6.0;
    let oy = r.bottom() - 18.0 * s;
    let wheel = |i: usize| Rect::from_min_size(pos2(ox + i as f32 * dw, oy), vec2(dw - 1.0, 15.0 * s));
    if layer == Layer::Static {
        p.rect_filled(r, 6.0 * s, ink::FACE_DARK);
        paint::arc(p, c, rr, ang(10f64.powf(12.4)), a1, Stroke::new(5.0 * s, ink::METER_RED));
        let tick = Stroke::new(1.2 * s, Color32::from_rgb(0xd8, 0xcf, 0xb6));
        for k in 0..=8 {
            let e10 = 9.0 + k as f64 * 0.5;
            let t = ang(10f64.powf(e10));
            let major = k % 2 == 0;
            p.line_segment([polar(c, rr, t), polar(c, rr - if major { 9.0 } else { 5.0 } * s, t)], tick);
            if major {
                let label = ["10⁹", "10¹⁰", "10¹¹", "10¹²", "10¹³"][k / 2];
                p.text(polar(c, rr - 19.0 * s, t), Align2::CENTER_CENTER, label, face::dial(11.0 * s), Color32::from_rgb(0xd8, 0xcf, 0xb6));
            }
        }
        // The odometer's wheels.
        for i in 0..12 {
            let cell = wheel(i);
            paint::vgradient(p, Rect::from_min_max(cell.min, pos2(cell.right(), cell.center().y)), Color32::from_rgb(0x0d, 0x0d, 0x0d), Color32::from_rgb(0x2a, 0x2a, 0x2a));
            paint::vgradient(p, Rect::from_min_max(pos2(cell.left(), cell.center().y), cell.max), Color32::from_rgb(0x2a, 0x2a, 0x2a), Color32::from_rgb(0x0d, 0x0d, 0x0d));
        }
        return;
    }
    let needle = |v: f64, col: Color32, len: f32| {
        p.line_segment([c, polar(c, len, ang(v))], Stroke::new(2.0 * s, col));
    };
    if e.steps_done > 0.0 {
        needle(e.steps_done, ink::AMBER, rr - 6.0 * s);
    }
    needle(e.steps_asked, Color32::from_rgb(0xf5, 0xef, 0xe0), rr - 2.0 * s);
    p.circle_filled(c, 6.0 * s, Color32::from_rgb(0x2b, 0x2d, 0x30));
    let mult = if e.steps_done > 0.0 { e.steps_asked / e.steps_done } else { 0.0 };
    let label = if mult >= 1.0 { format!("BLA ×{}", group(mult.round() as u64)) } else { "BLA —".to_owned() };
    paint::tabular(p, c + vec2(0.0, rr * 0.55), Align2::CENTER_CENTER, &label, face::dial(11.0 * s), ink::AMBER);
    // The odometer: the session's steps, twelve wheels, one layout.
    let total = e.total_steps.max(0.0).min(9.99e14) as u128;
    let digits = format!("{:012}", total % 1_000_000_000_000);
    let font = face::dial(13.0 * s);
    let line_h = p.fonts(|f| f.row_height(&font));
    let first = wheel(0);
    let cells = vec![dw; 12];
    paint::celled(p, pos2(first.left(), first.center().y - line_h * 0.5), &digits, &cells, font, Color32::from_rgb(0xf3, 0xea, 0xd2), Color32::TRANSPARENT, s);
}

/// Two GPUs' busy gauges, and frames flowing from each to the screen.
fn draw_engines(p: &Painter, layer: Layer, r: Rect, s: f32, inp: &Inputs) {
    let e = inp.engine;
    let m = inp.model;
    let (w, h) = (r.width(), r.height());
    let gr = (w * 0.18).min(h * 0.22);
    let centre = |cx: f32| pos2(r.left() + cx, r.top() + h * 0.36);
    let screen = pos2(r.center().x, r.top() + h * 0.8);
    let path = |who: u8, t: f32| -> Pos2 {
        let ax = r.left() + if who == 1 { w * 0.75 } else { w * 0.25 };
        let ay = r.top() + h * 0.5;
        pos2(ax + (screen.x - ax) * t, ay + (screen.y - ay) * (t * PI / 2.0).sin())
    };
    let gauges = [(w * 0.25, e.gpu_busy[0], ink::AMBER, true), (w * 0.75, e.gpu_busy[1], ink::LAMP_BLUE, e.worker)];
    if layer == Layer::Static {
        p.rect_filled(r, 6.0 * s, ink::FACE_DARK);
        let labels = [tr!(live_engines_gpu_window), if e.worker { tr!(live_engines_gpu_worker) } else { tr!(live_engines_no_worker) }];
        for ((cx, _, _, _), label) in gauges.into_iter().zip(labels) {
            let c = centre(cx);
            paint::arc(p, c, gr, PI, TAU, Stroke::new(6.0 * s, Color32::from_rgb(0x2c, 0x2f, 0x33)));
            p.text(c + vec2(0.0, 6.0 * s), Align2::CENTER_TOP, label, face::dial(10.0 * s), Color32::from_rgb(0xd8, 0xcf, 0xb6));
        }
        for who in [0u8, 1] {
            let pts: Vec<Pos2> = (0..=10).map(|i| path(who, i as f32 / 10.0)).collect();
            p.add(PathShape::line(pts, PathStroke::new(7.0 * s, Color32::from_rgba_unmultiplied(210, 201, 178, 70))));
        }
        let glyph = Rect::from_center_size(screen + vec2(0.0, 2.0 * s), vec2(28.0, 16.0) * s);
        p.rect_filled(glyph, 0.0_f32, Color32::from_rgb(0x0a, 0x0b, 0x0c));
        p.rect_stroke(glyph, 0.0_f32, Stroke::new(1.2_f32, Color32::from_rgb(0x6a, 0x6f, 0x75)), egui::StrokeKind::Inside);
        return;
    }
    for (cx, v, col, live) in gauges {
        let c = centre(cx);
        if live {
            paint::arc(p, c, gr, PI, PI + PI * v.clamp(0.0, 1.0), Stroke::new(6.0 * s, col));
        }
        let pct = if live { format!("{}%", (v.clamp(0.0, 1.0) * 100.0).round() as u32) } else { "—".to_owned() };
        paint::tabular(p, c + vec2(0.0, -4.0 * s), Align2::CENTER_BOTTOM, &pct, face::dial(13.0 * s), Color32::from_rgb(0xd8, 0xcf, 0xb6));
    }
    // Recent adoptions travel down their tube over 0.7 s of frames.
    let now = m.snap.frame;
    for &(f, who) in m.shown.iter() {
        let t = (now.saturating_sub(f) as f32 / 42.0).clamp(0.0, 1.0);
        if t < 1.0 {
            p.circle_filled(path(who, t), 2.4 * s, if who == 1 { Color32::from_rgb(0xbf, 0xe7, 0xff) } else { Color32::from_rgb(0xff, 0xc2, 0x7a) });
        }
    }
    let (a, b) = m.shown_share();
    let label = if m.shown.is_empty() {
        tr!(live_engines_on_screen_none)
    } else {
        tr!(live_engines_on_screen, window = (a * 100.0).round() as u32, worker = (b * 100.0).round() as u32)
    };
    paint::tabular(p, pos2(r.center().x, r.bottom() - 4.0 * s), Align2::CENTER_BOTTOM, &label, face::dial(11.0 * s), Color32::from_rgb(0xd8, 0xcf, 0xb6));
}

/// RAM and GPU memory as two tanks.
fn draw_tanks(p: &Painter, layer: Layer, r: Rect, s: f32, e: &Engine) {
    let w = r.width();
    let (top, bot, tw) = (r.top() + 10.0 * s, r.bottom() - 22.0 * s, w * 0.2);
    let tanks = [r.left() + w * 0.12, r.left() + w * 0.52];
    if layer == Layer::Static {
        p.rect_filled(r, 6.0 * s, ink::FACE_DARK);
        for left in tanks {
            p.rect_filled(Rect::from_min_max(pos2(left, top), pos2(left + tw, bot)), 0.0_f32, Color32::from_rgb(6, 7, 7));
        }
        let f = face::dial(9.0 * s);
        let legend = [
            (tr!(live_memory_gbuf), ink::PHOSPHOR),
            (tr!(live_memory_hold), Color32::from_rgb(0x4f, 0xbf, 0x74)),
            (tr!(live_memory_orbit), ink::AMBER),
            (tr!(live_memory_worker), ink::LAMP_BLUE),
        ];
        for (i, (label, col)) in legend.into_iter().enumerate() {
            p.text(pos2(r.left() + w * 0.76, r.top() + (10.0 + i as f32 * 12.0) * s), Align2::LEFT_TOP, label, f.clone(), col);
        }
        return;
    }
    let tank = |left: f32, label: String, segs: &[(f32, Color32)], cap: f32, peak: Option<f32>| {
        let mut base = bot;
        let cap = cap.max(1.0);
        for &(mb, col) in segs {
            let hh = ((bot - top) * mb / cap).min(base - top);
            if hh > 0.0 {
                p.rect_filled(Rect::from_min_max(pos2(left + 2.0, base - hh), pos2(left + tw - 2.0, base)), 0.0_f32, col);
                base -= hh;
            }
        }
        if let Some(pk) = peak {
            let py = bot - ((bot - top) * pk / cap).min(bot - top);
            p.line_segment([pos2(left - 3.0, py), pos2(left + tw + 3.0, py)], Stroke::new(1.4_f32, ink::RED));
        }
        p.rect_stroke(Rect::from_min_max(pos2(left, top), pos2(left + tw, bot)), 0.0_f32, Stroke::new(1.0_f32, Color32::from_rgb(0x4a, 0x4f, 0x55)), egui::StrokeKind::Inside);
        paint::tabular(p, pos2(left + tw / 2.0, r.bottom() - 6.0 * s), Align2::CENTER_BOTTOM, &label, face::dial(10.0 * s), Color32::from_rgb(0xd8, 0xcf, 0xb6));
    };
    let ram_cap = if e.ram_cap_mb > 0.0 { e.ram_cap_mb.min(e.ram_peak_mb.max(e.ram_mb) * 4.0).max(1024.0) } else { (e.ram_peak_mb * 2.0).max(1024.0) };
    tank(tanks[0], format!("RAM {} MB", group(e.ram_mb.round() as u64)), &[(e.ram_mb, ink::AMBER)], ram_cap, Some(e.ram_peak_mb));
    let kinds = [(e.vram_mb[0], ink::PHOSPHOR), (e.vram_mb[1], Color32::from_rgb(0x4f, 0xbf, 0x74)), (e.vram_mb[2], ink::AMBER), (e.vram_mb[3], ink::LAMP_BLUE)];
    let total: f32 = e.vram_mb.iter().sum();
    let vcap = if e.vram_cap_mb > 0.0 { e.vram_cap_mb } else { (total * 2.0).max(512.0) };
    tank(tanks[1], format!("VRAM {} MB", group(total.round() as u64)), &kinds, vcap, None);
}

/// CPU use per core as an LED bar graph, and busy threads orbiting their roles.
fn draw_cores(p: &Painter, layer: Layer, r: Rect, s: f32, inp: &Inputs) {
    let e = inp.engine;
    let m = inp.model;
    let n = e.cores.len().max(1);
    let segs = 8;
    let (top, bot) = (r.top() + 8.0 * s, r.bottom() - 18.0 * s);
    let bw = (r.width() * 0.5) / n as f32;
    let seg_h = (bot - top) / segs as f32;
    let led = |i: usize, k: usize| Rect::from_min_size(pos2(r.left() + 6.0 * s + i as f32 * bw, bot - (k + 1) as f32 * seg_h + 1.0), vec2((bw - 2.0).max(1.0), (seg_h - 2.0).max(1.0)));
    let hue = |k: usize| if k >= 6 { ink::RED } else if k >= 4 { ink::AMBER } else { ink::LAMP_GREEN };
    let c = pos2(r.left() + r.width() * 0.78, r.top() + r.height() * 0.45);
    let rr = (r.width() * 0.2).min(r.height() * 0.38);
    // The thread roles, in `registry::Role` order (their labels are in the static layer below).
    let roles = super::registry::ROLES;
    let centre = |k: usize| polar(c, rr * 0.5, -PI / 2.0 + k as f32 * TAU / roles as f32);
    if layer == Layer::Static {
        p.rect_filled(r, 6.0 * s, ink::FACE_DARK);
        // Every LED, dark: the dynamic layer lights the ones in use over them.
        let mut mesh = Mesh::default();
        for i in 0..e.cores.len() {
            for k in 0..segs {
                mesh.add_colored_rect(led(i, k), fade(hue(k), 0.1));
            }
        }
        p.add(Shape::mesh(mesh));
        p.text(pos2(r.left() + 6.0 * s, r.bottom() - 5.0 * s), Align2::LEFT_BOTTOM, tr!(live_cores_count, count = e.cores.len()), face::narrow(11.0 * s), Color32::from_rgb(0xd8, 0xcf, 0xb6));
        paint::phosphor_disc(p, c, rr);
        let labels = [tr!(live_cores_role_ref), tr!(live_cores_role_pool), tr!(live_cores_role_worker), tr!(live_cores_role_probe), tr!(live_cores_role_export)];
        for (k, label) in labels.into_iter().enumerate() {
            p.text(centre(k), Align2::CENTER_CENTER, label, face::narrow(8.0 * s), fade(ink::PHOSPHOR, 0.35));
        }
        return;
    }
    let mut mesh = Mesh::default();
    for (i, u) in e.cores.iter().enumerate() {
        let lvl = u.clamp(0.0, 1.0) * segs as f32;
        for k in 0..segs {
            if (k as f32) < lvl {
                mesh.add_colored_rect(led(i, k), fade(hue(k), 0.95));
            }
        }
    }
    p.add(Shape::mesh(mesh));
    let mut total = 0;
    for (k, &count) in e.threads.iter().enumerate() {
        for i in 0..count.min(32) {
            let a = m.clock_s * (2.0 + k as f32 * 0.4) + i as f32 * 0.9;
            let rad = rr * (0.16 + 0.03 * (i % 4) as f32);
            p.circle_filled(polar(centre(k), rad, a), 1.8 * s, if k == 1 { Color32::from_rgb(0xff, 0xd2, 0x7a) } else { Color32::from_rgb(0xb8, 0xff, 0xcc) });
        }
        total += count;
    }
    paint::tabular(p, pos2(c.x, r.bottom() - 5.0 * s), Align2::CENTER_BOTTOM, &tr!(live_cores_threads, count = total + 1), face::narrow(11.0 * s), Color32::from_rgb(0xd8, 0xcf, 0xb6));
}

/// Pixels falling into the picture: dim while a walk computes them, bright when they finish.
fn draw_pixels(p: &Painter, layer: Layer, r: Rect, s: f32, inp: &Inputs) {
    let cols = STREAM_COLS as f32;
    let rows = STREAM_ROWS;
    let (cw, ch) = ((r.width() - 8.0 * s) / cols, (r.height() - 22.0 * s) / rows);
    let cell = |x: f32, y: f32| Rect::from_min_size(pos2(r.left() + 4.0 * s + x * cw + 0.5, r.top() + 4.0 * s + y * ch + 0.5), vec2(cw - 1.5, ch - 1.5));
    if layer == Layer::Static {
        p.rect_filled(r, 6.0 * s, Color32::from_rgb(0x0b, 0x0c, 0x0d));
        let mut mesh = Mesh::default();
        for y in 0..rows as u32 {
            for x in 0..STREAM_COLS as u32 {
                mesh.add_colored_rect(cell(x as f32, y as f32), fade(ink::AMBER, 0.06));
            }
        }
        p.add(Shape::mesh(mesh));
        return;
    }
    let mut mesh = Mesh::default();
    for pt in &inp.model.particles {
        if pt.row < 0.0 {
            continue;
        }
        mesh.add_colored_rect(
            cell(pt.col as f32, pt.row.floor()),
            if pt.done { Color32::from_rgba_unmultiplied(255, 214, 150, 242) } else { Color32::from_rgba_unmultiplied(255, 140, 40, 140) },
        );
    }
    p.add(Shape::mesh(mesh));
    paint::tabular(p, pos2(r.center().x, r.bottom() - 5.0 * s), Align2::CENTER_BOTTOM, &format!("{:.1} Mpx/s", inp.engine.px_per_s / 1.0e6), face::narrow(11.0 * s), Color32::from_rgb(0xd8, 0xcf, 0xb6));
}

#[cfg(test)]
#[path = "../../tests/unit/blinken/panel/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/unit/blinken/panel/tile_plane_tests.rs"]
mod tile_plane_tests;
