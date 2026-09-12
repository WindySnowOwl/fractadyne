//! Export & view-state I/O: render-to-file (PNG/EXR, foreground + background worker),
//! the reloadable view-metadata blob (embedded in exports, also used by bookmarks/.fdn),
//! and Open-view. (The gallery browser stays with its UI in main.rs.)

use crate::{
    separate_paths, stitch_side_by_side, version_string, ExportFormat, ExportJob, FractadyneApp,
    FractalKind,
};

/// A deep single-view export whose bignum reference orbit is being built off the main thread (it can
/// take minutes at extreme depth). Held while `rx` is pending; when the reference lands the request
/// is assembled (reusing it — no rebuild) and the render dispatched to a worker. Keeps the UI
/// responsive instead of freezing the whole app during the reference build.
pub(crate) struct ExportPrep {
    /// The deep MAP-view reference, building off-thread.
    pub rx: std::sync::mpsc::Receiver<crate::render::RecomputeResult>,
    pub map_vp: fractadyne_core::Viewport,
    /// Single-view Julia flag (the dual map is always Mandelbrot, so `false` there).
    pub julia_mode: bool,
    /// `Some` ⇒ dual export: the Julia panel to render + combine (usually shallow → builds instantly).
    pub julia_vp: Option<fractadyne_core::Viewport>,
    pub dual_mode: crate::DualExport,
    pub path: std::path::PathBuf,
}

/// Pre-rasterized "Fd" brand mark for stamping into exports: premultiplied **linear** RGBA at a
/// high resolution (downscaled per export). Built once on the main thread from the egui font atlas
/// (same glyphs as the live-view watermark), then blended by the export worker — which has no egui
/// context — via [`stamp_watermark`].
#[derive(Clone)]
pub(crate) struct WmOverlay {
    pub w: usize,
    pub h: usize,
    pub px: Vec<[f32; 4]>, // premultiplied, linear
}

/// Rasterize the "Fd" mark (F in brand text, d in amber) from the egui font atlas into a
/// premultiplied-linear overlay with a soft dark halo (so it stays legible on any background).
pub(crate) fn build_watermark_overlay(ctx: &egui::Context) -> Option<WmOverlay> {
    let ppp = ctx.pixels_per_point();
    let pts = 40.0_f32; // layout size in points; rasterized at pts*ppp texels, downscaled per export
    let galley =
        ctx.fonts(|f| f.layout_job(crate::theme::brand_mark_job(pts, crate::theme::BRAND_TEXT, crate::theme::BRAND_ACCENT)));
    let pad = (pts * ppp * 0.20).ceil() as i32;
    let gw = (galley.size().x * ppp).ceil() as i32;
    let gh = (galley.size().y * ppp).ceil() as i32;
    let w = (gw + 2 * pad).max(1) as usize;
    let h = (gh + 2 * pad).max(1) as usize;
    // Straight-alpha linear glyph layer: rgb = linear color, a = coverage.
    let mut glyph = vec![[0.0f32; 4]; w * h];
    ctx.fonts(|f| {
        let atlas = f.image();
        let aw = atlas.size[0];
        for row in &galley.rows {
            for g in &row.glyphs {
                let uv = g.uv_rect;
                if uv.max[0] <= uv.min[0] || uv.max[1] <= uv.min[1] {
                    continue;
                }
                let lin = egui::Rgba::from(
                    galley.job.sections.get(g.section_index as usize)
                        .map(|s| s.format.color).unwrap_or(egui::Color32::WHITE),
                );
                let ox = ((g.pos.x + uv.offset.x) * ppp).round() as i32 + pad;
                let oy = ((g.pos.y + uv.offset.y) * ppp).round() as i32 + pad;
                for ty in uv.min[1]..uv.max[1] {
                    for tx in uv.min[0]..uv.max[0] {
                        let cov = atlas.pixels[ty as usize * aw + tx as usize];
                        if cov <= 0.0 {
                            continue;
                        }
                        let dx = ox + (tx - uv.min[0]) as i32;
                        let dy = oy + (ty - uv.min[1]) as i32;
                        if dx < 0 || dy < 0 || dx >= w as i32 || dy >= h as i32 {
                            continue;
                        }
                        let p = &mut glyph[dy as usize * w + dx as usize];
                        if cov > p[3] {
                            *p = [lin.r(), lin.g(), lin.b(), cov];
                        }
                    }
                }
            }
        }
    });
    // Soft halo = box-blurred glyph coverage; composite the glyph over a translucent black halo.
    let radius = (pts * ppp * 0.09).round().max(1.0) as i32;
    let halo = blur_coverage(&glyph, w, h, radius);
    let mut px = vec![[0.0f32; 4]; w * h];
    for i in 0..w * h {
        let ga = glyph[i][3];
        let ha = (halo[i] * 0.60).min(1.0); // dark halo strength
        // glyph (premult) over black halo (premult rgb = 0): rgb = glyph.rgb*ga; a = ga + ha(1-ga)
        px[i] = [glyph[i][0] * ga, glyph[i][1] * ga, glyph[i][2] * ga, ga + ha * (1.0 - ga)];
    }
    Some(WmOverlay { w, h, px })
}

/// Separable box blur of the coverage (alpha) channel — the halo footprint.
fn blur_coverage(src: &[[f32; 4]], w: usize, h: usize, r: i32) -> Vec<f32> {
    let mut a: Vec<f32> = src.iter().map(|p| p[3]).collect();
    let mut tmp = vec![0.0f32; w * h];
    let norm = 1.0 / (2 * r + 1) as f32;
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            for k in -r..=r {
                let xx = (x as i32 + k).clamp(0, w as i32 - 1) as usize;
                s += a[y * w + xx];
            }
            tmp[y * w + x] = s * norm;
        }
    }
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            for k in -r..=r {
                let yy = (y as i32 + k).clamp(0, h as i32 - 1) as usize;
                s += tmp[yy * w + x];
            }
            a[y * w + x] = s * norm;
        }
    }
    a
}

/// Alpha-blend the pre-rasterized mark into the lower-right of a **linear** RGBA export buffer.
/// Height scales to ~2.6% of the image (matching the live view); area-averaged downscale.
pub(crate) fn stamp_watermark(pixels: &mut [f32], w: u32, h: u32, ov: &WmOverlay) {
    if ov.w == 0 || ov.h == 0 || w == 0 || h == 0 {
        return;
    }
    // Below h=80 the 16-px floor exceeds the 20% ceiling — clamp panics on min > max.
    // (Found by the v0.2.7 panic hook's first catch: a 64×36 CLI render.)
    let th_max = (h as f32) * 0.2;
    let th = (h as f32 * 0.026).clamp(16.0f32.min(th_max), th_max);
    let scale = th / ov.h as f32; // < 1 (downscale)
    let tw = (ov.w as f32 * scale).round() as i32;
    let th = th.round() as i32;
    if tw <= 0 || th <= 0 {
        return;
    }
    let margin_x = (w as f32 * 0.012).round() as i32;
    let margin_y = (h as f32 * 0.012).round() as i32;
    let x0 = w as i32 - tw - margin_x;
    let y0 = h as i32 - th - margin_y;
    let inv = ov.w as f32 / tw as f32; // source texels per dest pixel
    for dy in 0..th {
        for dx in 0..tw {
            // Area-average the source footprint for this dest pixel.
            let sx0 = (dx as f32 * inv) as i32;
            let sx1 = (((dx + 1) as f32 * inv) as i32).max(sx0 + 1).min(ov.w as i32);
            let sy0 = (dy as f32 * inv) as i32;
            let sy1 = (((dy + 1) as f32 * inv) as i32).max(sy0 + 1).min(ov.h as i32);
            let (mut acc, mut n) = ([0.0f32; 4], 0.0f32);
            for sy in sy0..sy1 {
                for sx in sx0..sx1 {
                    let s = ov.px[sy as usize * ov.w + sx as usize];
                    acc[0] += s[0]; acc[1] += s[1]; acc[2] += s[2]; acc[3] += s[3];
                    n += 1.0;
                }
            }
            if n == 0.0 {
                continue;
            }
            let src = [acc[0] / n, acc[1] / n, acc[2] / n, acc[3] / n]; // premultiplied linear
            let (px_, py) = (x0 + dx, y0 + dy);
            if px_ < 0 || py < 0 || px_ >= w as i32 || py >= h as i32 {
                continue;
            }
            let idx = (py as usize * w as usize + px_ as usize) * 4;
            let ia = 1.0 - src[3];
            pixels[idx] = src[0] + pixels[idx] * ia;
            pixels[idx + 1] = src[1] + pixels[idx + 1] * ia;
            pixels[idx + 2] = src[2] + pixels[idx + 2] * ia;
            // leave alpha channel (export is opaque)
        }
    }
}

/// Version of the reloadable view-metadata format (the `format_version=` field shared by
/// exports, `.fdn` locations and bookmarks). Bump ONLY on a breaking change to an existing
/// field's meaning or units — purely additive new keys don't need it (the allow-list reader
/// ignores unknown keys and defaults missing ones, so old and new builds interoperate).
/// A file whose `format_version` exceeds this is from a newer build: we still load the
/// fields we recognise, but warn the user that newer settings/semantics may not apply.
pub(crate) const VIEW_FORMAT_VERSION: u32 = 1;

/// Largest zoom depth (octaves = log2 of magnification) accepted from an untrusted view
/// file. Past this the bignum working precision (∝ octaves) would balloon into a memory
/// DoS, so a hostile/garbage `upp_log2` is clamped here. ~10× the deepest validated zoom
/// (`--validate-deep` reaches 1e1000000× ≈ 3.3e6 octaves), so no real location is affected.
const MAX_LOAD_OCTAVES: f64 = 3.4e7;

/// Upper bound on `max_iter` accepted from an untrusted view file (an absurd value would
/// make an export grind for hours / exhaust the iteration budget). Well above any real use.
const MAX_LOAD_ITER: u32 = 10_000_000;

/// Wall-clock budget for an export's multi-reference glitch-correction loop. Correction runs
// (The 120 s `GLITCH_CORRECT_BUDGET` wall clock lived here until v0.2.41-beta.11. A
// wall-clock cut made corrected output load-dependent — two runs of the same binary at
// e4000 differed by 3–101 bytes — so the bound is now WORK, admission-priced: see
// `render::CorrectionBudget` and the `GLITCH_CPU_BUDGET`/`GLITCH_GPU_BUDGET` tunables.)

