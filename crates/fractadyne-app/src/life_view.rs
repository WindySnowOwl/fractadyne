//! Life in the app (design/automata.md, phase 2): the session's universe, playback, drawing, the
//! side-panel section, pattern files, and the frame that hands it all to the GPU.
//!
//! **Who holds the cells.** The GPU universe (`fractadyne_gpu::life`) is the one that runs. The app
//! keeps `loaded` — the universe it last told the GPU to load — and treats it as current only while
//! the GPU has not stepped past it. A change (a drawn cell, a new rule, a fill) is made to a CPU
//! copy and loaded whole: when the GPU has moved on, the app first asks for a download, holds the
//! change, and applies it to what comes back. So a change never lands on a stale universe.

use crate::{FractadyneApp, FractalKind};
use fractadyne_core::life::{self, Rule, Topology, Universe};
use fractadyne_gpu::life::{LifeFrame, LifeStatus};
use std::sync::{Arc, Mutex};

/// What a left-drag in the view does.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum LifeTool {
    Pan,
    /// Edit cells: a click flips a cell; a drag paints what its first cell became — alive over a
    /// dead cell, dead over a live one — so one tool both draws and erases.
    Draw,
}

/// A pointer event a harness feeds the app (`--uitest`'s life-draw step), at a CELL of the Life view.
#[derive(Clone, Copy, Debug)]
pub(crate) enum UiPointer {
    Move(i64, i64),
    Press(i64, i64),
    Release(i64, i64),
    /// The pointer leaves the window (as it was before the script: the harness has no pointer).
    Gone,
}

/// A change to the universe, waiting for a current copy to be made to.
#[derive(Clone, Debug)]
enum Change {
    /// Part of a mouse stroke. `start`: its first cell decides the stroke's state (the opposite of
    /// what that cell holds), resolved against the universe it is applied to — which may be a
    /// download that arrived after the press, never a stale copy.
    Stroke { cells: Vec<(i64, i64)>, start: bool },
    Rule(Rule),
    Clear,
    Fill { x: i64, y: i64, w: u32, h: u32, density: f64, seed: u64 },
}

/// What the Draw tool does, for its hover texts.
const DRAW_HINT: &str = "click a cell to flip it; drag to paint — a stroke that starts on a dead cell \
                         draws, one that starts on a live cell erases. A running universe pauses while \
                         you draw and runs on when you let go.";

/// Strokes undo at most this far back, and only universes up to `UNDO_MAX_TILES` tiles keep a copy.
const UNDO_DEPTH: usize = 64;
const UNDO_MAX_TILES: usize = 4096;

/// Generations a frame may run at most: each batch of 16 waits for a read-back.
const MAX_STEPS_PER_FRAME: u64 = 1024;
/// Tile-generations a frame may run (a tile is 4,096 cells): keeps a big universe's frame short.
const MAX_TILE_STEPS_PER_FRAME: u64 = 1 << 16;
/// The pattern a new session opens with.
const DEFAULT_PATTERN: &str = "Gosper glider gun";

pub(crate) struct LifeState {
    /// The rule as typed, and why the last one typed was refused.
    pub(crate) rule_text: String,
    pub(crate) rule_error: Option<String>,
    /// What Reset returns to: the pattern as opened, with any edits made at generation 0.
    pub(crate) start: Arc<Universe>,
    /// What the GPU was last told to load.
    pub(crate) loaded: Arc<Universe>,
    pub(crate) load_id: u64,
    pub(crate) pattern_name: String,
    pub(crate) playing: bool,
    /// Generations a second while playing.
    pub(crate) speed: f64,
    owed: f64,
    /// The generation to show: playback and the Step buttons raise it, the GPU steps towards it a
    /// frame's worth at a time. (A target, not a count to consume — see `LifeFrame::target`.)
    pub(crate) target: u64,
    /// `log₂` of the big Step button's stride.
    pub(crate) stride_log2: u32,
    pub(crate) tool: LifeTool,
    pub(crate) status: Arc<Mutex<LifeStatus>>,
    want_download: bool,
    pending: Vec<Change>,
    /// A download asked for by Save, to be written when it arrives.
    save_to: Option<std::path::PathBuf>,
    pub(crate) density: f64,
    pub(crate) seed: u64,
    last_t: Option<f64>,
    /// The cell the last drag point drew, so a fast stroke draws a line, not dots.
    last_cell: Option<(i64, i64)>,
    /// The input time of the last frame a stroke point was taken from: egui may lay a frame out
    /// twice, and the second pass must not take the same input again — under flip semantics a
    /// replayed press undoes what the first pass drew.
    stroke_frame: Option<f64>,
    /// A stroke is in progress (pressed on the view and not yet released).
    stroke_open: bool,
    /// The state the current stroke paints, once its first cell has been resolved.
    stroke_state: Option<u8>,
    /// Playback was running when the stroke began: it resumes when the stroke ends.
    resume_after_stroke: bool,
    /// The universe before each stroke, newest last (cleared once the universe steps on).
    undo: Vec<Arc<Universe>>,
    /// Faint cell borders when zoomed in to 8 px a cell or more.
    pub(crate) grid: bool,
    /// The "Pattern text" dialog.
    pub(crate) text_open: bool,
    pub(crate) text: String,
    pub(crate) text_error: Option<String>,
    /// Shown in the panel: why the last step, load or save failed.
    pub(crate) message: Option<String>,
}

