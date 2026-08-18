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

**Recommendation: Option A.** The bandwidth is a known, bounded cost on a path that is already
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
