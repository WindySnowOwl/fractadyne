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

### ⭐DECIDED (user, 2026-09-05): ONE path — everything goes through the LUT

The alternative was to keep the existing ≤8-stop shader path untouched and add the LUT only for
palettes needing more, which would have held every current render byte-identical and kept the
goldens and corpus green by construction. **That is explicitly NOT the plan.** Presets, the custom
gradient editor and every import all bake to the same LUT, and the old stop-walking path goes
away. One code path, one set of semantics, no "which palette am I on" branch to reason about.

⚠**This moves pixels, and that is accepted in advance.** Baking a piecewise-linear stop list into
N entries and interpolating between them reproduces the original exactly *except* near stop
positions that fall between LUT entries, where the kink is rounded. So:

- `--selftest` goldens (**18/18**) will drift.
- The F3 corpus (**38/38 at maxD 0**) will drift — every row, since the colour mapping changed.

**Both get a deliberate re-bless, AFTER the change is shown correct — never before.** The order
matters: re-blessing first turns the gate into a rubber stamp and destroys the only record of the
old output.

⭐**Acceptance criteria for the re-bless** (so "it changed" is not confused with "it broke"):

1. The delta must be **small and explainable** — bounded by LUT quantisation, concentrated at stop
   boundaries. Measure it (decoded-RGB maxD / meanΔ per corpus row), do not eyeball it. A large or
   widely-spread delta means the bake is wrong, not that the gate is stale.
2. **Raising the LUT size must shrink the delta.** Bake at 1024 and at 4096 and compare against the
   pre-change render: if the error does not fall, the difference is not quantisation and the bake
   has a real bug.
3. A **flat-segment `.map`** must come back with its bands intact — that is the case the whole
   design exists for, and it is the one an interpolating bake would quietly smooth.
4. `validation/golden/BLESSED-GPU.txt` must survive the re-bless (it is what gives non-3080
   testers the cross-GPU tolerance instead of strict comparison).

⚠**Budget the corpus re-bless deliberately**: blessing is far more expensive than checking, and
row 39 alone took ~6,600 s. Re-bless the routine 38 first, and treat the extreme row as its own
scheduled job.

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

## 5b. BUILT 2026-09-05 — what shipped, and what the numbers actually were

Steps 1–3 of §6 landed in **beta.23** and step 2 of the order (`.map`) in **beta.24**. Recording
the measurements here because §4's acceptance criteria were written as predictions and two of
them came out differently from the prediction.

**The goldens did NOT need a re-bless.** §4 budgeted a deliberate one and warned the F3 corpus
would drift on every row. The actual drift on `--selftest`: **170/170 checks, 18/18 goldens, all
PASS** — maxΔ **1** (one 8-bit level) on sixteen, maxΔ 0 on `newton`, and the maxΔ 8 on
`mandelbrot-1e6` is the figure already in the committed report, i.e. pre-existing. So the
re-bless plan is unspent for THEM, and the old output is still the record.

**The F3 corpus DID go red, exactly as §4 predicted, and was re-blessed** (user decision,
2026-09-05, after the evidence below). Its rows compare at maxD **0** — a hard zero — so the same
one-level shift that the goldens absorb inside their tolerance fails every row there. Measured,
all 38 routine rows, 1280×720 2×SS:

| | |
|---|---|
| rows CHANGED | **38 / 38** |
| maxD | **1** on every row |
| meanD | **0.00** on every row |
| differing pixels | **0.004% – 0.182%** (34 px to 1,675 px of 921,600) |
| depth range covered | 1× to **1.2e1008×** — the shift is depth-independent |

A structural change cannot look like that. The iterate pass was not touched; only `palette()`
was, and a uniform ±1 on a fraction of a percent of pixels at every depth is the signature of
rounding, not of different mathematics.

⚠⚠**ROW 39 (the EXTREME Misiurewicz row) WAS NOT RE-BLESSED** — the regenerate skips extremes
without `--extreme`, and that row alone is ~6,600 s. **Its committed baseline is therefore from
the PRE-LUT renderer**, so the next `--check --extreme` will report row 39 CHANGED. That is
expected and is NOT a regression; it needs its own scheduled re-bless.

