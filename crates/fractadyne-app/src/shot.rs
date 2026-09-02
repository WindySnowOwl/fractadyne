//! `--shot` — regenerate the site/marketing screenshot from a saved location, unattended.
//!
//! The hero image on fractadyne.org is a picture of the INTERFACE, so it goes stale every time the
//! interface changes — and it did: the one published before 0.2.40 still showed a `beta.104` title
//! bar, the previous icon set and control labels sitting after their controls. A screenshot nobody
//! can regenerate cheaply is a screenshot that will be wrong.
//!
//! ```text
//! fractadyne --shot local/hero.fdn --out hero.png --size 1920x1200
//! ```
//!
//! Loads the location, turns on the things the shot is meant to show (dual view, minimap, control
//! panel), waits for the deep reference orbit to FINISH building rather than for a fixed time, then
//! captures the whole window and exits. The wait matters: the same gate `--uitest` uses, because a
//! timed wait screenshots a half-built preview on a slow machine and a correct frame on a fast one,
//! which is how you end up publishing a black rectangle.
//!
//! Capture is egui's `ViewportCommand::Screenshot` round-trip — the reply lands a frame later as
//! `Event::Screenshot`.

use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Minimum quiet period on the reference-orbit length before a deep view counts as resolved.
const REF_QUIET: Duration = Duration::from_millis(900);
/// Floor on how long to let the picture settle, even when nothing is building. Also covers
/// the frame or two winit takes to apply the requested window size.
const MIN_SETTLE: Duration = Duration::from_millis(2_000);

pub(crate) struct Shot {
    pub(crate) location: PathBuf,
    pub(crate) out: PathBuf,
    pub(crate) size: (f32, f32),
    applied: bool,
    start: Instant,
    /// Hard cap: a wedged build must not hang an unattended run forever.
    deadline: Instant,
    ref_len_seen: u32,
    ref_changed_at: Instant,
    pending: bool,
}

impl Shot {
    pub(crate) fn new(location: PathBuf, out: PathBuf, size: (f32, f32), budget_s: u64) -> Self {
        let now = Instant::now();
        Self {
            location,
            out,
            size,
            applied: false,
            start: now,
            deadline: now + Duration::from_secs(budget_s),
            ref_len_seen: 0,
            ref_changed_at: now,
            pending: false,
        }
    }
}

impl crate::FractadyneApp {
    /// One frame of the `--shot` state machine, driven from `update()`.
    pub(crate) fn shot_frame(&mut self, ctx: &egui::Context) {
        ctx.request_repaint(); // unattended: there is no user input to wake the loop

        // ---- one-time setup -----------------------------------------------------------------
        if !self.harness.shot.as_ref().is_some_and(|s| s.applied) {
            let (loc, size) = {
                let s = self.harness.shot.as_ref().unwrap();
                (s.location.clone(), s.size)
            };
            let text = match std::fs::read_to_string(&loc) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("fractadyne: --shot: cannot read {}: {e}", loc.display());
                    crate::exit(2);
                }
            };
            // Same allow-listed, clamped loader the Open dialog uses — a location file is
            // untrusted input whether a person or a script hands it over.
            let load = self.load_view_metadata(&text);
            if let Some(v) = load.newer {
                eprintln!(
                    "fractadyne: --shot: {} declares format_version {v}, newer than this \
                     build understands; unknown fields were ignored",
                    loc.display()
                );
            }
            // A clamped field means the picture is NOT the one the file describes, which
            // matters when the output is going to be published.
            if !load.clamped.is_empty() {
                eprintln!("fractadyne: --shot: clamped {}", load.clamped.join(", "));
            }
            // What the shot is supposed to show. Set explicitly rather than inherited from a
            // session, so the image does not depend on how the app was last left.
            self.dual = self.fractal.supports_julia();
            self.dialogs.minimap = true;
            self.dialogs.right_panel_open = true;
            self.fullscreen = false;
            self.anim.palette_anim = crate::PaletteAnim::Off; // a moving palette is not reproducible
            self.effects.light_anim = false;
            // `--size` is in PIXELS, which is what someone sizing an image means, but
            // `InnerSize` is in logical POINTS. On this 1.5x display a requested
            // 1920x1200 first came out as a 2880x1800 file — silently scaled, and the
            // sort of wrong that survives review because the picture looks right.
            let ppp = ctx.pixels_per_point().max(0.1);
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                size.0 / ppp,
                size.1 / ppp,
            )));

            let now = Instant::now();
            let s = self.harness.shot.as_mut().unwrap();
            s.applied = true;
            s.start = now;
            s.ref_changed_at = now;
            return;
        }

        // ---- harvest a capture already in flight ---------------------------------------------
        if self.harness.shot.as_ref().is_some_and(|s| s.pending) {
            let image = ctx.input(|i| {
                i.events.iter().find_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
            });
            let Some(image) = image else { return }; // reply lands next frame
            let (w, h) = (image.size[0] as u32, image.size[1] as u32);
            let mut bytes = Vec::with_capacity(image.pixels.len() * 4);
            for px in &image.pixels {
                bytes.extend_from_slice(&[px.r(), px.g(), px.b(), px.a()]);
            }
            let out = self.harness.shot.as_ref().unwrap().out.clone();
            if let Some(dir) = out.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            match fractadyne_export::write_png_rgba8(&out, w, h, &bytes, None) {
                Ok(()) => {
                    println!("--shot: wrote {} ({w}x{h})", out.display());
                    crate::exit(0);
                }
                Err(e) => {
                    eprintln!("fractadyne: --shot: write {}: {e}", out.display());
                    crate::exit(2);
                }
            }
        }

        // ---- wait for the picture to be worth capturing ---------------------------------------
        let now = Instant::now();
        let orbit = self.perf.last_orbit_len;
        let building = self.recompute_rx[0].is_some()
            || self.perf.tile_pending[0]
            || self.perf.chunk_pending[0];
        let s = self.harness.shot.as_mut().unwrap();
        if orbit != s.ref_len_seen {
            s.ref_len_seen = orbit;
            s.ref_changed_at = now;
        }
        let out_of_time = now >= s.deadline;
        let settled = now >= s.start + MIN_SETTLE
            && !building
            && now >= s.ref_changed_at + REF_QUIET;
        if !(settled || out_of_time) {
            return;
        }
        if out_of_time {
            // Say so rather than quietly publishing whatever was on screen.
            eprintln!(
                "fractadyne: --shot: capturing after the {}s budget with work still in flight — \
                 the image may be unresolved",
                s.deadline.saturating_duration_since(s.start).as_secs()
            );
        }
        s.pending = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
    }
}
