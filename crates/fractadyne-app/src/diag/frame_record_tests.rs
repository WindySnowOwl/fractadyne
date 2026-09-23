use super::*;

/// The ring is process-global; tests that record into it take this first.
static SERIAL: Mutex<()> = Mutex::new(());

fn reset_ring() {
    let mut r = RING.lock().unwrap_or_else(|p| p.into_inner());
    r.buf.clear();
    r.next = 0;
}

/// A record whose every field holds a distinct, non-default value — built by walking `FIELDS`, so
/// a field added later is covered without touching this test. Bools alternate, so both values of
/// the one-byte encoding are exercised.
fn every_field_distinct() -> FrameRecord {
    let mut b = vec![0u8; FrameRecord::PAYLOAD_BYTES];
    let mut at = 0;
    for (i, (_, ty)) in FrameRecord::FIELDS.iter().enumerate() {
        let k = i as u64 + 1;
        match *ty {
            "bool" => (i % 2 == 0).put(&mut b, &mut at),
            "u8" => (k as u8).put(&mut b, &mut at),
            "u16" => (k as u16 * 257).put(&mut b, &mut at),
            "u32" => (k as u32 * 65_537 + 3).put(&mut b, &mut at),
            // Above 2^53: a JSON reader that parsed integers as doubles would lose these.
            "u64" => ((1u64 << 60) + k * 1_000_003).put(&mut b, &mut at),
            "f32" => (k as f32 * 1.5 + 0.25).put(&mut b, &mut at),
            "f64" => (k as f64 * 3.141_592_653_589_793 + 1e-9).put(&mut b, &mut at),
            other => panic!("FIELDS names a type the test does not know: {other}"),
        }
    }
    assert_eq!(at, FrameRecord::PAYLOAD_BYTES, "FIELDS disagrees with PAYLOAD_BYTES");
    let mut at = 0;
    FrameRecord::get_payload(&b, &mut at).expect("decodes")
}

#[test]
fn field_names_are_unique_and_the_record_fits_its_slot_with_room_to_grow() {
    let mut names: Vec<&str> = FrameRecord::FIELDS.iter().map(|(n, _)| *n).collect();
    names.sort_unstable();
    let before = names.len();
    names.dedup();
    assert_eq!(names.len(), before, "a field name is declared twice");
    assert!(SLOT_HEADER + FrameRecord::PAYLOAD_BYTES <= SLOT_BYTES);
    // Headroom is the point of an oversized slot: a schema addition must not re-key the file.
    assert!(
        SLOT_BYTES - SLOT_HEADER - FrameRecord::PAYLOAD_BYTES >= 32,
        "only {} bytes of slot headroom left — grow SLOT_BYTES deliberately",
        SLOT_BYTES - SLOT_HEADER - FrameRecord::PAYLOAD_BYTES
    );
}

/// A lossy round trip is a RED criterion of the W1 gate. Exact through the binary slot; exact
/// through JSON, every field, including u64s above 2^53 and floats to the last bit.
#[test]
fn every_field_round_trips_exactly_through_the_slot_and_the_json() {
    let r = every_field_distinct();
    let slot = encode_slot(&r, 42);
    assert_eq!(decode_slot(&slot, 42), Some(r), "binary round trip");

    let json = r.to_json();
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert_eq!(v.as_object().expect("an object").len(), FrameRecord::FIELDS.len(), "one key per field");
    // Re-encode what the JSON SAYS and decode it: equality proves every value survived.
    //
    // ⚠The values are read from the raw text with `str::parse`, NOT through serde_json: its
    // default float parser is best-effort, and measured here it mis-rounds 22 of 200 such f64s by
    // one ULP (exactness needs its `float_roundtrip` feature). The JSON is exact; a reader must be
    // too — Python's `float()` is, and the replay in W5 must not use serde_json's default.
    let body = json.trim_start_matches('{').trim_end_matches('}');
    let raw: std::collections::HashMap<&str, &str> = body
        .split(',')
        .map(|kv| {
            let (k, v) = kv.split_once(':').expect("key:value");
            (k.trim_matches('"'), v)
        })
        .collect();
    fn p<T: std::str::FromStr>(name: &str, x: &str) -> T {
        x.parse().unwrap_or_else(|_| panic!("{name}: {x:?} does not parse"))
    }
    let mut b = vec![0u8; FrameRecord::PAYLOAD_BYTES];
    let mut at = 0;
    for (name, ty) in FrameRecord::FIELDS {
        let x = *raw.get(name).unwrap_or_else(|| panic!("JSON lacks {name}"));
        match *ty {
            "bool" => p::<bool>(name, x).put(&mut b, &mut at),
            "u8" => p::<u8>(name, x).put(&mut b, &mut at),
            "u16" => p::<u16>(name, x).put(&mut b, &mut at),
            "u32" => p::<u32>(name, x).put(&mut b, &mut at),
            "u64" => p::<u64>(name, x).put(&mut b, &mut at),
            "f32" => p::<f32>(name, x).put(&mut b, &mut at),
            "f64" => p::<f64>(name, x).put(&mut b, &mut at),
            other => panic!("{other}"),
        }
    }
    let mut at = 0;
    assert_eq!(FrameRecord::get_payload(&b, &mut at), Some(r), "JSON round trip");
}

