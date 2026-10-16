use super::*;

fn entry(name: &str, source: &str, params: &[(&str, &str)]) -> SavedFormula {
    SavedFormula {
        name: name.into(),
        source: source.into(),
        params: params.iter().map(|(re, im)| [re.to_string(), im.to_string()]).collect(),
        ..Default::default()
    }
}

/// A directory of its own under the temp dir, removed when dropped.
struct Scratch(PathBuf);
impl Scratch {
    fn new(tag: &str) -> Scratch {
        let dir = std::env::temp_dir().join(format!("fd-formula-lib-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn tidy_repairs_line_breaks_and_names_rather_than_rejecting() {
    let e = entry("  Two\nlines  ", "t = sqr(z)\r\nz = t + c\rz = z\u{0}", &[(" 0.5 ", "0")]).tidy().unwrap();
    assert_eq!(e.source, "t = sqr(z)\nz = t + c\nz = z", "CR LF and a lone CR both become \\n; NUL dropped");
    assert_eq!(e.name, "Two lines");
    assert_eq!(e.params, vec![["0.5".to_string(), "0".to_string()]]);
    // A blank name is taken from the first statement that is not a comment.
    let e = entry("", "; my cubic\nz = z^3 + c", &[]).tidy().unwrap();
    assert_eq!(e.name, "z = z^3 + c");
    // A source that no longer parses is KEPT (it may be a newer build's); an empty one is not.
    assert!(entry("x", "z = z^^2", &[]).tidy().is_some());
    assert!(entry("x", " \r\n\t", &[]).tidy().is_none());
    assert!(entry("x", &"z".repeat(SOURCE_MAX + 1), &[]).tidy().is_none());
    // At most MAX_PARAMS parameters.
    let many: Vec<(&str, &str)> = vec![("1", "0"); MAX_PARAMS + 3];
    assert_eq!(entry("x", "z = z^2 + c", &many).tidy().unwrap().params.len(), MAX_PARAMS);
}

#[test]
fn the_same_formula_is_the_same_source_and_values_whatever_the_name() {
    let a = entry("A", "z = z^2 + p1*z + c", &[("0.25", "-0.1")]);
    assert!(a.same_formula(&entry("B", "z = z^2 + p1*z + c", &[("0.250", "-1e-1")])), "numbers compare as values");
    assert!(!a.same_formula(&entry("A", "z = z^2 + p1*z + c", &[("0.25", "0.1")])));
    assert!(!a.same_formula(&entry("A", "z = z^2 + p1*z - c", &[("0.25", "-0.1")])));
    // A parameter missing on one side reads as 0.
    assert!(entry("A", "z = z^2 + c", &[]).same_formula(&entry("A", "z = z^2 + c", &[("0", "0")])));
}

#[test]
fn saving_under_a_name_replaces_that_entry_and_keeps_the_list_sorted() {
    let mut list = Vec::new();
    assert_eq!(upsert(&mut list, entry("sine", "z = sin(z) + c", &[])), Some(false));
    assert_eq!(upsert(&mut list, entry("Cubic", "z = z^3 + c", &[])), Some(false));
    assert_eq!(upsert(&mut list, entry("cubic", "z = z^3 - c", &[])), Some(false), "names are case-sensitive keys");
    assert_eq!(upsert(&mut list, entry("Cubic", "z = z^3 - p1*z + c", &[("0.5", "0")])), Some(true));
    let names: Vec<&str> = list.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["Cubic", "cubic", "sine"]);
    assert_eq!(list[0].source, "z = z^3 - p1*z + c", "updated in place, not appended");
    assert_eq!(upsert(&mut list, entry("empty", "  ", &[])), None);
    assert_eq!(list.len(), 3);
}

#[test]
fn an_import_never_overwrites_the_library() {
    let mut list = vec![entry("Cubic", "z = z^3 + c", &[]), entry("Sine", "z = sin(z) + c", &[])];
    let incoming = vec![
        entry("My sine", "z = sin(z) + c", &[]),     // already held, under another name
        entry("Cubic", "z = z^3 - p1*z + c", &[("0.5", "0")]), // a different formula, name taken
        entry("Cubic", "z = z^3 - z + c", &[]),      // and again: (2) is taken by then
        entry("Burning", "z = (abs(real(z)) + flip(abs(imag(z))))^2 + c", &[]),
        entry("Blank", "", &[]),
        entry("Burning again", "z = (abs(real(z)) + flip(abs(imag(z))))^2 + c", &[]), // same file, twice
    ];
    let report = merge(&mut list, incoming);
    assert_eq!(
        report,
        MergeReport {
            added: 3,
            duplicates: 2,
            renamed: vec![("Cubic".into(), "Cubic (2)".into()), ("Cubic".into(), "Cubic (3)".into())],
            skipped: 1,
        }
    );
    assert_eq!(list.iter().find(|f| f.name == "Cubic").unwrap().source, "z = z^3 + c", "the user's own is untouched");
    let names: Vec<&str> = list.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["Burning", "Cubic", "Cubic (2)", "Cubic (3)", "Sine"]);
    assert_eq!(
        report.sentence("shared.toml"),
        "Imported 3 formulas from \"shared.toml\": 2 formulas were already in the library; \
         2 renamed where the name was taken; 1 entry was empty or too long, skipped."
    );
    let again = merge(&mut list, vec![entry("Cubic", "z = z^3 + c", &[])]);
    assert_eq!(again.sentence("x.toml"), "Nothing new in \"x.toml\": 1 formula was already in the library.");
    let one = merge(&mut list, vec![entry("Sine", "z = sin(z)*c", &[])]);
    assert_eq!(one.sentence("y.toml"), "Imported 1 formula from \"y.toml\": \"Sine\" was taken, so it is \"Sine (2)\".");
}

/// A Fractint `.frm` file (our own text here: no corpus is bundled) imports the entries that read,
/// each parameter it reads at 0 and what the translation changed in its `about`; the rest are
/// counted by why, and the toast names the commonest.
#[test]
fn a_frm_file_imports_what_reads() {
    let bytes = b"; caf\xe9 formulas\r\n\
        Mandel (XAXIS) {\r\n  z = 0:\r\n  z = sqr(z) + pixel\r\n  |z| <= 4\r\n}\r\n\
        Trig { z = pixel: z = fn1(z)*p2 + pixel, |z| < 64 }\r\n\
        Mandel { z = pixel: z = z*z*z + pixel, |z| <= 4 }\r\n\
        Screen { z = pixel: z = z*z + whitesq, |z| <= 4 }\r\n\
        Screen2 { z = pixel: z = z*z + scrnpix, |z| <= 4 }\r\n\
        Noise { z = pixel: z = z*z + rand, |z| <= 4 }\r\n";
    let got = from_frm(bytes, "mine.frm");
    let names: Vec<&str> = got.formulas.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["Mandel", "Trig", "Mandel"]);
    assert_eq!(got.formulas[0].about, "From mine.frm.");
    assert_eq!(got.formulas[1].about, "From mine.frm: fn1 is sin (Fractint's default).");
    assert_eq!(got.formulas[1].params, vec![["0".to_string(), "0".to_string()]; 2], "p1 and p2, as it reads p2");
    // Grouped by the reason, whichever name it was given for; the commonest first.
    assert_eq!(
        got.unread,
        [("screen and view variables are not supported".to_string(), 2), ("random numbers are not supported".to_string(), 1)]
    );
    let mut list = vec![entry("Mandel", "z = z^2 + c", &[])];
    let report = merge(&mut list, got.formulas.clone());
    assert_eq!((report.added, report.renamed.len()), (3, 2), "two more Mandels, renamed");
    assert_eq!(
        got.sentence().unwrap(),
        "3 entries don't read in this version (most often: screen and view variables are not supported)."
    );
    assert!(from_frm(b"; nothing here\n", "empty.frm").sentence().is_none());
}

