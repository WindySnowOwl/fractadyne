use super::*;

const D: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

fn blob_of(id: u64, bytes: &[u8]) -> BlobAnnounce {
    BlobAnnounce { id, len: bytes.len() as u64, sha256: crate::sha256_hex(bytes) }
}

fn samples() -> Vec<Msg> {
    let policy = Policy { max_width: 7680, max_height: 4320, max_ss: 4, max_iter: 10_000_000 };
    vec![
        Msg::Hello(Hello {
            protocol: 1,
            app_version: "0.3.0-beta.18".into(),
            git: "gabc1234".into(),
            allow_dirty: false,
            name: "PLUTO".into(),
            tunables: "stock".into(),
            policy,
            clock_unix_ms: 1_790_000_000_000,
        }),
        Msg::HelloAck(HelloAck {
            protocol: 1,
            app_version: "0.3.0-beta.18".into(),
            git: "gabc1234".into(),
            name: "studio".into(),
            verdict: Verdict::Refused("version mismatch".into()),
            link_sample_bytes: 1 << 20,
            gpu: None,
        }),
        Msg::HelloAck(HelloAck {
            protocol: 1,
            app_version: "v".into(),
            git: "g".into(),
            name: "studio".into(),
            verdict: Verdict::Admitted,
            link_sample_bytes: 0,
            gpu: Some(GpuInfo { adapter: "NVIDIA GeForce RTX 3080 · Vulkan".into(), driver: "NVIDIA 581.42".into(), orbit_len_cap: 7_452_444 }),
        }),
        Msg::SelfCheck(SelfCheck {
            items: vec![CheckItem { name: "device".into(), ok: true, hard: true, detail: "RTX 3080".into() }],
            gpu: Some(GpuInfo { adapter: "NVIDIA GeForce RTX 3080".into(), driver: "NVIDIA 581.42".into(), orbit_len_cap: 7_452_444 }),
            free_bytes: Some(412 << 30),
            link_sample: Some(BlobAnnounce { id: 1, len: 1 << 20, sha256: D.into() }),
            probe: Some(BlobAnnounce { id: 2, len: 48_000, sha256: D.into() }),
        }),
        Msg::JobOpen(JobOpen { job_id: "a1b2c3d4e5f60718".into(), name: "Grand tour".into(), bundle: BlobAnnounce { id: 2, len: 9000, sha256: D.into() } }),
        Msg::Assign(Assign { job_id: "j".into(), run_id: 7, start: 120, end: 136 }),
        Msg::Cancel(Cancel { job_id: "j".into(), run_id: None, reason: CancelReason::Stalled, from: None }),
        Msg::Cancel(Cancel { job_id: "j".into(), run_id: Some(4), reason: CancelReason::Reassigned, from: Some(130) }),
        Msg::JobClose(JobClose { job_id: "j".into() }),
        Msg::Heartbeat(Heartbeat {
            activity: ClientActivity::Rendering,
            job_id: Some("j".into()),
            run_id: Some(7),
            frame: Some(123),
            frame_ms: Some(14_000),
            frames_done: 3,
            free_bytes: None,
        }),
        Msg::FrameDone(FrameDone { job_id: "j".into(), run_id: 7, index: 123, render_ms: 812, blob: BlobAnnounce { id: 3, len: 74101, sha256: D.into() }, on_share: false }),
        Msg::FrameDone(FrameDone { job_id: "j".into(), run_id: 7, index: 124, render_ms: 790, blob: BlobAnnounce { id: 4, len: 73010, sha256: D.into() }, on_share: true }),
        Msg::FrameFailed(FrameFailed { job_id: "j".into(), run_id: 7, index: 124, class: FailClass::Storage, message: "disk full".into() }),
        Msg::RunAborted(RunAborted { job_id: "j".into(), run_id: 7, done_up_to: Some(123), reason: AbortReason::ChildCrash(-1) }),
        Msg::RunAborted(RunAborted { job_id: "j".into(), run_id: 8, done_up_to: None, reason: AbortReason::UserCancel }),
        Msg::DiagRequest(DiagRequest { items: vec![DiagItem::ChildLog, DiagItem::Heartbeats] }),
        Msg::DiagReport(DiagReport { items: vec![(DiagItem::ChildLog, "tour FAILED: …\n".into())] }),
        Msg::Bye(Bye { reason: "removed: 2 frames failed verification".into() }),
        Msg::Keepalive,
    ]
}

#[test]
fn every_message_round_trips_and_passes_its_own_checks() {
    for m in samples() {
        let bytes = encode_control(&m).expect("encodes");
        assert_eq!(bytes[0], KIND_CONTROL);
        match decode(&bytes).unwrap_or_else(|e| panic!("{}: {e}", kind_name(&m))) {
            Incoming::Control(back) => assert_eq!(back, m),
            other => panic!("decoded as {other:?}"),
        }
    }
}