fn universe_from(pattern: &life::Pattern, rule: Rule) -> Universe {
    let mut u = Universe::new(rule, Topology::Plane).expect("the plane is always valid");
    for &(x, y, s) in &pattern.cells {
        u.set(x, y, s.min((u.rule().states() - 1) as u8));
    }
    u
}

impl Default for LifeState {
    fn default() -> Self {
        let p = life::library::pattern_named(DEFAULT_PATTERN).expect("the default pattern is in the library");
        let rule = Rule::parse(p.rule).expect("library rules parse");
        let u = Arc::new(universe_from(&life::parse_rle(p.rle).expect("library patterns parse"), rule.clone()));
        LifeState {
            rule_text: rule.canonical(),
            rule_error: None,
            start: u.clone(),
            loaded: u,
            load_id: 1,
            pattern_name: p.name.to_string(),
            playing: false,
            speed: 30.0,
            owed: 0.0,
            target: 0,
            stride_log2: 4,
            tool: LifeTool::Pan,
            status: Arc::new(Mutex::new(LifeStatus::default())),
            want_download: false,
            pending: Vec::new(),
            save_to: None,
            density: 0.35,
            seed: 1,
            last_t: None,
            last_cell: None,
            stroke_frame: None,
            stroke_open: false,
            stroke_state: None,
            resume_after_stroke: false,
            undo: Vec::new(),
            grid: true,
            text_open: false,
            text: String::new(),
            text_error: None,
            message: None,
        }
    }
}

impl LifeState {
    /// A copy of the GPU's status.
    pub(crate) fn status(&self) -> LifeStatus {
        self.status.lock().map(|s| LifeStatus { downloaded: None, ..s.clone() }).unwrap_or_default()
    }

    /// The generation on screen.
    pub(crate) fn generation(&self) -> u64 {
        let s = self.status();
        if s.load_id == self.load_id { s.generation } else { self.loaded.generation() }
    }

    /// The population on screen (cells that differ from the background).
    pub(crate) fn population(&self) -> u64 {
        let s = self.status();
        if s.load_id == self.load_id { s.population } else { self.loaded.population() }
    }

    /// Whether `loaded` is still what the GPU holds (it has not stepped since).
    fn in_sync(&self) -> bool {
        let s = self.status();
        s.load_id != self.load_id || s.generation == self.loaded.generation()
    }

    /// Load `u` as the universe, generation and all (and stop any run still owed).
    fn load(&mut self, u: Universe) {
        self.target = u.generation();
        self.loaded = Arc::new(u);
        self.load_id += 1;
        self.owed = 0.0;
    }

    /// Advance `n` generations past what is on screen (or already asked for), paused.
    pub(crate) fn step(&mut self, n: u64) {
        self.playing = false;
        self.target = self.target.max(self.generation()).saturating_add(n);
    }

    /// Stop where the universe is now (playback may have run ahead of the screen by a frame or two).
    pub(crate) fn pause(&mut self) {
        self.playing = false;
        self.target = self.generation();
    }

    /// Whether generations are still to run.
    pub(crate) fn running(&self) -> bool {
        self.playing || self.target > self.generation()
    }

    /// Make `change` — now if `loaded` is current, else once a download arrives.
    fn change(&mut self, change: Change) {
        self.pending.push(change);
        if self.in_sync() {
            self.apply_pending(None);
        } else {
            self.pause();
            self.want_download = true;
        }
    }

    /// Apply the waiting changes to `base` (a download) or to `loaded`.
    fn apply_pending(&mut self, base: Option<Universe>) {
        let mut u = base.unwrap_or_else(|| (*self.loaded).clone());
        let at_start = u.generation() == 0;
        for c in std::mem::take(&mut self.pending) {
            // Every change but a stroke's continuation can be undone: keep the universe before it.
            if !matches!(c, Change::Stroke { start: false, .. }) || self.stroke_state.is_none() {
                self.remember_for_undo(&u);
            }
            match c {
                Change::Stroke { cells, start } => {
                    if start || self.stroke_state.is_none() {
                        let first = cells.first().map_or(0, |&(x, y)| u.get(x, y));
                        self.stroke_state = Some(u8::from(first == 0));
                    }
                    let state = self.stroke_state.unwrap_or(1);
                    for (x, y) in cells {
                        u.set(x, y, state);
                    }
                }
                Change::Rule(r) => u = rebuild(&u, r),
                Change::Clear => u.clear(),
                Change::Fill { x, y, w, h, density, seed } => u.random_fill(x, y, w, h, density, seed),
            }
        }
        if at_start || u.generation() == 0 {
            self.start = Arc::new(u.clone());
        }
        self.load(u);
    }