/// Report from restoring view metadata, so callers can surface anything noteworthy
/// (a newer file format, clamped values, unrecognized fields) instead of loading silently.
#[derive(Default)]
pub(crate) struct ViewLoad {
    /// `Some(v)` if the file's `format_version` exceeds this build's (loaded best-effort).
    pub(crate) newer: Option<u32>,
    /// Human-readable names of fields whose value was out of range and clamped/rejected.
    pub(crate) clamped: Vec<&'static str>,
    /// Unrecognized keys present in the file (a typo, or a newer format's new fields).
    pub(crate) unknown: Vec<String>,
    /// ⭐**Positional complaints**: a line that is not a comment and carries no `=`, a value
    /// that is not the number its key requires, a duplicated key. Each names the LINE and
    /// COLUMN, because "invalid file" is not something a person can act on and "line 7, col 11:
    /// center_re is not a number" is.
    pub(crate) problems: Vec<String>,
    /// A summary of copy-and-paste damage that was repaired on the way in (smart quotes, a
    /// Unicode minus sign, a zero-width space). ⭐Reported even though the load SUCCEEDED — the
    /// user needs to know their clipboard is mangling data, because it will happen again.
    pub(crate) repairs: Option<String>,
    /// The text carried a BEGIN marker but no END: almost certainly a truncated paste.
    pub(crate) truncated: bool,
    /// Essential keys the text did not contain at all — what is MISSING, so a partial paste
    /// says what to go back for.
    pub(crate) missing: Vec<&'static str>,
}

impl ViewLoad {
    /// A short summary of anything noteworthy, or `None` when the load was fully clean.
    pub(crate) fn note(&self) -> Option<String> {
        let mut parts: Vec<String> = Vec::new();
        if let Some(v) = self.newer {
            parts.push(format!(
                "saved by a newer Fractadyne (format v{v}); some settings may not apply — consider updating"
            ));
        }
        if !self.clamped.is_empty() {
            parts.push(format!("clamped out-of-range {}", self.clamped.join(", ")));
        }
        if !self.unknown.is_empty() {
            parts.push(format!("ignored unknown field(s): {}", self.unknown.join(", ")));
        }
        // ⚠Ordered worst-first: a truncated paste explains every other complaint below it, so
        // it must not be buried under a list of missing fields it already accounts for.
        if self.truncated {
            parts.insert(
                0,
                "the text ends before the END marker — the paste looks truncated".to_string(),
            );
        }
        if !self.missing.is_empty() {
            parts.push(format!("missing: {}", self.missing.join(", ")));
        }
        if !self.problems.is_empty() {
            parts.push(self.problems.join("; "));
        }
        if let Some(r) = &self.repairs {
            parts.push(r.clone());
        }
        (!parts.is_empty()).then(|| parts.join("; "))
    }
}

/// Keys the view-metadata reader understands; anything else in a file is reported as unknown.
pub(crate) const KNOWN_VIEW_KEYS: &[&str] = &[
    "app", "version", "format_version", "saved_unix", "saved", "notes", "fractal", "julia",
    "julia_c_re", "julia_c_im", "center_re", "center_im", "upp", "upp_log2", "zoom", "max_iter",
    "auto_iter", "palette", "cycle", "offset", "aa", "palette_custom", "thumb", "checksum",
    // The centre's source EXPRESSION and its offset from that anchor, when the centre was entered
    // as a re-derivable expression. Optional and written IN ADDITION to `center_re`/`center_im`, so
    // an older reader still lands on the resolved decimal; a newer one re-derives at the view's
    // precision so a deeper zoom of the same file stays exact. See `CenterExpr`.
    "center_re_expr", "center_im_expr", "center_re_offset", "center_im_offset",
    // Pre-v0.2.20 spellings, still read. See `LEGACY_VIEW_KEYS`.
    "center_x", "center_y",
];

/// Without these a "view" is not a view. Their absence is reported by NAME, so a paste that
/// lost its tail says what to go back for instead of silently landing somewhere else.
pub(crate) const ESSENTIAL_VIEW_KEYS: &[&str] = &["center_re", "center_im"];

/// Keys a PREVIOUS version of Fractadyne wrote, mapped to their current names: `(old, new)`.
///
/// ⛔⭐⭐**A rename without an alias orphans every file already written.** `v0.2.20` moved the
/// centre from `center_x`/`center_y` to `center_re`/`center_im` — the right call, since Re/Im
/// is what the deep-zoom community and our own `.kfr` output use — but it changed the writer
/// AND the reader in one step and left nothing to read the old spelling. Every `.fdn`, and
/// every exported PNG and EXR, written before that release therefore loaded with its
/// COORDINATES SILENTLY DROPPED: zoom, palette and iteration count applied, and the view stayed
/// wherever it happened to be. Our own shipped sample location was one of them, unnoticed for
/// fifty-two releases, and it is the file the README points new users at.
///
/// ⭐**Read-only.** The writer emits the current names and only those; this exists so old data
/// keeps working, not so the format has two spellings going forward.
///
/// ⚠**The new name wins when both are present**, so a file carrying both (nothing we write,
/// but a hand-merged one might) is read as the version that wrote the newer key.
pub(crate) const LEGACY_VIEW_KEYS: &[(&str, &str)] =
    &[("center_x", "center_re"), ("center_y", "center_im")];

/// What each key's value has to BE, so an unreadable one is reported with a position instead of
/// being silently skipped.
///
/// ⛔⭐⭐**This is the `.and_then(|s| s.parse().ok())` trap, one layer out.** That idiom makes an
/// unreadable value indistinguishable from an absent one — the defect closed in `01add37` across
/// ~20 CLI options. Every numeric key here is still parsed that way at its own call site (so a
/// bad value keeps the current setting rather than aborting the load), and this table is what
/// makes the difference VISIBLE.
const TYPED_VIEW_KEYS: &[(&str, ValueKind)] = &[
    ("format_version", ValueKind::Uint),
    ("saved_unix", ValueKind::Uint),
    ("julia", ValueKind::Flag),
    ("auto_iter", ValueKind::Flag),
    ("julia_c_re", ValueKind::Float),
    ("julia_c_im", ValueKind::Float),
    ("center_re", ValueKind::BigFloat),
    ("center_im", ValueKind::BigFloat),
    ("center_re_expr", ValueKind::Expr),
    ("center_im_expr", ValueKind::Expr),
    ("center_re_offset", ValueKind::BigFloat),
    ("center_im_offset", ValueKind::BigFloat),
    // The pre-v0.2.20 spellings get the same scrutiny as the current ones.
    ("center_x", ValueKind::BigFloat),
    ("center_y", ValueKind::BigFloat),
    ("upp", ValueKind::Float),
    ("upp_log2", ValueKind::Float),
    ("max_iter", ValueKind::Uint),
    ("palette", ValueKind::Uint),
    ("cycle", ValueKind::Float),
    ("offset", ValueKind::Float),
    ("aa", ValueKind::Uint),
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum ValueKind {
    Uint,
    Float,
    /// `0` or `1`.
    Flag,
    /// A full-precision decimal, read by `parse_bf`.
    BigFloat,
    /// A coordinate expression (a rational, or one built from `cos`/`sin`/`pi`/`sqrt`/…), read by
    /// `parse_bf` — which also accepts a plain decimal, so this is a strict superset of `BigFloat`
    /// that just reports itself differently when unreadable.
    Expr,
}

impl ValueKind {
    fn accepts(self, v: &str) -> bool {
        match self {
            // ⚠`f64` accepts "inf" and "NaN"; those are hostile values, not unreadable ones, and
            // the clamping at each call site is what deals with them. Shape only, here.
            ValueKind::Uint => v.parse::<u64>().is_ok(),
            ValueKind::Float => v.parse::<f64>().is_ok(),
            ValueKind::Flag => v == "0" || v == "1",
            ValueKind::BigFloat | ValueKind::Expr => fractadyne_core::parse_bf(v).is_some(),
        }
    }

    fn describe(self) -> &'static str {
        match self {
            ValueKind::Uint => "a whole number",
            ValueKind::Float => "a number",
            ValueKind::Flag => "0 or 1",
            ValueKind::BigFloat => "a decimal number",
            ValueKind::Expr => "a coordinate expression",
        }
    }
}

/// The line that opens a shareable view, and the one that closes it.
///
/// ⭐⭐**They exist for the clipboard, not the parser.** A location travels through forum posts
/// and chat messages, where it gets selected by hand and arrives with the first or last line
/// missing. The markers tell a person exactly what to select, and let the reader say "this is
/// truncated" instead of loading a half-view and leaving them somewhere unexplained.
///
/// ⚠⚠**No `=` in either marker.** A reader older than this format finds no `=` on the line and
/// skips it; put an `=` in and every one of those builds reports a bogus unknown key instead.
pub(crate) const VIEW_BEGIN_MARKER: &str = "# ----- BEGIN FRACTADYNE VIEW -----";
pub(crate) const VIEW_END_MARKER: &str = "# ----- END FRACTADYNE VIEW -----";

/// Whether a view's `checksum` field agrees with the data beside it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum ChecksumState {
    /// No `checksum` field — every file written before the field existed, and every view read
    /// out of a PNG chunk. ⭐Not a failure: absence is the norm for older files and must stay
    /// silent, or the warning becomes noise and gets ignored.
    #[default]
    Absent,
    Match,
    Mismatch {
        found: String,
        computed: String,
    },
}

/// The digest a view carries, over a CANONICAL form of its fields.
///
/// ⭐⭐**Canonical, NOT the raw bytes, and that distinction is the whole design.** A location
/// travels by clipboard: through chat clients that reflow whitespace, forum software that
/// rewrites line endings, editors that add or remove a trailing newline. Hashing the raw text
/// would flag every one of those as corruption — and a checksum that cries wolf is worse than
/// none, because people learn to click through it. So the digest is taken over the FIELDS:
/// sorted by key, value trimmed, joined with a newline. Whitespace, key order, comments, line
/// endings and the markers themselves can all change without disturbing it, while a changed
/// digit or a dropped line cannot.
///
/// ⚠**`thumb` is excluded**, and not to save time: a thumbnail is the one field a person might
/// reasonably strip to shorten a paste, and a view that still describes the right place should
/// not be called corrupt for having lost its picture. The thumbnail does not need covering
/// anyway — it is a PNG, and every PNG chunk already carries its own CRC-32, so a damaged one
/// fails to decode and is simply not shown.
///
/// ⚠**`checksum` excludes itself**, obviously, or it could never be computed.
///
/// ⛔**FNV-1a, and the threat model is ACCIDENT.** Truncated pastes, a dropped character, a
/// mangled encoding. It is not a signature and must never be described as one: anyone editing a
/// view by hand can recompute it. A cryptographic hash would need a dependency and would still
/// not make the file trustworthy, because the file was never signed.
pub(crate) fn view_digest<'a>(fields: impl Iterator<Item = (&'a str, &'a str)>) -> String {
    let mut pairs: Vec<(&str, &str)> = fields
        .filter(|(k, _)| *k != "checksum" && *k != "thumb")
        .collect();
    pairs.sort_unstable();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let byte = |b: u8, h: &mut u64| {
        *h ^= b as u64;
        *h = h.wrapping_mul(0x0000_0100_0000_01b3);
    };
    for (k, v) in pairs {
        for b in k.as_bytes() {
            byte(*b, &mut h);
        }
        byte(b'=', &mut h);
        for b in v.as_bytes() {
            byte(*b, &mut h);
        }
        byte(b'\n', &mut h);
    }
    format!("{h:016x}")
}

