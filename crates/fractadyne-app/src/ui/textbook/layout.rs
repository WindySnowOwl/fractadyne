//! TeX's math layout (The TeXbook, Appendix G) in its OpenType form: every dimension comes from the
//! math font's MATH table ([`super::font`]), as in LuaTeX, XeTeX and MathJax's OpenType output.
//!
//! The input is a math list of [`Node`]s; the output an [`LBox`] — width, height above the
//! baseline, depth below it — holding positioned glyphs and rules. Coordinates are POINTS with y
//! pointing DOWN from the box's baseline (so a glyph above the baseline has a negative y), x from
//! its left edge. Sizes are the em sizes egui actually draws ([`MathFont::rendered_em`]).

use super::font::{font, MathFont, Part};

/// TeX's atom classes, which decide the space between neighbours (the whole table is kept, though
/// the formula language makes no punctuation atoms yet).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub(crate) enum Class {
    Ord,
    Op,
    Bin,
    Rel,
    Open,
    Close,
    Punct,
    Inner,
}

/// A math list element.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Node {
    /// Characters of the math font, set as given (a math-italic 𝑧 is U+1D467: see [`italic`]).
    Glyphs { text: String, class: Class },
    Frac { num: Vec<Node>, den: Vec<Node> },
    /// A base with an exponent and/or a subscript.
    Scripts { base: Box<Node>, sup: Option<Vec<Node>>, sub: Option<Vec<Node>> },
    /// A body between delimiters that grow with it (TeX's `\left … \right`).
    Fenced { open: char, close: char, body: Vec<Node> },
    Radical(Vec<Node>),
    Overline(Vec<Node>),
    /// An empty place still to be filled: a dashed box.
    Slot,
}

impl Node {
    pub(crate) fn glyphs(text: impl Into<String>, class: Class) -> Node {
        Node::Glyphs { text: text.into(), class }
    }
    /// A variable: its letters in math italic.
    pub(crate) fn var(name: &str) -> Node {
        Node::glyphs(name.chars().map(italic).collect::<String>(), Class::Ord)
    }
    /// An operator name (sin, log, Re), upright.
    pub(crate) fn op(name: &str) -> Node {
        Node::glyphs(name, Class::Op)
    }
    pub(crate) fn num(text: &str) -> Node {
        Node::glyphs(text, Class::Ord)
    }
    pub(crate) fn bin(ch: char) -> Node {
        Node::glyphs(ch.to_string(), Class::Bin)
    }
    pub(crate) fn rel(ch: char) -> Node {
        Node::glyphs(ch.to_string(), Class::Rel)
    }
}

/// The MATHEMATICAL ITALIC form of an ASCII letter (others unchanged). Unicode left U+1D455 empty:
/// italic h is PLANCK CONSTANT, U+210E.
pub(crate) fn italic(ch: char) -> char {
    match ch {
        'h' => '\u{210E}',
        'a'..='z' => char::from_u32(0x1D44E + (ch as u32 - 'a' as u32)).unwrap_or(ch),
        'A'..='Z' => char::from_u32(0x1D434 + (ch as u32 - 'A' as u32)).unwrap_or(ch),
        _ => ch,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Style {
    Display,
    Text,
    Script,
    ScriptScript,
}

/// A style and whether it is cramped (exponents sit lower in a cramped style).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct St {
    pub(crate) style: Style,
    pub(crate) cramped: bool,
}

impl St {
    pub(crate) const DISPLAY: St = St { style: Style::Display, cramped: false };
    fn is_script(self) -> bool {
        matches!(self.style, Style::Script | Style::ScriptScript)
    }
    fn is_display(self) -> bool {
        self.style == Style::Display
    }
    /// A fraction's numerator: one style smaller (TeX's table).
    fn num(self) -> St {
        let style = match self.style {
            Style::Display => Style::Text,
            Style::Text => Style::Script,
            _ => Style::ScriptScript,
        };
        St { style, cramped: self.cramped }
    }
    fn den(self) -> St {
        St { cramped: true, ..self.num() }
    }
    fn sup(self) -> St {
        let style = if self.is_script() { Style::ScriptScript } else { Style::Script };
        St { style, cramped: self.cramped }
    }
    fn sub(self) -> St {
        St { cramped: true, ..self.sup() }
    }
    fn cramp(self) -> St {
        St { cramped: true, ..self }
    }
}

/// What a laid-out box holds.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Item {
    /// A character of the math font: drawn at nominal `size` points with its baseline at `y`.
    Glyph { ch: char, size: f32, x: f32, y: f32 },
    /// A filled rectangle (fraction bar, vinculum, overline): top edge at `y`.
    Rule { x: f32, y: f32, w: f32, h: f32 },
    /// An empty place to fill: top edge at `y`.
    Slot { x: f32, y: f32, w: f32, h: f32 },
}

