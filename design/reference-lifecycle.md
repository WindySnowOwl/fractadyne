# Reference lifecycle redesign — precision cliffs, fitness, and non-destructive replacement

Status: **L0+L1+L2-core LANDED** (2026-08-09, beta.49) — see § 6 for what shipped and how it
deviated from the plan below. Remaining: § 3 Layer 2's incumbent/challenger (deferred — see § 6),
Layer 3 protocol notes.
Owner: the 2:58 device-loss investigation.
Prereq reading: TODO.md § Open bugs (the device-loss entry, bottom-up), `topic-spar-family` memory.

## 1. What is actually wrong (all measured, nothing inferred)

### 1.1 A reference orbit's usable length is a function of PRECISION, with cliffs

`cargo test -p fractadyne-core --release --lib escape_length_vs_precision -- --ignored --nocapture`

At the exact three-spar dive centre (a Misiurewicz point — its true orbit never escapes), with the
point parsed at full precision and only the orbit arithmetic varied:

| orbit precision | computed escape length |
|---|---|
| 128 bits | 570 |
| 160–180 | 84,941 |
| 207–240 | 570,711 |
| 286+    | survives 700,000 (cap) |

A repelling (Misiurewicz-shadowing) orbit amplifies per-iteration rounding error until the computed
orbit spuriously escapes. The escape length is a **property of the precision**, stepped (cliffed),
not gradual. This one mechanism explains the whole spar family's "the reference naturally escapes
at N" observations — 626, 256,753 (e63/beta.27), 928,084 (e82/beta.30), 602,516 (e72) are all
precision cliffs of some build, none of them facts about the location.

The app's precision policy everywhere is `octaves + 64` — **depth-based, iteration-blind**. Nothing
ties the precision of a reference to the number of iterations it is being asked to stay true for.

### 1.2 The picker goes blind below the cliff

`best_reference` scores candidates by bignum orbit length **at the caller's precision**. When that
precision's cliff is below the 4,096-iteration quick scan, *every* candidate "escapes" in phase 1,
there is no survivor, and the function falls back to "longest escaper" — institutionalising a
short-escaper pick at exactly the views where the centre (fed 32 more bits) would survive the whole
render. Fed adequate precision the picker is healthy: offline picked the centre and produced
`len=26465 partial` at 286 bits; the live path later in the tour produced `len=52648 partial`.

### 1.3 The reuse gate pins one bad pick indefinitely

`try_reuse_reference` validates **correctness only**: precision headroom (`reuse.prec ≥
inp.precision`), drift (≤ 0.7 span), non-empty. There is no fitness term, and an escaped orbit is
returned by `extend_reference_orbit` unchanged. Measured on the grand tour: a 626-sample escaped
reference picked by a `prec=78` lookahead build (wall t≈172, pacer-dilated clock) is then reused by
**every** build — lookahead and live — for the next ~32 seconds while the ask climbs to 34,000.
Reuse cost ~2 ms per "build", so nothing ever looks wrong from the build side.

Cost mechanics: BLA refuses any skip that would run past the reference end
(`ref_n + span >= orbit_len` in mandelbrot.wgsl), so ask/len ≈ 54 means every pixel serially
rebases ~54 windows with `bla_skip=0` (live counters: `rebase=16,414,717 bla_skip=0`). Measured
~90× cost against a healthy reference at the same view (1018 ms vs 11 ms), which after the
df32→floatexp crossover (≈10× per-step) becomes ~1 s dispatches at 69×54 → `nvlddmkm` Event 153 →
device loss at tour 2:58. Reproduced 3-in-4 with a fresh config dir.

### 1.4 Naive eviction deadlocks — measured three times

Refusing reuse on a fitness predicate (`ask/len` > N traversals; three variants tried) regressed
`--livetest hold-e72` from 0% → 100% black identically each time. Mechanism: refusal discards
accumulated extension state (e72's incumbent is a 602,516-sample orbit accumulated one extension
per rebuild across the descent); the fresh replacement is capped (`LIVE_REF_CAP` in motion), the
freeze guard rejects long partials (`install_recompute`: partial > LIVE_REF_CAP+1 → refused), and
the boost/appetite coupling then can never lift the pixel clamp. **Any design in which "stop using
the bad reference" implies "throw the incumbent away" is wrong.** The choice must be between the
incumbent and a *ready, strictly better* replacement — never between the incumbent and nothing.

