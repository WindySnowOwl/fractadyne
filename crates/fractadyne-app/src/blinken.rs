//! Blinkenlights: a retro front panel over the fractal view that shows the live renderer at work
//! (design/blinkenlights.md). Every lamp, needle and trace is driven by a real event or reading:
//!
//! - [`events`]: the event ring every emitter pushes to, whether or not the panel is open;
//! - [`registry`]: how many threads are busy in each role;
//! - [`model`]: the events and a per-frame snapshot turned into instrument state (pure, tested);
//! - [`panel`]: the instruments drawn; [`paint`]: the period parts they are made of;
//! - [`help`]: what each instrument means, and what healthy and troubled look like.
//!
//! [`Overlay`] is the panel as the app holds it: its size (`B` cycles it), the corner it snaps to,
//! its opacity, the model, and the engine room's sampling.

pub(crate) mod events;
pub(crate) mod help;
pub(crate) mod model;
pub(crate) mod paint;
pub(crate) mod panel;
pub(crate) mod registry;

use egui::{Align2, Color32, Id, Pos2, Rect, Sense, vec2};
use model::{Engine, Model, Snapshot};
pub(crate) use panel::Size;
use std::collections::VecDeque;

/// The view corner the panel snaps to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum Corner {
    #[default]
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Corner::TopLeft => "tl",
            Corner::TopRight => "tr",
            Corner::BottomLeft => "bl",
            Corner::BottomRight => "br",
        }
    }
    pub(crate) fn from_name(s: &str) -> Corner {
        match s {
            "tr" => Corner::TopRight,
            "bl" => Corner::BottomLeft,
            "br" => Corner::BottomRight,
            _ => Corner::TopLeft,
        }
    }
    /// The top-left of a panel of `size` snapped to this corner of `view`, `margin` in from it.
    pub(crate) fn place(self, view: Rect, size: egui::Vec2, margin: f32) -> Pos2 {
        let x = match self {
            Corner::TopLeft | Corner::BottomLeft => view.left() + margin,
            Corner::TopRight | Corner::BottomRight => view.right() - margin - size.x,
        };
        let y = match self {
            Corner::TopLeft | Corner::TopRight => view.top() + margin,
            Corner::BottomLeft | Corner::BottomRight => view.bottom() - margin - size.y,
        };
        Pos2::new(x, y)
    }
    /// The corner of `view` nearest the centre of `panel`.
    pub(crate) fn nearest(view: Rect, panel: Rect) -> Corner {
        let c = panel.center();
        match (c.x < view.center().x, c.y < view.center().y) {
            (true, true) => Corner::TopLeft,
            (false, true) => Corner::TopRight,
            (true, false) => Corner::BottomLeft,
            (false, false) => Corner::BottomRight,
        }
    }
}

/// What the app hands the engine room each frame, besides the snapshot.
#[derive(Clone, Debug, Default)]
pub(crate) struct EngineFrame {
    /// Nominal steps (pixels × ss² × ask) of the passes dispatched this frame.
    pub(crate) nominal_steps: u64,
    /// Steps the GPU executed, when a counter reading landed this frame (0 = none).
    pub(crate) executed_steps: u64,
    /// The nominal steps that reading covers.
    pub(crate) executed_of: u64,
    /// Pixels in the passes dispatched this frame.
    pub(crate) px: u64,
    /// A worker GPU is enabled.
    pub(crate) worker: bool,
    pub(crate) ram: u64,
    pub(crate) ram_peak: u64,
    pub(crate) ram_total: u64,
    /// GPU memory held, bytes, by kind (G-buffers, hold and SSAA, orbit and BLA, worker reference).
    pub(crate) vram: [u64; 4],
    pub(crate) vram_cap_mb: u64,
}

/// One frame's contribution to the engine room's rates.
#[derive(Clone, Copy, Debug, Default)]
struct Sample {
    t_s: f64,
    asked: f64,
    exec: f64,
    exec_of: f64,
    gpu_ms: f64,
    worker_ms: f64,
    px: f64,
}

/// How long the engine room's rates average over, seconds.
const RATE_WINDOW_S: f64 = 1.0;
/// How often the core strip samples CPU use, seconds.
const CORE_SAMPLE_S: f64 = 0.25;
/// How long the pointer rests before a help card opens, seconds.
const HELP_DELAY_S: f64 = 0.25;
/// The panel's margin from the view's edge, points.
const MARGIN: f32 = 10.0;

