//! Custom orbit-trap outlines from files (coloring plan F4, part 4): an SVG drawing's paths and basic
//! shapes, flattened to the line segments of a [`fractadyne_core::outline::Outline`].
//!
//! The file is UNTRUSTED (it may come from anywhere): at most [`MAX_SVG_BYTES`], UTF-8, XML without
//! a DTD (so no entity expansion), walked with an explicit stack (no recursion to overflow), and at
//! most [`MAX_RAW_POINTS`] points before flattening. What it reads: `path`, `rect` (square corners),
//! `circle`, `ellipse`, `line`, `polyline` and `polygon`, each with the `transform`s of its element
//! and its ancestors, in user units (absolute length units converted; a percentage or a font-relative
//! length skips the shape). What it leaves out: text, images, `use`, and everything under `defs`,
//! `symbol`, `clipPath`, `mask`, `pattern` and `marker`, which draw nothing by themselves.

use fractadyne_core::outline::{Outline, MAX_SEGMENTS};
use svgtypes::{SimplePathSegment, SimplifyingPathParser};

/// Bytes of SVG read.
pub(crate) const MAX_SVG_BYTES: usize = 4 * 1024 * 1024;
/// Points (curve control points included) a drawing may have before flattening.
pub(crate) const MAX_RAW_POINTS: usize = 1_000_000;
/// The flattening's tolerance, of the drawing's half-diagonal: about the unit frame's 0.002, where a
/// unit circle flattens to some 50 segments.
const TOLERANCE: f64 = 0.002;
/// Elements whose content draws nothing by itself.
const SKIP: [&str; 12] =
    ["defs", "symbol", "clipPath", "mask", "pattern", "marker", "title", "desc", "metadata", "style", "script", "foreignObject"];

/// An affine map [a b c d e f]: x' = a x + c y + e, y' = b x + d y + f.
#[derive(Clone, Copy, Debug)]
struct Affine([f64; 6]);

