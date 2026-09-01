# Reference-pick redesign — handoff spec (2026-09-01)

The remaining half of the Fraktaler-3 gap at extreme depth. Written at the end of the session
that measured the problem and fixed the other half (`7cdd7c4`, the SA cost budget), so a fresh
session can start from the numbers instead of re-deriving them.

## The problem, measured

One location, 2.37e4000× (13,353-bit working precision), CLI render 400×250 `--iter 2008192`,
RTX 3080 / 3950X, run alone:

| reference-build stage | before `7cdd7c4` | after | share now |
|---|---|---|---|
| candidate scoring (`pick_pass`) | 113.7 s | **113.7 s** | **~58%** |
| orbit compute | 32.8 s | 32.8 s | 17% |
| series approximation | 258.2 s | 48.2 s (budgeted) | 25% |
| BLA build | 0.4 s | 0.4 s | — |

Fraktaler-3 renders the same location in **60.3 s** total; we are at **333 s** (was 529). The
pick is now the largest slice. ⚠The ratio INVERTS with depth — at the e500 hero the pick is 70%
of a 2.3 s build and SA is 0 — so never carry one depth's profile to another
(`validation/extreme-depth.md` has both profiles).

Timing method: `FRACTADYNE_TRACE=ref` — `pick [export]:` trace timestamp minus the
`building reference` crumb gives the pick; the `reference built` crumb's `orbit_ms/sa_ms/bla_ms`
splits the rest. Cross-checked against `--profile` (which now prints `series ms`).
⚠`--profile`'s `ref ms` column still EXCLUDES the pick — its timer starts after `pick_reference`.

## Where the 113.7 s goes (`pick_pass`, fractadyne-core/src/reference.rs)

- **Phase 1 — cheap rank**: 101 candidates (centre + 4 scales × 5×5 grid) walked to
  `quick = min(max_iter, REF_SCORE_SCAN=4096)` at FULL precision. Parallelized
  (`par_orbit_scores`) — wall ≈ 1–2 s on 32 threads. **Not the cost.**
- **Phase 2 — deep rank**: the centre survivor walked ALONE to `max_iter` (sequential), then up
  to `REF_DEEP_MAX−1 = 15` more survivors walked to `max_iter` in parallel. At e4000 every
  candidate survives phase 1 (`survivors=101`) and every deep walk escapes at ~443k steps, so
  phase 2 ≈ one sequential 443k-step bignum walk (~33 s) + a parallel batch of 15 more (~60–80 s
  wall with hyperthread scaling). **This is the cost.**
- The winner's escape length beat the centre's by **55 iterations** (443,199 vs 443,144): sixteen
  full-precision orbits were walked to separate escape lengths differing by ±0.01%.

## The structural observation (the headroom)

At extreme depth every candidate is within 0.5 view-spans of the centre — |Δc| ≈ 1e-4000 — so all
101 orbits are IDENTICAL until the perturbation grows to O(1), which takes ≥ |log2 δc| ≈ 13,289
iterations of ~2×/step growth (in practice they stay together for hundreds of thousands of
steps: escape lengths spread only ±55 at 443k). Walking 16 bignum orbits to compare them is
therefore almost pure duplication. **Walk ONE orbit (the centre's) in bignum; score every other
candidate by CPU perturbation against it** — a δ-iteration in floatexp costs ~ns/step against
~µs–ms/step in bignum at 13k bits. Phase 2 collapses to one walk (~33 s) + noise.

- The δ must be carried in floatexp, NOT f64 (underflows past ~1e-308) — the viewport already
  carries `units_per_pixel` this way, and `FloatExp` exists in core.
- Rebasing: a candidate's perturbed orbit can need rebasing onto the reference when it wanders;
  the GPU shader's rebase logic is the model (`rebase=695k` at e4000 — routine, cheap).
- A CPU floatexp perturbation loop does not currently exist in core (the CPU oracle is
  full bignum). Writing one is most of this task's new code — and it must be validated against
  the bignum walk it replaces before anything trusts its scores.

## What the pick's spec is FOR (do not silently weaken it)

"Longest-surviving reference" exists for LIVE zoom smoothness: a reference that survives the
whole render needs no rebasing, which is what keeps a continuous deep zoom smooth (the pick
comment documents the "jump on zoom" regression that motivated deep-ranking). A ONE-SHOT export
tolerates rebasing happily. Two consequences:

1. An export-only cheaper policy is on the table — but live and export choosing DIFFERENT
   references at the same view is exactly the G1-family trap (silent divergence between paths
   that must agree). If the policies diverge, it must be explicit, traced, and tested — not
   emergent.
2. Any replacement scorer must reproduce the ORDERING the deep rank produces (or a provably
   equivalent selection), not merely "a good reference". Judge the PICK, not the render — at
   depth, Newton/perturbation machinery renders acceptably from many references, so "the image
   looks fine" validates nothing (the Misiurewicz-ranking lesson: a degenerate criterion beat
   the shipped one on all three test locations at once and meant nothing).

## Existing gates a redesign must keep green

- **Corpus `--check` 38/38 maxD 0** — transitively pins the pick: a different winner ⇒ different
  reference ⇒ different pixels ⇒ red. (There is no NAMED pick-determinism check; the protection
  is byte-comparison in corpus/goldens/bench segments.)
- `--selftest` 168/168 + 18 goldens; `--bench-matrix` determinism half (algorithmic drift =
  exit 2; ⚠ its perf half cries wolf on this box — TODO.md).
