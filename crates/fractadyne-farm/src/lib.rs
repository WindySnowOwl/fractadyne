//! The render farm (design/remote-rendering.md): everything that lets one tour be rendered on
//! several machines, except the rendering itself, which stays the app's own `--render-tour` child.
//!
//! Pure on purpose — no GPU, no UI, no app state — so each part is tested on its own:
//!
//! - [`key`]: the farm key a user pastes into each client, and each machine's static identity;
//! - [`proto`]: the closed list of messages, each with a size cap and field checks;
//! - [`channel`]: the Noise_XXpsk3 session over TCP, key pinning, handshake rate limiting;
//! - [`sched`]: who renders which frames — a state machine with an injected clock;
//! - [`manifest`]: the job's state in `<out>/farm/`, from which a render resumes;
//! - [`settings`]: the render-affecting settings a job carries instead of a session file;
//! - [`names`]: the only strings that ever become parts of a file name.
//!
//! The app (`fractadyne-app::farm`) drives these: sockets, threads, child processes, files.

pub mod channel;
pub mod key;
pub mod manifest;
pub mod names;
pub mod proto;
pub mod sched;
pub mod settings;

/// The protocol both ends must speak, exactly. Part of the version gate with the app version and
/// commit: a client whose protocol differs is refused before any job is offered.
pub const PROTOCOL_VERSION: u32 = 1;

/// The controller's default listening port (design §13.9). Any unassigned high port works; this one
/// is shown in the controller and typed into each client.
pub const DEFAULT_PORT: u16 = 46733;

/// SHA-256 of `bytes`, lower-case hex — the digest every frame and blob is verified by.
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex(ring::digest::digest(&ring::digest::SHA256, bytes).as_ref())
}

/// Lower-case hex of `bytes`.
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// An incremental SHA-256, for a blob that arrives in chunks.
pub struct Sha256(ring::digest::Context);

impl Default for Sha256 {
    fn default() -> Self {
        Self(ring::digest::Context::new(&ring::digest::SHA256))
    }
}

impl Sha256 {
    pub fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }
    pub fn finish_hex(self) -> String {
        hex(self.0.finish().as_ref())
    }
}

#[cfg(test)]
mod lib_tests;