#[test]
fn a_formula_file_round_trips_every_character() {
    let list = vec![
        entry("Hybrid \"square\"", "t = sqr(z)\nz = t + p1*conj(t) + c ; a comment", &[("0.25", "0")]),
        entry("Two params", "z = z^p1 + p2", &[("2.2", "0.3"), ("-0.1", "1e-3")]),
        entry("Ünïcode ∑", "z = exp(z) + c", &[]),
    ];
    let text = file_text(&list);
    assert!(text.starts_with("# Fractadyne custom formulas."));
    assert!(text.contains("format = \"fractadyne-formulas\""));
    assert_eq!(parse_file(&text).unwrap(), list);
    // A hand-written file needs neither the header nor the version.
    let hand = "[[formula]]\nname = \"Mine\"\nsource = '''\nz = z^2\nz = z + c'''\n";
    assert_eq!(parse_file(hand).unwrap(), vec![entry("Mine", "z = z^2\nz = z + c", &[])]);
}

#[test]
fn a_file_that_is_not_a_formula_file_says_so() {
    assert!(parse_file("this is = not toml [").unwrap_err().starts_with("not a formula file ("));
    assert_eq!(
        parse_file("format = \"fractadyne-gradients\"\n[[formula]]\nname = \"x\"\nsource = \"z\"\n").unwrap_err(),
        "a \"fractadyne-gradients\" file, not a formula file"
    );
    assert!(parse_file("title = \"a session\"\n").unwrap_err().starts_with("no formulas in it"));
    // A newer file's unknown fields are ignored: it imports what this build reads.
    let newer = "format = \"fractadyne-formulas\"\nversion = 9\n[[formula]]\nname = \"x\"\nsource = \"z = z^2 + c\"\nauthor = \"someone\"\n";
    assert_eq!(parse_file(newer).unwrap(), vec![entry("x", "z = z^2 + c", &[])]);
}

