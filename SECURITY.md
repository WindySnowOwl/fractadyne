# Security Policy

## Supported versions

Fractadyne is developed on a rolling basis; only the **latest released version**
receives security fixes. Please reproduce any issue on the most recent release
(or `main`) before reporting.

| Version        | Supported |
| -------------- | --------- |
| latest release | ✅        |
| older releases | ❌        |

## Threat model

Fractadyne is a desktop fractal explorer. It renders on your GPU and, by design,
**opens files that may come from other people**:

- shareable location blobs (`.fdn`)
- guided-tour / camera scripts (`.toml`)
- profiling-region files (`.toml`)
- view metadata embedded in imported `PNG` / `OpenEXR` images
- imported Kalles-Fraktaler (`.kfr`) locations

These parsers are the primary attack surface and are deliberately hardened
(size-bounded input, an allow-list of keys, every value range-checked/clamped,
unknown keys ignored, no paths or code executed) and fuzzed. Reports of a way to
crash, hang, exhaust memory, or otherwise misbehave when opening one of these
files are especially welcome.

**The render farm** (Tools ▸ Render on farm…, File ▸ Render client…) is the app's
only network service, and it exists only while a farm runs. The controller listens
on one TCP port (46733 by default) and answers discovery probes on UDP 46733; a
render client opens no port — it dials the controller. Every connection is
encrypted and authenticated with the farm key (Noise `XXpsk3`): a party without the
key cannot complete the handshake, and after the first one each side pins the
other's identity. The protocol is a closed list of size-bounded messages with every
field checked; no path or command crosses it, a job's script arrives as text and
goes through the same hardened tour parser, and every frame a client returns is
checked before it is kept. A discovery reply carries only what the handshake shows
anyone who connects (the controller's name, port, build and identity fingerprint),
is never larger than the probe that asked for it, and is rate-limited. Reports of a
way past the handshake, or of a message or packet that crashes, hangs or misleads
either side, are especially welcome.

Out of scope: issues that require the attacker to already control the machine or
to have you run a modified build; general crashes with no untrusted-input vector
(please file those as ordinary bugs).

## Reporting a vulnerability

**Please do not open a public issue for a security vulnerability.**

Report privately instead, so a fix can ship before details are public:

1. **Preferred:** use GitHub's private vulnerability reporting —
   the **"Report a vulnerability"** button under the repository's
   **Security** tab (Security → Advisories).
2. **Alternatively:** email **feedback@fractadyne.org** with `[fractadyne security]` in
   the subject.

Please include:

- the affected version (and OS / GPU if the issue is render-related),
- a minimal file or steps that reproduce it,
- what you observed vs. expected, and the impact as you see it.

You can expect an acknowledgement within a few days. Once a fix is available it
will be released and the reporter credited in the release notes (unless you'd
prefer to remain anonymous). Coordinated disclosure is appreciated.
