# Palette import: how other renderers do it, and what we should build

Survey written 2026-09-04, prompted by "add the Fractint default palette". It turned out that
request cannot be satisfied by adding a preset, and the reason generalises into a design.

Every claim below marked ✅ was read out of a real file or a working parser, not from
documentation prose. Claims marked ⚠ are unverified and must be confirmed before code depends on
them.

## 1. Why this is an architecture question and not a preset

Our palette is `name` + up to **`MAX_STOPS = 8`** `(position, linear RGB)` stops
(`fractadyne-color/src/lib.rs`), uploaded as `stops: [[f32;4]; 8]` in the `ColorU` uniform — in
**two** places, the live path (`fractadyne-gpu/src/lib.rs`) and the offline one
(`export.rs`) — and the shader's `palette()` walks that list interpolating linearly between
adjacent stops. The paste-import path already resamples anything longer down to 8 via
`resample_colors`.

✅**Fractint's `default.map` has 37 hard jumps** (adjacent entries differing by >60/252) across
its 248 used entries. Eight stops give at most seven interpolated segments. So an 8-stop rendition
of it is not "lower fidelity" — it is *structurally incapable*, and the Fractint look **is** those
discontinuities. Any palette-import feature worth having hits the same wall on its first real
file.

## 2. What the other renderers actually do

### Fractint / Kalles Fraktaler — `.map` ✅

256 lines of `R G B`, optionally with a trailing comment on line 1. Applied as a **lookup table
indexed by iteration count**, with no interpolation between entries — which is why the classic
look is banded.

⭐**Two traps, both verified in `default.map` itself:**
- Values are **6-bit VGA (0–63) scaled ×4**, so every value is a multiple of 4 and the maximum is
  **252, not 255**. Dividing by 255 desaturates the whole palette slightly.
- The table is structural, not artistic: entries 0–15 are EGA/VGA text colours, 16–31 a greyscale
  ramp, 32–247 are 216 colours on an HSV cylinder, and 248–255 are black padding. Only 32–247 is
  "the gradient"; the first 32 are a system palette that happens to be in the file.

⚠KF also has `.kfp` palette files that override the colour settings inside a `.kfr`. Our
`parse_kfr` deliberately reads only `Re`/`Im`/`Zoom`/`Iterations` and ignores everything else, so
KF colour data passes us by today. Format unverified.

### Ultra Fractal — `.ugr` ✅

Verified against a real file (`fract4d/gnofract4d:maps/blatte1.ugr`) and gnofract4d's parser:

```
blatte10 {
gradient:
  title="blatte10" smooth=no index=0 color=3085069 index=25 color=3216141
  index=56 color=10761236 ... index=384 color=144
opacity:
  smooth=no index=0 opacity=255
}
```

- Plain text, **many gradients per file**, each a named `name { ... }` block — so an importer must
  present a *list* to choose from, not load "the" gradient.
- A `gradient:` section of interleaved `index=`/`color=` pairs, free-wrapped across lines, plus
  optional `title=`, `smooth=yes|no` and `rotation=`; a separate `opacity:` section.
- **Indices are 0–399**, not 0–255 and not 0–1.
- ⭐`color=` is a decimal integer packed **BGR — `0xBBGGRR`**, i.e. red is the LOW byte. Verified
  from gnofract4d's decode (`icolor & 0xFF` → red, `(icolor >> 16) & 0xFF` → blue). Getting this
  backwards swaps red and blue on every imported gradient and still looks plausible, which is
  exactly how it survives review.
- ⚠gnofract4d divides those bytes by **256.0**, not 255.0. Deliberate or not, it means their
  import is a hair dark; we should divide by 255 and say so.

### GIMP — `.ggr` ✅

The richest model of the three, and the one worth stealing. A `Name:` line, a segment count, then
one line per segment:

```
left mid right   lr lg lb la   rr rg rb ra   blend_mode color_mode
```