/// The panel as the app holds it.
pub(crate) struct Overlay {
    pub(crate) size: Size,
    pub(crate) corner: Corner,
    pub(crate) opacity: f32,
    model: Model,
    engine: Engine,
    /// The event ring's read position; `None` until the panel first opens.
    cursor: Option<u64>,
    scratch: Vec<events::Event>,
    cores: crate::sysinfo::CoreSampler,
    cores_at: f64,
    rates: VecDeque<Sample>,
    /// The panel's offset from its corner while it is dragged.
    drag: egui::Vec2,
    /// The help card under the pointer: its key, the region, and since when.
    hover: Option<(&'static str, Rect, f64)>,
    /// A help card held open whatever the pointer does (the `--uitest` screenshot of one).
    pub(crate) pinned_help: Option<&'static str>,
    /// The static layer's shapes, kept between frames (`panel::Statics`).
    statics: panel::Statics,
    /// The case's hammertone ([`paint::hammertone`]), generated and uploaded the first time the
    /// panel opens.
    hammer: Option<egui::TextureHandle>,
    /// The panel's own UI-thread cost while open (design/blinkenlights.md, "Cost": ≤ 0.5 ms a frame):
    /// the sum and worst of the frames since the last log line, and how many; and each section's
    /// sum (`panel::SECTIONS`).
    cost: (f64, f64, u32),
    sections: [f64; 11],
}

/// Frames between the panel's cost lines in the log.
const COST_LOG_FRAMES: u32 = 600;

impl Overlay {
    pub(crate) fn from_session(size: &str, corner: &str, opacity: f32) -> Overlay {
        Overlay {
            size: Size::from_name(size).unwrap_or_default(),
            corner: Corner::from_name(corner),
            opacity: if opacity.is_finite() { opacity.clamp(0.3, 1.0) } else { 0.95 },
            model: Model::default(),
            engine: Engine::default(),
            cursor: None,
            scratch: Vec::new(),
            cores: Default::default(),
            cores_at: f64::NEG_INFINITY,
            rates: VecDeque::new(),
            drag: egui::Vec2::ZERO,
            hover: None,
            pinned_help: None,
            cost: (0.0, 0.0, 0),
            sections: [0.0; 11],
            statics: panel::Statics::default(),
            hammer: None,
        }
    }

    /// `B`: off → strip → panel → full → off.
    pub(crate) fn cycle(&mut self) {
        self.size = self.size.next();
    }

    pub(crate) fn is_open(&self) -> bool {
        self.size != Size::Off
    }

    /// Records one frame's panel cost (UI thread, ms) and logs the mean and worst every
    /// [`COST_LOG_FRAMES`] frames the panel is open.
    pub(crate) fn note_cost(&mut self, ms: f64) {
        let c = &mut self.cost;
        c.0 += ms;
        c.1 = c.1.max(ms);
        c.2 += 1;
        if c.2 >= COST_LOG_FRAMES {
            let n = c.2 as f64;
            let parts: Vec<String> =
                panel::SECTIONS.iter().zip(self.sections.iter()).map(|(k, v)| format!("{k} {:.3}", v / n)).collect();
            crate::diag::log_line(
                "blinken",
                &format!(
                    "live panel ({}): {:.3} ms a frame mean, {:.3} worst, over {} frames, static layer drawn {} times | {}",
                    self.size.name(),
                    c.0 / n,
                    c.1,
                    c.2,
                    self.statics.drawn,
                    parts.join(", ")
                ),
            );
            *c = (0.0, 0.0, 0);
            self.sections = [0.0; 11];
        }
    }