impl Affine {
    const ID: Affine = Affine([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    fn apply(self, x: f64, y: f64) -> [f64; 2] {
        let m = self.0;
        [m[0] * x + m[2] * y + m[4], m[1] * x + m[3] * y + m[5]]
    }
    /// `self` after `inner` (an element's own transform inside its parent's).
    fn then(self, inner: Affine) -> Affine {
        let (p, c) = (self.0, inner.0);
        Affine([
            p[0] * c[0] + p[2] * c[1],
            p[1] * c[0] + p[3] * c[1],
            p[0] * c[2] + p[2] * c[3],
            p[1] * c[2] + p[3] * c[3],
            p[0] * c[4] + p[2] * c[5] + p[4],
            p[1] * c[4] + p[3] * c[5] + p[5],
        ])
    }
}

/// A piece of a subpath after its start: a line, or a cubic's two controls and its end.
#[derive(Clone, Copy)]
enum Piece {
    Line([f64; 2]),
    Cubic([f64; 2], [f64; 2], [f64; 2]),
}

/// One subpath: where it starts, its pieces, closed or not.
struct Sub {
    start: [f64; 2],
    pieces: Vec<Piece>,
    closed: bool,
}

/// Collects subpaths in the drawing's frame, counting its points against [`MAX_RAW_POINTS`].
struct Paths {
    subs: Vec<Sub>,
    points: usize,
}

impl Paths {
    fn push(&mut self, sub: Sub) -> Result<(), String> {
        self.points += 1 + sub.pieces.iter().map(|p| if matches!(p, Piece::Line(_)) { 1 } else { 3 }).sum::<usize>();
        if self.points > MAX_RAW_POINTS {
            return Err(format!("the drawing has more than {MAX_RAW_POINTS} points"));
        }
        if !sub.pieces.is_empty() {
            self.subs.push(sub);
        }
        Ok(())
    }
}

/// A length attribute in user units (px): absolute units converted; absent is `default`; a
/// percentage or a font-relative unit is `None` (the shape is skipped).
fn length(node: roxmltree::Node, name: &str, default: f64) -> Option<f64> {
    let Some(v) = node.attribute(name) else { return Some(default) };
    let l: svgtypes::Length = v.trim().parse().ok()?;
    use svgtypes::LengthUnit as U;
    let k = match l.unit {
        U::None | U::Px => 1.0,
        U::In => 96.0,
        U::Cm => 96.0 / 2.54,
        U::Mm => 96.0 / 25.4,
        U::Pt => 4.0 / 3.0,
        U::Pc => 16.0,
        _ => return None,
    };
    let n = l.number * k;
    n.is_finite().then_some(n)
}

/// The element's own `transform`, or the identity.
fn own_transform(node: roxmltree::Node) -> Result<Affine, String> {
    match node.attribute("transform") {
        None => Ok(Affine::ID),
        Some(t) => {
            let t: svgtypes::Transform = t.parse().map_err(|e| format!("a transform it cannot read ({e})"))?;
            let m = [t.a, t.b, t.c, t.d, t.e, t.f];
            if m.iter().all(|v| v.is_finite()) { Ok(Affine(m)) } else { Err("a transform that is not finite".into()) }
        }
    }
}

/// A cubic quarter of an ellipse: the control points' distance, of the radius.
const KAPPA: f64 = 0.552_284_749_830_793_4;

/// The ellipse at (cx, cy) with radii (rx, ry), as four cubics, through `m`.
fn ellipse(m: Affine, cx: f64, cy: f64, rx: f64, ry: f64) -> Sub {
    let p = |x: f64, y: f64| m.apply(cx + x * rx, cy + y * ry);
    let k = KAPPA;
    Sub {
        start: p(1.0, 0.0),
        pieces: vec![
            Piece::Cubic(p(1.0, k), p(k, 1.0), p(0.0, 1.0)),
            Piece::Cubic(p(-k, 1.0), p(-1.0, k), p(-1.0, 0.0)),
            Piece::Cubic(p(-1.0, -k), p(-k, -1.0), p(0.0, -1.0)),
            Piece::Cubic(p(k, -1.0), p(1.0, -k), p(1.0, 0.0)),
        ],
        closed: true,
    }
}

/// A `path` element's `d`, through `m`.
fn path_data(d: &str, m: Affine, out: &mut Paths) -> Result<(), String> {
    let mut cur: Option<Sub> = None;
    let mut at = [0.0, 0.0];
    for seg in SimplifyingPathParser::from(d) {
        let seg = seg.map_err(|e| format!("path data it cannot read ({e})"))?;
        match seg {
            SimplePathSegment::MoveTo { x, y } => {
                if let Some(s) = cur.take() {
                    out.push(s)?;
                }
                at = m.apply(x, y);
                cur = Some(Sub { start: at, pieces: Vec::new(), closed: false });
            }
            SimplePathSegment::LineTo { x, y } => {
                at = m.apply(x, y);
                cur.get_or_insert(Sub { start: at, pieces: Vec::new(), closed: false }).pieces.push(Piece::Line(at));
            }
            SimplePathSegment::CurveTo { x1, y1, x2, y2, x, y } => {
                let p = Piece::Cubic(m.apply(x1, y1), m.apply(x2, y2), m.apply(x, y));
                at = m.apply(x, y);
                cur.get_or_insert(Sub { start: at, pieces: Vec::new(), closed: false }).pieces.push(p);
            }
            SimplePathSegment::Quadratic { x1, y1, x, y } => {
                // The same curve as a cubic: its controls two thirds of the way to the quadratic's.
                let (q, e) = (m.apply(x1, y1), m.apply(x, y));
                let c1 = [at[0] + 2.0 / 3.0 * (q[0] - at[0]), at[1] + 2.0 / 3.0 * (q[1] - at[1])];
                let c2 = [e[0] + 2.0 / 3.0 * (q[0] - e[0]), e[1] + 2.0 / 3.0 * (q[1] - e[1])];
                at = e;
                cur.get_or_insert(Sub { start: at, pieces: Vec::new(), closed: false }).pieces.push(Piece::Cubic(c1, c2, e));
            }
            SimplePathSegment::ClosePath => {
                if let Some(mut s) = cur.take() {
                    s.closed = true;
                    at = s.start;
                    out.push(s)?;
                }
                cur = Some(Sub { start: at, pieces: Vec::new(), closed: false });
            }
        }
    }
    if let Some(s) = cur {
        out.push(s)?;
    }
    Ok(())
}

/// One element's own drawing, if it is a shape this reads.
fn shape(node: roxmltree::Node, m: Affine, out: &mut Paths) -> Result<(), String> {
    let n = |name: &str| length(node, name, 0.0);
    match node.tag_name().name() {
        "path" => path_data(node.attribute("d").unwrap_or(""), m, out),
        "rect" => {
            let (Some(x), Some(y), Some(w), Some(h)) = (n("x"), n("y"), n("width"), n("height")) else { return Ok(()) };
            if w <= 0.0 || h <= 0.0 {
                return Ok(());
            }
            let p = |a: f64, b: f64| m.apply(a, b);
            out.push(Sub {
                start: p(x, y),
                pieces: vec![Piece::Line(p(x + w, y)), Piece::Line(p(x + w, y + h)), Piece::Line(p(x, y + h))],
                closed: true,
            })
        }
        "circle" => match (n("cx"), n("cy"), n("r")) {
            (Some(cx), Some(cy), Some(r)) if r > 0.0 => out.push(ellipse(m, cx, cy, r, r)),
            _ => Ok(()),
        },
        "ellipse" => match (n("cx"), n("cy"), n("rx"), n("ry")) {
            (Some(cx), Some(cy), Some(rx), Some(ry)) if rx > 0.0 && ry > 0.0 => out.push(ellipse(m, cx, cy, rx, ry)),
            _ => Ok(()),
        },
        "line" => match (n("x1"), n("y1"), n("x2"), n("y2")) {
            (Some(x1), Some(y1), Some(x2), Some(y2)) => {
                out.push(Sub { start: m.apply(x1, y1), pieces: vec![Piece::Line(m.apply(x2, y2))], closed: false })
            }
            _ => Ok(()),
        },
        "polyline" | "polygon" => {
            let mut pts = svgtypes::PointsParser::from(node.attribute("points").unwrap_or("")).map(|(x, y)| m.apply(x, y));
            let Some(start) = pts.next() else { return Ok(()) };
            let pieces: Vec<Piece> = pts.map(Piece::Line).collect();
            out.push(Sub { start, pieces, closed: node.tag_name().name() == "polygon" })
        }
        _ => Ok(()),
    }
}

/// Every drawable subpath of an SVG document, in its own frame (y down).
fn read_svg(bytes: &[u8]) -> Result<Vec<Sub>, String> {
    if bytes.len() > MAX_SVG_BYTES {
        return Err(format!("the file is {} bytes; at most {MAX_SVG_BYTES}", bytes.len()));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| "the file is not UTF-8 text".to_string())?;
    let mut opt = roxmltree::ParsingOptions::default();
    opt.allow_dtd = false;
    opt.nodes_limit = 2_000_000;
    let doc = roxmltree::Document::parse_with_options(text, opt).map_err(|e| format!("not an SVG drawing ({e})"))?;
    if doc.root_element().tag_name().name() != "svg" {
        return Err("not an SVG drawing (its root is not <svg>)".into());
    }
    let mut out = Paths { subs: Vec::new(), points: 0 };
    let mut stack = vec![(doc.root_element(), Affine::ID)];
    while let Some((node, parent)) = stack.pop() {
        let name = node.tag_name().name();
        if SKIP.contains(&name) || node.attribute("display") == Some("none") {
            continue;
        }
        let m = parent.then(own_transform(node)?);
        shape(node, m, &mut out)?;
        if matches!(name, "svg" | "g" | "a" | "switch") {
            let kids: Vec<_> = node.children().filter(|c| c.is_element()).collect();
            stack.extend(kids.into_iter().rev().map(|c| (c, m)));
        }
    }
    if out.subs.is_empty() {
        return Err("the drawing has no paths or shapes (text, images and <use> are not read)".into());
    }
    Ok(out.subs)
}

/// The distance from `p` to the line through `a` and `b` (to `a` when they coincide).
fn line_dist(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = dx.hypot(dy);
    if len == 0.0 {
        return (p[0] - a[0]).hypot(p[1] - a[1]);
    }
    ((p[0] - a[0]) * dy - (p[1] - a[1]) * dx).abs() / len
}

/// A cubic from `p0`, split in halves until its controls are within `tol` of its chord (at most 2^16
/// pieces), its points after `p0` appended to `out`.
fn flatten_cubic(p0: [f64; 2], c1: [f64; 2], c2: [f64; 2], p3: [f64; 2], tol: f64, depth: u32, out: &mut Vec<[f64; 2]>) {
    if depth >= 16 || line_dist(c1, p0, p3).max(line_dist(c2, p0, p3)) <= tol {
        out.push(p3);
        return;
    }
    let mid = |a: [f64; 2], b: [f64; 2]| [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
    let (a, b, c) = (mid(p0, c1), mid(c1, c2), mid(c2, p3));
    let (d, e) = (mid(a, b), mid(b, c));
    let f = mid(d, e);
    flatten_cubic(p0, a, d, f, tol, depth + 1, out);
    flatten_cubic(f, e, c, p3, tol, depth + 1, out);
}

/// The subpaths as polylines at tolerance `tol`, y flipped up (an SVG's y runs down the page).
fn flatten(subs: &[Sub], tol: f64) -> Vec<Vec<[f64; 2]>> {
    subs.iter()
        .map(|s| {
            let mut pts = vec![s.start];
            let mut at = s.start;
            for p in &s.pieces {
                match *p {
                    Piece::Line(e) => {
                        pts.push(e);
                        at = e;
                    }
                    Piece::Cubic(c1, c2, e) => {
                        flatten_cubic(at, c1, c2, e, tol, 0, &mut pts);
                        at = e;
                    }
                }
            }
            if s.closed && pts.last() != Some(&s.start) {
                pts.push(s.start);
            }
            pts.into_iter().map(|p| [p[0], -p[1]]).collect()
        })
        .collect()
}

/// The outline of an SVG file's drawing (see the module's notes), flattened at [`TOLERANCE`] of its
/// size, coarser (doubling) until it fits [`MAX_SEGMENTS`].
pub(crate) fn outline_from_svg(bytes: &[u8]) -> Result<Outline, String> {
    let subs = read_svg(bytes)?;
    let pts = subs.iter().flat_map(|s| std::iter::once(&s.start).chain(s.pieces.iter().flat_map(|p| match p {
        Piece::Line(e) => vec![e],
        Piece::Cubic(a, b, e) => vec![a, b, e],
    })));
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for p in pts {
        if !(p[0].is_finite() && p[1].is_finite()) {
            return Err("the drawing has a coordinate that is not a number".into());
        }
        (x0, y0, x1, y1) = (x0.min(p[0]), y0.min(p[1]), x1.max(p[0]), y1.max(p[1]));
    }
    let half_diag = 0.5 * (x1 - x0).hypot(y1 - y0);
    if half_diag <= 0.0 {
        return Err("the drawing has no extent".into());
    }
    let mut tol = TOLERANCE * half_diag;
    for _ in 0..8 {
        let polys = flatten(&subs, tol);
        let segs: usize = polys.iter().map(|p| p.len().saturating_sub(1)).sum();
        if segs <= MAX_SEGMENTS {
            return Outline::from_paths(&polys).map_err(|e| e.to_string());
        }
        tol *= 2.0;
    }
    Err(format!("the drawing has too many lines for an outline even simplified (at most {MAX_SEGMENTS} segments)"))
}

/// The most characters of an outline's name a view or a session keeps.
pub(crate) const MAX_NAME_CHARS: usize = 120;

/// An outline's name as shown and stored: what it was loaded from, UNTRUSTED when a view brings
/// it (a file name on Linux may hold a line break): control characters made spaces, trimmed, at
/// most [`MAX_NAME_CHARS`].
pub(crate) fn outline_name(raw: &str) -> String {
    let s: String = raw.chars().map(|c| if c.is_control() { ' ' } else { c }).take(MAX_NAME_CHARS).collect();
    s.trim().to_string()
}

/// The view lines for the outline: `trap_outline=` (its text) and `trap_outline_name=`, when a
/// trap the picture depends on is the outline ([`crate::coloring_fields::outline_in_view`]). Empty
/// otherwise, so every other view is byte-identical to one written before the outline existed.
pub(crate) fn outline_view_lines(spec: &fractadyne_state::ColoringSpec) -> String {
    if spec.trap_outline.is_empty() || !crate::coloring_fields::outline_in_view(spec) {
        return String::new();
    }
    let name = outline_name(&spec.trap_outline_name);
    let name = if name.is_empty() { String::new() } else { format!("trap_outline_name={name}\n") };
    format!("trap_outline={}\n{name}", spec.trap_outline)
}

/// A view's outline keys into `spec`, after `coloring_fields::apply_view` (so the shapes are the
/// view's). The text is UNTRUSTED: parsed, and one that does not read is named in `problems`. A
/// view whose trap is the outline and that states its coloring (`states`) but carries none leaves
/// it without one, and says so: that trap then measures as the point. A view whose traps are not
/// the outline leaves the session's alone, as the table leaves what the picture does not use.
pub(crate) fn apply_outline_view(
    spec: &mut fractadyne_state::ColoringSpec,
    get: &dyn Fn(&str) -> Option<String>,
    states: bool,
    problems: &mut Vec<String>,
) {
    let used = crate::coloring_fields::outline_in_view(spec);
    match get("trap_outline").filter(|t| !t.trim().is_empty()) {
        Some(text) => match Outline::from_text(text.trim()) {
            Ok(o) => {
                spec.trap_outline = o.text().to_string();
                spec.trap_outline_name = get("trap_outline_name").map(|n| outline_name(&n)).unwrap_or_default();
            }
            Err(e) => {
                problems.push(format!("the trap outline does not read ({e}), so the outline trap measures as a point"));
                if used {
                    spec.trap_outline.clear();
                    spec.trap_outline_name.clear();
                }
            }
        },
        None if used && states => {
            problems.push("the view's trap is an outline but the view carries none, so it measures as a point".into());
            spec.trap_outline.clear();
            spec.trap_outline_name.clear();
        }
        None => {}
    }
}

/// An SVG file's outline (see [`outline_from_svg`]) and the file's name, or the name and why it is
/// not one; for Load SVG... and `--trap-outline`. The size first: a huge file is refused unread.
pub(crate) fn read_svg_outline(path: &std::path::Path) -> (String, Result<Outline, String>) {
    let name = outline_name(&path.file_name().map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned()));
    let read = match std::fs::metadata(path) {
        Ok(m) if m.len() > MAX_SVG_BYTES as u64 => Err(format!("the file is {} bytes; at most {MAX_SVG_BYTES}", m.len())),
        Ok(_) => std::fs::read(path).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    };
    let outline = read.and_then(|bytes| outline_from_svg(&bytes));
    (name, outline)
}

/// The view's outline from a session or a view file: its text parsed, or `None` (logged) for text
/// that is not one; empty text is none.
pub(crate) fn outline_from_text(text: &str, name: &str) -> Option<Outline> {
    if text.is_empty() {
        return None;
    }
    Outline::from_text(text)
        .map_err(|e| crate::diag::log_line("trap", &format!("the saved trap outline {name:?} was set aside: {e}")))
        .ok()
}

impl crate::FractadyneApp {
    /// Load SVG…: a drawing's outline as the view's trap outline. `true` when one was loaded; a file
    /// that is not one says why in a toast and changes nothing.
    pub(crate) fn load_trap_outline(&mut self, ctx: &eframe::egui::Context) -> bool {
        let dialog = rfd::FileDialog::new().add_filter(tr!(trap_outline_filter), &["svg"]).set_directory(self.dialog_dir_default());
        let Some(path) = crate::diag::modal(|| dialog.pick_file()) else {
            return false;
        };
        self.remember_dir(&path);
        self.load_trap_outline_from(&path, ctx)
    }

    /// An SVG file's outline as the view's (see [`Self::load_trap_outline`]).
    pub(crate) fn load_trap_outline_from(&mut self, path: &std::path::Path, ctx: &eframe::egui::Context) -> bool {
        let (name, outline) = read_svg_outline(path);
        match outline {
            Ok(o) => {
                let n = o.segment_count();
                crate::diag::log_line("trap", &format!("trap outline from {name}: {n} segments, longest cell list {}", o.longest_cell()));
                self.coloring.set_outline(Some(o), name.clone());
                self.set_toast(tr!(trap_outline_loaded, name = name, segments = n), ctx);
                true
            }
            Err(reason) => {
                crate::diag::log_line("trap", &format!("trap outline from {name} refused: {reason}"));
                self.set_toast(tr!(trap_outline_failed, name = name, reason = reason), ctx);
                false
            }
        }
    }

    /// The Drawing row under a trap whose shape is the outline: what it is, and Load SVG….
    pub(crate) fn trap_outline_row(&mut self, ui: &mut eframe::egui::Ui) {
        let mut load = false;
        let name = if self.coloring.trap_outline.is_some() { self.coloring.trap_outline_name.clone() } else { tr!(trap_outline_none) };
        crate::ui::labelled(ui, &tr!(panel_trap_outline), |ui| {
            load = ui.button(tr!(panel_trap_outline_load)).on_hover_text(tr!(panel_trap_outline_load_hover)).clicked();
            ui.add(eframe::egui::Label::new(name).truncate());
        });
        if load {
            let ctx = ui.ctx().clone();
            self.load_trap_outline(&ctx);
        }
    }

    /// After a shape menu: the outline picked with none loaded asks for one; refused or cancelled,
    /// the shape goes back to `before`.
    pub(crate) fn trap_outline_picked(&mut self, ctx: &eframe::egui::Context, picked: TrapType, before: TrapType) -> TrapType {
        if picked == TrapType::Outline && before != TrapType::Outline && self.coloring.trap_outline.is_none() && !self.load_trap_outline(ctx) {
            return before;
        }
        picked
    }
}

use fractadyne_core::coloring::TrapType;

#[cfg(test)]
#[path = "../tests/unit/trap_outline/tests.rs"]
mod tests;
