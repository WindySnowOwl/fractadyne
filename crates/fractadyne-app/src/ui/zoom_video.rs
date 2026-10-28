//! The "Render zoom video…" dialog (`design/zoom-video.md` §6.5, phase E3): a dive into the current
//! view as a video, its strips rendered and assembled in one run (`--render-zoom-video`).
//!
//! Like "Render tour…" (`tour_render.rs`), the render runs as a CHILD PROCESS of this executable,
//! for the same reasons: a lost GPU device there ends the render, not this session, and the command
//! line is the path the tests and measurements go through. The view goes on the child's command
//! line exactly — the centre as the text the viewport holds, the depth as `log2` magnification —
//! and its colouring comes from the SESSION, which is therefore saved just before the child starts:
//! the video is coloured as the picture on screen is.

use crate::ui::tour_render::{parse_frame_progress, RenderLine};
use crate::FractadyneApp;

/// The dialog's state.
pub(crate) struct ZoomVideoUi {
    pub(crate) open: bool,
    /// The view the video dives into, captured when the dialog opens: its centre, exactly (as the
    /// viewport's text), its depth (`log2` magnification), and the fractal's name.
    pub(crate) centre: (String, String),
    pub(crate) end_log2: f64,
    pub(crate) fractal: String,
    /// Why this view cannot become a zoom video (Julia sets, fractals the strips do not render, a
    /// colouring a strip cannot carry), or `None` when it can.
    pub(crate) unsupported: Option<String>,
    /// The video file.
    pub(crate) out: String,
    pub(crate) width: u32,
    pub(crate) height: u32,
    /// Size dropdown is on "Custom…" (sticky, as the export dialog's).
    pub(crate) custom_size: bool,
    pub(crate) fps: f64,
    /// Octaves of zoom a second.
    pub(crate) speed: f64,
    /// Where the dive starts (`log2` magnification; 0 = 1×).
    pub(crate) start_log2: f64,
    /// Turns an octave (0 = none).
    pub(crate) rotate: f64,
    /// Samples a pixel along each axis; the strips are oversampled as much.
    pub(crate) ss: u32,
    /// The palette follows the escape range (`--normalize`).
    pub(crate) normalize: bool,
    /// Keep the strips (`--keep-strips`), in `keep_dir`, to recolour later.
    pub(crate) keep: bool,
    pub(crate) keep_dir: String,
    /// The child's latest progress line and its `frame K/N`.
    pub(crate) progress: String,
    pub(crate) progress_frames: Option<(u64, u64)>,
    /// The first error line the child wrote: why a render stopped.
    pub(crate) error: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) child: Option<std::process::Child>,
    pub(crate) rx: Option<std::sync::mpsc::Receiver<RenderLine>>,
}

impl Default for ZoomVideoUi {
    fn default() -> Self {
        Self {
            open: false,
            centre: (String::new(), String::new()),
            end_log2: 0.0,
            fractal: String::new(),
            unsupported: None,
            out: String::new(),
            width: 1920,
            height: 1080,
            custom_size: false,
            fps: 60.0,
            speed: 1.0,
            start_log2: 0.0,
            rotate: 0.0,
            ss: 1,
            normalize: false,
            keep: false,
            keep_dir: String::new(),
            progress: String::new(),
            progress_frames: None,
            error: None,
            status: None,
            child: None,
            rx: None,
        }
    }
}

/// The video this dialog describes, in numbers: `(frames, seconds, strips, strip width, strip rows,
/// bytes of GPU memory the strip window takes)`. The same geometry `--render-zoom-video` will use.
pub(crate) fn zoom_video_plan(z: &ZoomVideoUi) -> (u64, f64, u64, u32, u32, u64) {
    let map = fractadyne_core::ExpMap::for_frames(z.width as f64, z.height as f64, z.ss as f64, z.start_log2);
    let strips = map.strips_to(z.end_log2, z.height as f64);
    let seconds = (z.end_log2 - z.start_log2).max(0.0) / z.speed.max(1e-9);
    let frames = crate::assemble::dive_frames(z.start_log2, z.end_log2.max(z.start_log2 + 1e-9), z.speed.max(1e-9), z.fps).len() as u64;
    let layers = fractadyne_gpu::layers_for_frame(z.width * z.ss, z.height * z.ss) as u64;
    let gpu = layers * map.width as u64 * map.rows as u64 * 16;
    (frames, seconds, strips, map.width, map.rows, gpu)
}

