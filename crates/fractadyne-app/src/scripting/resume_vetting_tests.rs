//! `--resume`'s frame vetting: the reader that decides which of thousands of already-rendered
//! frames survive an interrupted render.
//!
//! It had no direct coverage, and it is the code between a killed render and a corrupt frame
//! silently baked into the middle of a finished video — the disk-full failure at frame 1091 of
//! 9,931, which a naive `exists()` resume would have kept forever.
use super::{png_frame_size, FractadyneApp};

struct Tmp(std::path::PathBuf);
impl Tmp {
    fn new(tag: &str) -> Self {
        let d = std::env::temp_dir()
            .join(format!("fractadyne_resume_test_{}_{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("temp dir");
        Self(d)
    }
}
impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Write frame `n` as a real PNG of `w×h`, optionally truncated by `cut` bytes to imitate a
/// write that was killed part way.
fn frame(dir: &std::path::Path, n: u64, w: u32, h: u32, cut: usize) {
    let path = dir.join(format!("frame_{n:05}.png"));
    let px = vec![0.5f32; (w * h * 4) as usize];
    fractadyne_export::write_png(&path, w, h, &px, None).expect("write frame");
    if cut > 0 {
        let bytes = std::fs::read(&path).expect("read back");
        std::fs::write(&path, &bytes[..bytes.len() - cut]).expect("truncate");
    }
}

/// The structural check, on the shapes an interrupted write actually produces.
#[test]
fn png_frame_size_accepts_only_structurally_complete_frames() {
    let t = Tmp::new("size");
    let d = &t.0;
    frame(d, 0, 64, 36, 0);
    assert_eq!(png_frame_size(&d.join("frame_00000.png")), Some((64, 36)));

    // ⭐The case that matters most, and the one a successful DECODE would miss: a file one
    // byte short of complete. `read_png_rgba8_bytes` accepts it (pinned in fractadyne-export's
    // `corrupt_input_is_an_error_not_a_panic`) — the vetter must not.
    frame(d, 1, 64, 36, 1);
    assert_eq!(png_frame_size(&d.join("frame_00001.png")), None, "a 1-byte-short frame passed");

    frame(d, 2, 64, 36, 12); // IEND gone entirely
    assert_eq!(png_frame_size(&d.join("frame_00002.png")), None);
    frame(d, 3, 64, 36, 40); // into the image data
    assert_eq!(png_frame_size(&d.join("frame_00003.png")), None);

    // Non-PNGs, empties and absent files are all "unusable", never a panic.
    std::fs::write(d.join("frame_00004.png"), b"").unwrap();
    assert_eq!(png_frame_size(&d.join("frame_00004.png")), None);
    std::fs::write(d.join("frame_00005.png"), vec![0u8; 200]).unwrap();
    assert_eq!(png_frame_size(&d.join("frame_00005.png")), None);
    assert_eq!(png_frame_size(&d.join("nope.png")), None);
}

/// A clean sequence resumes; nothing is discarded.
#[test]
fn a_clean_sequence_resumes_untouched() {
    let t = Tmp::new("clean");
    for n in 0..3 {
        frame(&t.0, n, 64, 36, 0);
    }
    let msg = FractadyneApp::prepare_resume(&t.0, "frame", 64, 36).expect("ok");
    assert!(msg.contains("3 frames on disk (through 2)"), "{msg}");
    assert!(!msg.contains("discarded"), "{msg}");
    for n in 0..3 {
        assert!(t.0.join(format!("frame_{n:05}.png")).exists(), "frame {n} was removed");
    }
}

/// ⭐The whole reason this code exists: the frame a render dies on is the one that is PRESENT
/// but INCOMPLETE. It must be discarded — and only it, since a render can only be killed
/// mid-write on the frame it was writing.
#[test]
fn the_interrupted_trailing_frame_is_discarded_and_only_it() {
    let t = Tmp::new("interrupted");
    for n in 0..4 {
        frame(&t.0, n, 64, 36, 0);
    }
    frame(&t.0, 4, 64, 36, 1); // killed one byte from the end
    let msg = FractadyneApp::prepare_resume(&t.0, "frame", 64, 36).expect("ok");
    assert!(msg.contains("discarded 1 incomplete"), "{msg}");
    assert!(msg.contains("through 3"), "{msg}");
    assert!(!t.0.join("frame_00004.png").exists(), "the bad frame was left on disk");
    for n in 0..4 {
        assert!(t.0.join(format!("frame_{n:05}.png")).exists(), "frame {n} was removed");
    }
}

/// A COMPLETE frame at another size means the folder holds a different render. Resuming would
/// interleave two resolutions into one sequence, so it must refuse and say why — silently
/// producing an unusable mix is the failure worth preventing.
#[test]
fn a_different_resolution_is_refused_with_both_sizes_named() {
    let t = Tmp::new("size_clash");
    frame(&t.0, 0, 64, 36, 0);
    let err = FractadyneApp::prepare_resume(&t.0, "frame", 128, 72).expect_err("must refuse");
    assert!(err.contains("64×36") && err.contains("128×72"), "{err}");
    assert!(t.0.join("frame_00000.png").exists(), "a size clash must not delete frames");
}

/// Nothing on disk, and a folder of nothing but wreckage, are both non-errors: the render just
/// starts from the beginning.
#[test]
fn an_empty_or_entirely_unusable_folder_starts_over() {
    let t = Tmp::new("empty");
    assert_eq!(FractadyneApp::prepare_resume(&t.0, "frame", 64, 36).expect("ok"), "");
    for n in 0..2 {
        frame(&t.0, n, 64, 36, 1);
    }
    let msg = FractadyneApp::prepare_resume(&t.0, "frame", 64, 36).expect("ok");
    assert!(msg.contains("no usable frames found, discarded 2"), "{msg}");
    // A directory that does not exist at all is the very first run.
    let gone = t.0.join("not_created");
    assert_eq!(FractadyneApp::prepare_resume(&gone, "frame", 64, 36).expect("ok"), "");
}

/// Files that are not this render's frames must be ignored, not parsed as frame numbers.
#[test]
fn foreign_files_are_ignored() {
    let t = Tmp::new("foreign");
    frame(&t.0, 7, 64, 36, 0);
    std::fs::write(t.0.join("notes.txt"), b"hello").unwrap();
    std::fs::write(t.0.join("frame_00007.jpg"), b"x").unwrap();
    std::fs::write(t.0.join("other_00009.png"), b"x").unwrap();
    std::fs::write(t.0.join("frame_abc.png"), b"x").unwrap();
    let msg = FractadyneApp::prepare_resume(&t.0, "frame", 64, 36).expect("ok");
    assert!(msg.contains("1 frames on disk (through 7)"), "{msg}");
}
