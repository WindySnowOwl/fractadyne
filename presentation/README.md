# Fractadyne — fractalforums.org presentation package

A self-contained status + validation package for community feedback.

> **Update (2026-07+):** the "F3 blanks past ~1e13×" ceiling referenced in the older results
> below was **our configuration error**, not an F3 limit — F3's batch mode needs
> `maximum_reference_iterations` raised (see
> [`validation/corpus/README.md`](../validation/corpus/README.md)). With it set, **Fractadyne and
> Fraktaler-3 now match across a 20-location corpus up to ~4.6e1105×**
> (`validation/corpus/`) — far stronger evidence than this package's 1e12× ladder. The documents
> below are kept as the historical point-in-time package.

- **[catalog.md](catalog.md)** — externally-verifiable challenge coordinates (period/nucleus +
  set membership), reformatted from [`validation/catalog.toml`](../validation/catalog.toml).
- **results/** — generated on an RTX 3080 / Ryzen 3950X, Fraktaler-3 v3.1. The deep-zoom numerics
  are unchanged, so all results remain representative; `selftest.txt` was refreshed at **v0.2.0**
  (the suite has since grown to 83 checks + 17 goldens), `validate-deep` at v0.1.18, and the F3
  ladder + images at v0.1.10:
  - `crosscheck-f3-ladder.md` — pixel-exact agreement vs Fraktaler-3, 1e6× → 1e12× (the deeper
    corpus in `validation/corpus/` supersedes this ladder — see the update note above).
  - `selftest.txt` — GPU vs bignum-oracle self-test (61/61 at the time, goldens 17/17).
  - `validate-deep.md` — extreme-depth self-consistency battery (to 1e1000000×).
  - `refdiag.txt` — reference-orbit-length samples (illustrates the deep-perf gap).
  - `benchmark.txt` — standardized benchmark (system info + score).
- **images/** — a Fractadyne depth ladder (`01`–`06`, 1e6× → 1e300×) and a Fraktaler-3 ↔ Fractadyne
  side-by-side (`cmp_1e12_f3` / `cmp_1e12_fd`) at 1e12×. (The `cmp_1e100_*` / `cmp_1e300_*` panels
  predate the corpus work; `validation/corpus/` now holds matched deep pairs to ~4.6e1105×.)

Everything is reproducible from the commands in `results/crosscheck-f3-ladder.md`.
