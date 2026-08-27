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

### The optional accelerated (MPFR) build

Nothing above builds it, and nothing needs to: it is off by default and the standard build is
unaffected. To produce it on Windows:

```powershell
.\scripts\build-accelerated.ps1 -Deps    # prints the exact prerequisites
.\scripts\build-accelerated.ps1          # builds, verifies, and packages it
```

It needs the **GNU** Rust toolchain plus MSYS2, because MPFR does not build under MSVC —
which is the whole reason it ships as a separate download rather than a feature flag on the
normal binary. To just run the tests against it:

```sh
cargo +stable-x86_64-pc-windows-gnu test -p fractadyne-core --release --features rug
```

Two things about that script are load-bearing rather than incidental, and both are explained
in its header: it links GMP/MPFR **dynamically** (LGPLv3 4(d)(1) — notices only, instead of
4(d)(0)'s requirement to ship the app in relinkable form with every release), and it verifies
the **packaged** binary on a PATH stripped of MSYS2. An earlier version verified it with MSYS2
on PATH, passed, and blessed a package that failed on every machine without MSYS2.

> Note: the repo's `[profile.dev] debug = false`, `-j1` and no-LTO settings are a
> workaround for the original author's page-file-constrained machine, not a
> requirement — build normally on a machine with adequate memory.

## Before a PUBLIC RELEASE

The automated gates prove the renderer computes the right numbers. They cannot tell you the
window opened, the menus are reachable, a palette looks like a palette, or the app survives
being resized — so there is a manual pass as well:

```sh
python scripts/release_checklist.py     # regenerates validation/release-checklist.xlsx
```

Walk `validation/release-checklist.xlsx` end to end **on the exact build you intend to
publish** — launch, layout, window sizing, navigation, deep zoom, all ten formulas,
Julia/dual view, colouring, quality, locations/bookmarks, export, tours, help, the optional
accelerated build, persistence and stability. (The generator prints the step count; it is
not repeated here, because a number in prose goes stale and then nobody trusts either copy.) Record Actual result and Pass/Fail, and fill in the Cover sheet
(build, GPU, decision). The checklist names real controls; if a step can no longer be
performed because something moved, fix the step — do not skip it.

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
- **Don't use `\` line-continuations inside Rust string literals here.** The repo is CRLF, and a
  continuation followed by `
` does not strip the next line's indentation the way it does with a
  bare `
` — so the text reaches the user with a long run of spaces mid-sentence. Three user-facing
  messages shipped that way in one day before this was written down. Put the string on one line (long
  lines are fine for messages), or build it with `concat!` / separate pushes. Doc comments and normal
  code wrapping are unaffected; this is only about `"..."` string bodies.
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
