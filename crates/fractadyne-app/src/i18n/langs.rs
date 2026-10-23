//! Which languages are installed, which one the system prefers, and loading one
//! (design/multilingual.md §5, §8).
//!
//! A language is found in four places, and for each language the highest revision wins:
//! the program itself (English, the pseudo-language), the release package's `languages/` folder,
//! `<config>/languages/` (installed by hand or by the updater), and a translator's working folder
//! named by `FRACTADYNE_LANG_DIR`, which always wins so a translation can be checked as it is
//! written.

use super::catalog::{Catalog, Source, Stamp};
use super::pseudo;
use std::path::{Path, PathBuf};

/// The environment variable naming a translator's working folder.
pub(crate) const DEV_DIR_ENV: &str = "FRACTADYNE_LANG_DIR";
/// The environment variable choosing the language for one run (as `--language` does).
pub(crate) const LANG_ENV: &str = "FRACTADYNE_LANG";

/// An installed language, as the language menu lists it (read from the file's top, not parsed).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Installed {
    pub(crate) tag: String,
    pub(crate) name: String,
    pub(crate) english_name: String,
    pub(crate) revision: u64,
    pub(crate) source: Source,
}

impl Installed {
    /// The file it is read from (none for a built-in language).
    pub(crate) fn path(&self) -> Option<&Path> {
        match &self.source {
            Source::Builtin => None,
            Source::Package(p) | Source::User(p) | Source::Dev(p) => Some(p),
        }
    }

    /// How the language menu lists it: its own name, and its English name beside it unless the two
    /// are the same.
    pub(crate) fn menu_label(&self) -> String {
        if self.english_name.is_empty() || self.english_name == self.name {
            self.name.clone()
        } else {
            format!("{} — {}", self.name, self.english_name)
        }
    }
}

/// `<config>/languages/`.
pub(crate) fn user_dir() -> Option<PathBuf> {
    fractadyne_state::config_dir().map(|d| d.join("languages"))
}

/// The release package's `languages/` folder, beside the executable.
pub(crate) fn package_dir() -> Option<PathBuf> {
    std::env::current_exe().ok()?.parent().map(|d| d.join("languages"))
}

/// The translator's working folder, if `FRACTADYNE_LANG_DIR` names one.
pub(crate) fn dev_dir() -> Option<PathBuf> {
    std::env::var_os(DEV_DIR_ENV).filter(|v| !v.is_empty()).map(PathBuf::from)
}

/// Every installed language, built-in ones first, then by tag; one entry per language.
pub(crate) fn discover() -> Vec<Installed> {
    discover_in(package_dir().as_deref(), user_dir().as_deref(), dev_dir().as_deref())
}

pub(crate) fn discover_in(package: Option<&Path>, user: Option<&Path>, dev: Option<&Path>) -> Vec<Installed> {
    let en = super::english();
    let mut found = vec![Installed {
        tag: "en".into(),
        name: en.name.clone(),
        english_name: en.english_name.clone(),
        revision: 0,
        source: Source::Builtin,
    }];
    #[derive(Clone, Copy, PartialEq)]
    enum Place {
        Package,
        User,
        Dev,
    }
    let mut files: Vec<Installed> = Vec::new();
    for (dir, place) in [(package, Place::Package), (user, Place::User), (dev, Place::Dev)] {
        let Some(dir) = dir else { continue };
        for (path, tag_hint) in language_files(dir) {
            if let Some(mut i) = peek(&path, &tag_hint) {
                if place == Place::Dev {
                    i.revision = u64::MAX;
                }
                i.source = match place {
                    Place::Package => Source::Package(path),
                    Place::User => Source::User(path),
                    Place::Dev => Source::Dev(path),
                };
                files.push(i);
            }
        }
    }
    files.sort_by(|a, b| a.tag.cmp(&b.tag).then(b.revision.cmp(&a.revision)));
    files.dedup_by(|b, a| a.tag == b.tag);
    // A file may also stand in for English (a translator's English): it replaces the built-in entry.
    if let Some(pos) = files.iter().position(|f| f.tag == "en") {
        found[0] = files.remove(pos);
    }
    found.extend(files);
    found
}

