# Iteration chunking for mode 2 (floatexp)

Status: **design only, nothing implemented.** Written 2026-08-17 after the field device loss in
TODO.md § Open bugs ("DEVICE LOSS zooming HOME from a deep view").

## 1. Why this exists

The frame-cost controller cannot protect a mode-2 live frame, and not because it is mistuned.

Its only actuator is **resolution**. `render.rs` computes `want = px · gpu_iter` and then
`budget_res_scale = sqrt(budget / want)`; the comment there says it plainly — "reduce the
iteration-texture resolution". It never lowers `gpu_iter`. So the budget can only ever remove
*pixels*.

At mode 2 with a large iteration ask against a long reference, frame cost is dominated by the
**dependent iteration chain**, not by pixel count. Removing pixels therefore does not move the clock.
Measured, from the 2026-08-18 loss (build 1675, RTX 3080):

| frame | resolution | steps | wall | note |
|---|---|---|---|---|
| 1812–1813 | ~417×571 | 5.992e10 | 1001 ms | budget 6.000e10 |
| 1814 | — | — | — | lethal band → emergency retreat to 2.387e10 |
| 1814–1815 | 264×361 | 2.383e10 | **1003, 1012 ms** | 2.5× fewer steps, **+0 ms** |

`264 · 361 · 250000 = 2.383e10` exactly, which is how we know the retreat acted on resolution alone.
The controller did the right thing on the first lethal reading and it made no difference, because the
knob is orthogonal to the cost.

**Two fixes this forecloses.** Shrinking resolution harder buys nothing — the user was already at
264×361 of a full window, looking at a badly degraded frame that still lost the device. Lowering the
budget target is the same knob and equally inert.

The remaining lever is the iteration count per dispatch. `chunk_over` currently gates on
`chunk_mode.is_direct() || chunk_mode == RenderMode::Df32Pert`, so mode 2 has no way to split an
iteration range across frames. That gap is this document.

## 2. What must NOT be done

**Do not cap the iteration ask.** It is the obvious shortcut and it is known-bad: capping silently
rendered deep validation-corpus locations interior-black, because their structure escapes above the
cap, and it broke the "same iterations, both apps" contract with Fraktaler-3. The existing comment in
`current_export_request_with_ref` records this. Auto-iter off is an explicit instruction and the count
is honoured verbatim.

Note the asymmetry that makes chunking the only honest answer: the app's live policy is *honour the
iteration count, sacrifice resolution*. That is a deliberate and correct choice — and it is exactly
why the live view cannot escape a chain-latency wall without splitting the range.

## 3. How mode 0 already does it

Verified by reading `fs_iterate_chunk` and `ChunkState`.

Three `Rgba32Float` render attachments, ping-ponged (pass *k* reads set *k*&1, writes the other),
sized to the view and dropped on resize (`chunk_state = None` — state textures are size-matched):

| attachment | while RUNNING | at ESCAPE |
|---|---|---|
| `st_z` (vec4) | full `z` as df32 — re.hi, re.lo, im.hi, im.lo | full `z` (df32) |
| `st_dz` (vec4) | `δz` as df32 | display derivative's mantissa |
| `st_info` (vec4) | ch0 `status·2^20 + (iter>>12)`, ch1 `iter & 4095`, ch2 `ref_n`, ch3 derivative floatexp exponent | ch2 `smit`, ch3 same |

Two details worth carrying forward. `iter` is split across two channels because a `u32` bitcast can
be flushed as a denormal by a render target — the values are kept as small exact integers instead.
And mode 0 **already carries a floatexp derivative** through this path (mantissa in `st_dz`, exponent
in `st_info.ch3`), so extended-range state in the chunk loop is not new ground.

## 4. The constraint that decides the design

Mode 2's running state, given `Fe { m: Cdf, e: i32 }` — a complex df32 mantissa with one **shared**
exponent:

| quantity | floats |
|---|---|
| `δz` mantissa (complex df32) | 4 |
| `δz` exponent (shared) | 1 |
| derivative mantissa (complex df32) | 4 |
| derivative exponent (shared) | 1 |
| `status` + `iter` (packed as mode 0 does) | 2 |
| `ref_n` | 1 |
| **total** | **13** |

The existing three attachments hold **12**. Mode 2 does not need to store full `z` (it is
reconstructible as `reference[ref_n] + δz`, and the loop fetches that orbit entry every iteration
anyway), which frees `st_z` — but the shortfall is still exactly one slot, because two floatexp
exponents compete for the single free `st_info.ch3`.

Packing both exponents into one f32 does **not** work at our depth range. An f32 holds exact integers
to 2^24, so two fields need ≤12 bits each; at 1e1105 the binary exponent is ≈ −3670, which already
exceeds a 12-bit signed field. Two 13-bit fields need 26 bits. It does not fit.

So there are exactly two candidate designs, and the choice is a real trade rather than a detail.

### Option A — a fourth attachment

Add a fourth `Rgba32Float` target for the extra state. Well within the 8-attachment limit.

*Cost:* one more full-resolution R32×4 surface read and written every chunk pass, on the path whose
whole purpose is to be cheap; plus a layout change to `ChunkState`, its bind-group layout, and the
resolve pass. *Benefit:* no precision change whatsoever — mode 2 chunked is bit-identical to mode 2
unchunked, which makes validation a straight equality check.

### Option B — store `δz`'s mantissa as plain f32, dropping the lo limbs

Drops 2 floats and fits in the existing three attachments with a slot spare.

*Cost:* a genuine precision reduction — **but only where the lo limbs currently survive.** On all
three Windows NVIDIA backends the error-free transforms are compiler-folded, so mode 0/2 mantissas
are already effectively f32 and this costs nothing measurable there. On AMD (RX 6800 XT, Vulkan/GL)
the transforms *are* preserved, so Option B would make chunked mode 2 measurably worse than unchunked
mode 2 on exactly the vendor that currently does the arithmetic properly. That also means the
validation gate could not be an equality check, and a tolerance gate on the chunk boundary is far
weaker.