#[test]
fn the_library_saves_and_loads_through_the_disk() {
    let dir = Scratch::new("round-trip");
    let path = dir.0.join("formulas.toml");
    assert_eq!(load_from(&path), (Vec::new(), None), "no file yet: an empty library, nothing to report");
    let list = vec![entry("A", "z = z^2 + c", &[]), entry("B", "z = z^3 + p1", &[("0.5", "0")])];
    save_to(&path, &list).unwrap();
    assert!(!path.with_extension("toml.tmp").exists(), "the temp file was renamed over the library");
    assert_eq!(load_from(&path), (list.clone(), None));
    // Saving again replaces the file whole.
    save_to(&path, &list[..1]).unwrap();
    assert_eq!(load_from(&path).0, list[..1]);
}

#[test]
fn an_unreadable_library_is_moved_aside_never_overwritten() {
    let dir = Scratch::new("unreadable");
    let path = dir.0.join("formulas.toml");
    let garbage = "[[formula]]\nname = \"half a formula\nsource = ";
    std::fs::write(&path, garbage).unwrap();
    let (list, note) = load_from(&path);
    assert!(list.is_empty());
    let note = note.expect("the user is told");
    assert!(note.contains("formulas.unreadable.toml"), "{note}");
    assert!(!path.exists(), "moved, so the next save cannot replace it");
    assert_eq!(std::fs::read_to_string(dir.0.join("formulas.unreadable.toml")).unwrap(), garbage);
    // A second bad file does not replace the first one set aside.
    std::fs::write(&path, "also bad [").unwrap();
    let note = load_from(&path).1.unwrap();
    assert!(note.contains("formulas.unreadable-2.toml"), "{note}");
    assert_eq!(std::fs::read_to_string(dir.0.join("formulas.unreadable.toml")).unwrap(), garbage);
}