    /// One frame: the events since last frame and this frame's snapshot into the model, and the
    /// engine room's samples. Does nothing while the panel is off (the ring keeps filling).
    pub(crate) fn update(&mut self, snap: Snapshot, ef: &EngineFrame) {
        if !self.is_open() {
            self.cursor = None;
            return;
        }
        // Opening: start from now (old events would replay as a burst of lamps).
        let from = *self.cursor.get_or_insert_with(events::pushed);
        self.scratch.clear();
        self.cursor = Some(events::read_since(from, &mut self.scratch));
        // Only the main view's events (the dual view's second panel is not on this panel).
        self.scratch.retain(|e| e.view == 0);
        let t_s = crate::app_micros() as f64 / 1e6;
        let mut sample = Sample { t_s, asked: ef.nominal_steps as f64, exec: ef.executed_steps as f64, exec_of: ef.executed_of as f64, px: ef.px as f64, ..Sample::default() };
        for e in &self.scratch {
            match e.kind {
                events::Kind::PassTimed if e.a == events::Pass::Worker as u32 => sample.worker_ms += e.f.max(0.0) as f64,
                events::Kind::PassTimed => sample.gpu_ms += e.f.max(0.0) as f64,
                _ => {}
            }
        }
        self.model.update(&self.scratch, snap);
        self.rates.push_back(sample);
        while self.rates.front().is_some_and(|s| t_s - s.t_s > RATE_WINDOW_S) {
            self.rates.pop_front();
        }
        let span = self.rates.front().map_or(RATE_WINDOW_S, |s| (t_s - s.t_s).max(0.25));
        let sum = |f: fn(&Sample) -> f64| self.rates.iter().map(f).sum::<f64>();
        let e = &mut self.engine;
        e.steps_asked = sum(|s| s.asked) / span;
        let (exec, of) = (sum(|s| s.exec), sum(|s| s.exec_of));
        // Executed steps are counted on the passes a counter reading covers: scale the asked rate
        // by that sample's ratio, so a frame with no reading does not read as zero work.
        e.steps_done = if of > 0.0 { e.steps_asked * (exec / of).clamp(0.0, 1.0) } else { 0.0 };
        e.total_steps += ef.nominal_steps as f64;
        e.gpu_busy = [(sum(|s| s.gpu_ms) / span / 1000.0) as f32, (sum(|s| s.worker_ms) / span / 1000.0) as f32];
        e.worker = ef.worker;
        let mb = |b: u64| b as f32 / (1024.0 * 1024.0);
        e.ram_mb = mb(ef.ram);
        e.ram_peak_mb = mb(ef.ram_peak.max(ef.ram));
        e.ram_cap_mb = mb(ef.ram_total);
        e.vram_mb = ef.vram.map(mb);
        e.vram_cap_mb = ef.vram_cap_mb as f32;
        e.threads = registry::snapshot();
        e.px_per_s = sum(|s| s.px) / span;
        if t_s - self.cores_at >= CORE_SAMPLE_S {
            self.cores_at = t_s;
            if let Some(c) = self.cores.sample() {
                e.cores = c;
            }
        }
        let px = e.px_per_s;
        self.model.stream(snap.dt_s, px);
    }

