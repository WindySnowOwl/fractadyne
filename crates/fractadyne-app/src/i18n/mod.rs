//! The interface's languages (design/multilingual.md). One Fluent file per language: English is
//! compiled in (`i18n/en/fractadyne.ftl`), and every other language is a file read at run time and
//! checked before it is used (§6). `tr!(id)` and `tr!(id, name = value, …)` give a message in the
//! language in use, falling back to English message by message.
//!
//! Data never goes through here: coordinates, magnifications, files, the command line and logs stay
//! English (§2).

pub(crate) mod catalog;
pub(crate) mod fonts;
pub(crate) mod langs;
pub(crate) mod pseudo;

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

pub(crate) use catalog::{Catalog, Source};

/// A message of the English file: the index of one of the `id::*` constants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Msg(pub(crate) u16);

/// The message constants and tables, generated from the English file by build.rs: a misspelt or
/// removed message does not compile.
pub(crate) mod id {
    include!(concat!(env!("OUT_DIR"), "/i18n_ids.rs"));
}

/// The English file: the source language, and every message's fallback.
pub(crate) const ENGLISH: &str = include_str!("../../../../i18n/en/fractadyne.ftl");

/// A message's argument: a number (plural forms choose by it, and it is printed with the language's
/// punctuation) or text.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Arg {
    Num(f64),
    Text(String),
}

impl From<&str> for Arg {
    fn from(s: &str) -> Self {
        Arg::Text(s.to_owned())
    }
}

impl From<String> for Arg {
    fn from(s: String) -> Self {
        Arg::Text(s)
    }
}

impl From<&String> for Arg {
    fn from(s: &String) -> Self {
        Arg::Text(s.clone())
    }
}

macro_rules! arg_from_number {
    ($($t:ty),*) => {$(
        impl From<$t> for Arg {
            fn from(n: $t) -> Self {
                Arg::Num(n as f64)
            }
        }
    )*};
}
arg_from_number!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);

/// The message `id` in the language in use: `tr!(menu_file)`, `tr!(toast_saved, path = p, count = n)`.
macro_rules! tr {
    ($id:ident) => {
        $crate::i18n::text($crate::i18n::id::$id)
    };
    ($id:ident, $($k:ident = $v:expr),+ $(,)?) => {
        $crate::i18n::format($crate::i18n::id::$id, &[$((stringify!($k), $crate::i18n::Arg::from($v))),+])
    };
}

static ACTIVE: RwLock<Option<Arc<Catalog>>> = RwLock::new(None);
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// The English catalog, built once.
pub(crate) fn english() -> Arc<Catalog> {
    static E: OnceLock<Arc<Catalog>> = OnceLock::new();
    E.get_or_init(|| Arc::new(Catalog::english())).clone()
}

/// The catalog in use (English until another is set).
pub(crate) fn active() -> Arc<Catalog> {
    ACTIVE.read().ok().and_then(|g| g.clone()).unwrap_or_else(english)
}

/// Use `c` from the next message on. Caches of laid-out text compare `generation()` to know.
pub(crate) fn set_active(c: Arc<Catalog>) {
    set_punctuation(c.number);
    if let Ok(mut g) = ACTIVE.write() {
        *g = Some(c);
    }
    GENERATION.fetch_add(1, Ordering::Relaxed);
}

/// Bumped by every `set_active` and `set_decimal_mark`: a cache of laid-out text (the live panel's
/// static layer) redoes itself when this moves.
pub(crate) fn generation() -> u64 {
    GENERATION.load(Ordering::Relaxed)
}

/// The message `m` in the language in use (`tr!(id)`).
pub(crate) fn text(m: Msg) -> String {
    noted(active().text(m))
}

