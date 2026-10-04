use super::*;
use std::sync::Arc;

fn beacon() -> Beacon {
    Beacon { name: "STUDIO-PC".into(), port: 46733, app_version: "0.3.0-beta.18".into(), git: "g8a71de8".into(), identity: "3879-75ad-6509-e65a".into() }
}

#[test]
fn a_probe_is_exactly_its_length_and_magic() {
    let p = probe_packet();
    assert!(is_probe(&p));
    assert!(!is_probe(&p[..PROBE_LEN - 1]), "short");
    let mut long = p.clone();
    long.push(0);
    assert!(!is_probe(&long), "long");
    let mut wrong = p.clone();
    wrong[0] = b'X';
    assert!(!is_probe(&wrong), "wrong magic");
    assert!(!is_probe(&reply_packet(&beacon()).expect("a reply")), "a reply is not a probe");
}

#[test]
fn a_reply_round_trips_and_is_never_longer_than_a_probe() {
    let b = beacon();
    let r = reply_packet(&b).expect("a reply");
    assert!(r.len() <= PROBE_LEN);
    assert_eq!(parse_reply(&r), Ok(b));
    // The longest valid beacon: a 64-character name of 4-byte characters, 64-byte fields.
    let longest = Beacon { name: "𝔉".repeat(64), port: 65535, app_version: "v".repeat(64), git: "g".repeat(64), identity: "i".repeat(64) };
    let r = reply_packet(&longest).expect("the longest valid beacon fits");
    assert!(r.len() <= PROBE_LEN, "{} bytes", r.len());
    assert_eq!(parse_reply(&r), Ok(longest));
}

#[test]
fn a_reply_with_anything_wrong_is_refused() {
    let ok = reply_packet(&beacon()).expect("a reply");
    let body = |j: &str| [REPLY_MAGIC, j.as_bytes()].concat();
    assert!(parse_reply(&ok[1..]).is_err(), "no magic");
    assert!(parse_reply(&body("{")).is_err(), "not JSON");
    assert!(parse_reply(&body(r#"{"name":"A","port":1,"app_version":"v","git":"g","identity":"i","extra":1}"#)).is_err(), "an unknown field");
    assert!(parse_reply(&body(r#"{"name":"","port":1,"app_version":"v","git":"g","identity":"i"}"#)).is_err(), "an empty name");
    assert!(parse_reply(&body(r#"{"name":"A","port":0,"app_version":"v","git":"g","identity":"i"}"#)).is_err(), "port 0");
    assert!(parse_reply(&body("{\"name\":\"A\\u0007\",\"port\":1,\"app_version\":\"v\",\"git\":\"g\",\"identity\":\"i\"}")).is_err(), "a control character");
    let mut huge = ok.clone();
    huge.resize(PROBE_LEN + 1, b' ');
    assert!(parse_reply(&huge).is_err(), "longer than a probe");
    assert!(reply_packet(&Beacon { git: "g".repeat(65), ..beacon() }).is_err(), "a field past its bound is not sent either");
}

/// A controller answering on loopback, found by a client probing the same (test) port.
#[test]
fn a_controller_answering_on_loopback_is_found() {
    let sock = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).expect("a socket");
    let port = sock.local_addr().expect("its address").port();
    let stop = Arc::new(AtomicBool::new(false));
    let (b, s) = (beacon(), stop.clone());
    let server = std::thread::spawn(move || serve(&sock, &b, &s, |_, _| {}));
    let found = discover_on(port, Duration::from_millis(600)).expect("discovery runs");
    stop.store(true, Ordering::Relaxed);
    let sent = server.join().expect("the server thread").expect("it served");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].beacon, beacon());
    assert_eq!(found[0].address(), "127.0.0.1:46733", "the replying host, the farm's TCP port");
    assert!(sent >= 1);
}

/// Probes past the rate limit go unanswered — a flood cannot make the controller a firehose.
#[test]
fn replies_are_rate_limited() {
    let sock = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).expect("a socket");
    let port = sock.local_addr().expect("its address").port();
    let stop = Arc::new(AtomicBool::new(false));
    let (b, s) = (beacon(), stop.clone());
    let server = std::thread::spawn(move || serve(&sock, &b, &s, |_, _| {}));
    let flood = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).expect("a socket");
    for _ in 0..(MAX_REPLIES_PER_S * 3) {
        let _ = flood.send_to(&probe_packet(), (Ipv4Addr::LOCALHOST, port));
    }
    std::thread::sleep(Duration::from_millis(400));
    stop.store(true, Ordering::Relaxed);
    let sent = server.join().expect("the server thread").expect("it served");
    assert!(sent >= 1 && sent <= u64::from(MAX_REPLIES_PER_S), "{sent} replies to {} probes inside a second", MAX_REPLIES_PER_S * 3);
}