#[test]
fn json_writes_non_finite_floats_as_null_rather_than_invalid_json() {
    let r = FrameRecord { last_dt_ms: f64::NAN, boost: f64::INFINITY, norm_lo: f32::NAN, ..Default::default() };
    let v: serde_json::Value = serde_json::from_str(&r.to_json()).expect("still valid JSON");
    assert!(v["last_dt_ms"].is_null() && v["boost"].is_null() && v["norm_lo"].is_null());
}

/// A reader must never guess: each of these is a slot that is not this session's record.
#[test]
fn a_slot_is_refused_when_torn_foreign_or_malformed() {
    let r = every_field_distinct();
    let good = encode_slot(&r, 7);
    assert!(decode_slot(&good, 7).is_some());
    assert!(decode_slot(&good, 8).is_none(), "another session's slot");

    let mut torn = good;
    torn[SLOT_HEADER + 5] ^= 0x40;
    assert!(decode_slot(&torn, 7).is_none(), "a flipped payload byte fails the checksum");

    let mut schema = good;
    schema[4] = schema[4].wrapping_add(1);
    assert!(decode_slot(&schema, 7).is_none(), "another schema");

    let mut magic = good;
    magic[0] = b'X';
    assert!(decode_slot(&magic, 7).is_none(), "no magic");

    assert!(decode_slot(&[0u8; SLOT_BYTES], 7).is_none(), "an empty slot");

    // A bool byte of 2 with a matching checksum is still not a record.
    let bool_at = {
        let mut at = 0;
        let mut found = None;
        for (name, ty) in FrameRecord::FIELDS {
            if *ty == "bool" {
                found = Some(at);
                let _ = name;
                break;
            }
            at += match *ty {
                "u8" => 1,
                "u16" => 2,
                "u32" | "f32" => 4,
                "u64" | "f64" => 8,
                _ => unreachable!(),
            };
        }
        found.expect("the record has a bool")
    };
    let mut bad = good;
    bad[SLOT_HEADER + bool_at] = 2;
    let len = FrameRecord::PAYLOAD_BYTES;
    let sum = checksum(&bad[SLOT_HEADER..SLOT_HEADER + len]);
    bad[24..28].copy_from_slice(&sum.to_le_bytes());
    assert!(decode_slot(&bad, 7).is_none(), "a bool byte that is neither 0 nor 1");
}

/// The ring keeps the MOST RECENT records, in sequence order, and stays bounded — a crash report
/// carrying the first minutes of a session would describe the healthy start.
#[test]
fn the_ring_keeps_the_newest_records_in_order_and_is_bounded() {
    let _g = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    reset_ring();
    assert!(snapshot().is_empty());
    for f in 0..(RING_LEN as u64 + 100) {
        record(FrameRecord { frame: f, ..Default::default() });
    }
    let s = snapshot();
    assert_eq!(s.len(), RING_LEN);
    assert_eq!(s.first().unwrap().frame, 100, "the oldest 100 are gone");
    assert_eq!(s.last().unwrap().frame, RING_LEN as u64 + 99);
    assert!(s.windows(2).all(|w| w[1].seq == w[0].seq + 1), "no gap, no duplicate");
    assert!(s.iter().all(|r| r.schema == SCHEMA), "every record is stamped");
    reset_ring();
}

