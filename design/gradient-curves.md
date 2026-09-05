# Parametric gradient editing: curves between stops, in RGB and HSV

Evaluation written 2026-09-05, prompted by "evaluate parametric gradient editing using curves
between stops in RGB and HSV". Everything marked ✅ was read out of the tree or measured by running
it; ⚠ marks a claim that is reasoned but unverified.

## 1. The finding: the engine is already parametric. The editor is not.

This is the whole shape of the job, and it is much smaller than the phrase suggests.

✅**Already built, shipping, and gated** (`fractadyne-color/src/segment.rs`, beta.23–26):

| capability | state |
|---|---|
| Per-segment **blend curve** — linear, curved, sine, sphere-increasing, sphere-decreasing | evaluated, baked, persisted |
| Per-segment **colour space** — RGB, HSV counter-clockwise, HSV clockwise | evaluated, baked, persisted |
| Per-segment **midpoint** — moves where the blend reaches 50% without adding a stop | evaluated, baked, persisted |
| Per-endpoint **alpha** | parsed and stored (renderer has no alpha channel yet) |
| **Cost at render time** | one indexed LUT fetch — a curve costs exactly what a line costs |
| **Persistence** | `fractadyne_state::PaletteSegment`, round-trip tested |
| **Interchange** | `.ggr` import exercises all of it; a selftest check renders it |

✅**Not built**: any way for a user to set those three things. The gradient editor edits
`custom_palette: Vec<[pos, r, g, b]>` — colour and position, nothing else — and **clears
`custom_segments` on every edit** (`main.rs`, the ⚠ on that field says why).

So today the rich model is reachable only by importing a `.ggr` someone else authored. **"Parametric
gradient editing" is a UI feature on top of a finished engine**, plus one data-model decision.

⭐This also means the expensive half is already paid for and already has its acceptance evidence:
the LUT bake, the `.ggr` round trip, the `ggr-colour-space-is-per-segment` selftest check (4 vs 1533
distinct colours), and the session round-trip test that names midpoint/blend/space individually.

## 2. The one real design decision: which representation the editor owns

Right now there are two, and they are mutually exclusive by construction:

```
custom_segments: Vec<PaletteSegment>   // rich; WINS when non-empty; only .ggr writes it
custom_palette:  Vec<[pos,r,g,b]>      // stops; what the editor edits; every edit clears the above
```

That either/or is correct for *import* (an imported `.ggr` is not editable as stops without loss,
and the editor says so before discarding). It is wrong for *editing*, because the moment a user sets
a curve on a segment, the stop list can no longer describe their gradient.

**Recommended: make `custom_segments` the single source of truth for a custom gradient.**

- The editor edits segments. Stops become a *view* of segment boundaries — which is what they
  already are (`Gradient::to_stops` derives them, and carries a hard edge as a duplicate position).
- `custom_palette` stays for (a) sessions written before this, (b) `.map` band imports, which are
  genuinely a colour list, and (c) the paste box. Loading either produces segments via
  `Gradient::from_stops` / `from_bands`, which is already how rendering works.
- ⚠**This is a migration, and the migration is the risky part, not the maths.** A session written
  today has stops and no segments; it must keep rendering identically. `from_stops` produces exactly
  Linear/RGB/centred-midpoint segments, so the conversion is lossless *by construction* — but that
  needs a test asserting an old session renders byte-identically after the change, not an argument.

The alternative — keep both, and "promote to segments" on the first curve edit — is what you build
if you are afraid of the migration. It leaves two code paths and the same trap (a stop edit silently
discarding curves) permanently in the tree.

## 3. What is free, what is cheap, what is real work

**Free — UI only, no engine change, no pixel movement:**

- A blend picker and a space picker per segment. Both enums already bake and persist.
- A midpoint control. ⚠It is normalised *within* the segment (GIMP semantics), so a marker dragged
  along the strip maps to `(mid − left) / (right − left)`, not to an absolute position.

**Cheap and worth doing alongside:**

- ⭐**`.ggr` EXPORT.** The internal model *is* GIMP's, so writing one is a formatter over
  `Gradient` — on the order of 40 lines. It gives users a way out, makes every other application a
  test oracle, and turns `.ggr` import + export into a round-trip property test, which is the
  cheapest correctness evidence available for this whole area.
