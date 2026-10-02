//! Parametric and context-sensitive systems (design/lsystems.md §9.2; ABOP §1.8, §1.10).
//!
//! A parametric module carries numbers (`A(4, 4)`), and a production applies only where its
//! condition holds (`A(x, y) : y <= 3 = A(x*2, x+y)`); a context-sensitive production applies only
//! between given neighbours (`b < a = b`). Either way a node's rewrite depends on more than its
//! symbol and depth, so the tables that let the walk skip a subtree do not exist: these systems are
//! drawn by building the word, a generation at a time, at a fixed order — within a budget of
//! modules — and running the turtle over it, in double precision.
//!
//! Matching follows ABOP. A production matches a module whose letter is its predecessor's, with as
//! many parameters as it names, whose contexts are found, and whose condition holds; the first
//! production in the order written that matches is applied, and a module no production matches
//! stays as it is. In a bracketed word a left context is the module before on the path to the
//! root — a branch to the left (`[…]`) is stepped over, and the start of the branch the module is
//! in is stepped out of — and a right context the module after on the same branch, stepping over
//! branches; the end of a branch has none. Symbols named by `ignore` are not seen as contexts
//! (ABOP's `#ignore: +-F`).

use super::expr::{self, Expr};
use super::system::{fail, parse_word_args, LSystem, ParseError, Role, Tok};
use super::walk::{Drawn, Polygon, Segment, View, WalkStats};
use std::f64::consts::TAU;

/// A module as written in a production's successor (its arguments expressions) or in the axiom
/// (its arguments numbers).
#[derive(Clone, Debug, PartialEq)]
pub struct Module<A> {
    pub tok: Tok,
    pub args: Vec<A>,
}

/// A module of a predecessor or a context: its letter, and the formal parameters it binds (indices
/// into the production's).
#[derive(Clone, Debug, PartialEq)]
struct Pattern {
    key: u8,
    params: Vec<u16>,
}

/// A production: `left < pred > right : condition = successor`.
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    left: Vec<Pattern>,
    pred: Pattern,
    right: Vec<Pattern>,
    cond: Option<Expr>,
    succ: Vec<Module<Expr>>,
    /// How many formal parameters it names.
    vars: usize,
}

/// A parametric or context-sensitive system's grammar.
#[derive(Clone, Debug, PartialEq)]
pub struct Expanded {
    pub axiom: Vec<Module<f64>>,
    pub rules: Vec<Rule>,
    /// The letters context matching does not see.
    pub ignore: Vec<u8>,
    /// The definitions, `ignore`, axiom and productions as written, in order (the system's text).
    pub lines: Vec<String>,
    /// Whether a word holds a bracket, or a colour command (for the default colouring).
    pub branches: bool,
    pub colour_index: bool,
}

/// The letter a module is matched by: a symbol's own, a command's character.
pub(crate) fn key(t: &Tok) -> u8 {
    match *t {
        Tok::Sym(c) => c,
        Tok::Turn(k) if k > 0 => b'+',
        Tok::Turn(_) => b'-',
        Tok::Around => b'|',
        Tok::Reverse => b'!',
        Tok::Push => b'[',
        Tok::Pop => b']',
        Tok::Scale(_) => b'@',
        Tok::TurnBy(a) if a >= 0.0 => b'\\',
        Tok::TurnBy(_) => b'/',
        Tok::SetColour(_) => 0,
        Tok::AddColour(n) if n >= 0 => b'<',
        Tok::AddColour(_) => b'>',
        Tok::PolyStart => b'{',
        Tok::PolyEnd => b'}',
        Tok::Vertex => b'.',
    }
}

