//! One language's messages, checked and ready to format (design/multilingual.md §4, §6).
//!
//! A language file is untrusted text — anyone can write one, and the updater downloads them — so
//! before its messages are used the file is cleaned and each message checked: it parses, it uses
//! no argument the English message does not declare, it is not absurdly long, and it carries no URL.
//! A message that fails is set aside and English answers for it; the rest of the file is used.

use super::{Arg, Msg, Punctuation, id, pseudo};
use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use fluent_syntax::ast;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

/// A language file is refused above this size (a full translation is ~300 KB).
pub(crate) const MAX_FILE_BYTES: usize = 2 * 1024 * 1024;

/// The messages that describe the language rather than the interface (the file's "language data").
pub(crate) const DATA_IDS: &[&str] =
    &["language-name", "language-english-name", "language-script", "number-decimal", "number-group", "language-status"];

/// Where a catalog came from: the language window shows it, and the highest revision of a language
/// wins (a translator's working folder always).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Source {
    /// Compiled into the program (English, and the pseudo-language).
    Builtin,
    /// The release package's `languages/` folder.
    Package(PathBuf),
    /// `<config>/languages/`: installed by hand or by the updater.
    User(PathBuf),
    /// `FRACTADYNE_LANG_DIR`: a translator's working copy, reloaded when it changes.
    Dev(PathBuf),
}

/// A message set aside, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Dropped {
    pub(crate) id: String,
    pub(crate) why: &'static str,
}

/// One language, ready to format.
pub(crate) struct Catalog {
    /// Its BCP 47 tag: `en`, `de`, `zh-Hans`, `en-XA` (the pseudo-language).
    pub(crate) tag: String,
    /// Its own name (`Deutsch`), and its name in English (`German`).
    pub(crate) name: String,
    pub(crate) english_name: String,
    /// The writing system it needs (ISO 15924: `Latn`, `Cyrl`, `Jpan`, `Hans`, `Khmr`).
    pub(crate) script: String,
    /// The publish stamp's revision; 0 when unstamped.
    pub(crate) revision: u64,
    /// `draft` until a native speaker has reviewed it, then `reviewed` (`source` for English).
    pub(crate) status: String,
    /// Interface messages (not language data) this file translates and that passed the checks.
    pub(crate) translated: usize,
    /// Messages set aside by the checks.
    pub(crate) dropped: Vec<Dropped>,
    /// Entries the parser could not read (each skipped on its own).
    pub(crate) parse_errors: usize,
    pub(crate) number: Punctuation,
    pseudo: bool,
    english: Arc<FluentBundle<FluentResource>>,
    own: Option<FluentBundle<FluentResource>>,
    /// Per `Msg`: the file's own message may answer it.
    usable: Vec<bool>,
    /// Per `Msg` without arguments: its text, formatted once.
    cache: Vec<Option<String>>,
}

/// Interface messages in the English file (the denominator of every coverage figure).
pub(crate) fn interface_messages() -> usize {
    id::NAMES.iter().filter(|n| !DATA_IDS.contains(n)).count()
}

impl Catalog {
    /// English, from the compiled-in file.
    pub(crate) fn english() -> Catalog {
        let english = english_bundle();
        let mut c = Catalog::bare("en", english);
        c.translated = interface_messages();
        c.finish();
        c
    }

    /// The pseudo-language `en-XA`: English, accented, 40% longer and bracketed, so text still
    /// hard-coded in English and text cut off by its box both show (§10).
    pub(crate) fn pseudo() -> Catalog {
        let mut c = Catalog::bare(pseudo::TAG, pseudo_bundle());
        c.pseudo = true;
        c.translated = interface_messages();
        c.finish();
        c.name = pseudo::NAME.to_owned();
        c.english_name = pseudo::ENGLISH_NAME.to_owned();
        c
    }

