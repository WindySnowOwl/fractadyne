use super::*;

#[test]
fn the_checksum_is_zips_crc32() {
    // The standard check value of CRC-32/ISO-HDLC.
    assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    assert_eq!(crc32(b""), 0);
}

/// Read a stored zip back through its own central directory: (name, data) per entry.
fn read_back(zip: &[u8]) -> Vec<(String, Vec<u8>)> {
    let u16_at = |i: usize| u16::from_le_bytes([zip[i], zip[i + 1]]);
    let u32_at = |i: usize| u32::from_le_bytes([zip[i], zip[i + 1], zip[i + 2], zip[i + 3]]);
    let eocd = zip.len() - 22;
    assert_eq!(u32_at(eocd), 0x0605_4B50, "no end-of-directory record");
    let count = u16_at(eocd + 10) as usize;
    let mut cd = u32_at(eocd + 16) as usize;
    let mut out = Vec::new();
    for _ in 0..count {
        assert_eq!(u32_at(cd), 0x0201_4B50, "not a directory entry");
        let crc = u32_at(cd + 16);
        let size = u32_at(cd + 20) as usize;
        let name_len = u16_at(cd + 28) as usize;
        let local = u32_at(cd + 42) as usize;
        let name = String::from_utf8(zip[cd + 46..cd + 46 + name_len].to_vec()).unwrap();
        assert_eq!(u32_at(local), 0x0403_4B50, "{name}: not a local header");
        let local_name_len = u16_at(local + 26) as usize;
        let data_at = local + 30 + local_name_len + u16_at(local + 28) as usize;
        let data = zip[data_at..data_at + size].to_vec();
        assert_eq!(crc32(&data), crc, "{name}: checksum");
        out.push((name, data));
        cd += 46 + name_len;
    }
    out
}

#[test]
fn an_archive_reads_back_entry_for_entry() {
    let mut z = Zip::new((2026, 11, 1, 21, 30, 15));
    z.add("README.txt", b"hello");
    z.add("logs/fractadyne.log", "a line with µ and ✓\n".as_bytes());
    z.add("empty", b"");
    let bytes = z.finish();
    let back = read_back(&bytes);
    assert_eq!(back.len(), 3);
    assert_eq!(back[0], ("README.txt".to_string(), b"hello".to_vec()));
    assert_eq!(back[1].0, "logs/fractadyne.log");
    assert_eq!(String::from_utf8(back[1].1.clone()).unwrap(), "a line with µ and ✓\n");
    assert_eq!(back[2], ("empty".to_string(), Vec::new()));
}

fn scratch(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("fractadyne-bundle-test-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn the_plan_takes_the_newest_results_and_the_logs_most_useful_first() {
    let diag = scratch("diag");
    let logs = scratch("logs");
    for (name, body) in [("selftest-1.md", "old"), ("gputest-1.txt", "g")] {
        std::fs::write(diag.join(name), body).unwrap();
    }
    std::thread::sleep(std::time::Duration::from_millis(20));
    std::fs::write(diag.join("selftest-2.md"), "new").unwrap();
    for name in ["fractadyne.log", "frames.jsonl", "frames.bin", "crash-9.txt", "crash-view-9.fdn", "fractadyne.log.1"] {
        std::fs::write(logs.join(name), name).unwrap();
    }
    let p = plan(Some(&diag), Some(&logs));
    let names: Vec<&str> = p.iter().map(|i| i.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "selftest.md",
            "gputest.txt",
            "logs/fractadyne.log",
            "logs/frames.jsonl",
            "logs/frames.bin",
            "logs/crash-9.txt",
            "logs/crash-view-9.fdn",
            "logs/fractadyne.log.1"
        ]
    );
    assert!(p[0].path.ends_with("selftest-2.md"), "{:?}", p[0].path);
    assert!(!p.iter().find(|i| i.name == "logs/frames.bin").unwrap().text);
    // Nothing to carry: an empty plan, not an error.
    assert!(plan(None, None).is_empty());
    let _ = std::fs::remove_dir_all(&diag);
    let _ = std::fs::remove_dir_all(&logs);
}

#[test]
fn the_bundle_redacts_text_keeps_binary_and_says_what_it_left_out() {
    let logs = scratch("build");
    let home = directories::BaseDirs::new().map(|b| b.home_dir().to_string_lossy().to_string()).unwrap_or_default();
    std::fs::write(logs.join("fractadyne.log"), format!("session: {home}\\AppData\\x\n")).unwrap();
    std::fs::write(logs.join("frames.bin"), [0u8, 1, 2, 255]).unwrap();
    let big = logs.join("fractadyne.log.1");
    std::fs::write(&big, vec![b'x'; MAX_BYTES + 1]).unwrap();
    let items = plan(None, Some(&logs));
    let bytes = build("OS: test\n", &items, "0.3.0-beta.24 (build 1, gabc)", (2026, 11, 1, 22, 0, 0));
    // For checking the archive with another reader (Python's zipfile, 2026-11-01).
    if let Ok(out) = std::env::var("FRACTADYNE_BUNDLE_TEST_OUT") {
        std::fs::write(out, &bytes).unwrap();
    }
    let back = read_back(&bytes);
    let get = |n: &str| back.iter().find(|e| e.0 == n).map(|e| e.1.clone());
    let log = String::from_utf8(get("logs/fractadyne.log").unwrap()).unwrap();
    if home.len() >= 4 {
        assert!(!log.contains(&home), "{log}");
        assert!(log.contains("~"), "{log}");
    }
    assert_eq!(get("logs/frames.bin").unwrap(), vec![0u8, 1, 2, 255]);
    assert_eq!(get("machine.txt").unwrap(), b"OS: test\r\n".to_vec());
    assert!(get("logs/fractadyne.log.1").is_none(), "the over-size log went in");
    let readme = String::from_utf8(get("README.txt").unwrap()).unwrap();
    assert!(readme.contains("logs/fractadyne.log.1") && readme.contains("over the bundle's size limit"), "{readme}");
    assert!(readme.contains("0.3.0-beta.24"), "{readme}");
    let _ = std::fs::remove_dir_all(&logs);
}
