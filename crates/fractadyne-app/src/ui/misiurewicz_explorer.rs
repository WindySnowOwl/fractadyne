//! The Misiurewicz point explorer: browse what different Misiurewicz points LOOK like, and
//! zoom to any chosen one at an arbitrary depth (user request, 2026-08-31).
//!
//! The gallery stores POINTS, never bare `(k,p)` pairs — `(k,p)` names an EQUATION whose root
//! count is ~2^(k+p−2), so a picker keyed on the pair alone is meaningless above tiny k (the
//! TODO entry's counting note; verified exactly as 2^(k−1)−1 for p=1). Each entry carries the
//! solved coordinate itself, which is what selects the root: the coordinate seeds the deep
//! re-solve when a jump asks for more depth than the stored digits carry.
//!
//! Entries come from two places: the hand-curated [`crate::MISIUREWICZ_POI`] list (the
//! explorer is the generated, navigable version of that menu), and a seeded-Newton sweep over
//! the upper half-plane for every small type — solve [`fractadyne_core::find_misiurewicz`]
//! from a coarse seed grid, keep what converges, label each hit by its CANONICAL pair
//! (re-detected from the critical orbit, so a non-minimal solve ask cannot mislabel a point),
//! and dedupe by coordinate. The sweep is honest about coverage: a coarse grid reaches most
//! basins, not provably all, and the dialog says how many points it found rather than
//! claiming completeness. Upper half-plane only — conjugates mirror.
//!
//! Beside each thumbnail: the multiplier readout. `log₂|λ|` is the ZOOM PERIOD (the view
//! self-repeats every that-many octaves of a dive) and `arg λ` the twist per repeat — the
//! numbers that say what diving here will look like, before you go.

use eframe::egui;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::mpsc;
use std::time::Instant;

/// One gallery point: an exact pre-periodic parameter, its canonical type, and its readouts.
pub(crate) struct MisiEntry {
    pub(crate) name: String,
    /// Canonical (preperiod, period) — re-detected from the orbit, not the solve ask.
    pub(crate) kp: (u32, u32),
    /// The exact coordinate, as decimal strings at the generation precision. This is the SEED
    /// for any deeper re-solve; it only has to sit inside the point's Newton basin.
    pub(crate) cx: String,
    pub(crate) cy: String,
    /// log₂ magnification the thumbnail renders at (the entry's display/feature scale).
    pub(crate) thumb_l2: f64,
    /// (log₂|λ|, twist in degrees) of the landing cycle, when the multiplier converged.
    pub(crate) mult: Option<(f64, f64)>,
    pub(crate) curated: bool,
}

/// A running deep-solve for a jump: worker channel + what was asked.
pub(crate) struct MisiJump {
    rx: mpsc::Receiver<Result<fractadyne_core::Misiurewicz, fractadyne_core::MisiurewiczMiss>>,
    started: Instant,
    target_l2: f64,
    label: String,
}

/// Dialog state. Entries are generated once per session, on first open, off-thread.
#[derive(Default)]
pub(crate) struct MisiExplorer {
    pub(crate) open: bool,
    entries: Vec<MisiEntry>,
    gen_rx: Option<mpsc::Receiver<Vec<MisiEntry>>>,
    gen_started: Option<Instant>,
    thumbs: std::collections::HashMap<usize, egui::TextureHandle>,
    selected: Option<usize>,
    zoom: String,
    jump: Option<MisiJump>,
    msg: Option<String>,
    /// Harness request (`--uitest`): once the gallery is ready, select the first entry (the
    /// antenna tip — the sort puts curated entries first) and solve-jump it to 1e6×. Drives
    /// the REAL selection + solve + navigation path, not a copy of it.
    pub(crate) uitest_jump: bool,
}

/// The generated sweep's solve types. Small on purpose: root counts double per step of k+p,
/// and the gallery is a browser, not a census — (6,1) alone contributes up to 31 points.
const SWEEP_PAIRS: &[(u32, u32)] =
    &[(2, 1), (3, 1), (4, 1), (5, 1), (6, 1), (2, 2), (3, 2), (4, 2), (2, 3), (3, 3)];

/// Display magnification for generated entries, as log₂ (≈1e4× — deep enough that the local
/// branch/spiral structure fills the frame at every small type, shallow enough to render in
/// microseconds). Curated entries keep their hand-picked depths.
const SWEEP_THUMB_L2: f64 = 13.287_712_379_549_45; // log2(1e4)

