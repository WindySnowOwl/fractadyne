//! The self-test `round-trip` (user, 2026-11-25: "confirm that the drawing settings including all
//! coloring options round trip properly through the paths that export and import them").
//!
//! ⭐**Judged by the picture, not by the view's lines.** The view's writer and reader share one rule
//! for what a picture depends on (`coloring_fields`' `shows`), so a round trip of the lines agrees
//! with itself even when that rule is wrong: with the Orbit trap method and the inside Solid, the
//! trap's shape was left out of every view and the line comparison passed. Here each drawing is
//! rendered, sent out one way and read back in onto a session SCRAMBLED away from it in every
//! setting, and rendered again: the two pictures must match. A setting a way loses shows as the
//! scrambled session's value in the picture.
//!
//! The ways out and back in, each through the app's own writer and reader: the view text itself; a
//! `.fdn` file (Save location, File > Open); the Share text (copied, pasted); a bookmark (through
//! `bookmarks.toml`); an exported image in each format (PNG, JPG, AVIF, GIF, EXR: the view the export
//! embeds, read back by File > Open); and the session (the file a restart, and a farm child, reads:
//! `coloring_from_session` and `effects_from_session`, as start-up runs them). The session way
//! scrambles only the coloring, the part of the session this test reads back.

use super::{frame, SelfCheck};
use crate::{ColorMethod, ExportFormat, FractalKind, InteriorMode, TrapType};
use fractadyne_core::Viewport;

/// The pictures' size: big enough to show every effect, small enough for 400 renders.
const W: u32 = 160;
const H: u32 = 120;

/// A way out and back in.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Way {
    Text,
    Fdn,
    Share,
    Bookmark,
    Image(ExportFormat),
    Session,
}

const WAYS: [Way; 10] = [
    Way::Text,
    Way::Fdn,
    Way::Share,
    Way::Bookmark,
    Way::Image(ExportFormat::Png),
    Way::Image(ExportFormat::Jpeg),
    Way::Image(ExportFormat::Avif),
    Way::Image(ExportFormat::Gif),
    Way::Image(ExportFormat::Exr),
    Way::Session,
];

impl Way {
    fn name(self) -> String {
        match self {
            Way::Text => "view text".into(),
            Way::Fdn => ".fdn file".into(),
            Way::Share => "Share text".into(),
            Way::Bookmark => "bookmark".into(),
            Way::Image(f) => format!("{} image", f.ext()),
            Way::Session => "session".into(),
        }
    }
}

/// One drawing: the family and view, the iterations and anti-aliasing, and the coloring.
struct Drawing {
    name: String,
    fractal: FractalKind,
    julia: Option<(f64, f64)>,
    center: (&'static str, &'static str),
    mag: f64,
    max_iter: u32,
    aa: u32,
    spec: fractadyne_state::ColoringSpec,
}

/// A preset other than `ice` and `ember` (the kitchen sink's palettes), for the scrambled session.
fn other_preset() -> (usize, &'static str) {
    fractadyne_color::PRESETS
        .iter()
        .enumerate()
        .find(|(_, p)| p.id != "ice" && p.id != "ember")
        .map_or((0, ""), |(i, p)| (i, p.id))
}

