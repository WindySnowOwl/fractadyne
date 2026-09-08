# Reference-orbit cache

Returning to a location you have already visited at extreme depth costs **30–60+ minutes** — all of
it spent rebuilding a reference orbit you already had. This is the design for keeping it.

Status at `v0.2.41-beta.77`: the two codecs are built and tested. The gate, the cache and its
controls are not. See **Plan**.

## ⛔Read this first: `refcache_persist` already exists

`crates/fractadyne-app/src/refcache_persist.rs` already saves a deep reference on exit and restores
it on launch (`last_reference.bin`). **This design extends that module; it does not sit beside it.**

I did not find it before writing a second codec — a real process miss, recorded here so the next
person starts from the existing code rather than repeating the search.

| | today (`refcache_persist`) | what this design adds |
|---|---|---|
| entries | **one** — the last view | many, LRU against a budget |
| match rule | exact view centre + zoom exponent | the **reference point** → the whole neighbourhood |
| point encoding | decimal string | exact mantissa words |
| extendable | no tail stored | tail stored → `extend_reference_orbit` resumes |
| SA / BLA | stored | rebuilt on load (see below) |
| user control | none | path, usage, limit, clear |

So the existing module solves *"resume the session I closed"*. It does not solve *"go back to a
place I visited earlier, and explore around it"*, which is the reported need.

## The opportunity, in numbers

The cost is the orbit: at 9.98e60205× that is ~200,000-bit arithmetic run for up to two million
iterations. What the GPU consumes from it is `[f32; 4]` per iteration — **16 bytes**.

| artifact | iterations | size |
|---|---|---|
| live view (`LIVE_REF_CAP` = 256,000) | 256,000 | **4.1 MB** |
| full quality | 2,000,000 | 32 MB |
| exact point + extendable tail | — | ~150 KB |

At 4 MB per live orbit a 1 GB budget holds ~250 extreme locations.

⭐**One orbit serves a NEIGHBOURHOOD.** Perturbation renders every pixel as a delta from the
reference, so a cached orbit covers nearby views too — come back, then zoom into a *different*
region nearby, with no rebuild. Going *deeper* needs more iterations, which is what the tail is for.

## What is built (beta.77)

**`fractadyne_core::bfbytes`** — exact `BigFloat` bytes (sign, exponent, mantissa words).
⚠**Not because decimal is wrong.** `refcache_persist` already notes that astro-float's
`to_string()`/parse round-trip is not bit-stable at this precision, and that is *benign* for a
restored point: a last-bit difference at 200,000 bits is ~2^-200000, far below the pixel spacing.
The reasons exactness matters here are **key stability** (a neighbourhood cache is keyed on the
point; an unstable encoding means every lookup misses), **size** (~25 KB vs ~60,000 digits), and
avoiding a lenient `FromStr` in a load-bearing path.

**`render::orbit_blob`** — orbit + everything determining what it is (formula, Julia c, backend,
precision, iteration count, exact point, tail) under a digest over the whole file. Refuse, never
repair. Pinned by flipping one bit at every byte offset in turn, plus truncation, trailing bytes,
foreign magic, foreign version, and an absurd length that must not become an allocation.

⚠**SA and BLA deliberately not stored** (unlike `refcache_persist`, which does store them): they are
derived, cheap next to the orbit, and the BLA depends on live colouring settings
(`bla_stripe_freq`, `bla_trap_type`) — caching it drags those into the cache identity for no gain.
⛔This is a **deliberate divergence** from the existing module and needs measuring before it is
settled: if SA+BLA rebuild turns out to be slow at e60205, store them and key on the colouring.

⭐**`backend` is part of the identity** — `OrbitTail::backend` exists because an extension must
resume in the backend that started it; the same reasoning makes it part of what an orbit *is*.

## The seam that makes this low-risk

`RecomputeInputs` already carries `reuse: Option<ReuseRef>` = `{ point, prefix, tail, prec }` from
the beta.68 work. A cache hit is expressed as exactly that, and the existing path extends if needed
and rebuilds the derivatives.

⭐That path is **already gated**: `selfcheck_reference_reuse` asserts an extended orbit renders
bit-identically to a freshly picked one, with non-vacuity guards that reuse engaged and the orbit
grew.

## Cache design

**Key** — formula id, Julia flag + c, exact reference point words, precision, backend.
Stored alongside: iteration count, orbit length, `partial`.

⭐**Keyed on the REFERENCE POINT, not the view centre** — that is what makes "come back, then zoom
nearby" a hit. Consequence: lookup happens *after* `pick_reference`, so the saving is the orbit
build, not the point selection. At extreme depth the orbit is essentially all of the cost.

**Usable when** stored precision ≥ required, and either stored iterations ≥ required or a tail is
present and can be extended to it.

**Write** — only past a build-time threshold (a 5 ms orbit is not worth a file). Temp name then
rename, so an interrupted write cannot leave a half-file.

**Evict** — LRU against a size budget.

### User-facing controls (author's requirement, 2026-09-08)

Treated as a browser cache and visible as one: **where it is** (path shown and openable), **how big
it is** (usage against the limit), **the limit** (user-settable, persisted), and **clear it** (a
button with the confirmation weight of any destructive action, `UI-DESIGN.md` §8.2 — clearing costs
time, never data).

## ⛔The correction this design rests on

At beta.68 I measured reference reuse for *exports* at 1e30 — **175 ms fresh vs 159 ms extending,
~10%** — and recommended against saving orbits. Right for 1e30, where the reference is a sliver of
the render; wrong at e60205, where it is essentially all of it. *A cost measured as a FRACTION
expires when its denominator moves.*

## Plan

1. **The gate, first.** A decoded orbit must render **bit-identically** to a freshly built one, with
   a non-vacuity assertion that the reuse path engaged. At 1e30 (~175 ms fresh) so it runs in the
   default `--selftest` sweep even though the payoff is at e60205.
2. **Measure SA + BLA rebuild time at depth**, to settle the divergence flagged above.
3. **Grow `refcache_persist`** into the multi-entry cache: key, lookup after `pick_reference`,
   threshold write, atomic write, LRU. Bump its `FORMAT_VERSION`.
4. **The controls.**
5. **Then measure the thing that started this**: cold vs cached time to reach
   `validation/spiral-9.98e60205.fdn`, and put the number here.

## Related, and explicitly out of scope

- **TODO T2d — self-referential orbit compression.** A future optimisation *on top* of this, and its
  own note is emphatic that it is **not** the low-risk item to take first: compression carries a
  controlled error, so a loaded cache stops matching a freshly built reference. That is precisely
  the identity the gate in step 1 enforces, so this design must land uncompressed first.
- **Sharing.** Nothing here decides whether an orbit should travel with a `.fdn` (a `.orbit`
  sidecar). The cache solves "get back to where I was"; it does not solve "give someone else an hour
  of my compute". Different problem, different decision.
