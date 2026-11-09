//! The bisect's pieces: reading WGSL's functions and blocks, writing a variant, and the search.

use super::*;

const SAMPLE: &str = "\
struct Out { @location(0) a: vec4<f32>, }
// fn not_a_function() { this is a comment }
/* a /* nested */ block comment: fn nope() {} */
const K: f32 = 2.0;
fn sq(x: f32) -> f32 { return x * x; }
fn pair(x: f32) -> vec2<f32> {
    if (x > K) {
        return vec2<f32>(sq(x), 0.0);
    } else {
        loop {
            if (x < 0.0) { break; }
            return vec2<f32>(x, 1.0);
        }
    }
    switch (u32(x)) {
        case 1u: { return vec2<f32>(1.0); }
        default: { }
    }
    return vec2<f32>(0.0);
}
fn noop() { }
@fragment
fn fs_main(@builtin(position) p: vec4<f32>) -> @location(0) vec4<f32> {
    noop();
    return vec4<f32>(pair(p.x), 0.0, 1.0);
}
";

#[test]
fn comments_are_blanked_byte_for_byte() {
    let b = blank_comments(SAMPLE);
    assert_eq!(b.len(), SAMPLE.len());
    let s = String::from_utf8(b).unwrap();
    assert_eq!(s.lines().count(), SAMPLE.lines().count(), "newlines kept");
    assert!(!s.contains("not_a_function") && !s.contains("nope") && !s.contains("nested"), "{s}");
    assert!(s.contains("const K: f32 = 2.0;"));
    // A comment holding non-ASCII text blanks to ASCII spaces, so offsets still line up.
    // "// ⭐ é" is 2 + 1 + 3 + 1 + 2 bytes, and the space before it is code.
    assert_eq!(blank_comments("x // ⭐ é\ny"), format!("x{}\ny", " ".repeat(10)).into_bytes());
}

#[test]
fn functions_are_found_with_their_return_types_and_entry_points() {
    let f = functions(SAMPLE);
    let names: Vec<&str> = f.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["sq", "pair", "noop", "fs_main"], "nothing from the comments");
    assert_eq!(f[0].ret.as_deref(), Some("f32"));
    assert_eq!(f[1].ret.as_deref(), Some("vec2<f32>"));
    assert_eq!(f[2].ret, None);
    assert_eq!(f[3].ret.as_deref(), Some("vec4<f32>"), "the @location attribute is not the type");
    assert_eq!(f.iter().map(|f| f.entry).collect::<Vec<_>>(), [false, false, false, true]);
    for x in &f {
        assert_eq!(&SAMPLE[x.open..=x.open], "{");
        assert_eq!(&SAMPLE[x.close..=x.close], "}");
    }
}

#[test]
fn blocks_nest_and_a_switch_is_not_one_but_its_cases_are() {
    let f = functions(SAMPLE);
    let b = blocks(SAMPLE, &f);
    // In `pair`: if, else, loop (in else), if (in loop), case 1u, default. `fs_main` has none.
    let heads: Vec<String> = b.iter().map(|bl| head_of(SAMPLE, bl, &f)).collect();
    assert_eq!(heads, ["if (x > K)", "else", "loop", "if (x < 0.0)", "case 1u:", "default:"]);
    assert!(b.iter().all(|bl| bl.func == 1));
    assert_eq!(b.iter().map(|bl| bl.parent).collect::<Vec<_>>(), [None, None, Some(1), Some(2), None, None]);
    for bl in &b {
        assert_eq!(&SAMPLE[bl.open..=bl.open], "{");
        assert_eq!(&SAMPLE[bl.close..=bl.close], "}");
    }
}

#[test]
fn reachability_follows_the_calls() {
    let f = functions(SAMPLE);
    assert_eq!(reachable(SAMPLE, &f, &["fs_main"]), [true, true, true, true]);
    assert_eq!(reachable(SAMPLE, &f, &["sq"]), [true, false, false, false]);
}

#[test]
fn a_variant_stubs_functions_and_empties_blocks_with_a_return_on_every_path() {
    let f = functions(SAMPLE);
    let b = blocks(SAMPLE, &f);
    let mut v = Variant { stubbed: vec![false; f.len()], emptied: vec![false; b.len()] };
    v.stubbed[0] = true; // sq
    v.stubbed[2] = true; // noop
    v.stubbed[3] = true; // the entry point: never stubbed
    v.emptied[1] = true; // else (and the loop inside it)
    v.emptied[3] = true; // inside the emptied else: already gone
    let out = render(SAMPLE, &f, &b, &v);
    assert!(out.contains("fn sq(x: f32) -> f32 { return f32(); }"), "{out}");
    assert!(out.contains("fn noop() { return; }"), "{out}");
    assert!(out.contains("} else { return vec2<f32>(); }"), "{out}");
    assert!(!out.contains("loop"), "{out}");
    assert!(out.contains("noop();\n    return vec4<f32>(pair(p.x), 0.0, 1.0);"), "the entry point is untouched: {out}");
    // Nothing chosen: the source, byte for byte.
    let none = Variant { stubbed: vec![false; f.len()], emptied: vec![false; b.len()] };
    assert_eq!(render(SAMPLE, &f, &b, &none), SAMPLE);
}