⭐**The corpus check now reports differing PIXELS as well as maxD**, because maxD saturates (see
criterion 2 below) and "0/38 CHANGED, maxD 1" reads far more alarming than "0.03% of pixels moved
by one level" — the same fact.

⭐**Acceptance criterion 2 needed a different metric than the one it names.** "Bake at 1024 and at
4096 and compare … if the error does not fall, the bake has a bug" — but maxΔ **saturates at the
output quantum**: at 4096 the goldens still reported maxΔ 1, identically, because a value within
1e-5 of a rounding boundary still rounds the other way. Counting DIFFERING PIXELS shows what maxΔ
cannot:

| LUT entries | differing pixels, all 18 goldens | largest difference |
|---|---|---|
| 1024 | 16,372 | 1 level |
| 4096 | 1,496 | 1 level |

A **10.9× fall** for a 4× LUT. The error is quantisation, and 1024 is comfortably enough.
⭐The lesson generalises: *a metric that saturates cannot show a trend*, and "the number did not
move" would have read as a failed criterion here when the bake was in fact correct.

✅**Criterion 3 (a flat `.map` keeps its bands) verified END TO END**, not just in the bake. A
16-entry `.map` of distinct greys rendered through `--palette-map` at 400×400: **exactly the 16
declared levels, zero off-palette values** across 159,993 exterior pixels — and the PNG writer's
ordered dither did not smear them, because a `v/255` entry is exactly representable. The control
that makes this mean something: the *same file* with `--palette-map-smooth` gives **253** distinct
levels. Without it, "few colours" would have been indistinguishable from "few colours in the
image".

✅Criterion 4: `BLESSED-GPU.txt` untouched — nothing was re-blessed.

**Also fixed on the way past**: the gradient editor's preview bar was gamma-encoding (`c^(1/2.2)`),
the last survivor of the "stops are linear" belief §5 disproved, so every swatch read markedly
lighter than the pixel it stood for (0.502 previewed at 186, rendered at 128). It now samples the
same baked LUT the GPU fetches from, which closes the ⚠ left open at the end of §5's trap 2.

**And a gate that did not exist**: the Rust `ColorUniforms` and the WGSL `ColorU` describe the
same opaque bytes, and nothing checked it — a field added to one alone builds, runs, and colours
every pixel from the wrong offsets. `uniform_layout_tests` now parses the WGSL struct, lays it out
under WGSL's rules and compares sizes; verified it goes red by adding one `f32` to the shader
alone. That is the mechanical form of §4's "**both `ColorU` definitions must change together**".

### What `.map` import does, and the one decision inside it

`fractadyne_color::import::parse_map` → `MapPalette` → `bands()` (the default) or `smooth()`.
Reachable from the gradient editor (**Import .map…**, plus a *Hard bands* checkbox) and from the
CLI as **`--palette-map FILE`** / **`--palette-map-smooth`**. The CLI form exists because §7's
verification bar — compare against the source application's own render — needs a headless render
through a `.map`, which a GUI button cannot give a script.

⭐**The 6-bit VGA table is DETECTED and REPORTED, and deliberately not rescaled.** §5 trap 3 flags
Fractint's ×4 maximum of 252 as an off-by-a-few-percent error waiting to happen, and the tempting
fix is to rescale 252 → 255 since 63 meant full intensity on a VGA DAC. That would be wrong *for
this bar*: Fractint writes its own images with the same ×4, so its white pixels ARE 252, and
rescaling would fail every comparison against a Fractint render by ~1.2%. The importer reports the
table as 6-bit in the UI message instead, so the user knows why their white is not 255.

### `.ugr` import (beta.25)

`parse_ugr` → a list of `UgrGradient`, each `to_gradient()`-able. The editor shows a picker with a
preview swatch per row (a name like `blatte10` says nothing about what it looks like); the CLI is
**`--palette-ugr FILE`** / **`--palette-ugr-name NAME`**, first-gradient default, and it PRINTS
which it took — silently choosing one of dozens is how a scripted comparison measures the wrong
palette.