/// Split view text into `(key, value)` pairs the same way the reader does — comments and
/// markers skipped, values trimmed.
///
/// ⭐**Shared by the writer and the verifier deliberately.** Two implementations of "what counts
/// as a field" would drift, and the symptom would be every file reporting itself corrupt.
pub(crate) fn view_field_pairs(meta: &str) -> Vec<(&str, &str)> {
    fractadyne_text::numbered_lines(meta)
        .into_iter()
        .filter_map(|(_, raw)| {
            let t = raw.trim();
            if t.is_empty() || t.starts_with('#') {
                return None;
            }
            raw.split_once('=').map(|(k, v)| (k.trim(), v.trim()))
        })
        .collect()
}

/// Check a view's checksum WITHOUT applying it — so a caller can ask before changing anything.
///
/// ⚠⚠**The clipboard repair runs first, exactly as it does on load.** If a chat client turned a
/// hyphen into a Unicode minus sign, `clean` puts it back and the digest MATCHES — which is
/// right: the data survived. Only damage that could not be repaired reaches the comparison.
pub(crate) fn view_checksum_state(meta: &str) -> ChecksumState {
    let cleaned = fractadyne_text::clean(meta);
    let pairs = view_field_pairs(&cleaned.text);
    let Some((_, found)) = pairs.iter().find(|(k, _)| *k == "checksum") else {
        return ChecksumState::Absent;
    };
    let computed = view_digest(pairs.iter().copied());
    if found.eq_ignore_ascii_case(&computed) {
        ChecksumState::Match
    } else {
        ChecksumState::Mismatch { found: (*found).to_string(), computed }
    }
}

/// Wrap bare view metadata as a shareable document: guidance, markers, and a checksum.
///
/// ⭐**Only for the surfaces a HUMAN copies from** — the `.fdn` file and the Share dialog. The
/// copy embedded in a PNG or EXR stays bare: nobody selects it by hand, so the markers would be
/// bytes in every exported image for no one's benefit.
/// Decode a view's embedded thumbnail to `(width, height, rgba8)`, or `None` if it has none.
///
/// ⚠**Every failure is a `None`, never a panic or a placeholder.** This runs over files a user
/// dropped in a folder: a truncated base64 value, a corrupt PNG, a thumbnail from a future
/// version. The gallery simply shows that entry without a picture, which is exactly what it
/// already does for an image it cannot decode.
///
/// ⭐The PNG carries a CRC-32 per chunk, so a damaged thumbnail fails HERE rather than being
/// drawn as garbage — which is why the view checksum deliberately does not cover this field.
pub(crate) fn decode_embedded_thumbnail(meta: &str) -> Option<(u32, u32, Vec<u8>)> {
    let cleaned = fractadyne_text::clean(meta);
    let (_, b64) = view_field_pairs(&cleaned.text)
        .into_iter()
        .find(|(k, _)| *k == "thumb")?;
    let png = fractadyne_text::base64::decode(b64)?;
    fractadyne_export::read_png_rgba8_bytes(&png).ok()
}

/// Does this metadata blob look like a VIEW at all?
///
/// ⛔⭐⭐**A PNG `tEXt` chunk under our keyword is not necessarily a view.** The golden images
/// use the same `Fractadyne` keyword to store the command line that reproduces them, which is the
/// right thing for a golden and indistinguishable from a view to `read_png_metadata`. Without this
/// check, opening one reported "missing: center_re, center_im" — a complaint about a file that
/// was never claiming to be a location.
///
/// ⭐The rule is the gallery's own, which already had to solve this: the blob must carry
/// `app=Fractadyne`.
pub(crate) fn looks_like_a_view(meta: &str) -> bool {
    view_field_pairs(meta)
        .into_iter()
        .any(|(k, v)| k == "app" && v == "Fractadyne")
}

/// Parse and DIAGNOSE view text without applying any of it.
///
/// ⭐⭐**Split out so a file can be checked without moving the view.** `load_view_metadata`
/// jumps the camera and records history as it parses, which makes it useless for answering
/// "would this file load cleanly?" — the question a shipped-data gate has to ask of every
/// location in the repo.
///
/// Returns the report so far and the parsed fields as `(key, value, line, value column)`.
/// ⚠`clamped` stays empty: clamping is something APPLYING does, and this does not apply.
pub(crate) fn inspect_view_text(meta: &str) -> (ViewLoad, Vec<(String, String, usize, usize)>) {
    let mut report = ViewLoad::default();
    // ⭐⭐**Repair the clipboard first, parse second.** A location that has been through a chat
    // client or a word processor comes back with a Unicode minus sign where a hyphen was, or a
    // zero-width space inside a number — invisible changes that make a perfectly good
    // coordinate unparseable. `clean` also normalizes CR / CRLF / LF, so a file saved on any
    // platform (or mangled by a transfer that rewrote endings) reads the same.
    let cleaned = fractadyne_text::clean(meta);
    report.repairs = cleaned.summary();
    let meta = cleaned.text.as_str();

    // One pass, keeping WHERE each field came from. The old reader re-scanned every line for
    // every key and kept no positions, so it could not say anything about a bad line beyond
    // ignoring it.
    let mut fields: Vec<(String, String, usize, usize)> = Vec::new();
    let (mut saw_begin, mut saw_end) = (false, false);
    for (n, raw) in fractadyne_text::numbered_lines(meta) {
        let t = raw.trim();
        if t.is_empty() {
            continue;
        }
        if t.starts_with('#') {
            // ⚠Check BEGIN first: neither marker is a substring of the other, but relying on
            // that silently is how a renamed marker becomes a mystery.
            if t.contains("BEGIN FRACTADYNE VIEW") {
                saw_begin = true;
            } else if t.contains("END FRACTADYNE VIEW") {
                saw_end = true;
            }
            continue;
        }
        let Some((k, v)) = raw.split_once('=') else {
            // ⚠Named, not silently dropped — this is usually a wrapped long line from a paste,
            // and the user can only fix what they are told about.
            if report.problems.len() < 8 {
                let shown: String = t.chars().take(32).collect();
                report.problems.push(format!(
                    "line {n}: no \u{27}=\u{27} and not a comment, ignored ({shown:?})"
                ));
            }
            continue;
        };
        let key = k.trim().to_string();
        // 1-based CHARACTER column of the first character of the value — what an editor shows,
        // which a byte offset is not.
        let col = k.chars().count() + 1 + v.chars().take_while(|c| c.is_whitespace()).count() + 1;
        if fields.iter().any(|(existing, _, _, _)| *existing == key) && report.problems.len() < 8
        {
            report.problems.push(format!(
                "line {n}: {key:?} appears more than once; the first one is used"
            ));
        }
        fields.push((key, v.trim().to_string(), n, col));
    }
    // ⚠⚠BEGIN with no END is the shape a truncated paste takes. No BEGIN at all is NOT a
    // problem: bare metadata is what lives in a PNG chunk and in every file written before the
    // markers existed.
    report.truncated = saw_begin && !saw_end;

    // ⭐⭐**Fold the pre-v0.2.20 spellings in HERE**, before anything else looks at `fields`, so
    // every consumer — the essential-key check, the apply stage, the shipped-file gate — sees
    // one vocabulary. Doing it at each read site instead is how one of them gets missed.
    for (old, new) in LEGACY_VIEW_KEYS {
        let has_new = fields.iter().any(|(k, _, _, _)| k == new);
        if has_new {
            continue;
        }
        if let Some((_, v, line, col)) = fields.iter().find(|(k, _, _, _)| k == old).cloned() {
            fields.push(((*new).to_string(), v, line, col));
        }
    }

    let field = |key: &str| fields.iter().find(|(k, _, _, _)| k == key);

    // ⭐Every typed key that is PRESENT but unreadable, named with its position. The apply
    // stage still uses `.parse().ok()` and keeps the current value on failure — this is what
    // stops that being invisible.
    for (key, kind) in TYPED_VIEW_KEYS {
        if let Some((_, v, line, col)) = field(key) {
            if !kind.accepts(v) && report.problems.len() < 8 {
                let shown: String = v.chars().take(24).collect();
                // A coordinate that failed to evaluate says WHY (and where in the value) — the
                // same message the Go-to dialog gives, so a hand-edited file is as diagnosable.
                let why = match kind {
                    ValueKind::BigFloat | ValueKind::Expr => fractadyne_core::parse_real_expr(v, 0)
                        .err()
                        .map(|e| format!(" — {e}"))
                        .unwrap_or_default(),
                    _ => String::new(),
                };
                report.problems.push(format!(
                    "line {line}, col {col}: {key} needs {} but found {shown:?}{why}",
                    kind.describe()
                ));
            }
        }
    }
    for key in ESSENTIAL_VIEW_KEYS {
        if field(key).is_none() {
            report.missing.push(key);
        }
    }

    // A file with no `format_version` predates the field but is format-1 compatible.
    let file_ver = field("format_version")
        .and_then(|(_, v, _, _)| v.parse::<u32>().ok())
        .unwrap_or(VIEW_FORMAT_VERSION);
    report.newer = (file_ver > VIEW_FORMAT_VERSION).then_some(file_ver);

    // Keys we do not recognize (capped, so a junk file cannot flood the report).
    for (k, _, line, _) in &fields {
        if !k.is_empty()
            && !KNOWN_VIEW_KEYS.contains(&k.as_str())
            && !report.unknown.iter().any(|u| u.starts_with(k.as_str()))
        {
            report.unknown.push(format!("{k} (line {line})"));
            if report.unknown.len() >= 8 {
                break;
            }
        }
    }
    (report, fields)
}

pub(crate) fn wrap_view_text(bare: &str) -> String {
    let digest = view_digest(view_field_pairs(bare).into_iter());
    let mut out = String::with_capacity(bare.len() + 320);
    out.push_str("# Fractadyne view. Open it with File \u{25B8} Open view, or paste it into\n");
    out.push_str("# File \u{25B8} Share location. Copy the BEGIN and END lines too.\n");
    out.push_str("# Lines starting with # are comments and are ignored.\n");
    out.push_str(VIEW_BEGIN_MARKER);
    out.push('\n');
    out.push_str(bare);
    if !bare.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("# Detects accidental damage in transit; it is not a signature.\n");
    out.push_str(&format!("checksum={digest}\n"));
    out.push_str(VIEW_END_MARKER);
    out.push('\n');
    out
}