/// The `--render-zoom-video` argument list this dialog describes, shared by "Render" and "Copy
/// command" so the copied line is exactly what runs.
pub(crate) fn zoom_video_args(z: &ZoomVideoUi) -> Vec<String> {
    let mut a = vec![
        "--render-zoom-video".to_string(),
        z.out.clone(),
        "--center".to_string(),
        z.centre.0.clone(),
        z.centre.1.clone(),
        "--zoom-log2".to_string(),
        format!("{}", z.end_log2),
        "--size".to_string(),
        format!("{}x{}", z.width, z.height),
        "--assemble-fps".to_string(),
        format!("{}", z.fps),
        "--assemble-speed".to_string(),
        format!("{}", z.speed),
    ];
    if z.start_log2 != 0.0 {
        a.extend(["--expmap-start".to_string(), format!("{}", z.start_log2)]);
    }
    if z.rotate != 0.0 {
        a.extend(["--assemble-rotate".to_string(), format!("{}", z.rotate)]);
    }
    if z.ss > 1 {
        a.extend(["--expmap-oversample".to_string(), z.ss.to_string(), "--assemble-ss".to_string(), z.ss.to_string()]);
    }
    if z.normalize {
        a.push("--normalize".to_string());
    }
    if z.keep {
        a.extend(["--keep-strips".to_string(), z.keep_dir.clone()]);
    }
    a
}

impl FractadyneApp {
    /// Open the dialog for the current view (or show the render in progress).
    pub(crate) fn open_zoom_video(&mut self) {
        if self.zoom_video.child.is_some() {
            self.zoom_video.open = true;
            return;
        }
        use crate::fractal::FractalKind as F;
        let dir = self.dialog_dir_default();
        let z = &mut self.zoom_video;
        z.centre = (
            fractadyne_core::to_decimal_string(&self.viewport.center_x),
            fractadyne_core::to_decimal_string(&self.viewport.center_y),
        );
        z.end_log2 = self.viewport.log2_magnification();
        z.fractal = self.fractal.name().to_string();
        z.unsupported = if self.julia_mode {
            Some(tr!(zoom_video_no_julia))
        } else if !matches!(
            self.fractal,
            F::Mandelbrot | F::Multibrot3 | F::Multibrot4 | F::Multibrot5 | F::Multibrot6 | F::Multibrot7 | F::Multibrot8
        ) {
            Some(tr!(zoom_video_no_fractal, fractal = self.fractal.name()))
        } else if self.coloring.color_method.needs_aux() {
            Some(tr!(zoom_video_no_method))
        } else if z.end_log2 < 1.0 {
            Some(tr!(zoom_video_too_shallow))
        } else {
            None
        };
        // The session's own normalization is honoured anyway; the box adds it when it is off.
        z.normalize = self.coloring.normalize_live;
        if z.out.is_empty() {
            z.out = dir.join("zoom-video.mp4").display().to_string();
        }
        if z.keep_dir.is_empty() {
            z.keep_dir = dir.join("zoom-video-strips").display().to_string();
        }
        z.start_log2 = z.start_log2.clamp(0.0, (z.end_log2 - 1.0).max(0.0));
        z.progress.clear();
        z.progress_frames = None;
        z.status = None;
        z.open = true;
    }

    /// Drain the child's lines and reap it when it exits; every frame, so the dialog stays live
    /// when it is closed and reopened.
    pub(crate) fn poll_zoom_video(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.zoom_video.rx {
            while let Ok(line) = rx.try_recv() {
                match line {
                    RenderLine::Progress(l) => {
                        if let Some(fp) = parse_frame_progress(&l) {
                            self.zoom_video.progress_frames = Some(fp);
                        }
                        self.zoom_video.progress = l;
                    }
                    RenderLine::Error(l) => {
                        if self.zoom_video.error.is_none() {
                            self.zoom_video.error = Some(l);
                        }
                    }
                }
            }
        }
        let done = match self.zoom_video.child.as_mut() {
            Some(child) => match child.try_wait() {
                Ok(Some(st)) => Some(st.success()),
                Ok(None) => None,
                Err(_) => Some(false),
            },
            None => None,
        };
        if let Some(ok) = done {
            self.zoom_video.child = None;
            self.zoom_video.rx = None;
            if self.render_cfg.finish_sound && self.harness.uitest.is_none() {
                crate::tone::play_finish_sound(false);
            }
            self.zoom_video.status = Some(if ok {
                tr!(zoom_video_finished, file = &self.zoom_video.out)
            } else {
                match self.zoom_video.error.take() {
                    Some(e) => tr!(zoom_video_failed, why = e),
                    None => tr!(zoom_video_failed_or_stopped, last = &self.zoom_video.progress),
                }
            });
            ctx.request_repaint();
        }
        if self.zoom_video.child.is_some() {
            ctx.request_repaint();
        }
    }

