//! The "Render script…" dialog: render the tour that is currently loaded to a PNG frame sequence
//! (and optionally an mp4), from the GUI instead of the command line.
//!
//! The render itself runs as a CHILD PROCESS of this executable (`--render-tour`), for three
//! reasons, in order of how much they matter:
//!
//! 1. **It cannot lose the editor's device.** A deep tour render is the single heaviest thing this
//!    app does, and the documented failure mode is a lost GPU device (see the beta.29/36 crash
//!    reports). In a child process that kills the render; in-process it would kill the session.
//! 2. **It reuses the path that is actually tested.** `--render-tour` is what the corpus, the
//!    regression renders and every measurement in TODO.md go through.
//! 3. **The alternative is a refactor.** A tour render mutates app state per frame — viewport,
//!    fractal, iteration budget — so a worker thread would first have to extract all of it
//!    (REFACTOR-PLAN Phase 3's "TourRenderConfig + scripting crate").
//!
//! The cost is honest and small: the child re-reads the script from disk, so unsaved edits in an
//! editor are not rendered, and settings that live in the session rather than the script (e.g. a
//! manually chosen palette) do not carry over. The dialog says so.

use crate::FractadyneApp;

/// A line from the render child, tagged by which stream it came from. Progress lines overwrite the
/// live readout; error lines are kept separately so the failure message survives the progress
/// spam that follows it.
pub(crate) enum RenderLine {
    Progress(String),
    Error(String),
}

/// Parse `frame K/N` out of a child progress line (`"  frame 123/9931  (2m03s elapsed, …)"`) —
/// the pair that drives the dialog's progress bar. Tolerant of leading whitespace; `None` for
/// lines that aren't per-frame progress (the summary, ffmpeg output, resume notes), which simply
/// leave the bar where it was.
pub(crate) fn parse_frame_progress(line: &str) -> Option<(u64, u64)> {
    let rest = line.trim_start().strip_prefix("frame ")?;
    let mut it = rest.split_whitespace().next()?.split('/');
    let done: u64 = it.next()?.parse().ok()?;
    let planned: u64 = it.next()?.parse().ok()?;
    if it.next().is_some() || planned == 0 || done > planned {
        return None;
    }
    Some((done, planned))
}

#[cfg(test)]
mod tests {
    use super::parse_frame_progress;

    #[test]
    fn frame_progress_lines_parse() {
        // The exact shape `render_tour_to_dir` emits.
        assert_eq!(
            parse_frame_progress("  frame 5682/9931  (1h10m02s elapsed, 52m18s left, 1.35 fps)"),
            Some((5682, 9931))
        );
        assert_eq!(parse_frame_progress("frame 1/1"), Some((1, 1)));
        // Non-progress lines leave the bar alone.
        assert_eq!(parse_frame_progress("tour render: 233 frames → frames"), None);
        assert_eq!(parse_frame_progress("Encoding → out.mp4 (ffmpeg)…"), None);
        assert_eq!(parse_frame_progress("frame 12: There is not enough space"), None);
        assert_eq!(parse_frame_progress("frame 0/0"), None);
        assert_eq!(parse_frame_progress("frame 5/3"), None);
    }
}

