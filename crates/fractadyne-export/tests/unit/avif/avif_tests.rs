//! The AVIF writer and its metadata reader. No AV1 decoder ships (see the module note), so the
//! picture is held by its container: an AVIF brand, the frame's size, full color resolution (4:4:4)
//! in its AV1 configuration, a bigger file at a higher quality; and the view's metadata exactly.
use super::*;

/// Throwaway directory, per test, in the OS temp dir (repo convention).
struct Tmp(std::path::PathBuf);
impl Tmp {
    fn new(tag: &str) -> Self {
        let d = std::env::temp_dir().join(format!("fractadyne_avif_test_{}_{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("temp dir");
        Self(d)
    }
    fn path(&self, name: &str) -> std::path::PathBuf {
        self.0.join(name)
    }
}
impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A ramp and a fine two-hue pattern, as the JPEG tests use.
fn picture(w: u32, h: u32) -> Vec<f32> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let (r, g, b) = if x < w / 2 {
                let t = x as f32 / (w / 2) as f32;
                (0.1 + 0.7 * t, 0.2 + 0.3 * t, 0.6 - 0.4 * t)
            } else if (x + y) % 3 == 0 {
                (0.9, 0.3, 0.1)
            } else {
                (0.1, 0.4, 0.8)
            };
            v.extend_from_slice(&[r, g, b, 1.0]);
        }
    }
    v
}

/// The body of the first box of type `typ` anywhere in the file (a plain search: the test files hold
/// no other bytes that spell a box type and its size before it).
fn first_box<'a>(file: &'a [u8], typ: &[u8; 4]) -> &'a [u8] {
    let at = file.windows(4).position(|w| w == typ).expect("the box");
    let size = u32::from_be_bytes(file[at - 4..at].try_into().unwrap()) as usize;
    &file[at + 4..at - 4 + size]
}

#[test]
fn an_avif_is_full_color_resolution_at_its_size_and_grows_with_quality() {
    let t = Tmp::new("structure");
    let (w, h) = (96u32, 64u32);
    let px = picture(w, h);
    let mut last = 0u64;
    for q in [30u8, 60, 90] {
        let p = t.path(&format!("q{q}.avif"));
        write_avif(&p, w, h, &px, None, q).expect("written");
        let file = std::fs::read(&p).unwrap();
        assert_eq!(&file[4..8], b"ftyp");
        assert!(first_box(&file, b"ftyp").chunks(4).any(|b| b == b"avif"), "an AVIF brand");
        // ispe (a full box): version and flags, then the width and height.
        let ispe = first_box(&file, b"ispe");
        assert_eq!((be(&ispe[4..8]), be(&ispe[8..12])), (w as u64, h as u64), "quality {q}: the size");
        // av1C: the third byte's bits 3 and 2 are chroma_subsampling_x and _y, both 0 at 4:4:4.
        let av1c = first_box(&file, b"av1C");
        assert_eq!(av1c[2] & 0b0000_1100, 0, "quality {q}: subsampled ({:08b})", av1c[2]);
        let size = file.len() as u64;
        assert!(size > last, "quality {q}: {size} bytes, not more than {last}");
        last = size;
    }
}

#[test]
fn the_view_metadata_round_trips_exactly() {
    let t = Tmp::new("meta");
    let (w, h) = (32u32, 16u32);
    let px = picture(w, h);
    let deep = format!("app=Fractadyne\ncenter_re=-1.{}\nnotes=ÄÖ ✓\n", "9".repeat(200_000));
    for (name, text) in [("small", "app=Fractadyne\nzoom=8\n".to_string()), ("deep", deep)] {
        let p = t.path(&format!("{name}.avif"));
        write_avif(&p, w, h, &px, Some(&text), 80).expect("written");
        assert_eq!(read_avif_metadata(&p).expect("read").as_deref(), Some(text.as_str()), "{name}");
    }
    let p = t.path("none.avif");
    write_avif(&p, w, h, &px, None, 80).unwrap();
    assert_eq!(read_avif_metadata(&p).unwrap(), None, "written without metadata");
    let q = t.path("not.avif");
    std::fs::write(&q, b"\x89PNG\r\n\x1a\n........").unwrap();
    assert!(read_avif_metadata(&q).is_err(), "not an AVIF");
}

#[test]
fn the_description_tag_reads_back_from_either_byte_order() {
    let text = b"app=Fractadyne\nx=1\n";
    assert_eq!(tiff_description(&tiff_with_description(text)).unwrap().as_deref(), Some(&text[..]));
    // The same block big-endian, by hand: a reader of another writer's Exif must take it.
    let mut mm = Vec::new();
    mm.extend_from_slice(b"MM\0\x2a");
    mm.extend_from_slice(&8u32.to_be_bytes());
    mm.extend_from_slice(&1u16.to_be_bytes());
    mm.extend_from_slice(&0x010Eu16.to_be_bytes());
    mm.extend_from_slice(&2u16.to_be_bytes());
    mm.extend_from_slice(&(text.len() as u32 + 1).to_be_bytes());
    mm.extend_from_slice(&26u32.to_be_bytes());
    mm.extend_from_slice(&0u32.to_be_bytes());
    mm.extend_from_slice(text);
    mm.push(0);
    assert_eq!(tiff_description(&mm).unwrap().as_deref(), Some(&text[..]));
    // Another tag only: no description, not an error; a cut block: an error.
    let mut other = tiff_with_description(text);
    other[10] = 0x0F; // tag 0x010F, Make
    assert_eq!(tiff_description(&other).unwrap(), None);
    assert!(tiff_description(&tiff_with_description(text)[..20]).is_err());
}

#[test]
fn a_side_past_the_limit_is_refused_and_there_is_no_thumbnail() {
    let t = Tmp::new("limits");
    let p = t.path("wide.avif");
    let e = write_avif(&p, AVIF_MAX_SIDE + 1, 1, &vec![0.0; (AVIF_MAX_SIDE as usize + 1) * 4], None, 80).unwrap_err();
    assert!(e.to_string().contains("AVIF width"), "{e}");
    assert!(!p.exists());
    let q = t.path("pic.avif");
    write_avif(&q, 16, 8, &picture(16, 8), None, 80).unwrap();
    assert!(matches!(crate::read_thumbnail(&q, 10), Err(ExportError::UnsupportedFormat(_))), "no AV1 decoder ships");
}
