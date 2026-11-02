//! ⭐**THE DIAGNOSTICS BUNDLE** (Help ▸ Diagnostics ▸ Save diagnostics bundle…, 2026-11-01): one
//! file a tester attaches once, holding everything the next question about their machine would ask
//! for: the machine profile (`crate::machine`), the last self-test and GPU-test results, the log and
//! the one before it, the frame record, and the newest crash reports with their views.
//!
//! A zip with every entry STORED (no compression: the logs are small, and the format stays a few
//! dozen lines with no new dependency). Text goes through [`crate::machine::redact`]: the user's
//! home path reads `~` and the computer's name `<computer>`. The total stays under GitHub's 25 MB
//! attachment limit; what did not fit is listed in the bundle's README.

use std::path::{Path, PathBuf};

/// The most the bundle carries, bytes: under GitHub's 25 MB attachment limit.
pub(crate) const MAX_BYTES: usize = 24 << 20;

/// CRC-32 (IEEE), as zip stores it.
pub(crate) fn crc32(data: &[u8]) -> u32 {
    static TABLE: std::sync::OnceLock<[u32; 256]> = std::sync::OnceLock::new();
    let t = TABLE.get_or_init(|| {
        let mut t = [0u32; 256];
        for (i, slot) in t.iter_mut().enumerate() {
            let mut c = i as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            }
            *slot = c;
        }
        t
    });
    let mut c = 0xFFFF_FFFFu32;
    for &b in data {
        c = t[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
}

/// A zip archive built in memory, every entry stored.
pub(crate) struct Zip {
    out: Vec<u8>,
    central: Vec<u8>,
    count: u16,
    /// MS-DOS time and date, as the headers want them.
    dos: (u16, u16),
}

impl Zip {
    /// An empty archive whose entries are dated `(year, month, day, hour, minute, second)`.
    pub(crate) fn new(when: (i32, u32, u32, u32, u32, u32)) -> Self {
        let (y, mo, d, h, mi, s) = when;
        let date = (((y.clamp(1980, 2107) - 1980) as u16) << 9) | ((mo as u16 & 0xF) << 5) | (d as u16 & 0x1F);
        let time = ((h as u16 & 0x1F) << 11) | ((mi as u16 & 0x3F) << 5) | ((s / 2) as u16 & 0x1F);
        Zip { out: Vec::new(), central: Vec::new(), count: 0, dos: (time, date) }
    }

    /// Add `name` (UTF-8, `/` between folders) holding `data`.
    pub(crate) fn add(&mut self, name: &str, data: &[u8]) {
        let crc = crc32(data);
        let size = data.len() as u32;
        let offset = self.out.len() as u32;
        let name_b = name.as_bytes();
        let le16 = |v: &mut Vec<u8>, x: u16| v.extend_from_slice(&x.to_le_bytes());
        let le32 = |v: &mut Vec<u8>, x: u32| v.extend_from_slice(&x.to_le_bytes());
        // Local file header.
        let o = &mut self.out;
        le32(o, 0x0403_4B50);
        le16(o, 20); // version needed
        le16(o, 0x0800); // the name is UTF-8
        le16(o, 0); // stored
        le16(o, self.dos.0);
        le16(o, self.dos.1);
        le32(o, crc);
        le32(o, size);
        le32(o, size);
        le16(o, name_b.len() as u16);
        le16(o, 0);
        o.extend_from_slice(name_b);
        o.extend_from_slice(data);
        // Its central directory entry.
        let c = &mut self.central;
        le32(c, 0x0201_4B50);
        le16(c, 20); // made by
        le16(c, 20); // needed
        le16(c, 0x0800);
        le16(c, 0);
        le16(c, self.dos.0);
        le16(c, self.dos.1);
        le32(c, crc);
        le32(c, size);
        le32(c, size);
        le16(c, name_b.len() as u16);
        le16(c, 0); // extra
        le16(c, 0); // comment
        le16(c, 0); // disk
        le16(c, 0); // internal attributes
        le32(c, 0); // external attributes
        le32(c, offset);
        c.extend_from_slice(name_b);
        self.count += 1;
    }

    /// The archive's bytes.
    pub(crate) fn finish(mut self) -> Vec<u8> {
        let cd_offset = self.out.len() as u32;
        let cd_size = self.central.len() as u32;
        self.out.extend_from_slice(&self.central);
        let o = &mut self.out;
        o.extend_from_slice(&0x0605_4B50u32.to_le_bytes());
        o.extend_from_slice(&0u16.to_le_bytes()); // this disk
        o.extend_from_slice(&0u16.to_le_bytes()); // the disk the directory starts on
        o.extend_from_slice(&self.count.to_le_bytes());
        o.extend_from_slice(&self.count.to_le_bytes());
        o.extend_from_slice(&cd_size.to_le_bytes());
        o.extend_from_slice(&cd_offset.to_le_bytes());
        o.extend_from_slice(&0u16.to_le_bytes()); // comment
        self.out
    }
}

/// The newest files in `dir` whose names start with `prefix` and end with `suffix`, newest first.
pub(crate) fn newest(dir: &Path, prefix: &str, suffix: &str, n: usize) -> Vec<PathBuf> {
    let Ok(rd) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut v: Vec<(std::time::SystemTime, PathBuf)> = rd
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            (name.starts_with(prefix) && name.ends_with(suffix))
                .then(|| (e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH), e.path()))
        })
        .collect();
    v.sort_by(|a, b| b.0.cmp(&a.0));
    v.into_iter().take(n).map(|(_, p)| p).collect()
}

