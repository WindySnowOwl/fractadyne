# Fractadyne — fractalforums.org presentation package

A self-contained status + validation package for community feedback.

- **[fractalforums-post.md](fractalforums-post.md)** — the post: what it is, validation
  methodology, deep Fraktaler-3 cross-check, candid known gaps, and specific feedback requests.
- **[catalog.md](catalog.md)** — externally-verifiable challenge coordinates (period/nucleus +
  set membership), reformatted from [`validation/catalog.toml`](../validation/catalog.toml).
- **results/** — generated on an RTX 3080 / Ryzen 3950X, Fractadyne v0.1.10, Fraktaler-3 v3.1:
  - `crosscheck-f3-ladder.md` — pixel-exact agreement vs Fraktaler-3, 1e6× → 1e300×.
  - `selftest.txt` — GPU vs bignum-oracle self-test (55/55, goldens 4/4).
  - `validate-deep.md` — extreme-depth self-consistency battery (to 1e1000000×).
  - `refdiag.txt` — reference-orbit-length samples (illustrates the deep-perf gap).
  - `benchmark.txt` — standardized benchmark (system info + score).
- **images/** — a Fractadyne depth ladder (`01`–`06`, 1e6× → 1e300×) and Fraktaler-3 ↔ Fractadyne
  side-by-sides (`cmp_<zoom>_f3` / `cmp_<zoom>_fd`) at 1e12×, 1e100×, 1e300×.

Everything is reproducible from the commands in the post and in `results/crosscheck-f3-ladder.md`.