    /// A language file's text, cleaned and checked (§6). `tag` is the tag the file is known by (its
    /// file name or its publish stamp). Errs only for a file that cannot be a language file at all.
    pub(crate) fn from_text(text: &str, tag: &str) -> Result<Catalog, String> {
        if text.len() > MAX_FILE_BYTES {
            return Err(format!("{} KB, over the {} KB a language file may be", text.len() / 1024, MAX_FILE_BYTES / 1024));
        }
        let clean = clean_text(text);
        let stamp = Stamp::read(&clean);
        let lang: unic_langid::LanguageIdentifier = tag.parse().map_err(|_| format!("{tag:?} is not a language tag"))?;
        let (resource, parse_errors) = match FluentResource::try_new(clean) {
            Ok(r) => (r, 0),
            Err((r, errs)) => (r, errs.len()),
        };
        let english = english_bundle();
        let mut c = Catalog::bare(tag, english);
        c.revision = stamp.revision;
        c.parse_errors = parse_errors;
        let shapes = english_shapes();
        for entry in resource.entries() {
            let ast::Entry::Message(m) = entry else { continue };
            let Some(idx) = id::NAMES.iter().position(|n| *n == m.id.name) else { continue };
            let Some(value) = &m.value else {
                c.dropped.push(Dropped { id: m.id.name.to_owned(), why: "it has no text" });
                continue;
            };
            let mut vars = Vec::new();
            pattern_vars(value, &mut vars);
            let why = if vars.iter().any(|v| !id::VARS[idx].contains(v)) {
                Some("it uses an argument the English message does not have")
            } else if pattern_len(value) > shapes[idx] * 4 + 64 {
                Some("it is more than four times the length of the English")
            } else if pattern_has_url(value) {
                Some("it contains a web address")
            } else {
                None
            };
            match why {
                Some(why) => c.dropped.push(Dropped { id: m.id.name.to_owned(), why }),
                None => c.usable[idx] = true,
            }
        }
        let mut bundle = FluentBundle::new_concurrent(vec![lang]);
        configure(&mut bundle);
        // Duplicate entries keep the first; the parser already reported anything unreadable.
        let _ = bundle.add_resource(resource);
        c.own = Some(bundle);
        c.translated = id::NAMES.iter().enumerate().filter(|(i, n)| c.usable[*i] && !DATA_IDS.contains(n)).count();
        c.finish();
        Ok(c)
    }

    fn bare(tag: &str, english: Arc<FluentBundle<FluentResource>>) -> Catalog {
        Catalog {
            tag: tag.to_owned(),
            name: String::new(),
            english_name: String::new(),
            script: String::new(),
            revision: 0,
            status: String::new(),
            translated: 0,
            dropped: Vec::new(),
            parse_errors: 0,
            number: Punctuation::default(),
            pseudo: false,
            english,
            own: None,
            usable: vec![false; id::COUNT],
            cache: vec![None; id::COUNT],
        }
    }

    /// Read the language data and format every message without arguments once. A file's data comes
    /// from the file alone (a German file without a name is known by its tag, never as "English").
    fn finish(&mut self) {
        self.name = self.data("language-name").unwrap_or_else(|| self.tag.clone());
        self.english_name = self.data("language-english-name").unwrap_or_else(|| self.tag.clone());
        self.script = self.data("language-script").filter(|s| s.len() == 4 && s.chars().all(|c| c.is_ascii_alphabetic())).unwrap_or_else(|| "Latn".into());
        let one = |s: Option<String>, d: char| {
            s.and_then(|s| {
                let mut it = s.chars();
                match (it.next(), it.next()) {
                    (Some(c), None) if !c.is_alphanumeric() && !c.is_control() => Some(c),
                    _ => None,
                }
            })
            .unwrap_or(d)
        };
        self.status = self.data("language-status").unwrap_or_default();
        // Not trimmed: a no-break space is a separator, not padding around one.
        let decimal = one(self.data_raw("number-decimal"), '.');
        let group = one(self.data_raw("number-group"), ',');
        self.number = if decimal == group { Punctuation::default() } else { Punctuation { decimal, group } };
        for i in 0..id::COUNT {
            if id::VARS[i].is_empty() {
                let s = self.resolve(Msg(i as u16), None);
                self.cache[i] = Some(s);
            }
        }
    }

    /// A language-data message's text, trimmed: the file's own (English's for a catalog with no file).
    fn data(&self, name: &str) -> Option<String> {
        self.data_raw(name).map(|s| s.trim().to_owned()).filter(|s| !s.is_empty())
    }

    /// The same, as written.
    fn data_raw(&self, name: &str) -> Option<String> {
        let idx = id::NAMES.iter().position(|n| *n == name)?;
        match &self.own {
            Some(b) => self.usable[idx].then(|| format_in(b, name, None)).flatten(),
            None => format_in(&self.english, name, None),
        }
    }


    /// The message `m`.
    pub(crate) fn text(&self, m: Msg) -> String {
        match self.cache.get(m.0 as usize) {
            Some(Some(s)) => s.clone(),
            _ => self.format(m, &[]),
        }
    }

