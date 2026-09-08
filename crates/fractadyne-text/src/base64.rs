//! Standard base64 (RFC 4648, `+/` alphabet, `=` padded) — enough to carry a thumbnail inside a
//! text file, and nothing more.
//!
//! ⭐**Hand-rolled rather than a dependency**, because this is forty lines of table lookup against
//! a frozen specification, it is exercised by a round-trip test over every byte value, and the
//! alternative is a supply-chain edge on a public repo for something that will never change.
//!
//! ⚠**The `=` padding is safe in our `key=value` files** — the reader splits on the FIRST `=`, so
//! everything after the key's own separator is the value, padding included. That is a property of
//! the reader, not a coincidence, and the view-metadata tests pin it.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encode bytes as padded base64 on a single line (no line breaks — this rides inside one field).
pub fn encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { ALPHABET[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { ALPHABET[n as usize & 63] as char } else { '=' });
    }
    out
}

/// Decode padded base64. Returns `None` on any character outside the alphabet, or a bad length.
///
/// ⚠**Whitespace is skipped, everything else is refused.** A value that has been through an editor
/// may have picked up a stray space or a wrapped line; a value with a `!` in it is corrupt, and
/// decoding it to "nearly the right bytes" would hand a broken PNG to the image decoder instead of
/// saying so here.
pub fn decode(s: &str) -> Option<Vec<u8>> {
    let mut rev = [255u8; 256];
    for (i, c) in ALPHABET.iter().enumerate() {
        rev[*c as usize] = i as u8;
    }
    let mut acc: u32 = 0;
    let mut bits = 0u32;
    let mut pad = 0usize;
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    for c in s.chars() {
        if c.is_whitespace() {
            continue;
        }
        if c == '=' {
            pad += 1;
            // ⚠Padding is terminal: `A=A=` is not a thing.
            continue;
        }
        if pad > 0 || !c.is_ascii() {
            return None;
        }
        let v = rev[c as usize];
        if v == 255 {
            return None;
        }
        acc = (acc << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    if pad > 2 {
        return None;
    }
    // Leftover bits must be zero padding, not truncated data.
    if bits >= 6 || (acc & ((1 << bits) - 1)) != 0 {
        return None;
    }
    Some(out)
}