/// Every coloring setting away from its default; the method, the inside, the palette's mode and the
/// hook are set per drawing.
fn kitchen_sink() -> fractadyne_state::ColoringSpec {
    fractadyne_state::ColoringSpec {
        palette_idx: fractadyne_color::preset_by_id("ice").unwrap_or(1),
        palette_id: "ice".into(),
        cycle: 0.61,
        iter_div: 3,
        offset: 0.33,
        log_palette: false,
        transfer: "sqrt".into(),
        wrap: "mirror".into(),
        steps: 6,
        equalize: 0.5,
        palette_oklab: true,
        normalize_live: false,
        normalize_fit: true,
        light: true,
        light_angle: 1.25,
        light_height: 0.4,
        light_steep: true,
        light_steep_px: 12.0,
        light_highlight: 0.7,
        light_shininess: 96.0,
        light_highlight_color: [0.9, 0.85, 0.6],
        de: true,
        de_strength: 0.7,
        de_width: 2.5,
        edge: true,
        edge_inside: true,
        edge_width: 6.5,
        edge_strength: 0.45,
        edge_curve: "sqrt".into(),
        edge_color: [0.3, 0.2, 0.6],
        equipotentials: true,
        equipotential_spacing: 9.0,
        equipotential_width: 2.25,
        equipotential_strength: 0.8,
        equipotential_color: [0.95, 0.9, 0.2],
        color_method: ColorMethod::Smooth.key().into(),
        stripe_freq: 9.5,
        stripe_tail: true,
        stripe_tail_len: 64,
        trap_type: TrapType::Star.key().into(),
        trap_x: 0.3,
        trap_y: -0.45,
        trap_rotation: 25.0,
        trap_size: 0.6,
        interior_trap: TrapType::Cross.key().into(),
        interior_trap_x: -0.15,
        interior_trap_y: 0.35,
        interior_trap_rotation: -20.0,
        interior_trap_size: 1.5,
        // Shown only by the waves (`drawings`' last two).
        trap_phase: 30.0,
        interior_trap_phase: -45.0,
        trap_harmonics: 7,
        interior_trap_harmonics: 3,
        trap_sine_harmonics: 3,
        interior_trap_sine_harmonics: 4,
        custom_palette: vec![[0.0, 1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0]],
        custom_palette_flat: true,
        custom_segments: vec![fractadyne_state::PaletteSegment {
            left: 0.0,
            mid: 0.17,
            right: 1.0,
            left_color: [0.1, 0.2, 0.3, 1.0],
            right_color: [0.9, 0.8, 0.7, 0.5],
            blend: 2,
            space: 1,
            blend_params: [0.25, 0.9, 0.75, 0.1],
        }],
        use_custom_palette: true,
        use_duotone: false,
        use_binary: false,
        duotone_lo: [0.1, 0.2, 0.3],
        duotone_hi: [0.9, 0.8, 0.7],
        interior: InteriorMode::Trap.key().into(),
        interior_palette: "ember".into(),
        interior_color: [0.9, 0.2, 0.6],
        interior_fit: true,
        palette_aa: false,
        palette_brightness: -0.3,
        palette_contrast: 0.6,
        palette_saturation: 0.5,
        julia_linked: true,
        coloring_hook: String::new(),
        coloring_hook_params: vec![0.75, 0.25, 0.5, 1.0],
    }
}

