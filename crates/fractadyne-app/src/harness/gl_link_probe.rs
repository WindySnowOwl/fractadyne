//! `--gl-link-bisect` and `--gl-link-probe FILE`: find what in the renderer's shader an OpenGL
//! driver's compiler crashes on.
//!
//! Eris (2026-11-09; Intel HD Graphics 4400, which has only OpenGL): the driver dies with an access
//! violation linking `fs_iterate`, the renderer's first pipeline, every time, on the newest driver
//! the card has too. A crash inside the driver cannot be caught, so the bisect never links in its
//! own process: each variant of the shader goes to a `--gl-link-probe` child, and only the child
//! dies.
//!
//! The variants come from the WGSL source, in two stages (hierarchical delta debugging):
//! 1. every helper function, kept or stubbed (`return T();`);
//! 2. in what is left, the blocks (an `if`'s body, a loop's, a `case`'s), level by level from the
//!    outermost, kept or emptied (`return T();` again, so a function still returns on every path).
//!
//! WGSL scopes a block's `let`s and `var`s to it, so an emptied block never leaves a name dangling.
//! A variant the compiler rejects anyway counts as unresolved, which delta debugging allows for. A
//! variant counts as crashing only when it dies where the full shader died (the faulting module and
//! offset in the child's crash report), so a different crash cannot steer the search.

use std::collections::HashMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// The child's flag: link the WGSL in FILE as `fs_iterate`'s pipeline, on OpenGL, and exit.
pub(crate) const PROBE_FLAG: &str = "--gl-link-probe";
/// The parent's flag: the whole search.
pub(crate) const BISECT_FLAG: &str = "--gl-link-bisect";
/// With `--gl-link-bisect`: the search's time bound, in minutes.
pub(crate) const MINUTES_FLAG: &str = "--gl-link-minutes";

/// The child's exit codes besides a crash.
const EXIT_LINKED: i32 = 0;
const EXIT_NO_FILE: i32 = 2;
const EXIT_REJECTED: i32 = 3;
const EXIT_NO_ADAPTER: i32 = 4;

/// `--gl-link-probe` or `--gl-link-bisect`, if `args` asks for one: its exit code.
pub(crate) fn run(args: &[String]) -> Option<i32> {
    let value = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned();
    if args.iter().any(|a| a == PROBE_FLAG) {
        let Some(file) = value(PROBE_FLAG).filter(|v| !v.starts_with('-')) else {
            eprintln!("fractadyne: {PROBE_FLAG} needs a WGSL file");
            return Some(EXIT_NO_FILE);
        };
        return Some(probe_child(Path::new(&file)));
    }
    if args.iter().any(|a| a == BISECT_FLAG) {
        let minutes = match value(MINUTES_FLAG) {
            None => 50,
            Some(v) => match v.parse::<u64>() {
                Ok(m) if m > 0 => m,
                _ => {
                    eprintln!("fractadyne: {MINUTES_FLAG} needs a whole number of minutes, not {v:?}");
                    return Some(2);
                }
            },
        };
        return Some(bisect(Duration::from_secs(minutes * 60)));
    }
    None
}

// ---- the child --------------------------------------------------------------------------------