impl Item {
    fn shifted(&self, dx: f32, dy: f32) -> Item {
        match *self {
            Item::Glyph { ch, size, x, y } => Item::Glyph { ch, size, x: x + dx, y: y + dy },
            Item::Rule { x, y, w, h } => Item::Rule { x: x + dx, y: y + dy, w, h },
            Item::Slot { x, y, w, h } => Item::Slot { x: x + dx, y: y + dy, w, h },
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct LBox {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) depth: f32,
    pub(crate) items: Vec<Item>,
    /// The italic correction of a box that is a single italic character, for an exponent after it.
    italic: f32,
    /// A single character (TeX places scripts on one differently from a compound base).
    is_char: bool,
}

impl LBox {
    /// Append `b` with its left edge at `x` and its baseline `dy` below ours.
    fn put(&mut self, b: &LBox, x: f32, dy: f32) {
        self.items.extend(b.items.iter().map(|it| it.shifted(x, dy)));
        self.height = self.height.max(b.height - dy);
        self.depth = self.depth.max(b.depth + dy);
    }
}

/// How to lay out: the text size in points and the display's physical pixels per point.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ctx {
    pub(crate) size_pt: f32,
    pub(crate) ppp: f32,
}

impl Ctx {
    fn f(&self) -> &'static MathFont {
        font()
    }
    /// The nominal size of style `st`, in points (what egui is asked for).
    fn size(&self, st: St) -> f32 {
        let c = &self.f().constants;
        match st.style {
            Style::Display | Style::Text => self.size_pt,
            Style::Script => self.size_pt * c.script_percent / 100.0,
            Style::ScriptScript => self.size_pt * c.script_script_percent / 100.0,
        }
    }
    /// The em egui draws style `st` at, in points.
    fn em(&self, st: St) -> f32 {
        self.f().rendered_em(self.size(st), self.ppp)
    }
    /// Font units → points in style `st`.
    fn u(&self, st: St, units: f32) -> f32 {
        units * self.em(st) / self.f().units_per_em
    }
}

/// TeX's inter-atom spacing in mu (an 18th of the em): 0, 3 (thin), 4 (medium), 5 (thick); a
/// negative entry is only in the display and text styles.
fn space_mu(l: Class, r: Class, script: bool) -> f32 {
    use Class::*;
    let code: i8 = match (l, r) {
        (Ord, Op) | (Op, Ord) | (Op, Op) | (Close, Op) | (Inner, Op) => 1,
        (Ord, Bin) | (Bin, Ord) | (Bin, Op) | (Bin, Open) | (Bin, Inner) | (Close, Bin) | (Inner, Bin) => -2,
        (Ord, Rel) | (Op, Rel) | (Rel, Ord) | (Rel, Op) | (Rel, Open) | (Rel, Inner) | (Close, Rel) | (Inner, Rel) => -3,
        (Ord, Inner) | (Op, Inner) | (Close, Inner) | (Inner, Ord) | (Inner, Open) | (Inner, Punct) | (Inner, Inner) => -1,
        (Punct, _) => -1,
        _ => 0,
    };
    let mu = [0.0, 3.0, 4.0, 5.0][code.unsigned_abs() as usize];
    if code < 0 && script {
        0.0
    } else {
        mu
    }
}

/// A node's class as its left and right neighbours see it.
///
/// ⚠Growing parentheses space as ordinary ones do — an opening on the left, a closing on the right
/// — not as TeX's `\left…\right`, which makes an Inner atom and so a thin space after an operator
/// name: "sin (z)". LaTeX users write `\mathopen{}` to get rid of it; a function's argument here is
/// always parenthesised, so it is the rule.
fn classes_of(n: &Node) -> (Class, Class) {
    match n {
        Node::Glyphs { class, .. } => (*class, *class),
        Node::Scripts { base, .. } => {
            let (l, _) = classes_of(base);
            (l, if l == Class::Op { Class::Op } else { Class::Ord })
        }
        Node::Fenced { .. } => (Class::Open, Class::Close),
        Node::Frac { .. } => (Class::Inner, Class::Inner),
        Node::Radical(_) | Node::Overline(_) | Node::Slot => (Class::Ord, Class::Ord),
    }
}

