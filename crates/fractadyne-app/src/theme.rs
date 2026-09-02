//! Branding & theme: the Fractadyne color identity (amber accent), the **dark and light** egui
//! visuals built from a shared semantic [`Palette`], the **Spline Sans / Spline Sans Mono**
//! typography ([`install_fonts`]), the two-color wordmark, the procedural window icon, and a couple
//! of small painting helpers. Colors + typography follow the brand system; the app name is its own.

use eframe::egui;

/// Fixed brand accent (amber #E0A030) + wordmark text (#E6E7EA), for overlays drawn ON the fractal
/// render (crosshairs, markers, the export watermark) — the background there is the image, not the
/// UI theme, so these stay constant. UI *chrome* uses the theme-aware [`ui_accent`] instead.
pub(crate) const BRAND_ACCENT: egui::Color32 = egui::Color32::from_rgb(0xE0, 0xA0, 0x30);
pub(crate) const BRAND_TEXT: egui::Color32 = egui::Color32::from_rgb(0xE6, 0xE7, 0xEA);

/// Linear RGBA interpolation between two colors (`t` in 0..1).
pub(crate) fn lerp_color(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    egui::Color32::from_rgba_unmultiplied(
        mix(a.r(), b.r()),
        mix(a.g(), b.g()),
        mix(a.b(), b.b()),
        mix(a.a(), b.a()),
    )
}

/// Light or dark UI theme (persisted; see [`apply_theme`]).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum ThemeMode {
    #[default]
    Dark,
    Light,
}

impl ThemeMode {
    /// Persisted key in `SessionState`.
    pub(crate) fn key(self) -> &'static str {
        match self {
            ThemeMode::Dark => "dark",
            ThemeMode::Light => "light",
        }
    }
    pub(crate) fn from_key(s: &str) -> ThemeMode {
        if s == "light" {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        }
    }
    pub(crate) fn label(self) -> &'static str {
        match self {
            ThemeMode::Dark => "Dark",
            ThemeMode::Light => "Light",
        }
    }
}

const fn rgb(r: u8, g: u8, b: u8) -> egui::Color32 {
    egui::Color32::from_rgb(r, g, b)
}

/// Semantic color roles for one theme (from the brand palette). One `Palette` builds the full egui
/// `Visuals`; a couple of fields also feed the theme-aware brand chrome ([`ui_accent`]).
struct Palette {
    /// Outermost / darkest fill (dark) or warm base (light).
    window: egui::Color32,
    /// Side / central panels.
    panel: egui::Color32,
    /// Faint raised surface (group boxes, read-out troughs).
    surface: egui::Color32,
    /// Inputs / cards (the most-raised neutral fill).
    elevated: egui::Color32,
    /// Hovered widget fill.
    hover: egui::Color32,
    /// Pressed widget fill.
    active: egui::Color32,
    /// Hairlines / widget strokes / separators.
    border: egui::Color32,
    /// Primary text.
    text: egui::Color32,
    /// Amber accent.
    accent: egui::Color32,
    /// Text selection background (amber tint).
    selection: egui::Color32,
    dark_mode: bool,
}

impl Palette {
    /// Fissiodyne Dark: near-black warm charcoals, bright ink, amber `#E0A030`.
    fn dark() -> Self {
        Palette {
            window: rgb(0x14, 0x15, 0x18),
            panel: rgb(0x1A, 0x1B, 0x1E),
            surface: rgb(0x23, 0x24, 0x28),
            elevated: rgb(0x2C, 0x2E, 0x33),
            hover: rgb(0x3A, 0x3D, 0x44),
            active: rgb(0x45, 0x49, 0x52),
            border: rgb(0x3A, 0x3D, 0x44),
            text: rgb(0xE6, 0xE7, 0xEA),
            accent: rgb(0xE0, 0xA0, 0x30),
            selection: rgb(0x4A, 0x38, 0x14),
            dark_mode: true,
        }
    }
    /// Fissiodyne Light: warm off-whites, near-black ink, a deeper amber `#B98212` for contrast.
    fn light() -> Self {
        Palette {
            window: rgb(0xFB, 0xFA, 0xF8),
            panel: rgb(0xF8, 0xF7, 0xF4),
            surface: rgb(0xF1, 0xEF, 0xE9),
            elevated: rgb(0xFF, 0xFF, 0xFF),
            hover: rgb(0xEF, 0xEE, 0xEA),
            active: rgb(0xE5, 0xE2, 0xDC),
            border: rgb(0xD8, 0xD5, 0xCE),
            text: rgb(0x2A, 0x2B, 0x2E),
            accent: rgb(0xB9, 0x82, 0x12),
            selection: rgb(0xF0, 0xE3, 0xBC),
            dark_mode: false,
        }
    }
}