#[test]
fn a_row_shows_the_source_on_one_line() {
    assert_eq!(entry("x", "t = sqr(z)\n\n  z = t + c  ", &[]).one_line(), "t = sqr(z), z = t + c");
}

// ---- Starting views ----

/// A starting view survives a file round trip; one that does not read is dropped (the entry kept);
/// an entry without one writes no `view` table, so a library file stays as it was.
#[test]
fn a_starting_view_round_trips_and_a_bad_one_is_dropped() {
    let view = StartView {
        center: ["-0.74364388703".into(), "0.13182590421".into()],
        zoom: "1.2346e500".into(),
        iterations: Some(5000),
        julia: Some(["-0.8".into(), "0.156".into()]),
    };
    let with = SavedFormula { view: Some(view.clone()), ..entry("deep", "z = z^2 + c", &[]) };
    let back = parse_file(&file_text(&[with.clone()])).unwrap().remove(0).tidy().unwrap();
    assert_eq!(back, with);
    assert!((back.view.unwrap().log2_zoom().unwrap() - (1.2346_f64.log2() + 500.0 * std::f64::consts::LOG2_10)).abs() < 1e-9);
    assert!(!file_text(&[entry("plain", "z = z^2 + c", &[])]).contains("view"));
    for bad in [
        StartView { center: ["nan".into(), "0".into()], ..view.clone() },
        StartView { zoom: "lots".into(), ..view.clone() },
        StartView { zoom: "1e99999999".into(), ..view.clone() },
        StartView { julia: Some(["x".into(), "0".into()]), ..view.clone() },
    ] {
        let e = SavedFormula { view: Some(bad.clone()), ..entry("e", "z = z^2 + c", &[]) }.tidy().unwrap();
        assert_eq!(e.view, None, "{bad:?}");
    }
    let many = SavedFormula { view: Some(StartView { iterations: Some(u32::MAX), ..view }), ..entry("e", "z", &[]) };
    assert_eq!(many.tidy().unwrap().view.unwrap().iterations, Some(10_000_000));
}

/// A zoom written as text reads back as the same depth, plainly or past f64's range.
#[test]
fn zoom_text_reads_back_at_any_depth() {
    for l2 in [0.0, 1.5, -2.0, 39.9, 40.0, 1000.0, 3.3e6] {
        let back = crate::parse_zoom_to_log2(&zoom_text(l2)).unwrap();
        assert!((back - l2).abs() < 1e-4 * l2.abs().max(1.0), "{l2} → {} → {back}", zoom_text(l2));
    }
    assert_eq!(zoom_text(0.0), "1");
    assert_eq!(zoom_text(1.0), "2");
}

// ---- The collection ----

/// A frame of escape counts: `Some(n)` escaped at step n, `None` still bounded at the last step.
struct Frame {
    w: usize,
    h: usize,
    px: Vec<Option<u32>>,
}

