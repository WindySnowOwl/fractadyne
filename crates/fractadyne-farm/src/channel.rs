//! The encrypted channel: `Noise_XXpsk3_25519_ChaChaPoly_SHA256` over TCP (design §3).
//!
//! - The client dials and is the Noise initiator; the controller listens and responds.
//! - XX exchanges both static keys inside the encrypted handshake; `psk3` mixes the farm key into
//!   the third message, so a party without it cannot complete a handshake. (A connecting party
//!   without the key does learn the controller's static PUBLIC key from message two — public by
//!   definition — and nothing else.)
//! - After the first handshake each side pins the other's key ([`PinStore`]); a later handshake
//!   with a different key under the same name is refused — "this machine changed identity".
//! - Every transport message is length-prefixed (`u16`, big-endian) on the socket and carries one
//!   [`crate::proto`] payload. The session is stateless-nonce Noise, so a connection's reader and
//!   writer run on separate threads without a lock: each counts its own direction's nonces.
//! - The controller rate-limits handshakes per source address ([`RateLimiter`]).

use crate::key::{FarmKey, Identity};
use crate::proto::{self, Incoming, Msg};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub const NOISE_PARAMS: &str = "Noise_XXpsk3_25519_ChaChaPoly_SHA256";
/// A Noise message's limit, tag included.
pub const MAX_NOISE_MSG: usize = 65_535;
/// The whole handshake must finish within this.
pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

/// A Noise builder on ring's cipher, hash and RNG, with curve25519-dalek for X25519.
pub(crate) fn builder<'a>() -> Result<snow::Builder<'a>, String> {
    use snow::resolvers::{DefaultResolver, FallbackResolver, RingResolver};
    let params: snow::params::NoiseParams = NOISE_PARAMS.parse().map_err(|e| format!("noise params: {e:?}"))?;
    Ok(snow::Builder::with_resolver(
        params,
        Box::new(FallbackResolver::new(Box::new(RingResolver), Box::new(DefaultResolver))),
    ))
}

fn write_frame(w: &mut impl Write, msg: &[u8]) -> std::io::Result<()> {
    let len = u16::try_from(msg.len()).map_err(|_| std::io::Error::other("noise message over 65535 bytes"))?;
    let mut out = Vec::with_capacity(2 + msg.len());
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(msg);
    w.write_all(&out)
}

fn read_frame(r: &mut impl Read, buf: &mut Vec<u8>) -> std::io::Result<()> {
    let mut len = [0u8; 2];
    r.read_exact(&mut len)?;
    let n = u16::from_be_bytes(len) as usize;
    buf.resize(n, 0);
    r.read_exact(buf)
}

/// An established, authenticated connection, not yet split into its reader and writer.
pub struct Session {
    stream: TcpStream,
    transport: Arc<snow::StatelessTransportState>,
    /// The peer's static public key (its identity), as proven by the handshake.
    pub remote_static: Vec<u8>,
}

fn noise_err(stage: &str, e: snow::Error) -> String {
    format!("handshake failed at {stage}: {e}")
}

/// Client side: dial has happened; run the handshake as initiator.
pub fn initiate(stream: TcpStream, key: &FarmKey, me: &Identity) -> Result<Session, String> {
    stream.set_read_timeout(Some(HANDSHAKE_TIMEOUT)).map_err(|e| e.to_string())?;
    stream.set_write_timeout(Some(HANDSHAKE_TIMEOUT)).map_err(|e| e.to_string())?;
    stream.set_nodelay(true).ok();
    let mut hs = builder()?
        .local_private_key(me.private())
        .and_then(|b| b.psk(3, key.bytes()))
        .and_then(|b| b.build_initiator())
        .map_err(|e| noise_err("setup", e))?;
    let (mut buf, mut msg) = (vec![0u8; MAX_NOISE_MSG], Vec::new());
    let mut s = &stream;
    let n = hs.write_message(&[], &mut buf).map_err(|e| noise_err("message 1", e))?;
    write_frame(&mut s, &buf[..n]).map_err(|e| format!("could not reach the controller: {e}"))?;
    read_frame(&mut s, &mut msg).map_err(|e| format!("the controller did not answer the handshake: {e}"))?;
    hs.read_message(&msg, &mut buf).map_err(|e| noise_err("message 2", e))?;
    let n = hs.write_message(&[], &mut buf).map_err(|e| noise_err("message 3", e))?;
    write_frame(&mut s, &buf[..n]).map_err(|e| format!("handshake: {e}"))?;
    finish(stream, hs)
}