- Reference REUSE interacts with the pick: `try_reuse_reference` deliberately keeps the SAME
  reference across rebuilds ("re-picking a different valid reference every ~0.16 octave made the
  view jump"). A redesign must not disturb reuse's identity contract.
- `pick [live/export]:` trace line (`FRACTADYNE_TRACE=ref`) prints ask/scored-bits/survivors/
  winner_len/offset — extend it rather than replacing it; it is how every pick investigation in
  the log history was diagnosed.

## Suggested acceptance harness (build FIRST)

A dual-run equivalence check: run old `pick_pass` and the new scorer on the same inputs and
assert the SAME WINNER (point identity, not score equality) across a depth ladder — e24, e43,
e89 (misiurewicz_outcomes' dendrite), e148 (profile region), e500 (hero), e726/e1008 (corpus 19/
20 centres), e4000 (`validation/e4000-misiurewicz.fdn`). Keep the old path compiled (test-only or
feature-gated) until the ladder has run on at least the corpus depths + e4000. Expect legitimate
ties (multiple candidates with equal deep scores) — the old tie-break order must be replicated,
not approximated: phase 2 takes the FIRST survivor reaching the best length in candidate order.

## Traps already paid for (do not pay again)

- Candidate orbits at depth are indistinguishable in f64 — every scoring shortcut must be
  floatexp-or-better (`orbit_length_bf` is hardcoded astro-float; `--bignum` never reaches it —
  which is also why rug measured 1.07×).
- A ranking that "wins" by where the solve/render lands proves nothing (see above).
- `--profile --regions` now REFUSES bad input (loud errors) — trust it, but remember `ref ms`
  excludes the pick.
- Wall-clock bounds in determinism-relevant paths are forbidden (the glitch-correction loop is
  the standing counterexample — non-deterministic under load). Budget in steps, like
  `SA_COST_BUDGET`.
- The corpus check needs the exe at `target/release/fractadyne.exe` (generate_corpus.py default)
  and an unlocked exe; the user's running app holds the lock — build to a scratch
  `CARGO_TARGET_DIR` and copy, or ask.

---

## Outcome (2026-09-01, shipped 0.2.41-beta.1)

Implemented as specified: phase 2 walks the FIRST survivor once in bignum
(`orbit_length_bf_recorded`, extended-range `CFloatExp` samples — an f64 sample would flush
the near-nucleus dips the rebase test needs), and scores the other ≤15 survivors by floatexp
δ-iteration against it, in parallel across cores. Same candidate order, same
strict-improvement tie-break, same early break; auto-gate `!julia && formula_power().is_some()
&& both spans < 2^-44`; the walk engine stays compiled as the fallback and the harness
comparator. `--pickcheck` is the acceptance harness (committed ladder e17 → e4000, plus any
`.fdn` by path); `RefPickDiag` and the `pick [..]:` trace grew
`deep=perturb(scored=N rb=M fb=K)`.

**One thing the spec's sketch missed, caught by the scorer-vs-oracle unit probes**: every
rebase to index 0 sets `δz ← z − Z₀ = z`, so from the FIRST rebase the walk is effectively
~53-bit iteration, and chaotic amplification can move the escape length — one probe mis-scored
a true `max_iter` SURVIVOR as escaping at 0.6× its length, WITHIN the reference's span (so an
overshoot-past-the-reference window cannot catch it). Shipped rule: a perturbed score is
trusted only if the candidate escapes (or survives the ask) within
`PERTURB_POST_REBASE_TRUST = 256` steps of its first rebase; otherwise that candidate is
re-walked in bignum (batched parallel) — the drift-prone class gets the old arithmetic by
construction. Rebase-free scores are trusted at any length (linear-in-δ regime, polynomial
error growth).

**Ladder (release, 1280×720, both engines, all MATCH, exit 0):**

| rung | regime | walk | perturb | note |
|---|---|---|---|---|
| e17 / e43 / e89 | no phase-1 survivor → rescue/fallback-escaper | ~0.02 s | ~0.02 s | scorer never runs (do-no-harm) |
| e24 / e148 | centre survives capped ask → early return | 0.04–0.17 s | ≈ same | do-no-harm |
| e500 (corpus 09) | scored=15 rb=67 fb=0 | 1.29 s | 0.91 s | **1.41×** |
| e500 (hero) | scored=15 rb=68 fb=0 | 1.60 s | 1.23 s | 1.30×; winner IDENTICAL, its score ±2 (the spec's "point identity, not score equality") |
| e726 | scored=15 rb=77 fb=0 | 1.32 s | 0.90 s | 1.47× |
| e1008 | scored=15 rb=50 **fb=7** | 2.69 s | 2.82 s | trust valve fired in the wild; winner still identical; net wash by design |
| e4000 | scored=15 rb=122 fb=0 | **115.2 s** | **68.6 s** | **1.68×** |

**Where the remaining e4000 cost lives**: the perturb pick is now ~96% the ONE sequential
first-survivor walk (~66 s at 13,352 bits; the δ-scoring is ~2 s). The spec's "one walk
(~33 s)" under-split the old 113.7 s — the sequential first walk was always ~66 s of it. Two
levers remain, both untouched here: (1) `orbit_length_bf` is ~2× the cost of the build's own
orbit walk at the same length — a leaner scoring walk would halve the floor; (2) the winner is
usually the first survivor, whose orbit the BUILD then re-walks at `p + 128` — reuse across
that precision boundary was out of scope. Reference build at e4000: ~333 s → ~287 s measured
end of this change; F3's 60.3 s still stands as the target.