    /// The message `m` with its arguments.
    pub(crate) fn format(&self, m: Msg, args: &[(&'static str, Arg)]) -> String {
        if args.is_empty() {
            if let Some(Some(s)) = self.cache.get(m.0 as usize) {
                return s.clone();
            }
        }
        let mut fa = FluentArgs::new();
        for (k, v) in args {
            match v {
                Arg::Num(n) => fa.set(*k, FluentValue::from(*n)),
                Arg::Text(t) => fa.set(*k, FluentValue::from(t.as_str())),
            }
        }
        self.resolve(m, Some(&fa))
    }

    fn resolve(&self, m: Msg, args: Option<&FluentArgs>) -> String {
        let i = m.0 as usize;
        let Some(name) = id::NAMES.get(i) else { return String::new() };
        // The file's own message only when it formats cleanly; English's even when an argument is
        // missing (Fluent shows the gap as `{$name}`), so a message is never its bare ID.
        let own = self.own.as_ref().filter(|_| self.usable[i]).and_then(|b| format_in(b, name, args));
        let s = own.or_else(|| format_lenient(&self.english, name, args)).unwrap_or_else(|| (*name).to_owned());
        if self.pseudo { pseudo::wrap(&s) } else { s }
    }

    /// The share of the interface this catalog translates, 0–100.
    pub(crate) fn coverage(&self) -> u32 {
        let n = interface_messages().max(1);
        ((self.translated * 100 + n / 2) / n) as u32
    }
}

/// `name` formatted from `b`; `None` when `b` lacks it or formatting reported an error (a missing
/// argument, a broken reference), so English can answer instead.
fn format_in(b: &FluentBundle<FluentResource>, name: &str, args: Option<&FluentArgs>) -> Option<String> {
    let msg = b.get_message(name)?;
    let pattern = msg.value()?;
    let mut errors = Vec::new();
    let s = b.format_pattern(pattern, args, &mut errors);
    errors.is_empty().then(|| s.into_owned())
}

/// `name` formatted from `b` whatever errors formatting reported; `None` only when `b` lacks it.
fn format_lenient(b: &FluentBundle<FluentResource>, name: &str, args: Option<&FluentArgs>) -> Option<String> {
    let msg = b.get_message(name)?;
    let pattern = msg.value()?;
    let mut errors = Vec::new();
    Some(b.format_pattern(pattern, args, &mut errors).into_owned())
}

/// The English bundle, built once and shared by every catalog as its fallback.
/// The English file with the pseudo-language's accents on each message's own text (Fluent's
/// transform: the arguments are not touched).
fn pseudo_bundle() -> Arc<FluentBundle<FluentResource>> {
    let r = FluentResource::try_new(super::ENGLISH.to_owned()).unwrap_or_else(|(r, _)| r);
    let en: unic_langid::LanguageIdentifier = "en".parse().expect("en is a language tag");
    let mut b = FluentBundle::new_concurrent(vec![en]);
    configure(&mut b);
    b.set_transform(Some(pseudo::accent_text));
    let _ = b.add_resource(r);
    Arc::new(b)
}

fn english_bundle() -> Arc<FluentBundle<FluentResource>> {
    static B: OnceLock<Arc<FluentBundle<FluentResource>>> = OnceLock::new();
    B.get_or_init(|| {
        let r = FluentResource::try_new(super::ENGLISH.to_owned()).unwrap_or_else(|(r, _)| r);
        let en: unic_langid::LanguageIdentifier = "en".parse().expect("en is a language tag");
        let mut b = FluentBundle::new_concurrent(vec![en]);
        configure(&mut b);
        let _ = b.add_resource(r);
        Arc::new(b)
    })
    .clone()
}

/// Every bundle: no Unicode isolation marks around arguments (egui would draw them as boxes and the
/// interface is left-to-right only, §3), and numbers in the language in use's punctuation.
fn configure(b: &mut FluentBundle<FluentResource>) {
    b.set_use_isolating(false);
    b.set_formatter(Some(format_number));
}

fn format_number<M>(v: &FluentValue, _: &M) -> Option<String> {
    match v {
        FluentValue::Number(n) => Some(super::number(n.value)),
        _ => None,
    }
}

/// The English messages' lengths in characters (the longest variant of a selector), by `Msg`.
fn english_shapes() -> &'static [usize] {
    static S: OnceLock<Vec<usize>> = OnceLock::new();
    S.get_or_init(|| {
        let r = FluentResource::try_new(super::ENGLISH.to_owned()).unwrap_or_else(|(r, _)| r);
        let mut v = vec![0usize; id::COUNT];
        for e in r.entries() {
            if let ast::Entry::Message(m) = e {
                if let (Some(i), Some(p)) = (id::NAMES.iter().position(|n| *n == m.id.name), &m.value) {
                    v[i] = pattern_len(p);
                }
            }
        }
        v
    })
}

/// The variables a pattern uses, in order of first use.
pub(crate) fn pattern_vars<'s>(p: &ast::Pattern<&'s str>, out: &mut Vec<&'s str>) {
    for el in &p.elements {
        if let ast::PatternElement::Placeable { expression } = el {
            expression_vars(expression, out);
        }
    }
}