/// Build the gallery: curated POI entries plus the seeded-Newton sweep. Runs on a worker.
fn generate_entries() -> Vec<MisiEntry> {
    use fractadyne_core as fc;
    let mut out: Vec<MisiEntry> = Vec::new();
    let mut seen: Vec<(f64, f64)> = Vec::new(); // dedupe key: f64 coordinate, tol 1e-9

    // The curated list first, verbatim — names people know, at depths someone chose by eye.
    for (name, cx, cy, mag) in crate::MISIUREWICZ_POI {
        let l2 = mag.log2();
        let prec = 128usize;
        let (bx, by) = match (fc::parse_bf_prec(cx, prec), fc::parse_bf_prec(cy, prec)) {
            (Some(a), Some(b)) => (a, b),
            _ => continue,
        };
        // The canonical pair from the orbit itself (the names carry one, but re-derive rather
        // than parse prose); fall back to (0,0) = "unknown" if detection declines.
        let kp = fc::detect_misiurewicz_at_scale(&bx, &by, 0, 4096, 1024, prec, Some(1.585 - l2))
            .unwrap_or((0, 0));
        let mult = (kp.0 > 0)
            .then(|| fc::misiurewicz_multiplier(&bx, &by, kp.0, kp.1, 0, prec))
            .flatten()
            .map(|m| (m.log2_abs, m.arg.to_degrees()));
        seen.push((fc::to_f64(&bx), fc::to_f64(&by)));
        out.push(MisiEntry {
            name: (*name).to_string(),
            kp,
            cx: (*cx).to_string(),
            cy: (*cy).to_string(),
            thumb_l2: l2,
            mult,
            curated: true,
        });
    }

    // The sweep: a coarse seed grid over the upper half-plane (one extra row ON the real
    // axis, where the (k,1) antenna points live), solved for each small type. The solves are
    // independent, so they fan out across threads; the results are then SORTED before the
    // dedupe so the gallery's content and order are run-stable regardless of thread timing.
    let t0 = Instant::now();
    let solve_l2 = SWEEP_THUMB_L2;
    let mut ims: Vec<f64> = vec![0.0];
    for j in 0..14 {
        ims.push(0.03 + 1.22 * (j as f64) / 13.0);
    }
    let res: Vec<f64> = (0..22).map(|i| -2.05 + 2.75 * (i as f64) / 21.0).collect();
    let mut tasks: Vec<(u32, u32, f64, f64)> = Vec::new();
    for &(k, p) in SWEEP_PAIRS {
        for &im in &ims {
            for &re in &res {
                tasks.push((k, p, re, im));
            }
        }
    }
    let n_threads = std::thread::available_parallelism().map_or(4, |n| n.get().min(16));
    // Phase A — SOLVE ONLY, in parallel. Detection is deliberately NOT here: hundreds of
    // seeds converge onto each root (Newton basins are wide), and running the canonical
    // detector per convergence made the sweep minutes long. Solve first, dedupe by
    // coordinate, and detect once per UNIQUE point in phase B.
    let chunk = tasks.len().div_ceil(n_threads);
    let mut sols: Vec<(f64, f64, String, String)> = std::thread::scope(|s| {
        let handles: Vec<_> = tasks
            .chunks(chunk)
            .map(|part| {
                s.spawn(move || {
                    let mut hits = Vec::new();
                    for &(k, p, re, im) in part {
                        let prec = 128usize;
                        let seed =
                            [fc::BigFloat::from_f64(re, prec), fc::BigFloat::from_f64(im, prec)];
                        let Ok(m) = fc::find_misiurewicz(
                            &seed,
                            k,
                            p,
                            fc::SolveScale { log2_seed: 0.0, log2_target: solve_l2 },
                            0,
                        ) else {
                            continue;
                        };
                        let (fx, fy) = (fc::to_f64(&m.cx), fc::to_f64(&m.cy));
                        if fy < -1.0e-9 {
                            continue; // lower half-plane: the conjugate mirrors
                        }
                        hits.push((
                            fx,
                            fy,
                            fc::to_decimal_string(&m.cx),
                            fc::to_decimal_string(&m.cy),
                        ));
                    }
                    hits
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().unwrap_or_default()).collect()
    });
    // Deterministic order, then coordinate dedupe (run-stable regardless of thread timing).
    sols.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut unique: Vec<(f64, f64, String, String)> = Vec::new();
    for s in sols {
        if seen.iter().any(|(sx, sy)| (sx - s.0).abs() < 1.0e-9 && (sy - s.1).abs() < 1.0e-9) {
            continue;
        }
        seen.push((s.0, s.1));
        unique.push(s);
    }
    // Phase B — canonical label + multiplier, once per unique point, in parallel. A (4,1)
    // solve ask happily converges onto a (3,1) point (every (3,1) point satisfies the (4,1)
    // equation), so the label comes from the ORBIT, never from the ask; the detection window
    // is sized for these tiny types (k+p ≤ 9), not for deep-field discovery.
    let chunk_b = unique.len().div_ceil(n_threads).max(1);
    let mut found: Vec<MisiEntry> = std::thread::scope(|s| {
        let handles: Vec<_> = unique
            .chunks(chunk_b)
            .map(|part| {
                s.spawn(move || {
                    let mut entries = Vec::new();
                    for (fx, fy, cxs, cys) in part {
                        let prec = 128usize;
                        let (Some(bx), Some(by)) =
                            (fc::parse_bf_prec(cxs, prec), fc::parse_bf_prec(cys, prec))
                        else {
                            continue;
                        };
                        let Some(ckp) = fc::detect_misiurewicz_at_scale(
                            &bx,
                            &by,
                            0,
                            256,
                            64,
                            prec,
                            Some(1.585 - solve_l2),
                        ) else {
                            continue;
                        };
                        let mult = fc::misiurewicz_multiplier(&bx, &by, ckp.0, ckp.1, 0, prec)
                            .map(|l| (l.log2_abs, l.arg.to_degrees()));
                        // ⚠A Misiurewicz point lands on a REPELLING cycle (log₂|λ| > 0). The
                        // preperiodicity equation is also satisfied by hyperbolic CENTERS —
                        // the sweep's first run listed c = −1 as "(1,2)" with log₂|λ| ≈ −39
                        // and a solid-interior thumbnail — so an attracting or unreadable
                        // multiplier disqualifies the root, it does not just decorate it.
                        if !mult.is_some_and(|(l2, _)| l2 > 0.05) {
                            continue;
                        }
                        entries.push(MisiEntry {
                            name: format!("({},{}) at {fx:.4}{fy:+.4}i", ckp.0, ckp.1),
                            kp: ckp,
                            cx: cxs.clone(),
                            cy: cys.clone(),
                            thumb_l2: SWEEP_THUMB_L2,
                            mult,
                            curated: false,
                        });
                    }
                    entries
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().unwrap_or_default()).collect()
    });
    found.sort_by(|a, b| (a.kp.0 + a.kp.1, a.kp, &a.cx).cmp(&(b.kp.0 + b.kp.1, b.kp, &b.cx)));
    out.extend(found);
    crate::diag::breadcrumb(format!(
        "misiurewicz sweep: {} points ({} curated) from {} seeded solves in {:.1}s",
        out.len(),
        out.iter().filter(|e| e.curated).count(),
        tasks.len(),
        t0.elapsed().as_secs_f64()
    ));
    // Curated first, then by type size, then position — a stable, browsable order.
    out.sort_by(|a, b| {
        (std::cmp::Reverse(a.curated), a.kp.0 + a.kp.1, a.kp, &a.cx)
            .cmp(&(std::cmp::Reverse(b.curated), b.kp.0 + b.kp.1, b.kp, &b.cx))
    });
    out
}

impl crate::FractadyneApp {
    /// Whether the explorer's gallery is fully populated (generation done, every thumbnail
    /// rendered). The uitest capture gate waits on this so the screenshot shows the real
    /// gallery rather than the generation spinner; bounded by the step's hard cap as ever.
    pub(crate) fn misi_gallery_ready(&self) -> bool {
        !self.misi.open
            || (self.misi.gen_rx.is_none()
                && !self.misi.entries.is_empty()
                && self.misi.thumbs.len() >= self.misi.entries.len())
    }

    /// Whether the harness-driven jump is still in flight (requested or solving) — the
    /// uitest capture gate waits this out on the jump step.
    pub(crate) fn misi_jump_busy(&self) -> bool {
        self.misi.uitest_jump || self.misi.jump.is_some()
    }

    /// Open the explorer (menu entry point). Generation starts on first open only.
    pub(crate) fn open_misiurewicz_explorer(&mut self) {
        self.misi.open = true;
        self.misi.msg = None;
        if self.misi.entries.is_empty() && self.misi.gen_rx.is_none() {
            let (tx, rx) = mpsc::channel();
            self.misi.gen_rx = Some(rx);
            self.misi.gen_started = Some(Instant::now());
            std::thread::spawn(move || {
                let _ = tx.send(generate_entries());
            });
        }
        if self.misi.zoom.is_empty() {
            self.misi.zoom = "1e50".into();
        }
    }

    /// Poll the generation and jump workers, and render at most ONE missing thumbnail per
    /// frame (a thumbnail is a small synchronous offscreen render + readback — budgeting one
    /// per frame keeps the UI fluid while the gallery fills in over a second or two).
    pub(crate) fn poll_misiurewicz_explorer(
        &mut self,
        ctx: &egui::Context,
        gpu: Option<&(eframe::wgpu::Device, eframe::wgpu::Queue)>,
    ) {
        if let Some(rx) = &self.misi.gen_rx {
            match rx.try_recv() {
                Ok(entries) => {
                    self.misi.entries = entries;
                    self.misi.gen_rx = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => self.misi.gen_rx = None,
                Err(mpsc::TryRecvError::Empty) => {
                    if self.misi.open {
                        self.schedule_repaint(ctx);
                    }
                }
            }
        }
        // The jump solve is polled even with the dialog closed — closing abandons it, exactly
        // like the Go-to feature solve (an answer must never move the view out from under a
        // user who dismissed the wait).
        if !self.misi.open {
            self.misi.jump = None;
            return;
        }
        if let Some(j) = &self.misi.jump {
            self.schedule_repaint(ctx);
            if let Ok(outcome) = j.rx.try_recv() {
                let j = self.misi.jump.take().expect("checked above");
                match outcome {
                    Ok(m) => {
                        self.viewport.set_center_log2mag(m.cx, m.cy, j.target_l2);
                        self.finish_nav_jump();
                        self.misi.open = false;
                        let iter_note = crate::deep_jump_iter_shortfall(
                            j.target_l2,
                            self.render_cfg.max_iter,
                            self.render_cfg.auto_iter,
                        )
                        .map(|(have, typical)| {
                            format!(
                                " ⚠ Iterations is fixed at {have}; this depth typically needs \
                                 ~{typical} — raise it (or enable auto-iterations) if the view \
                                 renders flat."
                            )
                        })
                        .unwrap_or_default();
                        self.set_toast(
                            format!(
                                "Zoomed to {} — {}× ({:.1}s){iter_note}",
                                j.label,
                                crate::fmt_zoom_field(j.target_l2),
                                j.started.elapsed().as_secs_f64()
                            ),
                            ctx,
                        );
                    }
                    Err(why) => {
                        self.misi.msg = Some(format!(
                            "The deep solve did not land on {}: {why:?}. The stored point is \
                             still exact at its own depth — try a shallower target.",
                            j.label
                        ));
                    }
                }
            }
        }
        // Harness-requested jump: fire once the gallery is ready, through the same code a
        // click uses. The uitest busy gate holds the capture until the jump lands.
        if self.misi.uitest_jump
            && self.misi.gen_rx.is_none()
            && !self.misi.entries.is_empty()
            && self.misi.jump.is_none()
        {
            self.misi.uitest_jump = false;
            self.misi.selected = Some(0);
            self.misi.zoom = "1e6".into();
            self.start_misi_jump();
        }
        // One missing thumbnail per frame, dialog open only.
        if let Some((dev, q)) = gpu {
            let missing = (0..self.misi.entries.len())
                .find(|i| !self.misi.thumbs.contains_key(i));
            if let Some(i) = missing {
                if let Some(tex) = self.render_misi_thumb(ctx, dev, q, i) {
                    self.misi.thumbs.insert(i, tex);
                }
                self.schedule_repaint(ctx);
            }
        }
    }

    /// Render one entry's thumbnail offscreen at its display depth: the export pipeline at
    /// 144×90 (2× supersampled), colored with the CURRENT palette so the gallery matches what
    /// a jump will show. Shallow depths render direct/df32 — no bignum reference to build —
    /// so each is a sub-millisecond dispatch plus a readback.
    fn render_misi_thumb(
        &self,
        ctx: &egui::Context,
        dev: &eframe::wgpu::Device,
        q: &eframe::wgpu::Queue,
        i: usize,
    ) -> Option<egui::TextureHandle> {
        let e = &self.misi.entries[i];
        let prec = fractadyne_core::precision_for_octaves(e.thumb_l2.max(1.0).ceil() as u64) as usize;
        let cx = fractadyne_core::parse_bf_prec(&e.cx, prec)?;
        let cy = fractadyne_core::parse_bf_prec(&e.cy, prec)?;
        let mut vp = self.viewport.clone();
        vp.width_px = 144.0;
        vp.height_px = 90.0;
        vp.set_center_log2mag(cx, cy, e.thumb_l2);
        let mut req = self.current_export_request_for(&vp, false);
        req.width = 144;
        req.height = 90;
        req.ss = 2;
        let progress = AtomicU32::new(0);
        let cancel = AtomicBool::new(false);
        let res = fractadyne_gpu::render_export(dev, q, &req, &progress, &cancel).ok()?;
        let bytes = fractadyne_export::to_srgb8(&res.pixels);
        let img = egui::ColorImage::from_rgba_unmultiplied(
            [res.width as usize, res.height as usize],
            &bytes,
        );
        Some(ctx.load_texture(format!("misi-thumb-{i}"), img, Default::default()))
    }

    /// The explorer window: a wrapped gallery of thumbnails with type + multiplier readouts,
    /// and a depth field + solve-and-jump for the selected point.
    pub(crate) fn draw_misiurewicz_explorer(&mut self, ctx: &egui::Context) {
        if !self.misi.open {
            return;
        }
        let mut open = self.misi.open;
        let mut clicked: Option<usize> = None;
        let mut do_jump = false;
        let mut cancel_jump = false;
        egui::Window::new("Misiurewicz explorer")
            .open(&mut open)
            .default_width(620.0)
            .default_height(460.0)
            .resizable(true)
            .show(ctx, |ui| {
                if self.fractal.formula_id() != 0 {
                    ui.label("Misiurewicz browsing is Mandelbrot-only — switch the fractal to use it.");
                    return;
                }
                ui.horizontal(|ui| {
                    if self.misi.gen_rx.is_some() {
                        ui.add(egui::Spinner::new().size(14.0));
                        ui.label(
                            egui::RichText::new(format!(
                                "Solving the small-type families ({:.0}s)…",
                                self.misi.gen_started.map_or(0.0, |t| t.elapsed().as_secs_f64())
                            ))
                            .weak(),
                        );
                    } else {
                        let gen = self.misi.entries.iter().filter(|e| !e.curated).count();
                        ui.label(
                            egui::RichText::new(format!(
                                "{} points — {} curated, {gen} found by a seeded Newton sweep of \
                                 the small types (upper half-plane; a coarse grid finds most \
                                 basins, not provably all).",
                                self.misi.entries.len(),
                                self.misi.entries.iter().filter(|e| e.curated).count(),
                            ))
                            .weak()
                            .small(),
                        );
                    }
                });
                ui.separator();
                let avail_h = ui.available_height() - 76.0;
                egui::ScrollArea::vertical().max_height(avail_h.max(120.0)).show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for i in 0..self.misi.entries.len() {
                            let e = &self.misi.entries[i];
                            let sel = self.misi.selected == Some(i);
                            ui.allocate_ui(egui::vec2(158.0, 152.0), |ui| {
                                ui.vertical(|ui| {
                                    let r = match self.misi.thumbs.get(&i) {
                                        Some(tex) => ui.add(
                                            egui::ImageButton::new((
                                                tex.id(),
                                                egui::vec2(144.0, 90.0),
                                            ))
                                            .selected(sel),
                                        ),
                                        None => {
                                            let (rect, r) = ui.allocate_exact_size(
                                                egui::vec2(144.0, 90.0),
                                                egui::Sense::click(),
                                            );
                                            ui.painter().rect_filled(
                                                rect,
                                                2.0,
                                                ui.visuals().extreme_bg_color,
                                            );
                                            egui::Spinner::new().paint_at(
                                                ui,
                                                egui::Rect::from_center_size(
                                                    rect.center(),
                                                    egui::vec2(18.0, 18.0),
                                                ),
                                            );
                                            r
                                        }
                                    };
                                    if r.clicked() {
                                        clicked = Some(i);
                                    }
                                    r.on_hover_text(format!(
                                        "{}\nre = {}\nim = {}",
                                        e.name, e.cx, e.cy
                                    ));
                                    ui.add(
                                        egui::Label::new(egui::RichText::new(&e.name).small())
                                            .truncate(),
                                    );
                                    ui.label(
                                        egui::RichText::new(match e.mult {
                                            Some((l2, deg)) => format!(
                                                "({},{}) · {l2:.2} oct/repeat · {deg:+.1}°",
                                                e.kp.0, e.kp.1
                                            ),
                                            None => format!("({},{})", e.kp.0, e.kp.1),
                                        })
                                        .weak()
                                        .small()
                                        .monospace(),
                                    );
                                });
                            });
                        }
                    });
                });
                ui.separator();
                match self.misi.selected.and_then(|i| self.misi.entries.get(i)) {
                    Some(e) => {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&e.name).strong());
                            ui.add_space(10.0);
                            ui.label("Zoom to");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.misi.zoom)
                                    .desired_width(90.0)
                                    .hint_text("1e50"),
                            );
                            if let Some(elapsed) =
                                self.misi.jump.as_ref().map(|j| j.started.elapsed())
                            {
                                ui.add(egui::Spinner::new().size(14.0));
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{:>7.1}s",
                                        elapsed.as_secs_f64()
                                    ))
                                    .weak()
                                    .monospace(),
                                );
                                if ui.button("Cancel").clicked() {
                                    cancel_jump = true;
                                }
                            } else if ui
                                .button("Solve & jump")
                                .on_hover_text(
                                    "Newton-solve this exact point to the precision the asked \
                                     depth needs (from its stored coordinate as the seed), then \
                                     jump straight there. Deep asks take longer — the solve is \
                                     arbitrary-precision.",
                                )
                                .clicked()
                            {
                                do_jump = true;
                            }
                        });
                    }
                    None => {
                        ui.label(
                            egui::RichText::new(
                                "Select a point, name a depth, and jump — the multiplier line \
                                 under each thumbnail is what a dive there will look like: how \
                                 many octaves before the view repeats, and how far it twists \
                                 per repeat.",
                            )
                            .weak()
                            .small(),
                        );
                    }
                }
                if let Some(m) = &self.misi.msg {
                    ui.label(egui::RichText::new(m).color(ui.visuals().warn_fg_color).small());
                }
            });
        if let Some(i) = clicked {
            self.misi.selected = Some(i);
            self.misi.msg = None;
        }
        if do_jump {
            self.start_misi_jump();
        }
        if cancel_jump {
            self.misi.jump = None;
            self.misi.msg = Some("Solve cancelled.".into());
        }
        self.misi.open = open && self.misi.open;
    }

    /// Spawn the deep solve for the selected entry at the asked depth. The stored coordinate
    /// seeds Newton; the target depth sets the working precision (`SolveScale` is log-based,
    /// so any depth the app renders is reachable — capped by [`crate::MAX_SOLVE_OCTAVES`]).
    fn start_misi_jump(&mut self) {
        if self.misi.jump.is_some() {
            return;
        }
        let Some(e) = self.misi.selected.and_then(|i| self.misi.entries.get(i)) else {
            return;
        };
        let Some(target_l2) = crate::parse_zoom_to_log2(&self.misi.zoom).filter(|t| t.is_finite())
        else {
            self.misi.msg = Some(format!(
                "Cannot read \"{}\" as a magnification (write 1e50, 3.2e120, …).",
                self.misi.zoom
            ));
            return;
        };
        let target_l2 = target_l2.min(crate::MAX_SOLVE_OCTAVES);
        if e.kp.0 == 0 {
            self.misi.msg = Some("This entry has no detected (k,p) to solve with.".into());
            return;
        }
        let prec = fractadyne_core::precision_for_octaves(target_l2.max(1.0).ceil() as u64) as usize;
        let (Some(cx), Some(cy)) = (
            fractadyne_core::parse_bf_prec(&e.cx, prec),
            fractadyne_core::parse_bf_prec(&e.cy, prec),
        ) else {
            self.misi.msg = Some("Stored coordinate failed to parse.".into());
            return;
        };
        let (k, p) = e.kp;
        let seed_l2 = e.thumb_l2;
        let label = e.name.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(fractadyne_core::find_misiurewicz(
                &[cx, cy],
                k,
                p,
                fractadyne_core::SolveScale { log2_seed: seed_l2, log2_target: target_l2 },
                0,
            ));
        });
        self.misi.jump = Some(MisiJump { rx, started: Instant::now(), target_l2, label });
        self.misi.msg = None;
    }
}
