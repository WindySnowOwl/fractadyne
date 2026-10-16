//! Atomic file writes: a PNG exists under its final name only once it is complete.
//!
//! The property every reader of a frame folder depends on — `--resume`'s vetting, and a render
//! farm whose machines share one folder — is that a name either holds a finished file or nothing.
//! Each test pins one way that could fail: a leftover `.part`, a half-written file under the final
//! name, or a failed overwrite that destroys the previous complete file.
use super::*;

/// Throwaway directory, per test, in the OS temp dir (repo convention — no dev-dependency).
struct Tmp(std::path::PathBuf);
impl Tmp {
    fn new(tag: &str) -> Self {
        let d = std::env::temp_dir()
            .join(format!("fractadyne_atomic_test_{}_{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("temp dir");
        Self(d)
    }
    fn path(&self, name: &str) -> std::path::PathBuf {
        self.0.join(name)
    }
    fn names(&self) -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(&self.0)
            .expect("read dir")
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    }
}
impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn the_partial_name_is_the_final_name_plus_part() {
    let p = std::path::Path::new("out").join("tour_00042.png");
    assert_eq!(partial_path(&p), std::path::Path::new("out").join("tour_00042.png.part"));
}

/// A successful write leaves exactly the file — no `.part` beside it — and it reads back.
#[test]
fn a_finished_write_leaves_only_the_file() {
    let t = Tmp::new("finished");
    let (w, h) = (8u32, 4u32);
    let px = vec![0.25f32; (w * h * 4) as usize];
    write_png(&t.path("a.png"), w, h, &px, Some("app=fractadyne")).expect("write");
    write_png_rgba8(&t.path("b.png"), w, h, &vec![7u8; (w * h * 4) as usize], None).expect("write");
    assert_eq!(t.names(), vec!["a.png".to_string(), "b.png".to_string()]);
    assert_eq!(read_png_rgba8(&t.path("a.png")).expect("read").0, w);
    assert_eq!(read_png_metadata(&t.path("a.png")).expect("meta").as_deref(), Some("app=fractadyne"));
}

/// ⭐A write that fails part way must leave NOTHING under the final name — and when it was an
/// overwrite, the previous complete file must survive untouched. The old writer opened the final
/// name with `File::create`, which truncated the previous frame before the first byte of the new
/// one, and the caller then deleted whatever was left.
#[test]
fn a_failed_overwrite_keeps_the_previous_file_and_leaves_no_partial() {
    let t = Tmp::new("failed_overwrite");
    let (w, h) = (8u32, 4u32);
    let target = t.path("frame_00001.png");
    write_png(&target, w, h, &vec![0.5f32; (w * h * 4) as usize], None).expect("first write");
    let before = std::fs::read(&target).expect("read");

    let r = write_atomic(&target, |out| {
        out.write_all(b"\x89PNG\r\n\x1a\n half a frame")?;
        Err(ExportError::Io(std::io::Error::other("destination vanished")))
    });
    assert!(r.is_err(), "the injected failure must surface");
    assert_eq!(std::fs::read(&target).expect("read"), before, "the previous frame was damaged");
    assert_eq!(t.names(), vec!["frame_00001.png".to_string()], "a .part was left behind");
}

/// A failure AFTER the partial file is complete — here, the rename itself, onto a directory that
/// cannot be replaced by a file — still cleans up the partial and leaves the target alone.
#[test]
fn a_failed_rename_cleans_up_the_partial() {
    let t = Tmp::new("failed_rename");
    let target = t.path("taken.png");
    std::fs::create_dir(&target).expect("a directory where the file should go");
    let (w, h) = (4u32, 4u32);
    let r = write_png(&target, w, h, &vec![0.5f32; (w * h * 4) as usize], None);
    assert!(r.is_err(), "renaming a file over a directory must fail");
    assert!(target.is_dir(), "the directory in the way was disturbed");
    assert!(!partial_path(&target).exists(), "the .part was left behind");
}

/// The size check runs before anything touches the disk: no file, no partial.
#[test]
fn a_short_buffer_writes_nothing() {
    let t = Tmp::new("short");
    assert!(write_png(&t.path("s.png"), 8, 8, &[0.0; 16], None).is_err());
    assert!(t.names().is_empty(), "{:?}", t.names());
}