/// `e` rendered on the CPU at its starting view, `w`×`h`, as the GPU iterates it: z₀ = 0 and c the
/// pixel (Julia mode: z₀ the pixel and c the constant), escape past |z| = 256 — or, for a formula
/// with its own bailout, where that ends the orbit before the cap (the shader's test replaces the
/// radius) — a value that stops being finite tamed to an escape (`fractadyne_gpu::custom::tame_f64`).
fn render(e: &SavedFormula, w: usize, h: usize, max_iter: u32) -> Frame {
    let params: Vec<(f64, f64)> =
        e.params.iter().map(|[re, im]| (re.parse().expect("re"), im.parse().expect("im"))).collect();
    let cf = crate::custom_formula::CustomFormula::compile(&e.source, &params)
        .unwrap_or_else(|why| panic!("{}: {why}", e.name));
    let v = e.view.as_ref().unwrap_or_else(|| panic!("{}: no starting view", e.name));
    let l2 = v.log2_zoom().expect("zoom");
    let (cx, cy): (f64, f64) = (v.center[0].parse().expect("re"), v.center[1].parse().expect("im"));
    let tall = fractadyne_core::Viewport::REFERENCE_HEIGHT / l2.exp2();
    let wide = tall * w as f64 / h as f64;
    let julia = v.julia_c();
    let bail2 = 256.0 * 256.0;
    let mut px = Vec::with_capacity(w * h);
    for j in 0..h {
        for i in 0..w {
            let p = (cx + ((i as f64 + 0.5) / w as f64 - 0.5) * wide, cy + (0.5 - (j as f64 + 0.5) / h as f64) * tall);
            let (z0, c) = match julia {
                Some(k) => (p, k),
                None => ((0.0, 0.0), p),
            };
            let pts = fractadyne_core::ir::orbit_points(&cf.formula, z0, c, &cf.params, max_iter as usize, bail2)
                .expect("parameters supplied");
            let finite = |&(x, y): &(f64, f64)| x.is_finite() && y.is_finite();
            let gone = if cf.formula.has_bailout() {
                let ended = (pts.len() <= max_iter as usize).then(|| pts.len() - 1);
                pts.iter().position(|p| !finite(p)).or(ended)
            } else {
                pts.iter().position(|&(x, y)| !finite(&(x, y)) || x * x + y * y > bail2)
            };
            px.push(gone.map(|n| n as u32));
        }
    }
    Frame { w, h, px }
}

/// What the judge measured.
#[derive(Debug)]
struct Verdict {
    /// Distinct values (escape counts, and "inside" as one).
    levels: usize,
    /// Distinct values in the middle quarter of the frame: something to see where the view looks.
    centre_levels: usize,
    /// The commonest value's share of the frame.
    dominant: f64,
    /// Neighbouring pixels that agree (within 2 steps, or both inside): a picture, not noise.
    coherent: f64,
    /// Neighbouring pixels that differ sharply (3+ steps, or inside against outside): a boundary,
    /// not only the smooth gradient far from the set.
    edges: f64,
}

impl Verdict {
    fn ok(&self) -> bool {
        self.levels >= 8 && self.centre_levels >= 4 && self.dominant <= 0.92 && self.coherent >= 0.5 && self.edges >= 0.005
    }
}

fn judge(f: &Frame) -> Verdict {
    use std::collections::{HashMap, HashSet};
    let mut count: HashMap<Option<u32>, usize> = HashMap::new();
    for p in &f.px {
        *count.entry(*p).or_default() += 1;
    }
    let mut centre = HashSet::new();
    for j in f.h / 4..f.h * 3 / 4 {
        for i in f.w / 4..f.w * 3 / 4 {
            centre.insert(f.px[j * f.w + i]);
        }
    }
    let (mut pairs, mut agree, mut sharp) = (0usize, 0usize, 0usize);
    for j in 0..f.h {
        for i in 0..f.w {
            let a = f.px[j * f.w + i];
            let right = (i + 1 < f.w).then(|| f.px[j * f.w + i + 1]);
            let below = (j + 1 < f.h).then(|| f.px[(j + 1) * f.w + i]);
            for b in [right, below].into_iter().flatten() {
                pairs += 1;
                match (a, b) {
                    (None, None) => agree += 1,
                    (Some(x), Some(y)) if x.abs_diff(y) <= 2 => agree += 1,
                    _ => sharp += 1,
                }
            }
        }
    }
    let n = f.px.len() as f64;
    Verdict {
        levels: count.len(),
        centre_levels: centre.len(),
        dominant: *count.values().max().unwrap_or(&0) as f64 / n,
        coherent: agree as f64 / pairs as f64,
        edges: sharp as f64 / pairs as f64,
    }
}

