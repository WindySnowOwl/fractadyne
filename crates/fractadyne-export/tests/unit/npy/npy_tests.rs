//! `.npy` files (coloring plan B9): what the writer writes, the reader reads back bit for bit, and
//! NumPy's own rules hold for it (the data on a 64-byte boundary, the header ended by a newline). A
//! file NumPy wrote is read too, and what the reader does not take is refused by name.
use super::*;

/// Throwaway directory, per test, in the OS temp dir (repo convention — no dev-dependency).
fn tmp(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("fractadyne_npy_test_{}_{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("temp dir");
    d
}

#[test]
fn a_plane_round_trips_bit_for_bit() {
    let d = tmp("plane");
    let path = d.join("count.npy");
    // The values a lane holds: interior's -1, a NaN, infinities, the trap's 1e30, a denormal.
    let data = [-1.0, 0.0, -0.0, 1.5e-42, f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 1.0e30, 123456.78, 0.25, -2.0, 3.0];
    write_npy_f32(&path, &[3, 4], &data).expect("write");
    let (shape, back) = read_npy_f32(&path).expect("read");
    assert_eq!(shape, vec![3, 4]);
    assert_eq!(back.len(), data.len());
    for (a, b) in data.iter().zip(&back) {
        assert_eq!(a.to_bits(), b.to_bits(), "{a} came back as {b}");
    }
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn the_header_is_numpys_own() {
    let d = tmp("header");
    for (shape, dims) in [(vec![270usize, 480], "(270, 480)"), (vec![256], "(256,)"), (vec![256, 4], "(256, 4)")] {
        let path = d.join("x.npy");
        let n: usize = shape.iter().product();
        write_npy_f32(&path, &shape, &vec![0.5; n]).expect("write");
        let bytes = std::fs::read(&path).expect("bytes");
        assert_eq!(&bytes[..8], b"\x93NUMPY\x01\x00");
        let hlen = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
        assert_eq!((10 + hlen) % 64, 0, "the data must start on a 64-byte boundary");
        assert_eq!(bytes[10 + hlen - 1], b'\n', "the header ends with a newline");
        let dict = std::str::from_utf8(&bytes[10..10 + hlen]).expect("ascii");
        assert!(
            dict.starts_with(&format!("{{'descr': '<f4', 'fortran_order': False, 'shape': {dims}, }}")),
            "{dict}"
        );
        assert_eq!(bytes.len(), 10 + hlen + n * 4);
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// A file as NumPy 2 writes it (`np.save` of `np.arange(6, dtype='<f4').reshape(2, 3)`), byte for
/// byte, header padding included.
#[test]
fn a_file_numpy_wrote_is_read() {
    let mut bytes = b"\x93NUMPY\x01\x00v\x00".to_vec();
    let dict = "{'descr': '<f4', 'fortran_order': False, 'shape': (2, 3), }";
    bytes.extend_from_slice(dict.as_bytes());
    bytes.extend(std::iter::repeat_n(b' ', 0x76 - dict.len() - 1));
    bytes.push(b'\n');
    for v in 0..6 {
        bytes.extend_from_slice(&(v as f32).to_le_bytes());
    }
    let (shape, data) = read_npy_f32_bytes(&bytes).expect("read");
    assert_eq!(shape, vec![2, 3]);
    assert_eq!(data, vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]);
}

#[test]
fn what_it_does_not_read_is_refused_by_name() {
    let save = |dict: &str, data: usize| {
        let mut h = dict.to_string();
        let unpadded = 10 + h.len() + 1;
        h.push_str(&" ".repeat(unpadded.next_multiple_of(64) - unpadded));
        h.push('\n');
        let mut b = b"\x93NUMPY\x01\x00".to_vec();
        b.extend_from_slice(&(h.len() as u16).to_le_bytes());
        b.extend_from_slice(h.as_bytes());
        b.extend(std::iter::repeat_n(0u8, data));
        b
    };
    let why = |b: &[u8]| read_npy_f32_bytes(b).expect_err("refused").to_string();
    assert!(why(b"PK\x03\x04 not numpy").contains("magic"));
    assert!(why(&save("{'descr': '<f8', 'fortran_order': False, 'shape': (2,), }", 16)).contains("'<f8'"));
    assert!(why(&save("{'descr': '<f4', 'fortran_order': True, 'shape': (2, 2), }", 16)).contains("Fortran"));
    // One value short, and one too many.
    assert!(why(&save("{'descr': '<f4', 'fortran_order': False, 'shape': (2, 2), }", 12)).contains("expected 16"));
    assert!(why(&save("{'descr': '<f4', 'fortran_order': False, 'shape': (2, 2), }", 20)).contains("expected 16"));
    // A shape whose size overflows is refused, not allocated.
    assert!(why(&save("{'descr': '<f4', 'fortran_order': False, 'shape': (18446744073709551615, 2), }", 0)).contains("shape"));
}

#[test]
fn the_writer_refuses_data_that_is_not_the_shape() {
    let d = tmp("short");
    let err = write_npy_f32(&d.join("x.npy"), &[2, 3], &[0.0; 5]).expect_err("refused");
    assert!(matches!(err, ExportError::SizeMismatch { expected: 6, got: 5 }), "{err}");
    assert!(!d.join("x.npy").exists());
    let _ = std::fs::remove_dir_all(&d);
}