/// The message `m` with its arguments, in the language in use (`tr!(id, k = v, …)`).
pub(crate) fn format(m: Msg, args: &[(&'static str, Arg)]) -> String {
    noted(active().format(m, args))
}

/// Whether every message text handed out is kept (`--uitest`, see [`record_texts`]).
static RECORD: AtomicBool = AtomicBool::new(false);
static RECORDED: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// `--uitest`: keep every message text handed out from now on, so its text-layout check can tell
/// the interface's own text from data on the screen, in any language (its text-layout check).
pub(crate) fn record_texts() {
    RECORD.store(true, Ordering::Relaxed);
}

/// Every distinct message text handed out since [`record_texts`].
pub(crate) fn recorded_texts() -> Vec<String> {
    let mut v = RECORDED.lock().map(|g| g.clone()).unwrap_or_default();
    v.sort();
    v.dedup();
    v
}

fn noted(s: String) -> String {
    if RECORD.load(Ordering::Relaxed) {
        if let Ok(mut g) = RECORDED.lock() {
            // Deduplicated in bulk: a frame hands out the same texts again, and a sort per call costs.
            if g.len() > 50_000 {
                g.sort();
                g.dedup();
            }
            g.push(s.clone());
        }
    }
    s
}

/// How the language in use writes numbers: its decimal mark and its digit-group separator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Punctuation {
    pub(crate) decimal: char,
    pub(crate) group: char,
}

impl Default for Punctuation {
    fn default() -> Self {
        Punctuation { decimal: '.', group: ',' }
    }
}

/// The language in use's punctuation, packed (decimal << 32 | group), for the Fluent formatter,
/// which is a plain function and cannot carry it.
static PUNCT: AtomicU64 = AtomicU64::new((('.' as u64) << 32) | ',' as u64);

/// The user's decimal mark ([`DecimalMark`] as its index): over the language's own.
static MARK: AtomicU8 = AtomicU8::new(0);

fn set_punctuation(p: Punctuation) {
    PUNCT.store(((p.decimal as u64) << 32) | p.group as u64, Ordering::Relaxed);
}

/// The language in use's own punctuation, before the user's decimal mark.
fn language_punctuation() -> Punctuation {
    let v = PUNCT.load(Ordering::Relaxed);
    let ch = |c: u64| char::from_u32(c as u32).unwrap_or('?');
    Punctuation { decimal: ch(v >> 32), group: ch(v & 0xFFFF_FFFF) }
}

/// The punctuation numbers are written with: the language's, with the user's decimal mark.
pub(crate) fn punctuation() -> Punctuation {
    decimal_mark().apply(language_punctuation())
}

/// Which decimal mark numbers take (Language menu ▸ Decimal mark): the language's own, or the one
/// the user prefers, whatever the language.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum DecimalMark {
    #[default]
    Language,
    Point,
    Comma,
}

impl DecimalMark {
    pub(crate) const ALL: [DecimalMark; 3] = [DecimalMark::Language, DecimalMark::Point, DecimalMark::Comma];

    /// The persisted token (`SessionState::decimal_mark`): empty for the language's own.
    pub(crate) fn key(self) -> &'static str {
        match self {
            DecimalMark::Language => "",
            DecimalMark::Point => ".",
            DecimalMark::Comma => ",",
        }
    }

    /// The token read back; anything else is the language's own.
    pub(crate) fn from_key(s: &str) -> DecimalMark {
        match s {
            "." => DecimalMark::Point,
            "," => DecimalMark::Comma,
            _ => DecimalMark::Language,
        }
    }

    /// `p` with this decimal mark. The digit groups keep the language's separator unless it is the
    /// chosen mark itself, which would make `1.234.5` unreadable: then they take the other one.
    pub(crate) fn apply(self, p: Punctuation) -> Punctuation {
        let (decimal, other) = match self {
            DecimalMark::Language => return p,
            DecimalMark::Point => ('.', ','),
            DecimalMark::Comma => (',', '.'),
        };
        Punctuation { decimal, group: if p.group == decimal { other } else { p.group } }
    }
}

