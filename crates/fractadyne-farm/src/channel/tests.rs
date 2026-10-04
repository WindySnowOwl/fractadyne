use super::*;
use crate::proto::{Bye, Incoming, Msg};
use std::net::{Ipv4Addr, TcpListener};

/// Run `server` on an accepted loopback connection while `client` dials; return both results.
fn pair<S, C, A, B>(server: S, client: C) -> (A, B)
where
    S: FnOnce(TcpStream) -> A + Send + 'static,
    C: FnOnce(TcpStream) -> B,
    A: Send + 'static,
{
    let l = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = l.local_addr().expect("addr");
    let h = std::thread::spawn(move || server(l.accept().expect("accept").0));
    let b = client(TcpStream::connect(addr).expect("connect"));
    (h.join().expect("server thread"), b)
}

/// ⭐Two machines holding the same farm key complete the handshake, each learns the OTHER's
/// identity, and traffic flows both ways — control messages and a multi-chunk blob.
#[test]
fn same_key_connects_and_messages_flow_both_ways() {
    let key = FarmKey::generate().unwrap();
    let k2 = key.clone();
    let (ctrl_id, client_id) = (Identity::generate().unwrap(), Identity::generate().unwrap());
    let (ctrl_pub, client_pub) = (ctrl_id.public().to_vec(), client_id.public().to_vec());
    let blob: Vec<u8> = (0..200_000u32).map(|i| (i * 7) as u8).collect();
    let blob2 = blob.clone();
    let (server, client) = pair(
        move |s| {
            let sess = respond(s, &k2, &ctrl_id).expect("respond");
            let seen = sess.remote_static.clone();
            let (mut r, mut w) = sess.split(Some(Duration::from_secs(5)), Duration::from_secs(5)).unwrap();
            let first = r.recv().expect("hello");
            let mut got = Vec::new();
            while got.len() < blob2.len() {
                match r.recv().expect("chunk") {
                    Incoming::Chunk { offset, data, .. } => {
                        assert_eq!(offset as usize, got.len());
                        got.extend(data);
                    }
                    other => panic!("{other:?}"),
                }
            }
            w.send(&Msg::Bye(Bye { reason: "bye".into() })).unwrap();
            (seen, first, got)
        },
        move |s| {
            let sess = initiate(s, &key, &client_id).expect("initiate");
            let seen = sess.remote_static.clone();
            let (mut r, mut w) = sess.split(Some(Duration::from_secs(5)), Duration::from_secs(5)).unwrap();
            w.send(&Msg::Keepalive).unwrap();
            w.send_blob(9, &blob).unwrap();
            (seen, r.recv().expect("reply"))
        },
    );
    let (server_saw, first, got) = server;
    let (client_saw, reply) = client;
    assert_eq!(server_saw, client_pub, "the controller learned the wrong identity");
    assert_eq!(client_saw, ctrl_pub, "the client learned the wrong identity");
    assert_eq!(first, Incoming::Control(Msg::Keepalive));
    assert_eq!(got, blob_ref(), "the blob arrived altered");
    assert_eq!(reply, Incoming::Control(Msg::Bye(Bye { reason: "bye".into() })));
}

fn blob_ref() -> Vec<u8> {
    (0..200_000u32).map(|i| (i * 7) as u8).collect()
}

/// ⛔A client with a different farm key never gets a session; the controller says why.
#[test]
fn a_different_key_is_refused_at_the_handshake() {
    let (ctrl_key, wrong_key) = (FarmKey::generate().unwrap(), FarmKey::generate().unwrap());
    let (server, client) = pair(
        move |s| respond(s, &ctrl_key, &Identity::generate().unwrap()).map(|_| ()),
        move |s| {
            let sess = initiate(s, &wrong_key, &Identity::generate().unwrap());
            // The initiator only finds out when the controller drops the connection.
            sess.and_then(|s| {
                let (mut r, _w) = s.split(Some(Duration::from_secs(5)), Duration::from_secs(5))?;
                r.recv().map(|_| ()).map_err(|e| e.to_string())
            })
        },
    );
    let e = server.expect_err("the controller accepted a wrong key");
    assert!(e.contains("does not hold this farm's key"), "{e}");
    assert!(client.is_err(), "the client believed it was connected");
}

/// A peer that sends garbage instead of a handshake is dropped, not crashed on.
#[test]
fn garbage_instead_of_a_handshake_is_an_error() {
    let key = FarmKey::generate().unwrap();
    let (server, ()) = pair(
        move |s| respond(s, &key, &Identity::generate().unwrap()).map(|_| ()),
        |mut s| {
            let _ = s.write_all(&[0x00, 0x05, 1, 2, 3, 4, 5]);
            let _ = s.shutdown(std::net::Shutdown::Write);
        },
    );
    assert!(server.is_err());
}

#[test]
fn the_rate_limiter_allows_a_burst_then_cools_down() {
    let mut rl = RateLimiter::new(3, Duration::from_secs(10), Duration::from_secs(60));
    let ip: std::net::IpAddr = Ipv4Addr::new(192, 168, 1, 31).into();
    let other: std::net::IpAddr = Ipv4Addr::new(192, 168, 1, 32).into();
    let t0 = Instant::now();
    assert!(rl.admit(ip, t0));
    assert!(rl.admit(ip, t0 + Duration::from_secs(1)));
    assert!(rl.admit(ip, t0 + Duration::from_secs(2)));
    assert!(!rl.admit(ip, t0 + Duration::from_secs(3)), "a fourth attempt in 10 s was admitted");
    assert!(rl.admit(other, t0 + Duration::from_secs(3)), "one address's cooldown blocked another");
    assert!(!rl.admit(ip, t0 + Duration::from_secs(50)), "admitted during the cooldown");
    assert!(rl.admit(ip, t0 + Duration::from_secs(64)), "still refused after the cooldown");
    // Spaced attempts never trip it.
    let mut rl = RateLimiter::default();
    for i in 0..20 {
        assert!(rl.admit(ip, t0 + Duration::from_secs(6 * i)), "attempt {i}");
    }
}

#[test]
fn pins_are_trusted_on_first_use_and_a_changed_key_is_reported() {
    let dir = std::env::temp_dir().join(format!("fractadyne_farm_pin_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("pins.toml");
    let mut p = PinStore::load(&path).expect("no file yet is fine");
    assert_eq!(p.check("PLUTO (RX 6800 XT)", "aaaa-bbbb-cccc-dddd"), Pin::New);
    p.pin("PLUTO (RX 6800 XT)", "aaaa-bbbb-cccc-dddd").unwrap();
    let p = PinStore::load(&path).expect("reloads");
    assert_eq!(p.check("PLUTO (RX 6800 XT)", "aaaa-bbbb-cccc-dddd"), Pin::Known);
    assert_eq!(
        p.check("PLUTO (RX 6800 XT)", "eeee-ffff-0000-1111"),
        Pin::Changed { pinned: "aaaa-bbbb-cccc-dddd".into() }
    );
    let _ = std::fs::remove_dir_all(&dir);
}
