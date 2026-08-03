//! The central fractal canvas: single/dual view, orbit + minimap overlays, and the brand
//! watermark (REFACTOR-PLAN Phase 3, intra-crate UI split). `impl FractadyneApp` blocks moved
//! verbatim from `main.rs`.
use crate::*;

impl FractadyneApp {
    /// Dual linked view: Mandelbrot (left) ↔ Julia (right). Hovering the Mandelbrot
    /// sets the Julia parameter `c`.
    pub(crate) fn draw_dual(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let ppp = ctx.pixels_per_point() as f64;
        let full = ui.max_rect();
        // Split position from the persisted fraction; a small gap between the panels holds the
        // draggable separator (handled at the end of this fn, so it draws on top).
        const HANDLE_W: f32 = 6.0;
        let mid = full.min.x + full.width() * self.dual_split.clamp(0.15, 0.85);
        let left = egui::Rect::from_min_max(full.min, egui::pos2(mid - HANDLE_W * 0.5, full.max.y));
        let right = egui::Rect::from_min_max(egui::pos2(mid + HANDLE_W * 0.5, full.min.y), full.max);
        let scroll = ctx.input(|i| i.smooth_scroll_delta.y) as f64;

        // Continuous zoom (hold Space / Shift+Space) toward the cursor, on the panel
        // it is over. Applied before drawing so this frame reflects it.
        let pointer = ctx.input(|i| i.pointer.hover_pos());
        let (space, shift) = ctx.input(|i| (i.key_down(egui::Key::Space), i.modifiers.shift));
        let dt = (ctx.input(|i| i.stable_dt) as f64).clamp(0.0, 0.1);
        let panel = pointer.and_then(|p| {
            if left.contains(p) {
                Some((p, left, false))
            } else if right.contains(p) {
                Some((p, right, true))
            } else {
                None
            }
        });
        let rate = ZOOM_RATE * self.render_cfg.zoom_rate as f64;
        let target = if space && panel.is_some() {
            if shift {
                -rate
            } else {
                rate
            }
        } else {
            0.0
        };
        let ease = 1.0 - (-dt / EASE_TAU).exp();
        self.pointer.zoom_vel += (target - self.pointer.zoom_vel) * ease;
        if target != 0.0 || self.pointer.zoom_vel.abs() > 1e-3 {
            self.schedule_repaint(ctx);
        }
        if self.pointer.zoom_vel.abs() > 1e-3 {
            if let Some((p, r, is_julia)) = panel {
                let l = p - r.min;
                let factor = (-self.pointer.zoom_vel * dt).exp();
                let vp = if is_julia {
                    &mut self.julia_viewport
                } else {
                    &mut self.viewport
                };
                vp.set_size(r.width() as f64 * ppp, r.height() as f64 * ppp);
                vp.zoom_at(l.x as f64 * ppp, l.y as f64 * ppp, factor);
            }
        }

        // Left: Mandelbrot (sets the Mandelbrot viewport size, renders the parameter
        // plane). Drawn first so the size is current before we read a complex coord.
        let resp_l = self.nav_and_draw(ui, ctx, left, ppp, scroll, false);

        // Click toggles the Julia pin: freeze `c` at the clicked point (and mark it),
        // or release if the click lands on the existing marker → resume live hover.
        if resp_l.clicked() {
            if let Some(pos) = resp_l.interact_pointer_pos() {
                let l = pos - left.min;
                let cc = self
                    .viewport
                    .complex_at_pixel_f64(l.x as f64 * ppp, l.y as f64 * ppp);
                let on_marker = self.julia_pin.is_some_and(|pin| {
                    (pos - self.complex_screen_pos(pin, left, ppp)).length() < 12.0
                });
                if on_marker {
                    self.julia_pin = None; // release → live hover resumes
                } else {
                    self.julia_pin = Some(cc);
                    self.julia_c = cc;
                    self.pointer.settle_t[1] = ctx.input(|i| i.time); // only the Julia view changed
                    self.ref_cache[1].ref_pt = None; // Julia changed
                }
            }
        }

        // Cursor readout + (when not pinned) live Julia `c` from the hovered panel.
        let mut pc = None;
        if let Some((p, r, is_julia)) = panel {
            let l = p - r.min;
            let coord = if is_julia {
                self.julia_viewport
                    .complex_at_pixel_f64(l.x as f64 * ppp, l.y as f64 * ppp)
            } else {
                self.viewport
                    .complex_at_pixel_f64(l.x as f64 * ppp, l.y as f64 * ppp)
            };
            pc = Some(coord);
            if !is_julia && self.julia_pin.is_none() && coord != self.julia_c {
                self.julia_c = coord; // live: cursor over Mandelbrot drives the Julia
                self.pointer.settle_t[1] = ctx.input(|i| i.time); // only the Julia panel is changing
                self.ref_cache[1].ref_pt = None;
                self.schedule_repaint(ctx);
            }
        }

        // Right: Julia for the current c.
        let _ = self.nav_and_draw(ui, ctx, right, ppp, scroll, true);

        // Marker at the pinned point on the Mandelbrot panel.
        if let Some(pin) = self.julia_pin {
            let sp = self.complex_screen_pos(pin, left, ppp);
            if left.contains(sp) {
                let painter = ui.painter_at(left);
                painter.circle_stroke(sp, 6.0, egui::Stroke::new(1.5, egui::Color32::WHITE));
                painter.circle_filled(sp, 1.5, egui::Color32::WHITE);
            }
        }

        // Orbit overlay on the hovered panel — or, during a tour, a scripted point on the Mandelbrot.
        if self.anim.show_orbits {
            if let Some((p, r, is_julia)) = panel {
                let l = p - r.min;
                let vp = if is_julia {
                    &self.julia_viewport
                } else {
                    &self.viewport
                };
                let cpx = (l.x as f64 * ppp, l.y as f64 * ppp);
                let painter = ui.painter_at(r);
                self.draw_orbit(&painter, r, vp, cpx, is_julia, ppp);
            } else if let Some((ox, oy)) = self.anim.tour_orbit {
                let pr = self.viewport.precision;
                let cpx = self.viewport.complex_to_pixel(
                    &fractadyne_core::BigFloat::from_f64(ox, pr),
                    &fractadyne_core::BigFloat::from_f64(oy, pr),
                );
                let painter = ui.painter_at(left);
                self.draw_orbit(&painter, left, &self.viewport, cpx, false, ppp);
            }
        }

        // "Working" spinner in whichever panel's reference orbit is building off-thread (each view
        // recomputes independently — e.g. hovering the Mandelbrot rebuilds only the Julia's).
        let now = ctx.input(|i| i.time);
        let lp = ui.painter_at(left);
        self.draw_recompute_spinner(ctx, &lp, left, 0, now);
        let rp = ui.painter_at(right);
        self.draw_recompute_spinner(ctx, &rp, right, 1, now);

        // Draggable panel separator (fills the reserved gap at `mid`; drawn on top).
        let handle = egui::Rect::from_min_max(
            egui::pos2(mid - HANDLE_W * 0.5, full.min.y),
            egui::pos2(mid + HANDLE_W * 0.5, full.max.y),
        );
        let sep = ui.interact(handle, ui.id().with("dual_split"), egui::Sense::drag());
        if sep.dragged() {
            if let Some(p) = sep.interact_pointer_pos() {
                self.dual_split = ((p.x - full.min.x) / full.width().max(1.0)).clamp(0.15, 0.85);
            }
        }
        if sep.hovered() || sep.dragged() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        }
        let sep_col = if sep.hovered() || sep.dragged() {
            BRAND_ACCENT
        } else {
            egui::Color32::from_gray(50)
        };
        ui.painter().rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(mid - 0.5, full.min.y),
                egui::pos2(mid + 0.5, full.max.y),
            ),
            egui::CornerRadius::ZERO,
            sep_col,
        );

        self.pointer.pointer_complex = pc;
    }

    /// Draw the iteration orbit of `point` (the cursor's complex coordinate) onto
    /// `rect`, using `vp` for the complex→screen mapping. `is_julia` selects the
    /// start/parameter convention (matches the shader): escape-time families orbit
    /// `z₀ = 0, c = point`; Julia/Newton orbit `z₀ = point` with the fixed `c`.
    ///
    /// `cursor_px` is the cursor position in device pixels within the panel. At
    /// shallow/moderate zoom the orbit is iterated in `f64` from the exact cursor
    /// point. Past ~1e12× — where `f64` can't resolve the location — it iterates in
    /// **bignum from the cursor's high-precision coordinate**, so the orbit is the real
    /// orbit *and* still follows the cursor (recomputed each move).
    pub(crate) fn draw_orbit(
        &self,
        painter: &egui::Painter,
        rect: egui::Rect,
        vp: &Viewport,
        cursor_px: (f64, f64),
        is_julia: bool,
        ppp: f64,
    ) {
        let formula = self.fractal.formula_id();
        let mag = vp.magnification();
        let (cx0, cy0) = vp.center_f64();
        let key = OrbitKey {
            px: cursor_px.0,
            py: cursor_px.1,
            cx: cx0,
            cy: cy0,
            upp: vp.units_per_pixel.to_f64(),
            julia: is_julia,
            formula,
            jcx: self.julia_c.0,
            jcy: self.julia_c.1,
        };
        // Reuse the cached orbit unless the cursor or view changed (the bignum orbit is
        // costly, and the app may repaint for unrelated reasons e.g. palette animation).
        let cached = self
            .orbit_cache
            .borrow()
            .as_ref()
            .filter(|e| e.key == key)
            .map(|e| e.pts.clone());
        let pts: Vec<(f64, f64)> = if let Some(p) = cached {
            p
        } else {
            let mut p: Vec<(f64, f64)> = if mag > 1.0e12 && self.fractal.supports_perturbation() {
                // Deep: iterate in bignum from the cursor's arbitrary-precision coord,
                // running out toward escape so the divergent (cursor-sensitive) tail shows.
                let (cbx, cby) = vp.pixel_to_complex(cursor_px.0, cursor_px.1);
                let prec = fractadyne_core::precision_for_magnification(mag);
                let bf = |v: f64| fractadyne_core::BigFloat::from_f64(v, prec);
                let (z0x, z0y, cx, cy) = if is_julia {
                    (cbx, cby, bf(self.julia_c.0), bf(self.julia_c.1))
                } else {
                    (bf(0.0), bf(0.0), cbx, cby)
                };
                let (orbit, len) = fractadyne_core::reference_orbit(
                    &z0x, &z0y, &cx, &cy, formula, ORBIT_MAX_DEEP, prec,
                );
                let n = (len as usize).min(orbit.len());
                orbit[..n]
                    .iter()
                    .map(|s| (s[0] as f64 + s[2] as f64, s[1] as f64 + s[3] as f64))
                    .collect()
            } else {
                let point = vp.complex_at_pixel_f64(cursor_px.0, cursor_px.1);
                let newton = formula == 9;
                let (z0, c) = if is_julia {
                    (point, self.julia_c)
                } else if newton {
                    (point, (0.0, 0.0))
                } else {
                    ((0.0, 0.0), point)
                };
                fractadyne_core::orbit_points(z0, c, formula, ORBIT_MAX, 1.0e8)
            };
            // Trim the final escape to infinity so the normalized fit / racing dot reflect
            // the real |z|≲4 trajectory rather than one huge blown-up iterate.
            if let Some(k) = p.iter().position(|q| q.0 * q.0 + q.1 * q.1 > 16.0) {
                p.truncate(k + 1);
            }
            *self.orbit_cache.borrow_mut() = Some(OrbitCacheEntry {
                key,
                pts: p.clone(),
            });
            p
        };
        if pts.len() < 2 {
            return;
        }
        let screen: Vec<egui::Pos2> = if self.anim.orbit_normalize {
            // Normalized: fit the orbit's bounding box to the whole panel, so it reads
            // well at any zoom (the viewport mapping pushes it off-screen once you're
            // deep, since the orbit spans the whole |z|≲2 region). A faint wash keeps
            // the bright orbit legible while the fractal stays visible behind it.
            let mut min = pts[0];
            let mut max = pts[0];
            for &(x, y) in &pts {
                min.0 = min.0.min(x);
                min.1 = min.1.min(y);
                max.0 = max.0.max(x);
                max.1 = max.1.max(y);
            }
            let bw = (max.0 - min.0).max(1.0e-12);
            let bh = (max.1 - min.1).max(1.0e-12);
            let (bcx, bcy) = ((min.0 + max.0) * 0.5, (min.1 + max.1) * 0.5);
            painter.rect_filled(
                rect,
                egui::CornerRadius::ZERO,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 48),
            );
            painter.text(
                rect.min + egui::vec2(8.0, 6.0),
                egui::Align2::LEFT_TOP,
                "orbit (normalized)",
                egui::FontId::monospace(11.0),
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 140),
            );
            // Fit to the panel with an edge margin, preserving aspect ratio.
            let target = rect.shrink(28.0);
            let scale = (target.width() / bw as f32).min(target.height() / bh as f32);
            let center = rect.center();
            pts.iter()
                .map(|&(x, y)| {
                    egui::pos2(
                        center.x + ((x - bcx) as f32) * scale,
                        center.y - ((y - bcy) as f32) * scale, // flip y (screen +y down)
                    )
                })
                .collect()
        } else {
            let (cx, cy) = vp.center_f64();
            let upp = vp.units_per_pixel.to_f64();
            pts.iter()
                .map(|p| {
                    let px = (p.0 - cx) / upp + vp.width_px * 0.5;
                    let py = vp.height_px * 0.5 - (p.1 - cy) / upp;
                    egui::pos2(rect.min.x + (px / ppp) as f32, rect.min.y + (py / ppp) as f32)
                })
                .collect()
        };
        let n = screen.len();
        // Per-segment polyline: thick & warm near z₀, tapering thinner and shifting
        // hue toward the tail so the direction of iteration reads at a glance.
        let start = egui::Color32::from_rgba_unmultiplied(0xFF, 0xF0, 0x60, 235); // warm yellow
        let end = egui::Color32::from_rgba_unmultiplied(0xFF, 0x30, 0x88, 120); // magenta, faded
        for i in 0..n - 1 {
            let t = i as f32 / (n - 1).max(1) as f32;
            let w = 3.4 * (1.0 - t) + 0.6; // 4.0 → 0.6 px
            painter.line_segment(
                [screen[i], screen[i + 1]],
                egui::Stroke::new(w, lerp_color(start, end, t)),
            );
        }
        painter.circle_filled(screen[0], 3.5, egui::Color32::from_rgb(0x40, 0xE0, 0x60)); // z₀
        painter.circle_filled(screen[n - 1], 3.0, egui::Color32::from_rgb(0xFF, 0x50, 0x40)); // last

        // A dot racing out along the orbit on a loop, color cycling over time.
        if self.anim.orbit_anim && n >= 2 {
            let segs = (n - 1) as f32;
            let phase = self.anim.orbit_phase % segs; // 0..segs, restarts at z₀
            let k = phase.floor() as usize;
            let f = phase - k as f32;
            let k2 = (k + 1).min(n - 1);
            let pos = screen[k] + (screen[k2] - screen[k]) * f;
            let col = egui::Color32::from(egui::ecolor::Hsva::new(self.anim.orbit_hue, 0.85, 1.0, 1.0));
            let [r, g, b, _] = col.to_array();
            painter.circle_filled(pos, 8.0, egui::Color32::from_rgba_unmultiplied(r, g, b, 70));
            painter.circle_filled(pos, 4.0, col);
            painter.circle_stroke(pos, 4.0, egui::Stroke::new(1.0, egui::Color32::WHITE));
        }
    }

    /// Draw the minimap overlay (thumbnail + "you are here" marker + zoom depth), and
    /// handle click-to-jump. Anchored bottom-left, above the status bar.
    /// Draw the discreet "Fd" brand mark in the lower-right of the fractal area (live view). Uses
    /// the header font — F in the light brand text color, d in the amber accent — over a soft dark
    /// halo so it stays legible on any background. Exports rasterize the same mark (`render.rs`).
    pub(crate) fn draw_watermark(&self, ctx: &egui::Context, rect: egui::Rect) {
        let px = (rect.height() * 0.026).clamp(18.0, 34.0); // small (~20–30 px), discreet
        let mark = ctx.fonts(|f| {
            f.layout_job(theme::brand_mark_job(px, theme::BRAND_TEXT, theme::BRAND_ACCENT))
        });
        let halo = ctx.fonts(|f| {
            let c = egui::Color32::from_black_alpha(120);
            f.layout_job(theme::brand_mark_job(px, c, c))
        });
        let sz = mark.size();
        let pos = egui::pos2(rect.right() - sz.x - 12.0, rect.bottom() - sz.y - 10.0);
        let painter =
            ctx.layer_painter(egui::LayerId::new(egui::Order::Middle, egui::Id::new("fd_watermark")));
        // Soft glow: the dark mark stamped at a ring of small offsets, then the colored mark on top.
        for off in [
            egui::vec2(1.2, 0.0), egui::vec2(-1.2, 0.0), egui::vec2(0.0, 1.2), egui::vec2(0.0, -1.2),
            egui::vec2(1.0, 1.0), egui::vec2(-1.0, 1.0), egui::vec2(1.0, -1.0), egui::vec2(-1.0, -1.0),
        ] {
            painter.galley(pos + off, halo.clone(), egui::Color32::PLACEHOLDER);
        }
        painter.galley(pos, mark, egui::Color32::PLACEHOLDER);
    }

    /// Draw a small amber "working" spinner in the top-left of view `vi`'s `rect` while a discrete
    /// jump at deep zoom (goto / bookmark / undo / formula switch / dual-Julia hover) holds a
    /// reprojected placeholder — the v0.1.39 async cold-start window where the off-thread bignum
    /// reference build is in flight and no usable reference sits behind the frame yet — so the wait
    /// reads as "computing" rather than hung. `now` is `ctx.input(|i| i.time)`.
    ///
    /// Kept quiet outside that window (see the gate below) and debounced by `SHOW_DELAY` so a build
    /// quick enough to feel instant never flashes it. Top-left is the one free corner (perf overlay
    /// = top-right, minimap = bottom-left, watermark = bottom-right).
    pub(crate) fn draw_recompute_spinner(
        &mut self,
        ctx: &egui::Context,
        painter: &egui::Painter,
        rect: egui::Rect,
        vi: usize,
        now: f64,
    ) {
        const SHOW_DELAY: f64 = 0.15; // let builds quick enough to feel instant resolve unmarked
        // Show whenever a reference build (bignum orbit + SA + BLA) is in flight for this view — the
        // "working" cue while the deep view resolves, whether that's a cold start, a discrete jump, or
        // a deep pan/zoom refreshing its reference. The SHOW_DELAY below keeps quick (shallow) builds
        // unmarked so it never strobes, and consecutive frames of one build re-use the same start. Tour
        // playback re-invalidates the reference every keyframe, so suppress there rather than strobe.
        let busy = self.recompute_rx[vi].is_some() && self.playback.is_none();
        if busy {
            // A real gap since the last in-flight frame re-arms the delay, so each fresh build must
            // again outlast it (a quick one never shows); consecutive frames of one build do not. The
            // threshold MUST exceed the repaint-while-building interval (main.rs throttles idle
            // recompute repaints to ~50 ms) — otherwise every build frame counts as a gap and the
            // delay never accumulates, so the spinner never appears.
            if now - self.pointer.spin_last[vi] > 0.2 {
                self.pointer.spin_since[vi] = now;
            }
            self.pointer.spin_last[vi] = now;
        }
        // Drawn only once a build has held the placeholder past the delay; clears the instant the
        // reference lands (`busy` false) so the spinner never lingers over the finished frame.
        if !busy || now - self.pointer.spin_since[vi] < SHOW_DELAY {
            return;
        }
        // The event loop already repaints continuously while a recompute is in flight (main.rs), so
        // the arc animates; request one anyway to keep this self-contained.
        ctx.request_repaint();

        // Rotating arc, tail faded so the spin direction reads, over a soft dark disc so it stays
        // legible on any fractal content (bright or dark).
        let c = egui::pos2(rect.left() + 24.0, rect.top() + 24.0);
        let r = 9.0_f32;
        painter.circle_filled(c, r + 5.0, egui::Color32::from_black_alpha(70));
        let n = 18;
        let span = std::f32::consts::TAU * 0.78; // ~280° open arc
        let head = now as f32 * 4.2; // rotation rate (rad/s)
        let at = |a: f32| egui::pos2(c.x + r * a.cos(), c.y + r * a.sin());
        for i in 0..n {
            let t = i as f32 / n as f32;
            let alpha = (40.0 + 205.0 * t) as u8; // faint tail → bright head
            painter.line_segment(
                [at(head + span * t), at(head + span * (i + 1) as f32 / n as f32)],
                egui::Stroke::new(
                    2.4,
                    egui::Color32::from_rgba_unmultiplied(0xE0, 0xA0, 0x30, alpha),
                ),
            );
        }
    }

    pub(crate) fn draw_minimap(&mut self, ctx: &egui::Context) {
        if !self.dialogs.minimap || (self.julia_mode && !self.dual) {
            return;
        }
        let Some(tex) = self.minimap_tex.clone() else { return };
        let disp_w = 196.0_f32;
        let disp_h = disp_w * MINIMAP_TH as f32 / MINIMAP_TW as f32;
        let mut jump: Option<(f64, f64)> = None;
        egui::Area::new(egui::Id::new("fractadyne.minimap"))
            .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(10.0, -34.0))
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .stroke(egui::Stroke::new(1.0, BRAND_ACCENT))
                    .show(ui, |ui| {
                        let (rect, resp) = ui.allocate_exact_size(
                            egui::vec2(disp_w, disp_h),
                            egui::Sense::click(),
                        );
                        let p = ui.painter_at(rect);
                        p.image(
                            tex.id(),
                            rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE,
                        );
                        // "You are here": project the current center onto the thumbnail.
                        // Clamp to the thumbnail so the marker is ALWAYS visible (at the edge
                        // if the view sits outside the minimap's fixed region), and draw it
                        // with a dark halo so it reads on any background.
                        let (cx, cy) = self.viewport.center_f64();
                        let nx = ((cx - MINIMAP_CX) / (2.0 * MINIMAP_HX) + 0.5).clamp(0.0, 1.0) as f32;
                        let ny = (0.5 - (cy - MINIMAP_CY) / (2.0 * MINIMAP_HY)).clamp(0.0, 1.0) as f32;
                        let mk = rect.min + egui::vec2(nx * rect.width(), ny * rect.height());
                        let (sx, _sy) = self.viewport.complex_span();
                        let rw = (sx / (2.0 * MINIMAP_HX)) as f32 * rect.width();
                        let amber = BRAND_ACCENT;
                        let halo = egui::Color32::from_black_alpha(190);
                        if rw >= 6.0 {
                            // Shallow: draw the actual view rectangle (auto-clipped to the map).
                            let rh = rw * rect.height() / rect.width();
                            let vr = egui::Rect::from_center_size(mk, egui::vec2(rw, rh));
                            p.rect_stroke(vr, 0.0, egui::Stroke::new(3.0, halo), egui::StrokeKind::Middle);
                            p.rect_stroke(vr, 0.0, egui::Stroke::new(1.5, amber), egui::StrokeKind::Middle);
                        } else {
                            // Deep: a bright crosshair + centre dot (the view is sub-pixel here).
                            let c = 8.0;
                            let cross = |w: f32, col: egui::Color32, p: &egui::Painter| {
                                p.line_segment([mk - egui::vec2(c, 0.0), mk + egui::vec2(c, 0.0)], egui::Stroke::new(w, col));
                                p.line_segment([mk - egui::vec2(0.0, c), mk + egui::vec2(0.0, c)], egui::Stroke::new(w, col));
                            };
                            cross(3.5, halo, &p);
                            p.circle_stroke(mk, 5.0, egui::Stroke::new(3.5, halo));
                            cross(1.5, amber, &p);
                            p.circle_stroke(mk, 5.0, egui::Stroke::new(1.5, amber));
                            p.circle_filled(mk, 2.0, amber);
                        }
                        // Zoom-depth label.
                        p.text(
                            rect.left_bottom() + egui::vec2(4.0, -3.0),
                            egui::Align2::LEFT_BOTTOM,
                            format!("{}×", fmt_zoom_log2(self.viewport.log2_magnification())),
                            egui::FontId::monospace(11.0),
                            BRAND_TEXT,
                        );
                        if resp.clicked() {
                            if let Some(pos) = resp.interact_pointer_pos() {
                                let fx = ((pos.x - rect.min.x) / rect.width()) as f64;
                                let fy = ((pos.y - rect.min.y) / rect.height()) as f64;
                                let tx = MINIMAP_CX + (fx - 0.5) * 2.0 * MINIMAP_HX;
                                let ty = MINIMAP_CY - (fy - 0.5) * 2.0 * MINIMAP_HY;
                                jump = Some((tx, ty));
                            }
                        }
                        resp.on_hover_text(
                            "Overview / you-are-here. Click to jump to that region (home zoom).",
                        );
                    });
            });
        if let Some((tx, ty)) = jump {
            self.viewport.set_center_mag(
                fractadyne_core::BigFloat::from_f64(tx, 64),
                fractadyne_core::BigFloat::from_f64(ty, 64),
                1.0,
            );
            self.pointer.zoom_vel = 0.0;
            self.invalidate_refs();
            self.record_nav();
        }
    }

    /// Central fractal area: the render callback (or dual view), pan / zoom / click input, the
    /// orbit overlay, plus the watermark, guided-tour annotations, minimap, gradient editor,
    /// and help overlay drawn on top of it.
    pub(crate) fn draw_central(&mut self, ctx: &egui::Context) {
        let central = egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                if self.dual {
                    self.draw_dual(ui, ctx);
                    return;
                }
                let rect = ui.max_rect();
                let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
                let ppp = ctx.pixels_per_point() as f64;

                // 1 pixel = constant complex units; resizing reveals more/less plane. A size change
                // (window maximize / restore / edge-drag) marks the view interacting so resize frames
                // render at the coarse moving quality and settle to full AA once the size holds —
                // otherwise every resize step re-renders at full 8× AA and the resize stutters.
                let (nw, nh) = (rect.width() as f64 * ppp, rect.height() as f64 * ppp);
                if (nw - self.viewport.width_px).abs() > 0.5 || (nh - self.viewport.height_px).abs() > 0.5 {
                    self.pointer.settle_t[0] = ctx.input(|i| i.time);
                }
                self.viewport.set_size(nw, nh);

                // Zoom box (Shift+drag): rubber-band a rectangle, then zoom so it fills the
                // view. Deep-zoom-correct (recenter + scale via the bignum viewport methods).
                let shift = ctx.input(|i| i.modifiers.shift);
                if response.drag_started() && shift {
                    if let Some(p) = response.interact_pointer_pos() {
                        self.pointer.zoom_box = Some(ZoomBox { start: p, end: p, is_julia: false });
                    }
                }
                let mut zoom_boxing = false;
                if self.pointer.zoom_box.as_ref().is_some_and(|z| !z.is_julia) {
                    zoom_boxing = true;
                    if let Some(cur) = response.interact_pointer_pos() {
                        self.pointer.zoom_box.as_mut().unwrap().end = cur;
                    }
                    let zb = self.pointer.zoom_box.as_ref().unwrap();
                    let boxr = aspect_zoom_box(zb.start, zb.end, rect);
                    // Foreground layer so the box draws above the fractal paint callback.
                    let painter = ctx.layer_painter(egui::LayerId::new(
                        egui::Order::Foreground,
                        egui::Id::new("fract_zoom_box"),
                    ));
                    painter.rect_filled(
                        boxr,
                        egui::CornerRadius::ZERO,
                        egui::Color32::from_rgba_unmultiplied(0xE0, 0xA0, 0x30, 32),
                    );
                    painter.rect_stroke(
                        boxr,
                        egui::CornerRadius::ZERO,
                        egui::Stroke::new(1.5, BRAND_ACCENT),
                        egui::StrokeKind::Inside,
                    );
                    if response.drag_stopped() {
                        if boxr.width() > 6.0 && boxr.height() > 6.0 {
                            let bcx = (boxr.center().x - rect.min.x) as f64 * ppp;
                            let bcy = (boxr.center().y - rect.min.y) as f64 * ppp;
                            let factor = (boxr.width() / rect.width()) as f64; // < 1 ⇒ zoom in
                            let (w, h) = (self.viewport.width_px, self.viewport.height_px);
                            self.viewport.pan_pixels(w * 0.5 - bcx, h * 0.5 - bcy);
                            self.viewport.zoom_at(w * 0.5, h * 0.5, factor);
                            self.pointer.settle_t[0] = ctx.input(|i| i.time);
                        }
                        self.pointer.zoom_box = None;
                        zoom_boxing = false;
                    }
                }

                // Track the complex coordinate under the cursor (for the status bar).
                self.pointer.pointer_complex = response.hover_pos().map(|p| {
                    let l = p - rect.min;
                    self.viewport
                        .complex_at_pixel_f64(l.x as f64 * ppp, l.y as f64 * ppp)
                });

                // Pan with left-drag (unless dragging a zoom box).
                if !zoom_boxing && response.dragged_by(egui::PointerButton::Primary) {
                    let d = response.drag_delta();
                    self.viewport.pan_pixels(d.x as f64 * ppp, d.y as f64 * ppp);
                }

                // Cursor-centered wheel zoom (scroll up = zoom in).
                let scroll_y = ctx.input(|i| i.smooth_scroll_delta.y) as f64;
                if scroll_y != 0.0 {
                    if let Some(pos) = response.hover_pos() {
                        let local = pos - rect.min;
                        let factor = (-0.0015 * scroll_y).exp();
                        self.viewport
                            .zoom_at(local.x as f64 * ppp, local.y as f64 * ppp, factor);
                    }
                }

                // Continuous zoom: hold Space (in) / Shift+Space (out), toward the
                // cursor. Exponential rate, eased in/out, frame-rate independent.
                if let Some(pos) = response.hover_pos() {
                    self.pointer.last_cursor = Some(pos);
                }
                let (space, shift) =
                    ctx.input(|i| (i.key_down(egui::Key::Space), i.modifiers.shift));
                let rate = ZOOM_RATE * self.render_cfg.zoom_rate as f64;
                let target_vel = if space {
                    if shift { -rate } else { rate }
                } else {
                    0.0
                };
                let dt = (ctx.input(|i| i.stable_dt) as f64).clamp(0.0, 0.1);
                let ease = 1.0 - (-dt / EASE_TAU).exp();
                self.pointer.zoom_vel += (target_vel - self.pointer.zoom_vel) * ease;
                if target_vel != 0.0 || self.pointer.zoom_vel.abs() > 1e-3 {
                    self.schedule_repaint(ctx); // animate while held and during glide-out
                }
                if self.pointer.zoom_vel.abs() > 1e-3 {
                    let anchor = self.pointer.last_cursor.unwrap_or_else(|| rect.center());
                    let local = anchor - rect.min;
                    let factor = (-self.pointer.zoom_vel * dt).exp(); // vel>0 → factor<1 → zoom in
                    self.viewport
                        .zoom_at(local.x as f64 * ppp, local.y as f64 * ppp, factor);
                }

                // Box-zoom with right-drag: record the start, apply on release.
                if response.drag_started_by(egui::PointerButton::Secondary) {
                    self.pointer.box_start = response.interact_pointer_pos();
                }
                if response.drag_stopped_by(egui::PointerButton::Secondary) {
                    let end = response
                        .interact_pointer_pos()
                        .or_else(|| ctx.input(|i| i.pointer.latest_pos()));
                    if let (Some(start), Some(end)) = (self.pointer.box_start, end) {
                        let s = start - rect.min;
                        let e = end - rect.min;
                        if (e.x - s.x).abs() > 6.0 && (e.y - s.y).abs() > 6.0 {
                            self.viewport.zoom_to_rect(
                                s.x as f64 * ppp,
                                s.y as f64 * ppp,
                                e.x as f64 * ppp,
                                e.y as f64 * ppp,
                            );
                        }
                    }
                    self.pointer.box_start = None;
                }

                // Click-to-zoom tool (toggle, single view): a plain left-click dives in by the
                // configured factor recentered on the point; right-click backs out by the same
                // factor. `clicked()` / `secondary_clicked()` only fire when the press-release
                // wasn't a drag, so this coexists with left-drag pan and Shift/right-drag box-zoom
                // (both drags). Shift is reserved for box-zoom, so a Shift+click never dives. The
                // magnifier cursor advertises the armed tool while hovering.
                if self.click_zoom && !shift {
                    if response.hovered() && !response.dragged() {
                        ctx.set_cursor_icon(egui::CursorIcon::ZoomIn);
                    }
                    if (response.clicked() || response.secondary_clicked())
                        && !zoom_boxing
                    {
                        if let Some(p) = response.interact_pointer_pos() {
                            let l = p - rect.min;
                            let now = ctx.input(|i| i.time);
                            self.click_zoom_at(
                                l.x as f64 * ppp,
                                l.y as f64 * ppp,
                                response.secondary_clicked(),
                                now,
                            );
                        }
                    }
                }

                // Render the fractal at the current viewport with the chosen palette.
                let span_fe = self.viewport.complex_span_fe();
                let eff_iter = if self.render_cfg.auto_iter {
                    // LIVE-preview iteration cap (the cap viewport.rs's `recommended_max_iter`
                    // comment promises but never implemented). At extreme depth
                    // `recommended_max_iter` saturates at 500k, and a live view rendering a
                    // 500k-iter reference — non-escaping at a tip like c=−2, so a full 500k
                    // orbit + a ~4M-node BLA — overloads the GPU present and FREEZES the window
                    // on boot/settle (the ~1e2100× filament-tip session). The one-shot export
                    // path keeps the full appetite (it renders in a single bounded pass and is
                    // fast), so a real export still recovers the ultimate boundary detail; the
                    // live preview trades that for staying responsive at arbitrary depth.
                    self.viewport
                        .recommended_max_iter(self.render_cfg.max_iter)
                        .min(crate::LIVE_ITER_CAP)
                } else {
                    self.render_cfg.max_iter
                };
                // Quality-on-settle: skip AA while interacting (and for a short
                // settle window after), then render full AA once the view is still.
                let now = ctx.input(|i| i.time);
                // Drawing a zoom box (Shift+left-drag → `zoom_boxing`, or right-drag →
                // `box_start`) drags the pointer but does NOT move the view, so it must not
                // count as "active" — otherwise the render drops to the coarse moving preview
                // while you're framing the box. The zoom is applied on release.
                let active = self.pointer.zoom_vel.abs() > 1e-3
                    || (response.dragged() && !zoom_boxing && self.pointer.box_start.is_none())
                    || scroll_y != 0.0
                    || space;
                if active {
                    self.pointer.settle_t[0] = now;
                }
                let interacting = now - self.pointer.settle_t[0] < SETTLE_DELAY;
                // Progressive settle AA (see `nav_and_draw`): refine 1×→2×→4×→… over settled frames.
                let aa_target = if interacting {
                    self.pointer.settle_frame[0] = 0;
                    1
                } else {
                    let ss = aa_ramp(self.pointer.settle_frame[0], self.render_cfg.aa);
                    // Hold the ramp while a tiled settle is mid-grid: advancing ss changes the
                    // iterate key, which would restart the grid every few frames and the view
                    // would never finish sharpening. Resumes the frame after the grid completes.
                    if ss < self.render_cfg.aa && !self.perf.tile_pending[0] {
                        self.pointer.settle_frame[0] += 1;
                        self.schedule_repaint(ctx);
                    }
                    ss
                };

                let center_bf = [self.viewport.center_x.clone(), self.viewport.center_y.clone()];
                let center = self.viewport.center_f64();
                let mag = self.viewport.magnification();
                let log2mag = self.viewport.log2_magnification();
                let resolution = [
                    (rect.width() as f64 * ppp) as u32,
                    (rect.height() as f64 * ppp) as u32,
                ];
                // Pan reprojection: while dragging, translate the last detailed frame instead
                // of re-rendering coarse (see `nav_and_draw` for the full rationale).
                let panning = !zoom_boxing && self.pointer.box_start.is_none();
                if response.drag_started_by(egui::PointerButton::Primary) && panning {
                    self.pointer.pan_px = egui::Vec2::ZERO;
                    self.pointer.pan_view = Some(0);
                }
                if self.pointer.pan_view == Some(0)
                    && panning
                    && response.dragged_by(egui::PointerButton::Primary)
                {
                    let d = response.drag_delta();
                    self.pointer.pan_px += egui::vec2(d.x * ppp as f32, d.y * ppp as f32);
                }
                let reproject = if self.pointer.pan_view == Some(0) && interacting {
                    self.schedule_repaint(ctx);
                    Some([
                        self.pointer.pan_px.x / resolution[0].max(1) as f32,
                        self.pointer.pan_px.y / resolution[1].max(1) as f32,
                    ])
                } else {
                    if self.pointer.pan_view == Some(0) {
                        self.pointer.pan_view = None;
                    }
                    None
                };
                // Only the live view may start a tiled settle (the profiling/benchmark callers of
                // `build_params` time single dispatches).
                self.allow_tiled_settle = true;
                let params = self.build_params(
                    center_bf,
                    center,
                    span_fe,
                    mag,
                    log2mag,
                    self.fractal,
                    self.julia_mode,
                    eff_iter,
                    interacting,
                    aa_target,
                    resolution,
                    0,
                    reproject,
                );
                self.allow_tiled_settle = false;
                // A settle grid in progress needs the next frame promptly — one tile per frame.
                if self.perf.tile_pending[0] {
                    self.schedule_repaint(ctx);
                }
                add_mandelbrot(ui.painter(), rect, params);

                // Orbit overlay for the point under the cursor — or, during a tour, a scripted point.
                if self.anim.show_orbits {
                    let cpx = response
                        .hover_pos()
                        .map(|hp| {
                            let l = hp - rect.min;
                            (l.x as f64 * ppp, l.y as f64 * ppp)
                        })
                        .or_else(|| {
                            self.anim.tour_orbit.map(|(ox, oy)| {
                                let p = self.viewport.precision;
                                self.viewport.complex_to_pixel(
                                    &fractadyne_core::BigFloat::from_f64(ox, p),
                                    &fractadyne_core::BigFloat::from_f64(oy, p),
                                )
                            })
                        });
                    if let Some(cpx) = cpx {
                        let painter = ui.painter_at(rect);
                        self.draw_orbit(&painter, rect, &self.viewport, cpx, self.julia_mode, ppp);
                    }
                }

                // Draw the in-progress box-zoom selection on top of the fractal.
                if let Some(start) = self.pointer.box_start {
                    if response.dragged_by(egui::PointerButton::Secondary) {
                        if let Some(cur) = response.interact_pointer_pos() {
                            let sel = egui::Rect::from_two_pos(start, cur);
                            let painter = ui.painter();
                            painter.rect_filled(
                                sel,
                                egui::CornerRadius::ZERO,
                                egui::Color32::from_rgba_unmultiplied(0xE0, 0xA0, 0x30, 28),
                            );
                            painter.rect_stroke(
                                sel,
                                egui::CornerRadius::ZERO,
                                egui::Stroke::new(1.5, egui::Color32::from_rgb(0xE0, 0xA0, 0x30)),
                                egui::StrokeKind::Inside,
                            );
                        }
                    }
                }

                // "Working" spinner while this view's reference orbit builds off-thread (async
                // cold-start / deep refresh) — the frame is a held placeholder until it lands.
                let sp = ui.painter_at(rect);
                self.draw_recompute_spinner(ctx, &sp, rect, 0, now);
            });

        // ---- brand watermark (lower-right of the fractal area) ----
        if self.watermark {
            self.draw_watermark(ctx, central.response.rect);
        }

        // ---- guided-tour annotations (captions + coordinate-anchored callouts) ----
        if self.playback.is_some() {
            let caption_rects = self.draw_captions(ctx, central.response.rect);
            self.draw_callouts(ctx, central.response.rect, &caption_rects);
        }

        // ---- minimap overview ----
        self.draw_minimap(ctx);

        // ---- gradient editor ----
        self.palette_editor_window(ctx);

        // ---- keyboard / help overlay ----
        self.help_window(ctx);
    }
}
