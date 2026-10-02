//! Editing the textbook form (design/formula-textbook-editor.md §4.9): the document — the source's
//! lines, each statement a row of [`model`] atoms — and the commands that change it. Pure: the
//! widget (`editor.rs`) turns keys and clicks into these, and the tests drive them directly.
//!
//! A line no statement of which was edited prints back verbatim (its spacing, its parentheses, its
//! comment); an edited one is printed from its rows, its indentation and comment kept.

use super::model::{self, Atom, Caret, Line, Op, Row};
use crate::ui::formula_keypad::Action;

/// What a line of the source is.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Kind {
    /// Spaces only: kept in the text, not shown.
    Blank,
    /// Statements — at least one: a line with only a comment has an empty one, a place to type —
    /// with the text before the first (indentation) and after the last (spaces, the comment).
    Read { stmts: Vec<Row>, lead: String, tail: String, comment: Option<String>, edited: bool },
    /// A line that does not read: shown as its text, edited in Text mode.
    Unread { error: String },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DLine {
    /// The line as read, without its break: what prints until a statement on it is edited.
    pub(crate) text: String,
    /// Its break as the source had it: "\n", "\r\n", or "" (the last line).
    pub(crate) brk: &'static str,
    pub(crate) kind: Kind,
}

fn empty_line(brk: &'static str) -> DLine {
    let kind = Kind::Read { stmts: vec![Vec::new()], lead: String::new(), tail: String::new(), comment: None, edited: true };
    DLine { text: String::new(), brk, kind }
}

fn dline(raw: &str, brk: &'static str) -> DLine {
    let kind = if raw.trim().is_empty() {
        Kind::Blank
    } else {
        match model::read_line(raw, 0) {
            Line::Read { stmts, comment } => {
                let (lead, tail) = match (stmts.first(), stmts.last()) {
                    (Some(f), Some(l)) => (&raw[..f.span.start], &raw[l.span.end..]),
                    _ => raw.split_at(raw.len() - raw.trim_start().len()),
                };
                let mut rows: Vec<Row> = stmts.into_iter().map(|s| s.row).collect();
                if rows.is_empty() {
                    rows.push(Vec::new());
                }
                Kind::Read { stmts: rows, lead: lead.into(), tail: tail.into(), comment, edited: false }
            }
            Line::Unread { error, .. } => Kind::Unread { error },
        }
    };
    DLine { text: raw.to_string(), brk, kind }
}

/// What the widget shows, row by row.
pub(crate) enum Shown<'a> {
    /// A statement, and the comment after it if it is its line's last.
    Stmt { line: usize, stmt: usize, row: &'a Row, comment: Option<&'a str> },
    /// A line that does not read: its text and why.
    Unread { line: usize, text: &'a str, error: &'a str },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Doc {
    pub(crate) lines: Vec<DLine>,
}

impl Doc {
    /// A source as lines. There is always a statement to type into: a source without one gets an
    /// empty one at its end.
    pub(crate) fn read(src: &str) -> Doc {
        let mut lines = Vec::new();
        let mut rest = src;
        loop {
            match rest.find('\n') {
                Some(i) => {
                    let (raw, brk) = match rest[..i].strip_suffix('\r') {
                        Some(r) => (r, "\r\n"),
                        None => (&rest[..i], "\n"),
                    };
                    lines.push(dline(raw, brk));
                    rest = &rest[i + 1..];
                }
                None => {
                    lines.push(dline(rest, ""));
                    break;
                }
            }
        }
        let mut doc = Doc { lines };
        if !doc.lines.iter().any(|l| matches!(l.kind, Kind::Read { .. })) {
            // (The line before keeps its missing break: one is printed only once this has text.)
            if !doc.lines.last().is_some_and(|l| l.text.is_empty()) {
                doc.lines.push(empty_line(""));
            }
            if let Some(l) = doc.lines.last_mut() {
                // Nothing typed into it yet, it prints as the empty line it is.
                l.kind = Kind::Read { stmts: vec![Vec::new()], lead: String::new(), tail: String::new(), comment: None, edited: false };
            }
        }
        doc
    }

    /// The text: lines nobody edited as they were, edited ones printed from their rows.
    pub(crate) fn source(&self) -> String {
        self.printed().0
    }

    /// The text, and where each line starts in it.
    fn printed(&self) -> (String, Vec<usize>) {
        let mut s = String::new();
        let mut starts = Vec::with_capacity(self.lines.len());
        for (i, l) in self.lines.iter().enumerate() {
            let text = self.line_text(l);
            // A line after the source's last (one to type into, added by `read`) gets its break once
            // it has text.
            if i > 0 && self.lines[i - 1].brk.is_empty() && !text.is_empty() {
                s.push_str(self.break_style());
            }
            starts.push(s.len());
            s.push_str(&text);
            s.push_str(l.brk);
        }
        (s, starts)
    }

    fn line_text(&self, l: &DLine) -> String {
        match &l.kind {
            Kind::Read { stmts, lead, tail, edited: true, .. } => {
                let printed: Vec<String> = stmts.iter().filter(|r| !r.is_empty()).map(|r| model::print_row(r)).collect();
                // A statement typed in front of a comment that had none: a space between.
                let gap = if !printed.is_empty() && tail.starts_with(';') { " " } else { "" };
                format!("{lead}{}{gap}{tail}", printed.join(", "))
            }
            _ => l.text.clone(),
        }
    }

    /// Where line `line` starts in [`Doc::source`], in bytes.
    pub(crate) fn line_start(&self, line: usize) -> usize {
        let (s, starts) = self.printed();
        starts.get(line).copied().unwrap_or(s.len())
    }

