//! NumPy `.npy` files of 32-bit floats: what `--render-lanes` writes each lane of a view's G-buffer
//! as (coloring plan B9), so a Python prototype reads the app's own numbers with `np.load` and no
//! other library. Format 1.0 (NumPy's `format.py`): the magic `\x93NUMPY`, the version, a
//! little-endian u16 header length, a Python dict literal padded with spaces to a 64-byte boundary
//! and ended by a newline, then the data, C order. The reader takes what the writer writes
//! (`'<f4'`, C order) in format 1.0 or 2.0 and refuses anything else by name.

use std::path::Path;

use crate::ExportError;

const MAGIC: &[u8; 6] = b"\x93NUMPY";

/// The header dict for little-endian f32 data of `shape`, C order: NumPy's own spelling (a 1-D
/// shape keeps its trailing comma), padded so the data starts on a 64-byte boundary.
fn header(shape: &[usize]) -> Vec<u8> {
    let dims = match shape {
        [n] => format!("({n},)"),
        _ => format!("({})", shape.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ")),
    };
    let mut h = format!("{{'descr': '<f4', 'fortran_order': False, 'shape': {dims}, }}");
    // magic (6) + version (2) + length (2) + header, a multiple of 64 with the newline last.
    let unpadded = 10 + h.len() + 1;
    h.push_str(&" ".repeat(unpadded.next_multiple_of(64) - unpadded));
    h.push('\n');
    h.into_bytes()
}

/// Write `data` (C order, `shape` the dimensions, outermost first) as a `.npy` file of `'<f4'`,
/// atomically as every other writer here is. A `data` of the wrong length is refused.
pub fn write_npy_f32(path: &Path, shape: &[usize], data: &[f32]) -> Result<(), ExportError> {
    let expected = shape.iter().product::<usize>();
    if data.len() != expected {
        return Err(ExportError::SizeMismatch { expected, got: data.len() });
    }
    let head = header(shape);
    let len = u16::try_from(head.len()).map_err(|_| ExportError::Npy(format!("a header of {} bytes", head.len())))?;
    crate::write_atomic(path, |w| {
        w.write_all(MAGIC)?;
        w.write_all(&[1, 0])?;
        w.write_all(&len.to_le_bytes())?;
        w.write_all(&head)?;
        // Little-endian on every target this builds for; spelled out so it stays true on any.
        let mut bytes = Vec::with_capacity(data.len() * 4);
        for v in data {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        w.write_all(&bytes)?;
        Ok(())
    })
}

/// The value of `key` in a `.npy` header dict, up to the next top-level comma or the closing brace
/// (a shape's tuple is kept whole).
fn field<'a>(dict: &'a str, key: &str) -> Option<&'a str> {
    let at = dict.find(&format!("'{key}':"))? + key.len() + 3;
    let rest = dict[at..].trim_start();
    let end = if rest.starts_with('(') { rest.find(')')? + 1 } else { rest.find([',', '}'])? };
    Some(rest[..end].trim())
}

/// Read a `.npy` file of `'<f4'` in C order: its shape and its values. Anything else (another
/// type, Fortran order, a short or long file) is an error saying which.
pub fn read_npy_f32(path: &Path) -> Result<(Vec<usize>, Vec<f32>), ExportError> {
    read_npy_f32_bytes(&std::fs::read(path)?)
}

/// [`read_npy_f32`] of a file's bytes.
pub fn read_npy_f32_bytes(bytes: &[u8]) -> Result<(Vec<usize>, Vec<f32>), ExportError> {
    let bad = |why: &str| ExportError::Npy(why.to_string());
    if bytes.len() < 10 || &bytes[..6] != MAGIC {
        return Err(bad("no NumPy magic"));
    }
    let (start, hlen) = match bytes[6] {
        1 => (10, u16::from_le_bytes([bytes[8], bytes[9]]) as usize),
        2 | 3 if bytes.len() >= 12 => (12, u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize),
        v => return Err(ExportError::Npy(format!("format version {v}"))),
    };
    let dict = bytes
        .get(start..start + hlen)
        .and_then(|h| std::str::from_utf8(h).ok())
        .ok_or_else(|| bad("a header past the end of the file"))?;
    let descr = field(dict, "descr").ok_or_else(|| bad("no descr"))?;
    if descr != "'<f4'" {
        return Err(ExportError::Npy(format!("type {descr}, not '<f4'")));
    }
    if field(dict, "fortran_order") != Some("False") {
        return Err(bad("Fortran order"));
    }
    let dims = field(dict, "shape").ok_or_else(|| bad("no shape"))?;
    let shape = dims
        .trim_matches(|c| c == '(' || c == ')')
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<usize>().map_err(|_| ExportError::Npy(format!("shape {dims}"))))
        .collect::<Result<Vec<_>, _>>()?;
    let n = shape
        .iter()
        .try_fold(1usize, |a, &d| a.checked_mul(d))
        .ok_or_else(|| ExportError::Npy(format!("shape {dims}")))?;
    let data = &bytes[start + hlen..];
    if Some(data.len()) != n.checked_mul(4) {
        return Err(ExportError::SizeMismatch { expected: n.saturating_mul(4), got: data.len() });
    }
    let values = data.chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect();
    Ok((shape, values))
}

#[cfg(test)]
#[path = "../tests/unit/npy/npy_tests.rs"]
mod npy_tests;