/// WCAG relative luminance of an opaque colour (sRGB, gamma-decoded).
#[cfg(test)]
fn relative_luminance(c: egui::Color32) -> f32 {
    fn chan(v: u8) -> f32 {
        let s = v as f32 / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * chan(c.r()) + 0.7152 * chan(c.g()) + 0.0722 * chan(c.b())
}

/// WCAG contrast ratio between two opaque colours, 1.0 (identical) to 21.0 (black on white).
///
/// Used to hold the theme to a floor rather than to a taste: "sufficient contrast to read every
/// label" is checkable, "consistent theme throughout" is not.
#[cfg(test)]
pub(crate) fn contrast_ratio(a: egui::Color32, b: egui::Color32) -> f32 {
    let (la, lb) = (relative_luminance(a), relative_luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// The active theme's amber accent for brand *chrome* (wordmark, help headers, panel accents) — it
/// tracks light/dark, unlike the fixed [`BRAND_ACCENT`] used for overlays drawn on the render.
/// Stored in `Visuals::hyperlink_color` by [`apply_theme`].
pub(crate) fn ui_accent(ctx: &egui::Context) -> egui::Color32 {
    ctx.style().visuals.hyperlink_color
}

/// Register the Spline Sans (UI) + Spline Sans Mono (numeric / data) brand typefaces, keeping
/// egui's defaults as fallbacks for glyphs they lack (math arrows like →, emoji). Call once at
/// startup, before [`apply_theme`].
pub(crate) fn install_fonts(ctx: &egui::Context) {
    use std::sync::Arc;
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "SplineSans".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!("../assets/fonts/SplineSans.ttf"))),
    );
    fonts.font_data.insert(
        "SplineSansMono".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/SplineSansMono.ttf"
        ))),
    );
    // Lucide, subset to the icons actually used (`scripts/subset_lucide.py`). Its glyphs live in
    // the Private Use Area, so appending it can never shadow a real character — it only supplies
    // codepoints nothing else defines. This replaced the emoji that used to serve as icons: the
    // bundled fonts cover an arbitrary subset of those, four of which shipped as tofu squares,
    // and the ones that did render came from different families.
    fonts.font_data.insert(
        "Lucide".to_owned(),
        Arc::new(egui::FontData::from_static(include_bytes!("../assets/fonts/Lucide.ttf"))),
    );
    // Proportional: Spline Sans first, then the egui defaults (Ubuntu / Hack / emoji) so any glyph
    // Spline Sans lacks (math arrows, sub/superscripts in the Help formulas) still resolves.
    if let Some(prop) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        prop.insert(0, "SplineSans".to_owned());
        if !prop.iter().any(|f| f == "Hack") {
            prop.push("Hack".to_owned());
        }
        // Last: it only ever resolves Private Use Area codepoints nothing ahead of it defines.
        prop.push("Lucide".to_owned());
    }
    // Monospace (the numeric read-outs): Spline Sans Mono first, Hack behind it for fallbacks.
    if let Some(mono) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        mono.insert(0, "SplineSansMono".to_owned());
    }
    ctx.set_fonts(fonts);

    // Secondary text, one step up from egui's default.
    //
    // egui ships `Small` at 9.0 against a 12.5 body — 72%. That is fine on a laptop panel and at
    // the edge of legibility on a large high-resolution desktop monitor, where the OS scaling that
    // would fix it is often turned *down* precisely because everything else is already big enough.
    // `Small` carries real content here rather than decoration: the update-check result, the
    // "Cycle, offset, animation…" hints under menu groups, licence and attribution lines, and the
    // explanatory captions in dialogs. 11.0 keeps it clearly subordinate to body text (88%) while
    // being comfortably readable.
    //
    // ⚠Only `Small` moves. Body/Button/Heading/Monospace stay at egui's defaults, so this changes
    // the small print and nothing else — and the whole UI still scales independently via the UI
    // scale setting (`ui_scale`, 0.6–2.5), which is the right knob for "everything is too small"
    // as opposed to "the small print is too small".
    ctx.all_styles_mut(|style| {
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::new(11.0, egui::FontFamily::Proportional),
        );
    });
}