/// Link `file`'s WGSL as the renderer's `fs_iterate` pipeline on OpenGL, as start-up does: the
/// product-fusion probe first (its verdict is a constant of the iterate pipeline), then the
/// pipeline. A driver that crashes compiling it crashes this process, and its crash report names
/// where.
fn probe_child(file: &Path) -> i32 {
    use eframe::wgpu;
    let wgsl = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("fractadyne: {PROBE_FLAG}: {}: {e}", file.display());
            return EXIT_NO_FILE;
        }
    };
    let t0 = Instant::now();
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor { backends: wgpu::Backends::GL, ..Default::default() });
    let Some(adapter) = crate::gpu_worker::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    })) else {
        println!("no OpenGL adapter");
        return EXIT_NO_ADAPTER;
    };
    let info = adapter.get_info();
    println!("adapter: {} ({:?}), driver {} {}", info.name, info.backend, info.driver, info.driver_info);
    // An OpenGL adapter asked for its own limits, as the app asks (the defaults are WebGL2's).
    let dev = crate::gpu_worker::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("gl-link-probe"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            memory_hints: wgpu::MemoryHints::default(),
        },
        None,
    ));
    let (device, queue) = match dev {
        Ok(d) => d,
        Err(e) => {
            println!("device request failed: {e}");
            return EXIT_NO_ADAPTER;
        }
    };
    // A driver may put its own crash handler in place of ours as it loads.
    crate::diag::native_crash::reinstall("the graphics driver's start-up");
    crate::diag::set_phase(crate::diag::advice::Phase::PreparingShaders);
    crate::diag::breadcrumb("gl-link-probe: the product-fusion probe".into());
    let fusion = fractadyne_gpu::detect_product_fusion(&device, &queue);
    println!("product fusion: {}", match &fusion {
        Ok(v) if v.barrier() => "fused (the iterate pipeline gets the product barrier)".to_string(),
        Ok(_) => "none".to_string(),
        Err(e) => format!("the probe did not run ({e})"),
    });
    crate::diag::breadcrumb("gl-link-probe: building fs_iterate's pipeline".into());
    device.push_error_scope(wgpu::ErrorFilter::Validation);
    fractadyne_gpu::link_probe::build_iterate_pipeline(&device, &wgsl);
    if let Some(e) = crate::gpu_worker::block_on(device.pop_error_scope()) {
        println!("rejected: {e}");
        return EXIT_REJECTED;
    }
    // Test hook: crash, as a driver would, when the variant still holds this text. It lets the
    // whole search run on a machine whose driver links the shader fine.
    if let Ok(text) = std::env::var("FRACTADYNE_GL_PROBE_FAKE_CRASH") {
        if !text.is_empty() && wgsl.contains(&text) {
            crate::diag::native_crash::raise_for_test(false, false);
        }
    }
    println!("linked in {:.1} s", t0.elapsed().as_secs_f64());
    EXIT_LINKED
}

// ---- the WGSL --------------------------------------------------------------------------------

/// One function of a WGSL source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Func {
    pub(crate) name: String,
    /// The return type, attributes removed (`None`: the function returns nothing).
    pub(crate) ret: Option<String>,
    /// The byte offsets of the body's `{` and its `}`.
    pub(crate) open: usize,
    pub(crate) close: usize,
    /// An entry point (`@vertex`, `@fragment`, `@compute`): never stubbed.
    pub(crate) entry: bool,
}

/// One block inside a function's body: the braces of an `if`, `else`, loop or `case` body, or a
/// bare block. A `switch`'s own braces are not one (a switch needs its cases), but its cases are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Block {
    pub(crate) open: usize,
    pub(crate) close: usize,
    /// The function it is in (an index into [`functions`]' list).
    pub(crate) func: usize,
    /// The innermost block it is in, if any.
    pub(crate) parent: Option<usize>,
}

/// `src` with its comments blanked to spaces, byte for byte (newlines kept), so offsets into it
/// are offsets into `src` and nothing in a comment reads as code. WGSL block comments nest.
pub(crate) fn blank_comments(src: &str) -> Vec<u8> {
    let mut b = src.as_bytes().to_vec();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'/' && b.get(i + 1) == Some(&b'/') {
            while i < b.len() && b[i] != b'\n' {
                b[i] = b' ';
                i += 1;
            }
        } else if b[i] == b'/' && b.get(i + 1) == Some(&b'*') {
            let mut depth = 0usize;
            while i < b.len() {
                if b[i] == b'/' && b.get(i + 1) == Some(&b'*') {
                    depth += 1;
                    b[i] = b' ';
                    b[i + 1] = b' ';
                    i += 2;
                } else if b[i] == b'*' && b.get(i + 1) == Some(&b'/') {
                    depth -= 1;
                    b[i] = b' ';
                    b[i + 1] = b' ';
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    if b[i] != b'\n' {
                        b[i] = b' ';
                    }
                    i += 1;
                }
            }
        } else {
            i += 1;
        }
    }
    b
}

