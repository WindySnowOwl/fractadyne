# Fractadyne — fractalforums.org presentation package

A self-contained status + validation package for community feedback.

- **[catalog.md](catalog.md)** — externally-verifiable challenge coordinates (period/nucleus +
  set membership), reformatted from [`validation/catalog.toml`](../validation/catalog.toml).
- **results/** — generated on an RTX 3080 / Ryzen 3950X, Fraktaler-3 v3.1. The deep-zoom numerics
  are unchanged, so all results remain representative; `selftest.txt` was refreshed at **v0.2.0**,
  `validate-deep` at v0.1.18, and the F3 ladder + images at v0.1.10:
  - `crosscheck-f3-ladder.md` — pixel-exact agreement vs Fraktaler-3, 1e6× → 1e12× (F3's working
    range on this GPU; F3 blanks past ~1e13×, so deeper correctness is `validate-deep`, not F3).
  - `selftest.txt` — GPU vs bignum-oracle self-test (61/61, goldens 17/17).
  - `validate-deep.md` — extreme-depth self-consistency battery (to 1e1000000×).
  - `refdiag.txt` — reference-orbit-length samples (illustrates the deep-perf gap).
  - `benchmark.txt` — standardized benchmark (system info + score).
- **images/** — a Fractadyne depth ladder (`01`–`06`, 1e6× → 1e300×) and a Fraktaler-3 ↔ Fractadyne
  side-by-side (`cmp_1e12_f3` / `cmp_1e12_fd`) at 1e12×, where F3 renders on this GPU. (The
  `cmp_1e100_*` / `cmp_1e300_*` panels predate the F3 ~1e13× ceiling finding — Fractadyne's own
  `01`–`06` ladder covers those depths.)

Everything is reproducible from the commands in `results/crosscheck-f3-ladder.md`.