/// `frames.bin` is the abort backstop: slots written in place, read back by a later process.
/// Read back here through a SEPARATE handle while the writer's is still open and unflushed —
/// the state an abort leaves — and with a torn slot and another session's leftover in it.
#[test]
fn frames_bin_reads_back_this_sessions_records_counts_torn_slots_and_ignores_leftovers() {
    let dir = std::env::temp_dir().join(format!("fd-frames-{}", session_id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("frames.bin");
    let mut f = open_bin(&path).expect("open");
    for seq in 0..10u64 {
        let r = FrameRecord { seq, frame: 100 + seq, schema: SCHEMA, ..Default::default() };
        write_slot(&mut f, seq as usize, &encode_slot(&r, session_id())).unwrap();
    }
    // A leftover from another session at slot 20, and a torn slot of this session at slot 21.
    let old = FrameRecord { seq: 20, frame: 999, schema: SCHEMA, ..Default::default() };
    write_slot(&mut f, 20, &encode_slot(&old, session_id() ^ 1)).unwrap();
    let mut torn = encode_slot(&FrameRecord { seq: 21, ..Default::default() }, session_id());
    torn[SLOT_HEADER + 3] ^= 1;
    write_slot(&mut f, 21, &torn).unwrap();

    let got = read_bin(&path).expect("read").expect("our schema");
    assert_eq!(got.session, session_id());
    assert_eq!(got.records.len(), 10);
    assert_eq!(got.records.iter().map(|r| r.frame).collect::<Vec<_>>(), (100..110).collect::<Vec<_>>());
    assert_eq!(got.torn, 1, "the torn slot is reported, not silently dropped");
    drop(f);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_file_of_another_schema_is_refused_not_guessed_at() {
    let dir = std::env::temp_dir().join(format!("fd-frames-schema-{}", session_id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("frames.bin");
    let mut f = open_bin(&path).expect("open");
    f.seek(SeekFrom::Start(4)).unwrap();
    f.write_all(&(SCHEMA + 1).to_le_bytes()).unwrap();
    drop(f);
    assert!(read_bin(&path).expect("readable").is_none());
    let _ = std::fs::remove_dir_all(&dir);
}

/// The crash report must say which of "nothing recorded" and "these frames" holds, and carry the
/// counters with the render they describe — never as a bare per-frame number.
#[test]
fn the_crash_section_says_nothing_recorded_or_names_the_frames() {
    assert!(crash_section(&[], None).contains("no frame was recorded"));
    let rs: Vec<FrameRecord> = (0..50u64)
        .map(|i| FrameRecord {
            seq: i,
            frame: 1000 + i,
            t_ms: 5000 + 30 * i,
            plan_calls: 1,
            present: present::LIVE,
            read_n: (i == 49) as u8,
            read_src: read_src::GPU,
            read_ms: 12.0,
            read_verdict: verdict::DISCARDED,
            ctr_new: i == 48,
            ctr_tag: 1045,
            ctr_rebase: 33_000_000,
            ..Default::default()
        })
        .collect();
    let s = crash_section(&rs, Some("crash-1-0-frames.jsonl"));
    assert!(s.contains("50 records"), "{s}");
    assert!(s.contains("frames 1000..1049"), "{s}");
    assert!(s.contains("crash-1-0-frames.jsonl"), "names the companion:\n{s}");
    assert_eq!(s.lines().filter(|l| l.contains(" v0 ")).count(), CRASH_INLINE, "inline is bounded");
    assert!(s.contains("DISCARDED"), "the reading verdict survives");
    assert!(s.contains("ctr[tag 1045]: rebase=33000000"), "counters carry their render:\n{s}");
}

/// The committed schema file IS the encoding. A field change that does not regenerate it fails
/// here, before `framelog.py` can misread a real capture. Regenerate with
/// `fractadyne --dump-frame-schema > validation/frame-schema.json` (in bash — PowerShell `>` writes
/// UTF-16).
#[test]
fn the_committed_frame_schema_matches_the_code() {
    let committed = include_str!("../../../../validation/frame-schema.json").replace("\r\n", "\n");
    assert_eq!(
        committed,
        schema_json(),
        "validation/frame-schema.json is stale — regenerate it with --dump-frame-schema"
    );
    let v: serde_json::Value = serde_json::from_str(&committed).expect("valid JSON");
    assert_eq!(v["fields"].as_array().unwrap().len(), FrameRecord::FIELDS.len());
    assert_eq!(v["payload_bytes"], FrameRecord::PAYLOAD_BYTES);
}

#[test]
fn the_companion_jsonl_is_a_header_then_one_parseable_record_per_line() {
    let dir = std::env::temp_dir().join(format!("fd-frames-jsonl-{}", session_id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("x-frames.jsonl");
    let rs = vec![every_field_distinct(), FrameRecord { frame: 3, ..Default::default() }];
    write_jsonl(&path, "{\"adapter\":\"test\"}", &rs).unwrap();
    let text = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3);
    let h: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(h["kind"], "header");
    assert_eq!(h["schema"], SCHEMA);
    assert_eq!(h["header"]["adapter"], "test");
    for l in &lines[1..] {
        let v: serde_json::Value = serde_json::from_str(l).unwrap();
        assert_eq!(v.as_object().unwrap().len(), FrameRecord::FIELDS.len());
    }
    let _ = std::fs::remove_dir_all(&dir);
}
