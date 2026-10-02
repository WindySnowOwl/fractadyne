//! Pattern files (design/automata.md §4.3): run-length encoded (RLE, LifeWiki's and Golly's format,
//! multi-state letters included), plaintext `.cells`, and Life 1.05 / 1.06. Read from their
//! published descriptions; written as RLE, plaintext or Life 1.06.
//!
//! A file is untrusted input: every count is checked, and a pattern past [`MAX_CELLS`] live cells or
//! outside the plane's extent is refused rather than allocated.

use super::universe::LIMIT;
use std::fmt;

/// The most live cells a pattern file may hold (larger universes are a Hashlife matter, phase 3).
pub const MAX_CELLS: usize = 1 << 24;

/// A pattern read from a file: its live cells relative to the file's origin (y down, as files
/// read), and what the file says about itself.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pattern {
    /// `(x, y, state)` for every cell not dead (state ≥ 1), in row order.
    pub cells: Vec<(i64, i64, u8)>,
    /// The rule the file names, as written (`rule = B3/S23`, `#R 23/3`, …).
    pub rule: Option<String>,
    /// `#N` (RLE) or `!Name:` (plaintext).
    pub name: Option<String>,
    /// Comment lines, without their markers.
    pub comments: Vec<String>,
    /// The generation the file records (`#CXRLE Gen=`).
    pub generation: Option<u64>,
}

/// The formats [`parse_pattern`] recognises.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatternFormat {
    Rle,
    Plaintext,
    Life105,
    Life106,
}

/// Why a pattern file was refused: what and where.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for PatternError {}

fn fail<T>(line: usize, message: impl Into<String>) -> Result<T, PatternError> {
    Err(PatternError { line, message: message.into() })
}

impl Pattern {
    /// The smallest rectangle `(x0, y0, x1, y1)` (inclusive) holding every cell.
    pub fn bounding_box(&self) -> Option<(i64, i64, i64, i64)> {
        self.cells.iter().fold(None, |bb, &(x, y, _)| {
            Some(match bb {
                None => (x, y, x, y),
                Some((a, b, c, d)) => (a.min(x), b.min(y), c.max(x), d.max(y)),
            })
        })
    }

    fn push(&mut self, x: i64, y: i64, state: u8, line: usize) -> Result<(), PatternError> {
        if state == 0 {
            return Ok(());
        }
        if x.abs() >= LIMIT || y.abs() >= LIMIT {
            return fail(line, "a cell lies outside the universe's extent");
        }
        if self.cells.len() >= MAX_CELLS {
            return fail(line, format!("more than {MAX_CELLS} live cells"));
        }
        self.cells.push((x, y, state));
        Ok(())
    }

    fn finish(mut self) -> Pattern {
        self.cells.sort_unstable_by_key(|&(x, y, _)| (y, x));
        self.cells.dedup_by_key(|&mut (x, y, _)| (x, y));
        self
    }
}

/// Read a pattern in whichever format the text is in.
pub fn parse_pattern(text: &str) -> Result<(Pattern, PatternFormat), PatternError> {
    let format = sniff(text);
    let p = match format {
        PatternFormat::Rle => parse_rle(text)?,
        PatternFormat::Plaintext => parse_plaintext(text)?,
        PatternFormat::Life105 => parse_life105(text)?,
        PatternFormat::Life106 => parse_life106(text)?,
    };
    Ok((p, format))
}

fn sniff(text: &str) -> PatternFormat {
    let first = text.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("");
    if first.starts_with("#Life 1.05") {
        return PatternFormat::Life105;
    }
    if first.starts_with("#Life 1.06") {
        return PatternFormat::Life106;
    }
    let body = || text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with('!'));
    let rle_header = body().any(|l| l.starts_with('x') && l.contains('='));
    let rle_body = body().any(|l| l.contains('$') || l.ends_with('!') || l.chars().any(|c| c.is_ascii_digit()));
    if rle_header || rle_body {
        PatternFormat::Rle
    } else {
        PatternFormat::Plaintext
    }
}