/// The largest embedded gradient, shared by the writer and the reader. ⭐**One constant, both
/// ends** — a writer that emits more than the reader accepts produces files that only fail on
/// somebody else's machine.
pub(crate) const MAX_EMBEDDED_SEGMENTS: usize = 512;

/// Encode the custom gradient as ONE Latin-1, single-line field for view metadata.
///
/// ⭐⭐**A `.fdn` that restores the view but not the colours does not restore the IMAGE**, which is
/// the thing people share. The view fields have always travelled; the gradient did not, so a
/// carefully built palette arrived as whatever preset the recipient happened to be on.
///
/// ⚠**One line, ASCII, and no `=`** — this rides in PNG `tEXt` beside the view fields, which is a
/// Latin-1 key=value format. Segments are separated by `;` and fields by `,`.
///
/// ⛔**The blend/space numbers are the SAME FILE FORMAT as `.ggr` and the session** — GIMP's
/// numbering, append-only. They go through `as_u8`, never a cast, for the reason recorded on
/// `PaletteSegment`: renumbering silently re-interprets every saved gradient.
///
/// ⭐**Plain `{}` on the floats, not a fixed precision.** Rust's `Display` for `f32` emits the
/// shortest decimal that parses back to the identical bits, so this is both exact and shorter than
/// any `{:.N}` that would also be exact. A fixed `{:.9}` was the first attempt and it is subtly
/// worse: it is exact only while every value stays inside `0..1`.
fn encode_palette_segments(segs: &[fractadyne_state::PaletteSegment]) -> String {
    let mut out = String::new();
    for (i, s) in segs.iter().enumerate() {
        if i > 0 {
            out.push(';');
        }
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            s.left, s.mid, s.right,
            s.left_color[0], s.left_color[1], s.left_color[2], s.left_color[3],
            s.right_color[0], s.right_color[1], s.right_color[2], s.right_color[3],
            s.blend, s.space,
            s.blend_params[0], s.blend_params[1], s.blend_params[2], s.blend_params[3],
        ));
    }
    out
}

/// Decode [`encode_palette_segments`]. Untrusted input: a malformed segment is DROPPED rather than
/// defaulted, and the whole field is refused unless what survives still covers `0..1` in order — a
/// half-parsed gradient would render as a colour nobody chose.
fn decode_palette_segments(v: &str) -> Option<Vec<fractadyne_state::PaletteSegment>> {
    let mut out: Vec<fractadyne_state::PaletteSegment> = Vec::new();
    for part in v.split(';').filter(|p| !p.trim().is_empty()) {
        let f: Vec<&str> = part.split(',').map(|t| t.trim()).collect();
        if f.len() != 17 {
            return None;
        }
        let n: Vec<f32> = f.iter().map(|t| t.parse::<f32>().unwrap_or(f32::NAN)).collect();
        if n.iter().any(|x| !x.is_finite()) {
            return None;
        }
        out.push(fractadyne_state::PaletteSegment {
            left: n[0], mid: n[1], right: n[2],
            left_color: [n[3], n[4], n[5], n[6]],
            right_color: [n[7], n[8], n[9], n[10]],
            blend: f[11].parse::<u8>().ok()?,
            space: f[12].parse::<u8>().ok()?,
            blend_params: [n[13], n[14], n[15], n[16]],
        });
    }
    // ⚠Shape, not just syntax: covering 0..1 in order is what `Gradient::eval` assumes, and a
    // gradient that does not is a black band nobody asked for.
    if out.is_empty() || out.len() > MAX_EMBEDDED_SEGMENTS {
        return None;
    }
    if (out[0].left - 0.0).abs() > 1.0e-4 || (out[out.len() - 1].right - 1.0).abs() > 1.0e-4 {
        return None;
    }
    if out.windows(2).any(|w| (w[1].left - w[0].right).abs() > 1.0e-4) {
        return None;
    }
    if out.iter().any(|s| s.right <= s.left) {
        return None;
    }
    // ⚠The midpoint too: `Gradient::eval` divides by the distance to it, and a `mid` outside
    // its own segment is the one malformed shape the checks above still let through.
    if out.iter().any(|s| s.mid < s.left || s.mid > s.right) {
        return None;
    }
    Some(out)
}

impl FractadyneApp {
    /// Reloadable view-state metadata embedded in exports. The center is stored as
    /// full-precision decimal so deep-zoom positions round-trip exactly. `center_re`/`center_im`
    /// name the complex plane's real/imaginary axes (the domain convention, matching Fraktaler-3
    /// and Kalles Fraktaler); they hold the viewport's geometric `center_x`/`center_y`.
    pub(crate) fn view_metadata(&self) -> String {
        let (jcx, jcy) = self.julia_c;
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        // Latin-1 / single-line safe notes (PNG tEXt), max 120 chars.
        let notes: String = self
            .export.notes
            .chars()
            .filter(|c| !c.is_control() && (*c as u32) <= 0xFF)
            .take(120)
            .collect();
        format!(
            "app=Fractadyne\nversion={}\nformat_version={}\nsaved_unix={}\nsaved={}\n\
             notes={}\nfractal={}\njulia={}\njulia_c_re={:.17e}\njulia_c_im={:.17e}\n\
             center_re={}\ncenter_im={}\nupp={:.17e}\nupp_log2={:.17e}\nzoom={}\nmax_iter={}\nauto_iter={}\n\
             palette={}\ncycle={}\noffset={}\naa={}\n{}{}",
            version_string(),
            VIEW_FORMAT_VERSION,
            secs,
            Self::utc_date_string(secs),
            notes,
            self.fractal.name(),
            self.julia_mode as u32,
            jcx,
            jcy,
            fractadyne_core::to_decimal_string(&self.viewport.center_x),
            fractadyne_core::to_decimal_string(&self.viewport.center_y),
            self.viewport.units_per_pixel.to_f64(),
            // Extended-range scale (log2 of units_per_pixel) so deep (>1e308×) views reload
            // exactly; `upp` above is the saturating f64 (back-compat + human-readable).
            self.viewport.units_per_pixel.log2(),
            // Scientific string valid past f64 range — `magnification()` saturates to `inf` past
            // ~1e308×, which showed as `zoom=inf` in deep `.fdn`/bookmarks. Informational only
            // (loads use `upp_log2`); readers get a real value like `4.84e838`.
            crate::fmt_zoom_field(self.viewport.log2_magnification()),
            self.render_cfg.max_iter,
            self.render_cfg.auto_iter as u32,
            self.coloring.palette_idx,
            self.coloring.cycle,
            self.coloring.offset,
            self.render_cfg.aa,
            // The custom gradient, when there is one. Empty otherwise, so a preset view's
            // metadata is byte-identical to what it was before this field existed.
            if self.coloring.use_custom_palette
                && !self.coloring.custom_segments.is_empty()
                // ⚠⚠**The same cap the READER enforces.** Without it a gradient past the limit
                // would be written into every export and refused by our own loader — a field that
                // is always there and never works is worse than one that is absent, because the
                // absence is at least honest. Real gradients are well under this; an importer
                // producing hundreds of segments is the case that would trip it.
                && self.coloring.custom_segments.len() <= MAX_EMBEDDED_SEGMENTS
            {
                format!(
                    "palette_custom={}
",
                    encode_palette_segments(&self.coloring.custom_segments)
                )
            } else {
                String::new()
            },
            // The centre's source expression + offset, when it was entered as one (empty otherwise,
            // so an ordinary view's metadata is byte-identical to before this existed).
            self.center_expr_metadata(),
        )
    }

    /// The centre's source expression and its offset from that anchor — the `center_re_expr` /
    /// `center_im_expr` (+ `center_re_offset` / `center_im_offset`) lines — when the centre was
    /// entered as a re-derivable expression. Empty otherwise.
    ///
    /// Written IN ADDITION to the resolved `center_re`/`center_im`: an older reader (or one that
    /// never had an expression) still lands on the decimal centre, while a current reader
    /// reconstructs `expression(re-derived at the view's precision) + offset`, so a deeper zoom of
    /// the reloaded file stays exact instead of freezing at the digits the decimal was written with.
    /// The offset is zero while the centre still sits on the point and small while exploring near it;
    /// past the set's own ~8-unit span it cannot be a meaningful offset from the anchor (a discrete
    /// jump we failed to clear), so the expression is dropped and only the plain decimal is written.
    fn center_expr_metadata(&self) -> String {
        let Some(ce) = self.center_expr.as_ref() else {
            return String::new();
        };
        let p = self.viewport.precision.max(ce.prec);
        let (Some(anchor_re), Some(anchor_im)) = (
            fractadyne_core::parse_bf_prec(&ce.re, p),
            fractadyne_core::parse_bf_prec(&ce.im, p),
        ) else {
            return String::new();
        };
        let off_re = fractadyne_core::bf_sub(&self.viewport.center_x, &anchor_re, p);
        let off_im = fractadyne_core::bf_sub(&self.viewport.center_y, &anchor_im, p);
        if fractadyne_core::to_f64(&off_re).abs() >= 8.0 || fractadyne_core::to_f64(&off_im).abs() >= 8.0 {
            return String::new();
        }
        let mut s = format!("center_re_expr={}\ncenter_im_expr={}\n", ce.re, ce.im);
        if !off_re.is_zero() || !off_im.is_zero() {
            s.push_str(&format!(
                "center_re_offset={}\ncenter_im_offset={}\n",
                fractadyne_core::to_decimal_string(&off_re),
                fractadyne_core::to_decimal_string(&off_im),
            ));
        }
        s
    }