- A curve preview per segment (a small sparkline of the blend function). ⚠Draw the *real* function,
  not an idealised S — the two sphere blends are deliberately **not** midpoint-symmetric (0.866 at
  halfway; there is a test pinning it). A prettified preview would misrepresent them.

**Real work:**

- The stop strip itself. `UI-DESIGN.md` §8 already specifies it: "conventional (Photoshop/Inkscape-
  style stop strip): draggable stops, double-click to edit color, right-click to delete", and calls
  it "the one piece worth designing carefully — the most-touched custom surface".
- ⚠⚠**No harness drives hover, drag or scroll** (recorded repeatedly). A drag-based stop strip is
  therefore *untestable by machine* and rests on the author's eye. What can be tested is the model
  underneath: hit-testing, clamping, ordering, insert/delete, and the `to_stops`/`from_stops`
  round trip. Split the widget so those are pure functions with tests, and only the pointer handling
  is untested.

## 4. Traps, each one grounded

1. ⭐⭐**HSV from a grey sweeps the whole wheel, and the middle is a colour neither endpoint
   contains.** `rgb_to_hsv` returns hue 0 for any unsaturated colour, and `Space::HsvCcw` with equal
   hues takes a *full* turn (`1.0 − lh + rh` = 1.0). ✅Measured — a **black → red** segment:

   | t | 0.00 | 0.25 | 0.50 | 0.75 | 1.00 |
   |---|---|---|---|---|---|
   | rgb | 0.00 0.00 0.00 | 0.22 **0.25** 0.19 | 0.25 **0.50 0.50** | 0.47 0.19 **0.75** | 1.00 0.00 0.00 |

   21 of 65 samples are green-dominant. That is either a beautiful effect or a baffling one, and the
   user cannot predict it from the two swatches they picked. **The editor must show the consequence**
   (the preview does this for free) and should probably say "hue of a grey is undefined — this
   sweeps from red" when an endpoint has zero saturation.

2. ⚠**Blend and space numbers are a FILE FORMAT.** They are GIMP's `.ggr` numbering *and* they are
   written into the user's session (`PaletteSegment::blend/space`). New curves must **append**;
   renumbering silently re-interprets every saved gradient. There is a test pinning both directions.

3. ⭐**Everything here is DISPLAY-referred**, measured in beta.21 and re-stated in three doc
   comments. Curves therefore shape *gamma-space* values. That is consistent with the renderer, and
   it is exactly where §6's OkLab option gets expensive.

4. ⚠**`to_stops` is lossy for rich segments** — already documented, and the editor already has a
   "Convert to editable stops" button that states what it discards. If §2's recommendation is taken,
   that button becomes the *legacy* path rather than the normal one.

5. ⭐**LUT error is curvature-dependent.** The bake's acceptance criterion — error must *shrink* as
   the LUT grows — was measured on piecewise-linear gradients (16,372 → 1,496 differing pixels for
   1024 → 4096). A curved segment has more curvature between samples, so ⚠the same measurement must
   be repeated on a curve-heavy gradient before assuming 1024 is still enough. The harness for this
   exists.

## 5. Zero drift is achievable, and that is worth stating

The last two palette changes each moved pixels and cost a re-bless conversation. **This one need
not.** `Blend::Linear` + `Space::Rgb` + centred midpoint is the default, and it is exactly what
`from_stops` already produces — so every existing preset, session and import renders bit-identically
unless a user deliberately sets a curve. That should be an explicit acceptance criterion:

> `--selftest` 173/173 + goldens 18/18 and the F3 corpus 38/38 **unchanged**, with no re-bless.
> If anything drifts, a default moved, and that is a bug rather than a bless.

## 6. Beyond GIMP's five: what "parametric" could mean

The five named curves plus a midpoint already cover most of what gradient tools offer. If you want
genuinely parametric curves, the options rank like this:

| option | cost | expressive gain | interchange |
|---|---|---|---|
| **(a) named curves + midpoint** | **done** | covers most cases | `.ggr` exact both ways |
| **(b) power / gamma, one `f32`** (`f^γ`) | ~10 lines + UI slider | large — continuous control over easing | ⚠**ours only**; `.ggr` export must approximate |
| **(c) cubic-Bezier easing, 4 `f32`** (the CSS model) | moderate; a familiar 2-handle widget | largest; arbitrary ease-in/out | ⚠ours only |
| **(d) per-channel curves** (3× the params) | high, UI included | small *here* | ⚠ours only |