fn ident_byte(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Is the word `w` at `i`, not part of a longer identifier?
fn word_at(b: &[u8], i: usize, w: &str) -> bool {
    let w = w.as_bytes();
    b.len() >= i + w.len()
        && &b[i..i + w.len()] == w
        && (i == 0 || !ident_byte(b[i - 1]))
        && b.get(i + w.len()).is_none_or(|&c| !ident_byte(c))
}

/// The offset of the bracket that closes the one at `open` (`(`/`)` or `{`/`}`).
fn matching(b: &[u8], open: usize) -> Option<usize> {
    let (o, c) = (b[open], if b[open] == b'(' { b')' } else { b'}' });
    let mut depth = 0usize;
    for (k, &x) in b.iter().enumerate().skip(open) {
        if x == o {
            depth += 1;
        } else if x == c {
            depth -= 1;
            if depth == 0 {
                return Some(k);
            }
        }
    }
    None
}

/// `T` from a return type written `@location(0) T` or `T`.
fn strip_attributes(t: &str) -> String {
    let mut s = t.trim();
    while let Some(rest) = s.strip_prefix('@') {
        let name_end = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(rest.len());
        let mut after = rest[name_end..].trim_start();
        if after.starts_with('(') {
            let close = after.find(')').map_or(after.len(), |k| k + 1);
            after = after[close..].trim_start();
        }
        s = after;
    }
    s.to_string()
}

/// The functions of `src`, in source order.
pub(crate) fn functions(src: &str) -> Vec<Func> {
    let b = blank_comments(src);
    let mut out = Vec::new();
    let mut depth = 0usize;
    // Where the current top-level declaration began (after the last `;` or `}` at depth 0).
    let mut decl_start = 0usize;
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    decl_start = i + 1;
                }
            }
            b';' if depth == 0 => decl_start = i + 1,
            _ if depth == 0 && word_at(&b, i, "fn") => {
                let parsed = (|| {
                    let mut j = i + 2;
                    while j < b.len() && b[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    let name_start = j;
                    while j < b.len() && ident_byte(b[j]) {
                        j += 1;
                    }
                    let name = src[name_start..j].to_string();
                    let paren = j + b[j..].iter().position(|&c| c == b'(')?;
                    let paren_close = matching(&b, paren)?;
                    let open = paren_close + b[paren_close..].iter().position(|&c| c == b'{')?;
                    let close = matching(&b, open)?;
                    let between = std::str::from_utf8(&b[paren_close + 1..open]).ok()?;
                    let ret = between.trim().strip_prefix("->").map(strip_attributes).filter(|t| !t.is_empty());
                    let head = std::str::from_utf8(&b[decl_start..i]).ok()?;
                    let entry = ["@vertex", "@fragment", "@compute"].iter().any(|a| head.contains(a));
                    Some(Func { name, ret, open, close, entry })
                })();
                match parsed {
                    Some(f) => {
                        i = f.close + 1;
                        decl_start = i;
                        out.push(f);
                        continue;
                    }
                    None => {}
                }
            }
            _ => {}
        }
        i += 1;
    }
    out
}