#[test]
fn ddmin_finds_the_items_the_failure_needs() {
    let items: Vec<usize> = (0..40).collect();
    // Distinct sets: the search caches a variant it has linked, so a repeat costs no probe.
    let mut asked = std::collections::HashSet::new();
    let mut test = |keep: &[usize]| {
        asked.insert(keep.to_vec());
        if keep.contains(&3) && keep.contains(&29) { Verdict::Fail } else { Verdict::Pass }
    };
    let (kept, stopped) = ddmin(&items, &mut test);
    assert_eq!(kept, [3, 29]);
    assert!(!stopped);
    assert!(asked.len() <= 60, "{} distinct tests", asked.len());
}

#[test]
fn ddmin_steps_around_unresolved_variants_and_stops_when_told() {
    let items: Vec<usize> = (0..16).collect();
    // Any set without item 0 is rejected (unresolved); the failure needs 0 and 9.
    let mut test = |keep: &[usize]| match (keep.contains(&0), keep.contains(&9)) {
        (false, _) => Verdict::Unresolved,
        (true, true) => Verdict::Fail,
        (true, false) => Verdict::Pass,
    };
    assert_eq!(ddmin(&items, &mut test).0, [0, 9]);
    // Nothing needed: the empty set already fails.
    assert_eq!(ddmin(&items, &mut |_: &[usize]| Verdict::Fail), (vec![], false));
    // Out of time at once: the whole set back, marked as stopped.
    assert_eq!(ddmin(&items, &mut |_: &[usize]| Verdict::Stop), (items.clone(), true));
}

#[test]
fn a_crash_report_gives_its_faulting_place() {
    let native = "fractadyne crash report\npanic   : a native crash: access violation\nat      : <native: ig75icd64.dll+0x73489a>\nmemory  : rss 127 MB";
    assert_eq!(crash_signature(native).as_deref(), Some("ig75icd64.dll+0x73489a"));
    let panic = "panic   : wgpu error\nat      : crates/fractadyne-gpu/src/lib.rs:2136:5\n";
    assert_eq!(crash_signature(panic).as_deref(), Some("crates/fractadyne-gpu/src/lib.rs:2136:5"));
    assert_eq!(crash_signature("no such line"), None);
}

#[test]
fn the_renderers_shader_reads_whole() {
    let src = fractadyne_gpu::link_probe::RENDERER_WGSL;
    let f = functions(src);
    // Every `fn` that starts a line (after its attributes) is found.
    let written = src.lines().filter(|l| l.trim_start().starts_with("fn ")).count();
    assert_eq!(f.len(), written);
    assert!(f.iter().any(|x| x.name == "fs_iterate" && x.entry));
    assert!(f.iter().any(|x| x.name == "vs_main" && x.entry));
    let live = reachable(src, &f, &["vs_main", "fs_iterate"]);
    let iterate_at = f.iter().position(|x| x.name == "iterate_at").expect("iterate_at");
    assert!(live[iterate_at]);
    // Bodies and blocks are well formed and nest.
    for x in &f {
        assert!(x.open < x.close && &src[x.open..=x.open] == "{" && &src[x.close..=x.close] == "}", "{}", x.name);
    }
    let b = blocks(src, &f);
    assert!(b.len() > 100, "{} blocks", b.len());
    for bl in &b {
        assert!(bl.open < bl.close, "a block at {} never closed", line_of(src, bl.open));
        if let Some(p) = bl.parent {
            assert!(b[p].open < bl.open && bl.close < b[p].close);
        }
    }
    // Every helper stubbed: the source shrinks (its comments and 17 entry points stay), the
    // iteration itself becomes a return of the zero value, and the entry point still calls it.
    let all = Variant { stubbed: vec![true; f.len()], emptied: vec![false; b.len()] };
    let out = render(src, &f, &b, &all);
    assert!(out.len() < src.len() * 3 / 4, "{} of {}", out.len(), src.len());
    assert!(out.contains("fn iterate_at(gx: f32, gy: f32) -> FragOut { return FragOut(); }"));
    assert!(out.contains("return iterate_at(iu.px_offset.x + in.pos.x, iu.px_offset.y + in.pos.y);"));
}