    /// A stroke point at `cell`, from the frame whose input time is `frame`: `pressed` on the press
    /// frame, `click` on the release of a press that never became a drag.
    fn stroke(&mut self, cell: (i64, i64), pressed: bool, click: bool, frame: f64) {
        // The same frame laid out again: its input was taken already.
        if self.stroke_frame == Some(frame) {
            return;
        }
        self.stroke_frame = Some(frame);
        // A click's release continues the stroke its press began (egui forgets the press time by
        // then); a press and release within one frame start and end one.
        let started = pressed || (click && !self.stroke_open);
        if started {
            self.stroke_open = true;
            self.stroke_state = None;
            if self.playing {
                self.resume_after_stroke = true;
            }
            self.pause();
        } else if self.last_cell == Some(cell) {
            return; // held still: nothing new to paint
        }
        let from = if started { cell } else { self.last_cell.unwrap_or(cell) };
        self.last_cell = Some(cell);
        self.change(Change::Stroke { cells: line(from, cell), start: started });
    }

    /// The button is up: the stroke is over, and a universe it paused runs on.
    fn stroke_end(&mut self) {
        self.stroke_open = false;
        if std::mem::take(&mut self.resume_after_stroke) {
            self.playing = true;
        }
    }

    /// Keep `u` as the state an Undo returns to (small universes only: a copy each).
    fn remember_for_undo(&mut self, u: &Universe) {
        if u.tile_count() <= UNDO_MAX_TILES {
            self.undo.push(Arc::new(u.clone()));
            if self.undo.len() > UNDO_DEPTH {
                self.undo.remove(0);
            }
        }
    }

    /// Back to the universe before the last edit. `false` when there is none to go back to.
    pub(crate) fn undo_edit(&mut self) -> bool {
        let Some(u) = self.undo.pop() else { return false };
        self.pending.clear();
        self.want_download = false;
        self.pause();
        let u = (*u).clone();
        if u.generation() == self.start.generation() {
            self.start = Arc::new(u.clone());
        }
        self.load(u);
        true
    }

    /// Whether an edit can be undone.
    pub(crate) fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Take a download the GPU delivered: apply waiting changes to it, or save it.
    fn take_download(&mut self) -> Option<Universe> {
        let u = self.status.lock().ok()?.downloaded.take()?;
        self.want_download = false;
        Some(u)
    }
}

/// `u`'s cells under `rule`: states past the rule's are cleared; a live background (B0/S8) stays
/// live where the new rule is binary.
fn rebuild(u: &Universe, rule: Rule) -> Universe {
    let mut v = Universe::new(rule, u.topology()).expect("an existing topology is valid");
    v.set_generation(u.generation());
    if u.background() != 0 {
        v.set_background(u.background());
    }
    let max = (v.rule().states() - 1) as u8;
    for (x, y, s) in u.cells() {
        v.set(x, y, if s <= max { s } else { 0 });
    }
    v
}

impl FractadyneApp {
    /// The Life frame: playback for this frame, the colouring, and the universe's commands.
    pub(crate) fn build_life_params(&mut self, now: f64, resolution: [u32; 2], ss: u32) -> fractadyne_gpu::MandelbrotParams {
        // A download that arrived: changes waiting for it, or a save.
        if let Some(u) = self.life.take_download() {
            if let Some(path) = self.life.save_to.take() {
                self.life.message = Some(match write_pattern(&path, &u) {
                    Ok(()) => format!("Saved {}", path.display()),
                    Err(e) => format!("Save failed: {e}"),
                });
            }
            if !self.life.pending.is_empty() {
                self.life.apply_pending(Some(u));
            }
        }
        let status = self.life.status();
        if let Some(e) = status.error.clone() {
            self.life.pause();
            self.life.message = Some(format!("Stopped: {e}"));
            if let Ok(mut s) = self.life.status.lock() {
                s.error = None;
            }
        }
        // ⭐Everything here must be safe to run twice for one painted frame: egui may lay a frame out
        // again and paint only the second pass. Time accrues once (the second pass sees dt ≈ 0) and
        // the request is a TARGET generation, which a repeat asks for again rather than consumes.
        let dt = self.life.last_t.map_or(0.0, |t| (now - t).clamp(0.0, 0.25));
        self.life.last_t = Some(now);
        let cap = (MAX_TILE_STEPS_PER_FRAME / (status.tiles.max(1) as u64)).clamp(1, MAX_STEPS_PER_FRAME);
        // (Not while changes wait for a download: they must land where they were made.)
        if self.life.playing && self.life.pending.is_empty() {
            self.life.owed += self.life.speed * dt;
            let n = self.life.owed.floor();
            self.life.owed -= n;
            // Playback behind the clock lets the excess go rather than owe it for ever; a run asked
            // for (Step, a view's generation) is kept whole and runs a frame's worth at a time.
            let shown = self.life.generation();
            self.life.target = self.life.target.max(shown).saturating_add(n as u64).min(shown.saturating_add(2 * cap));
        }
        let window = life::cell_window(&self.viewport).unwrap_or(life::CellWindow {
            tile_x0: 0,
            tile_y0: 0,
            origin: [0.0, 0.0],
            cells_per_px: 1.0,
        });
        let frame = LifeFrame {
            load_id: self.life.load_id,
            load: self.life.loaded.clone(),
            target: self.life.target,
            max_steps: cap,
            window,
            status: self.life.status.clone(),
            download: self.life.want_download,
        };
        let (lut, lut_smooth) = self.active_lut();
        fractadyne_gpu::MandelbrotParams {
            life: Some(Arc::new(frame)),
            formula: fractadyne_core::formula::LIFE,
            lut,
            lut_smooth,
            // Life writes intensities in (0, 1]: half a palette cycle, rotated by the offset — a
            // live cell lands mid-palette, where palettes are bright, rather than at an end.
            cycle: 0.5,
            offset: self.coloring.offset,
            interior_col: self.interior_color(),
            aa_palette: crate::palette_aa_enabled(),
            resolution,
            ss,
            view_id: 0,
            ..Default::default()
        }
    }