### 1.5 Persistence can carry the poison across sessions

`build_saved_ref` skips `partial` references — but an escaped 626 is **not** partial, so it
qualifies for `refcache_persist`. A session that ends in the 626 era re-seeds the pathology at next
launch. Conversely a populated config dir whose saved reference is deep and healthy **masks** the
bug (the dive extends the restored orbit and never picks badly) — which is why the repro needs a
fresh dir, and why the field failures are intermittent: it depends on what the previous session
left behind.

## 2. Goals

- **G1 — truthful references**: an orbit asked to serve N iterations either stays true for ≥ N, or
  is *genuinely* escaping (a property of the point, stable under added precision). Numerical
  escapes never enter the cache.
- **G2 — no pinning, no destruction**: a bad reference cannot pin itself via reuse; replacement is
  adopt-if-strictly-better, so the incumbent keeps serving until the challenger is ready and wins.
- **G3 — verifiable end to end**: every layer lands with its own measurement — the live
  `rebase/bla_skip` counters, the pick/reuse traces, the fresh-dir repro with Event 153 as
  arbiter, and the existing gates (goldens, bench-matrix, livetest, corpus) at 0 drift.

Non-goals (tracked elsewhere in TODO.md): the LIVE_REF_CAP pixel-clamp black holds (e82/e95
architectural limit), offline tour-render cost/memory bounds, the livetest flap.

## 3. Design

### Layer 0 — observability (prereq, small)

- Trace every **pick**: scoring precision, survivor count, chosen point's offset from centre (in
  spans), scored length, and whether the no-survivor fallback fired. The fallback firing is the
  cliff detector's raw signal and must be visible.
- Trace every **reuse** decision: drift, prec check, len, escaped, ask, and the implied traversal
  ratio.
- Extend the slow-frame log (already carries `rebase/bla_skip`) with the current reference
  `len/escaped/prec` so one line connects cost to reference state.

### Layer 1 — kill numerical escapes at build time (the root fix)

**Escape-plateau probe.** After a pick or build produces an orbit that escaped at `len < ask`
(i.e. the reference would be traversed ≥ 2×), rebuild the orbit at `prec + 64` and compare:

- escape length **grew** → the escape was numerical. Escalate (another +64, bounded at 3 steps or
  `len ≥ ask`, whichever first) and use the escalated result. Record the pick as `cliff-rescued`.
- escape length **stable** → genuine escaper (true property of the point). Accept as today.

The ladder shows growth under the cliff is dramatic (570 → 84,941 for +32 bits), so one step
almost always resolves. Cost is one extra bignum orbit, paid **only** on suspicion; the
101-candidate scan is not repeated.

**Picker rescue.** When phase 1 finds no survivor (the blindness signature), rescore the *centre
candidate only* at `prec + 64` before accepting the longest-escaper fallback. If the centre
un-cliffs, it wins. This costs one orbit in exactly the pathological case.

**Placement**: app-side in the recompute worker (`render.rs`), with a small `pub` helper in core
for the rescore. `best_reference` itself stays pure/deterministic — bench-matrix's
pick-determinism tripwire must stay green (the probe changes picks only where the current pick is
the measured pathology; expect and bless no drift in the 22 segments, which run at healthy
precisions).

**Acceptance**: fresh-dir grand tour — the t≈147–172 era must show `cliff-rescued` picks instead
of 626 installs; crossover slow frames (if any) must show `bla_skip > 0`; ≥ 8 hygienic runs with 0
Event 153. Plus a fast non-`#[ignore]` core test pinning the probe's discriminator on the
three-spar coords.

### Layer 2 — incumbent/challenger cache (fitness without destruction)

Replace "reuse-or-repick" with **adopt-if-strictly-better**:

- The cache keeps serving the **incumbent** exactly as today — every existing consumer, the freeze
  guard, and the refusal ledger are untouched. By construction, no regression is possible while a
  challenger is pending: the worst case is today's behaviour.
- At build time compute fitness: incumbent `escaped && ask > K·len` (start K=4; the live
  `bla_skip≈0` counter is the empirical justification). Unfit ⇒ the worker still returns the
  incumbent-based result **and** sets `refit_wanted`.
- `refit_wanted` spawns one low-priority **challenger** build (shares `PREFETCH_MAX_INFLIGHT`; at
  most one challenger in flight): a fresh Layer-1 pick at iteration-aware precision.
- On completion, **install-compare**: adopt only if the challenger strictly beats the incumbent —
  it covers the ask (`len ≥ min(ask, cap)`, or survives) or its coverage exceeds the incumbent's
  by a margin (×2). Otherwise drop it and latch the attempt (retry only when the ask grows —
  the `ref_ext_refused` pattern, which already encodes "quiet until you can ask for strictly
  more").
- A challenger the freeze guard refuses leaves the incumbent serving — the e72 deadlock is
  structurally impossible: the incumbent (602k accumulated) is never discarded, and a replacement
  must beat it to enter.
- **Persistence gate**: `build_saved_ref` additionally refuses an unfit reference (escaped with
  `orbit_iter` ≪ the ask it was serving), so a bad era cannot cross sessions.

**Acceptance**: hold-e72 baseline behaviour identical (`--livetest` 0 drift — the incumbent path
is untouched); a seeded-bad-reference scenario (start the tour with a planted 626 cache via
`refcache_persist`) recovers within one challenger build; adoption events visible in the Layer-0
trace.

### Layer 3 — verification hardening

- Keep `escape_length_vs_precision` as the documented experiment; add the fast discriminator test.
- Unit-test the adoption predicate and the fitness predicate as pure functions (same pattern as
  `budget_step`/`refusal_survives_install`).
- Repro protocol pinned in TODO.md: fresh dir, `taskkill` before/after, N ≥ 8 runs, Event 153 as
  the only accepted verdict, tracing OFF for verdict runs (tracing suppresses the crash).
- `--livetest`: expect e61/e63 holds to improve if their black excess was partly unfit-reference
  (their PARTIAL orbit context says pixel-clamp, so improvement is possible but not promised);
  re-bless only improvements.

## 4. Sequencing

1. **L0** (small) — traces. No behaviour change; bless nothing.
2. **L1** (medium) — the probe + picker rescue. Likely cures the crash on its own; run the full
   acceptance battery before starting L2.
3. **L2** (larger) — incumbent/challenger. Cures the *class* (any future bad pick, any entry
   vector, including persistence).
4. **L3** — tests + protocol + docs.

Each layer lands with suite 104+/goldens 17/bench-matrix 0-drift/livetest 0-drift green.

## 5. Risks

- **Probe cost at extreme depth**: one +64-bit orbit on suspicion only; at depths where orbits are
  expensive the pick already costs ~1 s, and the probe replaces a catastrophically wrong result.
- **Challenger vs lookahead core contention**: shared in-flight cap; challengers are rare (fitness
  only fails in pathological eras).
- **Adoption = reference swap mid-dive** → the "view jumps on zoom" the pinning originally
  prevented. Mitigation: adoption only fires when the incumbent is *unfit* — the jump is strictly
  preferable to 90× frame cost — and lands at install boundaries exactly like today's re-picks.
- **Bench-matrix pick determinism**: the probe must be deterministic (fixed escalation schedule,
  no timing dependence) so the tripwire stays meaningful.


## 6. What actually shipped (beta.49) — and where the plan was wrong

Implementation order was driven by two measurements made AFTER the plan was written:

1. **The rounded-point axis** (`escape_length_of_rounded_centre`): `Playback::sample` routed a
   PINNED-CENTRE glide through `lerp_bf(a, a, ease, p)` with `p` = the *current interpolated
   depth's* precision — which rounds the centre, handing the pick a genuinely different point:
   78 bits → true escape 625 (**the crash's `len=626`**), 157 → 94,126, 206 → 602,515 (**hold-e72's
   "accumulated" 602,516 — it was never an accumulation**), 300+ → survives. Every mysterious
   orbit length in the ledger is `escape(round(centre, p)) + 1`. Fixed: a pinned-centre glide now
   returns the centre exactly; genuine pans still interpolate (arrivals are keyframes, which the
   hold branch returns exactly).
2. **The e72 lottery**: the deep holds' historically-green states were CARRIED by cliff-escaped
   (numerically wrong) orbits laundering themselves past the freeze guard — a truthful orbit comes
   back partial > `LIVE_REF_CAP` and was refused. So the sampler fix and the guard change are
   COUPLED: landing the first without the second regresses e72 to 100% black (measured, twice,
   before the coupling was understood).

Shipped:
- **L0**: pick trace (`pick [origin]: ask/scored@bits/survivors/winner_len/RESCUED/offset`),
  slow-frame log carries `rebase/bla_skip/ref_len/partial`.
- **L1 — cliff rescue** in `best_reference` (now `best_reference_diag`): no phase-1 survivor →
  full re-scan at `p+128` (the build precision); winner-escapes-early → centre rescored at
  `p+128`, taken only if it survives the render. Deterministic; healthy picks byte-identical
  (bench-matrix 0 drift). Selftest group `ref-pick` pins the exact bug pick (prec-78 era) and the
  no-rescue-on-healthy case.
- **L1 — exact pinned-centre glides** in `Playback::sample` (the entry-vector fix).
- **L2-core — the freeze guard now INSTALLS long partials at the FLOOR budget** instead of
  refusing them: `install_recompute` treats `partial > LIVE_REF_CAP+1` as maximally
  cost-discontinuous (`fe_budget := TDR_MIN_STEPS`, re-measure from a ~10 ms worst-case
  dispatch). The refusal's own comment said it stood "until the install frame itself can be
  cost-bounded"; beta.48's floor is that bound. This retires the A3 refusal loop (nothing
  refuses), and the pixel clamp lifts at every deep hold.