/// The language files in `dir`: `fractadyne-<tag>.ftl` (a package or the config folder), or
/// `<tag>/fractadyne.ftl` (the repository's `i18n/` layout, a translator's working copy).
fn language_files(dir: &Path) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else { return out };
    for e in rd.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        if p.is_file() {
            if let Some(tag) = name.strip_prefix("fractadyne-").and_then(|n| n.strip_suffix(".ftl")) {
                out.push((p, tag.to_owned()));
            }
        } else if p.is_dir() {
            let f = p.join("fractadyne.ftl");
            if f.is_file() {
                out.push((f, name));
            }
        }
    }
    out.sort();
    out
}

/// A file's tag, names and revision, from its first lines: enough to list it without parsing it.
fn peek(path: &Path, tag_hint: &str) -> Option<Installed> {
    use std::io::Read;
    let mut head = Vec::new();
    std::fs::File::open(path).ok()?.take(64 * 1024).read_to_end(&mut head).ok()?;
    let text = super::catalog::clean_text(&String::from_utf8_lossy(&head));
    let stamp = Stamp::read(&text);
    let tag = canonical_tag(stamp.tag.as_deref().unwrap_or(tag_hint))?;
    if tag == pseudo::TAG {
        return None;
    }
    let field = |key: &str| {
        text.lines().find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim_end() == key && !l.starts_with(' ')).then(|| v.trim().to_owned()).filter(|v| !v.is_empty() && !v.contains('{'))
        })
    };
    Some(Installed {
        name: field("language-name").unwrap_or_else(|| tag.clone()),
        english_name: field("language-english-name").unwrap_or_default(),
        tag,
        revision: stamp.revision,
        source: Source::Builtin,
    })
}

/// `tag` in canonical form (`zh-hans` → `zh-Hans`), or `None` when it is not a language tag.
pub(crate) fn canonical_tag(tag: &str) -> Option<String> {
    tag.parse::<unic_langid::LanguageIdentifier>().ok().map(|l| l.to_string())
}

/// The catalog of an installed language: built in, or its file loaded and checked.
pub(crate) fn load(i: &Installed) -> Result<Catalog, String> {
    match i.path() {
        None if i.tag == pseudo::TAG => Ok(Catalog::pseudo()),
        None => Ok(Catalog::english()),
        Some(p) => {
            let bytes = std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()))?;
            if bytes.len() > super::catalog::MAX_FILE_BYTES {
                return Err(format!("{}: too large for a language file", p.display()));
            }
            let text = String::from_utf8(bytes).map_err(|_| format!("{}: not UTF-8 text", p.display()))?;
            Catalog::from_text(&text, &i.tag)
        }
    }
}

/// The system's preferred languages, most preferred first (`fr-CA`, `en-US`…).
pub(crate) fn system_languages() -> Vec<String> {
    sys_locale::get_locales().filter_map(|l| canonical_tag(&l)).collect()
}

/// The installed language that best matches `requested` (most preferred first), if any does:
/// `fr-CA` takes `fr`, `zh-CN` takes `zh-Hans`. English matches only an English request, so a
/// language the app lacks falls through to the caller's default rather than to English silently.
pub(crate) fn best_match(requested: &[String], installed: &[Installed]) -> Option<String> {
    use fluent_langneg::{NegotiationStrategy, negotiate_languages};
    use unic_langid::LanguageIdentifier;
    let req: Vec<LanguageIdentifier> = requested.iter().filter_map(|t| t.parse().ok()).map(add_likely_script).collect();
    let avail: Vec<LanguageIdentifier> = installed.iter().filter(|i| i.tag != pseudo::TAG).filter_map(|i| i.tag.parse().ok()).collect();
    let got = negotiate_languages(&req, &avail, None, NegotiationStrategy::Lookup);
    got.first().map(|l| l.to_string())
}

/// Chinese written in mainland China and Singapore is Simplified, in Taiwan, Hong Kong and Macau
/// Traditional: a system reporting `zh-CN` should match a `zh-Hans` file.
fn add_likely_script(mut l: unic_langid::LanguageIdentifier) -> unic_langid::LanguageIdentifier {
    if l.language.as_str() == "zh" && l.script.is_none() {
        let traditional = l.region.is_some_and(|r| matches!(r.as_str(), "TW" | "HK" | "MO"));
        l.script = Some(if traditional { "Hant" } else { "Hans" }.parse().expect("a script code"));
    }
    l
}

#[cfg(test)]
#[path = "../../tests/unit/i18n/langs/tests.rs"]
mod tests;