    /// Draws the panel over `view` (the fractal area) and its help card, and handles dragging.
    pub(crate) fn show(&mut self, ctx: &egui::Context, view: Rect, df32_from: f32, fe_from: f32) {
        if !self.is_open() {
            return;
        }
        let width = panel::DESIGN_W;
        let h1 = panel::height(self.size, width);
        let s = ((view.width() - 2.0 * MARGIN) / width).min((view.height() - 2.0 * MARGIN) / h1).clamp(0.35, 1.0);
        let size = vec2(width, h1) * s;
        let at = self.corner.place(view, size, MARGIN) + self.drag;
        let clock = clock_text();
        let version = format!("v{}", env!("CARGO_PKG_VERSION"));
        let mut hits = Vec::new();
        let mut sections = [0.0f64; 11];
        let hammer = self
            .hammer
            .get_or_insert_with(|| {
                ctx.load_texture("blinken-hammertone", paint::hammertone(paint::HAMMER_TILE, 0x4a11_e7), egui::TextureOptions::LINEAR_REPEAT)
            })
            .id();
        let inp = panel::Inputs { model: &self.model, engine: &self.engine, clock: &clock, version: &version, fe_from, df32_from, hammer };
        let opacity = self.opacity;
        let area = egui::Area::new(Id::new("fractadyne.blinken"))
            .fixed_pos(at)
            .order(egui::Order::Middle)
            .interactable(true)
            .show(ctx, |ui| {
                let (rect, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
                let mut p = ui.painter_at(rect.expand(8.0));
                p.multiply_opacity(opacity);
                panel::draw(&p, rect.min, width, s, self.size, opacity, &inp, &mut hits, &mut sections, &mut self.statics);
                resp
            });
        let resp = area.inner;
        for (a, b) in self.sections.iter_mut().zip(sections) {
            *a += b;
        }
        if resp.dragged() {
            self.drag += resp.drag_delta();
        }
        if resp.drag_stopped() {
            let r = Rect::from_min_size(at, size);
            self.corner = Corner::nearest(view, r);
            self.drag = egui::Vec2::ZERO;
        }
        // A double-click cycles the size, as `B` does.
        if resp.double_clicked() {
            self.cycle();
        }
        self.show_help(ctx, &hits, resp.hovered() && !resp.dragged());
        // While anything on the panel moves, keep frames coming; once it is still, only the clock's
        // seconds need a frame, so an idle app still idles (design/blinkenlights.md, "Cost").
        let next = if self.model.active() { 50 } else { 1000 };
        ctx.request_repaint_after(std::time::Duration::from_millis(next));
    }

    fn show_help(&mut self, ctx: &egui::Context, hits: &[panel::Hit], hovered: bool) {
        let now = ctx.input(|i| i.time);
        let pointer = ctx.pointer_hover_pos();
        let hit = pointer.filter(|_| hovered).and_then(|p| hits.iter().find(|h| h.rect.contains(p)).copied());
        match (hit, self.hover) {
            (Some(h), Some((k, _, _))) if k == h.key => {}
            (Some(h), _) => self.hover = Some((h.key, h.rect, now)),
            (None, _) => self.hover = None,
        }
        if let Some(h) = self.pinned_help.and_then(|k| hits.iter().find(|h| h.key == k)) {
            self.hover = Some((h.key, h.rect, f64::NEG_INFINITY));
        }
        let Some((key, rect, since)) = self.hover else { return };
        if now - since < HELP_DELAY_S {
            ctx.request_repaint_after(std::time::Duration::from_millis(60));
            return;
        }
        // Only a key the panel draws has a card (the lists the help tests hold to the cards).
        if !panel::HELP_KEYS.contains(&key) && !panel::LAMP_KEYS.contains(&key) {
            return;
        }
        let Some(card) = help::card(key) else { return };
        let screen = ctx.screen_rect();
        let w = 350.0f32.min(screen.width() - 24.0);
        let x = (pointer.map_or(rect.left(), |p| p.x) - 30.0).clamp(screen.left() + 12.0, screen.right() - w - 12.0);
        let below = rect.bottom() + 8.0;
        // Below the region if a typical card fits, else above it.
        let pivot = if below + 260.0 < screen.bottom() { (Pos2::new(x, below), Align2::LEFT_TOP) } else { (Pos2::new(x, rect.top() - 8.0), Align2::LEFT_BOTTOM) };
        egui::Area::new(Id::new("fractadyne.blinken.help"))
            .order(egui::Order::Tooltip)
            .fixed_pos(pivot.0)
            .pivot(pivot.1)
            .interactable(false)
            .show(ctx, |ui| help_card(ui, card, w));
    }
}

/// A help card: the operator's manual, cream paper and dark ink.
fn help_card(ui: &mut egui::Ui, card: &help::Card, width: f32) {
    let ink = paint::ink::INK;
    egui::Frame::new()
        .fill(Color32::from_rgb(0xf1, 0xe7, 0xca))
        .stroke(egui::Stroke::new(1.0_f32, Color32::from_rgb(0x8f, 0x88, 0x75)))
        .corner_radius(5.0)
        .inner_margin(egui::Margin::symmetric(14, 11))
        .shadow(egui::Shadow { offset: [0, 10], blur: 24, spread: 0, color: Color32::from_black_alpha(150) })
        .show(ui, |ui| {
            ui.set_width(width - 28.0);
            ui.spacing_mut().item_spacing.y = 4.0;
            ui.label(egui::RichText::new(card.title.to_uppercase()).font(paint::face::engrave(13.0)).color(Color32::from_rgb(0x3b, 0x37, 0x2d)));
            ui.separator();
            let body = |t: &str| egui::RichText::new(t).size(13.5).color(ink);
            ui.label(body(card.what));
            if let Some(n) = card.note {
                ui.label(body(n));
            }
            for (label, items, dot) in [("HEALTHY", card.good, Color32::from_rgb(0x3d, 0x9a, 0x3d)), ("TROUBLE", card.bad, Color32::from_rgb(0xc4, 0x36, 0x1c))] {
                if items.is_empty() {
                    continue;
                }
                ui.add_space(4.0);
                ui.label(egui::RichText::new(label).size(10.5).strong().color(Color32::from_rgb(0x5a, 0x54, 0x44)));
                for it in items {
                    ui.horizontal_top(|ui| {
                        let (r, _) = ui.allocate_exact_size(vec2(12.0, 16.0), Sense::hover());
                        ui.painter().circle_filled(r.center() + vec2(0.0, 1.0), 4.0, dot);
                        // A label in a row extends unless told to wrap: the card keeps its width.
                        ui.add(egui::Label::new(body(it)).wrap());
                    });
                }
            }
        });
}

/// The wall clock for the nixies: `yyyy-mm-dd_hh:mm:ss.s` in local time ('_' is the gap between
/// date and time), or UTC where local time cannot be read.
pub(crate) fn clock_text() -> String {
    let (y, mo, d, h, mi, s, ms) = crate::sysinfo::local_now().unwrap_or_else(utc_now);
    format!("{y:04}-{mo:02}-{d:02}_{h:02}:{mi:02}:{s:02}.{}", ms / 100)
}

/// UTC from the system clock (civil-from-days, Howard Hinnant's algorithm).
fn utc_now() -> (i32, u32, u32, u32, u32, u32, u32) {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs() as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = (yoe + era * 400 + i64::from(m <= 2)) as i32;
    (y, m, d, (rem / 3600) as u32, (rem % 3600 / 60) as u32, (rem % 60) as u32, now.subsec_millis())
}

#[cfg(test)]
#[path = "../tests/unit/blinken/tests.rs"]
mod tests;