fn expression_vars<'s>(e: &ast::Expression<&'s str>, out: &mut Vec<&'s str>) {
    match e {
        ast::Expression::Inline(i) => inline_vars(i, out),
        ast::Expression::Select { selector, variants } => {
            inline_vars(selector, out);
            for v in variants {
                pattern_vars(&v.value, out);
            }
        }
    }
}

fn inline_vars<'s>(e: &ast::InlineExpression<&'s str>, out: &mut Vec<&'s str>) {
    match e {
        ast::InlineExpression::VariableReference { id } => {
            if !out.contains(&id.name) {
                out.push(id.name);
            }
        }
        ast::InlineExpression::FunctionReference { arguments, .. } => {
            for a in &arguments.positional {
                inline_vars(a, out);
            }
            for n in &arguments.named {
                inline_vars(&n.value, out);
            }
        }
        ast::InlineExpression::TermReference { arguments: Some(a), .. } => {
            for p in &a.positional {
                inline_vars(p, out);
            }
            for n in &a.named {
                inline_vars(&n.value, out);
            }
        }
        ast::InlineExpression::Placeable { expression } => expression_vars(expression, out),
        _ => {}
    }
}

/// A pattern's length in characters: its text, each placeable as 8, a selector as its longest variant.
pub(crate) fn pattern_len(p: &ast::Pattern<&str>) -> usize {
    p.elements
        .iter()
        .map(|el| match el {
            ast::PatternElement::TextElement { value } => value.chars().count(),
            ast::PatternElement::Placeable { expression: ast::Expression::Select { variants, .. } } => {
                variants.iter().map(|v| pattern_len(&v.value)).max().unwrap_or(0)
            }
            ast::PatternElement::Placeable { .. } => 8,
        })
        .sum()
}

fn pattern_has_url(p: &ast::Pattern<&str>) -> bool {
    p.elements.iter().any(|el| match el {
        ast::PatternElement::TextElement { value } => {
            let v = value.to_ascii_lowercase();
            v.contains("://") || v.contains("www.")
        }
        ast::PatternElement::Placeable { expression: ast::Expression::Select { variants, .. } } => variants.iter().any(|v| pattern_has_url(&v.value)),
        ast::PatternElement::Placeable { expression: ast::Expression::Inline(ast::InlineExpression::StringLiteral { value }) } => {
            let v = value.to_ascii_lowercase();
            v.contains("://") || v.contains("www.")
        }
        _ => false,
    })
}

/// A language file's text made safe to parse (§6): a byte-order mark dropped, line endings LF,
/// control characters other than newline and tab removed, and the bidirectional embedding,
/// override and isolate controls removed (they make text display in an order other than it reads,
/// and the interface draws left to right only). Typographic quotes and dashes, no-break spaces,
/// ZWJ/ZWNJ and the zero-width space are kept: they are language, not damage.
pub(crate) fn clean_text(s: &str) -> String {
    let s = s.strip_prefix('\u{FEFF}').unwrap_or(s);
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                if chars.peek() != Some(&'\n') {
                    out.push('\n');
                }
            }
            '\n' | '\t' => out.push(c),
            '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' => {}
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    out
}

/// A language file's publish stamp (§4): `### tag:` and `### revision:` lines at its top.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Stamp {
    pub(crate) tag: Option<String>,
    pub(crate) revision: u64,
}

impl Stamp {
    pub(crate) fn read(text: &str) -> Stamp {
        let mut st = Stamp::default();
        for line in text.lines().take_while(|l| l.starts_with("###") || l.trim().is_empty()) {
            let Some(rest) = line.strip_prefix("###") else { continue };
            let Some((k, v)) = rest.split_once(':') else { continue };
            match k.trim() {
                "tag" => st.tag = Some(v.trim().to_owned()).filter(|t| !t.is_empty()),
                "revision" => st.revision = v.trim().parse().unwrap_or(0),
                _ => {}
            }
        }
        st
    }
}

#[cfg(test)]
#[path = "../../tests/unit/i18n/catalog/tests.rs"]
mod tests;