**Recommendation: Option A.** (DECIDED — see §8 for the decision record, the measured VRAM figures behind it, and the concrete channel layout.) The bandwidth is a known, bounded cost on a path that is already
render-target bound, and it keeps the validation gate at bit-identical — which is the only gate strong
enough for this. Option B trades a hard guarantee for a soft one, and does it in the direction that
penalises the vendor whose arithmetic we just confirmed is the correct one.

## 5. Resume semantics

Per pixel, a chunk pass must resume mid-orbit exactly where the previous pass stopped:

- `δz` and the derivative restored from state, exponents included.
- `ref_n` restored — and **rebasing must be able to happen across a chunk boundary.** A pass that
  ends one iteration before a Zhuoran rebase, and a pass that ends one iteration after, must produce
  the same final pixel. This is the single most likely source of a chunk-boundary mismatch.
- `iter` restored so the escape iteration count is absolute, not per-chunk.
- BLA: each pass must build/own its BLA table. Precedent — the e100 fix (beta.101 `2896b5e`): glitch
  correction passes ran with `bla_on = 0` and an empty tree, which cost **0.04 Gsteps/s against the
  base pass's 174 in the same frame**. A chunk pass that silently loses its BLA would reproduce that
  exact pathology, and it would look like "chunking is slow" rather than "chunking is broken".

## 6. How this is gated

Chunking must be provably invisible in the output. The mode-0 work established the right bar and it
applies unchanged: **bit-identical selftests across a chunk boundary.**

1. A mode-2 view rendered unchunked vs. chunked into 2, 3 and 7 passes → identical decoded RGB.
   ⚠Compare decoded RGB, never file hashes: `--render` embeds metadata, so four identical renders
   produce four different sha256s.
2. The same, with the split placed deliberately *at* a rebase (see §5) and at an orbit-end wrap.
3. A view whose reference is shorter than the iteration ask (the 250k-against-119,563 shape from the
   field case), so pixels wrap the orbit several times across chunk boundaries.
4. Frame-time evidence, which is the whole point: the field view must hold a bounded frame time with
   `gpu_iter` split, where today it runs ~1 s per frame regardless of resolution.
5. `--livetest` grand tour 24/24 at zero drift, and the F3 corpus gate 20/20 — chunking must not
   perturb any existing result.

## 7. Scope estimate, honestly

This is not a small change. It touches the WGSL chunk entry point, the state encode/decode, the
`ChunkState` layout and bind groups, the resolve pass, the `chunk_over` gate and the per-frame chunk
budget arithmetic in `render.rs`, plus new selftests. The mode-0 equivalent was substantial and it had
the easier arithmetic.

Two things make it tractable: the ping-pong machinery, the info packing, and the resolve pass already
exist and are proven; and mode 0 already carries floatexp derivative state through them, so the
extended-range plumbing has precedent rather than needing invention.

It should be started deliberately, with the gates in §6 written before the shader work — a
half-migrated chunk path that renders *almost* the same is worse than no chunk path, because every
subsequent deep-zoom result becomes suspect.

## 8. Decision record and channel layout (2026-08-19)

**Chosen: Option A, as two fragment entry points in ONE WGSL module** sharing the state pack/unpack
helpers. Not two pipelines with two copies of the layout, and not one four-target pipeline for
everything.

### Why not the single four-target pipeline

It is simpler and keeps the state layout single-sourced, which was the original argument for it. The
numbers killed it. Chunk state is ping-ponged, so each `Rgba32Float` target costs `W·H·16·2`:

| resolution | 3 targets (today) | 4 targets | extra |
|---|---|---|---|
| 1920×1080 | 199 MB | 265 MB | +66 MB |
| 2560×1440 | 354 MB | 472 MB | +118 MB |
| 3840×2160 | 796 MB | 1,062 MB | +265 MB |

A single four-target pipeline makes direct/mode-0 carry a fourth full-resolution target it never
reads. At 4K that pushes chunk state past a gigabyte for nothing. (Live views are often
resolution-scaled and offline renders tile, so real figures are frequently smaller — the ratio is
what matters, and VRAM pressure here is already a live concern.)

Runtime compute is NOT the differentiator: the wasted write is ~33 MB per chunk pass at 1080p, so
~530 MB per settled frame at ~16 passes, ≈0.7 ms on a 3080 against a 400 ms budget. Noise. VRAM is
the whole difference.

### Why the duplication worry was overstated

The concern was two drifting copies of the most safety-critical state layout in the renderer. But the
module ALREADY has six fragment entry points (`fs_iterate`, `fs_iterate_chunk`, `fs_resolve`,
`fs_seed`, `fs_color`, `fs_gputest`), so a second chunk entry point is the existing idiom rather than
a new pattern — and only the output struct and the attachment list differ. The state encoding stays
in shared helpers.

It also makes the device question moot: mode-0 chunking keeps exactly today's 48-byte requirement and
only mode 2 asks for 64, so a hypothetical 48–63-byte device loses nothing it has now.

### The channel layout, derived from what mode 0 actually does

Read from the real save/resume in `fs_iterate_chunk`, not estimated. Mode 0 while RUNNING stores
`st_z` ← δz (df32), `st_dz` ← derivative MANTISSA (complex df32), `st_info` ← `(status+iter_hi,
iter_lo, ref_n, D.e)`; at ESCAPE it swaps `st_z` → full `z` and `ch2` → `smit`. So mode 0 already
carries a floatexp derivative through three targets — the extended-range plumbing is precedent, as
§3 said.

Mode 2 needs the same plus a second exponent (δz is `Fe`, not `Cdf`). Four targets = 16 channels:

| target | while RUNNING | at ESCAPE |
|---|---|---|
| T0 `st_z` | δz mantissa (`Cdf`: re.hi, re.lo, im.hi, im.lo) | **full `z` (df32)** — contract |
| T1 `st_dz` | derivative mantissa (`Cdf`) | **derivative mantissa** — contract |
| T2 `st_info` | ch0 `status·2^20 + iter>>12`, ch1 `iter & 4095`, ch2 `ref_n`, ch3 **δz exponent** | ch2 **`smit`**, ch3 **derivative exponent** — contract |
| T3 `st_exp` | ch0 **derivative exponent**, ch1–3 spare | unused |