Measured outcomes (all on the grand tour, fresh-dir protocol):
- Device loss: **0 in 4 hygienic runs** (base rate was 3-in-4; p ≈ 0.4% if unfixed), 0 `nvlddmkm`
  Event 153 in the window. The 626 era is gone from the traces.
- `--livetest`: **hold-e61 42.1% → 0.0% black, hold-e63 53.9% → 1.5% (matches offline),
  hold-e82 100% → 0.0%**, hold-e72 stays 0%; orbit contexts now show truthful long partials
  installed (408,193 / 508,193 / 2,008,193). One cosmetic drift: `seahorse-2` captures mid
  budget-re-climb after a healthier install (res varies run to run; verdict ok). Baseline
  re-blessed.
- e21000 canonical wedge case: 120 s live soak, responsive throughout, no crash, no hang.
- Suite 106/106, goldens 17/17, bench-matrix 0 algorithmic drift.

Plan deviations, recorded:
- The plan's L1 "escape-plateau probe on built orbits" was NOT implemented — the rescue subsumes
  its pick-time role, and the reuse-time role belongs to L2's challenger (a probe cannot fix a
  genuinely-escaping pinned point, and a rebuild-at-higher-precision result may be uninstallable
  mid-motion; both measured).
- A reuse-fitness refusal gate (L1 draft) was implemented, measured to regress hold-e72 by
  trajectory change even when scoped to `ask ≤ LIVE_REF_CAP`, and REMOVED. Attribution run
  (rescue-only) confirmed the gate was the sole regressor. Refusal-shaped eviction stays dead;
  if a pinned-bad-reference case surfaces again it must be solved by the incumbent/challenger
  design (§ 3 L2), never a predicate.
- With the entry vector (sampler) fixed and reuse now serving truthful orbits, the
  incumbent/challenger cache is DEFERRED until a real pinned-unfit case is observed — the live
  `bla_skip/rebase` counters and the pick trace make one visible within minutes if it exists.