⭐**(b) is the best value by a distance.** One float, a slider, an obvious mental model, and it
subsumes "curved" as a special case. (c) is the right answer if the goal is a curve *editor* rather
than a curve *parameter* — but note that at high `cycle` the palette repeats many times across a
narrow escape band, so **curve shape matters far less than palette position resolution** (§5a of
`palette-import.md`). Fine easing control is a shallow-view aesthetic feature; it will be invisible
on the deep views this renderer exists for.

⛔**(d) is not worth it here** for that same reason.

**The interchange cost is the real decision.** The model is currently a strict superset of nothing —
it *is* GIMP's, exactly — which is why `.ggr` import is lossless and export would be trivial. Adding
(b) or (c) breaks that symmetry: our gradients stop being expressible in the one format everyone
else reads. That is a defensible trade, but it should be made deliberately and written down, and
`.ggr` export should then say "approximated" rather than silently flattening.

### OkLab / OkLCh — worth raising, not worth doing yet

The modern answer to "gradients look uneven" is a perceptual space. It would genuinely help fractal
palettes, where a linear RGB ramp bunches perceived lightness. ⚠But our colours are display-referred
by deliberate choice, so an OkLab segment needs a real sRGB **decode → OkLab → encode** round trip
per sample — the first place the display-referred decision actually costs something, and a direct
contradiction of the "palette interpolation happens in gamma space, by design" comment in
`fractadyne-export`. It is a `Space` variant like any other (the enum already dispatches per
segment, and baking hides the cost), so it is *implementable* cheaply — but it deserves its own
decision, not a quiet third entry in a dropdown.

## 7. Recommended order

1. **Make the editor segment-native** (§2), with a test that an old stop-only session renders
   byte-identically. Nothing user-visible yet; this is the migration, done on its own so a
   regression here is unambiguous.
2. **Expose what already exists**: blend picker, space picker, midpoint control, per-segment curve
   preview. This is the actual feature, and it should ship with zero pixel drift (§5).
3. **`.ggr` export**, and a round-trip property test against import.
4. **Then** decide on (b)/(c) and OkLab, on evidence — including whether curve shape is even visible
   at the depths that matter.

⚠Steps 1 and 2 are separable and 1 is the risky one. Do not combine them.

---

## 8. DECIDED (user, 2026-09-05) — the plan

**Take the full segment-native editor**, and go beyond GIMP's five. The user's brief: *"the most
flexibility while still being manageable"*.

### The shape of the answer: parameters, not names

⭐⭐**A registry of 27 extra named curves is not flexibility, it is a worse editor.** Nobody can
tell "sine" from "sphere-decreasing" in a dropdown, and each new name is another thing to
document, persist, export and test. Real flexibility per segment comes from a CONTINUOUS parameter:
one family with a knob covers infinitely many shapes and is one widget.

So the 32 is **numbering space, deliberately under-populated**:

| blend kind | owner | notes |
|---|---|---|
| **0–4** | GIMP | linear, curved, sine, sphere-inc, sphere-dec. ⛔**FROZEN** — a file format, in `.ggr` AND in our sessions |
| **5** | ours | **cubic-Bézier easing**, 4 params. The parametric one |
| **6–31** | reserved | empty on purpose. Add only on evidence that Bézier cannot express something wanted |

⭐**Why Bézier and not power/gamma**, having recommended gamma in §6: gamma is one float and cannot
make an S-curve, which is the shape most often wanted ("linger at both ends"). Bézier is four
floats, subsumes gamma closely, adds ease-in-out and overshoot, and — decisively — has an
off-the-shelf mental model and widget (CSS `cubic-bezier`, every animation tool). The extra three
floats cost nothing per segment.
⭐**The simple knob is a UI affordance over it, not a second storage shape**: a single "bias" slider
drives symmetric handles for the common case, with presets (ease-in / ease-out / ease-in-out) that
just write control points. One thing stored, two ways to drive it.

### Storage

```rust
// fractadyne_state::PaletteSegment gains, both #[serde(default)]:
blend: u8,                 // KIND (existing) - 0..4 GIMP, 5 bezier, 6..31 reserved
blend_params: [f32; 4],    // bezier control points; ignored by kinds 0..4
```