/// The session every drawing is read back onto: away from the kitchen sink in every setting, and
/// in the method and the inside from `d`'s own.
fn scrambled_for(d: &fractadyne_state::ColoringSpec) -> fractadyne_state::ColoringSpec {
    let (idx, id) = other_preset();
    let distance = ColorMethod::Distance.key();
    fractadyne_state::ColoringSpec {
        palette_idx: idx,
        palette_id: id.into(),
        cycle: 0.2,
        iter_div: 1,
        offset: 0.05,
        log_palette: true,
        transfer: "linear".into(),
        wrap: "clamp".into(),
        steps: 0,
        equalize: 0.0,
        palette_oklab: false,
        normalize_live: true,
        normalize_fit: false,
        light: false,
        light_angle: -0.5,
        light_height: 2.0,
        light_steep: false,
        light_steep_px: 30.0,
        light_highlight: 0.2,
        light_shininess: 20.0,
        light_highlight_color: [0.2, 0.3, 0.4],
        de: false,
        de_strength: 0.3,
        de_width: 1.0,
        edge: false,
        edge_inside: false,
        edge_width: 3.0,
        edge_strength: 0.9,
        edge_curve: "hard".into(),
        edge_color: [0.7, 0.7, 0.1],
        equipotentials: false,
        equipotential_spacing: 20.0,
        equipotential_width: 1.0,
        equipotential_strength: 0.3,
        equipotential_color: [0.1, 0.1, 0.9],
        color_method: if d.color_method == distance { ColorMethod::Fringe.key() } else { distance }.into(),
        stripe_freq: 3.0,
        stripe_tail: false,
        stripe_tail_len: 900,
        trap_type: TrapType::Circle.key().into(),
        trap_x: -0.2,
        trap_y: 0.1,
        trap_rotation: -60.0,
        trap_size: 1.8,
        interior_trap: TrapType::Line.key().into(),
        interior_trap_x: 0.2,
        interior_trap_y: -0.3,
        interior_trap_rotation: 45.0,
        interior_trap_size: 0.4,
        trap_phase: -100.0,
        interior_trap_phase: 120.0,
        trap_harmonics: 11,
        interior_trap_harmonics: 9,
        trap_sine_harmonics: 2,
        interior_trap_sine_harmonics: 6,
        custom_palette: vec![[0.0, 0.2, 0.9, 0.2], [1.0, 0.9, 0.9, 0.1]],
        custom_palette_flat: false,
        custom_segments: vec![fractadyne_state::PaletteSegment {
            left: 0.0,
            mid: 0.5,
            right: 1.0,
            left_color: [0.8, 0.1, 0.1, 1.0],
            right_color: [0.1, 0.1, 0.8, 1.0],
            blend: 0,
            space: 0,
            blend_params: [0.5, 0.5, 0.5, 0.5],
        }],
        use_custom_palette: !d.use_custom_palette,
        use_duotone: !d.use_duotone,
        use_binary: false,
        duotone_lo: [0.5, 0.1, 0.1],
        duotone_hi: [0.1, 0.5, 0.9],
        interior: if d.interior == InteriorMode::Trap.key() { InteriorMode::Solid } else { InteriorMode::Trap }.key().into(),
        interior_palette: String::new(),
        interior_color: [0.1, 0.8, 0.3],
        interior_fit: false,
        palette_aa: true,
        palette_brightness: 0.2,
        palette_contrast: -0.2,
        palette_saturation: 1.4,
        julia_linked: false,
        coloring_hook: String::new(),
        coloring_hook_params: vec![0.1, 0.9, 0.3, 0.2],
    }
}

/// The drawings: every method with each inside, the palette's four modes in turn (a gradient of
/// its own, Duotone, Binary, a preset); a coloring hook; a Julia view; Burning Ship; a deep view;
/// and the waves, where the phase and the harmonics show (each wave outside and inside in turn).
fn drawings() -> Vec<Drawing> {
    let sink = kitchen_sink();
    let overview = ("-0.6", "0");
    let base = |name: String, spec: fractadyne_state::ColoringSpec| Drawing {
        name,
        fractal: FractalKind::Mandelbrot,
        julia: None,
        center: overview,
        mag: 1.0,
        max_iter: 500,
        aa: 2,
        spec,
    };
    let mut out = Vec::new();
    // Each method with each inside, in a gradient of its own (which shows every value the method
    // draws: Binary, say, shows only which side of a threshold it falls).
    for m in ColorMethod::ALL {
        for inside in InteriorMode::ALL {
            let spec = fractadyne_state::ColoringSpec { color_method: m.key().into(), interior: inside.key().into(), ..sink.clone() };
            out.push(base(format!("{}, inside {}", m.key(), inside.key()), spec));
        }
    }
    // The palette's other modes: Duotone, Binary, a preset.
    for (duo, bin, mode) in [(true, false, "Duotone"), (false, true, "Binary"), (false, false, "a preset")] {
        let spec = fractadyne_state::ColoringSpec { use_custom_palette: false, use_duotone: duo, use_binary: bin, ..sink.clone() };
        out.push(base(format!("smooth, inside trap, {mode}"), spec));
    }
    let hook = fractadyne_gpu::color_hook::EXAMPLES[1].1.replace("\r\n", "\n");
    out.push(base("a coloring hook".into(), fractadyne_state::ColoringSpec { coloring_hook: hook, ..sink.clone() }));
    out.push(Drawing {
        name: "a Julia view".into(),
        julia: Some((-0.8, 0.156)),
        center: ("0", "0"),
        ..base(String::new(), fractadyne_state::ColoringSpec { color_method: ColorMethod::OrbitTrap.key().into(), ..sink.clone() })
    });
    out.push(Drawing {
        name: "Burning Ship, stripes".into(),
        fractal: FractalKind::BurningShip,
        center: ("-0.5", "-0.5"),
        ..base(String::new(), fractadyne_state::ColoringSpec { color_method: ColorMethod::Stripe.key().into(), ..sink.clone() })
    });
    out.push(Drawing {
        name: "the seahorse at 2e3x".into(),
        center: ("-0.743643887037151", "0.131825904205330"),
        mag: 2.0e3,
        max_iter: 2_000,
        ..base(String::new(), sink.clone())
    });
    let waves = |outside: TrapType, inside: TrapType| fractadyne_state::ColoringSpec {
        color_method: ColorMethod::OrbitTrap.key().into(),
        trap_type: outside.key().into(),
        interior_trap: inside.key().into(),
        ..sink.clone()
    };
    out.push(base(
        "the trap a sine wave of 3 harmonics at 30 degrees, the inside's a square wave of 3 at -45".into(),
        waves(TrapType::Sine, TrapType::SquareWave),
    ));
    out.push(base(
        "the trap a square wave of 7 at 30 degrees, the inside's a sine wave of 4 harmonics at -45".into(),
        waves(TrapType::SquareWave, TrapType::Sine),
    ));
    out
}