    /// The line break new lines get: the source's own.
    fn break_style(&self) -> &'static str {
        if self.lines.iter().any(|l| l.brk == "\r\n") {
            "\r\n"
        } else {
            "\n"
        }
    }

    pub(crate) fn shown(&self) -> Vec<Shown<'_>> {
        let mut out = Vec::new();
        for (line, l) in self.lines.iter().enumerate() {
            match &l.kind {
                Kind::Blank => {}
                Kind::Read { stmts, comment, .. } => {
                    for (stmt, row) in stmts.iter().enumerate() {
                        let last = stmt + 1 == stmts.len();
                        out.push(Shown::Stmt { line, stmt, row, comment: comment.as_deref().filter(|_| last) });
                    }
                }
                Kind::Unread { error } => out.push(Shown::Unread { line, text: l.text.trim_end(), error }),
            }
        }
        out
    }

    /// Every statement, in order: (line, statement on it).
    pub(crate) fn statements(&self) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for (line, l) in self.lines.iter().enumerate() {
            if let Kind::Read { stmts, .. } = &l.kind {
                out.extend((0..stmts.len()).map(|s| (line, s)));
            }
        }
        out
    }

    fn stmt(&self, line: usize, stmt: usize) -> Option<&Row> {
        match &self.lines.get(line)?.kind {
            Kind::Read { stmts, .. } => stmts.get(stmt),
            _ => None,
        }
    }

    fn stmts_mut(&mut self, line: usize) -> Option<&mut Vec<Row>> {
        match &mut self.lines.get_mut(line)?.kind {
            Kind::Read { stmts, .. } => Some(stmts),
            _ => None,
        }
    }

    /// The row a caret is in.
    pub(crate) fn row(&self, c: &Caret) -> Option<&Row> {
        let mut row = self.stmt(c.line, c.stmt)?;
        for &(i, k) in &c.path {
            row = row.get(i)?.child(k)?;
        }
        Some(row)
    }

    fn row_mut(&mut self, c: &Caret) -> Option<&mut Row> {
        let mut row = self.stmts_mut(c.line)?.get_mut(c.stmt)?;
        for &(i, k) in &c.path {
            row = row.get_mut(i)?.child_mut(k)?;
        }
        Some(row)
    }

    /// Every place a caret can stand, in reading order: a row's places, each structure's rows
    /// between the places before and after it.
    pub(crate) fn places(&self) -> Vec<Caret> {
        fn walk(row: &[Atom], c: &mut Caret, out: &mut Vec<Caret>) {
            for pos in 0..=row.len() {
                out.push(Caret { pos, ..c.clone() });
                if let Some(a) = row.get(pos) {
                    for k in 0..a.arity() {
                        c.path.push((pos, k));
                        walk(a.child(k).map_or(&[][..], |r| r), c, out);
                        c.path.pop();
                    }
                }
            }
        }
        let mut out = Vec::new();
        for (line, stmt) in self.statements() {
            let mut c = Caret { line, stmt, ..Default::default() };
            walk(self.stmt(line, stmt).map_or(&[][..], |r| r), &mut c, &mut out);
        }
        out
    }
}

/// The row above a caret's: that row's place before the structure, and which of the structure's
/// rows the caret is in.
fn parent(c: &Caret) -> Option<(Caret, u8)> {
    let (&(i, k), up) = c.path.split_last()?;
    Some((Caret { path: up.to_vec(), pos: i, ..c.clone() }, k))
}

/// The selection: a range of atoms of one row.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Sel {
    /// The row (its statement and path), at the range's start.
    pub(crate) at: Caret,
    pub(crate) end: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Dir {
    Left,
    Right,
}

/// Where ↑ or ↓ goes.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Vertical {
    /// To this place.
    To(Caret),
    /// Into this row (the caret's `pos` aside), at the place nearest the caret's x.
    Nearest(Caret),
    Stay,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Editor {
    pub(crate) doc: Doc,
    pub(crate) caret: Caret,
    /// The selection's other end, while there is one.
    pub(crate) anchor: Option<Caret>,
    /// Backspace or Delete at the edge of a structure's row selected the structure (from its row
    /// `k`): the next press removes it and keeps what it held.
    unwrap: Option<u8>,
    undo: Vec<(Doc, Caret)>,
    redo: Vec<(Doc, Caret)>,
    /// The row the last edit typed a character into: typing a name is one undo step.
    typing: Option<Caret>,
    /// The source the document was read from or last printed to.
    pub(crate) synced: String,
    /// The completion list's highlighted entry, and the name Esc closed it for (where the name
    /// starts, and its text): it stays closed until that changes.
    pub(crate) list_sel: usize,
    pub(crate) list_dismissed: Option<(Caret, String)>,
}

/// Characters that end an exponent when typed at its end (MathQuill's `charsThatBreakOutOfSupSub`
/// as Desmos sets it): `z^2+c` is 𝑧² + 𝑐, as the text is; `z^-1` keeps its sign (the exponent was
/// empty).
const BREAK_OUT_OF_SUP: [char; 3] = ['+', '-', '='];

const UNDO_DEPTH: usize = 200;

impl Editor {
    pub(crate) fn new(src: &str) -> Editor {
        let mut e = Editor::default();
        e.sync(src);
        e
    }

    /// Follow `src` if it changed outside (Text mode, an example, the library): read it again, the
    /// caret at the end of the statement it was in (or the first). Returns whether it was read.
    pub(crate) fn sync(&mut self, src: &str) -> bool {
        if src == self.synced && !self.doc.lines.is_empty() {
            return false;
        }
        self.doc = Doc::read(src);
        self.synced = src.to_string();
        self.anchor = None;
        self.unwrap = None;
        self.typing = None;
        self.undo.clear();
        self.redo.clear();
        let all = self.doc.statements();
        let (line, stmt) = all
            .iter()
            .copied()
            .find(|&(l, s)| (l, s) == (self.caret.line, self.caret.stmt))
            .or_else(|| all.iter().copied().find(|&(l, _)| l >= self.caret.line))
            .or(all.first().copied())
            .unwrap_or((0, 0));
        let pos = self.doc.stmt(line, stmt).map_or(0, |r| r.len());
        self.caret = Caret { line, stmt, path: Vec::new(), pos };
        true
    }