13 of 16 channels used while running.

⚠⚠**THE ESCAPE LAYOUT IS A CONTRACT WITH `fs_resolve`, NOT A FREE CHOICE — and the first draft of this
table got it wrong.** `fs_resolve` is deliberately mode-agnostic: its own comment records that "both
modes store the FULL z (df32) in `st_z` at escape and the display derivative's mantissa in `st_dz`;
info ch2 = smit, ch3 = the derivative's floatexp exponent (0 for direct…) — so the shading below is
mode-agnostic". It reads the derivative exponent as `sm.w`, i.e. `st_info.ch3`. An earlier version of
this section parked the derivative exponent in `T3.ch0` unconditionally, which would have left resolve
reading a δz exponent (or zero) as the DE exponent and quietly corrupted distance-estimate shading and
relief lighting on every chunked mode-2 escape — output wrong in a way the bit-identity gate WOULD
catch, but only after the whole shader was written.
So: while RUNNING, `st_info.ch3` carries the δz exponent and `T3.ch0` carries the derivative's; at
ESCAPE, `st_info.ch3` reverts to the derivative's exponent and `T3` goes unused. **`fs_resolve` then
needs no change at all**, which is one fewer thing the implementation touches.

⚠**The three existing targets already mean different things per mode**, which is why a mode-2 layout
being different again is normal rather than a smell — but §3's table shows only the DIRECT (mode 1)
usage and should be read as such:

| | T0 `st_z` | T1 `st_dz` | T2 ch3 |
|---|---|---|---|
| direct (mode 1) | `z` (df32) | derivative (df32) | `0.0` |
| mode 0 | δz running / full `z` escaped | derivative **mantissa** | `D.e` |

Only the RUNNING half is mode-private; the ESCAPE half is the shared contract above. The two exponents are in separate channels because they cannot share one: an
f32 holds integers exactly only to 2^24, so two fields get 12 bits each, and at 1e1105 the binary
exponent is already ≈ −3670 and needs 13.

⚠Keep `iter` split across two channels exactly as mode 0 does. The reason is recorded at
`info_pack`: a `u32` bitcast can be flushed as a denormal by a render target, so both halves are held
as small exact integers instead.

### The state budget holds, because `chunk_over` already narrows the scope

Reading the whole mode-2 loop in `fs_iterate` (lines 718–1052) turned up per-pixel state the 13-float
budget above does not include:

- **Phoenix (formula 8)** carries `dz_prev: Fe` AND `Dprev: Fe` — ten more values. Alone this would
  blow four targets.
- **Aux coloring** carries an accumulator (`trap`, `tia_sum`, `sac_sum`, `n`, `prev_abs`) — ~5 floats.

Neither is a problem, because the chunked path is already gated to a narrower scope than the general
iterate loop, and the mode-2 body inherits the same gate:

```rust
let chunk_over = (chunk_mode.is_direct() || chunk_mode == RenderMode::Df32Pert)
    && fractal.formula_id() <= 3                 // holomorphic only ⇒ NO Phoenix
    && !self.coloring.color_method.needs_aux()   // aux coloring OFF
    && self.perf.chunk_ok && !offscreen && ...
```

Its comment says so directly: "Gated to the chunk shader's scope: holomorphic formulas 0..3, aux
coloring off (the chunk pass carries no orbit statistics)". So the mode-2 chunk body needs **formulas
0–3 only, no `dz_prev`/`Dprev`, no aux accumulator** — and the 13-of-16 layout stands.

⚠Consequences for the implementation, all of them simplifying:
- The BLA skip loop is `formula == 0` only, exactly as in `fs_iterate`.
- Tricorn (4) and the abs families (5–7) are out of scope; do not port their δ-updates.
- Glitch detection: the live path never enables it (`glitch_on == 0` there) and mode-0 chunking is
  additionally gated glitch-free. Mode 2's chunk body should still return `GLITCH_SENTINEL` if it is
  ever on, rather than silently diverging from `fs_iterate`.
- The per-pass counters (`n_rebase`, `n_ext`, `n_bla`) are diagnostics committed at exit; per-pass
  commits change their totals' meaning slightly, which is acceptable but worth not being surprised by.

### Remaining slices, in order