    /// Restore the view from view-state metadata (exported image, `.fdn`, or bookmark).
    /// Untrusted input: every field is allow-listed, parsed leniently, and clamped to a
    /// safe range; unknown keys are ignored and missing keys keep their current value.
    /// Returns whether the file's `format_version` is within this build's range so callers
    /// can warn on a forward-incompatible (newer) file.
    pub(crate) fn load_view_metadata(&mut self, meta: &str) -> ViewLoad {
        // ⭐Parse and diagnose first, apply second — see `inspect_view_text`.
        let (mut report, fields) = inspect_view_text(meta);
        let field = |key: &str| fields.iter().find(|(k, _, _, _)| k == key);
        let get = |key: &str| -> Option<String> { field(key).map(|(_, v, _, _)| v.clone()) };
        let file_ver = report.newer.unwrap_or(VIEW_FORMAT_VERSION);
        if let Some(f) = get("fractal").and_then(|s| FractalKind::from_name(&s)) {
            self.fractal = f;
        }
        self.julia_mode =
            get("julia").map(|s| s == "1").unwrap_or(false) && self.fractal.supports_julia();
        if let (Some(re), Some(im)) = (
            get("julia_c_re").and_then(|s| s.parse().ok()),
            get("julia_c_im").and_then(|s| s.parse().ok()),
        ) {
            self.julia_c = (re, im);
        }
        if let Some(cx) = get("center_re").and_then(|s| fractadyne_core::parse_bf(&s)) {
            self.viewport.center_x = cx;
        }
        if let Some(cy) = get("center_im").and_then(|s| fractadyne_core::parse_bf(&s)) {
            self.viewport.center_y = cy;
        }
        // Prefer the extended-range `upp_log2` (exact past 1e308×); fall back to the f64
        // `upp` for images saved before it existed. Clamp the depth so a hostile value
        // can't blow up the bignum working precision (memory DoS).
        if let Some(raw) = get("upp_log2").and_then(|s| s.parse::<f64>().ok()) {
            let l = raw.clamp(-MAX_LOAD_OCTAVES, MAX_LOAD_OCTAVES);
            if !raw.is_finite() || l != raw {
                report.clamped.push("zoom depth");
            }
            self.viewport.units_per_pixel = fractadyne_core::FloatExp::from_f64(1.0).mul_pow2(l);
        } else if let Some(raw) = get("upp").and_then(|s| s.parse::<f64>().ok()) {
            if raw.is_finite() && raw > 0.0 {
                let l = raw.log2().clamp(-MAX_LOAD_OCTAVES, MAX_LOAD_OCTAVES);
                if l != raw.log2() {
                    report.clamped.push("zoom depth");
                }
                self.viewport.units_per_pixel = fractadyne_core::FloatExp::from_f64(1.0).mul_pow2(l);
            } else {
                report.clamped.push("zoom depth");
            }
        } else if get("zoom").is_some() {
            // A file carrying ONLY `zoom` gets no depth at all: the view stays wherever it was,
            // which for a fresh load is the whole set. `zoom` is written for humans to read and
            // is an f64 — it cannot even represent the depths this app reaches, so it is not a
            // fallback. A hand-written location at 1e500x silently rendered the full Mandelbrot
            // until this said so (2026-08-29); the depth belongs in `upp_log2`.
            report.clamped.push("zoom (ignored — depth is read from upp_log2, not zoom)");
        }
        if let Some(mi) = get("max_iter").and_then(|s| s.parse::<u32>().ok()) {
            let c = mi.clamp(1, MAX_LOAD_ITER);
            if c != mi {
                report.clamped.push("max_iter");
            }
            self.render_cfg.max_iter = c;
        }
        if let Some(ai) = get("auto_iter") {
            self.render_cfg.auto_iter = ai == "1";
        }
                // ⭐⭐**The custom gradient, if the file carries one.** Restored BEFORE the preset index
        // below so a file with both lands on the gradient it was saved with, and only falls back
        // to the preset when the gradient is absent or does not parse.
        //
        // ⚠Untrusted: `decode_palette_segments` refuses anything that does not cover 0..1 in
        // order, so a truncated or hostile field leaves the current palette alone rather than
        // rendering a colour nobody chose.
        if let Some(segs) = get("palette_custom").and_then(|v| decode_palette_segments(&v)) {
            self.coloring.custom_segments = segs;
            self.coloring.custom_palette_flat = false;
            self.coloring.use_custom_palette = true;
            // ⚠Keep the derived stop list in step, exactly as every other path that installs
            // segments does — a loaded gradient must be indistinguishable from an edited one.
            let g = crate::segments_to_gradient(
                "Loaded gradient",
                &self.coloring.custom_segments.clone(),
            );
            self.store_segments(&g);
        } else if get("palette_custom").is_some() {
            report.clamped.push("custom palette");
        }
        if let Some(p) = get("palette").and_then(|s| s.parse::<usize>().ok()) {
            if p < fractadyne_color::PRESETS.len() {
                self.coloring.palette_idx = p;
            } else {
                report.clamped.push("palette");
            }
        }
        if let Some(c) = get("cycle").and_then(|s| s.parse::<f32>().ok()) {
            if c.is_finite() {
                let v = c.clamp(0.0, 1.0e6);
                if v != c {
                    report.clamped.push("cycle");
                }
                self.coloring.cycle = v;
            } else {
                report.clamped.push("cycle");
            }
        }
        if let Some(o) = get("offset").and_then(|s| s.parse::<f32>().ok()) {
            if o.is_finite() {
                let v = o.clamp(-1.0e6, 1.0e6);
                if v != o {
                    report.clamped.push("offset");
                }
                self.coloring.offset = v;
            } else {
                report.clamped.push("offset");
            }
        }
        if let Some(a) = get("aa").and_then(|s| s.parse::<u32>().ok()) {
            let c = a.clamp(1, 16);
            if c != a {
                report.clamped.push("anti-aliasing");
            }
            self.render_cfg.aa = c;
        }
        if let Some(n) = get("notes") {
            self.export.notes = n;
        }
        // Match the viewport's working precision to the restored zoom; drop caches.
        self.viewport.precision = fractadyne_core::precision_for_octaves(
            self.viewport.log2_magnification().max(0.0).ceil() as u64,
        );
        // ⭐If the file carries the centre's SOURCE EXPRESSION, re-derive it at that precision and
        // re-apply the saved offset — overriding the decimal `center_re`/`center_im` read above — so
        // the centre is exact for THIS depth and for any deeper zoom from here, not frozen at the
        // digits the decimal was written with. Read after the depth so the precision is known; a
        // file with no expression keys is unchanged. See `CenterExpr` / `center_expr_metadata`.
        self.center_expr = None;
        if let (Some(re_expr), Some(im_expr)) = (get("center_re_expr"), get("center_im_expr")) {
            let target = self.viewport.precision + 64;
            if let (Some(anchor_re), Some(anchor_im)) = (
                fractadyne_core::parse_bf_prec(&re_expr, target),
                fractadyne_core::parse_bf_prec(&im_expr, target),
            ) {
                let off_re = get("center_re_offset")
                    .and_then(|s| fractadyne_core::parse_bf_prec(&s, target))
                    .unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(0.0, target));
                let off_im = get("center_im_offset")
                    .and_then(|s| fractadyne_core::parse_bf_prec(&s, target))
                    .unwrap_or_else(|| fractadyne_core::BigFloat::from_f64(0.0, target));
                self.viewport.center_x = fractadyne_core::bf_add(&anchor_re, &off_re, target);
                self.viewport.center_y = fractadyne_core::bf_add(&anchor_im, &off_im, target);
                self.center_expr =
                    Some(crate::CenterExpr::loaded(re_expr, im_expr, anchor_re, anchor_im, target));
            }
        }
        self.invalidate_refs();
        self.pointer.zoom_vel = 0.0;
        self.record_nav();
        // ⚠Unknown keys and `newer` are already in `report` — `inspect_view_text` produced them.
        // A second pass here would list every unrecognized key twice.
        let _ = file_ver;
        report
    }

    /// Open any Fractadyne view or shared location and jump to it (native dialog). Accepts an
    /// exported **PNG/EXR** (view restored from its embedded metadata), a **`.fdn`** share-location
    /// file, or a Kalles Fraktaler **`.kfr`** location — dispatched by extension. This is the single
    /// discoverable entry point; `.fdn` no longer needs the Share-location dialog.
    /// Load view text — unless its checksum says it arrived damaged, in which case park it and
    /// let the user decide.
    ///
    /// ⭐⭐**The check happens BEFORE anything is applied.** `load_view_metadata` jumps the view
    /// and records history as it parses; asking afterwards would mean asking about a move that
    /// had already happened.
    ///
    /// ⚠A file with no checksum loads normally and silently. Every view written before the
    /// field existed, and every view read out of a PNG chunk, is in that case — warning about
    /// them would make the warning worthless.
    ///
    /// Returns `None` when the load was deferred to the prompt.
    pub(crate) fn load_view_checked(&mut self, text: &str, source: String) -> Option<ViewLoad> {
        if let ChecksumState::Mismatch { found, computed } = view_checksum_state(text) {
            self.dialogs.pending_view = Some(crate::PendingView {
                text: text.to_string(),
                source,
                found,
                computed,
            });
            return None;
        }
        Some(self.load_view_metadata(text))
    }

    pub(crate) fn open_view(&mut self, ctx: &egui::Context) {
        let path = rfd::FileDialog::new()
            .add_filter("Fractadyne view or location", &["png", "exr", "fdn", "kfr"])
            .add_filter("Image (PNG / EXR)", &["png", "exr"])
            .add_filter("Location (.fdn / .kfr)", &["fdn", "kfr"])
            .set_directory(self.dialog_dir(Self::pictures_dir))
            .pick_file();
        let Some(path) = path else { return };
        self.remember_dir(&path);
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        match ext.as_str() {
            // Shared location: a plain-text view-metadata blob (same format embedded in exports).
            "fdn" => match std::fs::read(&path) {
                Ok(bytes) if bytes.len() <= crate::SHARE_MAX => match String::from_utf8(bytes) {
                    Ok(t) if crate::location_text_verdict(&t).is_ok() => {
                        // `None` means the checksum failed and the prompt has it now.
                        if let Some(report) =
                            self.load_view_checked(&t, path.display().to_string())
                        {
                            let zoom =
                                crate::fmt_zoom_log2(self.viewport.log2_magnification());
                            self.set_toast(
                                match report.note() {
                                    None => format!("Loaded location @ {zoom}×"),
                                    Some(n) => format!("Loaded @ {zoom}× — {n}"),
                                },
                                ctx,
                            );
                        }
                    }
                    _ => self.set_toast(
                        format!("{} isn't a Fractadyne location.", path.display()),
                        ctx,
                    ),
                },
                Ok(_) => self.set_toast("File too large (not a .fdn location?).", ctx),
                Err(e) => self.set_toast(format!("Couldn't read {}: {e}", path.display()), ctx),
            },
            // Kalles Fraktaler location import.
            "kfr" => match self.load_kfr_file(&path) {
                Ok(m) => self.set_toast(m, ctx),
                Err(e) => self.set_toast(format!("Import failed: {e}"), ctx),
            },
            // Exported image: restore the view from its embedded metadata (EXR or PNG).
            _ => {
                let meta = if ext == "exr" {
                    fractadyne_export::read_exr_metadata(&path)
                } else {
                    fractadyne_export::read_png_metadata(&path)
                };
                match meta {
                    // ⚠Our keyword, but not necessarily a view: the golden images store a
                    // repro command line under it. Say so, rather than complaining that a
                    // location which never claimed to be one is missing its centre.
                    Ok(Some(m)) if !looks_like_a_view(&m) => self.set_toast(
                        format!("{} has Fractadyne metadata, but not a view.", path.display()),
                        ctx,
                    ),
                    Ok(Some(m)) => {
                        if let Some(report) =
                            self.load_view_checked(&m, path.display().to_string())
                        {
                            self.set_toast(
                                match report.note() {
                                    None => format!("Loaded view from {}", path.display()),
                                    Some(n) => {
                                        format!("Loaded view from {} — {n}", path.display())
                                    }
                                },
                                ctx,
                            );
                        }
                    }
                    Ok(None) => self.set_toast(
                        "That file has no embedded Fractadyne view metadata.".to_string(),
                        ctx,
                    ),
                    Err(e) => {
                        self.set_toast(format!("Couldn't read {}: {e}", path.display()), ctx)
                    }
                }
            }
        }
    }


    pub(crate) fn export_ext(&self) -> &'static str {
        match self.export.format {
            ExportFormat::Png => "png",
            ExportFormat::Exr => "exr",
        }
    }

    /// Default timestamped export filename for the current fractal.
    pub(crate) fn export_default_name(&self) -> String {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        crate::export_file_name(self.fractal.name(), stamp, self.export_ext())
    }

    /// Start a background export, prompting for a path (modal Save dialog).
    pub(crate) fn start_export(&mut self, ctx: &egui::Context, device: eframe::wgpu::Device, queue: eframe::wgpu::Queue) {
        if self.export.task.is_some() || self.export.prep.is_some() {
            return;
        }
        let ext = self.export_ext();
        let start_dir = self
            .export.last_dir
            .clone()
            .filter(|d| d.is_dir())
            .unwrap_or_else(|| self.dialog_dir(Self::pictures_dir));
        let path = rfd::FileDialog::new()
            .set_directory(start_dir)
            .set_file_name(self.export_default_name())
            .add_filter(ext.to_uppercase(), &[ext])
            .save_file();
        let Some(path) = path else {
            self.export.status = Some("Export canceled.".to_string());
            return;
        };
        self.start_export_to(ctx, device, queue, path);
    }

    /// Quick export (the Snapshot button's "full render"): no dialog — save to the last-used
    /// folder with an auto name, at the Export dialog's settings. A render that goes to the
    /// background says so in a toast, since the Export dialog (where the progress bar lives) is
    /// not open; a synchronous one toasts its result.
    pub(crate) fn quick_export(&mut self, ctx: &egui::Context, device: eframe::wgpu::Device, queue: eframe::wgpu::Queue) {
        if self.export.task.is_some() || self.export.prep.is_some() {
            self.set_toast("An export is already running — File → Export image… shows its progress.", ctx);
            return;
        }
        let dir = self
            .export.last_dir
            .clone()
            .filter(|d| d.is_dir())
            .unwrap_or_else(Self::pictures_dir);
        let path = dir.join(self.export_default_name());
        self.start_export_to(ctx, device, queue, path);
        let msg = if self.export.task.is_some() || self.export.prep.is_some() {
            format!(
                "Snapshot: rendering {}×{} in the background — File → Export image… shows progress and can cancel it.",
                self.export.width.max(1),
                self.export_height()
            )
        } else {
            self.export.status.clone().unwrap_or_default()
        };
        if !msg.is_empty() {
            self.set_toast(msg, ctx);
        }
    }

    /// The Snapshot button (toolbar camera, File ▸ Snapshot, Ctrl+S): per the persisted choice —
    /// ask on the first press, or capture the screen, or start a full render.
    pub(crate) fn snapshot(&mut self, ctx: &egui::Context) {
        match self.snapshot_mode {
            crate::SnapshotMode::Ask => self.dialogs.snapshot_choice_open = true,
            crate::SnapshotMode::Screen => self.quick_screenshot(),
            crate::SnapshotMode::Render => {
                if let Some((dev, q)) = self.gpu.clone() {
                    self.quick_export(ctx, dev, q);
                } else {
                    self.set_toast("GPU not available", ctx);
                }
            }
        }
    }

    /// Screen-capture snapshot: save the central view exactly as it appears, at screen resolution.
    /// Two-phase like the bookmark thumbnails (request now, harvest the `egui::Event::Screenshot`
    /// reply next frame) — see `process_pending_snapshot`. Zero render work.
    pub(crate) fn quick_screenshot(&mut self) {
        self.snapshot_request = true;
    }

    /// Where a screen-capture snapshot goes: the last export folder (else Pictures), named like
    /// an export but always PNG — a screenshot is 8-bit by construction, so EXR would be a lie.
    fn snapshot_path(&self) -> std::path::PathBuf {
        let dir = self
            .export.last_dir
            .clone()
            .filter(|d| d.is_dir())
            .unwrap_or_else(Self::pictures_dir);
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        dir.join(crate::export_file_name(self.fractal.name(), secs, "png"))
    }

    /// Fire a requested screen-capture snapshot and harvest its reply. The window screenshot is
    /// cropped to the central fractal panel (`central_rect_px`, the same crop the bookmark
    /// thumbnails use) and written as an 8-bit PNG carrying the view metadata, so the file
    /// reopens as a location like any export. One shot in flight at a time, and never while a
    /// bookmark thumbnail is — both harvest the one `Screenshot` event stream.
    pub(crate) fn process_pending_snapshot(&mut self, ctx: &egui::Context) {
        if let Some(path) = self.snapshot_shot.clone() {
            let shot = ctx.input(|inp| {
                inp.events.iter().find_map(|e| match e {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
            });
            let Some(img) = shot else { return };
            self.snapshot_shot = None;
            let (iw, ih) = (img.size[0] as u32, img.size[1] as u32);
            let (x0, y0, cw, chh) = crate::thumb_crop(self.central_rect_px, iw, ih);
            let mut rgba = Vec::with_capacity((cw * chh * 4) as usize);
            for y in y0..y0 + chh {
                let row = &img.pixels[(y * iw + x0) as usize..(y * iw + x0 + cw) as usize];
                for p in row {
                    rgba.extend_from_slice(&p.to_array());
                }
            }
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let meta = self.view_metadata();
            let msg = match fractadyne_export::write_png_rgba8(&path, cw, chh, &rgba, Some(&meta)) {
                Ok(()) => {
                    self.remember_dir(&path);
                    if let Some(parent) = path.parent() {
                        self.export.last_dir = Some(parent.to_path_buf());
                    }
                    format!("Saved screen snapshot {cw}×{chh} → {}", path.display())
                }
                Err(e) => format!("Snapshot failed: {e}"),
            };
            crate::diag::breadcrumb(msg.clone());
            self.export.status = Some(msg.clone());
            self.set_toast(msg, ctx);
            return;
        }
        if self.snapshot_request && self.thumb_shot.is_none() {
            self.snapshot_request = false;
            self.snapshot_shot = Some(self.snapshot_path());
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
    }

    /// Stamp the "Fd" mark into a linear RGBA image buffer if the watermark is enabled and built.
    pub(crate) fn apply_watermark(&self, pixels: &mut [f32], w: u32, h: u32) {
        if self.watermark {
            if let Some(ov) = &self.watermark_overlay {
                stamp_watermark(pixels, w, h, ov);
            }
        }
    }

    }

/// Nominal work of an export, in iteration steps: samples (pixels × ss²) × iterations — the unit
/// every dispatch budget in this codebase is priced in. Saturating, so an absurd request cannot
/// wrap to "cheap".
pub(crate) fn export_nominal_steps(width: u32, height: u32, ss: u32, max_iter: u32) -> u64 {
    (width as u64)
        .saturating_mul(height as u64)
        .saturating_mul((ss as u64).saturating_mul(ss as u64))
        .saturating_mul(max_iter as u64)
}

/// Above this nominal work an export never takes the synchronous main-thread path, whatever its
/// depth. 5e11 steps is a few seconds at the ~1e11 steps/s a mid-range GPU manages in the
/// perturbation modes — the longest the window may plausibly freeze for a "quick" save — and it
/// is 3,400× below the 2026-09-12 field case (1.7e15) that froze the UI for seven minutes and
/// then lost the device. Reached at, e.g., 3840×2160 with 2× supersampling above ~15,000
/// iterations, or a 1080p export past ~240,000.
pub(crate) const SYNC_EXPORT_MAX_STEPS: u64 = 500_000_000_000;

/// Is this export too much work for the synchronous (main-thread, glitch-corrected) path?
pub(crate) fn export_is_heavy(nominal_steps: u64) -> bool {
    nominal_steps > SYNC_EXPORT_MAX_STEPS
}

impl FractadyneApp {
    /// Render one export view — glitch-corrected when enabled and applicable, else the plain path.
    /// `vp` + `julia` identify the view (correction maps glitched pixels back to coordinates to seed
    /// fresh references). Correction is synchronous (multi-pass GPU + readback), so this must run on
    /// the thread owning the device; it falls back to the plain render for aux coloring / oversized
    /// views / when nothing is glitched. The correction loop is wall-clock bounded by
    /// the correction work budget (its dispatches are tiled; see `render::CorrectionBudget`).
    #[allow(clippy::too_many_arguments)] // REFACTOR-PLAN Phase 2/4: fold the request params into a struct
    fn render_export_view(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        vp: &fractadyne_core::Viewport,
        julia: bool,
        req: &fractadyne_gpu::ExportRequest,
        progress: &std::sync::atomic::AtomicU32,
        cancel: &std::sync::atomic::AtomicBool,
    ) -> Result<fractadyne_gpu::ExportResult, fractadyne_gpu::GpuError> {
        // Auto-normalized coloring (`--normalize`): map the palette cycle to the frame's escape-value
        // range so extreme-depth views don't alias into speckle. Falls through to the normal path for
        // aux coloring / all-interior frames / oversized supersampled buffers.
        if self.coloring.normalize {
            // Single export: no prior-frame range to smooth against (`None`) → the frame's own range.
            if let Some((res, _range)) = self
                // Interactive single export: the standard export tile budget (not the tour's tighter
                // one — an on-screen export is a one-off, not a many-frame sequence).
                .render_export_normalized(device, queue, vp, julia, req.width, req.height, req.ss, crate::render::NormRange::OwnFrame, None, 20_000_000_000)
            {
                return Ok(res);
            }
        }
        if self.render_cfg.glitch_correct {
            let budget = crate::render::CorrectionBudget::standard();
            if let Some(res) = self.render_export_corrected(device, queue, vp, julia, req.width, req.height, Some(req), budget) {
                return Ok(res);
            }
        }
        fractadyne_gpu::render_export(device, queue, req, progress, cancel)
    }

    /// End-of-render perf + counter summary (D3.1/D3.2/D3.3): pure-GPU pass times, the
    /// nominal steps/s figure the Fraktaler-3 throughput gap is measured in, and the shader
    /// event counters (execution proof — a zero counter on a path the render claims to
    /// exercise means dead code). Logged un-gated per export (exports are rare) and appended
    /// to logs/perf.jsonl under FRACTADYNE_PERF=1.
    fn log_export_perf(
        kind: &str,
        req: &fractadyne_gpu::ExportRequest,
        r: &fractadyne_gpu::ExportResult,
    ) {
        let px = (r.width as u64 * r.height as u64).saturating_mul((r.ss as u64) * (r.ss as u64));
        let nominal = px.saturating_mul(req.max_iter as u64);
        let gsps = if r.iterate_ms > 0.0 {
            nominal as f64 / (r.iterate_ms / 1000.0) / 1.0e9
        } else {
            0.0
        };
        let c = &r.counters;
        crate::diag::log_line(
            "perf",
            &format!(
                "{kind}: {}x{} ss={} mode={} iter={} gpu_iterate={:.1}ms gpu_color={:.1}ms \
                 max_dispatch={:.0}ms ~{gsps:.2} Gsteps/s (nominal) | counters: rebase={} ext={} \
                 glitch={} bla_skip={} maxiter={}",
                r.width,
                r.height,
                r.ss,
                req.mode,
                req.max_iter,
                r.iterate_ms,
                r.color_ms,
                r.max_dispatch_ms,
                c[fractadyne_gpu::CTR_REBASE],
                c[fractadyne_gpu::CTR_EXT_SAMPLE],
                c[fractadyne_gpu::CTR_GLITCH],
                c[fractadyne_gpu::CTR_BLA_SKIP],
                c[fractadyne_gpu::CTR_MAXITER],
            ),
        );
        crate::diag::perf_jsonl(&format!(
            "\"kind\":\"{kind}\",\"w\":{},\"h\":{},\"ss\":{},\"mode\":{},\"iter\":{},\
             \"gpu_iterate_ms\":{:.3},\"gpu_color_ms\":{:.3},\"max_dispatch_ms\":{:.1},\
             \"gsteps_nominal\":{gsps:.3},\
             \"ctr_rebase\":{},\"ctr_ext\":{},\"ctr_glitch\":{},\"ctr_bla\":{},\"ctr_maxiter\":{}",
            r.width,
            r.height,
            r.ss,
            req.mode,
            req.max_iter,
            r.iterate_ms,
            r.color_ms,
            r.max_dispatch_ms,
            c[fractadyne_gpu::CTR_REBASE],
            c[fractadyne_gpu::CTR_EXT_SAMPLE],
            c[fractadyne_gpu::CTR_GLITCH],
            c[fractadyne_gpu::CTR_BLA_SKIP],
            c[fractadyne_gpu::CTR_MAXITER],
        ));
    }

    /// The effective view a CLI render is about to draw, printed **un-gated** before any
    /// one-shot render (D4.2): a batch that silently renders the wrong location (F8 rendered
    /// the home view twenty times) announces itself on the first line. Center digits are
    /// elided past 48 chars — enough to distinguish locations at any depth.
    fn cli_render_manifest(&self, w: u32, h: u32, out: &std::path::Path) -> String {
        let elide = |s: String| -> String {
            if s.len() > 48 {
                format!("{}…({} digits)", &s[..48], s.len())
            } else {
                s
            }
        };
        let l2 = self.viewport.log2_magnification();
        let l10 = l2 * std::f64::consts::LOG10_2;
        format!(
            "center=({}, {}) zoom={:.4}e{}x iter={}{} size={}x{} ss={} glitch_correct={} out={}",
            elide(fractadyne_core::to_decimal_string(&self.viewport.center_x)),
            elide(fractadyne_core::to_decimal_string(&self.viewport.center_y)),
            10f64.powf(l10.fract().abs()),
            l10.floor() as i64,
            self.render_cfg.max_iter,
            if self.render_cfg.auto_iter { " (auto-capped)" } else { " (explicit)" },
            w,
            h,
            self.export.ss.max(1),
            self.render_cfg.glitch_correct,
            out.display(),
        )
    }

    /// Synchronously render the current view and write it to `path` (used by the
    /// headless `--render` CLI mode). Blocks until done; returns a status message.
    pub(crate) fn render_to_file(
        &self,
        ctx: &egui::Context,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        path: &std::path::Path,
    ) -> Result<String, crate::error::AppError> {
        use std::sync::atomic::AtomicBool;
        use std::sync::atomic::AtomicU32;
        crate::diag::log_line(
            "render",
            &self.cli_render_manifest(self.export.width, self.export_height(), path),
        );
        crate::diag::breadcrumb(format!(
            "CLI render {}x{} → {}",
            self.export.width,
            self.export_height(),
            path.display()
        ));
        let progress = std::sync::Arc::new(AtomicU32::new(0));
        // Progress to stderr every ~2 s (D1.5): distinguishes "slow" from "hung" without
        // killing the process, and stamps the watchdog while tiles are moving.
        // (`&progress` deref-coerces to `&AtomicU32` at the call sites below.)
        let _pump = crate::diag::progress_pump("render", progress.clone());
        let cancel = AtomicBool::new(false);
        let meta = self.view_metadata();
        let fmt = self.export.format;
        let write = |p: &std::path::Path, w: u32, h: u32, mut px: Vec<f32>| {
            self.apply_watermark(&mut px, w, h);
            match fmt {
                ExportFormat::Png => fractadyne_export::write_png(p, w, h, &px, Some(&meta)),
                ExportFormat::Exr => fractadyne_export::write_exr(p, w, h, &px, Some(&meta)),
            }
        };
        // Each view is glitch-corrected when enabled + applicable (single and both dual panels).
        let view = |vp: &fractadyne_core::Viewport, julia: bool, req: &fractadyne_gpu::ExportRequest| {
            self.render_export_view(device, queue, vp, julia, req, &progress, &cancel)
        };
        match self.build_export_job() {
            ExportJob::Single(req) => {
                let mut r = view(&self.viewport, self.julia_mode, &req)?;
                Self::log_export_perf("cli-render", &req, &r);
                if self.show_location {
                    crate::scripting::stamp_location(ctx, &mut r.pixels, r.width, r.height, &self.viewport);
                }
                write(path, r.width, r.height, r.pixels)?;
                Ok(format!("Saved {}×{} → {}", r.width, r.height, path.display()))
            }
            ExportJob::SideBySide(a, b) => {
                let ra = view(&self.viewport, false, &a)?;
                let rb = view(&self.julia_viewport, true, &b)?;
                Self::log_export_perf("cli-render-map", &a, &ra);
                Self::log_export_perf("cli-render-julia", &b, &rb);
                let (w, h, px) = stitch_side_by_side(&ra, &rb);
                write(path, w, h, px)?;
                Ok(format!("Saved {w}×{h} → {}", path.display()))
            }
            ExportJob::Separate(a, b) => {
                let (pmap, pjul) = separate_paths(path);
                let ra = view(&self.viewport, false, &a)?;
                write(&pmap, ra.width, ra.height, ra.pixels)?;
                let rb = view(&self.julia_viewport, true, &b)?;
                write(&pjul, rb.width, rb.height, rb.pixels)?;
                Ok(format!("Saved 2 files → {}", pmap.display()))
            }
        }
    }

    /// Render the **raw iteration texture** for the current view and write it as an EXR
    /// (`--render-iter`): four 32-bit float channels — R = smooth iteration (negative ⇒
    /// in-set/interior), G/B = slope normal (x, y), A = log₂(distance estimate in pixels).
    /// Lets a reviewer diff iteration data directly, removing coloring as a confound.
    /// Single-tile, clamped to the GPU's max texture dimension.
    pub(crate) fn render_iter_to_file(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        path: &std::path::Path,
    ) -> Result<String, crate::error::AppError> {
        crate::diag::log_line(
            "render",
            &self.cli_render_manifest(self.export.width, self.export_height(), path),
        );
        crate::diag::breadcrumb(format!("CLI iter render → {}", path.display()));
        let req = self.current_export_request_for(&self.viewport, self.julia_mode);
        let r = fractadyne_gpu::render_iter(device, queue, &req)?;
        let meta = format!(
            "{}\n# iteration-data EXR: R=smooth_iter (<0 = interior), G=normal.x, \
             B=normal.y, A=log2(distance_estimate_px)",
            self.view_metadata()
        );
        // RAW, not the colour writer: these channels are DATA (a smooth iteration count in the
        // hundreds of thousands, signed normals, a log2 distance estimate), and `write_exr`
        // clamps R/G/B to [0,1] and applies the sRGB transfer. Using it here left the iteration
        // channel a constant 1.0 — see `write_exr_raw`.
        fractadyne_export::write_exr_raw(path, r.width, r.height, &r.pixels, Some(&meta))?;
        Ok(format!("Saved iteration EXR {}×{} → {}", r.width, r.height, path.display()))
    }


    /// Fully render + write a glitch-corrected export synchronously (main thread), for every job
    /// layout. `Some(status)` = handled (success message, or a write-error message); `None` = a view
    /// can't be corrected (aux coloring / oversized), so the caller falls back to the plain threaded
    /// path. Correction re-renders per reference, so it can't use the tiled worker's progress model.
    fn export_corrected_sync(
        &self,
        device: &eframe::wgpu::Device,
        queue: &eframe::wgpu::Queue,
        path: &std::path::Path,
        job: &ExportJob,
        hud: Option<&crate::scripting::HudOverlay>,
    ) -> Option<String> {
        let meta = self.view_metadata();
        let correct = |vp: &fractadyne_core::Viewport, julia: bool, req: &fractadyne_gpu::ExportRequest| {
            let budget = crate::render::CorrectionBudget::standard();
            self.render_export_corrected(device, queue, vp, julia, req.width, req.height, Some(req), budget)
        };
        let write = |p: &std::path::Path,
                     w: u32,
                     h: u32,
                     mut px: Vec<f32>|
         -> Result<(), fractadyne_export::ExportError> {
            self.apply_watermark(&mut px, w, h);
            if let Some(ov) = hud {
                crate::scripting::blit_location_overlay(&mut px, w, h, ov);
            }
            match self.export.format {
                ExportFormat::Png => fractadyne_export::write_png(p, w, h, &px, Some(&meta)),
                ExportFormat::Exr => fractadyne_export::write_exr(p, w, h, &px, Some(&meta)),
            }
        };
        let status = |res: Result<(), fractadyne_export::ExportError>, ok: String| match res {
            Ok(_) => ok,
            Err(e) => format!("Export failed: {e}"),
        };
        match job {
            ExportJob::Single(req) => {
                let r = correct(&self.viewport, self.julia_mode, req)?;
                let (w, h) = (r.width, r.height);
                Some(status(write(path, w, h, r.pixels), format!("Saved {w}×{h} (glitch-corrected) → {}", path.display())))
            }
            ExportJob::SideBySide(a, b) => {
                let ra = correct(&self.viewport, false, a)?;
                let rb = correct(&self.julia_viewport, true, b)?;
                let (w, h, px) = stitch_side_by_side(&ra, &rb);
                Some(status(write(path, w, h, px), format!("Saved {w}×{h} (glitch-corrected) → {}", path.display())))
            }
            ExportJob::Separate(a, b) => {
                let ra = correct(&self.viewport, false, a)?;
                let rb = correct(&self.julia_viewport, true, b)?;
                let (pmap, pjul) = separate_paths(path);
                let w1 = write(&pmap, ra.width, ra.height, ra.pixels);
                let w2 = write(&pjul, rb.width, rb.height, rb.pixels);
                Some(status(w1.and(w2), format!("Saved 2 files (glitch-corrected) → {}", pmap.display())))
            }
        }
    }

    /// Render the current job on a worker thread and write to `path` (or, for dual
    /// "separate", to `path` + a sibling). The UI stays responsive; result via channel.
    pub(crate) fn start_export_to(
        &mut self,
        ctx: &egui::Context,
        device: eframe::wgpu::Device,
        queue: eframe::wgpu::Queue,
        path: std::path::PathBuf,
    ) {
        if self.export.task.is_some() || self.export.prep.is_some() {
            return;
        }
        // Start the export clock now — for a deep export this includes the (long) off-thread
        // reference build, which is part of the wait the user is timing.
        self.export.started = Some(std::time::Instant::now());

        // ⭐Remember where it is going, so the "open when done" option has a path that does not
        // depend on the wording of a status message.
        self.export.dest = Some(path.clone());
        if let Some(parent) = path.parent() {
            self.export.last_dir = Some(parent.to_path_buf());
        }
        self.remember_dir(&path);
        // Deep export: build the (slow, bignum) MAP reference orbit OFF the main thread so the UI
        // stays responsive instead of freezing (at extreme depth the reference build alone is
        // minutes). The render dispatches once it lands — see the `export_prep` poll in `update()`.
        // Works for single AND dual (the dual Julia panel is usually shallow and builds instantly in
        // the poll). Glitch correction is skipped at this depth (its multi-pass re-render would
        // re-block the UI); shallower exports keep the synchronous path below (fast; correction applies).
        //
        // ⭐⭐**"Shallow" was a proxy for "fast", and the proxy failed.** Depth alone chose the path;
        // the cost is pixels × samples × iterations. Field case 2026-09-12: 4.4e21× (below the
        // threshold) at 5120×4035, ss 4, 5,223,168 iterations = 1.7e15 nominal steps went down the
        // synchronous path — seven minutes of "Not Responding" on the main thread, then a device
        // loss with no way to cancel. The gate is now depth OR estimated work (`export_is_heavy`):
        // a heavy export takes the worker path (progress + cancel), and like the deep path it skips
        // glitch correction, which is stated in the status rather than silently dropped.
        let (ew, eh, ess) = (self.export.width.max(1), self.export_height(), self.export.ss.max(1));
        let eiter = self.export_eff_iter(&self.viewport, !self.dual && self.julia_mode);
        let heavy = export_is_heavy(export_nominal_steps(ew, eh, ess, eiter));
        if self.viewport.magnification() >= crate::PERT_FE_THRESHOLD || heavy {
            let map_julia = !self.dual && self.julia_mode; // the dual map panel is Mandelbrot
            // The reference is for the view on screen, so the current settings are the right ones.
            let budget = crate::render::IterBudget::current(self);
            if let Some(rx) = self.spawn_export_reference(&self.viewport, map_julia, budget) {
                self.export.prep = Some(ExportPrep {
                    rx,
                    map_vp: self.viewport.clone(),
                    julia_mode: self.julia_mode,
                    julia_vp: self.dual.then(|| self.julia_viewport.clone()),
                    dual_mode: self.export.dual_mode,
                    path,
                });
                self.export.status = Some(if heavy {
                    format!(
                        "Large export ({ew}×{eh}, {ess}× supersampling, {eiter} iterations) — \
                         rendering in the background; glitch correction is skipped for exports this size…"
                    )
                } else {
                    "Preparing deep export — building reference (this can take a while)…".to_string()
                });
                return;
            }
        }
        let job = self.build_export_job();
        // Optional location HUD: rasterized here on the main thread (needs the egui font atlas), then
        // blitted by the sync/worker write paths (which have no context). Built from the map view;
        // it lands on the image's top-left — which for a side-by-side dual stitch is the map panel.
        let hud = self
            .show_location
            .then(|| crate::scripting::build_location_overlay(ctx, &self.viewport, self.export_height()))
            .flatten();
        // Glitch correction re-renders per reference (synchronous, main thread), so it runs here
        // rather than on the tiled worker. Handles single + dual layouts; falls back to the threaded
        // path for aux coloring methods or views past the ~32 MP / single-texture correction limit.
        // Never for a heavy export (see above): a direct-mode view has no reference to prepare, so
        // it lands here, and the synchronous path would freeze the UI for the whole render.
        if self.render_cfg.glitch_correct && !heavy {
            if let Some(msg) = self.export_corrected_sync(&device, &queue, &path, &job, hud.as_ref()) {
                self.export.status = Some(self.finish_export_status(msg));
                return;
            }
        }
        self.spawn_export_worker(device, queue, job, path, hud);
    }

    /// Format an export duration compactly: `8.3s`, or `1m 04.0s` past a minute.
    pub(crate) fn fmt_export_duration(d: std::time::Duration) -> String {
        let s = d.as_secs_f64();
        if s < 60.0 {
            format!("{s:.1}s")
        } else {
            let m = (s / 60.0).floor() as u64;
            format!("{m}m {:04.1}s", s - m as f64 * 60.0)
        }
    }

    /// Finalize an export status line: on success append the total elapsed time; either way clear
    /// the timer. Cancel/failure messages pass through unchanged (no time — the run didn't finish).
    pub(crate) fn finish_export_status(&mut self, msg: String) -> String {
        // ⭐⭐Every finished export funnels through here — both the synchronous glitch-corrected
        // path and the background worker — which is why the "open when done" hook lives here and
        // not at the two call sites.
        let dest = self.export.dest.take();
        let ok = msg.starts_with("Saved");
        // ⚠Never during the scripted UI walk, which exports for real — the same exemption the
        // finish tone carries, and for the same reason: a gate must not spray the machine with
        // viewer windows.
        if ok && self.export.open_after && self.harness.uitest.is_none() {
            // ⚠Only a file that is actually there. A worker can report success for a stitched
            // pair whose reported path is the map half; either way, handing a missing path to the
            // shell opens a browser error, so check first and stay silent if it is not readable.
            if let Some(p) = dest.filter(|p| p.is_file()) {
                self.export.pending_open = Some(p);
            }
        }
        match self.export.started.take() {
            Some(t) if ok => format!("{msg}  (in {})", Self::fmt_export_duration(t.elapsed())),
            _ => msg,
        }
    }

    /// Render an already-assembled export job (references built) on a background worker and write it
    /// (watermark + HUD applied). Shares the export status channel + progress/cancel. Used by the
    /// deep-export path once the reference has been built off the main thread (`export_prep`).
    pub(crate) fn spawn_export_worker(
        &mut self,
        device: eframe::wgpu::Device,
        queue: eframe::wgpu::Queue,
        job: ExportJob,
        path: std::path::PathBuf,
        hud: Option<crate::scripting::HudOverlay>,
    ) {
        use std::sync::atomic::Ordering::Relaxed;
        let meta = self.view_metadata();
        let format = self.export.format;
        self.export.progress.store(0, Relaxed);
        self.export.cancel.store(false, Relaxed);
        let progress = self.export.progress.clone();
        let cancel = self.export.cancel.clone();
        let wm = self.watermark.then(|| self.watermark_overlay.clone()).flatten();
        let (tx, rx) = std::sync::mpsc::channel();
        self.export.task = Some(rx);
        self.export.status = Some("Rendering…".to_string());
        crate::diag::breadcrumb(format!("GUI export → {}", path.display()));
        std::thread::spawn(move || {
            let render = |req: &fractadyne_gpu::ExportRequest| {
                fractadyne_gpu::render_export(&device, &queue, req, &progress, &cancel)
            };
            let write = |p: &std::path::Path, w: u32, h: u32, mut px: Vec<f32>| {
                if let Some(ov) = &wm {
                    stamp_watermark(&mut px, w, h, ov);
                }
                if let Some(ov) = &hud {
                    crate::scripting::blit_location_overlay(&mut px, w, h, ov);
                }
                match format {
                    ExportFormat::Png => fractadyne_export::write_png(p, w, h, &px, Some(&meta)),
                    ExportFormat::Exr => fractadyne_export::write_exr(p, w, h, &px, Some(&meta)),
                }
            };
            let msg = (|| -> Result<String, crate::error::AppError> {
                match job {
                    ExportJob::Single(req) => {
                        let r = render(&req)?;
                        Self::log_export_perf("gui-export", &req, &r);
                        progress.store(2000, Relaxed);
                        let (rw, rh) = (r.width, r.height);
                        write(&path, rw, rh, r.pixels)?;
                        Ok(format!("Saved {}×{} → {}", rw, rh, path.display()))
                    }
                    ExportJob::SideBySide(a, b) => {
                        let ra = render(&a)?;
                        let rb = render(&b)?;
                        progress.store(2000, Relaxed);
                        let (w, h, px) = stitch_side_by_side(&ra, &rb);
                        write(&path, w, h, px)?;
                        Ok(format!("Saved {w}×{h} → {}", path.display()))
                    }
                    ExportJob::Separate(a, b) => {
                        let (pmap, pjul) = separate_paths(&path);
                        let ra = render(&a)?;
                        write(&pmap, ra.width, ra.height, ra.pixels)?;
                        let rb = render(&b)?;
                        progress.store(2000, Relaxed);
                        write(&pjul, rb.width, rb.height, rb.pixels)?;
                        Ok(format!("Saved 2 files → {}", pmap.display()))
                    }
                }
            })();
            let _ = tx.send(match msg {
                Ok(m) => m,
                Err(crate::error::AppError::Gpu(fractadyne_gpu::GpuError::Canceled)) => {
                    "Export canceled.".to_string()
                }
                Err(e) => format!("Export failed: {e}"),
            });
        });
    }
}

#[cfg(test)]
mod palette_embed;
#[cfg(test)]
mod view_text;
#[cfg(test)]
mod shipped_files;