/// The blocks inside the bodies of `funcs` (all of them), in source order.
pub(crate) fn blocks(src: &str, funcs: &[Func]) -> Vec<Block> {
    let b = blank_comments(src);
    let mut out: Vec<Block> = Vec::new();
    for (fi, f) in funcs.iter().enumerate() {
        // The open braces, each a block's index or `None` for a switch's own braces.
        let mut stack: Vec<Option<usize>> = Vec::new();
        for k in f.open + 1..f.close {
            match b[k] {
                b'{' => {
                    // The statement's head: back to the last `;`, `{` or `}`.
                    let head_start = b[f.open..k].iter().rposition(|&c| matches!(c, b';' | b'{' | b'}')).map_or(f.open, |p| f.open + p + 1);
                    let head = &b[head_start..k];
                    let is_switch = (0..head.len()).any(|p| word_at(head, p, "switch"));
                    if is_switch {
                        stack.push(None);
                    } else {
                        let parent = stack.iter().rev().find_map(|s| *s);
                        out.push(Block { open: k, close: k, func: fi, parent });
                        stack.push(Some(out.len() - 1));
                    }
                }
                b'}' => {
                    if let Some(Some(bi)) = stack.pop() {
                        out[bi].close = k;
                    }
                }
                _ => {}
            }
        }
    }
    out
}

/// The names of the functions `f`'s body calls.
fn calls<'a>(src: &str, b: &[u8], f: &Func, names: &HashMap<&'a str, usize>) -> Vec<usize> {
    let mut out = Vec::new();
    let mut k = f.open;
    while k < f.close {
        if ident_byte(b[k]) && (k == 0 || !ident_byte(b[k - 1])) {
            let s = k;
            while k < f.close && ident_byte(b[k]) {
                k += 1;
            }
            let mut j = k;
            while j < f.close && b[j].is_ascii_whitespace() {
                j += 1;
            }
            if b.get(j) == Some(&b'(') {
                if let Some(&callee) = names.get(&src[s..k]) {
                    out.push(callee);
                }
            }
        } else {
            k += 1;
        }
    }
    out
}

/// Which of `funcs` the entry points named in `roots` reach.
pub(crate) fn reachable(src: &str, funcs: &[Func], roots: &[&str]) -> Vec<bool> {
    let b = blank_comments(src);
    let names: HashMap<&str, usize> = funcs.iter().enumerate().map(|(i, f)| (f.name.as_str(), i)).collect();
    let mut seen = vec![false; funcs.len()];
    let mut todo: Vec<usize> = roots.iter().filter_map(|r| names.get(r).copied()).collect();
    while let Some(i) = todo.pop() {
        if std::mem::replace(&mut seen[i], true) {
            continue;
        }
        todo.extend(calls(src, &b, &funcs[i], &names));
    }
    seen
}

/// What a stubbed function's body or an emptied block becomes: a return of the zero value, so
/// every path still returns.
fn stub_body(ret: Option<&str>) -> String {
    match ret {
        Some(t) => format!("{{ return {t}(); }}"),
        None => "{ return; }".to_string(),
    }
}

/// One variant of the source: which functions are stubbed and which blocks emptied.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Variant {
    pub(crate) stubbed: Vec<bool>,
    pub(crate) emptied: Vec<bool>,
}

/// `src` as `v` changes it. A block inside a stubbed function or an emptied block is gone with it.
pub(crate) fn render(src: &str, funcs: &[Func], blocks: &[Block], v: &Variant) -> String {
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    for (i, f) in funcs.iter().enumerate() {
        if v.stubbed.get(i).copied().unwrap_or(false) && !f.entry {
            edits.push((f.open, f.close, stub_body(f.ret.as_deref())));
        }
    }
    for (i, bl) in blocks.iter().enumerate() {
        if v.emptied.get(i).copied().unwrap_or(false) {
            edits.push((bl.open, bl.close, stub_body(funcs[bl.func].ret.as_deref())));
        }
    }
    edits.sort_by_key(|e| e.0);
    let mut out = String::with_capacity(src.len());
    let mut at = 0usize;
    for (open, close, with) in edits {
        if open < at {
            continue; // inside an edit already made
        }
        out.push_str(&src[at..open]);
        out.push_str(&with);
        at = close + 1;
    }
    out.push_str(&src[at..]);
    out
}

// ---- delta debugging -----------------------------------------------------------------------------

/// What one test of a variant said.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    /// It fails as the original does (the same crash).
    Fail,
    /// It links.
    Pass,
    /// Neither: rejected, a different crash, too slow.
    Unresolved,
    /// The search is out of time: stop where it is.
    Stop,
}