fn class_of(n: &Node) -> Class {
    classes_of(n).0
}

/// Each node's (left, right) class after TeX's rule for binary operators: one with nothing to
/// operate on — first, or after an operator, a relation, an opening or punctuation, or last, or
/// before a relation, closing or punctuation — is set as an ordinary symbol (so a unary minus gets
/// no space around it).
fn effective_classes(nodes: &[Node]) -> Vec<(Class, Class)> {
    let mut cs: Vec<(Class, Class)> = nodes.iter().map(classes_of).collect();
    for i in 0..cs.len() {
        if cs[i].0 != Class::Bin {
            continue;
        }
        let before_ok = i > 0 && !matches!(cs[i - 1].1, Class::Bin | Class::Op | Class::Rel | Class::Open | Class::Punct);
        let after_ok = i + 1 < cs.len() && !matches!(cs[i + 1].0, Class::Rel | Class::Close | Class::Punct);
        if !(before_ok && after_ok) {
            cs[i] = (Class::Ord, Class::Ord);
        }
    }
    cs
}

/// A horizontal math list.
pub(crate) fn hlist(nodes: &[Node], st: St, ctx: &Ctx) -> LBox {
    hlist_with_marks(nodes, st, ctx).0
}

/// [`hlist`], with the x at which each node starts.
pub(crate) fn hlist_with_marks(nodes: &[Node], st: St, ctx: &Ctx) -> (LBox, Vec<f32>) {
    let classes = effective_classes(nodes);
    let mu = ctx.em(st) / 18.0;
    let mut out = LBox::default();
    let mut x = 0.0;
    let mut marks = Vec::with_capacity(nodes.len());
    for (i, n) in nodes.iter().enumerate() {
        if i > 0 {
            x += space_mu(classes[i - 1].1, classes[i].0, st.is_script()) * mu;
        }
        marks.push(x);
        let b = node(n, st, ctx);
        out.put(&b, x, 0.0);
        x += b.width;
        if nodes.len() == 1 {
            out.italic = b.italic;
            out.is_char = b.is_char;
        }
    }
    out.width = x;
    (out, marks)
}

fn node(n: &Node, st: St, ctx: &Ctx) -> LBox {
    match n {
        Node::Glyphs { text, class } => glyphs(text, *class, st, ctx),
        Node::Frac { num, den } => frac(num, den, st, ctx),
        Node::Scripts { base, sup, sub } => scripts(base, sup.as_deref(), sub.as_deref(), st, ctx),
        Node::Fenced { open, close, body } => fenced(*open, *close, body, st, ctx),
        Node::Radical(body) => radical(body, st, ctx),
        Node::Overline(body) => overline(body, st, ctx),
        Node::Slot => slot(st, ctx),
    }
}

fn glyphs(text: &str, class: Class, st: St, ctx: &Ctx) -> LBox {
    let f = ctx.f();
    let size = ctx.size(st);
    let mut b = LBox::default();
    let mut x = 0.0;
    let mut last = None;
    for ch in text.chars() {
        let (h, d) = f.ink(ch);
        b.items.push(Item::Glyph { ch, size, x, y: 0.0 });
        b.height = b.height.max(ctx.u(st, h));
        b.depth = b.depth.max(ctx.u(st, d));
        x += ctx.u(st, f.advance(ch));
        last = Some(ch);
    }
    let ic = last.map_or(0.0, |ch| ctx.u(st, f.italic_correction(ch)));
    b.is_char = text.chars().count() == 1;
    b.italic = ic;
    // An italic letter leans past its advance; TeX adds the correction unless a script follows
    // (`scripts` places the exponent with it instead).
    b.width = x + if class == Class::Ord { ic } else { 0.0 };
    b
}