impl FractadyneApp {
    /// Open the render dialog for the currently loaded script, seeding it from the script's own
    /// `[render]` block so the defaults are what the author intended rather than what this app
    /// happens to have in a field.
    pub(crate) fn open_tour_render(&mut self) {
        let Some(pb) = &self.playback else { return };
        let r = &pb.render;
        let (w, h) = match (r.width, r.height) {
            (Some(w), Some(h)) => (w, h),
            (Some(w), None) => (w, (w * 9 / 16).max(16)),
            _ => (1920, 1080),
        };
        self.tour_render.width = w;
        self.tour_render.height = h;
        // Let the dropdown name the script's size when it is a standard one; it falls back to
        // Custom on its own when no preset matches.
        self.tour_render.custom_size = false;
        self.tour_render.fps = r.fps.unwrap_or(30.0);
        self.tour_render.ss = r.ss.unwrap_or(1);
        self.tour_render.mp4 = r.mp4.is_some();
        self.tour_render.prefix = r.prefix.clone().unwrap_or_else(|| {
            pb.source
                .as_ref()
                .and_then(|p| p.file_stem())
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "frame".to_string())
        });
        if let Some(out) = &r.out {
            self.tour_render.out = out.display().to_string();
        }
        self.tour_render.segment = 0;
        self.tour_render.progress.clear();
        self.tour_render.progress_frames = None;
        self.tour_render.status = None;
        self.tour_render.open = true;
    }

    /// Poll the render child: drain its progress lines and reap it when it exits. Called every
    /// frame from `update` so the dialog stays live even if the user closes and reopens it.
    pub(crate) fn poll_tour_render(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.tour_render.rx {
            while let Ok(line) = rx.try_recv() {
                match line {
                    RenderLine::Progress(l) => {
                        if let Some(fp) = parse_frame_progress(&l) {
                            self.tour_render.progress_frames = Some(fp);
                        }
                        self.tour_render.progress = l;
                    }
                    // Keep the FIRST error: the one that stopped the render. Later lines are
                    // usually consequences of it.
                    RenderLine::Error(l) => {
                        if self.tour_render.error.is_none() {
                            self.tour_render.error = Some(l);
                        }
                    }
                }
            }
        }
        let done = match self.tour_render.child.as_mut() {
            Some(child) => match child.try_wait() {
                Ok(Some(st)) => Some(st.success()),
                Ok(None) => None,
                Err(_) => Some(false),
            },
            None => None,
        };
        if let Some(ok) = done {
            self.tour_render.child = None;
            self.tour_render.rx = None;
            // The finish tone — a tour render is the longest wait in the app.
            if self.render_cfg.finish_sound && self.uitest.is_none() {
                crate::sysinfo::play_finish_sound(false);
            }
            self.tour_render.status = Some(if ok {
                format!("Render finished → {}", self.tour_render.out)
            } else {
                // Quote the child's OWN reason when it gave one — it is the only thing here that
                // says WHY. Falling back to the last progress line (all this used to do) reports
                // where it stopped and leaves the cause in a log file.
                match self.tour_render.error.take() {
                    Some(e) => format!("Render failed: {e}"),
                    None => format!("Render failed or was stopped. {}", self.tour_render.progress),
                }
            });
            ctx.request_repaint();
        }
        if self.tour_render.child.is_some() {
            ctx.request_repaint(); // keep the progress readout moving
        }
    }

    /// The render dialog. Draws nothing unless opened from the playback transport.
    pub(crate) fn draw_tour_render_dialog(&mut self, ctx: &egui::Context) {
        if !self.tour_render.open {
            return;
        }
        // The script being rendered, and its chapters (for the segment picker).
        let Some((script, tour_name, total, chapters)) = self.playback.as_ref().and_then(|pb| {
            pb.source.clone().map(|s| {
                (
                    s,
                    pb.name.clone(),
                    pb.total,
                    pb.segments.iter().map(|g| (g.title.clone(), g.start, g.end)).collect::<Vec<_>>(),
                )
            })
        }) else {
            // Playback stopped (or it is the built-in benchmark, which has no file) — nothing to
            // render, so don't leave a dialog pointing at nothing.
            self.tour_render.open = false;
            return;
        };
        let running = self.tour_render.child.is_some();
        let mut open = self.tour_render.open;
        let (mut go, mut stop, mut browse, mut copy_cmd) = (false, false, false, false);

        // ABOVE the tour overlays. Captions and callouts paint into `Order::Middle` layers, which
        // is also where a Window lives, and they are painted after it — so during playback the
        // caption drew straight over this dialog. `Order::Foreground` puts the dialog where a
        // dialog belongs: above the content it is about. (The playback transport is already
        // Foreground for the same reason.)
        egui::Window::new("Render tour")
            .open(&mut open)
            .resizable(false)
            .default_width(460.0)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new(format!("{tour_name}  ·  {}", script.display()))
                        .weak()
                        .small(),
                );
                ui.add_space(4.0);
                ui.add_enabled_ui(!running, |ui| {
                    egui::Grid::new("tour_render_grid").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                        ui.label("Output folder");
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.tour_render.out)
                                        .desired_width(250.0),
                                );
                                if ui.button("Browse…").clicked() {
                                    browse = true;
                                }
                            });
                            // Show where this ACTUALLY lands. The default is the relative "frames",
                            // which resolves against the app's working directory — not the script's
                            // folder, and not anywhere the user necessarily expects. A render that
                            // takes hours should not leave you hunting for its output.
                            let abs = std::path::Path::new(&self.tour_render.out);
                            let abs = if abs.is_absolute() {
                                abs.to_path_buf()
                            } else {
                                std::env::current_dir().unwrap_or_default().join(abs)
                            };
                            ui.label(
                                egui::RichText::new(abs.display().to_string()).weak().small(),
                            )
                            .on_hover_text("Full path the frames are written to.");
                        });
                        ui.end_row();

                        ui.label("Frame prefix");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.tour_render.prefix)
                                .desired_width(160.0),
                        )
                        .on_hover_text("Frames are written <prefix>_00000.png");
                        ui.end_row();

                        ui.label("Size");
                        ui.vertical(|ui| {
                            // Same preset table as the image export dialog (`STANDARD_SIZES`).
                            // This dialog stores width AND height, so a preset sets them directly
                            // — no aspect round-trip to go wrong.
                            let (w, h) = (self.tour_render.width, self.tour_render.height);
                            let cur = crate::STANDARD_SIZES
                                .iter()
                                .find(|(_, pw, ph)| *pw == w && *ph == h)
                                .map(|(l, _, _)| *l);
                            let custom = self.tour_render.custom_size || cur.is_none();
                            egui::ComboBox::from_id_salt("tour_render_size")
                                .width(210.0)
                                .height(460.0) // see the export dialog: fit every preset
                                .selected_text(if custom {
                                    format!("Custom — {w}×{h}")
                                } else {
                                    cur.unwrap_or_default().to_string()
                                })
                                .show_ui(ui, |ui| {
                                    // Custom first — see the export dialog.
                                    if ui.selectable_label(custom, "Custom…").clicked() {
                                        self.tour_render.custom_size = true;
                                    }
                                    ui.separator();
                                    for (label, pw, ph) in crate::STANDARD_SIZES {
                                        let on = !custom && cur == Some(*label);
                                        if ui.selectable_label(on, *label).clicked() {
                                            self.tour_render.width = *pw;
                                            self.tour_render.height = *ph;
                                            self.tour_render.custom_size = false;
                                        }
                                    }
                                });
                            if custom {
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::DragValue::new(&mut self.tour_render.width)
                                            .range(16..=16384),
                                    );
                                    ui.label("×");
                                    ui.add(
                                        egui::DragValue::new(&mut self.tour_render.height)
                                            .range(16..=16384),
                                    );
                                });
                            }
                        });
                        ui.end_row();

                        ui.label("Frames / second");
                        ui.add(
                            egui::DragValue::new(&mut self.tour_render.fps)
                                .range(0.01..=240.0)
                                .speed(0.25),
                        )
                        .on_hover_text(
                            "Sub-1 fps is legitimate: sampling a long dive at 0.25 fps is how the \
                             deep regression holds get inspected without rendering thousands of frames.",
                        );
                        ui.end_row();

                        ui.label("Supersampling");
                        egui::ComboBox::from_id_salt("tour_render_ss")
                            .selected_text(format!("{}×", self.tour_render.ss))
                            .width(70.0)
                            .show_ui(ui, |ui| {
                                for n in [1u32, 2, 3, 4] {
                                    ui.selectable_value(&mut self.tour_render.ss, n, format!("{n}×"));
                                }
                            });
                        ui.end_row();

                        ui.label("Chapter");
                        egui::ComboBox::from_id_salt("tour_render_segment")
                            .selected_text(match self.tour_render.segment {
                                0 => "Whole tour".to_string(),
                                i => chapters.get(i - 1).map(|c| c.0.clone()).unwrap_or_default(),
                            })
                            .width(240.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.tour_render.segment, 0, "Whole tour");
                                for (i, (title, start, end)) in chapters.iter().enumerate() {
                                    ui.selectable_value(
                                        &mut self.tour_render.segment,
                                        i + 1,
                                        format!("{title}  ({start:.0}–{end:.0}s)"),
                                    );
                                }
                            })
                            .response
                            .on_hover_text(
                                "Rendering one chapter keeps the GLOBAL frame numbering, so its \
                                 frames drop straight back into a full sequence.",
                            );
                        ui.end_row();
                    });
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut self.tour_render.mp4, "Assemble mp4")
                            .on_hover_text("Runs ffmpeg afterwards (must be on PATH). The PNG frames are kept.");
                        ui.checkbox(&mut self.tour_render.overwrite, "Overwrite existing frames")
                            .on_hover_text("Off: the render stops to ask, which it cannot do from here.");
                    });
                    ui.checkbox(&mut self.tour_render.resume, "Resume an interrupted render")
                        .on_hover_text(
                            "Keep the frames already in the output folder and render only the \
                             missing ones. The newest frame is checked first and discarded if it \
                             is incomplete (the frame a render dies on is the one it was writing), \
                             stepping back until a good one is found; a folder holding frames at a \
                             different size is refused rather than mixed into this sequence.",
                        );
                    ui.horizontal(|ui| {
                        ui.label("Render order");
                        egui::ComboBox::from_id_salt("tour_render_order")
                            .selected_text(if self.tour_render.progressive {
                                "Progressive"
                            } else {
                                "Sequential"
                            })
                            .width(140.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.tour_render.progressive,
                                    false,
                                    "Sequential",
                                )
                                .on_hover_text("Frame 0, 1, 2, … in order (the default).");
                                ui.selectable_value(
                                    &mut self.tour_render.progressive,
                                    true,
                                    "Progressive",
                                )
                                .on_hover_text(
                                    "Preview the whole tour first, refine to full detail: the \
                                     keyframes render first, then the largest temporal gaps are \
                                     bisected until every frame exists. Frames land at their \
                                     correct indices either way, so Resume and the mp4 work \
                                     unchanged — you can stop early, eyeball the coarse \
                                     flip-book, and restart with Resume in either order.",
                                );
                            });
                    });
                });

                // What this is about to cost, before committing to it.
                let (t_from, t_to) = match self.tour_render.segment {
                    0 => (0.0, total),
                    i => chapters.get(i - 1).map(|c| (c.1, c.2)).unwrap_or((0.0, total)),
                };
                let frames = (((t_to - t_from) * self.tour_render.fps).round() as i64 + 1).max(1);
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(format!(
                        "{frames} frames · {:.0}s of tour · {}×{} ss{}",
                        t_to - t_from,
                        self.tour_render.width,
                        self.tour_render.height,
                        self.tour_render.ss
                    ))
                    .monospace(),
                );
                // DISK. A frame sequence is the one output here big enough to run a volume dry,
                // and it does so an hour and a half in, after the frames that DID fit were
                // written: measured at 4K, 5.28 MB/frame × 9931 frames ≈ 51 GB against 6 GB free,
                // failing at frame 1091. Nothing in this dialog hinted at that before starting.
                //
                // ~0.65 B/px is the measured PNG cost of a 4K fractal frame; it is an ESTIMATE and
                // labelled as one — deep frames with fine structure compress worse than shallow
                // ones, so treat it as the right order of magnitude, not a promise.
                let est_bytes = (self.tour_render.width as u64)
                    .saturating_mul(self.tour_render.height as u64)
                    .saturating_mul(frames as u64)
                    * 65
                    / 100;
                let gb = |b: u64| b as f64 / (1u64 << 30) as f64;
                let free = crate::sysinfo::free_disk_bytes(std::path::Path::new(&self.tour_render.out));
                let disk = match free {
                    Some(f) => format!("≈{:.1} GB of frames · {:.1} GB free", gb(est_bytes), gb(f)),
                    None => format!("≈{:.1} GB of frames", gb(est_bytes)),
                };
                // Warn on a genuine shortfall only. The estimate is rough, so a 1.2× margin keeps
                // this quiet when it is merely close and loud when the render cannot finish.
                let short = free.is_some_and(|f| (f as f64) < gb(est_bytes) * 1.2 * (1u64 << 30) as f64);
                ui.label(if short {
                    egui::RichText::new(format!("⚠ {disk} — not enough room for the whole sequence"))
                        .monospace()
                        .color(egui::Color32::from_rgb(0xE0, 0xA0, 0x30))
                } else {
                    egui::RichText::new(disk).monospace().weak()
                });
                ui.label(
                    egui::RichText::new(
                        "Renders the script FROM DISK in a separate process, so unsaved edits \
                         aren't included — and a lost GPU device there can't take this session \
                         with it.",
                    )
                    .weak()
                    .small(),
                );

                if running || !self.tour_render.progress.is_empty() {
                    ui.add_space(4.0);
                    ui.separator();
                    // The BAR carries the fraction; the raw child line underneath carries the
                    // elapsed/remaining/fps detail the bar can't. A line that isn't per-frame
                    // progress (resume notes, the ffmpeg step) leaves the bar at its last value
                    // rather than blanking a 70-minute render's only visual anchor.
                    if let Some((done, planned)) = self.tour_render.progress_frames {
                        ui.add(
                            egui::ProgressBar::new((done as f32 / planned as f32).clamp(0.0, 1.0))
                                .desired_height(14.0)
                                .text(
                                    egui::RichText::new(format!("{done} / {planned} frames"))
                                        .monospace()
                                        .small(),
                                ),
                        );
                    }
                    ui.horizontal(|ui| {
                        if running {
                            ui.add(egui::Spinner::new().size(12.0));
                        }
                        ui.label(
                            egui::RichText::new(if self.tour_render.progress.is_empty() {
                                "starting…".to_string()
                            } else {
                                self.tour_render.progress.clone()
                            })
                            .monospace()
                            .small(),
                        );
                    });
                }
                // Say that the held view is deliberate. Without this the live panel stops updating
                // the moment a render starts and reads as the app having frozen — the same
                // ambiguity that makes an unexplained black screen a support ticket.
                if running {
                    ui.label(
                        egui::RichText::new(
                            "Live view paused — the render has the GPU to itself, so it \
                             finishes sooner and measures its own frame costs accurately.",
                        )
                        .small()
                        .weak(),
                    );
                }
                if let Some(st) = &self.tour_render.status {
                    ui.colored_label(crate::theme::BRAND_ACCENT, egui::RichText::new(st).small());
                }

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if running {
                        if ui.button("Stop render").clicked() {
                            stop = true;
                        }
                    } else if ui
                        .button("Render")
                        .on_hover_text("Start rendering the frame sequence")
                        .clicked()
                    {
                        go = true;
                    }
                    if ui
                        .button("Copy command")
                        .on_hover_text("Copy the equivalent command line to the clipboard")
                        .clicked()
                    {
                        copy_cmd = true;
                    }
                });
            });
        self.tour_render.open = open;

        if browse {
            // Seed at the current output path if it exists, else the shared last-used directory.
            let seed = Some(std::path::PathBuf::from(&self.tour_render.out))
                .filter(|p| p.is_dir())
                .unwrap_or_else(|| self.dialog_dir_default());
            if let Some(dir) = rfd::FileDialog::new().set_directory(seed).pick_folder() {
                self.remember_dir(&dir);
                self.tour_render.out = dir.display().to_string();
            }
        }
        if copy_cmd {
            let args = self.tour_render_args(&script, &chapters);
            ctx.copy_text(format!("fractadyne {}", args.join(" ")));
        }
        if stop {
            if let Some(child) = self.tour_render.child.as_mut() {
                let _ = child.kill();
            }
        }
        if go {
            self.start_tour_render(&script, &chapters);
        }
    }

    /// The `--render-tour` argument list this dialog describes — shared by "Render" and
    /// "Copy command" so the copied line is exactly what runs, not an approximation of it.
    fn tour_render_args(
        &self,
        script: &std::path::Path,
        chapters: &[(String, f64, f64)],
    ) -> Vec<String> {
        let t = &self.tour_render;
        let q = |s: String| if s.contains(' ') { format!("\"{s}\"") } else { s };
        let mut a = vec![
            "--render-tour".to_string(),
            q(script.display().to_string()),
            "--out".to_string(),
            q(t.out.clone()),
            "--size".to_string(),
            format!("{}x{}", t.width, t.height),
            "--fps".to_string(),
            format!("{}", t.fps),
            "--ss".to_string(),
            t.ss.to_string(),
            "--prefix".to_string(),
            q(t.prefix.clone()),
        ];
        if t.segment > 0 {
            if let Some((title, _, _)) = chapters.get(t.segment - 1) {
                a.push("--segment".to_string());
                a.push(q(title.clone()));
            }
        }
        if t.mp4 {
            a.push("--mp4".to_string());
        }
        if t.progressive {
            a.push("--order".to_string());
            a.push("progressive".to_string());
        }
        if t.resume {
            a.push("--resume".to_string());
        }
        if t.overwrite {
            a.push("-y".to_string());
        }
        a
    }

    /// Launch the render child and start streaming its progress.
    fn start_tour_render(&mut self, script: &std::path::Path, chapters: &[(String, f64, f64)]) {
        use std::io::{BufRead, BufReader};
        let exe = match std::env::current_exe() {
            Ok(e) => e,
            Err(e) => {
                self.tour_render.status = Some(format!("Cannot find the executable: {e}"));
                return;
            }
        };
        let args = self.tour_render_args(script, chapters);
        self.tour_render.progress.clear();
        self.tour_render.progress_frames = None;
        self.tour_render.status = None;
        // A fresh run must not inherit the previous run's failure reason — `poll_tour_render`
        // quotes the FIRST error it has, which would otherwise be last run's.
        self.tour_render.error = None;
        // STDERR IS PIPED, not discarded. The child reports why it failed through `diag::log_line`,
        // which writes to stderr — so throwing stderr away meant the dialog could only ever repeat
        // the last progress line. Measured: a 4K render died at frame 1091 with the child saying
        // `tour FAILED: frame 1091: There is not enough space on the disk. (os error 112)`, and the
        // dialog reported "Render failed or was stopped. frame 1091/9931 …" — the one fact that
        // mattered was in the log file and nowhere the user would look.
        let child = std::process::Command::new(exe)
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();
        match child {
            Ok(mut c) => {
                let (tx, rx) = std::sync::mpsc::channel();
                self.tour_render.rx = Some(rx);
                if let Some(out) = c.stdout.take() {
                    let tx = tx.clone();
                    std::thread::spawn(move || {
                        for line in BufReader::new(out).lines().map_while(Result::ok) {
                            // Only the lines worth showing: per-frame progress and the summary.
                            let l = line.trim().to_string();
                            if !l.is_empty() && tx.send(RenderLine::Progress(l)).is_err() {
                                break; // the app dropped the receiver
                            }
                        }
                    });
                }
                if let Some(err) = c.stderr.take() {
                    std::thread::spawn(move || {
                        for line in BufReader::new(err).lines().map_while(Result::ok) {
                            // Keep only what explains a failure. The child's stderr also carries
                            // routine diagnostics (`[fd-start]`, trace categories), and promoting
                            // those to the dialog would bury the one line that matters.
                            let l = line.trim().to_string();
                            let interesting = l.contains("FAILED")
                                || l.contains("panic")
                                || l.contains("error")
                                || l.contains("Error");
                            if interesting && tx.send(RenderLine::Error(l)).is_err() {
                                break;
                            }
                        }
                    });
                }
                crate::diag::breadcrumb(format!("GUI tour render → {}", self.tour_render.out));
                self.tour_render.child = Some(c);
            }
            Err(e) => self.tour_render.status = Some(format!("Could not start the render: {e}")),
        }
    }
}