/// Where a separator sits in a production's left side: the first `ch` outside parentheses.
fn find_top(s: &str, ch: u8) -> Option<usize> {
    let mut depth = 0i32;
    for (i, &c) in s.as_bytes().iter().enumerate() {
        match c {
            b'(' => depth += 1,
            b')' => depth -= 1,
            c if c == ch && depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// Whether a production's left side (the text before its `=`) is a parametric or
/// context-sensitive one: it names a context (`<`, `>`), a condition (`:`) or formal parameters.
pub(crate) fn is_expanded_lhs(lhs: &str) -> bool {
    let t = lhs.trim();
    if t.contains('<') || t.contains('>') || t.contains(':') {
        return true;
    }
    // `X (0.5)` is a stochastic weight; `X(a, b)` names parameters.
    match (t.find('('), t.rfind(')')) {
        (Some(o), Some(c)) if c > o => t[o + 1..c].trim().parse::<f64>().is_err(),
        _ => false,
    }
}

/// The text of one of the lines that make a parametric system, with where it is.
pub(crate) enum Item {
    Define { line: usize, col: usize, name: String, value: String },
    Ignore { line: usize, col: usize, chars: String },
    Axiom { line: usize, col: usize, text: String },
    Production { line: usize, col: usize, lhs: String, rhs: String, rhs_col: usize },
}

/// Whether `s` holds an `=` that is not part of `==`, `<=`, `>=` or `!=`.
fn bare_eq(s: &str) -> bool {
    let b = s.as_bytes();
    (0..b.len()).any(|i| {
        b[i] == b'=' && !matches!(i.checked_sub(1).map(|j| b[j]), Some(b'=' | b'<' | b'>' | b'!')) && b.get(i + 1) != Some(&b'=')
    })
}

fn is_name(s: &str) -> bool {
    let mut c = s.chars();
    c.next().is_some_and(|f| f.is_ascii_alphabetic() || f == '_') && c.all(|x| x.is_ascii_alphanumeric() || x == '_')
}

/// Builds the grammar from its lines (in the order written: a `define` is seen by what follows it).
pub(crate) fn build(items: &[Item]) -> Result<Expanded, ParseError> {
    let mut consts: Vec<(String, f64)> = Vec::new();
    let mut ignore = Vec::new();
    let mut axiom = None;
    let mut rules = Vec::new();
    let mut lines = Vec::new();
    let mut branches = false;
    let mut colour_index = false;
    let mut note = |w: &[(Tok, Vec<(String, usize)>)]| {
        branches |= w.iter().any(|(t, _)| *t == Tok::Push);
        colour_index |= w.iter().any(|(t, _)| matches!(t, Tok::SetColour(_) | Tok::AddColour(_)));
    };
    let expr_err = |line: usize, col: usize, e: expr::ExprError| ParseError { line, col: col + e.at, message: e.message };
    for item in items {
        match item {
            Item::Define { line, col, name, value } => {
                if !is_name(name) {
                    return fail(*line, *col, format!("'{name}' cannot be a constant's name (letters, digits and _)"));
                }
                let e = expr::parse(value, &[], &consts).map_err(|e| expr_err(*line, *col + name.len() + 1, e))?;
                consts.push((name.clone(), e.eval(&[])));
                lines.push(format!("define {name} {}", value.trim()));
            }
            Item::Ignore { line, col, chars } => {
                for (k, c) in chars.bytes().enumerate() {
                    if c.is_ascii_whitespace() || c == b':' {
                        continue;
                    }
                    if matches!(c, b'[' | b']') {
                        return fail(*line, col + k, "brackets cannot be ignored (they shape the contexts)");
                    }
                    ignore.push(c);
                }
                lines.push(format!("ignore {}", chars.trim().trim_start_matches(':').trim()));
            }
            Item::Axiom { line, col, text } => {
                if axiom.is_some() {
                    return fail(*line, 0, "a second axiom");
                }
                let w = parse_word_args(text, *line, *col)?;
                note(&w);
                let mut mods = Vec::new();
                for (tok, args) in w {
                    let mut vals = Vec::new();
                    for (a, c) in args {
                        vals.push(expr::parse(&a, &[], &consts).map_err(|e| expr_err(*line, c, e))?.eval(&[]));
                    }
                    mods.push(Module { tok, args: vals });
                }
                axiom = Some(mods);
                lines.push(format!("axiom {}", text.trim()));
            }
            Item::Production { line, col, lhs, rhs, rhs_col } => {
                let rule = production(*line, *col, lhs, rhs, *rhs_col, &consts, &mut note)?;
                rules.push(rule);
                // A condition with ABOP's `=` for equality keeps its `→`, or it would read as the
                // production's `=`.
                let sep = if bare_eq(lhs) { "→" } else { "=" };
                lines.push(format!("{} {sep} {}", lhs.trim(), rhs.trim()));
            }
        }
    }
    let axiom = axiom.filter(|a| !a.is_empty()).ok_or_else(|| ParseError {
        line: 0,
        col: 0,
        message: "no axiom (the word the system starts from: 'axiom F')".into(),
    })?;
    Ok(Expanded { axiom, rules, ignore, lines, branches, colour_index })
}

/// One production: `left < pred > right : condition = successor`.
fn production(
    line: usize,
    col: usize,
    lhs: &str,
    rhs: &str,
    rhs_col: usize,
    consts: &[(String, f64)],
    note: &mut dyn FnMut(&[(Tok, Vec<(String, usize)>)]),
) -> Result<Rule, ParseError> {
    let (pat, cond) = match find_top(lhs, b':') {
        Some(k) => (&lhs[..k], Some((&lhs[k + 1..], col + k + 1))),
        None => (lhs, None),
    };
    let (left, rest, rest_col) = match find_top(pat, b'<') {
        Some(k) => (Some((&pat[..k], col)), &pat[k + 1..], col + k + 1),
        None => (None, pat, col),
    };
    let (mid, right) = match find_top(rest, b'>') {
        Some(k) => ((&rest[..k], rest_col), Some((&rest[k + 1..], rest_col + k + 1))),
        None => ((rest, rest_col), None),
    };
    let mut vars: Vec<String> = Vec::new();
    let patterns = |part: Option<(&str, usize)>, vars: &mut Vec<String>| -> Result<Vec<Pattern>, ParseError> {
        let Some((text, c0)) = part else { return Ok(Vec::new()) };
        // A wildcard: any context (ABOP's `*`).
        if text.trim() == "*" {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for (tok, args) in parse_word_args(text, line, c0)? {
            if matches!(tok, Tok::Push | Tok::Pop) {
                return fail(line, c0, "a context or predecessor cannot hold a bracket");
            }
            let mut params = Vec::new();
            for (a, c) in args {
                let name = a.trim();
                if !is_name(name) {
                    return fail(line, c, format!("'{name}' is not a parameter name (letters, digits and _)"));
                }
                params.push(vars.len() as u16);
                vars.push(name.to_string());
            }
            out.push(Pattern { key: key(&tok), params });
        }
        Ok(out)
    };
    let left = patterns(left, &mut vars)?;
    let pred = patterns(Some(mid), &mut vars)?;
    let right = patterns(right, &mut vars)?;
    let [pred] = <[Pattern; 1]>::try_from(pred).map_err(|_| ParseError {
        line,
        col: mid.1,
        message: "a production rewrites one module (its predecessor: 'A(x)', or 'a < A > b')".into(),
    })?;
    let expr_err = |c: usize, e: expr::ExprError| ParseError { line, col: c + e.at, message: e.message };
    let cond = match cond {
        Some((text, c)) if !text.trim().is_empty() => Some(expr::parse(text, &vars, consts).map_err(|e| expr_err(c, e))?),
        _ => None,
    };
    let w = parse_word_args(rhs, line, rhs_col)?;
    note(&w);
    let mut succ = Vec::new();
    for (tok, args) in w {
        let mut es = Vec::new();
        for (a, c) in args {
            es.push(expr::parse(&a, &vars, consts).map_err(|e| expr_err(c, e))?);
        }
        succ.push(Module { tok, args: es });
    }
    Ok(Rule { left, pred, right, cond, succ, vars: vars.len() })
}

/// One module of a built word: its command or symbol and where its parameters are.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WMod {
    pub tok: Tok,
    start: u32,
    n: u16,
}

/// A parametric word: its modules, their parameters side by side.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Word {
    pub mods: Vec<WMod>,
    pub params: Vec<f64>,
}

impl Word {
    pub fn args(&self, m: &WMod) -> &[f64] {
        &self.params[m.start as usize..m.start as usize + m.n as usize]
    }

    fn push(&mut self, tok: Tok, args: impl Iterator<Item = f64>) {
        let start = self.params.len() as u32;
        self.params.extend(args);
        let n = (self.params.len() as u32 - start) as u16;
        self.mods.push(WMod { tok, start, n });
    }

    /// The word as text (`B(1)B(4)A(1,0)`), for tests and diagnostics.
    pub fn text(&self) -> String {
        let mut s = String::new();
        for m in &self.mods {
            s.push_str(&super::system::word_text(&[m.tok]));
            let a = self.args(m);
            if !a.is_empty() {
                let parts: Vec<String> = a.iter().map(|v| format!("{v}")).collect();
                s.push_str(&format!("({})", parts.join(",")));
            }
        }
        s
    }
}

/// The most modules a word is built to.
pub const EXPAND_BUDGET: usize = 2_000_000;

/// A word built to an order.
#[derive(Clone, Debug)]
pub struct Expansion {
    pub word: Word,
    /// The order built: the one asked for, or the last whose word fits the budget.
    pub order: u32,
    pub wanted: u32,
    /// The lines it draws (what position along the curve is a fraction of), and the deepest
    /// bracket nesting.
    pub segments: u64,
    pub max_brackets: u16,
}

impl Expansion {
    /// Whether it stopped short of the order asked for.
    pub fn short(&self) -> bool {
        self.order < self.wanted
    }
}

/// Builds `sys`'s word to `order` (at most `budget` modules).
pub fn expand(sys: &LSystem, e: &Expanded, order: u32, budget: usize) -> Expansion {
    let mut w = Word::default();
    for m in &e.axiom {
        w.push(m.tok, m.args.iter().copied());
    }
    let mut by_key: Vec<Vec<usize>> = vec![Vec::new(); 256];
    for (i, r) in e.rules.iter().enumerate() {
        by_key[r.pred.key as usize].push(i);
    }
    let mut ignore = [false; 256];
    for &c in &e.ignore {
        ignore[c as usize] = true;
    }
    let contexts = e.rules.iter().any(|r| !r.left.is_empty() || !r.right.is_empty());
    let most_vars = e.rules.iter().map(|r| r.vars).max().unwrap_or(0);
    let mut done = 0;
    while done < order && !e.rules.is_empty() {
        match step(e, &by_key, &ignore, contexts, most_vars, &w, budget) {
            Some(next) => w = next,
            None => break,
        }
        done += 1;
    }
    let (mut segments, mut depth, mut max_brackets, mut poly) = (0u64, 0u16, 0u16, 0u32);
    for m in &w.mods {
        match m.tok {
            Tok::Sym(c) if poly == 0 && sys.roles[c as usize] == Role::Draw => segments += 1,
            Tok::Push => {
                depth += 1;
                max_brackets = max_brackets.max(depth);
            }
            Tok::Pop => depth = depth.saturating_sub(1),
            Tok::PolyStart => poly += 1,
            Tok::PolyEnd => poly = poly.saturating_sub(1),
            _ => {}
        }
    }
    // With no productions every order is the axiom.
    let reached = if e.rules.is_empty() { order } else { done };
    Expansion { word: w, order: reached, wanted: order, segments, max_brackets }
}

/// Each bracket's partner.
fn partners(w: &Word) -> Vec<u32> {
    let mut out = vec![u32::MAX; w.mods.len()];
    let mut stack = Vec::new();
    for (i, m) in w.mods.iter().enumerate() {
        match m.tok {
            Tok::Push => stack.push(i),
            Tok::Pop => {
                if let Some(o) = stack.pop() {
                    out[o] = i as u32;
                    out[i] = o as u32;
                }
            }
            _ => {}
        }
    }
    out
}

/// The module before `i` on the path to the root (ABOP §1.8), or `None` at the root.
fn left_of(w: &Word, pair: &[u32], ignore: &[bool; 256], i: usize) -> Option<usize> {
    let mut j = i;
    loop {
        if j == 0 {
            return None;
        }
        j -= 1;
        match key(&w.mods[j].tok) {
            // A branch to the left: stepped over, to its `[` (the next step moves past it).
            b']' => {
                let o = pair[j];
                if o == u32::MAX {
                    return None;
                }
                j = o as usize;
            }
            // The start of this module's branch: out, to the module the branch grows from.
            b'[' => {}
            k if ignore[k as usize] => {}
            _ => return Some(j),
        }
    }
}

/// The module after `i` on its branch, stepping over branches, or `None` at the branch's end.
fn right_of(w: &Word, pair: &[u32], ignore: &[bool; 256], i: usize) -> Option<usize> {
    let mut j = i + 1;
    while j < w.mods.len() {
        match key(&w.mods[j].tok) {
            b'[' => {
                let o = pair[j];
                if o == u32::MAX {
                    return None;
                }
                j = o as usize + 1;
            }
            b']' => return None,
            k if ignore[k as usize] => j += 1,
            _ => return Some(j),
        }
    }
    None
}

/// Whether module `m` of `w` is `p`; binds its parameters if so.
fn bind(p: &Pattern, w: &Word, m: &WMod, vals: &mut [f64]) -> bool {
    let a = w.args(m);
    if key(&m.tok) != p.key || a.len() != p.params.len() {
        return false;
    }
    for (&k, &v) in p.params.iter().zip(a) {
        vals[k as usize] = v;
    }
    true
}

/// One generation: every module rewritten by the first production that matches it (the contexts
/// read from this generation's word). `None` if the next word passes `budget` modules.
fn step(
    e: &Expanded,
    by_key: &[Vec<usize>],
    ignore: &[bool; 256],
    contexts: bool,
    most_vars: usize,
    w: &Word,
    budget: usize,
) -> Option<Word> {
    let pair = if contexts { partners(w) } else { Vec::new() };
    let mut next = Word::default();
    let mut vals = vec![0.0; most_vars];
    for (i, m) in w.mods.iter().enumerate() {
        let rule = by_key[key(&m.tok) as usize].iter().map(|&r| &e.rules[r]).find(|r| {
            if !bind(&r.pred, w, m, &mut vals) {
                return false;
            }
            let mut at = i;
            for p in r.left.iter().rev() {
                match left_of(w, &pair, ignore, at) {
                    Some(n) if bind(p, w, &w.mods[n], &mut vals) => at = n,
                    _ => return false,
                }
            }
            let mut at = i;
            for p in &r.right {
                match right_of(w, &pair, ignore, at) {
                    Some(n) if bind(p, w, &w.mods[n], &mut vals) => at = n,
                    _ => return false,
                }
            }
            r.cond.as_ref().is_none_or(|c| c.holds(&vals))
        });
        match rule {
            Some(r) => {
                for s in &r.succ {
                    next.push(s.tok, s.args.iter().map(|a| a.eval(&vals)));
                }
            }
            None => next.push(m.tok, w.args(m).iter().copied()),
        }
        if next.mods.len() > budget {
            return None;
        }
    }
    Some(next)
}

/// Runs the turtle over a built word, handing what falls in `view` to `sink` as the walk does
/// (pixels from the view's centre, y up): a step of 1 is a world unit; `F(l)` steps `l`, `+(a)`
/// turns `a` degrees, `@(f)` scales the step by `f`.
pub fn draw(sys: &LSystem, x: &Expansion, view: &View, budget: u64, sink: &mut dyn FnMut(Drawn)) -> WalkStats {
    #[derive(Clone, Copy)]
    struct S {
        pos: [f64; 2],
        angle: f64,
        len: f64,
        flip: bool,
        colour: i32,
        depth: u16,
    }
    let delta = sys.angle.degrees();
    let around = match sys.angle.division() {
        Some(n) => f64::from(n / 2) * 360.0 / f64::from(n),
        None => 180.0,
    };
    let inv = 1.0 / view.upp;
    let px = |p: [f64; 2]| [(p[0] - view.centre[0]) * inv, (p[1] - view.centre[1]) * inv];
    let mut s = S { pos: [0.0, 0.0], angle: sys.heading.to_radians(), len: 1.0, flip: false, colour: 1, depth: 0 };
    let mut stack = Vec::new();
    let mut open: Vec<Polygon> = Vec::new();
    let mut stats = WalkStats::default();
    let mut index = 0.0;
    let w = &x.word;
    for m in &w.mods {
        let a = w.args(m);
        let arg = |d: f64| a.first().copied().unwrap_or(d);
        let sign = if s.flip { -1.0 } else { 1.0 };
        s.angle = s.angle.rem_euclid(TAU);
        match m.tok {
            Tok::Sym(c) => {
                let role = sys.roles[c as usize];
                if role == Role::None {
                    continue;
                }
                let l = s.len * arg(1.0);
                if !l.is_finite() {
                    continue;
                }
                let from = s.pos;
                s.pos = [from[0] + l * s.angle.cos(), from[1] + l * s.angle.sin()];
                if let Some(p) = open.last_mut() {
                    p.pts.push(px(s.pos));
                    continue;
                }
                if role == Role::Draw {
                    let (qa, qb) = (px(from), px(s.pos));
                    let mid = [0.5 * (qa[0] + qb[0]), 0.5 * (qa[1] + qb[1])];
                    let half = 0.5 * (qb[0] - qa[0]).hypot(qb[1] - qa[1]);
                    if view.meets(mid, half + view.margin) {
                        if stats.segments + stats.vertices >= budget {
                            stats.stopped = true;
                            return stats;
                        }
                        let heading = (s.angle / TAU).rem_euclid(1.0);
                        sink(Drawn::Segment(&Segment { a: qa, b: qb, index, span: 1.0, depth: s.depth, heading, colour: s.colour }));
                        stats.segments += 1;
                    }
                    index += 1.0;
                }
            }
            Tok::Turn(k) => s.angle += sign * f64::from(k) * arg(delta).to_radians(),
            Tok::Around => s.angle += sign * around.to_radians(),
            Tok::TurnBy(t) => s.angle += sign * (t * arg(1.0)).to_radians(),
            Tok::Reverse => s.flip = !s.flip,
            Tok::Push => {
                stack.push(s);
                s.depth = s.depth.saturating_add(1);
            }
            Tok::Pop => {
                if let Some(t) = stack.pop() {
                    s = t;
                }
            }
            Tok::Scale(f) => s.len *= f * arg(1.0),
            Tok::SetColour(c) => s.colour = c,
            Tok::AddColour(c) => s.colour = s.colour.wrapping_add((f64::from(c) * arg(1.0)) as i32),
            Tok::PolyStart => open.push(Polygon {
                pts: vec![px(s.pos)],
                index,
                depth: s.depth,
                heading: (s.angle / TAU).rem_euclid(1.0),
                colour: s.colour,
            }),
            Tok::Vertex => {
                if let Some(p) = open.last_mut() {
                    p.pts.push(px(s.pos));
                }
            }
            Tok::PolyEnd => {
                if let Some(p) = open.pop() {
                    if p.pts.len() >= 3 && polygon_seen(view, &p.pts) {
                        if stats.segments + stats.vertices >= budget {
                            stats.stopped = true;
                            return stats;
                        }
                        stats.polygons += 1;
                        stats.vertices += p.pts.len() as u64;
                        sink(Drawn::Polygon(&p));
                    }
                }
            }
        }
    }
    stats
}

/// Whether a polygon's box (view pixels) meets the view.
fn polygon_seen(view: &View, pts: &[[f64; 2]]) -> bool {
    let (hw, hh) = (0.5 * view.size[0] + view.margin, 0.5 * view.size[1] + view.margin);
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for p in pts {
        lo = [lo[0].min(p[0]), lo[1].min(p[1])];
        hi = [hi[0].max(p[0]), hi[1].max(p[1])];
    }
    lo[0] <= hw && hi[0] >= -hw && lo[1] <= hh && hi[1] >= -hh
}

/// The bounding box of what a built word draws, world units `[x0, y0, x1, y1]` (`None`: nothing).
pub fn bounds(sys: &LSystem, x: &Expansion) -> Option<[f64; 4]> {
    let mut b = [f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
    let mut add = |p: [f64; 2]| {
        if p[0].is_finite() && p[1].is_finite() {
            b = [b[0].min(p[0]), b[1].min(p[1]), b[2].max(p[0]), b[3].max(p[1])];
        }
    };
    draw(sys, x, &View::EVERYTHING, u64::MAX, &mut |d| match d {
        Drawn::Segment(s) => {
            add(s.a);
            add(s.b);
        }
        Drawn::Polygon(p) => p.pts.iter().for_each(|&q| add(q)),
    });
    (b[0] <= b[2]).then_some(b)
}

#[cfg(test)]
mod tests;