/// Zeller's ddmin: a 1-minimal subset of `items` that still fails, given that all of `items` does.
/// `test` gets the items KEPT. Returns the subset and whether the search stopped early (then the
/// subset still fails, but may not be minimal).
pub(crate) fn ddmin(items: &[usize], test: &mut dyn FnMut(&[usize]) -> Verdict) -> (Vec<usize>, bool) {
    let mut c: Vec<usize> = items.to_vec();
    if c.is_empty() {
        return (c, false);
    }
    match test(&[]) {
        Verdict::Fail => return (Vec::new(), false),
        Verdict::Stop => return (c, true),
        _ => {}
    }
    let mut n = 2usize;
    while c.len() >= 2 {
        let n_eff = n.min(c.len());
        let parts: Vec<Vec<usize>> = (0..n_eff)
            .map(|k| c[k * c.len() / n_eff..(k + 1) * c.len() / n_eff].to_vec())
            .collect();
        let mut next: Option<(Vec<usize>, usize)> = None;
        for p in &parts {
            match test(p) {
                Verdict::Fail => {
                    next = Some((p.clone(), 2));
                    break;
                }
                Verdict::Stop => return (c, true),
                _ => {}
            }
        }
        if next.is_none() && n_eff > 2 {
            for k in 0..parts.len() {
                let comp: Vec<usize> = parts.iter().enumerate().filter(|(j, _)| *j != k).flat_map(|(_, p)| p.iter().copied()).collect();
                match test(&comp) {
                    Verdict::Fail => {
                        next = Some((comp, (n_eff - 1).max(2)));
                        break;
                    }
                    Verdict::Stop => return (c, true),
                    _ => {}
                }
            }
        }
        match next {
            Some((nc, nn)) => {
                c = nc;
                n = nn;
            }
            None => {
                if n_eff >= c.len() {
                    break;
                }
                n = (2 * n_eff).min(c.len());
            }
        }
    }
    (c, false)
}

/// The faulting place in a crash report's `at` line (`<native: ig75icd64.dll+0x73489a>`), or its
/// whole `at` value for any other crash.
pub(crate) fn crash_signature(report: &str) -> Option<String> {
    let at = report.lines().find_map(|l| l.strip_prefix("at")?.trim_start().strip_prefix(':').map(str::trim))?;
    Some(match at.strip_prefix("<native:").and_then(|s| s.strip_suffix('>')) {
        Some(place) => place.trim().to_string(),
        None => at.to_string(),
    })
}

// ---- the parent ---------------------------------------------------------------------------------

/// What linking one variant did.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Outcome {
    Linked,
    Rejected(String),
    Crashed(String),
    TimedOut,
    NotRun(String),
}

impl Outcome {
    fn describe(&self) -> String {
        match self {
            Outcome::Linked => "linked".into(),
            Outcome::Rejected(m) => format!("rejected: {m}"),
            Outcome::Crashed(s) => format!("crashed at {s}"),
            Outcome::TimedOut => "timed out".into(),
            Outcome::NotRun(m) => format!("not run: {m}"),
        }
    }
}

struct Search {
    exe: PathBuf,
    dir: PathBuf,
    report: Option<std::fs::File>,
    deadline: Instant,
    probes: u32,
    /// The original's crash: what a variant must repeat to count.
    signature: Option<String>,
    cache: HashMap<u64, Outcome>,
}

impl Search {
    fn line(&mut self, s: &str) {
        println!("{s}");
        let _ = std::io::stdout().flush();
        if let Some(f) = self.report.as_mut() {
            let _ = writeln!(f, "{s}");
        }
    }