fn frac(num: &[Node], den: &[Node], st: St, ctx: &Ctx) -> LBox {
    let c = &ctx.f().constants;
    let disp = st.is_display();
    let (nb, db) = (hlist(num, st.num(), ctx), hlist(den, st.den(), ctx));
    let t = ctx.u(st, c.fraction_rule_thickness);
    let axis = ctx.u(st, c.axis_height);
    let pick = |display: f32, text: f32| ctx.u(st, if disp { display } else { text });
    let mut up = pick(c.fraction_numerator_display_style_shift_up, c.fraction_numerator_shift_up);
    let mut down = pick(c.fraction_denominator_display_style_shift_down, c.fraction_denominator_shift_down);
    let num_gap = pick(c.fraction_num_display_style_gap_min, c.fraction_numerator_gap_min);
    let den_gap = pick(c.fraction_denom_display_style_gap_min, c.fraction_denominator_gap_min);
    // The numerator's bottom clears the bar by `num_gap`, the denominator's top by `den_gap`.
    up = up.max(axis + t / 2.0 + num_gap + nb.depth);
    down = down.max(den_gap + db.height - axis + t / 2.0);
    // TeX's null delimiters on either side (\nulldelimiterspace, 1.2pt at 10pt).
    let pad = 0.12 * ctx.em(st);
    let inner = nb.width.max(db.width);
    let mut b = LBox { width: inner + 2.0 * pad, ..Default::default() };
    b.put(&nb, pad + (inner - nb.width) / 2.0, -up);
    b.put(&db, pad + (inner - db.width) / 2.0, down);
    b.items.push(Item::Rule { x: pad, y: -(axis + t / 2.0), w: inner, h: t });
    b.height = b.height.max(axis + t / 2.0);
    b
}

fn scripts(base: &Node, sup: Option<&[Node]>, sub: Option<&[Node]>, st: St, ctx: &Ctx) -> LBox {
    let c = &ctx.f().constants;
    let bb = node(base, st, ctx);
    // A single italic letter: its exponent goes after the italic correction, its subscript tucks
    // under the lean (TeX's rule 17). `glyphs` counted the correction into an Ord run's width.
    let ic = if bb.is_char { bb.italic } else { 0.0 };
    let counted = bb.is_char && matches!(base, Node::Glyphs { class: Class::Ord, .. });
    let base_w = if counted { bb.width - ic } else { bb.width };
    let sp = sup.map(|s| hlist(s, st.sup(), ctx));
    let sb = sub.map(|s| hlist(s, st.sub(), ctx));
    let mut out = LBox::default();
    out.put(&bb, 0.0, 0.0);
    let mut u = 0.0;
    let mut v = 0.0;
    if let Some(s) = &sp {
        let shift = if st.cramped { c.superscript_shift_up_cramped } else { c.superscript_shift_up };
        u = ctx.u(st, shift).max(ctx.u(st, c.superscript_bottom_min) + s.depth);
        if !bb.is_char {
            u = u.max(bb.height - ctx.u(st, c.superscript_baseline_drop_max));
        }
    }
    if let Some(s) = &sb {
        v = ctx.u(st, c.subscript_shift_down).max(s.height - ctx.u(st, c.subscript_top_max));
        if !bb.is_char {
            v = v.max(bb.depth + ctx.u(st, c.subscript_baseline_drop_min));
        }
    }
    if let (Some(p), Some(q)) = (&sp, &sb) {
        // Keep the two apart; then lift a low exponent, lowering the subscript as much.
        let gap = (u - p.depth) - (q.height - v);
        let need = ctx.u(st, c.sub_superscript_gap_min);
        if gap < need {
            v += need - gap;
        }
        let psi = ctx.u(st, c.superscript_bottom_max_with_subscript) - (u - p.depth);
        if psi > 0.0 {
            u += psi;
            v -= psi;
        }
    }
    let mut w = base_w;
    if let Some(p) = &sp {
        out.put(p, base_w + ic, -u);
        w = w.max(base_w + ic + p.width);
    }
    if let Some(q) = &sb {
        out.put(q, base_w, v);
        w = w.max(base_w + q.width);
    }
    out.width = w + ctx.u(st, c.space_after_script);
    out
}