/// A frame as a PPM image, for looking at: inside black, outside coloured by escape count.
fn write_ppm(f: &Frame, path: &std::path::Path) {
    let mut out = format!("P6\n{} {}\n255\n", f.w, f.h).into_bytes();
    for p in &f.px {
        let rgb = match p {
            None => [0, 0, 0],
            Some(n) => {
                let t = (*n as f64).sqrt() * 0.9;
                let ch = |k: f64| ((0.5 + 0.5 * (t + k).sin()) * 235.0 + 20.0) as u8;
                [ch(0.0), ch(2.1), ch(4.2)]
            }
        };
        out.extend_from_slice(&rgb);
    }
    std::fs::write(path, out).unwrap();
}

/// The collection reads in full: every entry survives `tidy` (none dropped, no view refused), has
/// a category, a line about it and a starting view, and no two share a name or a formula and view.
#[test]
fn the_collection_reads_in_full() {
    let raw = parse_file(COLLECTION).expect("the collection is a formula file");
    let all = collection();
    assert_eq!(all.len(), raw.len(), "an entry tidy dropped");
    assert!(all.len() >= 35, "{} formulas", all.len());
    for (e, r) in all.iter().zip(&raw) {
        assert!(r.view.is_some() && e.view.is_some(), "{}: no starting view, or one tidy refused", e.name);
        assert!(!e.category.is_empty() && !e.about.is_empty(), "{}", e.name);
    }
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            assert_ne!(a.name, b.name);
            assert!(!a.same_formula(b) || a.view != b.view, "{} and {} are the same", a.name, b.name);
        }
    }
}

/// THE gate: every formula in the collection, rendered at its starting view, shows a picture —
/// structure, a boundary, something in the middle, and not noise. The judge is held to its job by
/// controls it must fail (a view of nothing, a formula that never escapes, noise) and one it must
/// pass (the Mandelbrot set). `FRACTADYNE_COLLECTION_PPM=<dir>` also writes each frame, larger.
#[test]
fn every_collection_formula_shows_a_picture_at_its_view() {
    let look = std::env::var_os("FRACTADYNE_COLLECTION_PPM").map(std::path::PathBuf::from);
    let mut bad = Vec::new();
    for (k, e) in collection().iter().enumerate() {
        let iters = e.view.as_ref().and_then(|v| v.iterations).unwrap_or(300).min(400);
        let v = judge(&render(e, 64, 40, iters));
        if !v.ok() {
            bad.push(format!("{}: {v:?}", e.name));
        }
        if let Some(dir) = &look {
            let f = render(e, 320, 200, e.view.as_ref().and_then(|v| v.iterations).unwrap_or(300));
            std::fs::create_dir_all(dir).unwrap();
            write_ppm(&f, &dir.join(format!("{k:02}.ppm")));
        }
    }
    assert!(bad.is_empty(), "no picture at the starting view:\n{}", bad.join("\n"));

    let control = |source: &str, center: [&str; 2], zoom: &str| SavedFormula {
        name: "control".into(),
        source: source.into(),
        view: Some(StartView { center: center.map(String::from), zoom: zoom.into(), iterations: Some(300), julia: None }),
        ..Default::default()
    };
    let nothing = judge(&render(&control("z = z^2 + c", ["1000", "1000"], "1"), 64, 40, 300));
    assert!(!nothing.ok(), "a view where everything escapes at once passed: {nothing:?}");
    let inside = judge(&render(&control("z = z*0.5 + c*0", ["0", "0"], "1"), 64, 40, 300));
    assert!(!inside.ok(), "a formula that never escapes passed: {inside:?}");
    let mut seed = 0x9e37_79b9_u64;
    let noise: Vec<Option<u32>> = (0..64 * 40)
        .map(|_| {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            Some((seed >> 33) as u32 % 60)
        })
        .collect();
    let noise = judge(&Frame { w: 64, h: 40, px: noise });
    assert!(!noise.ok(), "noise passed: {noise:?}");
    let m = judge(&render(&control("z = z^2 + c", ["-0.5", "0"], "1"), 64, 40, 300));
    assert!(m.ok(), "the Mandelbrot set failed the judge: {m:?}");
}