/// Controller side: a client connected; run the handshake as responder. A client without the farm
/// key fails here, at message three, and the connection is dropped.
pub fn respond(stream: TcpStream, key: &FarmKey, me: &Identity) -> Result<Session, String> {
    stream.set_read_timeout(Some(HANDSHAKE_TIMEOUT)).map_err(|e| e.to_string())?;
    stream.set_write_timeout(Some(HANDSHAKE_TIMEOUT)).map_err(|e| e.to_string())?;
    stream.set_nodelay(true).ok();
    let mut hs = builder()?
        .local_private_key(me.private())
        .and_then(|b| b.psk(3, key.bytes()))
        .and_then(|b| b.build_responder())
        .map_err(|e| noise_err("setup", e))?;
    let (mut buf, mut msg) = (vec![0u8; MAX_NOISE_MSG], Vec::new());
    let mut s = &stream;
    read_frame(&mut s, &mut msg).map_err(|e| format!("handshake: {e}"))?;
    hs.read_message(&msg, &mut buf).map_err(|e| noise_err("message 1", e))?;
    let n = hs.write_message(&[], &mut buf).map_err(|e| noise_err("message 2", e))?;
    write_frame(&mut s, &buf[..n]).map_err(|e| format!("handshake: {e}"))?;
    read_frame(&mut s, &mut msg).map_err(|e| format!("handshake: {e}"))?;
    hs.read_message(&msg, &mut buf)
        .map_err(|_| "the connecting machine does not hold this farm's key (handshake message 3 did not authenticate)".to_string())?;
    finish(stream, hs)
}

fn finish(stream: TcpStream, hs: snow::HandshakeState) -> Result<Session, String> {
    let remote_static = hs.get_remote_static().map(<[u8]>::to_vec).ok_or("handshake gave no peer key")?;
    let transport = hs.into_stateless_transport_mode().map_err(|e| noise_err("transport", e))?;
    stream.set_read_timeout(None).map_err(|e| e.to_string())?;
    Ok(Session { stream, transport: Arc::new(transport), remote_static })
}

impl Session {
    /// The reading and writing halves, for two threads. `read_timeout`: how long the reader waits
    /// for the next message before reporting [`RecvError::TimedOut`] (`None` = forever).
    pub fn split(self, read_timeout: Option<Duration>, write_timeout: Duration) -> Result<(Reader, Writer), String> {
        self.stream.set_read_timeout(read_timeout).map_err(|e| e.to_string())?;
        self.stream.set_write_timeout(Some(write_timeout)).map_err(|e| e.to_string())?;
        let w = self.stream.try_clone().map_err(|e| e.to_string())?;
        Ok((
            Reader { stream: self.stream, transport: self.transport.clone(), nonce: 0, frame: Vec::new(), plain: vec![0u8; MAX_NOISE_MSG] },
            Writer { stream: w, transport: self.transport, nonce: 0, buf: vec![0u8; MAX_NOISE_MSG] },
        ))
    }

    pub fn peer_addr(&self) -> Option<std::net::SocketAddr> {
        self.stream.peer_addr().ok()
    }
}

/// Why a read ended.
#[derive(Debug, PartialEq, Eq)]
pub enum RecvError {
    /// The peer closed the connection.
    Closed,
    /// Nothing arrived within the read timeout.
    TimedOut,
    /// A message that did not decrypt, decode or validate. The connection must close.
    Protocol(String),
    /// The network failed.
    Io(String),
}

impl std::fmt::Display for RecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecvError::Closed => f.write_str("the connection was closed"),
            RecvError::TimedOut => f.write_str("nothing arrived in time"),
            RecvError::Protocol(e) => write!(f, "protocol error: {e}"),
            RecvError::Io(e) => write!(f, "network error: {e}"),
        }
    }
}

pub struct Reader {
    stream: TcpStream,
    transport: Arc<snow::StatelessTransportState>,
    nonce: u64,
    frame: Vec<u8>,
    plain: Vec<u8>,
}

impl Reader {
    /// The next message, decrypted, decoded and validated.
    pub fn recv(&mut self) -> Result<Incoming, RecvError> {
        let mut s = &self.stream;
        if let Err(e) = read_frame(&mut s, &mut self.frame) {
            return Err(match e.kind() {
                std::io::ErrorKind::UnexpectedEof | std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted => RecvError::Closed,
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut => RecvError::TimedOut,
                _ => RecvError::Io(e.to_string()),
            });
        }
        let n = self
            .transport
            .read_message(self.nonce, &self.frame, &mut self.plain)
            .map_err(|e| RecvError::Protocol(format!("a message did not decrypt: {e}")))?;
        self.nonce += 1;
        proto::decode(&self.plain[..n]).map_err(RecvError::Protocol)
    }

    /// Close both directions (unblocks a writer too).
    pub fn shutdown(&self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }
}

pub struct Writer {
    stream: TcpStream,
    transport: Arc<snow::StatelessTransportState>,
    nonce: u64,
    buf: Vec<u8>,
}