⚠`Blend` in `fractadyne-color` gains a payload variant (`Bezier([f32; 4])`); `from_u8`/`as_u8` keep
carrying the KIND only, and the params travel beside them. That keeps the existing round-trip test
honest and keeps `.ggr` import (which only ever produces 0–4) untouched.

### Phases — in this order, and P1 alone first

**P1. Segment-native model.** `custom_segments` becomes the source of truth for a custom gradient;
stops become a derived view. `custom_palette` is retained for legacy sessions, `.map` bands and the
paste box, all of which convert on load (`from_stops` / `from_bands`). The editor still renders the
same rows it does today, derived — **nothing user-visible changes**.
⚠**This is the risky phase and it ships alone.** Acceptance: an old stop-only session renders
**byte-identically**, plus selftest 173/173 + 18/18 and corpus 38/38 with **no re-bless**. If
anything drifts, a default moved and that is a bug.

**P2. Expose what already exists.** Per-segment blend picker, space picker, midpoint control, and a
curve-preview sparkline. This is the feature; the engine is untouched. ⚠Draw the REAL blend
function — the two sphere curves are not midpoint-symmetric (0.866 at halfway, pinned by a test).
Zero drift again: `Linear`/`Rgb`/centred is the default and is exactly what `from_stops` produces.

**P3. The stop strip** (`UI-DESIGN.md` §8: draggable stops, double-click to edit colour, right-click
to delete). **Cap 32 hand-editable stops** (up from `EDITOR_MAX_STOPS = 24`); above that the
existing summary fallback stands, because an imported `.map` is 256 entries.
⚠⚠**No harness drives hover, drag or scroll** — so split the widget: hit-testing, clamping,
ordering, insert/delete and the stops↔segments round trip are PURE functions with tests, and only
the pointer plumbing rests on the eye. The window needs widening (~340 → ~520 px); 32 stops on a
520 px strip is ~16 px apart, which needs click-to-select plus a numeric position field and
arrow-key nudge to be usable.

**P4. The Bézier curve.** Kind 5, the 2-handle widget, the bias slider, the presets.
⚠**Re-run the LUT acceptance measurement on a CURVE-HEAVY gradient**: §4 trap 5 — the "error must
shrink as the LUT grows" evidence (16,372 → 1,496 differing pixels, 1024 → 4096) was taken on
piecewise-LINEAR gradients, and a curve has more curvature between samples. The harness exists.
This is the first phase that may move pixels, and only where a user sets a curve.

**P5. `.ggr` export** + a round-trip property test against import. ⚠Kinds ≥ 5 are **approximated
and must say so** — a silent flatten is the failure this whole design exists to avoid.

### Deferred, with reasons

- **OkLab / OkLCh**: implementable cheaply as another `Space`, but it needs a real sRGB
  decode/encode round trip and is the first place the display-referred decision costs something.
  Its own decision, not a quiet fourth entry in a dropdown.
- **Per-channel curves**: declined (§6) — 3× the parameters and UI for a gain that `cycle` erases.
- **Alpha**: parsed and stored already; the renderer has no alpha channel. Out of scope.

---

## 9. The editor redesign (user, 2026-09-05): "the current UI is difficult to use"

Written after the user reviewed the shipped P2 editor and asked for *"a spline-like editor with
control points and handles to adjust the curve continuously"*. ✅The findings below were read out of
`main.rs:7018-7326` against a screenshot of the running beta.29 editor.

### 9.1 What is actually wrong — five structural faults, not cosmetics

1. ⭐⭐**Two lists the user has to align by counting.** N stop rows, then N−1 curve rows, with
   nothing connecting row *i* of the curves to the pair of stops it sits between: no shared axis, no
   label, no highlight. At the current `EDITOR_MAX_STOPS = 24` this is not readable at all, and P3
   was about to raise it to 32.
2. ⭐**The gradient bar is inert.** It is the only surface where position is spatially true, and it
   is the one thing that cannot be touched. Position is instead edited by full-width `0..1` sliders
   whose handles do **not** line up with the bar 40 px above them — 0.620 and 0.820 are adjacent in
   the strip and ~150 px apart in the slider column. That is a 1-D control for a thing the user is
   already looking at in 2-D, and it consumes the window's whole width per row.