    /// The dialog. Draws nothing unless opened (Tools ▸ Render zoom video…).
    pub(crate) fn draw_zoom_video_dialog(&mut self, ctx: &egui::Context) {
        if !self.zoom_video.open {
            return;
        }
        let running = self.zoom_video.child.is_some();
        let mut open = self.zoom_video.open;
        let (mut go, mut stop, mut browse, mut browse_keep, mut copy_cmd, mut close_dialog) = (false, false, false, false, false, false);
        egui::Window::new(tr!(zoom_video_title))
            .id(egui::Id::new("fractadyne.zoom-video"))
            .open(&mut open)
            .resizable(false)
            .default_width(470.0)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let z = &mut self.zoom_video;
                ui.label(
                    egui::RichText::new(tr!(
                        zoom_video_view,
                        fractal = &z.fractal,
                        magnification = format!("{}×", crate::fmt_zoom_log2(z.end_log2)),
                        octaves = crate::i18n::decimal(z.end_log2, 1)
                    ))
                    .weak()
                    .small(),
                );
                if let Some(why) = &z.unsupported {
                    ui.colored_label(egui::Color32::from_rgb(0xE0, 0xA0, 0x30), egui::RichText::new(format!("⚠ {why}")).small());
                }
                ui.add_space(4.0);
                ui.add_enabled_ui(!running, |ui| {
                    egui::Grid::new("zoom_video_grid").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                        ui.label(tr!(zoom_video_out));
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut z.out).desired_width(260.0));
                            if ui.button(tr!(zoom_video_browse)).clicked() {
                                browse = true;
                            }
                        });
                        ui.end_row();

                        ui.label(tr!(zoom_video_size));
                        ui.vertical(|ui| {
                            let (w, h) = (z.width, z.height);
                            let cur = crate::STANDARD_SIZES.iter().find(|(_, pw, ph)| *pw == w && *ph == h).map(|(l, _, _)| *l);
                            let custom = z.custom_size || cur.is_none();
                            egui::ComboBox::from_id_salt("zoom_video_size")
                                .width(210.0)
                                .height(460.0)
                                .selected_text(if custom {
                                    tr!(zoom_video_size_custom_value, width = w.to_string(), height = h.to_string())
                                } else {
                                    crate::size_preset_label(cur.unwrap_or_default())
                                })
                                .show_ui(ui, |ui| {
                                    if ui.selectable_label(custom, tr!(zoom_video_size_custom)).clicked() {
                                        z.custom_size = true;
                                    }
                                    ui.separator();
                                    for (label, pw, ph) in crate::STANDARD_SIZES {
                                        let on = !custom && cur == Some(*label);
                                        if ui.selectable_label(on, crate::size_preset_label(label)).clicked() {
                                            z.width = *pw;
                                            z.height = *ph;
                                            z.custom_size = false;
                                        }
                                    }
                                });
                            if custom {
                                ui.horizontal(|ui| {
                                    ui.add(egui::DragValue::new(&mut z.width).range(16..=7680));
                                    ui.label("×");
                                    ui.add(egui::DragValue::new(&mut z.height).range(16..=4320));
                                });
                            }
                        });
                        ui.end_row();

                        ui.label(tr!(zoom_video_fps));
                        ui.add(egui::DragValue::new(&mut z.fps).range(1.0..=240.0).speed(0.5));
                        ui.end_row();

                        ui.label(tr!(zoom_video_speed));
                        ui.add(egui::DragValue::new(&mut z.speed).range(0.05..=20.0).speed(0.01).fixed_decimals(2))
                            .on_hover_text(tr!(zoom_video_speed_hover));
                        ui.end_row();

                        ui.label(tr!(zoom_video_start));
                        ui.horizontal(|ui| {
                            let top = (z.end_log2 - 1.0).max(0.0);
                            ui.add(egui::DragValue::new(&mut z.start_log2).range(0.0..=top).speed(0.1).fixed_decimals(1).prefix("2^"));
                            ui.label(egui::RichText::new(format!("{}×", crate::fmt_zoom_log2(z.start_log2))).weak().small());
                        })
                        .response
                        .on_hover_text(tr!(zoom_video_start_hover));
                        ui.end_row();

                        ui.label(tr!(zoom_video_rotate));
                        ui.add(egui::DragValue::new(&mut z.rotate).range(-4.0..=4.0).speed(0.01).fixed_decimals(2))
                            .on_hover_text(tr!(zoom_video_rotate_hover));
                        ui.end_row();

                        ui.label(tr!(zoom_video_ss));
                        egui::ComboBox::from_id_salt("zoom_video_ss")
                            .selected_text(format!("{}×", z.ss))
                            .width(70.0)
                            .show_ui(ui, |ui| {
                                for n in [1u32, 2, 3] {
                                    ui.selectable_value(&mut z.ss, n, format!("{n}×"));
                                }
                            })
                            .response
                            .on_hover_text(tr!(zoom_video_ss_hover));
                        ui.end_row();
                    });
                    ui.checkbox(&mut z.normalize, tr!(zoom_video_normalize)).on_hover_text(tr!(zoom_video_normalize_hover));
                    ui.checkbox(&mut z.keep, tr!(zoom_video_keep)).on_hover_text(tr!(zoom_video_keep_hover));
                    // The folder on a line of its own, as wide as the video file's: a path is the
                    // one thing here too long for a field beside a checkbox.
                    ui.add_enabled_ui(z.keep, |ui| {
                        ui.indent("zoom_video_keep_dir", |ui| {
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut z.keep_dir).desired_width(330.0));
                                if ui.button(tr!(zoom_video_browse)).clicked() {
                                    browse_keep = true;
                                }
                            });
                        });
                    });
                });

                // What this costs, before committing to it.
                let (frames, seconds, strips, sw, sh, gpu) = zoom_video_plan(z);
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(tr!(
                        zoom_video_summary,
                        frames = frames,
                        seconds = crate::i18n::decimal(seconds, 0),
                        strips = strips,
                        width = sw.to_string(),
                        rows = sh.to_string()
                    ))
                    .monospace(),
                );
                let gb = |b: u64| crate::i18n::decimal(b as f64 / (1u64 << 30) as f64, 1);
                ui.label(egui::RichText::new(tr!(zoom_video_memory, size = gb(gpu))).monospace().weak());
                if z.keep {
                    // ~11 bytes a texel: measured on a 720p dive's strips (31 strips, 805 MB).
                    let disk = strips * sw as u64 * sh as u64 * 11;
                    ui.label(egui::RichText::new(tr!(zoom_video_disk, size = gb(disk))).monospace().weak());
                }
                ui.label(egui::RichText::new(tr!(zoom_video_separate)).weak().small());

                if running || !z.progress.is_empty() {
                    ui.add_space(4.0);
                    ui.separator();
                    if let Some((done, planned)) = z.progress_frames {
                        ui.add(
                            egui::ProgressBar::new((done as f32 / planned as f32).clamp(0.0, 1.0))
                                .desired_height(14.0)
                                .text(egui::RichText::new(tr!(zoom_video_progress, done = done, total = planned)).monospace().small()),
                        );
                    }
                    ui.horizontal(|ui| {
                        if running {
                            ui.add(egui::Spinner::new().size(12.0));
                        }
                        ui.label(
                            egui::RichText::new(if z.progress.is_empty() { tr!(zoom_video_starting) } else { z.progress.clone() })
                                .monospace()
                                .small(),
                        );
                    });
                }
                if running {
                    ui.label(egui::RichText::new(tr!(zoom_video_live_paused)).small().weak());
                }
                if let Some(st) = &z.status {
                    ui.colored_label(crate::theme::BRAND_ACCENT, egui::RichText::new(st).small());
                }

                ui.separator();
                let can = z.unsupported.is_none() && !z.out.trim().is_empty();
                crate::theme::action_row(ui, |ui| {
                    if !running && crate::theme::cancel_button(ui, &tr!(zoom_video_cancel)).clicked() {
                        close_dialog = true;
                    }
                    if ui
                        .button(format!("{} {}", crate::icons::COPY, tr!(zoom_video_copy_command)))
                        .on_hover_text(tr!(zoom_video_copy_command_hover))
                        .clicked()
                    {
                        copy_cmd = true;
                    }
                    if running {
                        if ui.button(tr!(zoom_video_stop)).clicked() {
                            stop = true;
                        }
                    } else if ui
                        .add_enabled(can, egui::Button::new(tr!(zoom_video_render)))
                        .on_hover_text(tr!(zoom_video_render_hover))
                        .clicked()
                    {
                        go = true;
                    }
                });
            });
        self.zoom_video.open = open && !close_dialog;

        if browse {
            let seed = std::path::Path::new(&self.zoom_video.out)
                .parent()
                .filter(|p| p.is_dir())
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| self.dialog_dir_default());
            if let Some(f) = rfd::FileDialog::new()
                .set_directory(seed)
                .set_file_name("zoom-video.mp4")
                .add_filter(tr!(zoom_video_filter), &["mp4", "mkv", "mov"])
                .save_file()
            {
                if let Some(d) = f.parent() {
                    self.remember_dir(d);
                }
                self.zoom_video.out = f.display().to_string();
            }
        }
        if browse_keep {
            if let Some(dir) = rfd::FileDialog::new().set_directory(self.dialog_dir_default()).pick_folder() {
                self.remember_dir(&dir);
                self.zoom_video.keep_dir = dir.display().to_string();
            }
        }
        if copy_cmd {
            let q = |s: String| if s.contains(' ') { format!("\"{s}\"") } else { s };
            let line = zoom_video_args(&self.zoom_video).into_iter().map(q).collect::<Vec<_>>().join(" ");
            ctx.copy_text(format!("fractadyne {line}"));
        }
        if stop {
            if let Some(child) = self.zoom_video.child.as_mut() {
                let _ = child.kill();
            }
        }
        if go {
            self.start_zoom_video(ctx);
        }
    }

    /// Save the session (the child reads its colouring there), then launch the render child and
    /// stream its progress.
    fn start_zoom_video(&mut self, ctx: &egui::Context) {
        use std::io::{BufRead, BufReader};
        let exe = match std::env::current_exe() {
            Ok(e) => e,
            Err(e) => {
                self.zoom_video.status = Some(tr!(zoom_video_no_exe, why = e.to_string()));
                return;
            }
        };
        self.autosave(ctx);
        let args = zoom_video_args(&self.zoom_video);
        let z = &mut self.zoom_video;
        z.progress.clear();
        z.progress_frames = None;
        z.status = None;
        z.error = None;
        // Stderr is piped: the child says why it failed there (`diag::log_line`).
        let child = std::process::Command::new(exe)
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();
        match child {
            Ok(mut c) => {
                let (tx, rx) = std::sync::mpsc::channel();
                z.rx = Some(rx);
                if let Some(out) = c.stdout.take() {
                    let tx = tx.clone();
                    std::thread::spawn(move || {
                        for line in BufReader::new(out).lines().map_while(Result::ok) {
                            let l = line.trim().to_string();
                            if !l.is_empty() && tx.send(RenderLine::Progress(l)).is_err() {
                                break;
                            }
                        }
                    });
                }
                if let Some(err) = c.stderr.take() {
                    std::thread::spawn(move || {
                        for line in BufReader::new(err).lines().map_while(Result::ok) {
                            let l = line.trim().to_string();
                            let interesting = l.contains("FAILED") || l.contains("panic") || l.contains("error") || l.contains("Error");
                            if interesting && tx.send(RenderLine::Error(l)).is_err() {
                                break;
                            }
                        }
                    });
                }
                crate::diag::breadcrumb(format!("GUI zoom video → {}", z.out));
                z.child = Some(c);
            }
            Err(e) => z.status = Some(tr!(zoom_video_start_failed, why = e.to_string())),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/ui/zoom_video/tests.rs"]
mod tests;

/// `--uitest`'s seed for this window: harness code, filed under `src/harness/`.
#[path = "../harness/ui/zoom_video/uitest_seed.rs"]
mod uitest_seed;