/// Run-length encoded: `#` comment lines (`#N` name, `#C`/`#c`/`#O` comments, `#r` rule, `#R`/`#P`
/// position, `#CXRLE Pos=x,y Gen=g`), a header `x = m, y = n, rule = …` (optional), then runs of
/// `b`/`.` (dead), `o` (alive), `A`–`X` and `pA`–`yO` (states 1–255), `$` (end of row), ending at `!`.
pub fn parse_rle(text: &str) -> Result<Pattern, PatternError> {
    let mut p = Pattern::default();
    let (mut ox, mut oy) = (0i64, 0i64);
    let mut body = String::new();
    let mut body_lines: Vec<(usize, usize)> = Vec::new(); // (byte offset in `body`, file line)
    let mut seen_header = false;
    for (n, raw) in text.lines().enumerate() {
        let line_no = n + 1;
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix('#') {
            if !body.is_empty() {
                continue; // comments after the body are ignored
            }
            let (tag, val) = rest.split_at(rest.chars().next().map_or(0, char::len_utf8));
            let val = val.trim();
            match tag {
                "N" => p.name = Some(val.to_string()),
                "C" | "c" if val.starts_with("XRLE") => {
                    for field in val["XRLE".len()..].split_whitespace() {
                        if let Some(pos) = field.strip_prefix("Pos=") {
                            let (x, y) = parse_pair(pos, ',', line_no)?;
                            (ox, oy) = (x, y);
                        } else if let Some(g) = field.strip_prefix("Gen=") {
                            p.generation = g.parse().ok();
                        }
                    }
                }
                "C" | "c" | "O" => p.comments.push(val.to_string()),
                "r" => p.rule = Some(val.to_string()),
                "R" | "P" => (ox, oy) = parse_pair(val, ' ', line_no)?,
                _ => p.comments.push(rest.to_string()),
            }
            continue;
        }
        if !seen_header && body.is_empty() && line.starts_with('x') && line.contains('=') {
            seen_header = true;
            for field in line.split(',') {
                let Some((k, v)) = field.split_once('=') else {
                    return fail(line_no, format!("'{}' in the header is not key = value", field.trim()));
                };
                match k.trim() {
                    "rule" => p.rule = Some(v.trim().to_string()),
                    "x" | "y" => {
                        v.trim().parse::<u64>().or_else(|_| fail(line_no, format!("'{}' is not a size", v.trim())))?;
                    }
                    _ => {} // other keys (Golly's `rule = …:T` extras live in the rule) are ignored
                }
            }
            continue;
        }
        body_lines.push((body.len(), line_no));
        body.push_str(line);
        if line.contains('!') {
            break;
        }
    }
    let line_at = |off: usize| body_lines.iter().rev().find(|&&(o, _)| o <= off).map_or(1, |&(_, l)| l);
    let bytes = body.as_bytes();
    let (mut x, mut y) = (ox, oy);
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        let mut count: u64 = 0;
        let mut has_count = false;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            count = match count.checked_mul(10).and_then(|c| c.checked_add(u64::from(bytes[i] - b'0'))) {
                Some(c) if c < LIMIT as u64 => c,
                _ => return fail(line_at(start), "a run count is too large"),
            };
            has_count = true;
            i += 1;
        }
        let run = if has_count { count as i64 } else { 1 };
        let Some(&c) = bytes.get(i) else {
            return fail(line_at(start), "the pattern ends inside a run (no '!')");
        };
        i += 1;
        let state: u8 = match c {
            b'!' => break,
            b'$' => {
                y += run;
                x = ox;
                if y >= LIMIT {
                    return fail(line_at(start), "the pattern runs outside the universe's extent");
                }
                continue;
            }
            b'b' | b'.' => 0,
            b'o' => 1,
            b'A'..=b'X' => c - b'A' + 1,
            b'p'..=b'y' => match bytes.get(i) {
                Some(&d @ b'A'..=b'X') => {
                    i += 1;
                    let v = 24 * u32::from(c - b'p' + 1) + u32::from(d - b'A' + 1);
                    if v > 255 {
                        return fail(line_at(start), format!("state {v} is past 255"));
                    }
                    v as u8
                }
                _ => return fail(line_at(start), format!("'{}' must be followed by A-X", char::from(c))),
            },
            other => {
                return fail(line_at(start), format!("'{}' is not an RLE run (b, o, $, ! or a state letter)", char::from(other)))
            }
        };
        if state != 0 {
            if p.cells.len() as u64 + run as u64 > MAX_CELLS as u64 {
                return fail(line_at(start), format!("more than {MAX_CELLS} live cells"));
            }
            for k in 0..run {
                p.push(x + k, y, state, line_at(start))?;
            }
        }
        x += run;
        if x.abs() >= LIMIT || y.abs() >= LIMIT {
            return fail(line_at(start), "the pattern runs outside the universe's extent");
        }
    }
    Ok(p.finish())
}

