//! The custom-formula library: formulas saved by name in `formulas.toml` in the config dir (beside
//! the bookmarks and the gradient library), and the file format Export writes and Import reads. A
//! library file IS an export: sharing one formula is exporting a library of one, and a copy of
//! `formulas.toml` imports as it stands.
//!
//! An entry is TEXT, as typed: the step's source and each parameter's re and im. A formula's
//! identity is its text (`custom_formula.rs`), and a number stays as the user wrote it ("0.1", not
//! the double nearest it), so nothing is lost if constants are ever read more exactly than f64.
//!
//! Two things the bookmark and gradient files do not do, which a library of typed-in work needs:
//! the write is atomic (temp + rename), and a file that cannot be read is moved aside and reported
//! instead of being replaced, silently, by the next save.

use fractadyne_core::ir::parse::MAX_PARAMS;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// What a formula file says it is, so Import can turn away some other TOML with a sentence.
pub(crate) const FORMAT: &str = "fractadyne-formulas";
/// The file's schema. Additive: an unknown field is ignored, so a newer file imports best-effort.
pub(crate) const VERSION: u32 = 1;
/// The largest file Import reads: a library of thousands of formulas is a few hundred KiB.
pub(crate) const FILE_MAX: u64 = 4 * 1024 * 1024;
/// A longer name is cut; a longer source is not a formula and is skipped.
const NAME_MAX: usize = 120;
const SOURCE_MAX: usize = 16 * 1024;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct SavedFormula {
    pub(crate) name: String,
    /// The step as written; statements on separate lines.
    pub(crate) source: String,
    /// `p1`… as typed, `[re, im]` — only the ones the formula reads.
    #[serde(default)]
    pub(crate) params: Vec<[String; 2]>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct FormulaFile {
    #[serde(default)]
    format: String,
    #[serde(default)]
    version: u32,
    #[serde(default)]
    formula: Vec<SavedFormula>,
}

impl SavedFormula {
    /// The entry as stored: line breaks as `\n` (a CR LF or lone CR from another editor becomes one
    /// — `str::lines` would miss the lone CR), other control characters dropped, the name trimmed
    /// and cut to [`NAME_MAX`] (a blank one named after the source), at most [`MAX_PARAMS`]
    /// parameters. `None` for an empty source or one past [`SOURCE_MAX`]. Repair, don't reject:
    /// everything else stays as written, including a source that no longer parses (the list says
    /// why; it may be a newer build's formula, and it can still be edited).
    pub(crate) fn tidy(self) -> Option<SavedFormula> {
        let mut source = String::with_capacity(self.source.len());
        let mut chars = self.source.chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    source.push('\n');
                }
                '\n' | '\t' => source.push(ch),
                c if c.is_control() => {}
                c => source.push(c),
            }
        }
        let source = source.trim().to_string();
        if source.is_empty() || source.len() > SOURCE_MAX {
            return None;
        }
        let one_line = |s: &str| -> String { s.chars().map(|c| if c.is_control() { ' ' } else { c }).collect() };
        let mut name: String = one_line(&self.name).trim().chars().take(NAME_MAX).collect();
        if name.is_empty() {
            name = first_statement(&source).chars().take(40).collect();
        }
        let params = self
            .params
            .into_iter()
            .take(MAX_PARAMS)
            .map(|[re, im]| [one_line(&re).trim().to_string(), one_line(&im).trim().to_string()])
            .collect();
        Some(SavedFormula { name, source, params })
    }

    /// The same formula, whatever the names: the same source and parameter values (a parameter
    /// missing on one side is 0, as the formula reads it; "0.50" is "0.5").
    pub(crate) fn same_formula(&self, other: &SavedFormula) -> bool {
        let same_number = |a: &str, b: &str| match (a.trim().parse::<f64>(), b.trim().parse::<f64>()) {
            (Ok(x), Ok(y)) => x == y,
            _ => a.trim() == b.trim(),
        };
        let zero = ["0".to_string(), "0".to_string()];
        let n = self.params.len().max(other.params.len());
        self.source.trim() == other.source.trim()
            && (0..n).all(|i| {
                let (a, b) = (self.params.get(i).unwrap_or(&zero), other.params.get(i).unwrap_or(&zero));
                same_number(&a[0], &b[0]) && same_number(&a[1], &b[1])
            })
    }

    /// The source on one line for a list row: statements joined by the language's own `, `.
    pub(crate) fn one_line(&self) -> String {
        self.source.lines().map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join(", ")
    }
}

