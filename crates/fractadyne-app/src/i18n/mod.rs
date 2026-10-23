//! The interface's languages (design/multilingual.md). One Fluent file per language: English is
//! compiled in (`i18n/en/fractadyne.ftl`), and every other language is a file read at run time and
//! checked before it is used (§6). `tr!(id)` and `tr!(id, name = value, …)` give a message in the
//! language in use, falling back to English message by message.
//!
//! Data never goes through here: coordinates, magnifications, files, the command line and logs stay
//! English (§2).

pub(crate) mod catalog;
pub(crate) mod langs;
pub(crate) mod pseudo;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, RwLock};

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

/// Bumped by every `set_active`: a cache of laid-out text (the live panel's static layer) redoes
/// itself when this moves.
#[allow(dead_code)] // its first reader is the live panel, once its legends move into the file
pub(crate) fn generation() -> u64 {
    GENERATION.load(Ordering::Relaxed)
}

/// The message `m` in the language in use (`tr!(id)`).
pub(crate) fn text(m: Msg) -> String {
    active().text(m)
}

/// The message `m` with its arguments, in the language in use (`tr!(id, k = v, …)`).
pub(crate) fn format(m: Msg, args: &[(&'static str, Arg)]) -> String {
    active().format(m, args)
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

fn set_punctuation(p: Punctuation) {
    PUNCT.store(((p.decimal as u64) << 32) | p.group as u64, Ordering::Relaxed);
}

/// The punctuation of the language in use.
pub(crate) fn punctuation() -> Punctuation {
    let v = PUNCT.load(Ordering::Relaxed);
    let ch = |c: u64| char::from_u32(c as u32).unwrap_or('?');
    Punctuation { decimal: ch(v >> 32), group: ch(v & 0xFFFF_FFFF) }
}

/// A count as the language in use writes it: `1,234,567`, `1.234.567`, `1 234 567`.
#[allow(dead_code)] // the counts formatted today (`commas`, main.rs) move onto it with their areas (§7)
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