fn parse_pair(s: &str, sep: char, line: usize) -> Result<(i64, i64), PatternError> {
    let mut it = s.split(sep).map(str::trim).filter(|t| !t.is_empty());
    let mut num = || -> Result<i64, PatternError> {
        match it.next().map(str::parse::<i64>) {
            Some(Ok(v)) if v.abs() < LIMIT => Ok(v),
            _ => fail(line, format!("'{s}' is not a position")),
        }
    };
    Ok((num()?, num()?))
}

/// Plaintext (`.cells`): `!` comment lines (`!Name: …` names it), then one row per line, `.` dead and
/// `O` (or `*`) alive.
pub fn parse_plaintext(text: &str) -> Result<Pattern, PatternError> {
    let mut p = Pattern::default();
    let mut y = 0i64;
    for (n, raw) in text.lines().enumerate() {
        let line = raw.trim_end();
        if let Some(c) = line.strip_prefix('!') {
            match c.strip_prefix("Name:") {
                Some(name) => p.name = Some(name.trim().to_string()),
                None => p.comments.push(c.trim().to_string()),
            }
            continue;
        }
        for (x, ch) in line.chars().enumerate() {
            match ch {
                '.' => {}
                'O' | 'o' | '*' => p.push(x as i64, y, 1, n + 1)?,
                _ => return fail(n + 1, format!("'{ch}' in a plaintext row: expected '.' or 'O'")),
            }
        }
        y += 1;
    }
    Ok(p.finish())
}

/// Life 1.05: `#Life 1.05`, `#D` descriptions, `#N` (Conway's rules) or `#R s/b`, and `#P x y` blocks
/// of `.`/`*` rows.
pub fn parse_life105(text: &str) -> Result<Pattern, PatternError> {
    let mut p = Pattern::default();
    let (mut bx, mut y) = (0i64, 0i64);
    for (n, raw) in text.lines().enumerate().skip(1) {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix('#') {
            let (tag, val) = rest.split_at(rest.chars().next().map_or(0, char::len_utf8));
            match tag {
                "D" | "C" => p.comments.push(val.trim().to_string()),
                "N" => p.rule = Some("B3/S23".into()),
                "R" => p.rule = Some(val.trim().to_string()),
                "P" => (bx, y) = parse_pair(val, ' ', n + 1)?,
                _ => {}
            }
            continue;
        }
        for (x, ch) in line.chars().enumerate() {
            match ch {
                '.' => {}
                '*' | 'O' => p.push(bx + x as i64, y, 1, n + 1)?,
                _ => return fail(n + 1, format!("'{ch}' in a Life 1.05 row: expected '.' or '*'")),
            }
        }
        y += 1;
    }
    Ok(p.finish())
}

/// Life 1.06: `#Life 1.06`, then one `x y` pair per live cell.
pub fn parse_life106(text: &str) -> Result<Pattern, PatternError> {
    let mut p = Pattern::default();
    for (n, raw) in text.lines().enumerate().skip(1) {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (x, y) = parse_pair(line, ' ', n + 1)?;
        p.push(x, y, 1, n + 1)?;
    }
    Ok(p.finish())
}