    /// Whether a left-drag draws rather than pans.
    pub(crate) fn life_draws(&self) -> bool {
        self.fractal == FractalKind::Life && self.life.tool != LifeTool::Pan
    }

    /// The cell under pixel `(px, py)` of the view (physical pixels from its top-left) — from the
    /// exact cell window, so it is right however far from the origin the view is.
    pub(crate) fn life_cell_at(&self, px: f64, py: f64) -> Option<(i64, i64)> {
        let w = life::cell_window(&self.viewport)?;
        let at = |tile: i64, origin: f64, p: f64| tile.checked_mul(life::TILE)?.checked_add((origin + p * w.cells_per_px).floor() as i64);
        Some((at(w.tile_x0, w.origin[0], px)?, at(w.tile_y0, w.origin[1], py)?))
    }

    /// A stroke point at pixel `(px, py)` with the Draw tool (see [`LifeState::stroke`]). A stroke
    /// pauses a running universe and resumes it when it ends.
    pub(crate) fn life_stroke(&mut self, px: f64, py: f64, pressed: bool, click: bool, frame: f64) {
        if let Some(cell) = self.life_cell_at(px, py) {
            self.life.stroke(cell, pressed, click, frame);
        }
    }

    /// `p` as the egui event a real mouse would send: at the middle of its cell in the view, in
    /// points (the view's rect from the last frame's layout). `None` before there is a layout.
    pub(crate) fn life_pointer_event(&self, ctx: &egui::Context, p: UiPointer) -> Option<egui::Event> {
        let rect = self.perf.layout.central?;
        let w = life::cell_window(&self.viewport)?;
        let ppp = f64::from(ctx.pixels_per_point());
        let at = |cx: i64, cy: i64| {
            let px = ((cx - w.tile_x0 * life::TILE) as f64 + 0.5 - w.origin[0]) / w.cells_per_px / ppp;
            let py = ((cy - w.tile_y0 * life::TILE) as f64 + 0.5 - w.origin[1]) / w.cells_per_px / ppp;
            rect.min + egui::vec2(px as f32, py as f32)
        };
        let button = |cx, cy, pressed| egui::Event::PointerButton {
            pos: at(cx, cy),
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        Some(match p {
            UiPointer::Move(x, y) => egui::Event::PointerMoved(at(x, y)),
            UiPointer::Press(x, y) => button(x, y, true),
            UiPointer::Release(x, y) => button(x, y, false),
            UiPointer::Gone => egui::Event::PointerGone,
        })
    }

    /// The mouse was released: a universe the stroke paused runs on.
    pub(crate) fn life_stroke_end(&mut self) {
        self.life.stroke_end();
    }

    /// Frame the universe: its bounding box with a margin, or 128 cells around the origin.
    pub(crate) fn life_home(&mut self) {
        let (w, h) = (self.viewport.width_px.max(1.0), self.viewport.height_px.max(1.0));
        let (cx, cy, cells) = match self.life.loaded.bounding_box() {
            Some((x0, y0, x1, y1)) => {
                let span = ((x1 - x0 + 1) as f64 / w).max((y1 - y0 + 1) as f64 / h) * 1.25;
                ((x0 + x1 + 1) as f64 * 0.5, -((y0 + y1 + 1) as f64 * 0.5), span.max(64.0 / h))
            }
            None => (0.5, -0.5, 128.0 / h),
        };
        self.viewport.reset_to(cx, cy);
        self.viewport.units_per_pixel = fractadyne_core::FloatExp::from_f64(cells);
        self.pointer.zoom_vel = 0.0;
    }

    /// Open a pattern from the library.
    pub(crate) fn life_open_library(&mut self, name: &str) {
        let Some(p) = life::library::pattern_named(name) else { return };
        let Ok(pattern) = life::parse_rle(p.rle) else { return };
        self.life_open(pattern, Some(p.rule), p.name.to_string());
    }

    /// Show `pattern`: under the rule it names (else the current one), at generation 0, framed.
    fn life_open(&mut self, pattern: life::Pattern, rule: Option<&str>, name: String) {
        self.life_set(pattern, rule, name);
        if self.fractal != FractalKind::Life {
            self.set_fractal(FractalKind::Life);
        }
        self.life_home();
    }

    /// Make `pattern` the universe (under the rule it names, else the current one) at generation 0,
    /// paused, without changing what is on screen.
    fn life_set(&mut self, pattern: life::Pattern, rule: Option<&str>, name: String) {
        let rule = match rule.map(Rule::parse) {
            Some(Ok(r)) => r,
            Some(Err(e)) => {
                self.life.message = Some(format!("The pattern's rule was not understood ({e}); kept {}", self.life.loaded.rule()));
                self.life.loaded.rule().clone()
            }
            None => self.life.loaded.rule().clone(),
        };
        self.life.rule_text = rule.canonical();
        self.life.rule_error = None;
        let u = universe_from(&pattern, rule);
        self.life.start = Arc::new(u.clone());
        self.life.pending.clear();
        self.life.want_download = false;
        self.life.load(u);
        self.life.pattern_name = name;
        self.life.playing = false;
    }

    /// A Life view's universe: `pattern` (at generation `at`) under `rule`, then run on to generation
    /// `shown`; with `show`, switched to (the view's own centre and zoom are applied by the caller
    /// afterwards), else only kept (a session whose view is another family).
    pub(crate) fn life_open_view(&mut self, pattern: life::Pattern, rule: &str, name: String, at: u64, shown: u64, show: bool) -> Result<(), String> {
        /// More than this is a hostile or broken file, not a view (a run of hours).
        const MAX_RUN: u64 = 100_000_000;
        let rule = Rule::parse(rule).map_err(|e| format!("the Life rule does not read: {e}"))?;
        let canonical = rule.canonical();
        if show {
            self.life_open(pattern, Some(&canonical), name);
        } else {
            self.life_set(pattern, Some(&canonical), name);
        }
        let mut u = (*self.life.loaded).clone();
        u.set_generation(at);
        self.life.start = Arc::new(u.clone());
        self.life.load(u);
        let run = shown.saturating_sub(at);
        if run > MAX_RUN {
            return Err(format!("generation {shown} is {run} past the pattern's — more than {MAX_RUN} to run; shown at {at}"));
        }
        self.life.target = shown;
        Ok(())
    }

    /// File > Open pattern…: RLE, plaintext, Life 1.05 / 1.06.
    pub(crate) fn life_open_file(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Life patterns", &["rle", "cells", "lif", "life", "txt"])
            .set_directory(self.dialog_dir_default())
            .pick_file()
        else {
            return;
        };
        self.remember_dir(&path);
        let name = path.file_stem().map_or_else(|| "pattern".into(), |s| s.to_string_lossy().into_owned());
        match std::fs::read_to_string(&path).map_err(|e| e.to_string()).and_then(|t| life::parse_pattern(&t).map_err(|e| e.to_string())) {
            Ok((p, _)) => {
                let rule = p.rule.clone();
                let name = p.name.clone().unwrap_or(name);
                self.life_open(p, rule.as_deref(), name);
            }
            Err(e) => self.life.message = Some(format!("Could not open {}: {e}", path.display())),
        }
    }

    /// The Pattern text dialog's Load: the text as a pattern.
    pub(crate) fn life_open_text(&mut self) {
        match life::parse_pattern(&self.life.text) {
            Ok((p, _)) => {
                let rule = p.rule.clone();
                let name = p.name.clone().unwrap_or_else(|| "pasted pattern".into());
                self.life_open(p, rule.as_deref(), name);
                self.life.text_open = false;
                self.life.text_error = None;
            }
            Err(e) => self.life.text_error = Some(e.to_string()),
        }
    }

    /// File > Save pattern…: the universe as it is now, as RLE.
    pub(crate) fn life_save_file(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Run-length encoded pattern", &["rle"])
            .set_directory(self.dialog_dir_default())
            .set_file_name(format!("{}.rle", self.life.pattern_name))
            .save_file()
        else {
            return;
        };
        self.remember_dir(&path);
        if self.life.in_sync() {
            let u = (*self.life.loaded).clone();
            self.life.message = Some(match write_pattern(&path, &u) {
                Ok(()) => format!("Saved {}", path.display()),
                Err(e) => format!("Save failed: {e}"),
            });
        } else {
            self.life.save_to = Some(path);
            self.life.pause();
            self.life.want_download = true;
        }
    }

    /// The side panel's Life section.
    pub(crate) fn life_panel(&mut self, ui: &mut egui::Ui) {
        let status = self.life.status();
        if !status.available && status.load_id != 0 {
            ui.colored_label(egui::Color32::from_rgb(0xE0, 0x80, 0x40), "This graphics adapter cannot run Life (no compute shaders).");
        }
        // Pattern: the library, a file, pasted text.
        crate::ui::labelled(ui, "Pattern", |ui| {
            let mut pick = None;
            let r = egui::ComboBox::from_id_salt("life_pattern")
                .selected_text(self.life.pattern_name.clone())
                .width(150.0)
                .show_ui(ui, |ui| {
                    for p in life::library::PATTERNS {
                        if ui.selectable_label(self.life.pattern_name == p.name, p.name).on_hover_text(p.about).clicked() {
                            pick = Some(p.name);
                        }
                    }
                })
                .response;
            if let Some(name) = pick {
                self.life_open_library(name);
            }
            r
        });
        ui.horizontal(|ui| {
            if ui.button("Open…").on_hover_text("Open an RLE, .cells or Life 1.05/1.06 pattern file.").clicked() {
                self.life_open_file();
            }
            if ui.button("Pattern text…").on_hover_text("Paste a pattern (RLE as LifeWiki gives it, or plaintext).").clicked() {
                self.life.text_open = true;
            }
            if ui.button("Save…").on_hover_text("Save the universe as it is now, as RLE.").clicked() {
                self.life_save_file();
            }
        });
        // Rule: the library or typed.
        crate::ui::labelled(ui, "Rule", |ui| {
            let mut pick = None;
            let r = egui::ComboBox::from_id_salt("life_rule")
                .selected_text(
                    life::library::RULES
                        .iter()
                        .find(|r| r.rule == self.life.loaded.rule().canonical())
                        .map_or("custom", |r| r.name),
                )
                .width(150.0)
                .show_ui(ui, |ui| {
                    for r in life::library::RULES {
                        if ui.selectable_label(false, format!("{}  {}", r.name, r.rule)).on_hover_text(r.about).clicked() {
                            pick = Some(r.rule);
                        }
                    }
                })
                .response;
            if let Some(rule) = pick {
                self.life.rule_text = rule.to_string();
                self.life_apply_rule();
            }
            r
        });
        ui.horizontal(|ui| {
            let r = ui.add(egui::TextEdit::singleline(&mut self.life.rule_text).desired_width(150.0).hint_text("B3/S23"));
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) || ui.button("Apply").clicked() {
                self.life_apply_rule();
            }
        });
        if let Some(e) = &self.life.rule_error {
            ui.colored_label(egui::Color32::from_rgb(0xE0, 0x60, 0x60), e);
        }
        ui.separator();
        // Transport.
        ui.horizontal(|ui| {
            let label = if self.life.playing { "\u{23F8} Pause" } else { "\u{25B6} Play" };
            if ui.button(label).on_hover_text("Run the universe (Enter).").clicked() {
                self.life_toggle_play();
            }
            if ui.button("Step").on_hover_text("Advance one generation (N).").clicked() {
                self.life.step(1);
            }
            let stride = 1u64 << self.life.stride_log2;
            if ui.button(format!("+{}", grouped(stride))).on_hover_text("Advance this many generations (Shift+N).").clicked() {
                self.life.step(stride);
            }
            if ui.button("Reset").on_hover_text("Back to the pattern as opened (generation 0).").clicked() {
                self.life_reset();
            }
        });
        crate::ui::labelled(ui, "Speed", |ui| {
            ui.add(egui::Slider::new(&mut self.life.speed, 1.0..=10_000.0).logarithmic(true).suffix(" gen/s"))
        });
        crate::ui::labelled(ui, "Stride", |ui| {
            ui.add(egui::Slider::new(&mut self.life.stride_log2, 1..=16).custom_formatter(|v, _| grouped(1u64 << v as u32)))
        });
        egui::Grid::new("life_status").num_columns(2).show(ui, |ui| {
            ui.label("Generation");
            ui.label(grouped(self.life.generation()));
            ui.end_row();
            ui.label("Population");
            ui.label(grouped(self.life.population()));
            ui.end_row();
            ui.label("Tiles");
            ui.label(format!("{} of {}", grouped(status.tiles as u64), grouped(u64::from(status.capacity))))
                .on_hover_text("Stored 64×64-cell tiles (4 KiB each) and the GPU pool's capacity.");
            ui.end_row();
        });
        if let Some(m) = &self.life.message {
            ui.label(egui::RichText::new(m).small());
        }
        ui.separator();
        // Editing.
        ui.horizontal(|ui| {
            ui.label("Mouse");
            ui.selectable_value(&mut self.life.tool, LifeTool::Pan, "Pan").on_hover_text("Drag to move the view.");
            ui.selectable_value(&mut self.life.tool, LifeTool::Draw, "Draw").on_hover_text(DRAW_HINT);
            if ui
                .add_enabled(self.life.can_undo(), egui::Button::new("Undo"))
                .on_hover_text("Back to the universe before the last edit, fill, clear or rule change (Ctrl+Z while drawing).")
                .clicked()
            {
                self.life.undo_edit();
            }
        });
        ui.checkbox(&mut self.life.grid, "Grid lines").on_hover_text("Cell borders when zoomed in to 8 pixels a cell or more.");
        ui.horizontal(|ui| {
            ui.label("Random");
            ui.add(egui::DragValue::new(&mut self.life.density).range(0.01..=1.0).speed(0.01).fixed_decimals(2))
                .on_hover_text("The fraction of cells a fill sets alive.");
            ui.label("seed");
            ui.add(egui::DragValue::new(&mut self.life.seed).range(0..=u64::MAX >> 11));
            if ui.button("Fill view").on_hover_text("Fill the visible area (up to 2,048 × 2,048 cells about its centre) at random.").clicked() {
                self.life_fill_view();
            }
        });
        if ui.button("Clear").on_hover_text("Kill every cell (Undo brings them back).").clicked() {
            self.life.pause();
            self.life.change(Change::Clear);
        }
    }

