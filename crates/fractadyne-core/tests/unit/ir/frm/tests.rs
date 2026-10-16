use super::*;
use crate::ir::orbit_points;

fn bits(p: (f64, f64)) -> (u64, u64) {
    (p.0.to_bits(), p.1.to_bits())
}

/// A sample file in Fractint's layout, written for these tests: commentary outside braces, a
/// `comment` block, symmetry and `[…]` options in headers, a brace on its own line, CRLF and a
/// continued line.
const SAMPLE: &str = "; a file of formulas, for the tests\r\n\
comment {\r\n  This block is documentation: z = = = {\r\n}\r\n\
\r\n\
Mandel (XAXIS) { ; the Mandelbrot set\r\n  z = 0:\r\n  z = Sqr(Z) + Pixel\r\n  |z| <= 4 }\r\n\
\r\n\
Trig[float=y]\r\n\
{\r\n  z = pixel:\r\n  z = fn1(z) * \\\r\n      fn2(z) + p1\r\n  |z| < 64\r\n}\r\n\
\r\n\
Named-C (ORIGIN) {\r\n  c = pixel, z = 0, c_ = 3:\r\n  z = z*z + c\r\n  |z| <= 4\r\n}\r\n\
\r\n\
Noise {\r\n  z = pixel:\r\n  z = z*z + rand\r\n  |z| <= 4\r\n}\r\n";

#[test]
fn entries_are_read_in_order() {
    let entries = read_frm(SAMPLE);
    let names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["Mandel", "Trig", "Named-C", "Noise"], "commentary and `comment` blocks are skipped");
    let reads: Vec<_> = entries.iter().map(|e| e.reads.is_ok()).collect();
    assert_eq!(reads, [true, true, true, false], "{entries:#?}");
}

/// Names are lower-cased; the translated Mandelbrot is the Mandelbrot set, bailout and all.
#[test]
fn a_classic_entry_computes_what_fractint_computes() {
    let mandel = &read_frm(SAMPLE)[0];
    assert!(mandel.notes.is_empty(), "nothing to change: {:?}", mandel.notes);
    assert!(mandel.source.contains("sqr(z) + pixel"), "{}", mandel.source);
    let f = parse(&mandel.source).unwrap();
    assert!(f.has_bailout());
    for &c in &[(-0.75, 0.1), (0.3, 0.5), (-1.9, 0.0), (0.26, 0.0)] {
        let got = orbit_points(&f, (0.0, 0.0), c, &[], 500, 1.0e300).unwrap();
        let mut z = (0.0f64, 0.0f64);
        let mut want = vec![z];
        for _ in 0..500 {
            z = (z.0 * z.0 - z.1 * z.1 + c.0, 2.0 * (z.0 * z.1) + c.1);
            want.push(z);
            if !(z.0 * z.0 + z.1 * z.1 <= 4.0) {
                break;
            }
        }
        assert_eq!(got.iter().map(|p| bits(*p)).collect::<Vec<_>>(), want.iter().map(|p| bits(*p)).collect::<Vec<_>>(), "at {c:?}");
    }
}

/// `fn1`…`fn4` take Fractint's defaults and say so; a continued line joins its next.
#[test]
fn function_slots_take_fractints_defaults() {
    let trig = &read_frm(SAMPLE)[1];
    assert!(trig.source.contains("sin(z) *   ") && trig.source.contains("sqr(z) + p1"), "{}", trig.source);
    assert_eq!(trig.notes, ["fn1 is sin (Fractint's default)", "fn2 is sqr (Fractint's default)"]);
    let by_hand = parse("z = pixel:\n z = sin(z)*sqr(z) + p1\n |z| < 64").unwrap();
    assert_eq!(parse(&trig.source).unwrap(), by_hand);
}

/// Fractint's `c` is an ordinary variable; renamed past every name in use, it computes the same.
#[test]
fn fractints_c_is_renamed_to_a_free_name() {
    let named = &read_frm(SAMPLE)[2];
    assert!(named.source.contains("c__ = pixel") && named.source.contains("z*z + c__"), "`c_` is taken: {}", named.source);
    assert_eq!(named.notes, ["Fractint's variable c is c__ (c is the pixel here)"]);
    let f = parse(&named.source).unwrap();
    let mandel = parse(&read_frm(SAMPLE)[0].source).unwrap();
    for &c in &[(-0.75, 0.1), (0.3, 0.5), (-1.2, 0.3)] {
        assert_eq!(
            orbit_points(&f, (0.0, 0.0), c, &[], 300, 1.0e300).unwrap(),
            orbit_points(&mandel, (0.0, 0.0), c, &[], 300, 1.0e300).unwrap(),
            "at {c:?}"
        );
    }
}