3. ⭐**The curve preview is 30×16 px.** It exists precisely to compensate for names nobody can
   decode ("Sine" vs "Sphere ↑"). At that size it cannot — so *neither* the name nor the picture
   communicates, and the control that carries the feature is the least legible thing in the window.
4. **No selection concept**, so every parameter of every segment is resident: 6 segments × 3 widgets
   = 18 controls competing at once, growing linearly with stop count.
5. **Nothing is direct manipulation.** Every edit in the window is a dropdown or a drag-value.

⭐**The unifying diagnosis: there is no selection, and because there is no selection everything must
be on screen, and because everything is on screen nothing can be big enough to read.** P3 (the stop
strip) and P4 (the Bézier curve) were planned as separate phases, but both need selection and both
would otherwise invent their own. **They share one layout decision, so §8's P3/P4 split is revised
below.**

### 9.2 Three readings of "spline editor" — they are genuinely different features

| | what x/y mean | changes | interchange |
|---|---|---|---|
| **A. Per-segment ease** (CSS `cubic-bezier`) | x = position *within* the selected segment, y = blend factor 0→1 | the **timing** along a segment | `.ggr` approximates (§8 kind 5) |
| **B. Whole-gradient graph editor** | x = position `0..1` across the gradient, each segment a cell rising 0→1 | same data as A, all segments at once | same as A |
| **C. Per-channel R/G/B splines** (Photoshop *Curves*) | x = position, y = channel value | the **colour path** itself | ⛔breaks everything |

⭐**A and B are the same model with two viewports**, which is why the answer is not to choose. A is
the only one big enough to grab a handle in; B is the only one that shows the gradient as one
picture and shares the strip's x-axis. B alone fails on arithmetic: 32 segments across the planned
520 px window is **16 px per cell**, and a 16 px cell cannot host two draggable handles.

⛔**C is declined, again and for a new reason.** §6(d) declined per-channel curves because `cycle`
erases the gain. The stronger objection is structural: C makes free control points the primary
object and **stops stop being the model**, so `.ggr` / `.map` / `.ugr` / `.ase` import → edit →
export round-trips all break, and every importer in the tree targets the segment model. ⭐It survives
as a **read-only feedback plot** ("what did my ease do to R/G/B?"), which costs nothing because it
reads the same bake the preview already builds.

### 9.3 DECIDED — the A+B hybrid layout

```
┌────────────────────────────────────────────┐
│ ███████████ gradient preview ██████████████ │  the bake, unchanged
│  ●    ●   ●    ●     ●      ●         ●    │  P3: draggable stop markers
├────────────────────────────────────────────┤
│ ╱ │ ╭─ │ ╱ │ ╭─│ ╱  │  ╱   │   ← B: ribbon │  one cell per segment,
│   │[==selected==]│    │      │     click    │  real factor curve, click-select
├────────────────────────────────────────────┤
│  segment 3 · 0.275 → 0.400                 │
│  ┌──────────────────┐  ■ colour            │  A: the editing surface
│  │             ○──╮ │  pos  0.275          │  ~200 px square,
│  │          ╭──╯   │  space  RGB     ▾     │  two handles off pinned
│  │      ╭──╯       │                       │  (0,0) and (1,1)
│  │  ╭──╯   ○       │  bias ──●───          │
│  │╭─╯               │  ease-in-out    ▾    │
│  └──────────────────┘                      │
└────────────────────────────────────────────┘
```

⭐**The three rows share one x-axis** (rows 1 and 2 exactly; row 3 is the selected cell magnified),
which is fault 2 and fault 1 fixed by construction rather than by a label. Selection replaces the
18-widget wall with one segment's controls at a legible size — fault 3 and 4. The strip, the ribbon
and the canvas are all drag surfaces — fault 5.

The stop list collapses from N rows to **the selected stop's** colour swatch + numeric position +
delete. ⚠Keep the numeric field: 32 stops on a 520 px strip is ~16 px apart, so click-to-select plus
a typed position and arrow-key nudge is what makes the strip usable, not the drag alone (§8 P3
already said this).

⚠The import buttons (`.map` / `.ugr` / `.ggr` / `.ase`) currently force the window's width with six
buttons in one row. They are file tasks, not editing tasks — collapse them behind a single
**Import ▾** menu button, as `Copy preset…` already is.

### 9.4 Five decisions this forces, each with a wrong answer