1. ~~**Plumbing + entry point together**~~ — **DONE.** `fs_iterate_chunk_fe` + a `ChunkOut4` output
   struct + `@group(1) @binding(3) st_exp` in the same module; `state_bind_group_layout_n`,
   `make_state_textures(.., targets)` and a slice-taking `make_state_bg` on the Rust side; a
   `chunk_fe_pipeline`/`resolve_fe_pipeline`/`state_bgl4` trio on `Renderer` built whenever the
   device grants 64 bytes/sample; and `prepare` selecting the trio on `self.mode == 2`.
   `chunk_over` still excludes mode 2, so this is behaviourally inert. Verified: full suite
   121/121 + 17/17 (including `iter-chunk`'s five mode-0 bit-identity checks), and the app boots
   and renders — which is what proves naga validated the new entry point, since the pipeline is
   built in `Renderer::new`.
2. ~~**The gates** (§6)~~ — **DONE, and they run OFFLINE, which reorders the plan.** §8 put the live
   `chunk_over` flip second and the gates third; that is backwards, because a bit-identity gate needs
   a *dispatchable* chunked mode 2 and the live path cannot be driven from a selftest. The mode-0
   gates already run through the offline `render_iter_chunked`, so mode 2 was added to that path's
   scope (four-target plumbing, `fs_iterate_chunk_fe`, the BLA tree uploaded and `bla_on` passed
   through) and the `iter-chunk` group grew five rows. Live behaviour is still untouched.
3. **Flip the `chunk_over` gate** for mode 2, behind `chunking_mode2_available` — the only remaining
   step, and now the only one whose blast radius is the live view.

### What the gates actually cover (2026-08-19)

| row | mode | rebases | BLA skips | split |
|---|---|---|---|---|
| corpus loc 07, 1.3e30× | 2 | 187,097 | 124 | 2, 3 and 7 passes |
| 38-digit nucleus, 1.3e30× (interior-filled) | 2 | 361,830 | 12,325,890 | 7 passes |
| 97-sample reference, 21k ask | 2 | 1,790,800 | — (BLA off) | 8 passes |

All bit-identical, 0 texels differing. The counters are **identical across the 2-, 3- and 7-pass
splits** (187,097 and 124 every time), which is a stronger statement than the pixel equality: the
chunk boundary does not perturb the rebase or skip *sequence* at all. The interior row is the one
that exercises §5 hardest — every pixel runs all 21,000 iterations across seven boundaries with BLA
live. The 97-sample row is the field case's shape (an ask far above the reference length) in
miniature, at ~1.8M orbit wraps.

⚠**Two traps the gates had to be repaired for, both worth remembering.**

*A deep magnification is not a deep test.* The first draft used the group's 15-digit seahorse
coordinate at 1e30×. That coordinate is garbage past ~1e15×: its reference escaped after 3,090
samples, SA seeded every pixel at 3,088, the pixels escaped ~2 iterations later, and the chunked
render agreed **trivially** — zero rebases, zero BLA skips, nothing of the chunk path exercised, and
four green checks. Mode-2 rows use corpus loc 07 (44 digits) and the 38-digit nucleus instead.

*Bit-identity alone cannot certify that BLA survived chunking.* If BLA silently switched off in
**both** renders they would still agree, and the chunked path would be running the beta.101 e100
pathology behind a green gate. So the untruncated mode-2 rows additionally require `bla_skip > 0`,
and every row prints its rebase and skip counts. This is what caught the seahorse problem above —
the equality check was perfectly happy.

The rows also assert the mode they claim: `make`'s `mag` argument is 3/4 of the view magnification,
so a mode-2 row written at 1e28 would silently render in mode 0 and the group would quietly become
five more mode-0 checks.

### What slice 1 changed about the plan (three corrections, found by writing it)

**`fs_resolve` needs no WGSL change — but it does need a second PIPELINE.** §8 said resolve was
untouched, and its *source* is. A bind group must match its pipeline's layout exactly (a layout may
declare bindings the entry point never reads, but a 4-entry group cannot be set on a 3-entry
pipeline), so the same `fs_resolve` entry point is compiled twice: once against `state_bgl` and once
against `state_bgl4`. One extra pipeline object, zero extra per-frame work, and it beats carrying a
second narrow bind group per ping-pong side.

**Glitch detection is omitted, and the GATE is what guarantees that.** §8 suggested the mode-2 chunk
body "should still return `GLITCH_SENTINEL` if it is ever on". It cannot: a chunk pass writes state,
not a `FragOut`, so a sentinel would need a fourth status that `fs_resolve` learns to translate —
which reopens the resolve contract this design worked to keep closed. The body therefore does what
`fs_iterate_chunk`'s mode-0 branch already does: no glitch code at all, with the app's `chunk_over`
required to keep `glitch_on == 0` (live never enables it). Recorded here as a decision rather than
left as a silent divergence from the paragraph above.

**⚠A BLA skip can carry `iter` past the pass's `stop`, so slice 2's cost model over-counts.** The
skip is the first thing the loop tries, and `iter += span` can land well beyond `end_iter` in one
step. That is harmless for correctness — the skip sequence is the same one the unchunked loop takes,
and the next pass finds `stop` already behind it and passes the state through — and it is nearly
free in time, because a span of 2^l costs one iteration's work. But `render.rs` prices a chunk frame
as `steps = px · (end − cur)`, and mode 0 had no BLA, so this is the first time the iteration axis
and the cost axis come apart. Slice 2 must not assume the two are the same number.

### ⚠Open hazard for slice 2: the progression signature ignores the reference

`chunk_sig` is `(center, magnification, gpu_iter, resolution, ss)` — it does **not** include the
reference orbit. A reference rebuilt or extended while the view sits settled changes `orbit_len`
mid-progression, and `orbit_len` feeds both the per-pass BLA table and the `ref_n + 1 >= orbit_len`
rebase trigger. Mode 0 is exposed to the second of those already; mode 2 is exposed to both, and mode
2 is exactly where deep views extend their references mid-settle (the e72/e82/e94 family). A pass
that resumes `ref_n` against a *different* orbit than the pass that stored it is not covered by any
gate in §6. Decide in slice 2 whether the signature grows an orbit identity or the progression is
explicitly restarted on an orbit swap.

## 9. The live flip is BUILT AND PROVEN CORRECT, but HELD on a motion-presentation regression (2026-08-19)

Slice 3 (`chunk_over` accepting `RenderMode::Floatexp` behind `chunk_fe_ok`, plus `chunk_sig`
growing the reference length) is written, builds, and passes every gate:

- suite **126 checks + 17 goldens**;
- `--livetest` grand tour **24/24 checkpoints, 0 drifted, 0 new** vs the blessed baseline — including
  six deep mode-2 holds (1e55 → 1e95) at explicit counts up to 4,000,000, all at +0.0pt;
- `--autodive 32 --autodive-home 3`: reached 1e32x, 6 lethal readings, peak measured iterate
  1348 ms, **no device lost**, no crash file.

And it does the thing it was built for. Traced at the user's saved session (spar, 2^341.5x, explicit
4,000,000, auto-iter off), a SETTLED view now walks the cursor to the full 4,000,000 by frame 39 —
**3.4 s** after settling, across ~35 bounded passes — where before it was one unbounded dispatch.

**It is held anyway, at the user's decision, because it degrades the MOVING picture at mode 2.**

### The mechanism, traced not guessed

`FRACTADYNE_TRACE=tile` on that session:

```
chunk f=1 cur=0 step=235    gpu_iter=4000000 interacting=true
chunk f=4 cur=0 step=256    gpu_iter=4000000 interacting=true
...
chunk f=39 cur=4000000 step=208503 interacting=false
```

While `interacting`, the progression restarts every frame (`cur=0`), so a moving frame renders only
`step` of the ask — 312,755 of 4,000,000 (7.8%) with a measured budget, 256 (0.006%) at bootstrap or
after a budget collapse. During motion the app holds and reprojects the last good frame but takes one
REAL refresh frame every `REFRESH_OCTAVES` to stream detail; that refresh frame is now chunked, and
it then **becomes the frozen texture** reprojected until the next refresh. Under-iterated content gets
latched and held. Reported from the field as "the interior regions look mostly like noise".

