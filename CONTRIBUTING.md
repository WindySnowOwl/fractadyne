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

Requires the Rust toolchain (stable, via [rustup](https://rustup.rs)). The first
build fetches wgpu/egui and takes a few minutes.

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
- Keep changes focused; match the surrounding code's style, naming, and comment
  density. Update `CHANGELOG.md` and, for a functional change, bump the workspace
  version in `Cargo.toml`.
- New behaviour should come with a test or a validation step (a core unit test, a
  `--selftest` case, or a documented manual check).

## Design docs

`DESIGN.md` (architecture), `UI-DESIGN.md` (UI), and `TODO.md` (backlog) describe
where things live and what's planned — a good place to find an on-ramp.

## License

By contributing, you agree that your contributions are licensed under the
project's dual **MIT OR Apache-2.0** license (see `LICENSE-MIT` / `LICENSE-APACHE`).
