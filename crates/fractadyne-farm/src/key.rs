//! The farm key and each machine's identity.
//!
//! **The farm key** is 256 random bits, generated once by the controller and pasted into every
//! client. It is the Noise pre-shared key: a party without it cannot complete a handshake. It is
//! shown as `fdn1-` and eleven groups of five base32 letters, the last of them a checksum, so a
//! mistyped key is caught as "this key has a typo" — not, much later, as "the controller refused
//! you", which would send someone hunting a network problem that does not exist.
//!
//! **An identity** is a machine's static X25519 key pair, made once per install. Its public half's
//! fingerprint is what each side pins after the first handshake (`channel::PinStore`).

use zeroize::Zeroize;

/// Prefix of a written farm key: format 1.
const KEY_PREFIX: &str = "fdn1-";
/// Key bytes plus the two checksum bytes that are encoded with them.
const KEY_LEN: usize = 32;
const CHECK_LEN: usize = 2;
const ALPHABET: &[u8; 32] = b"abcdefghijklmnopqrstuvwxyz234567";

/// The farm key. Zeroed when dropped; never printed by `Debug`.
#[derive(Clone, PartialEq, Eq)]
pub struct FarmKey([u8; KEY_LEN]);

impl std::fmt::Debug for FarmKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("FarmKey(…)")
    }
}

impl Drop for FarmKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl FarmKey {
    /// A fresh random key.
    pub fn generate() -> Result<Self, String> {
        let mut k = [0u8; KEY_LEN];
        ring::rand::SecureRandom::fill(&ring::rand::SystemRandom::new(), &mut k)
            .map_err(|_| "the system random number generator failed".to_string())?;
        Ok(Self(k))
    }

    pub fn bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }

    #[cfg(test)]
    pub(crate) fn from_bytes(b: [u8; KEY_LEN]) -> Self {
        Self(b)
    }

    /// `fdn1-xxxxx-xxxxx-…`: the key and a 2-byte checksum, base32, in groups of five.
    pub fn to_text(&self) -> String {
        let mut raw = self.0.to_vec();
        raw.extend_from_slice(&checksum(&self.0));
        let letters = base32(&raw);
        raw.zeroize();
        let groups: Vec<&str> = letters.as_bytes().chunks(5).map(|c| std::str::from_utf8(c).unwrap_or("")).collect();
        format!("{KEY_PREFIX}{}", groups.join("-"))
    }

    /// Read a key as a person typed or pasted it: case, spaces, dashes and line breaks do not
    /// matter; a wrong letter is reported as a typo by the checksum.
    pub fn from_text(s: &str) -> Result<Self, String> {
        let t: String = s.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_ascii_lowercase();
        let body = t.strip_prefix(KEY_PREFIX).ok_or("a farm key starts with \"fdn1-\"")?;
        let letters: String = body.chars().filter(|c| *c != '-').collect();
        let (raw, canonical) = unbase32(&letters).ok_or("a farm key uses only the letters a–z and digits 2–7")?;
        if !canonical {
            // The last letter carries padding bits a written key always leaves at zero.
            return Err("this farm key has a typo (its last letter is not one the controller writes) — copy it again from the controller".into());
        }
        if raw.len() != KEY_LEN + CHECK_LEN {
            return Err(format!("this farm key is the wrong length ({} letters; a key has 55)", letters.len()));
        }
        let mut k = [0u8; KEY_LEN];
        k.copy_from_slice(&raw[..KEY_LEN]);
        if checksum(&k) != raw[KEY_LEN..] {
            k.zeroize();
            return Err("this farm key has a typo (its check letters do not match) — copy it again from the controller".into());
        }
        Ok(Self(k))
    }
}

fn checksum(k: &[u8]) -> [u8; CHECK_LEN] {
    let d = ring::digest::digest(&ring::digest::SHA256, k);
    [d.as_ref()[0], d.as_ref()[1]]
}

