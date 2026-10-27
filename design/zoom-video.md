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

### 8.1 E0 as built (2026-10-27, branch `feat/zoom-video`)

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
  an A/A control of main against itself read 24/43 and 16/43. The angle uses the strip module's own
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

## 9. Decisions for you

**Decided 2026-10-27 (the user):**

1. **Order: as recommended.** E0 → E1 (strips zoomasm can assemble) first, then E2.
2. **The non-goal is retired** (`TODO.md`, the M6 non-goals line).
3. **Audio: none in the app for now.** The user adds audio to the finished video with their own
   tools; E2's `-i audio … -shortest` mux is out of scope until asked for.
4. **Iteration cap: both, as an option; the default is the rising (non-decreasing per-strip) cap.**
   One cap for the whole dive stays available for a dive where a step in the cap shows.

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