    /// Link `wgsl` in a child. `keep` names a folder to keep the child's logs and output in.
    fn probe(&mut self, what: &str, wgsl: &str, keep: Option<&str>, env: &[(&str, &str)]) -> Outcome {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        wgsl.hash(&mut h);
        let key = h.finish();
        if keep.is_none() {
            if let Some(o) = self.cache.get(&key) {
                return o.clone();
            }
        }
        let left = self.deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Outcome::NotRun("out of time".into());
        }
        self.probes += 1;
        let n = self.probes;
        let name = keep.map_or_else(|| format!("probe-{n:03}"), str::to_string);
        let child_dir = self.dir.join(&name);
        let _ = std::fs::create_dir_all(&child_dir);
        let file = child_dir.join("variant.wgsl");
        if let Err(e) = std::fs::write(&file, wgsl) {
            return Outcome::NotRun(format!("could not write {}: {e}", file.display()));
        }
        let started = Instant::now();
        let mut cmd = Command::new(&self.exe);
        cmd.arg(PROBE_FLAG)
            .arg(&file)
            .arg("--log-dir")
            .arg(&child_dir)
            .env("FRACTADYNE_NO_SOUND", "1")
            .env("FRACTADYNE_NO_CRASH_DIALOG", "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (k, v) in env {
            cmd.env(k, v);
        }
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => return Outcome::NotRun(format!("could not start a probe: {e}")),
        };
        // Drained on their own threads: a child blocked on a full pipe never exits.
        let drain = |p: Option<Box<dyn std::io::Read + Send>>| {
            p.map(|mut p| {
                std::thread::spawn(move || {
                    let mut s = Vec::new();
                    let _ = p.read_to_end(&mut s);
                    String::from_utf8_lossy(&s).into_owned()
                })
            })
        };
        let out_h = drain(child.stdout.take().map(|p| Box::new(p) as Box<dyn std::io::Read + Send>));
        let err_h = drain(child.stderr.take().map(|p| Box::new(p) as Box<dyn std::io::Read + Send>));
        // One link may take minutes on a slow card; never past the search's own end.
        let limit = left.min(Duration::from_secs(600));
        let status = loop {
            match child.try_wait() {
                Ok(Some(s)) => break Some(s),
                Ok(None) if started.elapsed() > limit => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(100)),
                Err(_) => break None,
            }
        };
        let secs = started.elapsed().as_secs_f64();
        let stdout = out_h.and_then(|h| h.join().ok()).unwrap_or_default();
        let stderr = err_h.and_then(|h| h.join().ok()).unwrap_or_default();
        let outcome = match status.and_then(|s| s.code()) {
            None => Outcome::TimedOut,
            Some(EXIT_LINKED) => Outcome::Linked,
            Some(EXIT_REJECTED) => Outcome::Rejected(
                stdout.lines().find_map(|l| l.strip_prefix("rejected: ")).unwrap_or("").chars().take(300).collect(),
            ),
            Some(code) => {
                let report = std::fs::read_dir(&child_dir)
                    .ok()
                    .into_iter()
                    .flatten()
                    .flatten()
                    .map(|e| e.path())
                    .find(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with("crash-") && n.ends_with(".txt")))
                    .and_then(|p| std::fs::read_to_string(p).ok());
                let sig = report.as_deref().and_then(crash_signature).unwrap_or_else(|| format!("exit 0x{:08x}", code as u32));
                Outcome::Crashed(sig)
            }
        };
        self.line(&format!("{name:>10}  {what}: {} ({secs:.1} s)", outcome.describe()));
        if keep.is_some() {
            let _ = std::fs::write(child_dir.join("output.txt"), format!("{stdout}\n--- stderr ---\n{stderr}"));
        } else {
            let _ = std::fs::remove_dir_all(&child_dir);
            self.cache.insert(key, outcome.clone());
        }
        outcome
    }

    fn verdict(&self, o: &Outcome) -> Verdict {
        match o {
            Outcome::Crashed(s) if Some(s) == self.signature.as_ref() => Verdict::Fail,
            Outcome::Linked => Verdict::Pass,
            Outcome::NotRun(m) if m == "out of time" => Verdict::Stop,
            _ => Verdict::Unresolved,
        }
    }
}