/// One file the bundle may carry: where it goes in the zip, where it comes from, whether it is text
/// (redacted) or binary (as is).
pub(crate) struct Item {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) text: bool,
}

/// What the bundle would carry, most useful first (so a cut drops the least useful): the newest
/// self-test and GPU-test results in `diagnostics`, then from `logs` the log, the frame record,
/// the newest crash reports and views, the newest crash's frame record, and the log before this one.
pub(crate) fn plan(diagnostics: Option<&Path>, logs: Option<&Path>) -> Vec<Item> {
    let mut v = Vec::new();
    let mut push = |name: String, path: PathBuf, text: bool| {
        if path.is_file() {
            v.push(Item { name, path, text });
        }
    };
    if let Some(d) = diagnostics {
        if let Some(p) = newest(d, "selftest-", ".md", 1).into_iter().next() {
            push("selftest.md".into(), p, true);
        }
        if let Some(p) = newest(d, "gputest-", ".txt", 1).into_iter().next() {
            push("gputest.txt".into(), p, true);
        }
    }
    if let Some(l) = logs {
        push("logs/fractadyne.log".into(), l.join("fractadyne.log"), true);
        push("logs/frames.jsonl".into(), l.join("frames.jsonl"), true);
        push("logs/frames.bin".into(), l.join("frames.bin"), false);
        for p in newest(l, "crash-", ".txt", 3) {
            let name = format!("logs/{}", p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
            push(name, p, true);
        }
        for p in newest(l, "crash-view-", ".fdn", 3) {
            let name = format!("logs/{}", p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
            push(name, p, true);
        }
        if let Some(p) = newest(l, "crash-", "-frames.jsonl", 1).into_iter().next() {
            let name = format!("logs/{}", p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
            push(name, p, true);
        }
        push("logs/fractadyne.log.1".into(), l.join("fractadyne.log.1"), true);
    }
    v
}

/// Build the bundle: a README, `machine.txt` (`profile`), then the `items` in order while they fit
/// [`MAX_BYTES`]. `version` and `when` go in the README.
pub(crate) fn build(profile: &str, items: &[Item], version: &str, when: (i32, u32, u32, u32, u32, u32)) -> Vec<u8> {
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    let mut left_out = Vec::new();
    let mut total = profile.len();
    for it in items {
        let Ok(bytes) = std::fs::read(&it.path) else {
            left_out.push(format!("{} (could not be read)", it.name));
            continue;
        };
        let bytes = if it.text { crate::machine::redact(&String::from_utf8_lossy(&bytes)).into_bytes() } else { bytes };
        if total + bytes.len() > MAX_BYTES {
            left_out.push(format!("{} ({} KB, over the bundle's size limit)", it.name, bytes.len() / 1024));
            continue;
        }
        total += bytes.len();
        entries.push((it.name.clone(), bytes));
    }
    let (y, mo, d, h, mi, s) = when;
    let mut readme = format!(
        "Fractadyne diagnostics bundle\r\n{version}\r\nmade {y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02} (local time)\r\n\r\n\
         machine.txt  the machine: OS, CPU, memory, graphics adapters and drivers, Windows' GPU settings,\r\n\
         \x20            display-driver resets in the event log\r\n\
         selftest.md  the last self-test (Help > Diagnostics), with the GPU's speed in every mode\r\n\
         gputest.txt  the last GPU arithmetic test\r\n\
         logs/        the log, the log before it, the frame record, the newest crash reports\r\n\r\n\
         The user's home folder reads as ~ and the computer's name as <computer> in every text file.\r\n"
    );
    if left_out.is_empty() {
        readme.push_str("Nothing was left out.\r\n");
    } else {
        readme.push_str("Left out:\r\n");
        for l in &left_out {
            readme.push_str(&format!("  {l}\r\n"));
        }
    }
    let mut zip = Zip::new(when);
    zip.add("README.txt", readme.as_bytes());
    zip.add("machine.txt", profile.replace('\n', "\r\n").as_bytes());
    for (name, bytes) in &entries {
        zip.add(name, bytes);
    }
    zip.finish()
}

#[cfg(test)]
#[path = "../../tests/unit/diag/bundle/bundle_tests.rs"]
mod bundle_tests;
