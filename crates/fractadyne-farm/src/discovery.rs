//! Finding a controller on the local network (design/remote-rendering.md §12, Phase 4), so a client
//! can pick it from a list instead of typing its address. The farm key is still required: what is
//! found here only fills in an address, and the handshake decides everything else.
//!
//! A client sends a probe — [`PROBE_MAGIC`] padded to [`PROBE_LEN`] bytes — as a UDP broadcast to
//! [`DISCOVERY_PORT`] from a throw-away socket, and collects the unicast replies for a moment. A
//! listening controller answers each probe with [`REPLY_MAGIC`] and a small JSON [`Beacon`]: its
//! name, the TCP port it listens on, its build and its identity's fingerprint (all public: the
//! handshake shows them to anyone who connects). So:
//!
//! - the client still opens no port — and needs no firewall rule: Windows lets a unicast reply to a
//!   broadcast through by default;
//! - a reply is never larger than the probe that asked for it, and replies are rate-limited, so a
//!   probe with a forged source address cannot turn the controller into an amplifier;
//! - both sides parse fixed, size-bounded packets with every field checked, as the farm's control
//!   messages are.
//!
//! Not mDNS, which the design first named: on Windows its answers arrive as multicast to port 5353,
//! so every client would need a listening socket and a firewall rule, and it would add five crates.

use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// The UDP port a controller answers on — the farm's default TCP port, on UDP.
pub const DISCOVERY_PORT: u16 = 46733;
pub const PROBE_MAGIC: &[u8] = b"FDFARM?1";
pub const REPLY_MAGIC: &[u8] = b"FDFARM!1";
/// A probe's exact length: no reply may be longer, so a forged probe gains an attacker nothing. Room
/// for the longest valid beacon: a 64-character name at 4 bytes a character, three 64-byte fields.
pub const PROBE_LEN: usize = 640;
/// Replies a controller sends in any one second, at most.
pub const MAX_REPLIES_PER_S: u32 = 20;

/// What a controller says about itself.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Beacon {
    pub name: String,
    /// The TCP port its farm listens on.
    pub port: u16,
    pub app_version: String,
    pub git: String,
    /// Its identity's fingerprint, as both farm windows show it.
    pub identity: String,
}

/// A controller found on the network: where it answered from, and what it said.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub from: SocketAddr,
    pub beacon: Beacon,
}

impl Found {
    /// The address a client dials: the replying host, the farm's TCP port.
    pub fn address(&self) -> String {
        SocketAddr::new(self.from.ip(), self.beacon.port).to_string()
    }
}

pub fn probe_packet() -> Vec<u8> {
    let mut p = vec![0u8; PROBE_LEN];
    p[..PROBE_MAGIC.len()].copy_from_slice(PROBE_MAGIC);
    p
}

/// Exactly a probe: the right length, starting with the magic.
pub fn is_probe(buf: &[u8]) -> bool {
    buf.len() == PROBE_LEN && buf.starts_with(PROBE_MAGIC)
}

fn short(what: &str, s: &str) -> Result<(), String> {
    if s.is_empty() || s.len() > 64 || s.chars().any(char::is_control) {
        return Err(format!("{what}: 1 to 64 bytes of printable text"));
    }
    Ok(())
}

fn check(b: &Beacon) -> Result<(), String> {
    crate::names::check_display_name(&b.name)?;
    short("version", &b.app_version)?;
    short("build", &b.git)?;
    short("identity", &b.identity)?;
    if b.port == 0 {
        return Err("port 0".into());
    }
    Ok(())
}

/// A reply: the magic and the beacon, never longer than a probe.
pub fn reply_packet(b: &Beacon) -> Result<Vec<u8>, String> {
    check(b)?;
    let mut p = REPLY_MAGIC.to_vec();
    p.extend(serde_json::to_vec(b).map_err(|e| e.to_string())?);
    if p.len() > PROBE_LEN {
        return Err(format!("a {}-byte reply is longer than a probe", p.len()));
    }
    Ok(p)
}

/// A reply read back, every field checked.
pub fn parse_reply(buf: &[u8]) -> Result<Beacon, String> {
    if buf.len() > PROBE_LEN {
        return Err("longer than a probe".into());
    }
    let body = buf.strip_prefix(REPLY_MAGIC).ok_or("not a farm reply")?;
    let b: Beacon = serde_json::from_slice(body).map_err(|e| e.to_string())?;
    check(&b)?;
    Ok(b)
}

/// Answer probes on `socket` until `stop` is set: each with `beacon`, at most
/// [`MAX_REPLIES_PER_S`] a second. Returns the replies sent.
pub fn serve(socket: &UdpSocket, beacon: &Beacon, stop: &AtomicBool) -> Result<u64, String> {
    let reply = reply_packet(beacon)?;
    socket.set_read_timeout(Some(Duration::from_millis(250))).map_err(|e| e.to_string())?;
    let mut buf = [0u8; PROBE_LEN + 1];
    let (mut window, mut in_window, mut sent) = (Instant::now(), 0u32, 0u64);
    while !stop.load(Ordering::Relaxed) {
        let Ok((n, from)) = socket.recv_from(&mut buf) else { continue };
        if !is_probe(&buf[..n]) {
            continue;
        }
        if window.elapsed() >= Duration::from_secs(1) {
            (window, in_window) = (Instant::now(), 0);
        }
        if in_window >= MAX_REPLIES_PER_S {
            continue;
        }
        in_window += 1;
        if socket.send_to(&reply, from).is_ok() {
            sent += 1;
        }
    }
    Ok(sent)
}

/// Ask the network (and this machine) which controllers are listening, on `port`, collecting the
/// replies for `wait`. One entry per controller (identity and port), in the order they answered —
/// at its network address when it answered on one as well as on loopback.
pub fn discover_on(port: u16, wait: Duration) -> Result<Vec<Found>, String> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).map_err(|e| e.to_string())?;
    socket.set_broadcast(true).map_err(|e| e.to_string())?;
    let probe = probe_packet();
    // The broadcast reaches the network; loopback finds a controller on this machine (also one
    // listening on 127.0.0.1 only, which no broadcast reaches).
    let mut sent = false;
    for to in [Ipv4Addr::BROADCAST, Ipv4Addr::LOCALHOST] {
        sent |= socket.send_to(&probe, (to, port)).is_ok();
    }
    if !sent {
        return Err("could not send on this network".into());
    }
    let deadline = Instant::now() + wait;
    let mut found: Vec<Found> = Vec::new();
    let mut buf = [0u8; PROBE_LEN + 1];
    while let Some(left) = deadline.checked_duration_since(Instant::now()).filter(|d| !d.is_zero()) {
        socket.set_read_timeout(Some(left)).map_err(|e| e.to_string())?;
        let Ok((n, from)) = socket.recv_from(&mut buf) else { break };
        if let Ok(beacon) = parse_reply(&buf[..n]) {
            let f = Found { from, beacon };
            match found.iter_mut().find(|g| g.beacon.identity == f.beacon.identity && g.beacon.port == f.beacon.port) {
                Some(g) if g.from.ip().is_loopback() && !f.from.ip().is_loopback() => *g = f,
                Some(_) => {}
                None => found.push(f),
            }
        }
    }
    Ok(found)
}

/// [`discover_on`] the farm's port.
pub fn discover(wait: Duration) -> Result<Vec<Found>, String> {
    discover_on(DISCOVERY_PORT, wait)
}

#[cfg(test)]
mod tests;
