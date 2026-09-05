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
