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

Out of scope: issues that require the attacker to already control the machine or
to have you run a modified build; general crashes with no untrusted-input vector
(please file those as ordinary bugs).

## Reporting a vulnerability

**Please do not open a public issue for a security vulnerability.**

Report privately instead, so a fix can ship before details are public:

1. **Preferred:** use GitHub's private vulnerability reporting —
   the **"Report a vulnerability"** button under the repository's
   **Security** tab (Security → Advisories).
2. **Alternatively:** email **pub@rithea.com** with `[fractadyne security]` in
   the subject.

Please include:

- the affected version (and OS / GPU if the issue is render-related),
- a minimal file or steps that reproduce it,
- what you observed vs. expected, and the impact as you see it.

You can expect an acknowledgement within a few days. Once a fix is available it
will be released and the reporter credited in the release notes (unless you'd
prefer to remain anonymous). Coordinated disclosure is appreciated.