    /// The caret at the end of the first statement on the line holding source byte `at`.
    pub(crate) fn place_at(&mut self, at: usize) {
        let line = (0..self.doc.lines.len()).rev().find(|&l| self.doc.line_start(l) <= at).unwrap_or(0);
        if let Some(&(l, s)) = self.doc.statements().iter().find(|&&(l, _)| l >= line) {
            let pos = self.doc.stmt(l, s).map_or(0, |r| r.len());
            self.caret = Caret { line: l, stmt: s, path: Vec::new(), pos };
            self.anchor = None;
        }
    }

    /// A structure has an empty row: a box still to fill (the text cannot read until it is).
    pub(crate) fn has_empty_box(&self) -> bool {
        self.doc.places().iter().any(|p| !p.path.is_empty() && self.row(p).is_empty())
    }

    /// Where the caret's line starts in the source, for Text mode.
    pub(crate) fn caret_offset(&self) -> usize {
        self.doc.line_start(self.caret.line)
    }

    /// Where the caret's line ends in the source (before its break).
    pub(crate) fn caret_line_end(&self) -> usize {
        self.caret_offset() + self.doc.lines.get(self.caret.line).map_or(0, |l| self.doc.line_text(l).len())
    }

    fn row(&self, c: &Caret) -> &[Atom] {
        self.doc.row(c).map_or(&[], |r| r)
    }

    /// The selection, from its anchor to the caret, as a range of one row: the row both ends share
    /// (an end inside a structure takes the whole structure).
    pub(crate) fn selection(&self) -> Option<Sel> {
        let a = self.anchor.as_ref()?;
        let c = &self.caret;
        if (a.line, a.stmt) != (c.line, c.stmt) {
            return None;
        }
        let d = a.path.iter().zip(&c.path).take_while(|(x, y)| x == y).count();
        let index = |e: &Caret| e.path.get(d).map_or((e.pos, e.pos), |&(i, _)| (i, i + 1));
        let ((a0, a1), (c0, c1)) = (index(a), index(c));
        let (start, end) = (a0.min(c0), a1.max(c1));
        (start < end).then(|| Sel { at: Caret { path: c.path[..d].to_vec(), pos: start, ..c.clone() }, end })
    }

    /// The selection as text, for the clipboard.
    pub(crate) fn copy(&self) -> Option<String> {
        let s = self.selection()?;
        self.doc.row(&s.at).map(|r| model::print_row(&r[s.at.pos..s.end]))
    }

    pub(crate) fn cut(&mut self) -> Option<String> {
        let text = self.copy()?;
        self.begin(false);
        self.delete_selection();
        self.done();
        Some(text)
    }

    // ---- Bookkeeping ----

    /// Before an edit: an undo snapshot (one for a run of typed characters in one row).
    fn begin(&mut self, typing: bool) {
        let row = Caret { pos: 0, ..self.caret.clone() };
        if !(typing && self.typing.as_ref() == Some(&row)) {
            self.undo.push((self.doc.clone(), self.caret.clone()));
            if self.undo.len() > UNDO_DEPTH {
                self.undo.remove(0);
            }
        }
        self.typing = typing.then_some(row);
        self.redo.clear();
    }

    /// After an edit: the caret's statement back in the grammar's form, its line marked edited,
    /// the text printed.
    fn done(&mut self) {
        let Editor { doc, caret, anchor, .. } = self;
        let (line, stmt) = (caret.line, caret.stmt);
        if let Some(DLine { kind: Kind::Read { stmts, edited, .. }, .. }) = doc.lines.get_mut(line) {
            *edited = true;
            if let Some(row) = stmts.get_mut(stmt) {
                let mut cs: Vec<&mut Caret> = vec![caret];
                cs.extend(anchor.as_mut().filter(|a| (a.line, a.stmt) == (line, stmt)));
                model::normalize(row, &mut Vec::new(), &mut cs);
            }
        }
        self.synced = self.doc.source();
    }

    /// The selection's atoms, taken out (the caret where they were).
    fn delete_selection(&mut self) -> Option<Row> {
        let s = self.selection();
        self.anchor = None;
        let s = s?;
        let row = self.doc.row_mut(&s.at)?;
        let taken: Row = row.drain(s.at.pos..s.end.min(row.len())).collect();
        self.caret = s.at;
        Some(taken)
    }

    /// Put `atoms` at the caret (in place of the selection), the caret after them.
    fn insert(&mut self, atoms: Row) {
        self.delete_selection();
        let c = self.caret.clone();
        if let Some(row) = self.doc.row_mut(&c) {
            let p = c.pos.min(row.len());
            let n = atoms.len();
            row.splice(p..p, atoms);
            self.caret.pos = p + n;
        }
    }

    /// Put a structure at the caret holding the selection in its row `k`. Nothing selected, the
    /// caret goes into that row; something, after the structure.
    fn structure(&mut self, make: impl FnOnce(Row) -> Atom, k: u8) {
        let held = self.delete_selection().unwrap_or_default();
        let had = !held.is_empty();
        let c = self.caret.clone();
        if let Some(row) = self.doc.row_mut(&c) {
            let p = c.pos.min(row.len());
            row.insert(p, make(held));
            if had {
                self.caret.pos = p + 1;
            } else {
                self.caret.path.push((p, k));
                self.caret.pos = 0;
            }
        }
    }

