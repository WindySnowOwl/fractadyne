//! The normalize-anchor file: a tour's time-keyed palette ranges, measured once and carried to
//! every machine that renders the tour (`--dump-norm-anchors FILE` writes it, `--norm-anchors FILE`
//! reads it).
//!
//! ⭐Why it exists: a normalized tour maps its palette through ranges MEASURED on the GPU at each
//! keyframe, and escape values differ across GPUs — so two machines rendering one tour measured
//! two mappings, and a video assembled from both flickered at every seam (TODO.md "distributed
//! normalize coherence"). Measuring once and shipping the numbers makes the mapping a property of
//! the tour again, not of the machine (design/remote-rendering.md §9).
//!
//! The file is untrusted input like any other the app reads (SECURITY.md): size-bounded, every
//! field checked, and it must describe THIS tour — same script text, frame rate, frame count and
//! iteration base — or it is refused with both values named. A mismatch means the anchors were
//! measured for a different render, and using them would be a silently wrong palette.

/// Largest file accepted. A tour has at most a few hundred keyframes; an anchor is ~80 bytes.
const MAX_FILE_BYTES: u64 = 1 << 20;
/// Most anchors accepted — far above any real tour, far below anything that costs memory.
const MAX_ANCHORS: usize = 10_000;
const FORMAT: &str = "fractadyne-norm-anchors";
const VERSION: i64 = 1;

/// What the anchors were measured FOR. Every field must match the render that reads them.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AnchorContext {
    /// FNV-1a 64 over the script text with line endings normalised to LF (a script copied between
    /// Windows and Linux machines is still the same script).
    pub(crate) script_digest: u64,
    pub(crate) fps: f64,
    pub(crate) frames: u64,
    pub(crate) base_iter: u32,
    pub(crate) base_auto: bool,
    /// The tour's length, seconds — the clamp in each anchor's time `(frame / fps).min(total)`.
    pub(crate) total: f64,
}

/// One measured anchor, as the renderer uses it: the keyframe's frame index, its time, the range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Anchor {
    pub(crate) frame: u64,
    pub(crate) t: f64,
    pub(crate) range: (f32, f32),
}

/// The time a frame index stands for — the SAME formula the tour renderer uses for every frame,
/// so an anchor read from a file lands on exactly the time a locally measured one would.
pub(crate) fn frame_time(frame: u64, fps: f64, total: f64) -> f64 {
    if total <= 0.0 {
        0.0
    } else {
        (frame as f64 / fps).min(total)
    }
}

/// FNV-1a 64 of the script text, CRLF/CR folded to LF first.
pub(crate) fn script_digest(text: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut feed = |b: u8| {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    };
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\r' => {
                feed(b'\n');
                if bytes.get(i + 1) == Some(&b'\n') {
                    i += 1;
                }
            }
            b => feed(b),
        }
        i += 1;
    }
    h
}