/// RLE for `cells` (`(x, y, state)`, state ≥ 1): `#N` when named, a `#CXRLE` line with the top-left
/// position and the generation, the header with the rule, and runs wrapped at 70 columns. States
/// above 1, or a rule with more than two, use the multi-state letters.
pub fn write_rle(cells: &[(i64, i64, u8)], rule: &str, multistate: bool, name: Option<&str>, generation: Option<u64>) -> String {
    let mut sorted: Vec<(i64, i64, u8)> = cells.iter().copied().filter(|c| c.2 != 0).collect();
    sorted.sort_unstable_by_key(|&(x, y, _)| (y, x));
    let multistate = multistate || sorted.iter().any(|c| c.2 > 1);
    let mut out = String::new();
    if let Some(n) = name {
        out.push_str(&format!("#N {n}\n"));
    }
    let Some((x0, y0, x1, y1)) = Pattern { cells: sorted.clone(), ..Pattern::default() }.bounding_box() else {
        out.push_str(&format!("x = 0, y = 0, rule = {rule}\n!\n"));
        return out;
    };
    out.push_str(&format!("#CXRLE Pos={x0},{y0}"));
    if let Some(g) = generation {
        out.push_str(&format!(" Gen={g}"));
    }
    out.push('\n');
    out.push_str(&format!("x = {}, y = {}, rule = {rule}\n", x1 - x0 + 1, y1 - y0 + 1));

    let tag = |s: u8| -> String {
        match (multistate, s) {
            (false, 0) => "b".into(),
            (false, _) => "o".into(),
            (true, 0) => ".".into(),
            (true, s) if s <= 24 => char::from(b'A' + s - 1).to_string(),
            (true, s) => {
                let (hi, lo) = ((s - 1) / 24, (s - 1) % 24);
                format!("{}{}", char::from(b'p' + hi - 1), char::from(b'A' + lo))
            }
        }
    };
    let mut tokens: Vec<String> = Vec::new();
    let mut emit = |run: i64, t: &str| {
        tokens.push(if run == 1 { t.to_string() } else { format!("{run}{t}") });
    };
    let (mut x, mut y) = (x0, y0);
    let mut i = 0;
    while i < sorted.len() {
        let (cx, cy, s) = sorted[i];
        if cy > y {
            emit(cy - y, "$");
            (x, y) = (x0, cy);
        }
        if cx > x {
            emit(cx - x, &tag(0));
        }
        let mut run = 1;
        while i + run < sorted.len() && sorted[i + run] == (cx + run as i64, cy, s) {
            run += 1;
        }
        emit(run as i64, &tag(s));
        x = cx + run as i64;
        i += run;
    }
    tokens.push("!".into());
    let mut line = String::new();
    for t in tokens {
        if line.len() + t.len() > 70 {
            out.push_str(&line);
            out.push('\n');
            line.clear();
        }
        line.push_str(&t);
    }
    out.push_str(&line);
    out.push('\n');
    out
}

/// Plaintext (`.cells`) for a binary pattern, from its bounding box's top-left.
pub fn write_plaintext(cells: &[(i64, i64, u8)], name: Option<&str>) -> String {
    let p = Pattern { cells: cells.iter().copied().filter(|c| c.2 != 0).collect(), ..Pattern::default() }.finish();
    let mut out = String::new();
    if let Some(n) = name {
        out.push_str(&format!("!Name: {n}\n"));
    }
    let Some((x0, y0, x1, y1)) = p.bounding_box() else { return out };
    let mut rows = vec![vec![b'.'; (x1 - x0 + 1) as usize]; (y1 - y0 + 1) as usize];
    for &(x, y, _) in &p.cells {
        rows[(y - y0) as usize][(x - x0) as usize] = b'O';
    }
    for r in rows {
        let end = r.iter().rposition(|&c| c == b'O').map_or(0, |i| i + 1);
        out.push_str(std::str::from_utf8(&r[..end]).expect("ASCII"));
        out.push('\n');
    }
    out
}

/// Life 1.06 for a binary pattern: absolute coordinates, one cell a line.
pub fn write_life106(cells: &[(i64, i64, u8)]) -> String {
    let mut out = String::from("#Life 1.06\n");
    for &(x, y, s) in cells {
        if s != 0 {
            out.push_str(&format!("{x} {y}\n"));
        }
    }
    out
}

#[cfg(test)]
mod tests;