    // ---- Typing ----

    /// A typed character. Returns whether the text changed.
    pub(crate) fn type_char(&mut self, ch: char) -> bool {
        self.unwrap = None;
        if BREAK_OUT_OF_SUP.contains(&ch) {
            self.break_out_of_sup();
        }
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '_' => {
                self.begin(true);
                self.insert(vec![Atom::Char(ch)]);
            }
            '+' | '-' | '=' => {
                self.begin(false);
                // A sign is typed as a character: one after a number's `e` is its exponent's sign
                // (`1e-5`); normalising makes every other one an operator.
                self.insert(vec![if ch == '=' { Atom::Op(Op::Eq) } else { Atom::Char(ch) }]);
            }
            '*' => {
                self.begin(false);
                self.insert(vec![Atom::Op(Op::Times)]);
            }
            '/' => {
                self.begin(false);
                self.fraction();
            }
            '^' => return self.power(),
            '(' => {
                self.begin(false);
                self.structure(Atom::Group, 0);
            }
            ')' => return self.close(),
            '|' => return self.bars(),
            ',' => return self.comma(),
            _ => return false,
        }
        self.done();
        true
    }

    /// At the end of an exponent that has something in it, step out after it.
    fn break_out_of_sup(&mut self) {
        if self.selection().is_some() || self.caret.pos != self.row(&self.caret).len() || self.caret.pos == 0 {
            return;
        }
        if let Some((up, _)) = parent(&self.caret) {
            if matches!(self.row(&up).get(up.pos), Some(Atom::Sup(_))) {
                self.caret = Caret { pos: up.pos + 1, ..up };
            }
        }
    }

    /// `/`: a fraction over the selection, or over the term before the caret — what the parser
    /// takes as the left operand of `/`: back to a binary `+`/`−` or `=`, unary signs included.
    fn fraction(&mut self) {
        let num = match self.delete_selection() {
            Some(sel) => sel,
            None => {
                let c = self.caret.clone();
                let Some(row) = self.doc.row_mut(&c) else { return };
                let p = c.pos.min(row.len());
                let mut j = p;
                while j > 0 {
                    let binary = |k: usize| k > 0 && !matches!(row[k - 1], Atom::Op(_));
                    match row[j - 1] {
                        Atom::Op(Op::Eq) => break,
                        Atom::Op(Op::Plus | Op::Minus) if binary(j - 1) => break,
                        _ => j -= 1,
                    }
                }
                self.caret.pos = j;
                row.drain(j..p).collect()
            }
        };
        let k = if num.is_empty() { 0 } else { 1 };
        let c = self.caret.clone();
        if let Some(row) = self.doc.row_mut(&c) {
            let p = c.pos.min(row.len());
            row.insert(p, Atom::Frac { num, den: Vec::new() });
            self.caret.path.push((p, k));
            self.caret.pos = 0;
        }
    }

    /// `^`: an exponent on what is before the caret (nothing there: nothing happens), or on the
    /// selection — in parentheses unless it is one primary.
    fn power(&mut self) -> bool {
        if self.selection().is_some() {
            self.begin(false);
            let sel = self.delete_selection().unwrap_or_default();
            let base = if model::single_primary(&sel) { sel } else { vec![Atom::Group(sel)] };
            self.insert(base);
        } else {
            let row = self.row(&self.caret);
            let p = self.caret.pos.min(row.len());
            let base = p > 0 && !matches!(row[p - 1], Atom::Op(_) | Atom::Char('+' | '-'));
            if !base {
                return false;
            }
            self.begin(false);
        }
        self.structure(Atom::Sup, 0);
        self.done();
        true
    }

    /// `)`: step out of the parentheses (or call, or complex constant) the caret is in. In none,
    /// what is before the caret becomes a group — from the row's start, or after its `=`.
    fn close(&mut self) -> bool {
        let mut c = self.caret.clone();
        while let Some((up, _)) = parent(&c) {
            if matches!(self.row(&up).get(up.pos), Some(Atom::Group(_) | Atom::Func { .. } | Atom::Complex { .. })) {
                self.caret = Caret { pos: up.pos + 1, ..up };
                self.anchor = None;
                return false;
            }
            c = up;
        }
        let c = self.caret.clone();
        let row = self.row(&c);
        let p = c.pos.min(row.len());
        let from = if c.path.is_empty() { row[..p].iter().rposition(|a| *a == Atom::Op(Op::Eq)).map_or(0, |k| k + 1) } else { 0 };
        if from == p {
            return false;
        }
        self.begin(false);
        self.anchor = None;
        if let Some(row) = self.doc.row_mut(&c) {
            let inner: Row = row.drain(from..p).collect();
            row.insert(from, Atom::Group(inner));
            self.caret.pos = from + 1;
        }
        self.done();
        true
    }

    /// `|`: at the end of bars, step out of them; else bars round the selection (or empty ones).
    fn bars(&mut self) -> bool {
        if self.selection().is_none() && self.caret.pos == self.row(&self.caret).len() {
            if let Some((up, _)) = parent(&self.caret) {
                if matches!(self.row(&up).get(up.pos), Some(Atom::Bars(_))) {
                    self.caret = Caret { pos: up.pos + 1, ..up };
                    return false;
                }
            }
        }
        self.begin(false);
        self.structure(Atom::Bars, 0);
        self.done();
        true
    }

    /// `,`: in parentheses, a complex constant (what is before the caret its real part); in its real
    /// part, on to the imaginary; in a statement's own row, a new statement after it on the line.
    fn comma(&mut self) -> bool {
        let c = self.caret.clone();
        let Some((up, k)) = parent(&c) else {
            self.begin(false);
            self.delete_selection();
            let c = self.caret.clone();
            if let Some(stmts) = self.doc.stmts_mut(c.line) {
                let at = c.pos.min(stmts[c.stmt].len());
                let rest = stmts[c.stmt].split_off(at);
                stmts.insert(c.stmt + 1, rest);
                self.caret = Caret { line: c.line, stmt: c.stmt + 1, path: Vec::new(), pos: 0 };
            }
            self.done();
            return true;
        };
        match self.row(&up).get(up.pos) {
            Some(Atom::Group(_)) => {
                self.begin(false);
                self.delete_selection();
                let pos = self.caret.pos;
                if let Some(row) = self.doc.row_mut(&up) {
                    if let Atom::Group(mut re) = row.remove(up.pos) {
                        let im = re.split_off(pos.min(re.len()));
                        row.insert(up.pos, Atom::Complex { re, im });
                    }
                }
                self.caret = Caret { path: [&up.path[..], &[(up.pos, 1)]].concat(), pos: 0, ..up };
                self.done();
                true
            }
            Some(Atom::Complex { .. }) if k == 0 => {
                self.anchor = None;
                self.caret = Caret { path: [&up.path[..], &[(up.pos, 1)]].concat(), pos: 0, ..up };
                false
            }
            _ => false,
        }
    }

    /// Enter: a new line after the caret's, holding what was after the caret in its statement (the
    /// statement's top-level row is split: a caret inside a structure splits after it) and the
    /// statements after it on the line. The comment stays where it was.
    pub(crate) fn enter(&mut self) -> bool {
        self.unwrap = None;
        self.begin(false);
        self.delete_selection();
        let c = self.caret.clone();
        let p = c.path.first().map_or(c.pos, |&(i, _)| i + 1);
        let style = self.doc.break_style();
        let Some(DLine { kind: Kind::Read { stmts, edited, .. }, brk, .. }) = self.doc.lines.get_mut(c.line) else {
            return false;
        };
        let at = p.min(stmts[c.stmt].len());
        let rest = stmts[c.stmt].split_off(at);
        let mut moved = stmts.split_off(c.stmt + 1);
        moved.insert(0, rest);
        *edited = true;
        let new_brk = *brk;
        *brk = style;
        let mut line = empty_line(new_brk);
        if let Kind::Read { stmts, .. } = &mut line.kind {
            *stmts = moved;
        }
        self.doc.lines.insert(c.line + 1, line);
        self.caret = Caret { line: c.line + 1, stmt: 0, path: Vec::new(), pos: 0 };
        self.done();
        true
    }

    /// Backspace: the selection; the character or operator before the caret; into a structure
    /// before it (to its last row's end); at a structure's row's start, select the structure — the
    /// next press removes it and keeps its contents (an empty one goes at once); at a statement's
    /// start, join it to the one before.
    pub(crate) fn backspace(&mut self) -> bool {
        self.erase(Dir::Left)
    }

    /// Delete: Backspace's mirror.
    pub(crate) fn delete(&mut self) -> bool {
        self.erase(Dir::Right)
    }

    fn erase(&mut self, dir: Dir) -> bool {
        let unwrap = self.unwrap.take();
        if let Some(s) = self.selection() {
            self.begin(false);
            match unwrap {
                Some(k) if s.end == s.at.pos + 1 => self.unwrap_at(s, k, dir),
                _ => {
                    self.delete_selection();
                }
            }
            self.done();
            return true;
        }
        let c = self.caret.clone();
        let row = self.row(&c);
        let i = match dir {
            Dir::Left => c.pos.checked_sub(1),
            Dir::Right => (c.pos < row.len()).then_some(c.pos),
        };
        if let Some(i) = i {
            let a = &row[i];
            if a.arity() > 0 {
                // Into it: a structure is taken apart from inside.
                let k = if dir == Dir::Left { a.arity() - 1 } else { 0 };
                let len = a.child(k).map_or(0, |r| r.len());
                self.caret.path.push((i, k));
                self.caret.pos = if dir == Dir::Left { len } else { 0 };
                self.anchor = None;
                return false;
            }
            self.begin(false);
            if let Some(row) = self.doc.row_mut(&c) {
                row.remove(i);
            }
            self.caret.pos = i;
            self.done();
            return true;
        }
        if let Some((up, k)) = parent(&c) {
            let a = &self.row(&up)[up.pos];
            if (0..a.arity()).all(|k| a.child(k).is_none_or(|r| r.is_empty())) {
                self.begin(false);
                if let Some(row) = self.doc.row_mut(&up) {
                    row.remove(up.pos);
                }
                self.caret = up;
                self.done();
                return true;
            }
            let after = Caret { pos: up.pos + 1, ..up.clone() };
            (self.anchor, self.caret) = if dir == Dir::Left { (Some(up), after) } else { (Some(after), up) };
            self.unwrap = Some(k);
            return false;
        }
        match dir {
            Dir::Left => self.join_previous(),
            Dir::Right => self.join_next(),
        }
    }

    /// Remove the structure `s` selects and keep what it held, the caret where it stood in row
    /// `from`: its start (Backspace) or end (Delete). A call keeps its argument's parentheses.
    fn unwrap_at(&mut self, s: Sel, from: u8, dir: Dir) {
        let Some(row) = self.doc.row_mut(&s.at) else { return };
        let i = s.at.pos;
        let a = row.remove(i);
        let parts: Vec<Row> = (0..a.arity()).map(|k| a.child(k).cloned().unwrap_or_default()).collect();
        let (contents, pos) = match a {
            Atom::Func { arg, .. } => (vec![Atom::Group(arg)], if dir == Dir::Left { i } else { i + 1 }),
            _ => {
                let before: usize = parts[..usize::from(from)].iter().map(Vec::len).sum();
                let own = parts.get(usize::from(from)).map_or(0, Vec::len);
                (parts.concat(), i + before + if dir == Dir::Left { 0 } else { own })
            }
        };
        row.splice(i..i, contents);
        self.anchor = None;
        self.caret = Caret { pos, ..s.at };
    }

    /// At a statement's start: join it to the end of the one before (on this line, or the last on
    /// the line above). Not past a comment, which would end up mid-line, or a line that does not
    /// read; a blank line above just goes.
    fn join_previous(&mut self) -> bool {
        let c = self.caret.clone();
        if c.stmt > 0 {
            self.begin(false);
            if let Some(stmts) = self.doc.stmts_mut(c.line) {
                let cur = stmts.remove(c.stmt);
                let at = stmts[c.stmt - 1].len();
                stmts[c.stmt - 1].extend(cur);
                self.caret = Caret { line: c.line, stmt: c.stmt - 1, path: Vec::new(), pos: at };
            }
            self.done();
            return true;
        }
        let Some(pl) = c.line.checked_sub(1) else { return false };
        self.join_lines(pl, true)
    }

    /// At a statement's end: join the next to it.
    fn join_next(&mut self) -> bool {
        let c = self.caret.clone();
        let n = self.doc.stmts_mut(c.line).map_or(0, |s| s.len());
        if c.stmt + 1 < n {
            self.begin(false);
            if let Some(stmts) = self.doc.stmts_mut(c.line) {
                let next = stmts.remove(c.stmt + 1);
                stmts[c.stmt].extend(next);
            }
            self.done();
            return true;
        }
        if c.line + 1 >= self.doc.lines.len() {
            return false;
        }
        self.join_lines(c.line, false)
    }

    /// Join line `top + 1` onto line `top`; the caret at the junction. `from_below`: the caret was
    /// on the lower line.
    fn join_lines(&mut self, top: usize, from_below: bool) -> bool {
        let lower = top + 1;
        let (upper_kind, lower_kind) = (&self.doc.lines[top].kind, &self.doc.lines[lower].kind);
        // A blank line between goes on its own.
        let blank = if matches!(upper_kind, Kind::Blank) {
            Some(top)
        } else if matches!(lower_kind, Kind::Blank) {
            Some(lower)
        } else {
            None
        };
        if let Some(b) = blank {
            self.begin(false);
            let gone = self.doc.lines.remove(b);
            if b == lower {
                self.doc.lines[top].brk = gone.brk;
            } else if from_below {
                self.caret.line -= 1;
            }
            self.synced = self.doc.source();
            return true;
        }
        let comment_between = match upper_kind {
            Kind::Read { tail, .. } => tail.contains(';'),
            _ => true,
        };
        if comment_between || !matches!(lower_kind, Kind::Read { .. }) {
            return false;
        }
        self.begin(false);
        let low = self.doc.lines.remove(lower);
        let Kind::Read { stmts: mut moved, tail: low_tail, comment: low_comment, .. } = low.kind else { return false };
        let up = &mut self.doc.lines[top];
        up.brk = low.brk;
        if let Kind::Read { stmts, tail, comment, edited, .. } = &mut up.kind {
            let last = stmts.len() - 1;
            let at = stmts[last].len();
            let first = moved.remove(0);
            stmts[last].extend(first);
            stmts.extend(moved);
            *tail = low_tail;
            *comment = low_comment;
            *edited = true;
            self.caret = Caret { line: top, stmt: last, path: Vec::new(), pos: at };
        }
        self.done();
        true
    }

    /// Paste: text that reads goes in as its rows (several statements as several statements);
    /// text that does not is typed, character by character.
    pub(crate) fn paste(&mut self, text: &str) -> bool {
        let text = text.replace("\r\n", "\n");
        let mut changed = false;
        for (n, mut line) in text.split('\n').enumerate() {
            if n > 0 {
                changed |= self.enter();
            }
            // Read alone, `- 1/z` is a negative fraction; after an operand it is a subtraction.
            let lead = line.trim_start();
            if self.after_operand() && (lead.starts_with('+') || lead.starts_with('-')) {
                changed |= self.type_char(lead.chars().next().unwrap_or('+'));
                line = &lead[1..];
            }
            match model::read_line(line, 0) {
                Line::Read { stmts, .. } => {
                    for (j, st) in stmts.into_iter().enumerate() {
                        if j > 0 {
                            self.type_char(',');
                        }
                        self.unwrap = None;
                        self.begin(false);
                        self.insert(st.row);
                        self.done();
                        changed = true;
                    }
                }
                Line::Unread { .. } => {
                    for ch in line.chars() {
                        changed |= self.type_char(ch);
                    }
                }
            }
        }
        changed
    }

    /// A keypad key, as the commands it stands for (design §4.8). `None`: one the Textbook editor
    /// has no command for (a comment), for Text mode.
    pub(crate) fn press(&mut self, action: Action) -> Option<bool> {
        Some(match action {
            Action::Insert("\n") => self.enter(),
            Action::Insert(" ; ") => return None,
            // □²: the exponent typed and stepped out of, as the text's cursor ends after `^2`.
            Action::Insert("^2") => {
                if !self.type_char('^') {
                    return Some(false);
                }
                self.type_char('2');
                self.step(Dir::Right, false);
                true
            }
            // (a, b): in place of the selection, as the text's key, the constant's two places to
            // fill, the caret in the first.
            Action::Insert("(0.5, 0.5)") => {
                self.unwrap = None;
                self.begin(false);
                self.delete_selection();
                self.structure(|_| Atom::Complex { re: Vec::new(), im: Vec::new() }, 0);
                self.done();
                true
            }
            Action::Insert(s) => s.chars().fold(false, |changed, ch| self.type_char(ch) | changed),
            Action::Wrap("|", "|") => self.type_char('|'),
            Action::Wrap(open, ")") if open.ends_with('(') => {
                let name = open.trim_end_matches('(').to_string();
                self.unwrap = None;
                self.begin(false);
                self.structure(|arg| Atom::Func { name, arg }, 0);
                self.done();
                true
            }
            Action::Wrap(..) => return None,
            Action::Backspace => self.backspace(),
            Action::Left => {
                self.step(Dir::Left, false);
                false
            }
            Action::Right => {
                self.step(Dir::Right, false);
                false
            }
        })
    }

    /// The name being typed at the caret — where it starts in the caret's row and what it is so
    /// far — when the caret is at its end (a letter after the caret would make it a longer name).
    pub(crate) fn name_at_caret(&self) -> Option<(usize, String)> {
        if self.selection().is_some() {
            return None;
        }
        let row = self.row(&self.caret);
        let p = self.caret.pos;
        if matches!(row.get(p), Some(Atom::Char(c)) if c.is_ascii_alphanumeric() || *c == '_') {
            return None;
        }
        let (t, text) = model::token_ending_at(row, p)?;
        (t.kind == model::Kind::Name).then_some((t.start, text))
    }

    /// Replace the name being typed (atoms `start` to the caret) with `name`. A function gets its
    /// parentheses and the caret in them — or, already followed by some, becomes their call.
    pub(crate) fn complete(&mut self, start: usize, name: &str, call: bool) -> bool {
        self.unwrap = None;
        self.begin(false);
        let c = self.caret.clone();
        let Some(row) = self.doc.row_mut(&c) else { return false };
        let end = c.pos.min(row.len());
        let start = start.min(end);
        let followed = matches!(row.get(end), Some(Atom::Group(_)));
        row.splice(start..end, model::chars(name));
        let after = start + name.chars().count();
        self.caret.pos = after;
        if call && !followed {
            row.insert(after, Atom::Group(Vec::new()));
            self.caret.path.push((after, 0));
            self.caret.pos = 0;
        }
        // Normalising makes the name and the parentheses one call, the caret in its argument.
        self.done();
        true
    }

    /// What is typed here follows an operand: the caret (or the selection, which typing replaces)
    /// comes after a name, number or structure.
    fn after_operand(&self) -> bool {
        let at = self.selection().map_or_else(|| self.caret.clone(), |s| s.at);
        let row = self.row(&at);
        at.pos > 0 && !matches!(row.get(at.pos - 1), Some(Atom::Op(_) | Atom::Char('+' | '-')) | None)
    }

    pub(crate) fn undo(&mut self) -> bool {
        self.history(true)
    }

    pub(crate) fn redo(&mut self) -> bool {
        self.history(false)
    }

    fn history(&mut self, back: bool) -> bool {
        let (from, to) = if back { (&mut self.undo, &mut self.redo) } else { (&mut self.redo, &mut self.undo) };
        let Some((doc, caret)) = from.pop() else { return false };
        to.push((std::mem::replace(&mut self.doc, doc), std::mem::replace(&mut self.caret, caret)));
        self.anchor = None;
        self.unwrap = None;
        self.typing = None;
        self.synced = self.doc.source();
        true
    }

    // ---- Moving ----

    /// Put the caret at `c` (a click), extending the selection if `select`.
    pub(crate) fn set_caret(&mut self, c: Caret, select: bool) {
        self.unwrap = None;
        self.typing = None;
        if select {
            if self.anchor.is_none() {
                self.anchor = Some(self.caret.clone());
            }
        } else {
            self.anchor = None;
        }
        self.caret = c;
    }

    /// ← or →: through characters, into and out of structures (MathQuill's order), on to the next
    /// statement. Selecting, structures are stepped over whole and the statement is the limit.
    pub(crate) fn step(&mut self, dir: Dir, select: bool) {
        if !select {
            if let Some(s) = self.selection() {
                self.anchor = None;
                self.caret = if dir == Dir::Right { Caret { pos: s.end, ..s.at } } else { s.at };
                return;
            }
        }
        let next = if select { self.select_step(dir) } else { self.plain_step(dir) };
        if let Some(n) = next {
            self.set_caret(n, select);
        }
    }

    fn plain_step(&self, dir: Dir) -> Option<Caret> {
        let c = &self.caret;
        let row = self.row(c);
        match dir {
            Dir::Right if c.pos < row.len() => Some(match &row[c.pos] {
                a if a.arity() > 0 => Caret { path: [&c.path[..], &[(c.pos, 0)]].concat(), pos: 0, ..c.clone() },
                _ => Caret { pos: c.pos + 1, ..c.clone() },
            }),
            Dir::Left if c.pos > 0 => Some(match &row[c.pos - 1] {
                a if a.arity() > 0 => {
                    let k = a.arity() - 1;
                    let len = a.child(k).map_or(0, |r| r.len());
                    Caret { path: [&c.path[..], &[(c.pos - 1, k)]].concat(), pos: len, ..c.clone() }
                }
                _ => Caret { pos: c.pos - 1, ..c.clone() },
            }),
            _ => match parent(c) {
                Some((up, k)) => {
                    let arity = self.row(&up)[up.pos].arity();
                    let mut path = up.path.clone();
                    Some(match dir {
                        Dir::Right if k + 1 < arity => {
                            path.push((up.pos, k + 1));
                            Caret { path, pos: 0, ..up }
                        }
                        Dir::Left if k > 0 => {
                            path.push((up.pos, k - 1));
                            let len = self.row(&Caret { path: path.clone(), ..up.clone() }).len();
                            Caret { path, pos: len, ..up }
                        }
                        Dir::Right => Caret { pos: up.pos + 1, ..up },
                        Dir::Left => up,
                    })
                }
                None => {
                    let all = self.doc.statements();
                    let i = all.iter().position(|&s| s == (c.line, c.stmt))?;
                    let (line, stmt) = *match dir {
                        Dir::Right => all.get(i + 1)?,
                        Dir::Left => all.get(i.checked_sub(1)?)?,
                    };
                    let pos = if dir == Dir::Right { 0 } else { self.doc.stmt(line, stmt).map_or(0, |r| r.len()) };
                    Some(Caret { line, stmt, path: Vec::new(), pos })
                }
            },
        }
    }

    fn select_step(&self, dir: Dir) -> Option<Caret> {
        let c = &self.caret;
        let len = self.row(c).len();
        match dir {
            Dir::Right if c.pos < len => Some(Caret { pos: c.pos + 1, ..c.clone() }),
            Dir::Left if c.pos > 0 => Some(Caret { pos: c.pos - 1, ..c.clone() }),
            _ => parent(c).map(|(up, _)| match dir {
                Dir::Right => Caret { pos: up.pos + 1, ..up },
                Dir::Left => up,
            }),
        }
    }

    /// Home or End: the start or end of the statement.
    pub(crate) fn home_end(&mut self, end: bool, select: bool) {
        let c = &self.caret;
        let pos = if end { self.doc.stmt(c.line, c.stmt).map_or(0, |r| r.len()) } else { 0 };
        let to = Caret { line: c.line, stmt: c.stmt, path: Vec::new(), pos };
        self.set_caret(to, select);
    }

    /// The name or number at the caret selected (a double click); beside none, the atom after it.
    pub(crate) fn select_word(&mut self) {
        let c = self.caret.clone();
        let row = self.row(&c);
        let is_char = |i: usize| matches!(row.get(i), Some(Atom::Char(_)));
        let (start, end) = if is_char(c.pos) || (c.pos > 0 && is_char(c.pos - 1)) {
            let i = if is_char(c.pos) { c.pos } else { c.pos - 1 };
            let (toks, _) = model::run_tokens(row, model::run_start(row, i));
            match toks.iter().find(|t| t.start <= i && i < t.end) {
                Some(t) => (t.start, t.end),
                None => return,
            }
        } else if c.pos < row.len() {
            (c.pos, c.pos + 1)
        } else {
            return;
        };
        self.set_caret(Caret { pos: start, ..c.clone() }, false);
        self.set_caret(Caret { pos: end, ..c }, true);
    }

    /// The whole statement selected.
    pub(crate) fn select_all(&mut self) {
        let c = &self.caret;
        let len = self.doc.stmt(c.line, c.stmt).map_or(0, |r| r.len());
        self.set_caret(Caret { line: c.line, stmt: c.stmt, path: Vec::new(), pos: 0 }, false);
        self.set_caret(Caret { pos: len, ..self.caret.clone() }, true);
    }

    /// Tab: the next empty place to fill (Shift+Tab: the one before). Returns whether there was one.
    pub(crate) fn tab(&mut self, back: bool) -> bool {
        let places = self.doc.places();
        let here = places.iter().position(|p| *p == self.caret);
        let empty = |p: &Caret| !p.path.is_empty() && self.row(p).is_empty() && *p != self.caret;
        let found = match (back, here) {
            (false, Some(h)) => places[h + 1..].iter().find(|p| empty(p)),
            (true, Some(h)) => places[..h].iter().rev().find(|p| empty(p)),
            (_, None) => places.iter().find(|p| empty(p)),
        };
        match found.cloned() {
            Some(p) => {
                self.set_caret(p, false);
                true
            }
            None => false,
        }
    }

    /// ↑ or ↓: numerator ↔ denominator of the fraction the caret is in; down out of an exponent,
    /// up into one beside the caret; else the statement above or below.
    pub(crate) fn vertical(&self, up: bool) -> Vertical {
        let mut c = self.caret.clone();
        while let Some((above, k)) = parent(&c) {
            match self.row(&above).get(above.pos) {
                Some(Atom::Frac { .. }) if (up && k == 1) || (!up && k == 0) => {
                    let path = [&above.path[..], &[(above.pos, 1 - k)]].concat();
                    return Vertical::Nearest(Caret { path, pos: 0, ..above });
                }
                Some(Atom::Sup(_)) if !up => return Vertical::To(Caret { pos: above.pos + 1, ..above }),
                _ => {}
            }
            c = above;
        }
        let row = self.row(&self.caret);
        let p = self.caret.pos;
        if up {
            if let Some(Atom::Sup(_)) = row.get(p) {
                return Vertical::To(Caret { path: [&self.caret.path[..], &[(p, 0)]].concat(), pos: 0, ..self.caret.clone() });
            }
            if let Some(Atom::Sup(x)) = p.checked_sub(1).and_then(|q| row.get(q)) {
                return Vertical::To(Caret { path: [&self.caret.path[..], &[(p - 1, 0)]].concat(), pos: x.len(), ..self.caret.clone() });
            }
        }
        let all = self.doc.statements();
        let Some(i) = all.iter().position(|&s| s == (self.caret.line, self.caret.stmt)) else { return Vertical::Stay };
        let j = if up { i.checked_sub(1) } else { Some(i + 1) };
        match j.and_then(|j| all.get(j)) {
            Some(&(line, stmt)) => Vertical::Nearest(Caret { line, stmt, path: Vec::new(), pos: 0 }),
            None => Vertical::Stay,
        }
    }
}

#[cfg(test)]
#[path = "edit_tests.rs"]
mod tests;