/// The file's text. Numbers are written with `{:?}`, which is Rust's shortest representation that
/// parses back to the SAME value; each range bound is an `f32` widened to `f64` (exact), so the
/// file round-trips bit for bit — the whole point is that every machine applies identical ranges.
pub(crate) fn encode(ctx: &AnchorContext, app_version: &str, git: &str, anchors: &[Anchor]) -> String {
    let mut s = String::new();
    s.push_str("# Fractadyne normalize anchors - written by `--dump-norm-anchors`, read by `--norm-anchors`.\n");
    s.push_str("# The tour's palette ranges, measured once so every machine rendering it colours it alike.\n");
    s.push_str("# Values are exact: do not edit them. A file for a different script, frame rate or\n");
    s.push_str("# iteration base is refused.\n");
    // Informational strings, kept to printable ASCII so the TOML is valid whatever they contain.
    let plain = |v: &str| -> String {
        v.chars().filter(|c| c.is_ascii_graphic() && *c != '"' && *c != '\\' || *c == ' ').collect()
    };
    s.push_str(&format!("format = \"{FORMAT}\"\nversion = {VERSION}\n"));
    s.push_str(&format!("app_version = \"{}\"\ngit = \"{}\"\n", plain(app_version), plain(git)));
    s.push_str(&format!("script = \"fnv1a64:{:016x}\"\n", ctx.script_digest));
    s.push_str(&format!("fps = {:?}\nframes = {}\n", ctx.fps, ctx.frames));
    s.push_str(&format!("base_iter = {}\nbase_auto = {}\n", ctx.base_iter, ctx.base_auto));
    for a in anchors {
        s.push_str(&format!(
            "\n[[anchor]]\nframe = {}\nt = {:?}\nlo = {:?}\nhi = {:?}\n",
            a.frame, a.t, a.range.0 as f64, a.range.1 as f64
        ));
    }
    s
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FileDoc {
    format: String,
    version: i64,
    app_version: String,
    git: String,
    script: String,
    fps: f64,
    frames: u64,
    base_iter: u32,
    base_auto: bool,
    #[serde(default)]
    anchor: Vec<FileAnchor>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FileAnchor {
    frame: u64,
    t: f64,
    lo: f64,
    hi: f64,
}

/// What a file said beyond the anchors: the build that measured them, for a warning when it is
/// not this one (the farm gates versions itself; a hand-carried file may not have been).
#[derive(Debug, PartialEq)]
pub(crate) struct Decoded {
    pub(crate) anchors: Vec<Anchor>,
    pub(crate) app_version: String,
    pub(crate) git: String,
}

/// Read and check a file against the render that will use it. Every refusal names what differed.
pub(crate) fn read(path: &std::path::Path, want: &AnchorContext) -> Result<Decoded, String> {
    let len = std::fs::metadata(path).map_err(|e| format!("{}: {e}", path.display()))?.len();
    if len > MAX_FILE_BYTES {
        return Err(format!("{}: {len} bytes is too large for an anchor file", path.display()));
    }
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    decode(&text, want).map_err(|e| format!("{}: {e}", path.display()))
}

/// The checks of [`read`], on text. Pure, so each refusal is pinned by test.
pub(crate) fn decode(text: &str, want: &AnchorContext) -> Result<Decoded, String> {
    let doc: FileDoc = toml::from_str(text).map_err(|e| format!("not an anchor file: {e}"))?;
    if doc.format != FORMAT {
        return Err(format!("format is \"{}\", expected \"{FORMAT}\"", doc.format));
    }
    if doc.version != VERSION {
        return Err(format!("version {} is not one this build reads ({VERSION})", doc.version));
    }
    let want_script = format!("fnv1a64:{:016x}", want.script_digest);
    if doc.script != want_script {
        return Err(format!(
            "measured for a different script ({}; this script is {want_script})",
            doc.script
        ));
    }
    // Exact comparisons on purpose: both sides parse the same decimal text to the same double.
    if doc.fps != want.fps {
        return Err(format!("measured at {} fps; this render is {} fps", doc.fps, want.fps));
    }
    if doc.frames != want.frames {
        return Err(format!("measured for {} frames; this render has {}", doc.frames, want.frames));
    }
    if (doc.base_iter, doc.base_auto) != (want.base_iter, want.base_auto) {
        return Err(format!(
            "measured with iteration base {} (auto {}); this render uses {} (auto {}) — set [render] max_iter in the script so every machine agrees",
            doc.base_iter, doc.base_auto, want.base_iter, want.base_auto
        ));
    }
    if doc.anchor.len() > MAX_ANCHORS {
        return Err(format!("{} anchors is more than the {MAX_ANCHORS} allowed", doc.anchor.len()));
    }
    let mut anchors = Vec::with_capacity(doc.anchor.len());
    let mut prev: Option<u64> = None;
    for (i, a) in doc.anchor.iter().enumerate() {
        let n = i + 1;
        if a.frame >= want.frames {
            return Err(format!("anchor {n}: frame {} is past the tour's last frame {}", a.frame, want.frames - 1));
        }
        if prev.is_some_and(|p| a.frame <= p) {
            return Err(format!("anchor {n}: frames must strictly increase ({} after {})", a.frame, prev.unwrap_or(0)));
        }
        prev = Some(a.frame);
        // The time is recomputed, never trusted: it must be the time this render gives the frame.
        let t = frame_time(a.frame, want.fps, want.total);
        if a.t != t {
            return Err(format!("anchor {n}: time {} does not match frame {} ({t})", a.t, a.frame));
        }
        let exact_f32 = |v: f64| v.is_finite() && (v as f32) as f64 == v;
        if !exact_f32(a.lo) || !exact_f32(a.hi) || a.lo > a.hi {
            return Err(format!("anchor {n}: range [{}, {}] was not written by --dump-norm-anchors", a.lo, a.hi));
        }
        anchors.push(Anchor { frame: a.frame, t, range: (a.lo as f32, a.hi as f32) });
    }
    Ok(Decoded { anchors, app_version: doc.app_version, git: doc.git })
}

#[cfg(test)]
mod tests;