⭐⭐**§2's BGR warning is now verified at the PIXEL, not just in the parser.** `color=255` renders
RED and `color=16711680` renders BLUE through the full pipeline; under the natural RGB reading
those swap, and the result still looks like a plausible palette. That is the whole reason the
warning was written down, so the test renders rather than asserting on a parse.

Decisions inside it, all stated rather than silent:

| | |
|---|---|
| `index` | `/399`, matching gnofract4d's segment-per-adjacent-pair reading; ends clamp |
| byte scale | **`/255`**, not gnofract4d's `/256` — so ours reaches pure white and theirs does not |
| `rotation=` | **applied**, direction ⚠UNVERIFIED against UF; pinned by a test so a fix is one sign |
| `smooth=yes` | **recorded, not honoured** — UF's smooth is a spline through control points, not a per-segment blend function. gnofract4d imports linearly for the same reason |
| `opacity:` | parsed and carried, though the renderer's palette has no alpha today |
| non-gradient blocks | skipped, so a file mixing formula/parameter blocks still loads |

⭐**A rotation exposed a real hole in the model, found by a round-trip test rather than by
reading**: rotating a non-seamless gradient creates a genuine DISCONTINUITY, and `to_stops` was
smoothing it into a ramp on the way into the session file. A hard edge is now carried as a
DUPLICATE POSITION — which is how every gradient editor expresses one, and which `from_stops`
already read back correctly. `Gradient::rotated` also SPLITS the segment that straddles the seam
rather than sorting and losing its far half.
### `.ggr` import (beta.26) — the one that loses nothing

§3 said GIMP's segment model is a superset of every other format here and that we should steal it.
We did, in beta.23 — so `.ggr` import needs no lowering at all: a file segment IS a
`segment::Segment`, midpoint, blend function and colour space included. `parse_ggr` → `Gradient`,
and that is the whole conversion.

⭐⭐**The HSV sweep is verified AT THE PIXEL, with a one-integer control.** The same one-segment
file, red endpoints at both ends, rendered twice with only the colouring column changed:

| colouring column | distinct colours in the render |
|---|---|
| `0` (RGB) | **4** — flat red, as RGB interpolation between identical endpoints must be |
| `1` (HSV counter-clockwise) | **810** — a full circuit of the hue wheel, green and blue both reaching 255 |

That is the case §2 called "a single segment can sweep the long way round the hue wheel", and it is
the reason the model carries a colour space per segment rather than a global one.

Both the 13-column and the GIMP 2.x 15-column forms load (the two extra record where each
endpoint's colour comes from — foreground, background, fixed — which is not a thing a fractal
renderer has). The `Name:` line is optional, because GIMP 1.x files predate it.

⚠**§2's `+` compressed continuation is REJECTED BY NAME, not implemented.** That claim was never
confirmed against a real file or a working parser, and guessing at a compression scheme is how a
palette gets silently mis-imported. A file using one fails loudly and says why.

⭐⭐**This forced a new session field, and the reason is worth keeping.** The app persisted a custom
palette as `[pos, r, g, b]` stops, which cannot hold a midpoint, a blend curve OR a hue sweep — so
storing a `.ggr` that way would have flattened all three on the **first restart**, silently, and
the only symptom would have been "it looks different from when I imported it". `custom_segments`
holds the real thing and WINS over the stop list when set. ⚠Every path that sets stops must clear
it (presets, paste, `.map`, `.ugr`, add-stop) or those edits appear to do nothing at all: the UI
responds and the picture does not, which is the most confusing failure available. The editor
refuses to pretend it can edit what it cannot — a "Convert to editable stops" button says what it
discards, instead of flattening the gradient on the first click.
### `.ase` import (beta.27) — and why `.cs` was NOT written