/// The 1-based line of byte offset `at` in `src`.
fn line_of(src: &str, at: usize) -> usize {
    src.as_bytes()[..at.min(src.len())].iter().filter(|&&c| c == b'\n').count() + 1
}

/// A block's head as written (`if (x > 0.0)`, `case 3u:`), for the report.
fn head_of(src: &str, bl: &Block, funcs: &[Func]) -> String {
    let b = blank_comments(src);
    let f = &funcs[bl.func];
    let start = b[f.open..bl.open].iter().rposition(|&c| matches!(c, b';' | b'{' | b'}')).map_or(f.open, |p| f.open + p + 1);
    let head: String = String::from_utf8_lossy(&b[start..bl.open]).split_whitespace().collect::<Vec<_>>().join(" ");
    if head.is_empty() { "{ (a bare block)".into() } else { head.chars().take(90).collect() }
}

fn bisect(budget: Duration) -> i32 {
    let t0 = Instant::now();
    let src = fractadyne_gpu::link_probe::renderer_wgsl();
    let Ok(exe) = std::env::current_exe() else {
        eprintln!("fractadyne: {BISECT_FLAG}: cannot find this program's own path");
        return 1;
    };
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let base = crate::diag::logs_dir().unwrap_or_else(|| PathBuf::from("logs"));
    let dir = base.join("gl-link-probe").join(stamp.to_string());
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("fractadyne: {BISECT_FLAG}: cannot create {}: {e}", dir.display());
        return 1;
    }
    let mut s = Search {
        exe,
        report: std::fs::File::create(dir.join("report.txt")).ok(),
        dir: dir.clone(),
        deadline: t0 + budget,
        probes: 0,
        signature: None,
        cache: HashMap::new(),
    };
    let funcs = functions(src);
    let blks = blocks(src, &funcs);
    let live = reachable(src, &funcs, &["vs_main", "fs_iterate"]);
    let candidates: Vec<usize> = (0..funcs.len()).filter(|&i| !funcs[i].entry).collect();
    s.line(&format!(
        "{BISECT_FLAG} ({}): the renderer's shader has {} functions ({} entry points; {} reached from vs_main + fs_iterate) and {} blocks. Results in {}. Time bound {} min.",
        crate::version_string(),
        funcs.len(),
        funcs.len() - candidates.len(),
        live.iter().filter(|&&r| r).count(),
        blks.len(),
        dir.display(),
        budget.as_secs() / 60
    ));

    // The controls. The original must crash, and its crash is the one every variant must repeat.
    let original = s.probe("the shader as shipped", src, Some("control-original"), &[]);
    match &original {
        Outcome::Crashed(sig) => s.signature = Some(sig.clone()),
        other => {
            s.line(&format!(
                "The shader as shipped did not crash in a probe ({}): nothing to bisect. On this machine the driver {}.",
                other.describe(),
                if *other == Outcome::Linked { "links it" } else { "does something else with it" }
            ));
            return 1;
        }
    }
    let n_funcs = funcs.len();
    let n_blocks = blks.len();
    let all_stubbed = Variant { stubbed: vec![true; n_funcs], emptied: vec![false; n_blocks] };
    let floor = s.probe("every helper function stubbed", &render(src, &funcs, &blks, &all_stubbed), Some("control-all-stubbed"), &[]);
    s.line(&format!("Control: every helper stubbed {} (a variant counts when it crashes at {}).", floor.describe(), s.signature.as_deref().unwrap_or("?")));

    // Stage 1: the functions.
    let mut stopped = false;
    let kept_funcs: Vec<usize> = {
        let mut test = |keep: &[usize]| {
            let mut v = Variant { stubbed: vec![true; n_funcs], emptied: vec![false; n_blocks] };
            for &k in keep {
                v.stubbed[k] = false;
            }
            let what = format!("{} of {} helpers kept", keep.len(), candidates.len());
            let o = s.probe(&what, &render(src, &funcs, &blks, &v), None, &[]);
            s.verdict(&o)
        };
        let (kept, stop) = ddmin(&candidates, &mut test);
        stopped |= stop;
        kept
    };
    let names: Vec<&str> = kept_funcs.iter().map(|&i| funcs[i].name.as_str()).collect();
    s.line(&format!("Stage 1: {} helper(s) needed for the crash: {}{}", names.len(), names.join(", "), if stopped { " (stopped early: not minimal)" } else { "" }));

    // Stage 2: the blocks in what is left, outermost first.
    let mut stubbed = vec![true; n_funcs];
    for &k in &kept_funcs {
        stubbed[k] = false;
    }
    let in_play = |bi: usize| funcs[blks[bi].func].entry || !stubbed[blks[bi].func];
    let mut emptied = vec![false; n_blocks];
    let mut frontier: Vec<usize> = (0..n_blocks).filter(|&i| blks[i].parent.is_none() && in_play(i)).collect();
    let mut level = 1;
    while !frontier.is_empty() && !stopped {
        let (kept, stop) = {
            let base = emptied.clone();
            let front = frontier.clone();
            let mut test = |keep: &[usize]| {
                let mut v = Variant { stubbed: stubbed.clone(), emptied: base.clone() };
                for &b in &front {
                    v.emptied[b] = !keep.contains(&b);
                }
                let what = format!("level {level}: {} of {} blocks kept", keep.len(), front.len());
                let o = s.probe(&what, &render(src, &funcs, &blks, &v), None, &[]);
                s.verdict(&o)
            };
            ddmin(&frontier, &mut test)
        };
        stopped |= stop;
        for &b in &frontier {
            emptied[b] = !kept.contains(&b);
        }
        s.line(&format!("Stage 2, level {level}: {} of {} blocks needed", kept.len(), frontier.len()));
        frontier = (0..n_blocks).filter(|&i| blks[i].parent.is_some_and(|p| kept.contains(&p))).collect();
        level += 1;
    }

    // The result: the smallest variant that still crashes, its GLSL, and what it keeps.
    let minimal = Variant { stubbed: stubbed.clone(), emptied: emptied.clone() };
    let text = render(src, &funcs, &blks, &minimal);
    let _ = std::fs::write(dir.join("minimal.wgsl"), &text);
    let check = s.probe("the minimal variant, with its GLSL logged", &text, Some("minimal"), &[("RUST_LOG", "wgpu_hal=debug,wgpu_core=warn")]);
    let mut kept_lines = Vec::new();
    for &f in &kept_funcs {
        kept_lines.push(format!("fn {} (line {})", funcs[f].name, line_of(src, funcs[f].open)));
    }
    // The blocks that are kept and have no kept block inside: the innermost places it needs.
    let alive = |i: usize| in_play(i) && !emptied[i] && {
        let mut p = blks[i].parent;
        let mut ok = true;
        while let Some(q) = p {
            if emptied[q] {
                ok = false;
            }
            p = blks[q].parent;
        }
        ok
    };
    for i in 0..n_blocks {
        let has_kept_child = (0..n_blocks).any(|j| blks[j].parent == Some(i) && alive(j));
        if alive(i) && !has_kept_child && level > 1 {
            kept_lines.push(format!("block at line {} in {}: {}", line_of(src, blks[i].open), funcs[blks[i].func].name, head_of(src, &blks[i], &funcs)));
        }
    }
    s.line(&format!(
        "Result after {} probes in {:.1} min{}: the minimal variant {} ({} bytes of {}). Kept:\n  {}",
        s.probes,
        t0.elapsed().as_secs_f64() / 60.0,
        if stopped { ", STOPPED at the time bound" } else { "" },
        check.describe(),
        text.len(),
        src.len(),
        kept_lines.join("\n  ")
    ));
    if s.verdict(&check) == Verdict::Fail { 0 } else { 1 }
}

#[cfg(test)]
#[path = "../../tests/unit/harness/gl_link_probe/gl_link_probe_tests.rs"]
mod gl_link_probe_tests;
