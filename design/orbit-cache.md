# Reference-orbit cache

Returning to a location you have already visited at extreme depth costs **30–60+ minutes** — all of
it spent rebuilding a reference orbit you already had. This is the design for keeping it.

Status at `v0.2.41-beta.78`: **built, gated, and in the app.** The codecs (beta.77), the store, the
worker-side lookup and write-back, the `orbit-cache` selftest and the File ▸ Settings ▸ Reference
cache… window are all in. What remains is the measurement that started this — cold vs cached at
`validation/spiral-9.98e60205.fdn` — see **Plan**.

## ⛔Read this first: `refcache_persist` already existed

`crates/fractadyne-app/src/refcache_persist.rs` already saved a deep reference on exit and restored
it on launch (`last_reference.bin`). **This design grew that module; it does not sit beside it.**

I did not find it before writing a second codec — a real process miss, recorded here so the next
person starts from the existing code rather than repeating the search.

| | before (`refcache_persist`, ≤ beta.77) | now |
|---|---|---|
| entries | **one** — the last view | many, in `<config_dir>/orbits/`, against a budget |
| match rule | exact view centre + zoom exponent | the **reference point inside the view**, at a usable precision |
| point encoding | decimal string | exact mantissa words (`bfbytes`) |
| extendable | no tail stored | tail stored → `extend_reference_orbit` resumes |
| SA / BLA | stored | rebuilt on load (see below) |
| eviction | — | **cheapest orbit first** (`orbit_len × precision²`), recency breaks ties |
| user control | none | path (openable), usage vs limit, limit, clear |

So the existing module solved *"resume the session I closed"*. That is now the trivial case of a
cache hit: the restored view's cold start finds its own orbit. The reported need — *"go back to a
place I visited earlier, and explore around it"* — is the general case.

## The opportunity, in numbers

The cost is the orbit: at 9.98e60205× that is ~200,000-bit arithmetic run for up to two million
iterations. What the GPU consumes from it is `[f32; 4]` per iteration — **16 bytes**.

| artifact | iterations | size |
|---|---|---|
| live view (`LIVE_REF_CAP` = 256,000) | 256,000 | **4.1 MB** |
| full quality | 2,000,000 | 32 MB |
| exact point + extendable tail | — | ~150 KB |

At 4 MB per live orbit the default 1 GB budget holds ~250 extreme locations.

⭐**One orbit serves a NEIGHBOURHOOD.** Perturbation renders every pixel as a delta from the
reference, so a cached orbit covers nearby views too — come back, then zoom into a *different*
region nearby, with no rebuild. Going *deeper* needs more iterations, which is what the tail is for.

## What is built

**`fractadyne_core::bfbytes`** (beta.77) — exact `BigFloat` bytes (sign, exponent, mantissa words).
⚠**Not because decimal is wrong.** `refcache_persist` already noted that astro-float's
`to_string()`/parse round-trip is not bit-stable at this precision, and that is *benign* for a
restored point: a last-bit difference at 200,000 bits is ~2^-200000, far below the pixel spacing.
The reasons exactness matters here are **key stability** (an entry is named by the point's words; an
unstable encoding means every lookup misses), **size** (~25 KB vs ~60,000 digits), and avoiding a
lenient `FromStr` in a load-bearing path.

**`render::orbit_blob`** (beta.77, layout v2 in beta.78) — orbit + everything determining what it is
(formula, Julia c, backend, precision, iteration count, exact point, tail) in **two sections, each
under its own digest**: a small header (identity + point) the cache index reads from a file prefix,
and the body (tail + orbit) under a digest over the whole file. Refuse, never repair. Pinned by
flipping one bit at every byte offset in turn (header read and full decode alike), plus truncation,
trailing bytes, foreign magic, foreign version, an absurd length that must not become an
allocation, and a key that changes when the point's LAST word does.

**`refcache_persist`** (beta.78) — the store: a process-global index of headers over
`<config_dir>/orbits/*.orbit`, lookup by admissibility, threshold write, temp-then-rename, cost-aware
eviction, clear, and the on/off switch. Unit-tested against a scratch directory.

⚠**SA and BLA deliberately not stored** (unlike the old one-slot file): they are derived, cheap next
to the orbit, and the BLA depends on live colouring settings (`bla_stripe_freq`, `bla_trap_type`) —
caching it would drag those into the cache identity for no gain. ✅**Measured, not argued** (the
numbers already in `reference.rs`, taken at 2.37e4000×, 13,353 bits): of a 405 s reference build,
BLA was **0.4 s**; the SA walk is not run at all when a BLA tree will exist, which is every deep
Mandelbrot view. The e60205 split is being taken with `FRACTADYNE_TRACE=ref` on the opt-in
`deep-location` gate and goes in the table below when it lands.

⭐**`backend` is part of the identity** — `OrbitTail::backend` exists because an extension must
resume in the backend that started it; the same reasoning makes it part of what an orbit *is*.

## The seam that makes this low-risk

`RecomputeInputs` already carries `reuse: Option<ReuseRef>` = `{ point, prefix, tail, prec }` from
the beta.68 work. A cache hit is expressed as exactly that, and the existing path extends if needed
and rebuilds the derivatives.

⭐That path was **already gated**: `selfcheck_reference_reuse` asserts an extended orbit renders
bit-identically to a freshly picked one, with non-vacuity guards that reuse engaged and the orbit
grew. The new `orbit-cache` check is the same body with the reference sent through the store's own
write, lookup and load on the way — plus an **unaided** arm: a worker given no reuse hint must
report `from_disk`, because a lookup that silently misses builds fresh and renders the same picture,
which the identity arm cannot see.