/// The first statement that is not a comment, for naming an unnamed entry.
fn first_statement(source: &str) -> &str {
    source
        .lines()
        .map(|l| l.split(';').next().unwrap_or("").trim())
        .find(|l| !l.is_empty())
        .unwrap_or("formula")
}

/// Alphabetical, ignoring case — the order the gradient library keeps.
pub(crate) fn sort(list: &mut [SavedFormula]) {
    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()).then_with(|| a.name.cmp(&b.name)));
}

/// Save `entry` under its name, REPLACING an entry of that name: saving twice under one name means
/// "update it" (as in the gradient library). Returns whether an entry was replaced, or `None` if
/// the entry is empty (see [`SavedFormula::tidy`]).
pub(crate) fn upsert(list: &mut Vec<SavedFormula>, entry: SavedFormula) -> Option<bool> {
    let entry = entry.tidy()?;
    let replaced = match list.iter().position(|f| f.name == entry.name) {
        Some(i) => {
            list[i] = entry;
            true
        }
        None => {
            list.push(entry);
            false
        }
    };
    sort(list);
    Some(replaced)
}

/// What an import did, for the sentence that reports it.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct MergeReport {
    pub(crate) added: usize,
    /// Formulas the library already holds (under any name): skipped.
    pub(crate) duplicates: usize,
    /// `(name in the file, name given)`: a DIFFERENT formula whose name was taken.
    pub(crate) renamed: Vec<(String, String)>,
    /// Empty or oversized sources: skipped.
    pub(crate) skipped: usize,
}

/// Add `incoming` to `list` without losing anything already there: a formula the library already
/// holds is skipped, and a new formula whose name is taken is added as "name (2)" rather than
/// replacing the entry of that name — an import must never overwrite the user's own work.
pub(crate) fn merge(list: &mut Vec<SavedFormula>, incoming: Vec<SavedFormula>) -> MergeReport {
    let mut report = MergeReport::default();
    for entry in incoming {
        let Some(entry) = entry.tidy() else {
            report.skipped += 1;
            continue;
        };
        if list.iter().any(|f| f.same_formula(&entry)) {
            report.duplicates += 1;
            continue;
        }
        let taken = |name: &str| list.iter().any(|f| f.name == name);
        let mut name = entry.name.clone();
        if taken(&name) {
            name = (2..).map(|k| format!("{} ({k})", entry.name)).find(|n| !taken(n)).expect("unbounded");
            report.renamed.push((entry.name.clone(), name.clone()));
        }
        list.push(SavedFormula { name, ..entry });
        report.added += 1;
    }
    sort(list);
    report
}

impl MergeReport {
    /// One sentence (or two) for the toast. `file` is the file's name.
    pub(crate) fn sentence(&self, file: &str) -> String {
        let plural = |n: usize, one: &str, many: &str| if n == 1 { format!("1 {one}") } else { format!("{n} {many}") };
        let mut s = if self.added == 0 {
            format!("Nothing new in \"{file}\"")
        } else {
            format!("Imported {} from \"{file}\"", plural(self.added, "formula", "formulas"))
        };
        let mut notes = Vec::new();
        if self.duplicates > 0 {
            notes.push(format!(
                "{} already in the library",
                plural(self.duplicates, "formula was", "formulas were")
            ));
        }
        match self.renamed.as_slice() {
            [] => {}
            [(from, to)] => notes.push(format!("\"{from}\" was taken, so it is \"{to}\"")),
            many => notes.push(format!("{} renamed where the name was taken", many.len())),
        }
        if self.skipped > 0 {
            notes.push(format!("{} empty or too long, skipped", plural(self.skipped, "entry was", "entries were")));
        }
        if notes.is_empty() {
            s.push('.');
        } else {
            s.push_str(": ");
            s.push_str(&notes.join("; "));
            s.push('.');
        }
        s
    }
}