/// The user's decimal mark.
pub(crate) fn decimal_mark() -> DecimalMark {
    match MARK.load(Ordering::Relaxed) {
        1 => DecimalMark::Point,
        2 => DecimalMark::Comma,
        _ => DecimalMark::Language,
    }
}

/// Write numbers with `m` from the next message on.
pub(crate) fn set_decimal_mark(m: DecimalMark) {
    let i = match m {
        DecimalMark::Language => 0,
        DecimalMark::Point => 1,
        DecimalMark::Comma => 2,
    };
    if MARK.swap(i, Ordering::Relaxed) != i {
        GENERATION.fetch_add(1, Ordering::Relaxed);
    }
}

/// The language's own punctuation with `m`, for showing what each choice looks like.
pub(crate) fn punctuation_with(m: DecimalMark) -> Punctuation {
    m.apply(language_punctuation())
}

/// A count as the language in use writes it: `1,234,567`, `1.234.567`, `1 234 567`.
pub(crate) fn count(n: u64) -> String {
    group_digits(&n.to_string(), punctuation().group)
}

/// A number as the language in use writes it: whole numbers grouped, fractions with its decimal
/// mark (`12,345.6` / `12.345,6`). Not for coordinates or magnifications, which keep `.` (§2).
pub(crate) fn number(v: f64) -> String {
    number_with(v, punctuation())
}

pub(crate) fn number_with(v: f64, p: Punctuation) -> String {
    if !v.is_finite() {
        return v.to_string();
    }
    let s = if v.fract() == 0.0 && v.abs() < 1e15 { format!("{}", v as i64) } else { v.to_string() };
    let (sign, body) = s.strip_prefix('-').map_or(("", s.as_str()), |b| ("-", b));
    let (int, frac) = body.split_once('.').map_or((body, None), |(i, f)| (i, Some(f)));
    let mut out = String::from(sign);
    out += &group_digits(int, p.group);
    if let Some(f) = frac {
        out.push(p.decimal);
        out += f;
    }
    out
}

/// A measurement with `places` decimals, as numbers are written in the interface: the whole part
/// grouped, the user's decimal mark (`1,234.5 ms`, `1.234,5 ms`). For what the interface measures
/// and says (times, sizes, ratios, rates), never for coordinates or magnifications, which keep `.`
/// (§2). Rounds as `format!("{:.N}")` does, `-0.0` included.
pub(crate) fn decimal(x: impl Into<f64>, places: usize) -> String {
    decimal_with(x.into(), places, false, punctuation())
}

/// [`decimal`] with a sign always written: `+12.5`, `-3.0`.
pub(crate) fn decimal_signed(x: impl Into<f64>, places: usize) -> String {
    decimal_with(x.into(), places, true, punctuation())
}

pub(crate) fn decimal_with(x: f64, places: usize, plus: bool, p: Punctuation) -> String {
    if !x.is_finite() {
        return x.to_string();
    }
    let s = if plus { format!("{x:+.places$}") } else { format!("{x:.places$}") };
    let (sign, body) = match s.chars().next() {
        Some(c @ ('-' | '+')) => (c.to_string(), &s[1..]),
        _ => (String::new(), s.as_str()),
    };
    let (int, frac) = body.split_once('.').map_or((body, None), |(i, f)| (i, Some(f)));
    let mut out = sign;
    out += &group_digits(int, p.group);
    if let Some(f) = frac {
        out.push(p.decimal);
        out += f;
    }
    out
}

/// `digits` (ASCII, no sign) in groups of three from the right, separated by `sep`.
pub(crate) fn group_digits(digits: &str, sep: char) -> String {
    let n = digits.len();
    let mut out = String::with_capacity(n + n / 3 * sep.len_utf8());
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (n - i) % 3 == 0 {
            out.push(sep);
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
#[path = "../../tests/unit/i18n/tests.rs"]
mod tests;