/// Apply the Fractadyne theme for `mode` (charcoal panels + amber on dark, warm off-white + deeper
/// amber on light) by building egui `Visuals` from the semantic [`Palette`]. Cheap; call on startup
/// and whenever the theme toggles. Fonts are separate — see [`install_fonts`].
pub(crate) fn apply_theme(ctx: &egui::Context, mode: ThemeMode) {
    let p = match mode {
        ThemeMode::Dark => Palette::dark(),
        ThemeMode::Light => Palette::light(),
    };
    let mut v = if p.dark_mode {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    v.panel_fill = p.panel;
    v.window_fill = p.panel;
    v.window_stroke = egui::Stroke::new(1.0_f32, p.border);
    v.extreme_bg_color = p.window; // text-edit / slider-trough backing
    v.faint_bg_color = p.surface; // striped/group backgrounds
    v.hyperlink_color = p.accent; // doubles as the theme accent (see `ui_accent`)
    v.selection.bg_fill = p.selection;
    v.selection.stroke = egui::Stroke::new(1.0_f32, p.accent);

    // Primary text + separators.
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, p.text);
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0_f32, p.border);

    // Interactive widgets: neutral fills, amber outline/text on hover & press.
    v.widgets.inactive.weak_bg_fill = p.surface;
    v.widgets.inactive.bg_fill = p.elevated;
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, p.border);
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0_f32, p.text);
    v.widgets.hovered.bg_fill = p.hover;
    v.widgets.hovered.weak_bg_fill = p.hover;
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, p.accent);
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.5_f32, p.accent);
    v.widgets.active.bg_fill = p.active;
    v.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, p.accent);
    v.widgets.active.fg_stroke = egui::Stroke::new(1.5_f32, p.accent);
    v.widgets.open.bg_fill = p.elevated;
    v.widgets.open.bg_stroke = egui::Stroke::new(1.0_f32, p.border);

    ctx.set_visuals(v);
}

/// The "Fd" brand mark as a two-color layout job (F in `f_col`, d in `d_col`) at `px` size,
/// using the same proportional font as the header wordmark. Shared by the live-view watermark
/// (drawn with the painter) and the export watermark (rasterized from the font atlas).
pub(crate) fn brand_mark_job(px: f32, f_col: egui::Color32, d_col: egui::Color32) -> egui::text::LayoutJob {
    let font = egui::FontId::proportional(px);
    let mut job = egui::text::LayoutJob::default();
    job.append("F", 0.0, egui::TextFormat { font_id: font.clone(), color: f_col, ..Default::default() });
    job.append("d", 0.0, egui::TextFormat { font_id: font, color: d_col, ..Default::default() });
    job
}

/// The two-color "Fractadyne" logotype (Fracta + amber dyne), for the top bar. Theme-aware: on dark
/// it's the bright brand ink (white `#E6E7EA`) + gold accent; on light it's the near-black ink +
/// the deeper amber. The "dyne" always uses the theme accent.
pub(crate) fn brand_wordmark(ui: &mut egui::Ui) {
    let font = egui::FontId::proportional(15.0);
    let text = if ui.visuals().dark_mode {
        BRAND_TEXT
    } else {
        ui.visuals().text_color()
    };
    let accent = ui.visuals().hyperlink_color;
    let mut job = egui::text::LayoutJob::default();
    job.append(
        "Fracta",
        0.0,
        egui::TextFormat { font_id: font.clone(), color: text, ..Default::default() },
    );
    job.append(
        "dyne",
        0.0,
        egui::TextFormat { font_id: font, color: accent, ..Default::default() },
    );
    ui.add_space(2.0);
    ui.label(job);
}