    /// The toolbar's Life group, in the slot Julia and the dual view take for an escape-time
    /// family (neither applies to a universe): run / pause, step, back to the start, draw.
    pub(crate) fn life_toolbar(&mut self, ui: &mut egui::Ui) {
        let (icon, hint) = if self.life.playing {
            (crate::icons::PAUSE, "Pause the universe (Enter)")
        } else {
            (crate::icons::PLAY, "Run the universe (Enter)")
        };
        if ui.button(icon).on_hover_text(hint).clicked() {
            self.life_toggle_play();
        }
        let stride = 1u64 << self.life.stride_log2;
        if ui
            .button(crate::icons::FORWARD)
            .on_hover_text(format!("Step one generation (N); Shift+click or Shift+N steps {}", grouped(stride)))
            .clicked()
        {
            let n = if ui.input(|i| i.modifiers.shift) { stride } else { 1 };
            self.life.step(n);
        }
        if ui.button(crate::icons::SKIP_BACK).on_hover_text("Back to the pattern as opened (generation 0)").clicked() {
            self.life_reset();
        }
        if ui
            .add(egui::SelectableLabel::new(self.life.tool == LifeTool::Draw, crate::icons::EDIT))
            .on_hover_text(format!("Draw (D): {DRAW_HINT}"))
            .clicked()
        {
            self.life_toggle_draw();
        }
    }