/// The settings that are the same in a drawing and the session it is read back onto, over every
/// drawing: none may be, or a way could lose it unseen.
fn unexercised(all: &[Drawing]) -> Vec<String> {
    let fields = |s: &fractadyne_state::ColoringSpec| match serde_json::to_value(s) {
        Ok(serde_json::Value::Object(m)) => m,
        _ => serde_json::Map::new(),
    };
    let mut same: Vec<String> = fields(&kitchen_sink()).keys().cloned().collect();
    for d in all {
        let (a, b) = (fields(&d.spec), fields(&scrambled_for(&d.spec)));
        same.retain(|k| a.get(k) == b.get(k));
    }
    same
}

/// A value cut for a message.
fn short(v: &str) -> String {
    if v.chars().count() > 40 {
        format!("{}...", v.chars().take(40).collect::<String>())
    } else {
        v.to_string()
    }
}

impl crate::FractadyneApp {
    /// The view, as it is set, drawn at `W`x`H` (8-bit sRGB).
    fn rt_render(&self, device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue) -> Option<Vec<u8>> {
        let mut vp = Viewport::new(W as f64, H as f64);
        vp.set_center_mag(self.viewport.center_x.clone(), self.viewport.center_y.clone(), self.viewport.magnification());
        vp.precision = self.viewport.precision.max(64);
        let req = self.current_export_request_for(&vp, self.julia_mode);
        let progress = std::sync::atomic::AtomicU32::new(0);
        let cancel = std::sync::atomic::AtomicBool::new(false);
        fractadyne_gpu::render_export(device, queue, &req, &progress, &cancel)
            .ok()
            .map(|r| fractadyne_export::to_srgb8_dithered(&r.pixels, r.width))
    }

    /// Set the session to drawing `d`.
    fn rt_set(&mut self, d: &Drawing) {
        self.dual = false;
        self.fractal = d.fractal;
        self.julia_mode = d.julia.is_some();
        if let Some(c) = d.julia {
            self.julia_c = c;
        }
        self.render_cfg.auto_iter = false;
        self.render_cfg.max_iter = d.max_iter;
        self.render_cfg.aa = d.aa;
        let at = |t: &str| fractadyne_core::parse_bf_prec(t, 128).expect("a drawing's center");
        self.viewport.set_center_mag(at(d.center.0), at(d.center.1), d.mag);
        self.anim.palette_anim = crate::PaletteAnim::Off;
        self.apply_coloring_spec(&d.spec);
        self.effects.light_anim = false;
        self.effects.de_anim = false;
    }