/// A delimiter `ch` at least `target` points tall in total, centred on the math axis.
fn delimiter(ch: char, target: f32, st: St, ctx: &Ctx) -> LBox {
    let axis = ctx.u(st, ctx.f().constants.axis_height);
    let b = sized_glyph(ch, target, st, ctx);
    let shift = axis - (b.height - b.depth) / 2.0; // move up by this to centre on the axis
    let mut out = LBox { width: b.width, ..Default::default() };
    out.put(&b, 0.0, -shift);
    out
}

/// `ch` at least `need` points tall in total: the first size variant tall enough, else an assembly
/// of the font's parts, else (a font without one) the largest variant there is. On its own baseline.
fn sized_glyph(ch: char, need: f32, st: St, ctx: &Ctx) -> LBox {
    let f = ctx.f();
    let variants = f.vertical_variants(ch);
    let mut b = match variants.iter().find(|(_, adv)| ctx.u(st, *adv) >= need) {
        Some(&(g, _)) => glyphs(&g.to_string(), Class::Open, st, ctx),
        None => assembly(&f.vertical_assembly(ch), need, st, ctx)
            .unwrap_or_else(|| glyphs(&variants.last().map_or(ch, |v| v.0).to_string(), Class::Open, st, ctx)),
    };
    b.is_char = false;
    b.italic = 0.0;
    b
}

/// The font's parts for a stretchy glyph, stacked bottom to top to at least `target` points.
///
/// OpenType's rule: neighbouring parts overlap by at least `minConnectorOverlap` and at most the
/// shorter of the two connectors. So `k` extender repeats reach anything between
/// Σadvance − Σ(most overlap) and Σadvance − joints·(least overlap); the fewest repeats that reach
/// `target` are used, every joint overlapping by the same amount, as much as still reaches it.
fn assembly(parts: &[Part], target: f32, st: St, ctx: &Ctx) -> Option<LBox> {
    if parts.is_empty() {
        return None;
    }
    let f = ctx.f();
    let u = |v: f32| ctx.u(st, v);
    let least = u(f.constants.min_connector_overlap);
    let has_extender = parts.iter().any(|p| p.extender);
    for reps in 0..64usize {
        if reps > 0 && !has_extender {
            break;
        }
        let seq: Vec<Part> =
            parts.iter().flat_map(|p| std::iter::repeat_n(*p, if p.extender { reps } else { 1 })).collect();
        let total: f32 = seq.iter().map(|p| u(p.full_advance)).sum();
        let most: Vec<f32> = seq.windows(2).map(|w| u(w[0].end_connector.min(w[1].start_connector))).collect();
        let joints = most.len() as f32;
        if total - joints * least < target {
            continue; // too short even overlapping as little as allowed: one more extender
        }
        let cap = most.iter().copied().fold(f32::INFINITY, f32::min).max(least);
        let overlap = if joints > 0.0 { ((total - target) / joints).clamp(least, cap) } else { 0.0 };
        let size = ctx.size(st);
        let mut b = LBox::default();
        let mut bottom = 0.0; // height (upwards) of this part's bottom edge
        for (i, p) in seq.iter().enumerate() {
            if i > 0 {
                bottom -= overlap;
            }
            let (h, d) = f.ink(p.ch);
            let baseline = bottom + u(d); // the glyph's ink bottom sits on `bottom`
            b.items.push(Item::Glyph { ch: p.ch, size, x: 0.0, y: -baseline });
            b.height = b.height.max(baseline + u(h));
            b.depth = b.depth.max(u(d) - baseline);
            b.width = b.width.max(u(f.advance(p.ch)));
            bottom += u(p.full_advance);
        }
        return Some(b);
    }
    None
}

fn fenced(open: char, close: char, body: &[Node], st: St, ctx: &Ctx) -> LBox {
    let c = &ctx.f().constants;
    let inner = hlist(body, st, ctx);
    let axis = ctx.u(st, c.axis_height);
    // TeX's sizing: cover the body's extent about the axis, at least 90.1% of it, short of it by
    // no more than half an em (\delimiterfactor 901, \delimitershortfall 5pt at 10pt).
    let delta = (inner.height - axis).max(inner.depth + axis).max(0.0);
    let mut target = (2.0 * delta * 0.901).max(2.0 * delta - 0.5 * ctx.em(st));
    if inner.height + inner.depth < ctx.u(st, c.delimited_sub_formula_min_height) {
        target = 0.0; // a short body keeps the ordinary glyph
    }
    let (lb, rb) = (delimiter(open, target, st, ctx), delimiter(close, target, st, ctx));
    let mut b = LBox::default();
    b.put(&lb, 0.0, 0.0);
    b.put(&inner, lb.width, 0.0);
    b.put(&rb, lb.width + inner.width, 0.0);
    b.width = lb.width + inner.width + rb.width;
    // Delimiters that did not grow are the ordinary glyphs, and an exponent after them goes where
    // LaTeX puts one after a plain `)` or `|` — the character rule, superscriptShiftUp — not where
    // it puts one after `\right)`, the box's height less the drop (|𝑧|² sat visibly higher).
    b.is_char = target == 0.0;
    b
}