/// The formulas in a formula file, or why it is not one.
pub(crate) fn parse_file(text: &str) -> Result<Vec<SavedFormula>, String> {
    let file: FormulaFile = toml::from_str(text).map_err(|e| {
        // The toml crate's message carries a source snippet over several lines; the first says it.
        let e = e.to_string();
        format!("not a formula file ({})", e.lines().next().unwrap_or("unreadable").trim())
    })?;
    if !file.format.is_empty() && file.format != FORMAT {
        return Err(format!("a \"{}\" file, not a formula file", file.format));
    }
    if file.formula.is_empty() {
        return Err("no formulas in it (a formula file lists them as [[formula]] tables)".to_string());
    }
    Ok(file.formula)
}

/// A formula file holding `list`.
pub(crate) fn file_text(list: &[SavedFormula]) -> String {
    let file = FormulaFile { format: FORMAT.to_string(), version: VERSION, formula: list.to_vec() };
    let body = toml::to_string_pretty(&file).expect("strings and arrays of strings always serialize");
    format!(
        "# Fractadyne custom formulas. Import them with Fractal > Formula library > Import.\n\
         # Each [[formula]] has a name, the step's source and p1, p2, ... as [re, im] text.\n\n{body}"
    )
}

/// `formulas.toml` in the config dir (which honours `FRACTADYNE_CONFIG_DIR`).
pub(crate) fn library_path() -> Option<PathBuf> {
    fractadyne_state::config_dir().map(|d| d.join("formulas.toml"))
}

/// The saved library, and a sentence for the user if the file exists but could not be used.
pub(crate) fn load() -> (Vec<SavedFormula>, Option<String>) {
    match library_path() {
        Some(p) => load_from(&p),
        None => (Vec::new(), None),
    }
}

/// [`load`] from `path`. A missing file is an empty library. A file that cannot be read or parsed
/// is MOVED ASIDE (`formulas.unreadable.toml`, or `…-2`…) so the next save cannot destroy it.
pub(crate) fn load_from(path: &Path) -> (Vec<SavedFormula>, Option<String>) {
    let problem = match std::fs::read_to_string(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return (Vec::new(), None),
        Err(e) => e.to_string(),
        Ok(text) => match toml::from_str::<FormulaFile>(&text) {
            Ok(file) => {
                let mut list: Vec<SavedFormula> = file.formula.into_iter().filter_map(SavedFormula::tidy).collect();
                sort(&mut list);
                return (list, None);
            }
            Err(e) => e.to_string().lines().next().unwrap_or("unreadable").trim().to_string(),
        },
    };
    let aside = (1..100)
        .map(|k| {
            let name = if k == 1 { "formulas.unreadable.toml".to_string() } else { format!("formulas.unreadable-{k}.toml") };
            path.with_file_name(name)
        })
        .find(|p| !p.exists());
    let moved = aside.as_ref().filter(|a| std::fs::rename(path, a).is_ok());
    let note = match moved {
        Some(a) => format!(
            "The formula library could not be read ({problem}). It was kept as {} and a new, empty library started.",
            a.display()
        ),
        None => format!(
            "The formula library could not be read ({problem}), nor moved aside: copy {} somewhere safe \
             before saving a formula, which replaces it.",
            path.display()
        ),
    };
    crate::diag::log_line("formula", &note);
    (Vec::new(), Some(note))
}

/// Write the library to [`library_path`].
pub(crate) fn save(list: &[SavedFormula]) -> Result<(), String> {
    let path = library_path().ok_or("no config directory")?;
    save_to(&path, list)
}

/// Write the library to `path`, atomically: a temp file beside it, then a rename over it, so a
/// crash mid-write leaves the old library, not half of the new one.
pub(crate) fn save_to(path: &Path, list: &[SavedFormula]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, file_text(list)).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        e.to_string()
    })
}

#[cfg(test)]
#[path = "formula_library_tests.rs"]
mod tests;