⚠It is NOT an artifact that can be rendered away. A frame that can only afford 312k of 4M iterations
at 2^341x genuinely looks like that, and pixels are not the lever (that is the same finding in §1 that
motivated this whole document). The pre-flip picture looked complete during motion only because it
ignored the frame budget — those were the 1000 ms+ frames that lost the device.

### ⛔The obvious fix is REFUTED — do not implement it

"Refresh only when the pass can cover a useful FRACTION of the ask" is wrong, and cheap to disprove.
Measured at corpus loc 07, 1e30x, `--compare` against the full 21,000-iteration render:

| iterations rendered | vs full 21,000 |
|---|---|
| 1,638 (7.8% of the ask) | **2 of 160,000 pixels differ** |
| 5,250 (25%) | 0 differ |
| 10,500 (50%) | 0 differ |

That location resolves fully below 1,638 iterations — the 21,000 ask is simply OVERSIZED, which is
the common case whenever a user sets one large count to cover many depths. A fraction-of-ask gate
would hold and go blocky there for no reason, re-introducing precisely the regression recorded at
`reuse_hold`: floatexp used to hold THROUGHOUT, so a fast dive past ~1e28x "went increasingly blocky
until you stopped to let it settle".

### What the gate actually has to key on, and why it is not free

The distinguishing signal is not the fraction of the ask but **whether pixels are still unresolved at
the iterations the frame can afford**. The app already measures that (`CTR_MAXITER` via
`maxiter_sink` → `capped_frac`).

⚠**But `capped_frac` is deliberately CLEARED on every interacting frame** (`render.rs`, the
`if interacting` block): "a moving frame's reading describes another view". That clear is
load-bearing — comparing one view's capped fraction against another's falsely reads as "the raise
did not help", latches `iter_plateau`, and stuck a real session at boost 1.0 on a black screen. So
the signal is unavailable exactly where the gate needs it, and carrying the last settled reading
through motion is the "heuristic tuned at one depth" shape this codebase has been bitten by before
([[topic-spar-family]]).

Whoever picks this up: the hold/refresh path is the most regression-scarred code in the renderer (its
own comments record the dual-Julia reprojection latch, the prefer-detail freeze that disabled the
refresh cadence entirely, the resize squash, and the e590 stepping regression). It wants a designed
gate with its own selftest, not a condition bolted onto `reuse_hold`.

### Scope, honestly

The regression is confined to CONTINUOUS motion that never settles, and it resolves in ~3.4 s once
motion stops. `--autodive` is unpaced by construction, so it displays that worst frame forever; the
paced grand tour settles and matches the blessed baseline exactly. That is why the live gate is green
and the defect is still real — **the checkpoints measure SETTLED results and say nothing about the
frames between them**, which is the same "measures what is RENDERED, not what is DISPLAYED" trap as
[[topic-pixellation-settle]].

The flip is saved as `local/mode2-live-flip.patch`. Slices 1 and 2 are committed, pushed and
independent of it: the four-target shader path and the offline bit-identity gates stand on their own.

### ⚠Also observed, unrelated and unfiled

During the grand-tour run the watchdog reported `possible hang: no activity for 130s — building
reference [export]: iter=4000000 prec=379`. The tour passed and cold deep reference builds are known
to be slow, but a 130-second silent stretch on the export path deserves its own look.

## 10. Motion presentation: the three options, and why only one survives (2026-08-19)

§9 leaves one question: what SHOULD a moving mode-2 frame show when the ask is 4,000,000 and the
frame budget affords 312,755? There are exactly three answers, and two are already known-bad from
this repo's own history.

At 2^341x with a 4M ask, fresh complete detail costs ~1 s per frame. That is not a tuning failure —
pixels are not the lever (§1), so no budget setting produces "fresh" and "complete" and "fast" at
once. Each option below is a choice about which of those three to give up.

### Option A — show the partial frame (what the held flip does today)

Give up COMPLETE. The refresh frame renders `step` of the ask and is displayed.
⛔**Rejected: this is the reported defect.** At depth most pixels need most of their iterations, so a
7.8% frame is interior colour plus sparse escapes — and because that frame becomes the frozen
texture, it is reprojected until the next refresh. Field description: "the interior regions look
mostly like noise".

### Option B — hold instead of refreshing when the pass would be partial

Give up FRESH. Keep reprojecting the last complete frame; skip the refresh.
⛔**Rejected, and this is the important one: it re-introduces a FIELD-REPORTED bug.** The comment at
`render.rs` ("Prefer detail does NOT add a freeze here") records that its first cut froze every
interacting frame, "which disabled the reuse-hold's REFRESH_OCTAVES cadence entirely, so a continuous
dive just magnified the frozen frame into ever-larger blocks (field report: giant pixels at 3.5e12x
despite the toggle)".
⚠A gate keyed on "would this pass be partial" holds FOREVER while moving, at any view where
`chunk_over` is true — because the progression restarts every frame, so it never completes during
motion. That is exactly the always-freeze shape above. Trading noise for a known regression is not a
fix, and the fraction-of-ask variant that would narrow this option was already refuted by measurement
in §9.

### Option C — spread ONE refresh across several frames at a PINNED view ✅

Give up nothing visible; pay a latency. When a refresh is due, snapshot the view and run its chunk
progression across successive frames while the display keeps reprojecting the PREVIOUS complete
frozen texture. Adopt the result as the new frozen texture only when the progression finishes.

- Each dispatch stays inside the frame budget ⇒ the device-loss fix is preserved.
- The display never shows partial content ⇒ fixes A.
- The refresh cadence still fires and still lands real detail ⇒ avoids B.
- Cost: a refresh lands a few frames after it was requested, so streamed detail lags the dive
  slightly. That is the only thing given up.