impl Writer {
    fn send_payload(&mut self, payload: &[u8]) -> Result<u64, String> {
        let n = self
            .transport
            .write_message(self.nonce, payload, &mut self.buf)
            .map_err(|e| format!("encrypt: {e}"))?;
        self.nonce += 1;
        let mut s = &self.stream;
        write_frame(&mut s, &self.buf[..n]).map_err(|e| format!("send: {e}"))?;
        Ok(n as u64 + 2)
    }

    /// Send a control message. Returns the bytes put on the wire (for the link metrics).
    pub fn send(&mut self, m: &Msg) -> Result<u64, String> {
        let p = proto::encode_control(m)?;
        self.send_payload(&p)
    }

    /// Send a whole blob as chunks, after its announcement has been sent.
    pub fn send_blob(&mut self, id: u64, bytes: &[u8]) -> Result<u64, String> {
        let mut total = 0;
        for (off, c) in proto::chunks(bytes) {
            total += self.send_payload(&proto::encode_chunk(id, off, c))?;
        }
        Ok(total)
    }

    pub fn shutdown(&self) {
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }
}

/// Handshake admission per source address: at most `burst` attempts in `window`, then refused for
/// `cooldown`. Pure (the caller passes the time), so the policy is pinned by test.
pub struct RateLimiter {
    burst: usize,
    window: Duration,
    cooldown: Duration,
    seen: std::collections::HashMap<std::net::IpAddr, (std::collections::VecDeque<Instant>, Option<Instant>)>,
}

impl Default for RateLimiter {
    /// The design's policy: 3 handshakes per 10 s per address, then 60 s refused.
    fn default() -> Self {
        Self::new(3, Duration::from_secs(10), Duration::from_secs(60))
    }
}

impl RateLimiter {
    pub fn new(burst: usize, window: Duration, cooldown: Duration) -> Self {
        Self { burst, window, cooldown, seen: Default::default() }
    }

    /// May `ip` attempt a handshake at `now`?
    pub fn admit(&mut self, ip: std::net::IpAddr, now: Instant) -> bool {
        let (times, until) = self.seen.entry(ip).or_default();
        if let Some(t) = *until {
            if now < t {
                return false;
            }
            *until = None;
            times.clear();
        }
        while times.front().is_some_and(|&t| now.duration_since(t) > self.window) {
            times.pop_front();
        }
        if times.len() >= self.burst {
            *until = Some(now + self.cooldown);
            return false;
        }
        times.push_back(now);
        // Bound the table: forget addresses idle for a long time.
        if self.seen.len() > 4096 {
            let horizon = self.window + self.cooldown;
            self.seen.retain(|_, (t, u)| u.is_some_and(|u| u > now) || t.back().is_some_and(|&b| now.duration_since(b) < horizon));
        }
        true
    }
}

/// The keys this machine has pinned: a name (a client's, or a controller's address) → fingerprint.
pub struct PinStore {
    path: std::path::PathBuf,
    pins: std::collections::BTreeMap<String, String>,
}

/// What a pin check found.
#[derive(Debug, PartialEq, Eq)]
pub enum Pin {
    /// Never seen: pin it on success (trust on first use, with the farm key as the gate).
    New,
    /// Seen, and the key is the same.
    Known,
    /// Seen with a DIFFERENT key — refuse, and say so.
    Changed { pinned: String },
}

impl PinStore {
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        #[derive(serde::Deserialize, Default)]
        #[serde(deny_unknown_fields)]
        struct F {
            #[serde(default)]
            pins: std::collections::BTreeMap<String, String>,
        }
        let pins = match std::fs::read_to_string(path) {
            Ok(s) => toml::from_str::<F>(&s).map_err(|e| format!("{}: {e}", path.display()))?.pins,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Default::default(),
            Err(e) => return Err(format!("{}: {e}", path.display())),
        };
        Ok(Self { path: path.to_path_buf(), pins })
    }

    pub fn check(&self, who: &str, fingerprint: &str) -> Pin {
        match self.pins.get(who) {
            None => Pin::New,
            Some(p) if p == fingerprint => Pin::Known,
            Some(p) => Pin::Changed { pinned: p.clone() },
        }
    }

    /// Pin (or re-pin, after the user accepted a change) and save.
    pub fn pin(&mut self, who: &str, fingerprint: &str) -> Result<(), String> {
        self.pins.insert(who.to_string(), fingerprint.to_string());
        self.save()
    }

    pub fn forget(&mut self, who: &str) -> Result<(), String> {
        self.pins.remove(who);
        self.save()
    }

    fn save(&self) -> Result<(), String> {
        #[derive(serde::Serialize)]
        struct F<'a> {
            pins: &'a std::collections::BTreeMap<String, String>,
        }
        if let Some(d) = self.path.parent() {
            std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
        }
        let text = toml::to_string(&F { pins: &self.pins }).map_err(|e| e.to_string())?;
        crate::key::write_atomic(&self.path, format!("# Pinned render-farm identities.\n{text}").as_bytes())
    }
}

#[cfg(test)]
mod tests;