    /// Scramble the session away from drawing `d`: its coloring, and (`whole`) the family, the view,
    /// the iterations and the anti-aliasing too.
    fn rt_scramble(&mut self, d: &Drawing, whole: bool) {
        self.apply_coloring_spec(&scrambled_for(&d.spec));
        if whole {
            self.fractal = if d.fractal == FractalKind::Mandelbrot { FractalKind::Tricorn } else { FractalKind::Mandelbrot };
            self.julia_mode = d.julia.is_none();
            self.julia_c = (0.285, 0.01);
            self.render_cfg.max_iter = d.max_iter + 333;
            self.render_cfg.aa = if d.aa == 1 { 2 } else { 1 };
            let at = |t: &str| fractadyne_core::parse_bf_prec(t, 128).expect("the scrambled center");
            self.viewport.set_center_mag(at("0.25"), at("0.5"), 3.0);
        }
    }

    /// Out by `way` and back in onto a session scrambled away from `d`: what the import said, if
    /// anything; Err when the way itself failed.
    fn rt_trip(&mut self, d: &Drawing, way: Way, dir: &std::path::Path) -> Result<Option<String>, String> {
        let checked = |app: &mut Self, text: &str, source: &str| -> Result<Option<String>, String> {
            app.load_view_checked(text, source.into()).map(|r| r.note()).ok_or_else(|| "its checksum did not match".to_string())
        };
        match way {
            Way::Text => {
                let t = self.view_metadata();
                self.rt_scramble(d, true);
                Ok(self.load_view_metadata(&t).note())
            }
            Way::Fdn => {
                // As the Share dialog's Save writes this view, and File > Open reads a `.fdn`.
                let p = dir.join("round-trip.fdn");
                let text = crate::export::wrap_view_text(&format!("{}\n", self.view_metadata()));
                fractadyne_state::durable::write_atomic(&p, text.as_bytes()).map_err(|e| e.to_string())?;
                self.rt_scramble(d, true);
                let read = crate::txt_location::read_location_text(&p).map_err(|e| format!("{e:?}"))?;
                checked(self, &read.text, "round-trip.fdn")
            }
            Way::Share => {
                self.rebuild_share_text();
                let t = self.share.text.clone();
                self.rt_scramble(d, true);
                // As Apply reads the box.
                let t = t.trim();
                crate::location_text_verdict(t)?;
                checked(self, t, "Share")
            }
            Way::Bookmark => {
                let file = crate::BookmarkFile {
                    bookmark: vec![crate::Bookmark { name: "round trip".into(), meta: self.view_metadata(), thumb: String::new() }],
                };
                let text = toml::to_string_pretty(&file).map_err(|e| e.to_string())?;
                self.rt_scramble(d, true);
                let back: crate::BookmarkFile = toml::from_str(&text).map_err(|e| e.to_string())?;
                let meta = back.bookmark.into_iter().next().map(|b| b.meta).ok_or("the bookmark did not come back")?;
                Ok(self.load_view_metadata(&meta).note())
            }
            Way::Image(format) => {
                // The view an export of the view on screen embeds, in a small picture of the format.
                let p = dir.join(format!("round-trip.{}", format.ext()));
                let meta = self.export_meta_live().main;
                let enc = crate::export::ExportEncoding { format, ..self.export_encoding() };
                enc.write(&p, 8, 8, &[0.5f32; 8 * 8 * 4], &meta).map_err(|e| e.to_string())?;
                self.rt_scramble(d, true);
                let read = match format {
                    ExportFormat::Png => fractadyne_export::read_png_metadata(&p),
                    ExportFormat::Jpeg => fractadyne_export::read_jpeg_metadata(&p),
                    ExportFormat::Avif => fractadyne_export::read_avif_metadata(&p),
                    ExportFormat::Gif => fractadyne_export::read_gif_metadata(&p),
                    ExportFormat::Exr => fractadyne_export::read_exr_metadata(&p),
                };
                let m = read.map_err(|e| e.to_string())?.ok_or("the file came back without a view")?;
                if !crate::export::looks_like_a_view(&m) {
                    return Err("the file's text is not a view".into());
                }
                checked(self, &m, &format!("round-trip.{}", format.ext()))
            }
            Way::Session => {
                // As the autosave writes the coloring and start-up reads it back.
                let s = fractadyne_state::SessionState {
                    coloring: self.coloring_spec(),
                    light_anim: self.effects.light_anim,
                    de_anim: self.effects.de_anim,
                    coloring_hook_path: String::new(),
                    ..Default::default()
                };
                let text = toml::to_string_pretty(&s).map_err(|e| e.to_string())?;
                self.rt_scramble(d, false);
                let (mut back, load) = fractadyne_state::parse_with_status(&text);
                if load != fractadyne_state::StateLoad::Ok {
                    return Err(format!("the session read back as {load:?}"));
                }
                let fixed = crate::session_sanitize::sanitize(&mut back);
                self.coloring = crate::coloring_from_session(&back, &[]);
                self.effects = crate::effects_from_session(&back);
                self.restore_session_hook(&back.coloring.coloring_hook, &back.coloring.coloring_hook_params, &back.coloring_hook_path);
                if self.coloring.custom_segments.is_empty() && !self.coloring.custom_palette.is_empty() {
                    self.rebuild_segments_from_palette();
                }
                Ok((!fixed.is_empty()).then(|| format!("start-up's sanitize changed {fixed:?}")))
            }
        }
    }