Positions are floats in 0..1, colours floats 0..1 **with alpha**, plus **per-segment**:
- a **blend function**: linear, curved, sine, sphere-increasing, sphere-decreasing;
- a **colour space**: RGB, HSV counter-clockwise, HSV clockwise — so a single segment can sweep
  the long way round the hue wheel;
- a **midpoint**, which shifts where the blend reaches 50% without adding a stop.

A `+` prefix marks a compressed continuation reusing the previous segment's right edge.

### Swatch lists — Adobe `.ase`, ColorSchemer `.cs`, hex/CSS ✅

Flat ordered lists of colours with no positions and no interpolation semantics. Importing means
choosing a spacing convention (even) and a blend (linear). We already accept the hex/CSS case via
`parse_palette_text` (`#rgb` and `#rrggbb`).

### The rest ⚠

XaoS generates palettes algorithmically from a seed rather than storing them; Fraktaler-3 colours
via its own formula rather than a palette file; Imagina has its own scheme. None of these are
*import sources* — they are reminders that "palette" is not universal, and that a renderer can
legitimately have no palette file at all.

## 3. The prior art worth copying

**gnofract4d** ([fract4d/gnofract4d](https://github.com/fract4d/gnofract4d), `fract4d/gradient.py`)
already solves precisely this problem in production: it loads `.map`, `.ugr`, `.ggr`, `.cs` and
`.ase`, plus lists and URLs, and converts **all** of them into one internal representation —
GIMP's segment model (`Segment(left, left_color, right, right_color, mid, blend_mode,
color_mode)`).

That is the design answer, and it is load-bearing: **GIMP's segment model is a superset of every
other format here.** Each one maps into it without loss:

| Source | Maps to |
|---|---|
| `.ggr` | natively |
| `.ugr` | one linear RGB segment per adjacent index pair, position = `index / 399` |
| `.map` | 256 segments, **flat** (left colour == right colour) to preserve the banding, or 255 linear segments if the user wants it smoothed |
| `.ase` / `.cs` / hex list | evenly spaced linear RGB segments |
| our presets | unchanged — 8 stops is just a short segment list |

## 4. Proposed architecture

Three layers, and the middle one is what we lack:

1. **Importers** — format → segment list. Independent, individually testable, no GPU.
2. **A segment model on the CPU**: `left`, `mid`, `right`, RGBA endpoints, blend function, colour
   space. Superset of everything above; the custom-gradient editor and the built-in presets are
   both expressible in it.
3. **Bake to a LUT** — evaluate the segment list into an N-entry table (256 minimum; 1024 gives
   headroom for `cycle`) and upload that, replacing the 8-stop uniform array.

Baking is what removes the `MAX_STOPS = 8` ceiling *and* makes hard-banded `.map` files, curved
GIMP blends and HSV-sweep segments all cost the same at render time — the shader stops walking a
stop list and does one indexed fetch. Either widen the uniform (256 × 16 B = 4 KB, comfortably
inside a uniform buffer) or add a 1-D palette texture at binding 3; the texture is tidier, the
uniform is a smaller diff. **Both `ColorU` definitions must change together** — the live one and
the export one — or offline renders silently keep the old palette.

## 5. The three fidelity traps

Ranked by how quietly they produce a wrong picture that survives review:

1. ⭐⭐**Interpolation semantics are per-format, not global.** Fractint indexes a LUT with no
   interpolation; UF and GIMP interpolate between stops. Applying our smooth interpolation to a
   `.map` turns 37 hard bands into a smear that no longer resembles the source. The segment model
   handles this by making flatness a property of the *segment*, not a global switch.
2. ⭐⭐**Colour space — and we currently contradict ourselves about it.** Two comments in the tree
   assert opposite things, and both are load-bearing:
   - `fractadyne-color/src/lib.rs`: *"every stop in this crate is LINEAR"* — and
     `parse_palette_text` converts sRGB→linear, pinned by the test
     `palette_text_converts_srgb_to_linear` (`#808080` → 0.2159).
   - `fractadyne-export/src/lib.rs`: *"The renderer is **display-referred**. `fs_color` writes
     palette colors (0..1) straight into a **non-sRGB** framebuffer… the bytes the GPU stores ARE
     the sRGB values the monitor shows… **Palette interpolation and relief lighting therefore also
     happen in gamma space, by design**."* ✅Confirmed: there is no linear→sRGB encode anywhere in
     `mandelbrot.wgsl`.

   ✅⭐⭐**SETTLED BY EXPERIMENT 2026-09-04 — the renderer is display-referred, and
   `parse_palette_text` is wrong.** Method: a session with a UNIFORM custom palette (every stop
   the same colour, so palette POSITION cannot influence the result), `--render` at 240×240, then
   a histogram of the PNG. `light`/`de`/`normalize_live` forced off so nothing modulates the
   colour. Both runs asserted the `session: … — loaded` line first; two earlier attempts silently
   fell back to defaults and would have "measured" nothing.

   | stop value written to the session | dominant rendered pixel |
   |---|---|
   | **0.2159** — what `parse_palette_text("#808080")` produces | **`#373737`** (49,575 / 57,600 px) |
   | **0.502** — the DISPLAY value of `#808080` (control) | **`#808080`** (51,196 px) |

   The control settles the mechanism, not merely the direction: **stop values are written straight
   through to the framebuffer as display bytes.** `srgb8_to_linear(128) = 0.2159`, and
   `0.2159 × 255 = 55.05 → 55 = 0x37`, matching the observed byte exactly.

   ⇒ **Pasting `#808080` gives you `#373737`.** Every pasted palette is one sRGB decode too dark.
   The `fractadyne-export` comment is right and the `fractadyne-color` one describes an intent the
   renderer does not honour. The presets escape the bug because they were authored by eye against
   the live view, so their numbers are display values by construction — the space only bites where
   the user has a stated expectation, which is exactly what import is.

   ✅**FIXED 2026-09-05 — fix (a) applied.** `parse_palette_text` now returns `v / 255`
   (`srgb8_to_stop`), the space the renderer actually interprets. Verified end to end: the value
   the fixed parser produces for `#808080` (0.50196…) rendered as **#808080**. The test that
   pinned the bug (`palette_text_converts_srgb_to_linear`, asserting 0.2159) is inverted and
   renamed `palette_text_is_display_referred_not_linear`, and the "linear RGB" claims in
   `fractadyne-color`, `fractadyne-state` and `main.rs` now say DISPLAY-space. Fix (b) below
   remains the open architectural alternative.

   ⭐⭐**A second bug fell out of writing the test: a `.map` triple was being read as three hex
   shorthands.** `168 168 168` is a real line in Fractint's `default.map` (the VGA light grey) and
   each token is also three valid hex digits, so the "hex tokens win" rule turned one grey into
   THREE `#114488` colours. **Every `.map` line whose three values all land in 100–255 was
   silently mis-imported** — in the one external format this parser already advertised. Fixed by
   preferring the integer reading when every token is pure decimal, the count is a multiple of
   three, and all values are ≤255; shorthand containing a letter (`f80`) or a lone token (`128`)
   still takes the hex path. Pinned by `map_triples_beat_bare_hex_shorthand` using the exact greys
   and whites from `default.map`.

   **Two candidate fixes, and they are not equivalent:**
   - **(a) Stop converting on import** — `parse_palette_text` returns `v / 255`. One line plus
     inverting `palette_text_converts_srgb_to_linear`, which currently pins the bug. Matches the
     renderer as designed, and is what the importers below should target.
   - **(b) Make the renderer linear** — encode sRGB at the end of `fs_color` and treat stops as
     truly linear. Colour-managed, better gradient midtones, and it would change the appearance of
     every existing preset and saved palette plus the export path's stated assumptions. A much
     larger change, and it contradicts a documented deliberate choice ("it matches what the user
     sees while exploring").

   (a) is right for now; (b) is a separate decision that the LUT work would be the natural moment
   to revisit. Either way, **state the space once, in one place** — it is currently asserted twice
   in opposite directions.

   ⚠Not tested: the gradient editor writes `ui.color_edit_button_rgb` values into `custom_palette`
   with no conversion, so whether the editor's SWATCH matches the rendered colour is a second,
   separate question.

   Separately and regardless: interpolating *in* linear gives different midtones than an
   application that interpolates in sRGB. Matching a reference render means matching the
   interpolation space, not just the endpoints.
3. ⭐**Value scaling.** Fractint's 6-bit ×4 maximum of 252; gnofract4d's /256 for UGR. Off-by-a-
   few-percent errors that look like nothing and fail an exactness comparison.

## 5a. Precision: how many bits, and where they actually run out

✅Verified formats through the pipeline:

| Stage | Precision |
|---|---|
| Import formats (`.map`, `.ugr`, `.ggr`, hex) | **8 bits/channel** — Fractint effectively **6-bit** (0–63 ×4, max 252) |
| Palette stops | `[f32; 4]` — **32-bit float** |
| Iterate + aux textures (`ITER_FORMAT`) | `Rgba32Float` — **32-bit float ×4** (smooth iteration, slope normal, DE) |
| Offline export target (`EXPORT_FORMAT`) | `Rgba32Float` |
| Shader colour maths | f32 |
| **Live display** | `Bgra8Unorm`/`Rgba8Unorm`, non-sRGB — **8 bits/channel**, and **no dither** |
| **PNG export** | 8 bits/channel **with ordered Bayer dither** (`to_srgb8_dithered`) |
| **EXR export** | 32-bit float, linear |

**8-bit palette DATA is not a limiting factor.** Endpoints arrive at 8 bits, but everything from
the stop onward is f32, so the interpolated values between them are continuous; the quantisation
costs at most ±1/255 at the stops themselves.

**8-bit OUTPUT is a limiting factor**, and we already know it: 256 levels per channel across a
slow gradient contours visibly, which is why the PNG writer dithers at all (banding was recorded
as "the #1 newcomer complaint"). ⚠Note the asymmetry that follows from the table: **the exported
PNG is dithered and the live view is not**, so a gradient can look smoother in the export than it
did on screen. Worth confirming and, if real, worth a dither in `fs_color` too — it is a few lines
and the same ±½ LSB trick.

⭐⭐**But the real ceiling for a fractal renderer is not channel depth at all — it is palette
POSITION resolution.** What matters is how finely the continuous smooth-iteration value can
address the palette, and two things bound it: the **LUT length** once we bake, and at depth the
compression of escape values into a narrow band (what `--normalize` and the log scale exist for).
At a high `cycle` the palette repeats many times across that narrow band, and position
quantisation shows up long before colour quantisation does.

⇒ **Bake to 1024 entries, not 256**, and interpolate *between* LUT entries rather than
nearest-fetching — except where a format demands flat bands, which is exactly Fractint's case.
1024 × 16 B = 16 KB, still comfortably inside a uniform buffer, and trivial as a 1-D texture.

## 6. Recommended order

1. **Segment model + LUT bake + the GPU change.** Nothing else is possible without it, and it
   immediately lets the existing paste-import stop throwing away colours past the 8th.
2. **`.map`** — simplest format, and `default.map` is a ready-made fixture with known structure.
3. **`.ugr`** — biggest community corpus; needs the multi-gradient picker UI.
4. **`.ggr`** — richest, and the model is already native by then.
5. **`.ase` / `.cs`** — cheap once swatch-list import exists.

## 7. Before writing code

- ⚠**Licensing.** Fractint ships under the Stone Soup licence, not MIT/Apache. Importing a user's
  own `.map` raises nothing; *embedding* Fractint's table in this repo is redistribution and needs
  a call. The import-first order above sidesteps it by construction.
- ⚠**Verification bar** (set previously for `.map` import, and it should hold for all of these):
  exactness, not "it loads" — render through the imported palette and compare against the source
  application's own render of the same palette. `default.map` plus a Fractint render is the first
  such fixture; a `.ugr` plus an Ultra Fractal render would be the second.
- ⚠KF `.kfp` and the colour keys inside `.kfr` are unverified. Worth a look, since we already
  parse `.kfr` for locations and ignore its colour data.
