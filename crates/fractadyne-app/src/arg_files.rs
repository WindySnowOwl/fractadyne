//! F-02 response-file (`@FILE` / `--args-file`) expansion limits.
//!
//! Depth-16 alone never bounded the work: one enormous file, an over-long token, a self-include,
//! or exponential fan-out through repeated includes could each exhaust memory or startup time
//! before diagnostics exist. These tests pin that every axis is now bounded, and — the other
//! direction — that a normal command line and ordinary nesting still expand.
use super::expand_arg_files;

/// Throwaway directory, per test, in the OS temp dir (repo convention — no dev-dependency).
struct Tmp(std::path::PathBuf);
impl Tmp {
    fn new(tag: &str) -> Self {
        let d = std::env::temp_dir()
            .join(format!("fractadyne_argfiles_{}_{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("temp dir");
        Self(d)
    }
    fn write(&self, name: &str, contents: &[u8]) -> std::path::PathBuf {
        let p = self.0.join(name);
        std::fs::write(&p, contents).expect("write args file");
        p
    }
}
impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

#[test]
fn plain_arguments_pass_through_untouched() {
    let out = expand_arg_files(&s(&["fractadyne", "--center", "-0.75", "0.0"])).unwrap();
    assert_eq!(out, s(&["fractadyne", "--center", "-0.75", "0.0"]));
}

#[test]
fn a_response_file_is_spliced_in_place() {
    let tmp = Tmp::new("splice");
    let f = tmp.write("args.txt", b"--size 480x270  # a comment\n--no-sound\n");
    let out = expand_arg_files(&s(&["prog", &format!("@{}", f.display()), "--fast"])).unwrap();
    assert_eq!(out, s(&["prog", "--size", "480x270", "--no-sound", "--fast"]));
}

#[test]
fn a_file_over_the_per_file_cap_is_refused() {
    let tmp = Tmp::new("bigfile");
    // > 4 MiB. Spaces tokenize to nothing, but the size check fires before the read/tokenize.
    let f = tmp.write("big.txt", &vec![b' '; 4 * 1024 * 1024 + 1]);
    let err = expand_arg_files(&s(&["prog", &format!("@{}", f.display())])).unwrap_err();
    assert!(err.contains("too large"), "unexpected error: {err}");
}

#[test]
fn repeated_includes_are_bounded_by_the_combined_cap() {
    let tmp = Tmp::new("total");
    // One ~4 MiB file included five times = ~20 MiB read, over the 16 MiB combined cap. This is the
    // exponential-fan-out shape (repeated includes), caught even though no single file is too big
    // and there is no cycle.
    let f = tmp.write("chunk.txt", &vec![b' '; 4 * 1024 * 1024 - 16]);
    let at = format!("@{}", f.display());
    let err = expand_arg_files(&s(&["prog", &at, &at, &at, &at, &at])).unwrap_err();
    assert!(err.contains("combined size limit"), "unexpected error: {err}");
}

#[test]
fn an_over_long_token_is_refused() {
    let tmp = Tmp::new("longtok");
    let f = tmp.write("tok.txt", &vec![b'x'; 64 * 1024 + 1]); // one token > 64 KiB
    let err = expand_arg_files(&s(&["prog", &format!("@{}", f.display())])).unwrap_err();
    assert!(err.contains("token too long"), "unexpected error: {err}");
}

#[test]
fn too_many_expanded_tokens_are_refused() {
    let tmp = Tmp::new("manytok");
    // 100_001 one-char tokens (~200 KB — well under the file cap) trips the argv-length cap.
    let mut content = Vec::with_capacity(200_002);
    for _ in 0..100_001 {
        content.extend_from_slice(b"x ");
    }
    let f = tmp.write("many.txt", &content);
    let err = expand_arg_files(&s(&["prog", &format!("@{}", f.display())])).unwrap_err();
    assert!(err.contains("too many arguments"), "unexpected error: {err}");
}

#[test]
fn a_self_including_file_is_rejected_as_a_cycle() {
    let tmp = Tmp::new("cycle");
    let p = tmp.0.join("loop.txt");
    std::fs::write(&p, format!("@{}", p.display())).expect("write");
    let err = expand_arg_files(&s(&["prog", &format!("@{}", p.display())])).unwrap_err();
    assert!(err.contains("cycle"), "unexpected error: {err}");
}

#[test]
fn the_same_file_may_be_included_twice_in_sequence() {
    // Cycle detection is stack-based, so a diamond/sequential re-include (bounded, legitimate) is
    // NOT a cycle — only a ring is.
    let tmp = Tmp::new("twice");
    let f = tmp.write("once.txt", b"--no-sound");
    let at = format!("@{}", f.display());
    let out = expand_arg_files(&s(&["prog", &at, &at])).unwrap();
    assert_eq!(out, s(&["prog", "--no-sound", "--no-sound"]));
}

#[test]
fn a_missing_response_file_is_a_hard_error() {
    let tmp = Tmp::new("missing");
    let missing = tmp.0.join("nope.txt");
    let err = expand_arg_files(&s(&["prog", &format!("@{}", missing.display())])).unwrap_err();
    assert!(err.contains("args file"), "unexpected error: {err}");
}