## Cache design, as built

**Identity** — formula id, Julia flag + c, backend, build precision, exact reference point words →
one 64-bit id, one file. A longer orbit of the same identity replaces a shorter one; a shorter one
never replaces a longer one.

⭐⭐**Lookup by NEIGHBOURHOOD, BEFORE the pick.** The design as first written keyed the lookup on
the *picked* point — "lookup after `pick_reference`, so the saving is the orbit build". That was
wrong by numbers already in the tree: at 2.37e4000× the candidate scoring cost **113.7 s against
32.8 s for the orbit**. A cache that paid the pick to find its key would have left most of the
saving on the table, and would have missed every nearby view whose pick landed on a different
point. So `recompute_worker` (and the staged cold start) consult the store *first*, admitting any
entry of the right identity whose point lies inside the view at a usable precision — **the same
test `try_reuse_reference` applies** (`reuse_drift`, one function, passed to the store as a
closure), so an entry can never be selected by one rule and refused by another. Among admissible
entries: the longest orbit, ties to the closest point.

**Order of preference in the worker**: the live view's own reference (memory) → the disk → a fresh
pick and build. Every result that is worth a file is offered back to the store on a detached
thread, so the worker returns at once.

**Write** — a NEW identity only past `ORBIT_CACHE_MIN_BUILD_MS` (1 s of orbit build; a 5 ms orbit
is not worth a file, and a dive at moderate depth would otherwise write one per rebuild). An
existing identity is replaced whenever the new orbit is longer, however quick the extension.
Temp name then rename, so an interrupted write cannot leave a half-file.

⭐⭐**Evict by BUILD COST, not recency.** LRU was the first draft, and it is wrong for this cache: a
dive writes a stream of cheap entries, and under LRU they would evict the hour-long orbit visited
last week before any of themselves. The value of an entry is the time it saves, and
`orbit_len × precision²` is that time up to a machine constant; the cheapest goes first, recency
breaks ties. An entry that would not survive its own eviction pass is refused before it is written.

**On/off** — ON for an ordinary launch, **OFF for every task invocation** (`is_task_invocation`):
a cached orbit makes a timed run look faster than the code is, a determinism twin would hit its own
first render, and the corpus must build what it measures. `--orbit-cache` / `--no-orbit-cache`
outrank the default.

### User-facing controls (author's requirement, 2026-09-08)

File ▸ Settings ▸ **Reference cache…** — treated as a browser cache and visible as one: **where it
is** (path shown, "Open folder"), **how big it is** (usage bar against the limit, entry count, the
most valuable entries listed), **the limit** (persisted in the session as `orbit_cache_mb`, applied
at once), and **clear it** (red fill and an inline confirmation — clearing costs time, never data;
`UI-DESIGN.md` §8.2). Photographed by `--uitest` (`reference-cache`, seeded with one real entry).

## ⛔The correction this design rests on

At beta.68 I measured reference reuse for *exports* at 1e30 — **175 ms fresh vs 159 ms extending,
~10%** — and recommended against saving orbits. Right for 1e30, where the reference is a sliver of
the render; wrong at e60205, where it is essentially all of it. *A cost measured as a FRACTION
expires when its denominator moves.*

## Plan

1. ✅**The gate, first.** `orbit-cache` in the default `--selftest` sweep, at 1e30.
2. ✅**SA + BLA rebuild at depth** — settled by the e4000 measurement above; the e60205 split is
   pending from the trace run.
3. ✅**Grow `refcache_persist`** — done, with the two corrections recorded above (lookup before the
   pick; cost-aware eviction).
4. ✅**The controls.**
5. ▶**Then measure the thing that started this**: cold vs cached time to reach
   `validation/spiral-9.98e60205.fdn`, and put the number here.

| 9.98e60205×, 200,193 bits | cold | cached |
|---|---|---|
| export-grade, 2,000,000 ask (`--selftest deep-location`): pick **5.77 h** (101 survivors, deep-perturb scoring) + orbit **3.02 h** (escaped at 1,645,896) + BLA **2.0 s**, SA skipped — **8.8 h** | measured 2026-09-08 | *(pending — the cache-on pair)* |
| live, 256,000 cap, same ask (`--shot`, the author's return-visit scenario) | *(pending)* | *(pending)* |

⭐**The pick is 1.9× the orbit at this depth**, so a lookup keyed on the picked point would have
paid 5.8 of the 8.8 hours to find its key. ⚠The `deep-location` banner's "30-60+ minutes" was the
author's LIVE experience; the check itself builds export-grade and takes ~9 hours here.

## Related, and explicitly out of scope

- **TODO T2d — self-referential orbit compression.** A future optimisation *on top* of this, and its
  own note is emphatic that it is **not** the low-risk item to take first: compression carries a
  controlled error, so a loaded cache stops matching a freshly built reference. That is precisely
  the identity the `orbit-cache` gate enforces, so this design landed uncompressed first.
- **Sharing.** Nothing here decides whether an orbit should travel with a `.fdn` (a `.orbit`
  sidecar). The cache solves "get back to where I was"; it does not solve "give someone else an hour
  of my compute". Different problem, different decision.
- **Two instances.** The index is per process; a second Fractadyne writing to the same directory is
  not seen until the next launch (or a Clear). Harmless — a missed entry is a rebuild — and rare.