1. ⭐⭐**Midpoint and Bézier are redundant, and must not compose.** `mid` pre-warps `linear_factor`
   *before* every blend (`Segment::factor`). If kind 5 applied on top of that warp there would be two
   knobs for one shape and the handles would lie about where the curve goes. **DECIDED: kind 5
   ignores `mid`**, the canvas hides the midpoint marker for it, and the segment's stored `mid` is
   preserved untouched so switching back to kinds 0–4 restores it. ⚠**Pin this with a test** — `mid`
   still persists in `PaletteSegment`, so "ignored" is a claim about the evaluator, not the file.
2. ⭐**Kinds 0–4 are not Béziers.** Sine and the two spheres cannot be expressed exactly by a cubic
   ease (the spheres read 0.866 at halfway — there is a test). So on a kind 0–4 segment the canvas
   draws the **real** curve via `Segment::factor` with **handles hidden**, plus a *"Convert to
   editable curve"* button that fits handles approximately and says so. ⭐This is deliberately the
   same idiom as the existing "Convert to editable stops": a lossy conversion is a button the user
   presses, never a surprise on first drag.
3. ⭐**An interior control point on the ease curve is NOT a stop, and two handles is the cap.** An
   ease point re-times travel along the segment; a stop bends the colour path. They are different
   operations and both are wanted — but a 2-handle cubic already covers arbitrary monotone
   re-timing, so adding interior ease points would create a second way to express something the
   split-segment button already does better. **Two handles, endpoints pinned at (0,0) and (1,1).**
4. ⚠**Overshoot.** x must stay in `[0,1]` or the curve is not a function — clamp it. y outside
   `[0,1]` extrapolates past the endpoint colours, which is often exactly the wanted effect
   (anticipation / overshoot) and can go out of gamut. **Allow y overshoot, clamp at the COLOUR, not
   at the factor** — clamping the factor would silently flatten the handle the user is dragging.
5. ⚠**Evaluation cost is a bake-time question only.** A cubic-Bézier ease needs `x(u) = t` solved
   per sample (Newton with a bisection fallback, as CSS does). That runs 1024 times per bake and
   **zero times per pixel** — the same argument that made every other feature here free. ⛔Do not
   put a solver in the shader.

### 9.5 What is testable and what rests on the author's eye

⚠⚠**No harness drives hover, drag or scroll** — recorded repeatedly, and it is the reason to split
the widget rather than a reason to skip tests:

- **Pure, tested:** hit-testing (which stop / which segment is under x), clamping and ordering on
  drag, insert/delete, the stops↔segments round trip, the Bézier solver (monotonicity, endpoints,
  `x(0)=0`, `x(1)=1`, agreement with a reference at sampled points), the kind-0–4 → Bézier fit's
  reported error, and decision 1's "kind 5 ignores `mid`".
- **Eye only:** pointer plumbing and layout. ⭐The ribbon and the canvas both draw through
  `Segment::factor`, so a drift between preview and render is impossible by construction — that is
  one class of bug that needs no test because it has no code path.

### 9.6 Revised phases

§8's P3 and P4 **merge at the layout level and stay separate at the model level**:

- ✅**P3′ — the layout, no new model. SHIPPED beta.30.** Draggable stop strip, the segment ribbon,
  selection, the selected-segment canvas drawing kinds 0–4 read-only with a draggable midpoint ring
  and a strip of the segment's own colours, the collapsed stop row (swatch + typed position + nudge
  + remove), `Import ▾`, cap 32, width capped at 496 pt. ⭐**Zero drift measured**: `--selftest`
  **173/173 + 18/18**, corpus **38/38 maxD 0**, no re-bless. Pure geometry in `gradient_strip.rs`
  (8 tests, **all five helpers verified RED by mutation**).

  ⭐⭐**Three things only the screenshot found**, and all three had been live since P1:
  1. The **"Imported gradient … Convert to editable stops"** notice keyed off *has segments*, which
     meant "came from a `.ggr`" only until P1 made every custom gradient have segments. It was
     offering to convert a preset copy into what it already was. Now gated on
     `Gradient::is_stop_expressible`, which asks the question that matters — does any segment carry
     a non-linear blend, a hue sweep or an off-centre midpoint.
  2. The stop-count line still said positions "may overlap; they're sorted automatically" —
     true of the flat list the editor owned before P1, false of segment boundaries.
  3. The **window's width was set by whichever label was longest**, because a label inside
     `ui.horizontal` never wraps. Every copy edit silently resized the editor.

  ⚠**And the harness was lying by omission.** `--uitest` has a `Screen::PaletteEditor` step, so the
  repeated note that "no harness opens the gradient editor" was wrong — but it seeded **no custom
  gradient**, so it screenshotted the *empty state* and passed. The one screenshot meant to review
  this surface never contained it. It now seeds a preset with varied per-segment curves.
  ⚠It also found a **stowaway**: `uitest_close_all` never closed the Diagnostics window, which had
  been standing behind every screenshot from step 15 on, invisible only because the window in front
  happened to be wide enough to cover it.