    pub(crate) fn life_toggle_play(&mut self) {
        if self.life.playing {
            self.life.pause();
        } else {
            self.life.playing = true;
        }
    }

    pub(crate) fn life_toggle_draw(&mut self) {
        self.life.tool = if self.life.tool == LifeTool::Draw { LifeTool::Pan } else { LifeTool::Draw };
    }

    /// Back to the start pattern (generation 0, with the edits made there).
    pub(crate) fn life_reset(&mut self) {
        self.life.pause();
        self.life.pending.clear();
        self.life.want_download = false;
        let u = (*self.life.loaded).clone();
        self.life.remember_for_undo(&u);
        let start = (*self.life.start).clone();
        self.life.load(start);
    }

    /// Life's keys, while no text field has the keyboard: Enter runs and pauses, N steps one
    /// generation (Shift+N a stride), D toggles drawing, and Ctrl+Z undoes an edit while drawing.
    /// Returns whether it took Ctrl+Z (so navigation undo does not also run).
    pub(crate) fn life_hotkeys(&mut self, ctx: &egui::Context) -> bool {
        if self.fractal != FractalKind::Life {
            return false;
        }
        // A focused button takes Enter itself (egui clicks it): leave Enter to it then.
        let focused = ctx.memory(|m| m.focused().is_some());
        let (enter, n, shift_n, d, undo) = ctx.input(|i| {
            let plain = !i.modifiers.command && !i.modifiers.alt;
            (
                plain && !i.modifiers.shift && i.key_pressed(egui::Key::Enter),
                plain && !i.modifiers.shift && i.key_pressed(egui::Key::N),
                plain && i.modifiers.shift && i.key_pressed(egui::Key::N),
                plain && !i.modifiers.shift && i.key_pressed(egui::Key::D),
                i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::Z),
            )
        });
        if enter && !focused {
            self.life_toggle_play();
        }
        if n {
            self.life.step(1);
        }
        if shift_n {
            self.life.step(1u64 << self.life.stride_log2);
        }
        if d {
            self.life_toggle_draw();
        }
        undo && self.life.tool == LifeTool::Draw && self.life.undo_edit()
    }

    /// Over the view: faint cell borders when zoomed in, and the cell under the cursor outlined
    /// while drawing. `rect` is the view in points; `hover` the pointer, if over it.
    pub(crate) fn life_overlay(&self, painter: &egui::Painter, rect: egui::Rect, ppp: f32, hover: Option<egui::Pos2>) {
        let Some(w) = life::cell_window(&self.viewport) else { return };
        // One cell, in points; the view's top-left corner sits `origin` cells into tile (x0, y0).
        let cell = (1.0 / w.cells_per_px) as f32 / ppp;
        let to_x = |cx: i64| rect.min.x + ((cx - w.tile_x0 * life::TILE) as f64 - w.origin[0]) as f32 * cell;
        let to_y = |cy: i64| rect.min.y + ((cy - w.tile_y0 * life::TILE) as f64 - w.origin[1]) as f32 * cell;
        let first = |tile: i64, origin: f64| tile * life::TILE + origin.ceil() as i64;
        if self.life.grid && cell >= 8.0 / ppp {
            let alpha = ((cell * ppp - 8.0) * 3.0).clamp(0.0, 28.0) as u8;
            let stroke = egui::Stroke::new(1.0 / ppp, egui::Color32::from_white_alpha(alpha));
            let mut x = first(w.tile_x0, w.origin[0]);
            while to_x(x) <= rect.max.x {
                painter.vline(to_x(x), rect.y_range(), stroke);
                x += 1;
            }
            let mut y = first(w.tile_y0, w.origin[1]);
            while to_y(y) <= rect.max.y {
                painter.hline(rect.x_range(), to_y(y), stroke);
                y += 1;
            }
        }
        if self.life.tool == LifeTool::Draw {
            if let Some(p) = hover {
                let l = p - rect.min;
                if let Some((cx, cy)) = self.life_cell_at(f64::from(l.x * ppp), f64::from(l.y * ppp)) {
                    let r = egui::Rect::from_min_max(egui::pos2(to_x(cx), to_y(cy)), egui::pos2(to_x(cx + 1), to_y(cy + 1)));
                    // At least a few points across, so the target shows when cells are tiny.
                    let r = egui::Rect::from_center_size(r.center(), r.size().max(egui::vec2(5.0, 5.0)));
                    painter.rect_stroke(r, egui::CornerRadius::ZERO, egui::Stroke::new(1.5_f32, crate::BRAND_ACCENT), egui::StrokeKind::Outside);
                }
            }
        }
    }

    /// Apply the typed rule to the universe as it is.
    pub(crate) fn life_apply_rule(&mut self) {
        match Rule::parse(&self.life.rule_text) {
            Ok(r) => {
                self.life.rule_error = None;
                self.life.rule_text = r.canonical();
                if r != *self.life.loaded.rule() {
                    self.life.change(Change::Rule(r));
                }
            }
            Err(e) => self.life.rule_error = Some(e.to_string()),
        }
    }

    /// Fill what the view shows (capped about its centre) at random.
    fn life_fill_view(&mut self) {
        let Some(w) = life::cell_window(&self.viewport) else { return };
        let cells_w = (self.viewport.width_px * w.cells_per_px).ceil().min(2048.0) as u32;
        let cells_h = (self.viewport.height_px * w.cells_per_px).ceil().min(2048.0) as u32;
        let (cx, cy) = self.viewport.center_f64();
        let (x, y) = ((cx - f64::from(cells_w) * 0.5).floor() as i64, (-cy - f64::from(cells_h) * 0.5).floor() as i64);
        let (density, seed) = (self.life.density, self.life.seed);
        self.life.seed = self.life.seed.wrapping_add(1);
        self.life.change(Change::Fill { x, y, w: cells_w.max(1), h: cells_h.max(1), density, seed });
    }

    /// The Pattern text dialog.
    pub(crate) fn life_text_dialog(&mut self, ctx: &egui::Context) {
        if !self.life.text_open {
            return;
        }
        let mut open = true;
        egui::Window::new("Pattern text").open(&mut open).default_width(420.0).show(ctx, |ui| {
            ui.label("Paste a pattern: RLE (as LifeWiki and Golly give it), plaintext .cells, or Life 1.05 / 1.06.");
            egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                ui.add(egui::TextEdit::multiline(&mut self.life.text).code_editor().desired_rows(10).desired_width(f32::INFINITY));
            });
            if let Some(e) = &self.life.text_error {
                ui.colored_label(egui::Color32::from_rgb(0xE0, 0x60, 0x60), e);
            }
            ui.horizontal(|ui| {
                if ui.button("Load").clicked() {
                    self.life_open_text();
                }
                if ui.button("Current as RLE").on_hover_text("Fill the box with the universe as last loaded.").clicked() {
                    let u = &self.life.loaded;
                    self.life.text = life::write_rle(&u.cells(), &u.rule().canonical(), u.rule().states() > 2, Some(&self.life.pattern_name), Some(u.generation()));
                }
            });
        });
        if !open {
            self.life.text_open = false;
        }
    }
}