⭐**The reframe that makes this obviously right:** before the flip, mode 2 already paid ~1 s for a
complete deep motion frame — that is exactly what lost the device. Option C pays the same total work
for the same picture, split into watchdog-safe pieces. **It is strictly better than the pre-flip
status quo**, not a new compromise.

Precedent exists in-tree, so this is not invention: the offline `render_iter_chunked` already submits
one bounded pass per range and polls between them, and beta.81's hold-prefetch already builds
references DURING a glide and installs them when ready. Option C applies the same pattern to the
refresh frame's pixels instead of its reference.

⛔**Do NOT implement C by blocking the main thread** — submit + `poll(Wait)` inside the paint callback
to finish the progression in one frame gives the right picture and reproduces the pre-flip freeze,
and blocking main inside a wgpu wait is the specific shape where a device loss can only be recovered
by the device-lost CALLBACK (see the beta.29/30 wedge notes). The passes must land across frames.

### What implementing C requires

1. A per-view "refresh in flight" state: the pinned view signature, the chunk cursor, and the frame
   it started on. The pinned signature is what stops `chunk_sig` resetting the cursor while the live
   view keeps moving — the reason the progression cannot accumulate today.
2. Present-gating for its duration, reusing the existing gate rather than adding a second one
   (⚠beta.103: a gate that never drops means the finished frame is never shown — whatever engages it
   needs a proven drop path, and a completed grid repeating its final rect is how that was missed).
3. Adopt-on-completion: update `frozen_l2` / `frozen_at` / `frozen_upp_l2` only when the progression
   finished, so a partial refresh can never become the held frame.
4. An abandon path: if the view moves far enough that the pinned refresh is worthless before it
   completes, drop it and start a new one rather than landing stale detail.

### How it has to be gated

⚠**The existing live gate cannot see this defect** — `--livetest` checkpoints measure SETTLED results,
which is why the flip passes 24/24 with the regression present. A motion-presentation change needs a
gate that samples frames DURING motion. `--autodive 32 --autodive-home 3` reaches the regime (⚠`22`
does not: 1e22 is below the 1e28 mode-2 threshold — measured 0 mode-2 frames), and the honest signal
is `FRACTADYNE_TRACE=tile` `chunk f=… cur=… step=…` plus whether a partial progression was ever
adopted, not the checkpoint table.

## 11. Option C, engineered against the actual code (2026-08-19)

§10 chose WHAT to build; this section is the result of walking §10 through every line it has to
touch, before writing any of it. Everything below cites the tree at `903bab9` (slice 3, rebased
onto `02f96a3`).

### The one fact that makes the implementation small

`build_params` is FULLY PARAMETERIZED on the view (`center_bf/center/span/magnification/log2mag`
are arguments), and the frozen latch at the end records whatever view was passed in. Nothing in the
live path renders a non-live view today — but nothing prevents it either. So a pinned refresh is
not a second render path: it is the SAME frame build, with the view inputs shadowed at the top of
`build_params` and the latch deferred. The display side needs literally nothing new: the present
gate (`hold_copy`/`display_hold`/`hold_uv`) already displays a snapshot while chunk passes compose
into the live G-buffer — that combination is proven in the settled prefer-detail gate — and the
freeze math already computes a per-frame transform that tracks a moving view over frozen content.

### The frame shapes

A view is in exactly one of these shapes per frame while `chunk_over`:

- **Hold** (unchanged): `reuse_hold` true → reproject the texture, no iterate.
- **Pin-start**: interacting, a refresh is due (`reuse_hold` false), the frame really renders
  (`reproject.is_none()` survives the freeze block), `step < gpu_iter`, a frozen frame exists.
  Capture the pin (BigFloat center, span, mag, log2mag, upp_l2, final resolution/ss/gpu_iter,
  orbit_id+orbit_len), dispatch pass `[0, step)`, take the hold snapshot (`hold_copy` — the
  texture still holds the last COMPLETE frame, because holds don't iterate), engage
  `display_hold`, and DEFER the frozen latch.
- **Pin-continue**: view inputs shadowed with the pin's at the top of `build_params`; the frame
  computes exactly what it would if the user sat at the pinned view — same reference offsets, same
  SA/BLA gating, same step math — and dispatches the next range. The cursor lives where it always
  did (`chunk_cursor`/`chunk_idx`; the interacting reset is bypassed while pinned, which is the
  entire mechanical fix). `display_hold` stays up; `hold_uv` is recomputed EVERY frame from the
  LIVE view against the (un-overwritten) frozen bookkeeping, so the held image keeps tracking the
  dive. `chunk_pending` stays true so repaints keep coming through the settle-delay tail.
- **Adopt** (the frame after the last pass): cursor reached the pinned ask → write
  `frozen_center/frozen_l2/frozen_upp_l2/frozen_at` = the PIN's view, clear the pin, drop
  `display_hold`. The frame then proceeds live: small drift → `reuse_hold` → the reveal is the
  ordinary reproject of the now-complete texture; large drift → the next pin starts the same frame
  (and `hold_copy` retakes the snapshot from the just-completed texture, because adoption cleared
  `hold_active`).
- **Abandon**: clear the pin WITHOUT adopting; the state-texture progress is discarded (cursor
  resets via the ordinary sig mismatch). Triggers, checked at the top of the frame:
  settle edge (`!interacting`); orbit changed (`orbit_id`/`orbit_len` differ from capture — the
  §8 hazard, now a pin guard); live view drifted past `PIN_ABANDON_OCTAVES` (2.0) or panned past
  `PIN_ABANDON_SPANS` (1.5 view-spans); pin older than `PIN_MAX_FRAMES` (240 — a backstop, not a
  cadence: the budget controller measures every pin pass, so a bootstrap-collapsed step of 256
  grows ×1.5 per pass and completes in ~20-25 frames); `chunk_over` went false (the frame then
  renders complete un-chunked and re-latches coherently); or the frame unexpectedly froze
  (`reproject.is_some()` mid-pin — pan grab, reference starvation).
  Every abandon path either re-pins immediately at the live view (snapshot retained —
  `hold_active` stays true, so `hold_copy` is NOT retaken over the dirty texture) or hands off to
  a path that renders complete content. The residue is `chunk_dirty`: while the texture diverges
  from the frozen bookkeeping and the view is interacting, any freeze frame keeps `display_hold`
  up instead of reprojecting the dirty texture.

### Why pin-start commits LATE in the frame

`reuse_hold` is decided early (render.rs:2925), the chunk step late (render.rs:3493), and the
freeze verdict (`too_stale` → `reproject = Some`) later still (render.rs:3998-4056). A pin must
not start (or advance) on a frame the freeze then converts to a reprojection: the cursor would
advance with no pass dispatched and the resume would be garbage. So a START is decided at a commit
point AFTER the freeze verdict (immediately before the `hold_uv` capture), which also does the
cursor bookkeeping the interacting chunk block deliberately skipped. A CONTINUE frame, by
contrast, advances its cursor in the chunk block directly: the pre-step already abandoned on
every input that could freeze it (orbit change, caller reproject, drift, pan, panel, settle), so
the freeze is unreachable by analysis — and a belt at the commit point discards the progression
if it ever fires anyway.

**As built, the display decision did NOT move** (the plan above said it would): direct-mode and
pan-reprojection frames skip the perturbation block entirely, so a decision living at the commit
point would leave their flags undecided. Instead the original site keeps the settled gate
unchanged and gains a middle arm — pin active OR dirty residue, while interacting → serve the
snapshot, preserving it across re-pins (`hold_active` staying up is what stops `hold_copy` from
retaking the snapshot over a mid-compose texture) — and the commit point flips the flags only for
the START it alone can see.

### What is NOT changed, deliberately

- Non-pin behavior is bit-identical: settled progressions, mode-0/direct chunked motion without a
  frozen frame (cold start), tours, offscreen renders, the settled prefer-detail gate (its
  capture-once `hold_uv` semantics stay — the pin recomputes per frame only because it defers the
  latch that would otherwise stale the transform).
- The budget/measurement plumbing: pin passes are real dispatches and price themselves exactly as
  settled chunk passes do (`fe_steps_last` = range cost). Probes stay `!interacting`-gated.
- `capped_frac` stays cleared on interacting frames (the load-bearing clear).
- The step floor (1 unmeasured / 256 measured) and the explicit-count ceilings are untouched.

### Known residual, accepted and recorded

A SETTLED chunk progression still latches `frozen_*` per pass (each pass is `reproject.is_none()`),
so a user who grabs the view mid-settle can briefly hold a partial texture — the same narrow gap
mode-0 chunking has had since beta.64-69, self-healing within one refresh cadence now that the pin
lands complete refreshes during motion. Fixing it means gating the settled latch on progression
completeness AND snapshotting settled non-prefer-detail progressions; out of scope here.

Also accepted: `frozen_l2` is the autopilot stepped-dive readiness signal (autopilot.rs:96-99), so
adopt-on-completion makes stepped dives wait for a COMPLETE frame at depth before stepping — a
pacing change that matches the signal's stated meaning.

### The gate, which lands FIRST and must be RED on the held flip

Two counters in `Perf`, written where the latch runs: `adopt_partial` (the latch ran while this
frame's chunk pass left `end < gpu_iter` — the latched texture is under-iterated) and
`adopt_complete`. A `FRACTADYNE_TRACE=tile` `adopt …` line accompanies them. These are the
observables §10 said do not exist.

`--motiontest`: an in-loop harness in the `autodive_frame`/`uitest_frame` house shape (flag added
to `is_task_invocation` AND `launched_for_a_task`). Phases: jump to corpus loc 07 at 1.3e31×
with an explicit 4,000,000 ask (auto-iter off); wait for the reference; drive a continuous
zoom-in via `pointer.zoom_vel` (~6 s — the wheel-dive shape); `zoom_home` (the field-crash
shape) and ride the glide down; settle. Per-frame it samples the counters plus
`interacting`/`chunk_over` frame counts (anti-vacuity: the run FAILS if no interacting chunked
frames occurred). Verdict:

- **A1 (the regression)**: `adopt_partial == 0` for the whole run. On the held flip this is
  violated within the first refresh cadence — the RED that proves the gate can see the defect.
- **A2 (anti-option-B)**: `adopt_complete ≥ 1` DURING the interacting window — detail must keep
  streaming; a hold-forever "fix" fails here.
- **A3 (post-fix invariant)**: `dirty_shown == 0` — no frame displayed the live texture while it
  diverged from the frozen bookkeeping during interaction.
- Exit 0 pass / 2 assert-fail (torture `classify` contract); prints one verdict block.

Flap doctrine applies: the assertions are invariants and floors, never raw per-frame numbers.
`--autodive 32 --autodive-home 3` remains the device-loss rung; motiontest is the presentation
rung — it asserts what the autodive cannot, and it runs in ~2 minutes instead of ~20.

### Unit tests

The abandon/adopt/continue verdict is a pure function (`pin_verdict`) over a copyable input struct
(drifts precomputed as f64), `controller_props`/`relaunch_policy` house style: adopt only at
cursor==ask; every abandon reason; start eligibility (incl. the single-pass bypass `step >=
gpu_iter`, which keeps shallow refreshes exactly as they are today); the settle edge.

### Landed (2026-08-19), and what the gate measured

Both halves are on the branch: the gate first (RED against the held flip, as it had to be), then
the pin (GREEN). Same run shape, same machine, ~23 s each:

|                         | held flip (RED) | with the pin (GREEN) |
|-------------------------|-----------------|----------------------|
| interacting chunk frames| 585             | 536                  |
| adopt partial (A1)      | **131**         | **0**                |
| adopt complete (A2)     | **0**           | **3**                |
| dirty shown (A3)        | 0               | 0                    |

Three complete refreshes landed DURING motion — detail streams and is never partial. The cadence
is lower than the pre-flip ~7/s because a complete deep refresh now costs its honest number of
budget-bounded passes; the pre-flip cadence was purchased with the 1 s dispatches that lost the
device. During sustained fast motion a pin whose view runs more than `PIN_ABANDON_OCTAVES` ahead
abandons and re-pins, so the held frame can magnify up to ~2 octaves before fresh detail lands —
the §10 "streamed detail lags the dive slightly" cost, in numbers. If the field finds that too
coarse, `PIN_ABANDON_OCTAVES`/`PIN_MAX_FRAMES` are the calibrated knobs, and the motiontest's A2
floor is the regression fence behind any retune.

One deliberate scope note: `pin_verdict` orders Adopt before every abandon reason, so a
progression that completes at the settle edge (or past the drift threshold) still adopts — the
work is done and the texture is whole; discarding it would buy nothing.

## 12. The settled walk: price-serialized, regionally licensed, and owner of the deep compose (2026-08-20)

Three field device losses and seven repro deaths at one recipe — a minibrot interior at an
explicit 4,000,000, left to settle — took four design iterations in one night to close. Each
iteration exposed the next layer; all four are in the shipped design because each guards a
distinct measured kill:

1. **Price-serialized walking.** The settled walk holds AT MOST ONE unpriced pass in flight. A
   dispatched pass accumulates the following frame intervals until a frame returns quicker than
   `CHUNK_DRAIN_DT_MS` — with one pass in flight, a quick present is PROOF the queue drained —
   and that sum is the pass's wall price. Wall time is the one signal saturation cannot silence:
   the GPU timestamps arm only when nothing is in flight, which on a saturated queue is never —
   the fatal sessions contain not a single lethal reading while ~1 s frames flowed, and the
   budget GREW mid-kill on cheap-slice readings. While a pass is in flight the walk emits its
   last range verbatim (unchanged triple → the GPU dedupes, zero work — chunking's version of
   the zero-area tile hold). ⚠The pricing backstop (60× target) is a wedge escape only; an
   earlier 20× backstop released the next dispatch ONTO a still-running 10 s pass.
2. **Wall-price sizing** (`chunk_step_factor`, pure + tests): the next pass is the budget scaled
   by target/last-price, clamped [1/16, 1]. The budget's own ×1.5-per-reading climb walked
   straight into the lethal band because its readings were the cheap slices.
3. **Regional licenses, floor-opened** (`chunk_band_license`/`chunk_band_update`, pure + tests):
   the ask's cost is REGIONAL — the wrap/rebase-storm band around cur ≈ orbit_len runs ~10-70×
   the cold rate — so the license ledger is per cursor band (`CHUNK_BANDS`), survives same-sig
   restarts (re-crossing the storm with amnesia re-rolled the dice dozens of times a session),
   and clears on a sig change. ⚠**Every unvisited band opens at the FLOOR, never at a
   neighbour's license**: an inherited 36k license met a mid-band storm and ran TEN SECONDS in
   one submission. The floor (~256 iterations) is worst-case-prior sizing — ~70 ms even at the
   worst rate ever measured here — so first contact with any region is a survivable single;
   everything larger is earned from that region's own prices (×2 below half-target, ×1.25 at
   target, ×0.5 above it, ×0.25 past double — no hold gap where an over-target size
   re-dispatches itself).
4. **The iteration axis owns the deep compose.** `chunk_over` now triggers past ONE dispatch
   budget (`tdr_steps`), not past the multi-tile allowance. The allowance comparison handed the
   settled compose to TILES whenever a converged budget covered the need — and a tile prices
   itself nominally while its real cost is its FULL iteration chain: at the storm ask every
   122×122 tile ran all 4,000,000 iterations, ~1 s each, back to back. The fatal manifests all
   say `tile=true`. The eligibility comment had stated the principle since beta.64: this cost
   axis is per-pixel iteration DEPTH, and no spatial split bounds it. Tiles remain for what
   chunking cannot serve (aux coloring, formulas > 3, missing device capability).

Verdict at the lethal session: 300 s, zero device losses, zero tiles, and the walk COMPLETED the
full 4,000,000 through the storm — the first time this view has ever finished settling on this
hardware. Both compose paths produce bit-identical pixels (each equals the single-dispatch frame,
selftest-pinned), so handing the deep compose to the walk changes timing only.

## 13. The pricing that fed the walk was itself poisoned: the key-changed stamp vs the range cost (2026-08-20)

The RX 6800 XT autodive death (`crash-1787261212-0`) was not a failure of the walk — it was a
failure of the ledger the walk is priced from. Two independent rules collided:

- **R12** (the readback-starvation fix): a frame whose iterate KEY changed must stamp
  `fe_steps_last` with its nominal cost, because a real timing with no step count to price
  against is a measurement thrown away.
- **The chunk pairing** (§12): a chunked frame's dispatch is a RANGE, so the chunk block stamps
  the range's cost, not the frame's.

Both are right alone. Together, order decided: the R12 stamp ran AFTER the chunk block and
overwrote the range cost with the full-frame count whenever an install re-keyed a chunked frame.
On the dev 3080 this almost never fires — dives start shallow and installs are sparse. The
Radeon run booted DEEP from a saved session: installs re-keyed a chunked frame every ~300 ms,
each ~210 ms bounded pass was recorded as `1.838e11` steps (an implied ~875 Gsteps/s), and the
budget — already converged honestly at `6.834e9` — walked ×1.5 per fantasy reading to its
`6.0e10` ceiling in three seconds. The Home sweep's shallow side then dispatched budget-sized
chunks that were real 1–2 s submissions (`bla_skip=0`, nominal ≈ real), and the device loss
surfaced in the autopilot probe's synchronous readback at the re-dive moment. The probe's
manifest stamp — panel dims inherited from `current_export_request_for` before the 56×56
override — pointed the first hour of triage at the export path.

The fix is one guard: `if key_changed && chunk_range.is_none()`. A chunked frame's pairing is
already correct and must win; an un-chunked re-key still stamps exactly as R12 requires. The
completed-walk tail re-key goes unstamped on purpose — one stale pairing costs one mis-priced
measurement, which the ratio search corrects (R12's own tolerance, now actually honoured).

**The lesson in one line: a cost ledger with two writers needs an owner per frame, not an
order of operations.** And its corollary, already proven twice this cycle in the other
direction (nominal-vs-real): every safety margin downstream of a mispriced ledger is sized
from fantasy — the walk, the bands, the licenses were all doing their jobs correctly against
numbers that were wrong at the source.