/// Each refusal is a field the network must never be trusted with.
#[test]
fn bad_fields_are_protocol_errors() {
    let bad = [
        // a path where a job id goes
        r#"{"kind":"assign","body":{"job_id":"../x","run_id":1,"start":0,"end":5}}"#,
        // an empty, a reversed and an enormous run
        r#"{"kind":"assign","body":{"job_id":"j","run_id":1,"start":5,"end":5}}"#,
        r#"{"kind":"assign","body":{"job_id":"j","run_id":1,"start":9,"end":5}}"#,
        r#"{"kind":"assign","body":{"job_id":"j","run_id":1,"start":0,"end":20000}}"#,
        // a blob over its cap, and a digest that is not one
        r#"{"kind":"frame_done","body":{"job_id":"j","run_id":1,"index":0,"render_ms":1,"blob":{"id":1,"len":999999999999,"sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"}}}"#,
        r#"{"kind":"frame_done","body":{"job_id":"j","run_id":1,"index":0,"render_ms":1,"blob":{"id":1,"len":10,"sha256":"E3B0"}}}"#,
        // an unknown field, an unknown kind
        r#"{"kind":"job_close","body":{"job_id":"j","path":"C:/Windows"}}"#,
        r#"{"kind":"run_command","body":{"cmd":"format c:"}}"#,
        // control characters in text that a UI will show
        r#"{"kind":"bye","body":{"reason":"\u001b[2J"}}"#,
        // a diagnostics request for nothing, or for too much
        r#"{"kind":"diag_request","body":{"items":[]}}"#,
        // a cut of no run in particular
        r#"{"kind":"cancel","body":{"job_id":"j","run_id":null,"reason":"reassigned","from":12}}"#,
        // a probe over its cap; a probe on the link sample's blob id
        r#"{"kind":"self_check","body":{"items":[],"gpu":null,"free_bytes":null,"link_sample":null,"probe":{"id":2,"len":99999999,"sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"}}}"#,
        r#"{"kind":"self_check","body":{"items":[],"gpu":null,"free_bytes":null,"link_sample":{"id":1,"len":10,"sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"},"probe":{"id":1,"len":10,"sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"}}}"#,
    ];
    for json in bad {
        let mut p = vec![KIND_CONTROL];
        p.extend_from_slice(json.as_bytes());
        assert!(decode(&p).is_err(), "accepted: {json}");
    }
}

#[test]
fn malformed_payloads_are_refused() {
    assert!(decode(&[]).is_err());
    assert!(decode(&[9, 1, 2]).is_err(), "unknown kind");
    assert!(decode(&[KIND_CHUNK, 1, 2, 3]).is_err(), "short chunk header");
    let mut huge = vec![KIND_CHUNK];
    huge.extend(std::iter::repeat_n(0u8, 16 + CHUNK_BYTES + 1));
    assert!(decode(&huge).is_err(), "oversized chunk");
    let mut big = vec![KIND_CONTROL];
    big.extend(std::iter::repeat_n(b' ', MAX_CONTROL_BYTES + 1));
    assert!(decode(&big).is_err(), "oversized control message");
}

#[test]
fn a_message_over_the_cap_is_not_sent() {
    let m = Msg::DiagReport(DiagReport {
        items: vec![(DiagItem::ChildLog, "\u{1}".repeat(15_000)), (DiagItem::Handshake, "x".repeat(16_000)), (DiagItem::Heartbeats, "\u{2}".repeat(16_000))],
    });
    // Control characters escape to six bytes each in JSON, so this is far over 64 KiB.
    assert!(encode_control(&m).is_err());
}

#[test]
fn chunks_round_trip_and_cover_a_blob_exactly() {
    let data: Vec<u8> = (0..(CHUNK_BYTES * 2 + 123)).map(|i| (i % 251) as u8).collect();
    let mut sink = BlobSink::new(blob_of(5, &data), Vec::new());
    let mut last = BlobProgress::More;
    for (off, c) in chunks(&data) {
        let wire = encode_chunk(5, off, c);
        match decode(&wire).expect("chunk decodes") {
            Incoming::Chunk { id, offset, data: d } => {
                assert_eq!((id, offset), (5, off));
                last = sink.push(offset, &d).expect("accepted");
            }
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(last, BlobProgress::Complete);
    assert_eq!(sink.into_inner(), data);
}

/// Out of order, past the end, or the wrong bytes: each is an error, never a "complete" blob.
#[test]
fn a_blob_must_arrive_in_order_whole_and_unaltered() {
    let data = vec![42u8; 1000];
    let mut s = BlobSink::new(blob_of(1, &data), Vec::new());
    assert!(matches!(s.push(10, &data[10..20]), Err(BlobError::Protocol(_))), "out of order");

    let mut s = BlobSink::new(blob_of(1, &data), Vec::new());
    assert_eq!(s.push(0, &data[..500]), Ok(BlobProgress::More));
    assert!(matches!(s.push(500, &[0u8; 600]), Err(BlobError::Protocol(_))), "overrun");

    // ⭐The tampered blob is a DIGEST error — the controller strikes the sender — not a protocol
    // error, which would only close the connection.
    let mut tampered = data.clone();
    tampered[999] = 43;
    let mut s = BlobSink::new(blob_of(1, &data), Vec::new());
    match s.push(0, &tampered) {
        Err(BlobError::Digest(e)) => assert!(e.contains("does not match"), "{e}"),
        other => panic!("expected a digest error, got {other:?}"),
    }
}

#[test]
fn a_log_is_truncated_to_its_tail_on_a_character_boundary() {
    let s = format!("{}é{}", "a".repeat(10), "z".repeat(10));
    let t = tail(&s, 11);
    assert!(t.len() <= 11 && t.ends_with("zzzz"), "{t}");
    assert_eq!(tail("short", 100), "short");
}