fn base32(bytes: &[u8]) -> String {
    let mut out = String::new();
    let (mut acc, mut bits) = (0u32, 0u32);
    for &b in bytes {
        acc = (acc << 8) | b as u32;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(ALPHABET[((acc >> bits) & 31) as usize] as char);
        }
    }
    if bits > 0 {
        out.push(ALPHABET[((acc << (5 - bits)) & 31) as usize] as char);
    }
    out
}

/// Decode base32. The flag is false when the leftover bits after the last whole byte are not zero —
/// text no encoder writes, so a typo in the final letter.
fn unbase32(s: &str) -> Option<(Vec<u8>, bool)> {
    let mut out = Vec::new();
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in s.bytes() {
        let v = ALPHABET.iter().position(|&a| a == c)? as u32;
        acc = (acc << 5) | v;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
        acc &= (1 << bits) - 1;
    }
    Some((out, acc == 0))
}

/// A machine's static key pair. The private half is zeroed when dropped and never printed.
pub struct Identity {
    private: Vec<u8>,
    public: Vec<u8>,
}

impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Identity({})", fingerprint(&self.public))
    }
}

impl Drop for Identity {
    fn drop(&mut self) {
        self.private.zeroize();
    }
}

impl Identity {
    /// A fresh key pair.
    pub fn generate() -> Result<Self, String> {
        let kp = crate::channel::builder()?
            .generate_keypair()
            .map_err(|e| format!("could not generate a key pair: {e}"))?;
        Ok(Self { private: kp.private, public: kp.public })
    }

    pub fn public(&self) -> &[u8] {
        &self.public
    }

    pub(crate) fn private(&self) -> &[u8] {
        &self.private
    }

    pub fn fingerprint(&self) -> String {
        fingerprint(&self.public)
    }

    /// The identity file's text: two hex lines. The file lives in the user's own config folder.
    pub fn to_file_text(&self) -> String {
        format!(
            "# Fractadyne render-farm identity. Private: do not share or copy to another machine.\nprivate = \"{}\"\npublic = \"{}\"\n",
            crate::hex(&self.private),
            crate::hex(&self.public)
        )
    }

    pub fn from_file_text(s: &str) -> Result<Self, String> {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct F {
            private: String,
            public: String,
        }
        let f: F = toml::from_str(s).map_err(|e| format!("identity file: {e}"))?;
        let private = unhex(&f.private).filter(|v| v.len() == 32).ok_or("identity file: bad private key")?;
        let public = unhex(&f.public).filter(|v| v.len() == 32).ok_or("identity file: bad public key")?;
        Ok(Self { private, public })
    }

    /// Load the identity at `path`, or make one and save it there (temp-then-rename).
    pub fn load_or_create(path: &std::path::Path) -> Result<Self, String> {
        match std::fs::read_to_string(path) {
            Ok(s) => Self::from_file_text(&s).map_err(|e| format!("{}: {e}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let id = Self::generate()?;
                if let Some(dir) = path.parent() {
                    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
                }
                write_atomic(path, id.to_file_text().as_bytes())?;
                Ok(id)
            }
            Err(e) => Err(format!("{}: {e}", path.display())),
        }
    }
}

/// The short, human-comparable form of a public key: 16 hex digits of its SHA-256, in fours.
pub fn fingerprint(public: &[u8]) -> String {
    let h = crate::sha256_hex(public);
    format!("{}-{}-{}-{}", &h[0..4], &h[4..8], &h[8..12], &h[12..16])
}

pub(crate) fn unhex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok()).collect()
}

/// Write `bytes` to `path` so that the file is either the old one or the complete new one.
pub(crate) fn write_atomic(path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".part");
    let tmp = std::path::PathBuf::from(tmp);
    std::fs::write(&tmp, bytes)
        .and_then(|()| std::fs::rename(&tmp, path))
        .map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            format!("{}: {e}", path.display())
        })
}

#[cfg(test)]
mod tests;