    /// What of the drawing is set now, to compare before and after a way: the coloring field by field
    /// (as a view of it, `coloring_spec`), and the family, the view, the iterations and the AA.
    fn rt_state(&self) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = match serde_json::to_value(self.coloring_spec()) {
            Ok(serde_json::Value::Object(m)) => m.into_iter().map(|(k, v)| (k, v.to_string())).collect(),
            _ => Vec::new(),
        };
        out.push(("fractal".into(), format!("{:?}", self.fractal)));
        out.push(("julia".into(), format!("{} {:?}", self.julia_mode, self.julia_c)));
        out.push(("center".into(), format!("{} {}", fractadyne_core::to_decimal_string(&self.viewport.center_x), fractadyne_core::to_decimal_string(&self.viewport.center_y))));
        out.push(("magnification".into(), format!("{:.6e}", self.viewport.magnification())));
        out.push(("max_iter".into(), format!("{} auto {}", self.render_cfg.max_iter, self.render_cfg.auto_iter)));
        out.push(("aa".into(), self.render_cfg.aa.to_string()));
        out
    }

    /// After a way brought `a`'s drawing back different: the coloring settings that, put back to what
    /// they were (`before`) one at a time, restore the picture (within 0.01 levels), each with how
    /// close it came. The session is left as the way left it.
    fn rt_culprits(&mut self, before: &[(String, String)], a: &[u8], device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue) -> Vec<String> {
        let got = self.coloring_spec();
        let Ok(serde_json::Value::Object(now)) = serde_json::to_value(&got) else { return Vec::new() };
        let mut out = Vec::new();
        for (k, v) in before {
            let Some(cur) = now.get(k) else { continue };
            if cur.to_string() == *v {
                continue;
            }
            let Ok(was) = serde_json::from_str::<serde_json::Value>(v) else { continue };
            let mut m = now.clone();
            m.insert(k.clone(), was);
            let Ok(spec) = serde_json::from_value::<fractadyne_state::ColoringSpec>(serde_json::Value::Object(m)) else { continue };
            self.apply_coloring_spec(&spec);
            if let Some(b) = self.rt_render(device, queue) {
                let dist = frame::distance(a, &b);
                if dist <= 0.01 {
                    out.push(format!("{k} ({dist:.3})"));
                }
            }
        }
        self.apply_coloring_spec(&got);
        out
    }

    /// The checks: one per way, over every drawing, and the coverage of the scramble.
    pub(super) fn round_trip_checks(&mut self, device: &eframe::wgpu::Device, queue: &eframe::wgpu::Queue) -> Vec<SelfCheck> {
        let was = (self.coloring_spec(), self.fractal, self.julia_mode, self.julia_c, self.viewport.clone(), self.dual);
        let was_cfg = (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.render_cfg.aa);
        let dir = std::env::temp_dir().join(format!("fractadyne-round-trip-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let all = drawings();
        let mut checks = Vec::new();
        let same = unexercised(&all);
        checks.push(SelfCheck {
            category: "View format",
            name: "round trip: every coloring setting differs between a drawing and the session it is read onto".into(),
            params: format!("{} drawings; the kitchen sink and the scrambled session, field by field", all.len()),
            result: if same.is_empty() { "every setting differs in some drawing".into() } else { format!("never different: {}", same.join(", ")) },
            threshold: "none the same in every drawing (else a way could lose it unseen)",
            pass: same.is_empty(),
        });
        // Each drawing's own picture, and that the scramble changes it (else the trips see nothing).
        let mut base: Vec<Option<Vec<u8>>> = Vec::new();
        let mut vacuous: Vec<String> = Vec::new();
        for d in &all {
            self.rt_set(d);
            let a = self.rt_render(device, queue);
            self.rt_scramble(d, false);
            let s = self.rt_render(device, queue);
            match (&a, &s) {
                (Some(a), Some(s)) if frame::distance(a, s) >= 2.0 && frame::coherent(a) => {}
                (Some(a), Some(s)) => vacuous.push(format!("{} (the scramble moved it {:.2})", d.name, frame::distance(a, s))),
                _ => vacuous.push(format!("{} (did not render)", d.name)),
            }
            base.push(a);
        }
        checks.push(SelfCheck {
            category: "View format",
            name: "round trip: each drawing renders, and the scrambled coloring changes it".into(),
            params: format!("{} drawings at {W}x{H}", all.len()),
            result: if vacuous.is_empty() { "all".into() } else { vacuous.join("; ") },
            threshold: "every drawing coherent, and >= 2 levels from its scrambled coloring",
            pass: vacuous.is_empty(),
        });
        for way in WAYS {
            let mut bad: Vec<String> = Vec::new();
            let mut worst = 0.0f64;
            for (d, a) in all.iter().zip(&base) {
                self.rt_set(d);
                let before = self.rt_state();
                match self.rt_trip(d, way, &dir) {
                    Err(e) => bad.push(format!("{}: {e}", d.name)),
                    Ok(note) => {
                        if let Some(n) = note {
                            bad.push(format!("{}: the import said {n:?}", d.name));
                        }
                        match (a, self.rt_render(device, queue)) {
                            (Some(a), Some(b)) => {
                                let dist = frame::distance(a, &b);
                                worst = worst.max(dist);
                                if dist > 0.01 {
                                    let after = self.rt_state();
                                    let moved: Vec<String> = before
                                        .iter()
                                        .filter(|(k, v)| after.iter().find(|(k2, _)| k2 == k).is_none_or(|(_, v2)| v2 != v))
                                        .map(|(k, v)| {
                                            let now = after.iter().find(|(k2, _)| k2 == k).map_or("(gone)", |(_, v2)| v2.as_str());
                                            format!("{k} {} -> {}", short(v), short(now))
                                        })
                                        .collect();
                                    let culprits = self.rt_culprits(&before, a, device, queue);
                                    bad.push(format!(
                                        "{}: the picture came back {dist:.2} levels off; {}; changed: {}",
                                        d.name,
                                        if culprits.is_empty() { "no one coloring setting put back restores it".into() } else { format!("put back, these restore it: {}", culprits.join(", ")) },
                                        if moved.is_empty() { "nothing this test reads".into() } else { moved.join(", ") }
                                    ));
                                }
                            }
                            _ => bad.push(format!("{}: did not render", d.name)),
                        }
                    }
                }
            }
            checks.push(SelfCheck {
                category: "View format",
                name: format!("round trip through the {}: every drawing comes back the same", way.name()),
                params: format!("{} drawings, read back onto a scrambled session, {W}x{H}", all.len()),
                result: if bad.is_empty() { format!("all the same (worst meanΔ {worst:.4})") } else { bad.join("; ") },
                threshold: "every drawing's picture within 0.01 levels, and nothing said on import",
                pass: bad.is_empty(),
            });
        }
        let _ = std::fs::remove_dir_all(&dir);
        self.apply_coloring_spec(&was.0);
        (self.fractal, self.julia_mode, self.julia_c, self.viewport, self.dual) = (was.1, was.2, was.3, was.4, was.5);
        (self.render_cfg.max_iter, self.render_cfg.auto_iter, self.render_cfg.aa) = was_cfg;
        checks
    }
}