fn radical(body: &[Node], st: St, ctx: &Ctx) -> LBox {
    let c = &ctx.f().constants;
    let inner = hlist(body, st.cramp(), ctx);
    let t = ctx.u(st, c.radical_rule_thickness);
    let mut gap = ctx.u(st, if st.is_display() { c.radical_display_style_vertical_gap } else { c.radical_vertical_gap });
    let need = inner.height + inner.depth + gap + t;
    let sign = sized_glyph('\u{221A}', need, st, ctx);
    // A sign taller than needed gives half its excess to the gap (TeX's rule 11).
    let excess = (sign.height + sign.depth) - need;
    if excess > 0.0 {
        gap += excess / 2.0;
    }
    let top = inner.height + gap + t; // the vinculum's top edge, above the baseline
    let shift = top - sign.height; // raise the sign so its top meets the vinculum's
    let mut b = LBox::default();
    b.put(&sign, 0.0, -shift);
    b.put(&inner, sign.width, 0.0);
    b.items.push(Item::Rule { x: sign.width, y: -top, w: inner.width, h: t });
    b.width = sign.width + inner.width;
    b.height = b.height.max(top + ctx.u(st, c.radical_extra_ascender));
    b
}

fn overline(body: &[Node], st: St, ctx: &Ctx) -> LBox {
    let c = &ctx.f().constants;
    let inner = hlist(body, st.cramp(), ctx);
    let gap = ctx.u(st, c.overbar_vertical_gap);
    let t = ctx.u(st, c.overbar_rule_thickness);
    let mut b = LBox::default();
    b.put(&inner, 0.0, 0.0);
    let top = inner.height + gap + t;
    b.items.push(Item::Rule { x: 0.0, y: -top, w: inner.width, h: t });
    b.width = inner.width;
    b.height = top + ctx.u(st, c.overbar_extra_ascender);
    b
}

fn slot(st: St, ctx: &Ctx) -> LBox {
    let em = ctx.em(st);
    let (w, h, d) = (0.55 * em, 0.68 * em, 0.12 * em);
    LBox { width: w, height: h, depth: d, items: vec![Item::Slot { x: 0.0, y: -h, w, h: h + d }], ..Default::default() }
}

/// A formula of statements, one row each in display style, aligned at each row's first relation
/// (`=`) as `align*` aligns: the boxes' items in points from the TOP-LEFT of the whole.
pub(crate) struct Laid {
    pub(crate) size: egui::Vec2,
    pub(crate) items: Vec<Item>,
}

pub(crate) fn rows(rows: &[Vec<Node>], ctx: &Ctx) -> Laid {
    let laid: Vec<(LBox, Option<f32>)> = rows
        .iter()
        .map(|r| {
            let (b, marks) = hlist_with_marks(r, St::DISPLAY, ctx);
            let rel = r.iter().position(|n| class_of(n) == Class::Rel).map(|i| marks[i]);
            (b, rel)
        })
        .collect();
    let align = laid.iter().filter_map(|(_, r)| *r).fold(0.0_f32, f32::max);
    let gap = 0.35 * ctx.em(St::DISPLAY);
    let mut items = Vec::new();
    let (mut y, mut width) = (0.0, 0.0_f32);
    for (i, (b, rel)) in laid.iter().enumerate() {
        let dx = rel.map_or(0.0, |r| align - r);
        if i > 0 {
            y += gap;
        }
        let baseline = y + b.height;
        items.extend(b.items.iter().map(|it| it.shifted(dx, baseline)));
        y = baseline + b.depth;
        width = width.max(dx + b.width);
    }
    Laid { size: egui::vec2(width, y), items }
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod tests;
