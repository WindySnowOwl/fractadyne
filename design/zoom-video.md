# Zoom videos from exponential maps — design

Status: **design (2026-10-19); decisions taken 2026-10-27 (§9); E0 in progress.** Line
references are to `main` at `cc43741f`.

This covers two things a deep-zoom video maker would use, in the order they should be built:

1. **Exponential-map export** that zoomasm can assemble: the format the Kalles Fraktaler →
   zoomasm workflow already uses, so Fractadyne plugs into a pipeline people have.
2. **Assembly in Fractadyne itself** ("zoomasm functionality in the app"): render the strips and
   turn them into a video in one step, coloured by Fractadyne's own colouring, with no strip
   files left behind unless asked for.

The short answer to "can we implement zoomasm functionality in our app" is **yes**. The maths is
published and small, Fractadyne already renders, colours on the GPU and drives ffmpeg, and the
one real change to the renderer is a second way of placing pixels (§4). The licence decides how it
is built, not whether (§2).

## 1. What an exponential map buys

A zoom video is a straight dive into one point. Every frame of it is a disk of the plane around
that point, at a scale that halves every octave. Rendering each frame separately computes most of
the plane many times over: at 60 fps and one octave a second, each annulus of the dive appears in
about 60 frames, each time at a different scale.

An **exponential map** (Munafo's name; https://mrob.com/pub/muency/exponentialmap.html) renders
each annulus once. Its columns are the angle around the target and its rows the logarithm of the
distance from it: point `(x, y)` of the strip is `c = c₀ + exp(s·(y₀ − y) + i·s·x)` with
`s = 2π/W` (W = strip width), which is conformal, so the strip's pixels are square in the plane
and circles about the target become rows. One strip covers one octave when its height is
`W·ln2/2π ≈ 0.110·W` rows. A video frame at any zoom is then a resampling of the strips that cover
its radii (§6).

Cost (derived from the geometry, not from any program's documentation):

- A strip must be as wide as the circumference through the frame's corners, `W = π·diagonal`, so
  the frame's corner gets one strip texel per pixel. For 16:9 that is `W·H ≈ 2.55` frames of
  pixels per octave: **1080p: 6,920 × 764 = 5.3 Mpx; 4K: 13,841 × 1,527 = 21.1 Mpx.**
- Rendering frames costs `fps / (octaves per second)` frames per octave. At 60 fps and one octave a
  second that is **23.5× the pixels** of the strips; at 30 fps and half an octave a second, the same.
  The ratio grows with frame rate and with a slower zoom; the strips' cost does not depend on
  either.
- Example: a dive to 1e1000× is 3,322 octaves. Strips: 3,334 of them (the last ~12 cover the final
  frame's centre, §3) ≈ 17.6 Gpx at 1080p. Frames: 199,316 at 60 fps and an octave a second,
  413 Gpx.
- The target is fixed for the whole dive, so **one reference orbit serves every strip**; the tour
  pipeline already builds one reference and reuses it (`render.rs:642`, the SA trace
  `core/src/reference.rs:1394-1575`).

The price: the camera can only dive straight in. It can **rotate** for free (an angle offset at
assembly) and change speed freely, but it cannot pan. Fractadyne's own "Script to current view"
tours pan while shallow and then dive; §8 covers mixing the two.

## 2. Licence: clean-room only

zoomasm is **AGPL-3.0-or-later** (https://mathr.co.uk/zoomasm/), as are Kalles Fraktaler and
Fraktaler-3 (https://mathr.co.uk/kf/kf.html, https://fraktaler.mathr.co.uk/). Fractadyne is MIT /
Apache-2.0. So, as with every other AGPL program this project has measured against:

- Implement from **published mathematics** (Munafo's page above; the emndl write-up
  https://en.wikibooks.org/wiki/Fractals/emndl, `c = c₀ + exp(x + iy)`) and **documented file
  formats** (KF's manual lists its EXR channels: https://mathr.co.uk/kf/manual.html).
- **Never read their source**, including the vendored Fraktaler-3 tree in `diag/fraktaler/source/`
  (gitignored). Where the documentation leaves a convention open (row order, angle origin, the NF
  bias), settle it by **running the programs and reading their output files**, which is data
  (§5.3).
- Running zoomasm, KF or F3 to check compatibility is fine; linking or translating their code is not.

## 3. Geometry and conventions

Our own convention, used everywhere inside Fractadyne (the KF-compatible writer converts, §5):

- **Strip `k`** covers radii `r ∈ [R₀·2^-(k+1)·(1−ε), R₀·2^-k]`: `R₀` = half the diagonal of the
  first frame, so strip 0 covers its corners; `ε` a small overlap (one row) so neighbouring strips
  share a row for blending.
- **Texel `(i, j)`** of strip `k` is at angle `θ = 2π(i + ½)/W` (anticlockwise from the positive
  real axis) and radius `r = R₀·2^-k·exp(−2π(j + ½)/W)` (row 0 outermost).
- **The centre of the last frames.** A frame at zoom `Z` needs radii from its corner down to about
  half a pixel, `log2(diagonal)` octaves below its outer radius: 11.1 at 1080p, 12.1 at 4K. The
  dive's strips therefore run ~12 octaves past the final frame's zoom (or the centre of the last
  ~12 octaves of frames is filled from one flat render at the end; the extra strips are simpler and
  cost ~12 strips).
- **Sampling density.** Along any frame, strip texels are never larger than the frame's pixels and
  get smaller towards the centre (a texel at radius `r` is `r·2π/W`, a frame pixel is fixed): the
  strips oversample the inner frame, so assembly is a downsampling problem (§6), and a strip
  oversampling factor `W = q·π·diagonal` buys antialiasing at the corners.

## 4. Rendering a strip

The renderer places pixels on an affine grid and nowhere else. Every iterate entry point goes
through `iterate_at` (`gpu/src/mandelbrot.wgsl:1181-1190`):
`off = step × (gx − w/2, h/2 − gy)` in df32, with one `delta_exp` for the frame
(`core/src/viewport.rs:279-287`); the chunked mode-2 path builds the same offset in `chunk_fe_at`.
The only non-rectangular precedent is the glitch-correction gather pass, which feeds a list of
integer texels back through `iterate_at` (`mandelbrot.wgsl:2094-2108`).

What a strip needs:

| Piece | Today | For a strip |
|---|---|---|
| Pixel offset | affine `step × (gx − w/2, h/2 − gy)` | `exp(−s·(gy + ½))·(cos, sin)(s·(gx + ½))`, `s = 2π/W`, scaled by the strip's outer radius. A strip spans ×2 in radius, so its `delta_exp` is the outer radius's exponent and the mantissa offset stays in [½, 1] — df32 holds it. Sample positions carry f32 angle error (~1e-7 of the radius ≈ 1e-4 texel), which moves a sample, never a result. A uniform `map_mode` selects it once per pixel, outside the loop. |
| DE | the escape distance over one per-frame step (`de_log2`, `mandelbrot.wgsl:486-496`) | over the texel's own size `r·2π/W` (one log2 per pixel) |
| Slope normals (relief) | screen frame | the c-plane frame (the same thing on an unrotated affine frame), so assembly can rotate them |
| SA / BLA radius, reference pick span | from the view rectangle (`render.rs:1409-1419`, `3434-3443`; `core/src/reference.rs:2967`) | the strip's outer radius |
| Reference | picked per view | the dive's centre, once; deep strips extend it (`REUSE_MAX_DRIFT` path) |
| Iteration cap | per view | per strip, **non-decreasing with depth**; a cap that changes between neighbouring strips makes a capped/escaped seam along their shared row, so steps in the cap happen where the colouring hides them (or the dive uses one cap) |
| Tiles, step-bounded passes, packing, SSAA | rectangular tiles of the output | unchanged: a strip is a rectangle of texels; SSAA jitters `(gx, gy)` before the mapping |
| Glitch correction | gather list of texels | unchanged (texel coordinates) |

Nothing in the iteration loop changes, so a strip texel computes exactly what a flat render
computes at the same `c` — the property the validation in §5.3 leans on.

**Per-pixel precision note.** The iterate texture stores the smooth count as one f32
(`gpu/src/lib.rs:38`). Past 2^24 ≈ 16.8M iterations it cannot hold the fraction at all (an f32
spacing there is 1–2 iterations), and long dives reach such caps. The strip formats below store the
integer count and the fraction separately, as KF does; the live texture's own limit is a separate
item for TODO.

## 5. Output A: zoomasm-compatible EXR

### 5.1 Channels (KF's documented layout)

From KF's manual (https://mathr.co.uk/kf/manual.html):

| Channel | Type | Meaning | Ours |
|---|---|---|---|
| `N` | uint32 | escape count + `IterationsBias`; `0xFFFFFFFF` = never escaped, 0 = not computed / glitch | integer part of the count, interior, glitch |
| `N0`, `N1` | uint32 | low / high words when counts pass `0xFFFFFFFE` | same, for caps past 4.29e9 |
| `NF` | float | fraction in [0, 1) | smooth fraction |
| `T` | float | phase of the first escaped `z`, in turns | the final `arg z` the decomposition method keeps (aux.a) |
| `DEX`, `DEY` | float | directional DE, magnitude ≈ 1 at a neighbouring boundary pixel, pointing away from it | DE vector over the texel size, rotated into the strip frame (a c-plane vector `v` at offset `δc` is `v/δc·W/2π` in strip texels: the exponential map is `log`) |
| `R`, `G`, `B` | half | linear colour (optional) | our colouring, baked |

Header attributes `Iterations` and `IterationsBias`; KF also stores its location and settings.
The `exr` crate (already used by `fractadyne-export`, `lib.rs:884-907`) writes uint and half
channels.

### 5.2 What zoomasm then does

zoomasm reads a numbered sequence of such strips, one octave per file, and assembles frames at any
resolution and frame rate along a timeline of (time, depth) waypoints, with rotation, motion blur,
samples per pixel, a user GLSL colouring shader with access to the raw channels, audio, and output
through ffmpeg (https://mathr.co.uk/zoomasm/zoomasm-3.4.html). Version 3.4 (2025-06-11); Linux is
its supported platform, Windows builds are cross-compiled and untested by its author, macOS untested.

### 5.3 Conventions to settle by experiment (not by reading code)

**Settled 2026-10-27 by running Kalles Fraktaler 2.15.5.2** (its batch mode, `kf -s S.kfs -l L.kfr
-x out.exr`, with Derivatives, ExponentialMap and every EXR channel on) and reading its files
against an independent double-precision orbit; §8.2 has the result and `app/src/expmap_kf.rs` the
rules. Fraktaler-3 3.1's batch mode wrote RGB only on this machine, so it could not settle the raw
channels; its strips gave the angle's origin and direction (the same as ours) and showed its rows
run the other way up from KF's (zoomasm's Flip box).

The documentation leaves open: which edge of a strip is outermost, where the angle starts and which
way it runs, the file numbering order (zoomasm has **Reverse** and **Flip** toggles for exactly
these), and an apparent disagreement about the fraction (the format text gives `N + NF − bias`,
the shader API's `getN` reads as `N + (1 − NF)`). To settle them:

1. **Fraktaler-3 writes exponential maps** (`transform.exponential_map`, recommended
   12288×1360 for zoomasm; https://fraktaler.mathr.co.uk/) and is installed with the bench kit.
   Render the same location's strips in F3 and in Fractadyne and compare channel by channel, as the
   F3 corpus already compares raw counts (`--crosscheck-f3`). Matching N/NF texel for texel fixes
   layout, orientation and bias at once; F3 is already the reference this project validates
   against.
2. **Run zoomasm 3.4** headless (`--record` with a session file) on our strips with its
   `rgb-passthrough` and `binary-decomposition` presets, and compare its frames with frames
   rendered flat at the same zooms. Pass = the same structure everywhere (LOOK at the frames; then
   count differing pixels in the binary-decomposition frames, which have no palette to hide behind).
   **Done 2026-10-28** with probe colourings of our own instead of the presets (§8.2): it found
   that KF's default orientation shows our dives upside down.

Disk cost is the workflow's, not ours: KF's channels are ~26 bytes a texel, ~140 MB a 1080p strip
before EXR compression, so a 3,000-octave dive is on the order of 100–400 GB. Users of this
workflow already live with that; §6 avoids it.

## 6. Output B: assembly in Fractadyne

### 6.1 Why build it when zoomasm exists

- **No disk mountain.** A frame at zoom `Z` needs only the ~12–16 strips from `octave(Z)` inward.
  Rendering strips in dive order and assembling each frame as soon as its strips exist keeps a
  sliding window of strips (~1 GB at 1080p) instead of the whole dive.
- **Fractadyne's colouring**, all of it (palettes, the six methods, relief lighting, normalize /
  log mapping), applied on the GPU to the strips: no GLSL to write.
- **Recolour without re-rendering**, when the strips are kept: palette, method, cycle speed — the
  iteration data does not change.
- **One step from the app**, on Windows (where zoomasm is untested) and Linux, through the ffmpeg
  the tour export already uses (`scripting.rs:494-559`, `exec_resolve.rs`).
- **Rotation** comes free: the tour format has none today.

### 6.2 Reprojection

For output pixel `p` of a frame at zoom `Z`, rotation `φ`, with `δ = (p − centre)·upp(Z)`:

- `r = |δ|`, `θ = arg δ + φ`; strip `k = floor(log2(R₀/r))`, row `j = (ln(R₀·2^-k) − ln r)·W/2π − ½`,
  column `i = θ·W/2π − ½` (wrapping).
- Strip texels are at least as fine as frame pixels everywhere (§3), so each output pixel takes
  several samples over its own footprint (a sample count, like zoomasm's "samples per pixel") and
  averages the **colours**. It never interpolates iteration counts: colour each texel, then
  average. (The XaoS-reuse work found bilinear interpolation of the iteration texture worse than
  nearest, `design/xaos-reuse.md`; the same reason applies.)
- Two strips meet on a shared row (the overlap `ε`); a pixel whose footprint crosses the seam
  takes samples from both.
- Motion blur, if wanted: average sub-frame times (a shutter fraction) — a later option.

### 6.3 Colouring at assembly

The colour pass works on a texture of per-pixel quantities (`fs_color`,
`mandelbrot.wgsl:3429-3570`; `coord_of` 3329-3359). A strip in our own raw format is such a
texture, so the colour pass runs over each strip once (texel space) and the reprojection samples the
coloured strip. Two things need care:

- **Neighbourhood operations** (relief lighting's 3×3 normal average, the AA footprint estimate,
  the DE glow) assume neighbouring texels are neighbouring screen pixels. In a strip they are
  neighbouring texels of a conformal map, which is locally a rotated, scaled screen; normals stored
  in the c-plane frame (§4) and the texel's own size make these exact up to that local scaling.
- **Normalize / log palette mapping** reads the frame's escape range. Over a dive, the range should
  follow the frames as the live view's does (the 2026-10-18 palette-follows-zoom fix), computed per
  frame from the strips it samples, with the same smoothing — or fixed for the whole video.

### 6.4 Timeline, captions, audio, encoding

- **Timeline:** the tour format's keyframes already carry time, zoom (text, log-linear between
  keyframes) and easing (`scripting.rs:977-1005`, 1840). An exp-map tour is a tour whose keyframes
  share one centre; add an optional `rotate` field (degrees, eased). Captions, callouts and fades
  are drawn as the tour renderer draws them now (`scripting.rs:3978-4045`).
- **Encoding:** pipe raw frames to ffmpeg's stdin instead of writing PNGs and assembling after
  (`scripting.rs:518-527` does the latter). Encoding then runs beside assembly; at 4K the encoder,
  not the GPU, is the bound (assembly is a few texture samples a pixel).
- **Audio:** mux a user's audio file with ffmpeg (`-i audio … -shortest`). zoomasm's audio-aware
  timeline editing is out of scope.

### 6.5 Where it appears

- CLI: `--render-expmap DIR` (strips, our format and/or `--kf-exr`), `--assemble DIR --out F.mp4`,
  and the one-step `--render-tour T.toml --expmap` for a tour that qualifies.
- App: File → "Render zoom video…" from the current view: centre = the view's centre, start and end
  depth (default 1× → here), duration or octaves per second, frame rate, size, rotation, colouring =
  the current one. Progress shows strips and frames. "Keep strips" enables recolour and resume.
- Farm and several GPUs: a strip is a self-contained job sharing one reference blob — the farm's
  unit (a run of frames, `farm/src/sched.rs`) becomes a run of strips (`TODO.md:9120-9122` planned
  exactly this). Resume = strips already on disk.

## 7. Validation

- **Geometry:** strip texel (strip `k`, column `i`, row `j`) → `δc` against a bignum computation of
  `R₀·2^-k·e^{−s(j + ½)}·(cos θ, sin θ)`, `θ = s(i + ½)`, at random texels (unit test, core crate).
- **A strip computes what a flat render computes:** gather-render the strip's sample points
  through the existing gather pass and compare with the strip — byte-identical, since both call
  `iterate_at` with the same `δc`.
- **Against Fraktaler-3:** §5.3 (1). Per-texel N/NF where comparable, as the corpus does.
- **Against zoomasm:** §5.3 (2).
- **Assembly against direct rendering:** frames assembled from strips against frames rendered flat
  at the same zoom, coloured the same way. Expect differences only from resampling (count them;
  LOOK at the worst frame). Include frames at strip seams, the first frame (strip 0's corners), and
  the last (the centre's extra octaves).
- **The selftest and goldens stay byte-identical** with the strip mode switched off: it only adds
  a uniform branch outside the loop (measure that it costs nothing, as the RN_BLA override was
  measured, `mandelbrot.wgsl:882-885`).

## 8. Phases

| Phase | Builds | Gate |
|---|---|---|
| **E0** | Strip geometry, `map_mode` in `iterate_at` and `chunk_fe_at`, per-texel DE size, strip radius for SA/BLA/pick; `--render-expmap DIR` writing our raw strips (integer count + fraction, DE, normals, T) with a manifest | geometry unit test; strip = gather render; selftest/goldens unchanged; cost of the switched-off branch measured |
| **E1** | KF-compatible EXR writer (N/N0/N1/NF/T/DEX/DEY, optional RGB, header) | F3 exp-map comparison; zoomasm 3.4 assembles our strips with no seams, right orientation |
| **E2** | `--assemble`: reprojection, multi-sample, colour pass over strips, rotation, timeline from a tour, ffmpeg pipe, audio mux, sliding strip window | assembled vs flat frames (§7); a 1e100 dive at 1080p60 end to end, timed against `--render-tour` of the same dive |
| **E3** | App dialog, preview/scrub of the assembled video in the view, recolour from kept strips, farm and `--gpus` distribution by strips | uitest screens; a farm run with two GPUs |
| **E4** (optional) | Tours that pan then dive: flat frames for the pan, strips for the dive, a crossfade where they meet | a "Script to current view" tour rendered both ways, compared |

E0 + E1 alone deliver the zoomasm workflow; E2 is where Fractadyne stops needing zoomasm.

### 8.1 E0 as built (2026-10-27, branch `feat/zoom-video`): every gate passed

- **Geometry:** `fractadyne_core::ExpMap` (`core/src/expmap.rs`), §3 exactly; overlap 1 row (rows =
  ⌈W·ln2/2π⌉ + 1). Texel offsets are exact at any depth (the octave count is an integer exponent).
- **Renderer: a module of its own.** The affine mapping is written out in more places than the note
  knew (`iterate_at`, `fs_iterate_chunk`, `chunk_fe_at`, and a copy in the custom module), and
  `fs_resolve` needs the texel's size for its distance estimate. `mandelbrot.wgsl` marks six
  STRIP_* slots (the three prologues, `de_log2`'s texel size, `fs_resolve`'s texel size and
  output); `strip.rs` fills them with calls into `strip.wgsl` for a strip's own pipelines, and
  every other pipeline compiles `mandelbrot.wgsl` as it is — main's text plus the uniform row and
  the slots' comments. ⛔Measured on the way: (1) the strip branch at `iterate_at`'s escapes, never
  taken, moved the single pass's smooth count by an ulp and broke the chunked-equals-single-pass
  gate (46 selftest checks); (2) one shared `pixel_offset` with a runtime branch cost every render
  1-2% (dev.51 against main: 37/43 bench-matrix segments slower on both of PLUTO's cards); (3) so
  did the same code with the branch compiled out by an `override` (dev.52: 38/43 and 32/43), while
  an A/A control of main against itself read 24/43 and 16/43. With strips in a module of their own
  (dev.54) the gate passes: 17/43 slower on the RTX 3070 (sign test p 0.22, run totals within
  0.7%) and 9/43 on the RX 6800 XT (about 1% FASTER), every algorithm signature identical; 5
  interleaved runs a build a card, PLUTO, 2026-10-27. The angle uses the strip module's own
  cos/sin (WGSL's built-ins are promised only to 2^-11, half a strip texel). In a strip the AUX
  target carries N (as `iter>>12`, `iter&4095`), NF = 1−nu and T, written by the chunked path's
  `fs_resolve` only; a strip always chunks from its first tile.
- **The raw strip:** `strip_NNNNNN.exr` with channels `N` (uint32; 0xFFFFFFFF capped, 0 glitched),
  `NF`, `T`, `DE` (log2 texels of the strip), `NX`, `NY`, PIZ, written atomically; `expmap.json`
  (the dive's identity, which a rerun must match; strips present are kept). NF is `1 − nu` with the
  renderer's bailout of 256, so it lies around −2 to −3, not in [0, 1): E1 converts.
- **Reference, SA, BLA:** each strip renders as an export of a VIEW sized to the strip's disk (its
  half diagonal is the strip's outer radius), so the pick, the series radius and the BLA radius
  follow without new code. ⭐ONE orbit for the whole dive: built for its deepest strip, at that
  strip's cap and precision, and handed to every strip as a `ReuseRef`; each strip works out only its
  own series (the walk is recorded once and replayed, `set_sa_trace_cache`) and BLA. A 130-octave
  dive at 320×180 (corpus 07's point): 22.6 s with a reference per strip, 17.7 s with one; the
  strips differ at 4.4% of texels, and the oracle judges strip 17 equally right both ways (37 and
  36 of 48 exact, 0 wrong — the rest are texels where the oracle itself does not settle, ~1/4 of
  this point's). The selftest's `expmap` group renders its perturbation strips on a reference built
  four strips deeper and requires that the reuse happened.
- **Caps:** `--expmap-cap rising` (default) = the app's depth rule at each strip's inner edge, never
  falling; `fixed` = the deepest strip's (or `--iter`'s N).
- **The gate that replaced "strip = gather render":** the selftest's `expmap` group compares a strip
  in each arithmetic mode with the arbitrary-precision oracle at the points the CPU geometry gives,
  with a quarter-turn control and a ±1e-3-texel stencil for points where the oracle itself does not
  settle — independent of the renderer, where a gather pass would share its `iterate_at`.
  Mutation-checked. Frames rebuilt from strips by nearest texel matched flat renders of the same
  frames (a scratch tool; LOOKED at).
- **Not yet:** Julia, custom formulas, the other escape-time families (refused with a message).

### 8.2 E1 as built (2026-10-27, branch `feat/zoom-video-e1`)

`--render-expmap DIR --expmap-format zoomasm` writes Kalles Fraktaler's strips. **KF's conventions,
measured** from KF 2.15.5.2's own EXRs (flat and exponential-map, powers 2, 3 and 8) against a
double-precision orbit — each rule below fits KF's every texel to f32 rounding:

| | KF 2.15.5.2 | Our strips (§3) |
|---|---|---|
| Rows | exactly one octave in `H` rows (KF's "zoom size 2"); `H` free | conformal + 1 overlap row |
| Texel `(x, y)` | `θ = 2πx/W`, `r = top·2^(−y/H)`: sampled at its CORNER; row 0 outermost; angle anticlockwise from +re (`ImagPointsUp: 0`, its default) | `θ = 2π(x+½)/W`, centre |
| Orientation | `ImagPointsUp: 1` writes the same strip mirrored in angle (texel `x` = default texel `(W − x) mod W`, so clockwise) with `DEY` negated, every other value equal (99.9% of texels identical after the mirror). **We write this one**: zoomasm then shows the dive the right way up | imaginary up |
| `top` | the half diagonal of a flat `W×H` image at KF's zoom (`2/zoom·hypot(W,H)/H`; zoom = our magnification) | `R₀·2^-k` |
| Escape | `\|zₙ\| > 10^(4/p)` (100, 21.5 … 3.16), `z₁ = c` | `\|z\| > 256` |
| `N` | `n + 1024` (`IterationsBias`); interior `0xFFFFFFFF`, every other channel 0 | — |
| `NF` | `1 − log_p(ln\|zₙ\| / ln ER)` | `1 − log_p(log2\|z\|)` |
| `T` | `fract(p·arg zₙ / 2π)` — p times the angle, not the angle | `atan2/2π + ½` |
| `DEX, DEY` | `\|z\|ln\|z\|/\|dz/dc\|` over a pixel, along `(Re u, −Im u)`, `u = z/(dz/dc)`: the plane's vector, imaginary DOWN (KF's default `ImagPointsUp: 0`), NOT rotated into the strip; a strip's pixel is `r·2/hypot(W,H)` | log2 DE over the texel, unit normal (plane, imaginary up) |
| Header | layer attributes `Iterations`, `IterationsBias` (i32) | — |

Because `ER^p = 10^4 > 256` at every power, the renderer's escape iterate is KF's or the one after;
`zₙ₋₁^p = zₙ − c` makes the step back EXACT (carrying the smooth count across instead was off by
2.4e-5 at low counts, the `+ c`). Octave strips are `ExpMap::with_octave_rows()` (`StripRows::Octave`:
rows = the whole number nearest `W·ln2/2π`, sampled at texel corners); the shader takes the sample
offset in the strip uniform's spare word (`bitcast<f32>(map_pad)`, so the shared module's text is
unchanged) and its row step as `map_s`.

**Gates:**
- **Against KF itself**, the same strips rendered by both programs (640x360 frames, `W×H` = 2307x255):
  at 0.3+0.1i, strips 0–5 agree on the interior to 99.9%+ and on the count, fraction, phase and DE
  vector to ~1e-6 on the median texel. Strip 6's (85% equal counts) and the seahorse dive's at
  2^28–2^33 (84–94%) disagreements were refereed by a 120-bit orbit: at 0.3+0.1i KF was right and our
  f32 direct mode off — and a flat `--render-iter` at the same pixel size is off the same way (65%
  of its escaped pixels within 1e-3, strip 6 64%), so a strip is as exact as a frame; at depth both
  programs are equally exact (within 1e-3 of the orbit: ours 72% / 81%, KF 70% / 85%).
- **Unit tests:** the conversion against KF's definitions on real f64 orbits at powers 2–8 (both
  escape relations exercised hundreds of times; mutation-checked: the step back, the escape radius,
  DEY's sign); the octave layout against a 256-bit computation; the EXR integer attributes round trip.
- **Selftest `expmap`:** a fourth case, a zoomasm-layout df32 strip against the oracle (34/48 exact,
  0 wrong); sampling it at texel centres instead fails it (0/48 exact, 31 wrong).
- **zoomasm 3.4 assembling the strips (2026-10-28).** A 30-strip seahorse dive (−0.7436…+0.1318…i
  to 2^20, 640x360, fixed cap 20,000) recorded to lossless PPM frames, one an octave, with probe
  colourings of our own written from the manual's GLSL API (binary decomposition of `T`, bands of
  `getN()`, the DE's length and its direction); KF rendered the same 30 strips for comparison.
  - **Geometry exact:** with Shutter 0, zoomasm's frame at depth `Z` is Fractadyne's flat render at
    magnification `2^Z` (interior IoU 0.97 / 0.99 / 0.94 at `Z` 0 / 5 / 19, fitted scale 1.000–1.005).
  - **Orientation:** strips in KF's default layout came out upside down (IoU flipped 0.97 / 0.99 /
    0.94, as is 0.54 / 0.74 / 0.41); zoomasm's Flip box reverses each strip's ROWS (broken rings),
    so it cannot help. Fixed by writing `ImagPointsUp: 1` (`expmap_kf::kf_column`, `DEY` sign):
    as is 0.97 / 0.99 / 0.94. KF's own `ImagPointsUp: 1` strips come out the same, so zoomasm
    goes by the data, not KF's header.
  - **Ours vs KF's, frame for frame:** interior within 0.05% every frame, the DE's length all but
    identical, no glitch texels in either; `T` and count bands differ on 0.1–1.7% of pixels over
    the first 5 octaves, up to 14% at `Z` 9, 4–8% deeper. Refereed: strip 9 (direct mode) 42% of
    texels differ by > 1e-3 and a 120-bit orbit sides with KF (ours 0% / KF 94% of 150 sampled),
    all within a texel of the set (median 0.009 texel); strip 13 (perturbation) a tie (24% / 27%,
    neither 50%). Like for like (the same area, texels weighted by area), strip 9 is exactly as
    exact as a flat `--render-iter` at 2^9 (61.6% / 61.1% within 1e-3 of an f64 orbit): the
    shallow gap to KF is the renderer's single-precision direct mode, flat frames included.
  - **zoomasm facts, measured:** a session loads only with its input and colour paths filled in
    (else "load settings error", even for one zoomasm saved itself); a waypoint's `t` is a FRACTION
    of the soundtrack (the last is 1.0); `--record` needs the GUI (with `--no-gui` it sits idle);
    Shutter 50% at one sample a pixel samples each pixel at its own moment (two recordings of the
    same strips differ) and a quarter of a frame's depth later. A 20 s 720p video (31 strips,
    default rising cap) took 17 s to render and 16 s for zoomasm to encode.
- **Not yet:** RGB channels (KF writes its colouring as half R, G, B; zoomasm's rgb-passthrough
  preset reads them). Shallow strips by perturbation or double-single, for KF's precision near the
  set (cost to measure).

### 8.3 E2 as built so far (2026-10-28, branch `feat/zoom-video-e2`)

`--assemble DIR --out F.mp4` (video) and `--assemble DIR --assemble-at L[,L…]` (PNG frames; with
`--render-iter`, each frame's raw main target as an EXR), from Fractadyne-format strips on disk.

- **Frame = reprojected main target + the app's colour pass.** `fs_color` knows nothing of the
  plane: it reads a main target and assumes `ss×ss` texels a pixel. So the assembler fills a
  FRAME-SHAPED main target, `(w·ss)×(h·ss)`, from the strips (`gpu/src/assemble.wgsl`, a module of
  its own, nothing in `mandelbrot.wgsl` changed), and colours it with `fs_color` from the unchanged
  iterate module, with the uniform `color_iter_buffer_ss` builds (`color_uniforms_for`, shared). §6.3's
  "colour each strip once, then reproject colours" is NOT what was built: colouring after
  reprojection gets every neighbourhood operation (relief's 3×3, the palette AA footprint) and the
  distance estimate in frame samples for free, exactly as a flat render has them.
- **Each sample takes its nearest texel** (`ExpMap::locate` is the CPU definition; the shader
  mirrors it with its own atan, WGSL's is only promised to 4096 ULP). The distance estimate is
  rescaled from the texel's radial size to the sample spacing; the normal is turned by −φ on a
  rotated frame (rotation is in the GPU path, not yet on the command line).
- **Depth is split** into whole octaves (i64, CPU) and a fraction, so a sample's own part stays an
  f32 good to ~1e-3 of a row at any depth. The window is a ring of array layers (strip `k` in layer
  `k mod layers`), `layers_for_frame` deep (11 at 640x360), with a 1e-4-octave margin each side
  so a sample on an octave boundary always finds its strip.
- **Strips are read ahead** on threads of their own (two beyond the window); frames go to ffmpeg
  through a writer thread (`app/src/video_pipe.rs`: raw RGBA on stdin, the PNG path's ordered
  dither, a `.partial` file renamed into place).
- **Gates** (selftest group `assemble`; mutation-checked):
  - synthetic strips whose every texel holds its own number: 155,520 samples over 12 frames (1x1
    and 2x2 samples, a rotated frame, past the dive's end) read the texel `locate` names, 0 wrong
    (9 within 1e-3 texel of an edge); DE rescale exact to 6e-7, normals to 2e-8. A half-texel
    column shift: 77,593 wrong; the row step left out of the DE rescale: DE error 5.8.
  - a real dive (160x90, cap 3000) at 2^0, 4.5, 10, 15.5, 20 against `render_iter` of the same frame,
    with a CONTROL (the same flat render moved (0.3, 0.2) px): the assembled frame agrees with the
    flat one at least as well as the control at every depth (within 0.1 iteration: 97.0/51.9/63.7/
    63.8/8.2% vs the control's 93.2/33.6/56.3/51.2/4.5%), DE bias ≤ 0.0065 (the left-out row step:
    +6.52), and the coloured frame is identical (max difference 0) to `color_iter_buffer_ss` on the
    same samples. Sample-for-sample differences are large wherever pixels are chaotic, which is
    why the check is relative to the control, not absolute.
  - The same comparison at 640x360 by hand (the seahorse dive, fixed cap 5000) gave the same
    ordering at all six depths.
- **Speed:** a 640x360 dive assembles at ~200 frames a second (601 frames in 3 s on the 3080).
- **Palette normalization over a dive** (`--normalize`, or the session's `normalize_live`), §6.3's
  "computed per frame from the strips it samples": each strip, as it is read, is summarized in 8×8
  blocks (least, greatest and mean escaped value, escaped count). An ANCHOR at every whole octave
  (and the dive's end) is the frame's window as the live view takes it: from the least value to the
  98% AREA quantile (`norm_top_quantile`; blocks weighted by escaped texels × radius²), because the
  slowest pixels sit just under the cap wherever interior is in view — min/max put the whole
  picture in the bottom 20% of the palette (measured: top 4998 vs 1689 at the same view). Frames
  interpolate the anchors GEOMETRICALLY (the window grows by factors: linear moved it 13% in an
  octave's first frame and 3% in its last; the first frame step of the 2^20 dive's mean colour
  went 11.1 → 4.4, below the un-normalized video's 4.6). The live view switches linear→log at a
  window ratio of 1000; a video DISSOLVES between the two across a factor of 2 either side
  (`log_weight`, through the colour pass's palette fade), so a dive across it never jumps. Each
  anchor is logged (`[fd-assemble] palette anchor at 2^L: window LO to TOP (W% log)`).
- **One step, no strips on disk** (`--render-zoom-video F`, §6.1's sliding window): the dive is
  planned as `--render-expmap` plans it (`plan_dive`, shared), its one reference built for the
  deepest strip, and each strip rendered (`render_strip`, the same path) just before the first frame
  that reads it — or the first palette anchor that does — straight into the GPU window
  (`StripSource::Render`; `StripSource::Folder` is `--assemble`'s read-ahead). `--keep-strips DIR`
  also writes them (Fractadyne format, with a manifest) for a later `--assemble DIR`. The 2^20
  seahorse dive at 640x360: 30 strips and 601 frames in 7.5 s; assembling the kept strips again,
  2.8 s. ⛔**Compare videos LOSSLESSLY**: the two videos at crf 18 "differed" on 24% of pixels by
  up to 94 levels — x264's decisions cascade from any input difference until the next keyframe
  (whole stretches matched exactly, others not) — while the same two at `--assemble-crf 0` match
  on 563 of 601 frames exactly and differ on 0.00009% of pixels by at most 3 levels (the smooth
  count: the in-memory strip carries the renderer's `sm.z`, a file `N + NF` rounded once).
- **Tours and rotation** (§6.4): `--assemble-tour T` takes each frame's depth from the tour's own
  playback (`Playback::sample`: keyframe zooms, times, holds, easing; the tour's fps unless
  `--assemble-fps`), and REFUSES a tour that leaves the dive's centre by more than a thousandth of a
  pixel of the frame it is at, or goes outside the strips' depths — a strip video cannot pan (start
  the tour where its last pan ends). `--assemble-rotate R`: R turns an octave (the GPU path had
  rotation from the start; normals turn with the frame, so relief light stays fixed to the screen).
  Captions, callouts and the tour's palettes are not applied yet.
- **§8's gate at reduced size (2026-10-28, the 3080):** the 1e100 dive into the target of
  `tours/dive-to-view-star-e98.toml` (4.70e99×, 331.1 octaves) at 320x180, 30 fps, 1 octave a
  second, a fixed cap of 20,000, normalized. `--render-zoom-video`: **62.5 s** in all — 340 strips
  in 33.8 s (12 direct, 79 df32 perturbation, 249 floatexp; median 107 ms a floatexp strip) and
  9,934 frames assembled (2.7 ms of GPU a frame) and encoded. `--render-tour` of the same dive (a
  two-keyframe tour on the same centre, linear zoom, the same cap and size, ss 1): **919 s** for its
  9,935 PNG frames, without encoding — **14.7× slower**. The frames LOOKED right at every depth (the
  e98 stars, the palette anchored once an octave from 190–239 at 2^40 to 2229–2280 at 2^320).
- **§8's gate at full size (2026-10-28, the 3080, the user's go-ahead):** the same dive at 1920x1080,
  60 fps, cap 20,000, normalized: `--render-zoom-video` **738 s** (12.3 min) for 19,867 frames
  (331.1 s of video): 343 strips of 6921x765 in 118.9 s (median 326 ms a floatexp strip, 422 ms a
  df32 one, the largest 599 ms — no device trouble), a 12-strip window (1.0 GB), 22.1 ms of GPU a
  frame. `--render-tour` of one-second slices of the same dive (61 frames each, 1080p60, same cap)
  ran at 5.34 / 5.56 / 5.29 / 5.20 fps at 2^20 / 2^100 / 2^200 / 2^320 — flat with depth — so the
  whole tour is ≈ 19,867 / 5.3 ≈ **3,750 s (62 min)** of PNG frames before any encoding: the strip
  path is **~5× faster** here, against 14.7× at 320x180. ⭐At 1080p the STRIPS are cheap (16% of
  the run); the frames are the cost: 22 ms of GPU each (a 33 MB RGBA32F readback among it) plus the
  CPU dither and x264 — the next speed-up is there (quantize on the GPU, pipeline the readback,
  a hardware encoder), not in the strips.
- **Not yet:** captions/callouts/palettes from a tour; the per-frame cost above.

### 8.4 E3 so far (2026-10-28): the app dialog

Tools ▸ "Render zoom video…" (`app/src/ui/zoom_video.rs`; in Tools beside "Tour from current
view…" rather than File, where the app's other video tools are). Modelled on "Render tour…": the
render is a CHILD `--render-zoom-video` (a lost device ends the render, not the session; the
tested path), its stdout progress (`frame K/N (…)`, now printed every 30 frames) drives a bar,
its stderr's first error is quoted on failure, and the live view holds while it runs. The view
goes on the command line exactly (the centre as the viewport's text, `--zoom-log2`); the colouring
comes from the SESSION, saved just before the child starts. Options: file, size, fps, octaves a
second, start depth, turns an octave, anti-aliasing (strips oversampled to match), BLA (on by
default, below), normalize, keep strips (folder). Before Render it shows the frames, length, strips, the window's GPU memory
and, keeping strips, their disk (~11 bytes a texel, measured). It refuses Julia, other fractals and
aux colourings up front, with the reason. `--uitest` step `zoom-video` (seeded mid-render): text
fits, no overlap — after the strips folder moved to its own line (a 200-pt field beside the
checkbox cut the path). The command it builds was run as is (strips kept, 2x anti-aliasing,
rotation, normalize): 181 frames, progress on stdout, 21 strips + manifest kept.
- **Not yet in E3:** preview/scrub in the view, recolour from kept strips in the app, farm and
  `--gpus` distribution by strips.

**The user's first real render (2026-10-28, RTX 3080, 10 GB)** — a dive to 2.2e36× (120.7 octaves),
their session's 150,000 base iterations (caps 150,220 to 179,040), 2x anti-aliasing:
- 5120x2880: the child refused after 13 s (a 36911-texel strip, past 32768); 3840x2160: it PANICKED
  allocating a 13-layer window of 27684x3058 (~17.6 GB) — the uncaptured-error handler. Now the
  dialog refuses both before Render (`zoom_video_fit`: the strip and the sample grid against the
  device's texture limit; the window against the card's memory, refused above 90%, warned above
  60%), and `Assembler::new` makes its allocations inside error scopes, so running out is an error
  that names the remedy.
- 1920x1080 ran (a 4.4 GB window), but its strips slowed from seconds to 1-3.5 minutes each (strip
  53 of 132: 213 s) while "left" (frames left / the average frame rate) said 1h14m with hours to go.
  Now each strip prints a line, and "left" is the strips still to render at the last three strips'
  pace plus the frames' own time (`assemble::time_left`) — still a floor when deeper strips cost more.
- **Why the strips were slow** (each strip now logs its `[perf]` line: GPU time, tiles, BLA skips,
  iterations a step). Strip 53 (21.2 M texels, mean escape 68,637): mode 0 (df32 perturbation, the
  1e4-1e28× band) has NO BLA (`bla_eligible` needs floatexp) — 1.1 iterations a loop trip — and ran
  on the corrector's tiles, sized `sqrt(CORRECT_WORK_BUDGET / cap)` ≈ 111 px: 1,786 tiles of a few
  percent of the GPU each. A bigger budget alone (2e10: 123 s; 2e11: 73 s) is bit-identical but put
  a 3.1 s dispatch on the desktop's card. The floatexp path's occupancy-sized, step-bounded tiles do
  it in 7 tiles at ≤240 ms a dispatch, and its full steps are df32 too:

  | strip (cap) | mode 0, today | floatexp, BLA off | floatexp + BLA |
  |---|---|---|---|
  | 13 (153,080) | 3.1 s | — | 2.0 s |
  | 34 (157,700) | 81 s | 26 s | 64 s |
  | 45 (160,120) | 110 s | 29 s | 17 s |
  | 53 (161,880) | 209 s | 41 s | 27 s |
  | 70 (165,620) | 195 s | 45 s | 17 s |

  Judged against the bignum oracle (`naive_dwell_bf`, 237-294 bits, 300 texels a strip, the
  selftest's exact / sensitive / wrong; 75-80% of texels at 2^45-2^70 are SENSITIVE — the oracle's
  own count changes within 1e-3 texel): floatexp with BLA off scored exactly as mode 0 on every strip
  and matches it on 99.997-99.998% of all texels. With BLA it is a different picture (8-73% of texels
  off by more than 2 iterations): fewer wrong than mode 0 at 2^45-2^70 (strip 53: 6.9% exact / 12.9%
  wrong against 1.7% / 20.3% on 1,000 texels) but 15 of 300 wrong by up to 10 iterations near the
  minibrot at 2^34, where mode 0 had none (and BLA was slower there). So strips in the df32 band now
  render as floatexp (`with_strip_arithmetic`), with BLA as the user chose (§9, decided 2026-10-28):
  an option, the dialog's "BLA acceleration" box, ON by default; the dialog always passes `--bla`
  or `--no-bla`, so the session's own setting never decides a video. Warm wall times, BLA off / on,
  interleaved: strip 34 27 / 66 s, 45 29 / 17 s, 70 47 / 17 s (the table's GPU times hold; the
  first run of a new build pays its shaders' compile once, ~17 s). The selftest's `expmap` group
  judges the BLA-off strip against the oracle and the BLA strip against THAT by the BLA group's
  rule (off the steep texels, within 0.5 iteration): 0 of 10,120 texels off, and 3,835 off with
  `BLA_EPS` loosened to 1e-2 (the control). Against the oracle directly a BLA strip misses the
  exact-arithmetic bar (27 of 48 exact against 60%; one texel 0.09 iterations outside the stencil).
- ⚠The LIVE view runs the same band the same way (mode 0, no BLA). Not changed here.

### 8.5 E4 (2026-10-28, branch `feat/zoom-video-farm`): strips on several machines

The user: "Update the zoom video to use multiple machines like the render script functionality."
A deep dive's time is its STRIPS (the user's 2.2e36 dive at 2x: 132 strips of 2 s to 66 s each;
its 7,246 frames assemble in minutes), and strips are independent once the dive's one reference is
known: the render farm (`design/remote-rendering.md`) hands them out as it hands out a tour's
frames, and this machine assembles the video when the last one is in.

- **The command:** `--render-zoom-video F … --farm` is the controller (the same line plus a flag;
  farm options as `--farm-render`'s: `--listen`, `--min-clients`, `--local`, `--share-root`,
  `--sharing`, …), and `--render-zoom-video F … --gpus all|N,M` a farm of this machine's cards,
  as `--render-tour --gpus` is. The strips go to `--keep-strips DIR`, else to `<F's stem>-strips/`
  beside the video (deleted once the video is made); the job's state is in that folder's `farm/`,
  so a stopped job resumes from it like a tour's.
- **One plan, checked everywhere:** the controller asks a child of its own for the dive's manifest
  (`--render-expmap DIR --expmap-plan`: the plan only, no strip), sends it in the job's bundle with
  the dive's flags (typed fields, never a raw argument list), and every client writes it into its
  render folder before its first run: a client whose own plan differs (another session, another
  build) is refused by `write_or_check_manifest`, loudly, instead of rendering another dive's
  strips.
- **One reference per dive, whatever the run:** a run of strips `A-B` now builds the dive's
  reference for the dive's DEEPEST strip, not its own deepest (it used to), so strips rendered in
  pieces, on any machine, are the strips one machine renders in one go — bit for bit on one GPU
  class. Orbit sharing passes it on (the cache is flushed before a strip child exits, which the
  `--render` exit path did not do).
- **Units:** a strip is a farm unit (`UnitKind::Strip`, named `strip_NNNNNN.exr`); the scheduler is
  unchanged (it was already generic over an index). A `--farm-child` strip run prints
  `frame-done index=K …` per strip; a strip already in its folder is reported, not skipped.
- **Verification:** SHA-256 as for frames, then the whole file read as assembly reads it
  (`assemble::strip_main`: every channel, `width × rows`) and its own metadata naming strip K at the
  plan's cap. A strip that fails is quarantined and counts a strike, as a frame does.
- **Transport:** a strip is 26 MB (720p) to ~520 MB (1080p at 3x); the per-unit cap goes from
  256 MiB to 1 GiB (protocol 7). ⛔A client sent a blob's chunks back to back, so its 2 s heartbeat
  waited behind a whole file and the controller dropped it after 10 s silent (a 233 MB strip over a
  290 Mb/s link takes ~6.5 s; a 256 MiB orbit had the same exposure): heartbeats now go out between
  chunks; every other message keeps its order.
- **The end:** the controller assembles the video here (`--assemble DIR --out F` with the command's
  assembly options, as a child in this machine's own configuration, so the colouring is the
  session's), relays its `frame K/N` lines, then deletes the strips unless they were kept.
- **In the app:** the Render zoom video dialog offers "Use all graphics cards" (two or more) and
  "Render on farm…", which opens Tools ▸ Render on farm… with this video as its job.
- **As built, measured on this machine (RTX 3080):**
  - A loopback farm (controller + `--local`), the user's view to 2^30 at 320x180: 39 strips in 50 s,
    the video assembled; every strip identical IN EVERY CHANNEL to `--render-expmap`'s on one
    machine. ⛔Not byte-identical, and that is not a difference: the EXR writer orders its
    compressed blocks as they finish — two single-machine runs differ as bytes on 29 of 39 files.
    Compare decoded channels, never file hashes.
  - `--gpus 1,1` (two sessions on the one card): 43 strips split 32/11, all identical, the
    assembly's `frame K/N` lines reaching the dialog unprefixed.
  - Stopped at 10 of 54 strips (the window's `stop`), then the same command: "10 of 54 strips
    already done", 44 rendered, all 54 identical, the video made.
  - `--farmtest-strips` (new): A's two corrupted strips quarantined and A removed, C through the
    shared drive (its folders cleaned up), 32 of 32 strips identical, status strips → assembling →
    finished. The tour `--farmtest` passes unchanged after the controller's refactor.
  - Fixed on the way: a resume of an already complete job (tour or strips) printed "complete"
    twice — `periodic` finished it again after `start_job` had.
- **Two machines (2026-10-28, dev.55):** this machine (RTX 3080, controller + a local client) and
  PLUTO through the field agent (RX 6800 XT on AMD 26.10.1, RTX 3070 on NVIDIA 617.14, a session
  each): the user's view to 2^60 at 1280x720, 50,000 iterations — 71 strips of 4614×511 in 104 s
  (43 here, 16 on the 3070, 12 on the 6800 XT), then the 1,801-frame video here. Against this
  machine alone: the 3080's and the 3070's strips identical in N on every texel; the 6800 XT's
  differ on 13% (31% in its worst strip). In the VIDEO, frames assembled where its strips show
  (2^28–2^40): 14–38% of pixels differ at all, 0.6–3.1% by more than 8/255, 0.2–1.7% by more than
  32/255 (mean 0.65–1.98/255); frames where none show (2^12, 2^44, 2^50) identical. ⭐LOOKED at
  2^31 side by side with a difference map: scattered single pixels in the filaments and on the
  minibrot's edge, no ring where its strips meet the others. Motion not judged.
- **Deepest strips first** (`sched::Config::from_end`, after the user's ask the same day): each new
  run takes the highest strips still free, so the slowest go out first and the cheap shallow ones
  fill the end; a run is still rendered ascending, so stealing its tail works unchanged. The
  scheduler's chaos test runs both ways.
- **Not in E4:** assembly pipelined with strips arriving (the frames wait for the last strip);
  a pre-build of a very deep dive's reference on the controller, and picking up a shared orbit
  mid-build (tours have both); keeping a dive on one GPU class (strips from two classes meet at a
  ring; measured on PLUTO before deciding).

### 8.6 Higher resolutions (2026-10-28, branch `feat/zoom-video-hires`)

The user asked what would make strips work at higher resolutions, then: "proceed as suggested",
with a note in the dialog on how it works and how it differs from Render tour. Two walls, both in
ASSEMBLY (§8.4's first render hit both):

- **The strip's width.** A strip is `q·π·diagonal` texels around: 36,911 at 5K with 2x
  anti-aliasing, past the 3080's 32,768-texel textures. ⭐Only the assembler's window was limited:
  a strip RENDERS in tiles of at most 2,048 (`render_iter_tiled`) and is read back into memory, so
  its width never met a texture. Now the window holds a strip wider than the card's textures in
  PIECES around its circle (`strip_pieces`: the fewest pieces, as equal as whole texels allow,
  each a layer of the window's array; the shader reads piece `i / piece_width` of slot `s` at layer
  `s·pieces + piece`). A sample reads the very texel it read before.
- **The window's memory.** A frame reads every strip from its corners to its centre at once:
  `layers_for_frame` = 13-15 strips, each `W × 0.11W` texels at 16 bytes (count, normal, DE) —
  ~19 GB at 4K 2x. The colour pass reads the count always, the distance estimate only for the
  distance colouring and the glow, and the normal only for relief (`fs_color`; the aux colourings
  are refused), so the window now holds only what the frames' colouring reads
  (`WindowTexels::for_colour`): 4 bytes for smooth colouring, 8 with the distance estimate or glow,
  16 with relief. What it leaves out is zero in the reprojected sample and never read. A frame whose
  colouring reads more than the window holds is refused (`Assembler::frame`), and `--assemble-at`
  with `--render-iter` (raw samples to a file) keeps everything. The aux target beside the frame's
  sample grid, which no allowed colouring reads, is one texel instead of 16 bytes a sample.

Graphics memory for assembly (the window and the frame, 10^9 bytes), before → after, smooth
colouring (the window's strips: 12 at 720p, 13-15 above):

| video | strip | before | now | |
|---|---|---|---|---|
| 720p, 1x | 4614 × 511 | 0.5 GB | 0.2 GB | (measured: 113 MB window + 44 MB frame) |
| 1080p, 2x | 13842 × 1529 | 4.7 GB | 1.3 GB | |
| 4K, 2x | 27683 × 3055 | 20.3 GB (crashed, then refused) | 5.5 GB | fits the 10 GB 3080 |
| 8K, 1x | 27683 × 3055 | 21.1 GB | 6.3 GB | |
| 5K, 2x | 36911 × 4073 | refused: strip too wide | 10.4 GB, 2 pieces | needs a ~12 GB card |
| 8K, 2x | 55366 × 6109 | refused: strip too wide | 23.5 GB, 2 pieces | |

With relief the window is as before (it reads everything); with distance colouring or glow, half.
The dialog now counts the frame too and warns or refuses on the total.

- **Checked:** the selftest's geometry check runs again with every strip in three pieces (346
  texels as 116 + 116 + 114) — every sample the texel `ExpMap::locate` names; a new check renders a
  dive's strips and colours its frames six ways (smooth and distance, with and without glow and
  relief), from the whole window and from one holding only what each colouring reads, in three
  pieces: bit-identical. Its CONTROL: the count-only window's samples differ from the whole
  window's in the normal and DE it leaves out, never in the count, and relief is refused on it.
- **Measured end to end (RTX 3080, a copy of the user's session: smooth colouring, `--iter 3000`):**
  - Through the real path: a 720p dive's kept strips (35 of 4614×511) assembled at five depths
    with the strips whole and in five pieces (`--set ASSEMBLE_PIECE_MAX=1000`): the PNGs
    byte-identical. The window held the count: 113 MB, and the frame 44 MB.
  - One 5K-at-2x strip, 36,911 × 4,073 texels (wider than the card's textures), rendered by
    `--render-expmap` in 14.7 s and written whole (1.2 GB EXR).
  - The user's 4K at 2x, a dive to 2^16 at two octaves a second and 30 fps: 29 strips of
    27683×3055 in 72 s, 241 frames, 141 s in all, no error; the window 4,736 MB holding the count
    and the frame 796 MB (the same video crashed at ~17.6 GB before §8.4's fix, then was refused).
    Frames looked at, a 1:1 crop of the centre included: clean.
- **The dialog's note** (the user: a user "would be confused by the two apparently similar
  functions"): one line under the view says what a zoom video is and how it differs from Render
  tour (faster, straight zoom only); a section that opens says how it works, what it makes, its
  limits, what Render tour does instead and which to choose (that line set apart). The Render tour
  dialog says the same from its side in one line, and the Tools menu's hovers in a sentence each.
  The memory line's hover says what the strips keep for the colouring chosen. `--uitest` step
  `zoom-video-about` photographs the section open: text fits.
- **Not done (the third fix, larger):** holding the inner strips coarser. Toward a frame's centre
  each octave inward is twice as fine as the frame reads it, so a window held at the resolution
  frames read would take ~2.3 strips' worth instead of 13-15 (about 6x less), at the cost of
  samples landing up to half a sample from where they land now and the full strips waiting in
  system memory or on disk until frames need them whole. 8K at 2x, or relief at 4K 2x, needs it.
  The farm's disk (every strip kept until assembly, ~11 bytes a texel: ~0.9 GB a strip at 4K 2x)
  is unchanged.

## 9. Decisions for you

**Decided 2026-10-27 (the user):**

1. **Order: as recommended.** E0 → E1 (strips zoomasm can assemble) first, then E2.
2. **The non-goal is retired** (`TODO.md`, the M6 non-goals line).
3. **Audio: none in the app for now.** The user adds audio to the finished video with their own
   tools; E2's `-i audio … -shortest` mux is out of scope until asked for.
4. **Iteration cap: both, as an option; the default is the rising (non-decreasing per-strip) cap.**
   One cap for the whole dive stays available for a dive where a step in the cap shows.

**Decided 2026-10-28 (the user):**

- **BLA on the df32-band strips: an option, ON by default** (§8.4): the dialog's "BLA
  acceleration" box, passed to the render as `--bla` / `--no-bla`.

The questions as they were put:

1. **Order.** Recommended: E0 → E1 first (small, immediately useful to zoomasm users, and F3 and
   zoomasm give outside validation of the geometry), then E2. Alternative: E0 → E2, skipping KF
   compatibility until someone asks.
2. **The recorded non-goal.** `TODO.md:8994` lists "built-in video encoding (export
   exponential-map EXR and let zoomasm own assembly)" as a non-goal. The tour export already
   encodes mp4 through ffmpeg, and E2 is built-in assembly, so it would be retired.
3. **Audio:** mux only (recommended), or timeline tools synced to audio like zoomasm's.
4. **Iteration cap over a dive:** one cap for the whole dive (simple, no seams, shallow strips
   iterate interior pixels to a deep cap) or a non-decreasing per-strip cap (cheaper; steps in the
   cap need hiding).

## 10. Risks and unknowns

- KF's DEX/DEY normalisation and T's exact definition are only described in prose; §5.3 decides
  them from F3's output, and F3 may itself differ from KF. Where they disagree, match what zoomasm
  renders correctly.
- A strip's per-pixel cost differs from a frame's: the outer rows of a strip lie outside the
  frame's rectangle (the corners' circle), so a strip renders ~2.55 frames' worth of pixels where
  a frame-by-frame dive's pixels are all on screen. The 23.5× figure already counts this.
- Interior-heavy dives: shallow strips at a deep dive's cap pay for their interior at that cap
  (§9.4).
- `rn_on()` and the renormalized step (`mandelbrot.wgsl:832`) are per pixel and need nothing
  rectangular; the live view's split refreshes and settle grids are live-only and not involved.