/// The status bar's three Life readouts — scale, generation, population — each padded to a fixed
/// width, so a growing count never wraps the bar (the reflow rule every readout keeps; a wrap
/// resizes the canvas, which reads as an interaction).
pub(crate) fn status_readouts(cells_per_px: f64, generation: u64, population: u64) -> (String, String, String) {
    // Counts up to 999,999,999,999 grouped (15 characters), past that in scientific notation; a
    // scale is at most `1 px = 9.99e300 cells` (21).
    const COUNT_W: usize = 15;
    const SCALE_W: usize = 21;
    let count = |n: u64| if n < 1_000_000_000_000 { grouped(n) } else { format!("{:.3e}", n as f64) };
    let sig = |v: f64| if (0.01..10_000.0).contains(&v) { format!("{:.3}", v).trim_end_matches('0').trim_end_matches('.').to_string() } else { format!("{v:.2e}") };
    let scale = if cells_per_px >= 1.0 {
        format!("1 px = {} cells", sig(cells_per_px))
    } else {
        format!("{} px / cell", sig(1.0 / cells_per_px.max(1e-300)))
    };
    (
        format!("scale {scale:>SCALE_W$}"),
        format!("gen {:>COUNT_W$}", count(generation)),
        format!("pop {:>COUNT_W$}", count(population)),
    )
}

/// `n` with its digits grouped by commas (the user's standard for counts).
pub(crate) fn grouped(n: u64) -> String {
    crate::commas(&n.to_string())
}

/// The cells of the line from `a` to `b` (Bresenham), both ends included.
fn line(a: (i64, i64), b: (i64, i64)) -> Vec<(i64, i64)> {
    let (mut x, mut y) = a;
    let (dx, dy) = ((b.0 - x).abs(), -(b.1 - y).abs());
    let (sx, sy) = (if x < b.0 { 1 } else { -1 }, if y < b.1 { 1 } else { -1 });
    let mut err = dx + dy;
    let mut out = Vec::new();
    loop {
        out.push((x, y));
        if (x, y) == b || out.len() > 1 << 16 {
            return out;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

fn write_pattern(path: &std::path::Path, u: &Universe) -> Result<(), String> {
    let name = path.file_stem().map(|s| s.to_string_lossy().into_owned());
    let text = life::write_rle(&u.cells(), &u.rule().canonical(), u.rule().states() > 2, name.as_deref(), Some(u.generation()));
    std::fs::write(path, text).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests;