/// What the language lacks is named, never mis-read.
#[test]
fn an_unsupported_feature_is_named() {
    let noise = &read_frm(SAMPLE)[3];
    let why = noise.reads.as_ref().unwrap_err();
    assert!(why.contains("rand"), "{why}");
}

/// Fractint ignores blanks: `end if` is `endif`, `else if` is `elseif`. The words so joined are
/// noted. A blank between a word and anything else stays.
#[test]
fn blanks_inside_words_are_dropped() {
    let e = &read_frm("A {\n z = pixel, n = 0:\n n = n + 1\n if (n > 3)\n  z = z*z + pixel\n else if (n > 1)\n  z = z + 1\n end if\n |z| <= 4 ; end if here stays\n}")[0];
    assert!(e.reads.is_ok(), "{e:?}");
    assert!(e.source.contains(" endif\n") && e.source.contains(" elseif (n > 1)") && e.source.contains("; end if here stays"), "{}", e.source);
    assert_eq!(e.notes, ["blanks inside names are dropped, as Fractint drops them: elseif, endif"]);
    let plain = &read_frm("B { z = pixel: z = z * z + pixel, |z| <= 4 }")[0];
    assert!(plain.notes.is_empty() && plain.source.contains("z * z + pixel"), "{plain:?}");
}

/// A name nothing sets is 0, as in Fractint, and said; the formula computes with 0 there.
#[test]
fn a_name_nothing_sets_is_zero() {
    let e = &read_frm("Newt { z = pixel: f1 = z^3 - 1, f2 = 3*z^2\n z = z - f1/(f2 - f3*f1)\n |f1| > 1e-10 }")[0];
    assert!(e.reads.is_ok(), "{e:?}");
    assert_eq!(e.notes, ["f3 is never set: 0, as in Fractint"]);
    let by_hand = parse("f3 = 0, z = pixel: f1 = z^3 - 1, f2 = 3*z^2\n z = z - f1/(f2 - f3*f1)\n |f1| > 1e-10").unwrap();
    let f = parse(&e.source).unwrap();
    for &c in &[(0.4, 0.9), (-1.2, 0.1), (0.7, -0.3)] {
        assert_eq!(orbit_points(&f, (0.0, 0.0), c, &[], 100, 1.0e300), orbit_points(&by_hand, (0.0, 0.0), c, &[], 100, 1.0e300));
    }
    // Without an init section, one is made.
    let e = &read_frm("G { z = z*z + pixel + g, |z| <= 4 }")[0];
    assert!(e.reads.is_ok() && e.source.starts_with("g = 0:\n"), "{e:?}");
}

/// Fractint's last statement is the bailout test whatever it is; one that does not compare is
/// written as the test it is (`!= 0`), never read as the new z.
#[test]
fn the_last_statement_is_the_test() {
    let e = &read_frm("Count { z = pixel, n = 10:\n z = z*z + pixel\n n = n - 1\n n ; the count\n}")[0];
    assert!(e.reads.is_ok(), "{e:?}");
    assert!(e.source.contains("(n) != 0 ; the count"), "{}", e.source);
    assert_eq!(e.notes, ["the last statement, n, is the bailout test (as in Fractint): n != 0"]);
    // Ten steps, then n is 0: z₀ and ten iterates (whatever z did).
    let f = parse(&e.source).unwrap();
    assert_eq!(orbit_points(&f, (0.0, 0.0), (0.1, 0.1), &[], 100, 1.0e300).unwrap().len(), 11);
    // A comparison, or an `if` block last, is left as it is.
    for body in ["z = z*z + pixel, |z| <= 4", "if (|z| > 4)\n z = 0\nendif"] {
        let e = &read_frm(&format!("X {{ z = pixel: {body} }}"))[0];
        assert!(!e.source.contains("!= 0"), "{}", e.source);
    }
}