Adobe swatch exchange: big-endian binary, `"ASEF"` + a block walk, group markers flattened (a
gradient keeps the file's ORDER and nothing else), RGB / Gray / naive CMYK. A swatch list carries
no positions and no interpolation semantics, so "evenly spaced, linearly blended" is a choice
**we** make — stated, not implied.

⚠⚠**This is the one importer NOT verified against a real file.** Every other one here was written
against a real file or a working parser; no `.ase` was to hand, so this came from the published
layout alone. Two consequences worth being explicit about:

1. It is deliberately **STRICT** — signature, block lengths and colour model must all agree — so a
   file that disagrees with this understanding is REJECTED with a reason rather than importing
   plausible-looking wrong colours. ⭐**The first real `.ase` anyone imports is the actual test.**
2. ⚠**The unit-test fixtures are built from the same understanding as the parser**, so them
   agreeing proves consistency, not correctness. That is written at the top of the fixture builder
   so nobody later mistakes green tests for a verified format.

⚠**LAB swatches are REFUSED by name, not converted.** LAB → sRGB needs a white point (Adobe uses
D50) and a chromatic adaptation, and writing that from memory is the kind of guess that produces
colours which look fine and are wrong. A file MIXING LAB with other models is refused too —
importing "the rest" would silently drop swatches, which is worse than refusing. Same call as the
`.ggr` `+` continuation.

⛔**`.cs` (ColorSchemer) is deliberately NOT implemented.** It is binary and I have no verified
layout for it. Writing one from a guess would contradict the two decisions immediately above, made
in the same session. gnofract4d's `fract4d/gradient.py` is the reference if it is ever wanted — but
the product is long dead, and the paste box already handles text swatch lists, which is what most
people actually have.
### Selftest coverage (beta.28) — the manual checks became gates

Every end-to-end check above was run BY HAND when its importer shipped, and a manual check is not
a gate: it does not run again. `--selftest` is now **170 → 173**:

| check | threshold |
|---|---|
| `map-bands-are-exact` | zero off-palette pixels, and the smoothed control has >3× the levels |
| `ugr-color-is-bgr` | each render's own channel leads the other by 2×, **both ways round** |
| `ggr-colour-space-is-per-segment` | the hue sweep yields >10× the distinct colours of the RGB reading |

⭐⭐**Each is a CONTROL PAIR — the same file with ONE field changed — because the naive form of each
cannot fail.** "The `.map` render has few colours" is also true of a smoothed import of a dark
palette; "the `.ugr` render is red" is also true if red and blue were swapped and the file happened
to be red. Requiring the OPPOSITE answer from the altered file is what gives them the ability to
go red at all — the same reasoning as §5b's "a saturating metric cannot show a trend".

⭐**And they were VERIFIED RED**, by breaking the three mechanisms they test (the flat flag off, the
BGR decode read as RGB, the colour space forced to `Rgb`):

| check | healthy | mutated |
|---|---|---|
| `map-bands-are-exact` | 5 levels, **0** off-palette | 253 levels, **32,854** off-palette |
| `ugr-color-is-bgr` | r 149.7 / b 3.2, then r 2.1 / b 150.7 | exactly swapped |
| `ggr-colour-space-is-per-segment` | 4 vs **1533** colours | 4 vs **4** |

⚠`.ase` is deliberately NOT covered: its own format is unverified, so a render check would only
confirm that our parser agrees with itself — which the unit fixtures already do, and which is not
the same as being right.

⚠⚠**A trap worth recording from doing this**: restoring the pre-mutation sources with `mv` gave
them the BACKUP's older mtime, so cargo saw the sources as older than the artifacts and skipped
recompiling — `Finished` was printed and the binary was still the MUTATED one, so the "reverted"
run reproduced the failures exactly. `touch` the files after restoring. Same family as the
already-recorded "`cargo test` does not rebuild the exe".
⚠**Still outstanding for §7's bar**: an actual Fractint render of `default.map` to compare
against. What is proven today is that the bands are exact and that the file's values reach the
framebuffer unaltered; what is NOT proven is that our iteration→palette-index mapping matches
Fractint's. Those are different claims and only a side-by-side settles the second.
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