/// Window icon: a Fractadyne render (bundled asset). The source is trimmed of its border,
/// center-cropped to a square, and box-downsampled to 256×256. Falls back to the procedural
/// mark if the embedded PNG can't be decoded.
pub(crate) fn brand_icon() -> egui::IconData {
    const RAW: &[u8] = include_bytes!("../assets/icon.png");
    if let Ok((w, h, rgba)) = fractadyne_export::read_png_rgba8_bytes(RAW) {
        if w > 0 && h > 0 {
            return icon_from_render(w as usize, h as usize, &rgba);
        }
    }
    procedural_icon()
}

/// Trim a small border margin, center-crop to a square, and box-downsample to `OUT×OUT` RGBA.
fn icon_from_render(w: usize, h: usize, rgba: &[u8]) -> egui::IconData {
    const OUT: usize = 256;
    // Trim ~3% off each edge to drop any capture border, then take the largest centered square.
    let mx = (w * 3 / 100).max(2);
    let my = (h * 3 / 100).max(2);
    let (ix0, iy0) = (mx, my);
    let (iw, ih) = (w - 2 * mx, h - 2 * my);
    let side = iw.min(ih);
    let cx0 = ix0 + (iw - side) / 2;
    let cy0 = iy0 + (ih - side) / 2;
    let mut out = vec![0u8; OUT * OUT * 4];
    for oy in 0..OUT {
        for ox in 0..OUT {
            // Box-average the source block mapping to this output texel.
            let sx0 = cx0 + ox * side / OUT;
            let sx1 = (cx0 + (ox + 1) * side / OUT).max(sx0 + 1).min(w);
            let sy0 = cy0 + oy * side / OUT;
            let sy1 = (cy0 + (oy + 1) * side / OUT).max(sy0 + 1).min(h);
            let (mut r, mut g, mut b, mut n) = (0u32, 0u32, 0u32, 0u32);
            for sy in sy0..sy1 {
                for sx in sx0..sx1 {
                    let i = (sy * w + sx) * 4;
                    r += rgba[i] as u32;
                    g += rgba[i + 1] as u32;
                    b += rgba[i + 2] as u32;
                    n += 1;
                }
            }
            let n = n.max(1);
            let o = (oy * OUT + ox) * 4;
            out[o] = (r / n) as u8;
            out[o + 1] = (g / n) as u8;
            out[o + 2] = (b / n) as u8;
            out[o + 3] = 255;
        }
    }
    egui::IconData { rgba: out, width: OUT as u32, height: OUT as u32 }
}

/// Procedural window icon: concentric amber rings on a dark disc (transparent corners).
fn procedural_icon() -> egui::IconData {
    let n: u32 = 64;
    let mut rgba = vec![0u8; (n * n * 4) as usize];
    let center = (n as f32 - 1.0) * 0.5;
    let acc = [0xE0_u8, 0xA0, 0x30];
    let acc2 = [0xFF_u8, 0xCB, 0x6B];
    let bg = [0x10_u8, 0x10, 0x15];
    for y in 0..n {
        for x in 0..n {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let d = (dx * dx + dy * dy).sqrt() / (n as f32 * 0.5);
            let i = ((y * n + x) * 4) as usize;
            if d > 1.0 {
                rgba[i + 3] = 0; // transparent outside the disc
                continue;
            }
            // concentric rings blending the two accents over the dark disc
            let ring = 0.5 + 0.5 * (d * 22.0).cos();
            let t = (ring * (1.0 - d)).clamp(0.0, 1.0);
            let blend = 0.5 + 0.5 * (d * 6.0).cos();
            let a0 = [
                (acc[0] as f32 * blend + acc2[0] as f32 * (1.0 - blend)),
                (acc[1] as f32 * blend + acc2[1] as f32 * (1.0 - blend)),
                (acc[2] as f32 * blend + acc2[2] as f32 * (1.0 - blend)),
            ];
            for k in 0..3 {
                rgba[i + k] = (bg[k] as f32 + (a0[k] - bg[k] as f32) * t) as u8;
            }
            rgba[i + 3] = 255;
        }
    }
    egui::IconData { rgba, width: n, height: n }
}

#[cfg(test)]
mod theme_tests;
