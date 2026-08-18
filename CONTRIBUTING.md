# Contributing to Fractadyne

Thanks for your interest! Fractadyne is a native deep-zoom fractal explorer
(Rust + wgpu/egui). Contributions — bug reports, fixes, features, docs — are
welcome.

## Reporting bugs & requesting features

Use the issue templates (Issues → **New issue**). A good bug report includes the
version (Help → About, or the title bar), your OS + GPU, exact steps or a file
(`.fdn` / tour `.toml` / exported image) that reproduces it, and what you saw vs.
expected. For **security** issues, follow [SECURITY.md](SECURITY.md) instead of
opening a public issue.

## Building

**Windows quick start:** from the repo root, run

```powershell
./scripts/setup.ps1
```

It checks for (and can install) the Rust toolchain and the MSVC C++ build tools,
then does a verification build. Safe to re-run. Pass `-Yes` for unattended setup
or `-SkipBuild` to only set up the toolchain.

Starting from nothing — no toolchain, no checkout — `./scripts/windows-build.ps1
-Deps` installs the prerequisites, clones the repo, and builds it; re-run it
without `-Deps` to fetch the latest `main` and rebuild. `./scripts/linux-build.sh`
is the Debian/Ubuntu equivalent.

⚠**Keep `.ps1` files either pure ASCII or UTF-8 with a BOM.** Windows PowerShell
5.1 — what a fresh Windows install runs — decodes a BOM-less file as CP1252, so a
UTF-8 em-dash becomes a stray `"` that swallows the rest of the script. Five
scripts in this repo, `setup.ps1` among them, failed to *parse* on 5.1 for that
reason until 2026-08-15. The scripts a stranger runs (`setup.ps1`,
`windows-build.ps1`, `gpu-validate.ps1`) are kept ASCII-only, since a BOM may not
survive being copied or downloaded raw.

Otherwise (or on Linux/macOS), all you need is the Rust toolchain (stable, via
[rustup](https://rustup.rs)); on Linux also install the GTK/X11/Wayland dev
packages that eframe/`rfd` link against. The first build fetches wgpu/egui and
takes a few minutes.

```sh
cargo run   -p fractadyne-app     # launch the app
cargo build -p fractadyne-app     # build only
cargo test  -p fractadyne-core    # pure-Rust exact-math suite (no GPU/GUI)
```

A running GPU is needed for the app and for the end-to-end render self-test:

```sh
cargo run -p fractadyne-app -- --selftest   # GPU vs CPU/bignum oracle + goldens
```

> Note: the repo's `[profile.dev] debug = false`, `-j1` and no-LTO settings are a
> workaround for the original author's page-file-constrained machine, not a
> requirement — build normally on a machine with adequate memory.

## Before you open a PR

- **`cargo fmt`** and **`cargo clippy`** should be clean.
- **`cargo test -p fractadyne-core`** must pass (CI runs this on Linux and builds
  the whole workspace on Windows; see `.github/workflows/ci.yml`).
- For anything touching the numerics or render pipeline, run `--selftest` locally
  (CI can't — the runners have no GPU) and mention the result in the PR.
- **If you are not on the reference GPU, expect some differences and don't chase
  them.** The golden images and the path-signature baseline were recorded on one
  card; cross-vendor floating point legitimately differs, so both are compared
  with a wider tolerance elsewhere and the run says which mode it used. The other
  checks should pass everywhere — those failing is a real signal. `--gputest` is
  worth a run too: it reports whether your stack's shader compiler preserves the
  extended-precision arithmetic deep zoom depends on (NVIDIA's does not), and a
  result from hardware we don't own is genuinely valuable. `scripts/gpu-validate.ps1`
  / `.sh` runs the whole battery and leaves one bundle you can attach.
- Keep changes focused; match the surrounding code's style, naming, and comment
  density. Update `CHANGELOG.md` and, for a functional change, bump the workspace
  version in `Cargo.toml`.
- New behaviour should come with a test or a validation step (a core unit test, a
  `--selftest` case, or a documented manual check).
- **Coverage, if you want to see what your change is missing:** `scripts/coverage.ps1`
  writes a local HTML + lcov report. It needs `cargo-llvm-cov` and the
  `llvm-tools-preview` rustup component; the script probes for both and prints the
  install commands rather than installing anything behind your back (pass `-Install`
  to opt in). Note that a plain `cargo llvm-cov test` badly understates this project:
  `cargo test` covers the pure logic only, while everything touching a GPU lives in
  `--selftest` and `--livetest`, which run as the built binary. The script instruments
  those too and merges them into one report, so don't substitute a bare
  `cargo llvm-cov test` number for it -- it would argue for unit tests duplicating what
  the harnesses already prove.

## Design docs

`DESIGN.md` (architecture), `UI-DESIGN.md` (UI), and `TODO.md` (backlog) describe
where things live and what's planned — a good place to find an on-ramp.

## License

By contributing, you agree that your contributions are licensed under the
project's dual **MIT OR Apache-2.0** license (see `LICENSE-MIT` / `LICENSE-APACHE`).