/// A continued line may have blanks after its backslash; a comment ending in one continues
/// nothing.
#[test]
fn continuations() {
    let e = &read_frm("A {\n z = pixel: \\  \n z = z*z \\\t\n + pixel ; see C:\\\n |z| <= 4 }")[0];
    assert!(e.reads.is_ok(), "{e:?}");
    let f = parse(&e.source).unwrap();
    assert_eq!(f, parse("z = pixel: z = z*z + pixel\n |z| <= 4").unwrap());
}

/// A DOS-era file is Latin-1; its accents (in comments) do not stop it reading.
#[test]
fn a_latin1_file_reads() {
    let bytes = b"; \xe9t\xe9\nA { z = pixel: ; caf\xe9\n z = z*z + pixel, |z| <= 4 }";
    let entries = read_frm_bytes(bytes);
    assert_eq!(entries.len(), 1);
    assert!(entries[0].reads.is_ok(), "{:?}", entries[0]);
    assert!(entries[0].source.contains("caf\u{e9}"));
}

/// The conformance run (design/custom-formulas.md §5, phase 3): every `.frm` under
/// `FRACTADYNE_FRM_CORPUS` read, distinct bodies counted, the share that reads printed with the
/// commonest reasons the rest do not. The corpora are never bundled, so the run is opt-in:
/// `FRACTADYNE_FRM_CORPUS=<dir> cargo test -p fractadyne-core frm_corpus -- --ignored --nocapture`.
#[test]
#[ignore]
fn frm_corpus() {
    let Ok(dir) = std::env::var("FRACTADYNE_FRM_CORPUS") else {
        panic!("set FRACTADYNE_FRM_CORPUS to a directory of .frm files");
    };
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("frm")))
        .collect();
    files.sort();
    let mut bodies = std::collections::BTreeMap::<String, Result<(), String>>::new();
    let mut entries = 0;
    for path in &files {
        for e in read_frm_bytes(&std::fs::read(path).unwrap()) {
            entries += 1;
            // The same body under another name, or other comments and spacing, counts once.
            let key: String = e.source.lines().map(|l| l.split(';').next().unwrap_or("")).flat_map(|l| l.split_whitespace()).collect::<Vec<_>>().join(" ");
            bodies.entry(key).or_insert(e.reads);
        }
    }
    let read = bodies.values().filter(|r| r.is_ok()).count();
    let mut why = std::collections::BTreeMap::<String, usize>::new();
    for r in bodies.values() {
        if let Err(m) = r {
            // The message up to its first quote-delimited detail, so like failures group.
            let head: String = m.chars().take(70).collect();
            *why.entry(head).or_default() += 1;
        }
    }
    let mut why: Vec<_> = why.into_iter().collect();
    why.sort_by(|a, b| b.1.cmp(&a.1));
    println!("{} files, {entries} entries, {} distinct bodies, {read} read ({:.1}%)", files.len(), bodies.len(), 100.0 * read as f64 / bodies.len() as f64);
    for (m, n) in why.iter().take(40) {
        println!("{n:6}  {m}");
    }
    // Every body that does not read, with its reason, to look at.
    if let Ok(out) = std::env::var("FRACTADYNE_FRM_FAILURES") {
        let mut text = String::new();
        for (body, r) in &bodies {
            if let Err(m) = r {
                text.push_str(&format!("{m}\n    {body}\n"));
            }
        }
        std::fs::write(out, text).unwrap();
    }
    assert!(read as f64 >= 0.88 * bodies.len() as f64, "the design's gate is 88% of distinct bodies");
}

/// A file with no entries, or one cut off mid-entry, reads what is whole.
#[test]
fn a_damaged_file_reads_what_is_whole() {
    assert!(read_frm("").is_empty());
    assert!(read_frm("; only commentary\n").is_empty());
    let cut = read_frm("A { z = z*z + c }\nB { z = z*z");
    assert_eq!(cut.len(), 1);
    assert_eq!(cut[0].name, "A");
    // A brace in a comment does not open or close an entry.
    let commented = read_frm("; { not an entry }\nA { ; } still A\n z = z*z + pixel }");
    assert_eq!(commented.len(), 1);
    assert!(commented[0].reads.is_ok(), "{:?}", commented[0]);
}