- ✅**P3″ — what using it asked for. SHIPPED beta.31.** The author ran the beta.30 editor and came
  back with four things; all four are in, plus the bug report that came with them.

  ⭐⭐**"The drag points sometimes hang up where they stop moving" — DIAGNOSED, and the first
  diagnosis was wrong.** `Response::drag_started()` does not fire until the pointer has passed
  egui's drag threshold, and the editor hit-tested `interact_pointer_pos()` *at that moment* — a
  point already several pixels from the press. The fix is to hit-test the **press origin**. ⚠The
  first write-up blamed the 6 px threshold itself; the test's own guard assertion failed and showed
  that 6 px is *not* enough to leave a 9 px catch radius at 15.8 px spacing. **The threshold is a
  LOWER bound, not the drift** — the drift is however far a flick travelled in the frame that
  crossed it, so it is unbounded. ⭐And the silent-cancel case needs *few* stops, not many: at the
  32-stop cap the catch zones overlap everywhere, so a drift always grabs *something* (merely the
  wrong thing), whereas on the 7-stop gradient in the report most of the strip is dead band and the
  drag is cancelled with no feedback at all. Both regimes are pinned in `gradient_strip.rs`.

  - **Ring view.** ⭐The justification is not decoration: a palette is *cycled*, so it is
    topologically a circle, and the ring is the only view in which the **seam** exists. On a bar the
    two ends are as far apart as they can be; in the render they are adjacent, so a hard edge there
    is invisible in the editor that made it. Hit-testing is by **arc length**, not by angle, so the
    catch zone stays the same physical size as the bar's — and it **wraps**, or the stop at the seam
    becomes unclickable from one side. The annulus is drawn through the same `Lut` as the bar.
  - **An add lane above the bar** whose *line* is the click target ("click where you want it"),
    with a `⊕` at the end for the widest gap — the one thing a button can decide. The old
    double-click-on-bare-strip stays as the fast route.
  - **A `⊖` under the selected marker**, tracking it. ⚠Interior stops only: the two ends are pinned
    by contract, so a `⊖` there would be an offer the editor cannot honour.
  - **Save / Cancel over a named library** (`gradients.toml`, beside `bookmarks.toml`).
    ⭐⭐**Stored as SEGMENTS, never stops** — a stop-list library would reload, render, and look
    right while having dropped every midpoint, curve and hue sweep, which are the only properties
    that make a gradient worth saving. `gradient_library.rs` pins the round trip **through the bake**
    (what the GPU fetches, so a field that survives the file but is misread on the way back still
    fails) and carries a **control** proving that storing stops instead would have changed the
    picture. ⚠**Cancel reverts; the window's ✕ does not** — closing a window is not a statement
    about the work in it, and every edit here is already live in the view.

  ⭐Zero drift again: selftest 173/173 + 18/18, corpus 38/38 maxD 0. Tests app 254 → 260.

- **P4′ — kind 5.** `blend_params: [f32; 4]` on `PaletteSegment` (`#[serde(default)]`), the
  `Blend::Bezier([f32;4])` payload variant, the handles going live, the bias slider and presets.
  ⚠First phase that may move pixels, and only where a user sets a curve.
  ⚠**Re-run the LUT acceptance measurement on a curve-heavy gradient** (§4 trap 5) — the "error must
  shrink as the LUT grows" evidence was taken on piecewise-**linear** gradients.
- **P5 — `.ggr` export**, unchanged, with kinds ≥ 5 marked approximated.

⭐The read-only per-channel plot from §9.2 is a **P6 candidate, not scope** — it is the honest
version of option C and costs one extra canvas over the existing bake.
