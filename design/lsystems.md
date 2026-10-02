# L-systems — turtle-drawn curves and plants, with deep zoom

Status: **phases 0–4 built** (2026-10-02, branch `feat/lsystems`) — phase 4: filled shapes,
stochastic, parametric and context-sensitive productions, SVG export, draw-on and angle animation
(the user: "run the radeon tests and do phase 4"). Still to do: raster export of an L-system view,
and tours with order, angle and draw-on tracks (they render through it). The user:
"Work on the design doc" (after "Are L-systems implemented" — no). Integration facts in §2 are from
this tree at `21e2bd7` (Life merged). The user's decisions on the open questions are in §9 (all six:
yes).

What building them settled, beyond the text below:

- **The world step per order** is the reciprocal of a *measure* — the displacement of the reached
  symbol that travels furthest at the deepest row — turned to that symbol's direction there, so the
  picture neither grows nor spins (the dragon turns 45° an order). Where the measure is zero or near
  it at a low order (the dragon's `X` draws nothing at order 0), the growth law sizes the step.
- **Period 2**: a picture that alternates between two shapes from order to order (the arrowhead
  mirrors; Bourke's weed sways 8%) is detected from its bounding boxes, and the order steps by two.
- **Overlap cap**: segments per square step of the picture's bounding box sit near 1 for every
  plane-filling curve at every order, and climb for a curve that overlaps itself (Tiles: 1.6, 3.1,
  12, 46 at orders 1, 3, 7, 11). The order that follows the zoom stops at 3 — past it the picture is
  a solid blob, and more order is cost only.
- **The step target is 3 px (or 2.5 line widths)**, not 1.5: at a step no wider than its lines, a
  space-filling curve draws as a solid block.
- **The walk runs off the UI thread**, one at a time; a frame draws the last walk placed under the
  current view (scale and offset), so a zoom follows at the walk's rate, never blank.
- **Not in phase 2 after all**: image export (refused with a message, as for Life) and tours with an
  order or angle track.

Phase 3 (unlimited zoom) settled:

- **The tables lose precision with depth, and how fast is measurable.** A subtree's displacement is
  a sum of its children's that cancel: the Sierpinski triangle's `F` advances 2 steps with 5 steps
  of children, so rounding grows ~×2.2 a level (1.1 bits). Per system, `loss` = the most any
  production's children reach over what it reaches (0 when every turn is a quarter turn and there
  are no step factors: then the sums are of integers). At the deepest row the triangle's tables
  were noise — its orientation came out turned 90° and its growth ×3 for ×2 — so the measure,
  orientation and growth are taken at a moderate depth (8–64, ≥ 22 good bits), exactly
  (`BigFloat`), and the growth law carries the step beyond it.
- **Headings are whole numbers in the deep tables too.** Every angle written (a division, a decimal
  such as 25.7° = 257/3,600 of a turn, a `\a`, the heading, `|`) is a rational fraction of a turn,
  so a heading is an integer count of their common unit and a subtree's turn an integer sum; its
  unit vector is computed from the count, exact for quarter turns. The first version multiplied
  unit vectors, which carries rounding four-fold into every level above (the Koch curve): by depth
  415 its headings were (0, 0).
- **The deep walk**: `BigFloat` while a subtree reaches more than `2^S` px, then the `f64` walk,
  where `S = 40 / (1 + loss / log₂ growth)` (31.7 for Koch, ~19 for the triangle) keeps `f64`'s
  error under 2⁻¹² px; precision = the view's bits + order × loss + 96. The app walks deep when the
  picture is larger than `2^S` px or the order is past the `f64` tables.
- **A cull must allow for its own rounding.** In `f64` at 2⁷³ px the disc test rounds by hundreds
  of pixels; near a subtree's far end its reach is exactly the distance to the view, and the
  subtree holding the view was culled (the Koch curve's end at 3⁴⁰). The deep test keeps a 1e-9
  relative slack.
- **Self-similarity about the START proves nothing about precision**: coordinates near 0 are tiny
  and any floating precision holds them (that test passed with the tables cut to 64 bits). About
  the END, (1, 0), they are 1 − 3^-k: that test fails at 64 bits and passes at the computed
  precision, at 3⁴⁰ and 3²⁰⁰ (and 3¹⁰⁰ in the self-test).

Phase 4, filled shapes, settled:

- **A polygon is the turtle's path between `{` and `}`**, closed in the word that opens it; inside,
  a step adds a vertex and draws no line, and `.` adds one without moving. A `{` inside an open
  polygon is an error, placed like any other.
- **The tables carry two reaches**: the lines' (for culling lines), and every point a subtree
  visits, drawn or not (a polygon's vertices are its moves). The work count includes vertices, so a
  picture of shapes alone frames, and the budget counts them.
- **A subtree inside a polygon that is off the view or under a pixel gives its end point**: a
  chord of the outline, as a line under a pixel is drawn as one segment. The polygon is clipped to
  the view (plus the line margin) before it is cut into triangles: zoomed into the filled
  snowflake's edge, its polygon reaches 1e30 px past the view. Triangles are drawn under the lines.
- **The overlap cap reads each order's own bounding box.** Measured by one order's box, a plant
  whose leaves keep their size while its stem doubles was "dense" at every low order (its box at
  order 13 is a line): it was held at order 0, which draws nothing.
- **A system drawn at a fixed order frames at that order** (Bourke's mango leaf at order 18 is a
  corner of itself at order 300), and a home view the user has not moved is framed again when the
  canvas changes size (the first layout after a file opens the app).
- The status bar's count is everything drawn (segments and filled shapes); the panel says which.

Phase 4, stochastic productions, settled:

- **Syntax**: one line per alternative with its weight, `X (0.33) = word` (ABOP's `X →(.33)`);
  weights are relative; a symbol's lines are all weighted or it has one; `seed n`. `|` was not
  available as a separator — it is Fractint's turn-around.
- **The choice does not depend on the depth.** A node keeps its variant as the order rises (only its
  depth changes), so it keeps its production: order n + 1 is order n with one more level of detail.
  A choice by depth would redraw the plant every time the zoom gained an order.
- **A child's variant is a permutation of its parent's for each place in the word**
  (`perm[(v + shift[j]) mod K]`). A hash of (v, j) is a random map, and a line of descent through
  the same place — a stem — falls into a cycle of about √(πK/8) ≈ 5 variants under one: the stem
  would repeat itself every five levels.
- **K = 64.** The `f64` tables are rows × symbols × K entries (≈ 18 ms for the stochastic plant,
  depth 429); the exact measure (`BigFloat`) is a row per variant, so a stochastic system takes it
  at depth 24 at most, where thousands of leaves already average its growth (×2.41 for seed 1,
  ×2.29 for seed 2); the deep tables at 1,100 bits to order 300 build in ~160 ms. A weight under
  1/64 may never be drawn.
- **Measured in the variant the axiom gives it**: the picture's measure is its own first subtree,
  so it sits still as the order rises, as a deterministic one does.
- Checked: the walk, the culled walk and the deep walk against a reference that rewrites the whole
  word a generation at a time carrying each symbol's variant (four stochastic exercises: a plant,
  a curve, a dragon whose `X` folds either way, leaves on either side); two mutations of the
  walk's variant bookkeeping each fail those tests.
- **Seen**: branch-depth colouring is depth over (order × nesting), so at a deep zoom every visible
  branch is a dark shade — for every plant, not only stochastic ones. To revisit.

Phase 4, parametric and context-sensitive productions, settled (`lsystem/expand.rs`, `expr.rs`):

- **One engine for both: build the word.** As §9.2 said, neither keeps the tables, so these are
  drawn at a fixed order (the system's `order`, or the toolbar's) by rewriting the whole word a
  generation at a time — up to two million modules (the order is lowered to the last that fits,
  and the panel says so) — and running the turtle over it in `f64`, culled segment by segment.
  The memoised parametric tables of §9.2 were not built: the engine covers every system, and the
  common geometric ones are no slower to look at this way at their orders.
- **ABOP's semantics, from its text** (chapter 1, pdftotext of algorithmicbotany.org's PDF): a
  production matches a module with its letter, as many parameters as it names, its contexts and a
  true condition; the first in the order written applies; an unmatched module stays. In a
  bracketed word the left context is the module before on the path to the root (a branch to the
  left stepped over, the branch the module is in stepped out of), the right context the module
  after on its branch (branches stepped over; none at a branch's end); `ignore` letters are
  unseen. `*` is any context. Contexts may hold several modules but no brackets.
- **Syntax**: `left < A(x, y) > right : condition = successor`, `define NAME expression`,
  `ignore +-F` (ABOP's `#define`, `#ignore:` and `→` are read too). A condition compares with
  `==` (a bare `=` would be the production's), unless the production uses `→`. Expressions:
  `+ - * / % ^`, comparisons, `! & |`, and functions; trigonometry in degrees. Turtle commands take
  arguments: `F(l)` steps l, `+(a)` turns a degrees, `@(f)` scales the step, `\(a)` and `/(a)`.
  `A(2) = …` alone is a stochastic weight, as before; weights cannot be combined with parameters.
- **Checked against the book**: equation 1.7's derivation as Figure 1.34 prints it; §1.10.2's
  context production on `A(4)B(5)C(6)`; the signals of Figure 1.30 up and down a branching
  structure; Hogeweg and Hesper's plant (Figure 1.31a) for five generations worked by hand; the row
  of trees ends where its base does at every order; equations 1.9 and 1.10 agree to R^(n−1),
  segment for segment. The self-test checks three of these words.
- **Library**: Hogeweg–Hesper plants 1.31a and 1.31d, the row of trees (1.37b), the branching
  pattern (1.39). ABOP's turtle starts facing up: the plants say `heading 90` (the first shots grew
  sideways).

Phase 4, SVG export, settled (`lsystem_view/svg.rs`):

- **The screen's walk, written as vectors**: the view's segments (culled, sub-pixel subtrees as
  chords — so the file is bounded by the view's pixels at any zoom, deep views included) as
  `<path>`s, consecutive segments that meet in the same colour joined into one; filled shapes as
  `<polygon>`s of their clipped outlines (triangles would show seams), under the lines; the
  interior colour as the background. Coordinates to 0.01 px, y flipped.
- **Colours as the screen's**: the colour pass for an L-system is `palette(value + offset)` from
  the baked LUT (`Lut::sample` is its Rust twin), its channels the bytes shown.
- **Checked**: the file read back is the segment list (to 0.005 px, colour for colour); a view's
  SVG has a polygon per shape and a line per segment; Inkscape renders of the mango leaf, the
  stochastic plant and the filled snowflake (`--shot … --svg`) match the app's screenshots.
- File ▸ Export SVG… and the panel's SVG… button; `--shot LOC --svg FILE` for unattended runs.

Phase 4, animation, settled:

- **Draw on, on the GPU**: each segment carries where along the curve it starts and ends (0–1, its
  index over the walk's total) and each fill where its shape starts; a `progress` uniform hides
  what is past it and shortens the segment it falls in. So the curve draws itself at the frame
  rate with no walk (a slider, or play: 8 s end to end). The device check gains "drawn on to 60%"
  (texel-exact against the model); a shader that stops shortening fails it (16 texels).
- **Angle sweep**: play beside the angle slider sweeps δ at 4°/s between 1° and 179°, there and
  back. Each step rebuilds the tables, so every walk lands a step behind the angle: a walk is now
  kept unless the SYSTEM changed (`system_id`, new per system opened), not dropped when the tables
  did — the picture would otherwise blink out on every step.
- Neither is saved with a view; tours with order, angle and draw-on tracks wait for raster export
  (a tour renders through it).

## 1. Goal

L-systems (Lindenmayer systems) as a fractal class beside escape time and Life: a grammar rewrites a
string of symbols, and a turtle reads the result as drawing commands. The classics this must draw:

- **Curves**: Koch curve and snowflake, quadratic Koch island, Cesàro, Lévy C curve, Heighway dragon,
  terdragon, Sierpinski arrowhead and square, Hilbert, Moore, Peano, Gosper (flowsnake).
- **Plants and trees**: the bracketed systems of *The Algorithmic Beauty of Plants* (ABOP, figures
  1.24 a–f: weeds, bushes, the fractal plant), with branching `[` `]`.
- **Tilings**: Penrose (P3) and similar edge-rewriting systems.

What makes it Fractadyne rather than a plotter: **zoom without limit**. A Koch curve at 1e100× is a
Koch curve, drawn as finely as at 1×, at the same cost; the dragon's boundary at any depth is exact.
Unlike most L-system viewers, which draw a fixed number of iterations and blur on zoom, the order of
the curve follows the zoom (§4.4) and positions near the view are computed exactly (§4.6).

### Non-goals

- 3D turtles (ABOP's `& ^ \ /` pitch/roll), surfaces, and the plant-modelling extensions that need
  them.
- A general vector editor; IFS (Barnsley fern) and other grid-free fractals (separate designs; they
  reuse the class layer too).
- Porting Fractint's or any other program's code (§10).

## 2. What exists today

- **The class layer** (Life, `dc19add`): `FractalClass` on every `FractalSpec` row
  (`app/src/fractal.rs:81`), `FractalKind::AUTOMATA` for the pickers (`:385`), `is_escape_time()`
  (`:398`) gating the escape-time-only UI (Quality/Effects panels, minimap, status readouts). Ids
  outside the shader's range: `formula::CUSTOM` = 1000, `formula::LIFE` = 1100 (`core/src/lib.rs:151`).
- **A non-escape-time renderer's path into the picture**: `MandelbrotParams::life`
  (`gpu/src/lib.rs:2377`) carries a frame descriptor; `prepare()` runs that class's passes and keys
  the re-render on `IterKey.life` (`:207`); the display pass writes the **iteration texture**
  (main = (value, 0, 0, 1e30), interior < 0, aux = `AUX_NONE`), so `fs_color` colours it unchanged —
  palettes, cycling, the gradient editor, supersampled anti-aliasing. `LifeRenderer::update`
  (`gpu/src/life.rs:687`) is the model; the app side is `build_life_params`
  (`app/src/life_view.rs:341`), which skips the escape-time frame machinery entirely.
- **Overlays over the view**: `life_overlay` (`life_view.rs:831`) draws on the central painter after
  the paint callback — the place for an L-system's on-screen hints.
- **View files** carry a class's state (`format_version` 3 for Life, `export.rs:200`); the session
  keeps it; **image export** refuses non-escape-time views with a message (`export.rs:1313`, `:1860`).
- **The formula library and the `.frm` / `.par` importers** are the precedent for a library of
  standard systems, user-defined ones, and Fractint's `.l` files.
- No SVG or other vector writer exists (`fractadyne-export` writes PNG and EXR).
- `DESIGN.md` §4.1 planned "CPU grammar expansion → turtle graphics → line/vertex buffer", deep zoom
  "limited by float transform precision", and named the risk "L-system string explosion at high
  depth — stream geometry, on-the-fly expansion". This design takes both further: the string is never
  built (§4.3), and positions near the view are exact (§4.6).

## 3. The class

`FractalClass::LSystem`, one `FractalKind::LSystem` (id `formula::LSYSTEM` = 1200), listed under an
"L-systems" group in the pickers. The system shown — from the standard set, a file, or written by the
user — is the class's state (`FractadyneApp::lsystem`), as the universe is Life's.

- **Coordinates**: the app's `Viewport` (BigFloat centre, extended-range scale). The curve's turtle
  starts at the origin heading right (+x), with its first-order size normalised to fit the home view
  (§4.4); y up, as the complex plane, so a plant grows upward.
- **Gated off**: Julia, iterations, perturbation, the finders, the autopilot, the minimap (as Life).
- **Picture**: the class's own pass writes the iteration texture (§5), so everything downstream of the
  texture works unchanged.

## 4. Engine

### 4.1 Systems and the turtle

A system is a **bracketed D0L system** (deterministic, context-free) with turtle commands:

- an **alphabet** of single characters; an **axiom**; **productions** `X → word` (a symbol without one
  rewrites to itself);
- a **turn angle** δ, given in degrees or, Fractint's way, as a division of the circle (`angle 6` =
  60°);
- **turtle commands** (ABOP's conventions, which Fractint's mostly share):

| Symbol | Turtle |
|---|---|
| `F`, `D` | draw one step forward |
| `f`, `G`, `M` | move one step forward without drawing |
| `+` / `-` | turn left / right by δ |
| `\|` | turn 180° |
| `!` | swap the meanings of `+` and `-` (Fractint) |
| `[` / `]` | push / pop the turtle state (branching) |
| `@x` | multiply the step by x (`@I x` = 1/x, `@Q x` = √x; Fractint) |
| `\a` / `/a` | turn left / right by a degrees (Fractint) |
| `Cn`, `<n`, `>n` | set / raise / lower the colour index (Fractint) |
| anything else | no action (a variable that only rewrites) |

Which symbols draw is part of the system: `F` and `D` by default; a system may name others (ABOP's
edge-rewriting curves draw with `F_l` / `F_r`, written here as `L` / `R` declared as draw symbols).
Fractint's exact command semantics — case handling, what `|` does when the division is odd, the forms
of `@` — are taken from Fractint's documentation when the parser is written, and pinned by tests
against systems whose pictures are known.

**Extensions** (phase 4, all wanted — §9 says how each meets the walk): stochastic productions (`X → a (0.3) | b (0.7)`, seeded so a
picture reproduces), parametric productions (ABOP ch. 1.10: `F(l) → F(l/3) + …`), context-sensitive
productions, filled polygons (ABOP's `{` `}`, for leaves and a filled Koch snowflake).

### 4.2 Text format, and Fractint's `.l` files

A native format, line-based, parsed in core with positioned errors (the formula dialog's pattern):

```
# Koch snowflake
angle 60
axiom F--F--F
F = F+F--F+F
```

Keys: `angle` (degrees, or `angle /6` for a division), `axiom`, `draw` (extra draw symbols), `move`
(extra move symbols), productions `X = word`, `#` comments. **Fractint `.l` files open directly**
(File ▸ Open; the precedent of `.frm` and `.par`): a file holds named entries
`Name { Angle n  Axiom …  X=… }` with `;` comments; the entries are listed as the `.frm` importer lists
formulas, and an entry opens as a system.

### 4.3 Expansion without the string

The string after n rewrites has length ~sⁿ (Koch: 4ⁿ; at order 20, 10¹²). It is never built. The
engine walks the **derivation tree** depth first: a node is a symbol with d rewrites still to apply;
a node with d = 0 (or a symbol without a production) runs its turtle command. Memory is O(n × word
length).

For every symbol X and every remaining depth d (up to the deepest order in use, ~60–200), four
tables are precomputed once per system, in units of the final step length:

- **D_X(d)** — the net displacement of X's sub-path, a vector in the turtle's frame (heading 0);
- **T_X(d)** — its net turn (an integer count of δ, plus any free-angle turns);
- **S_X(d)** — its net step-scale factor (the product of `@` factors outside brackets; 1 for most);
- **R_X(d)** — a bound on how far its sub-path strays from its start;
- **N_X(d)** — how many segments it draws (an exact integer: u128, or BigInt past it).

Each is a recurrence over X's production word — a child's start offset is the sum of the earlier
children's displacements, each rotated by the turns before it, scaled by the scales before it, and
reset at a `]` — so a subtree's effect on everything after it is known **without expanding it**.
That is what lets the walk skip.

### 4.4 Order follows the zoom

Drawing order n with the step shrunk by the system's **growth factor** s per order (Koch 3, Hilbert 2,
dragon √2, Gosper √7; measured as the limit of R_axiom(d+1)/R_axiom(d), or given) keeps the picture's
size fixed while detail is added. The order is chosen from the zoom so the final step is about a pixel:
**zoom in by s and the order goes up by one** — the curve never turns into straight lines. A fixed
order (Fractint's "order") is an option, for the stage-by-stage pictures of a textbook.

### 4.5 Culling and level of detail: cost follows the screen, not the curve

During the walk, before descending into a subtree whose start is p and whose step is L:

- **Cull**: if the disc of radius R_X(d)·L about p misses the view, skip it — advance the turtle by
  D_X(d), T_X(d), S_X(d) and the segment index by N_X(d).
- **Level of detail**: if R_X(d)·L is under about a pixel, emit one segment — the subtree's chord, p
  to p + D_X(d)·L — instead of its N_X(d) segments.

So the segments emitted for a view are bounded by its pixels (times a small constant), whatever the
order: a Koch curve at order 60 seen whole costs what order 8 does. The walk is parallel by subtree
(scoped threads over the top levels, as the reference build uses), off the UI thread, with a segment
budget per frame.

### 4.6 Unlimited zoom: exact where it matters

At 1e100× the step is 10⁻¹⁰⁰ of the picture, far past f64. The walk keeps two precisions, the pattern
of the perturbation engine and of the digit automata's exact prefixes:

- **Subtree roots near the view** — the few hundred whose discs meet it, along the spine of the
  derivation from the axiom down — have their turtle positions in **BigFloat**, at the viewport's
  precision. A heading is an **integer** count of δ (plus free-angle turns, which make a system
  shallow-only), so its cosine and sine come from a table computed once in BigFloat; a root's position
  is its parent's plus a rotated, scaled table entry, exactly.
- **Below a root whose extent is a few thousand pixels**, the walk switches to f64 positions relative
  to that root, and emits segments in view-relative pixel coordinates (f32): root offset from the view
  centre (BigFloat → f64 pixels, once) plus the local f64 path.

The segment index runs the same way — exact integers for roots, local counts below — so colouring by
position along the curve (§5) stays meaningful at any depth.

## 5. Rendering

- **The segment pass**: the CPU walk fills an instance buffer (endpoints in texels, a colour value, a
  width); a render pass draws one quad per segment (a vertex shader widens each segment to its line
  width, round-capped) into the **iteration texture**, cleared to interior. Lines stay a fixed width
  on screen at every zoom (default ~1.5 px, adjustable). Supersampling and the colour pass's box filter
  anti-alias them, as they do escape-time edges.
- **The colour value** (main.r), by the user's choice: **position along the curve** (the segment
  index, cycled through the palette — the gradient along a dragon or a Hilbert curve), **branch depth**
  (bracket nesting: trunk to leaves), **heading** (an angle palette), or **Fractint's colour index**
  (`C`, `<`, `>`). Background = interior colour.
- **What changes it**: a new system, order, view, line width or colour choice. A changed view re-walks
  only the visible part; panning reuses the previous segments while the next walk finishes off-thread.
- **Safety**: one draw call of N instances, N within a per-frame budget (a few million); the walk's
  budget, not the GPU, bounds the work.

## 6. The app

- **Panel** ("L-system" section, in place of the escape-time controls): system (the library + user
  systems), Edit… (the text format in an editor with errors in place, as the formula dialog), order
  (Auto / fixed n), angle (a slider that overrides δ — plants sway as it moves), line width, colour by,
  segments drawn (a readout). File ▸ Open accepts `.l`; Save writes the native format.
- **Standard set**, entered from their published definitions: the curves, plants and tilings of §1.
  **Custom**: any system the parser accepts, named and saved in a user library beside the formulas
  (the formula library's model).
- **Toolbar**: an L-system has no Julia or dual view; the slot shows the order (− / +) and a "draw on"
  toggle (§7).
- **Status bar**: scale, order, drawn (segments and filled shapes) — fixed widths.

## 7. Persistence, tours, export, animation

- **View files** (`format_version` 4 for an L-system view): `fractal=L-system`, `lsystem=` (the text
  format on one escaped line, as `formula=`), `order=auto|n`, `angle=`, `line_width=`, `colour_by=`.
  **Session**: the same, kept whichever family is shown (as Life's `life`).
- **Tours**: keyframes gain `order`, `angle` and `progress` (the next item); the camera interpolates
  as now.
- **Draw-on animation**: a progress value hides segments past a fraction of the curve's index — the
  curve drawing itself, live and in tour videos. **Angle morphing** animates δ.
- **Image export**: the same pass at the export size (lines scaled to the export's pixels), the walk
  culled to the export's rectangle; tiled for very large exports.
- **SVG export** (§9): the visible segments as paths, coloured by the palette — a vector file for print
  or a plotter.

## 8. Phases and gates

| Phase | Content | Gate |
|---|---|---|
| **0** | Class layer: `FractalClass::LSystem`, `formula::LSYSTEM` = 1200, the pickers' group, gating, view files (format 4) and session keys | goldens byte-identical; self-test unchanged; an L-system view refused by name in an older build (format 4) |
| **1** | Core: parser (native + Fractint `.l`), the tables of §4.3, the culling / LOD walk, auto order, the standard set; a naive reference expander (builds the string, runs the turtle) | walk = reference segment for segment at small orders (every library system); R bounds hold; culled ⊇ reference ∩ view; LOD segment count ≤ C × pixels; facts (§10) |
| **2** | The app: segment pass into the iteration texture, colour modes, panel + editor + library + custom, files, session, tours, raster export | GPU coverage = a CPU quad rasteriser (top-left rule, no AA) on fixed views; goldens; uitest screen; Radeon |
| **3** | Unlimited zoom: BigFloat roots, exact headings, exact indices | self-similarity exact: Koch zoomed 3⁴⁰ about its start = Koch, dragon 2³⁰ about a fixed point; BigFloat roots = rational arithmetic for 90° systems; a 1e100× view at 1× cost |
| **4** | Per §9: stochastic (variants) / parametric / context-sensitive productions, filled polygons, SVG export, draw-on and angle animation | each against its definition and the reference expander; stochastic: the variant walk = a reference that expands the same hashed choices; SVG re-read = the segment list |

## 9. Decisions (the user's, 2026-10-02)

The user: "Yes to 1-6".

1. **Unlimited zoom**: yes → phase 3 (§4.6).
2. **Beyond bracketed D0L**: yes to all three → phase 4 (§4.1). The walk of §4.3–4.5 skips a
   subtree because its effect is a function of (symbol, depth) alone; each extension keeps that or
   says where it stops:
   - **Stochastic**: a node carries a **variant** v ∈ 0..K (K = 64, as built — see "Phase 4,
     stochastic, settled"); its production is chosen by a hash of (seed, symbol, v) — not the depth
     — and its k-th child's variant follows from (v, k). Everything
     about a node is then a function of (symbol, depth, v), so the tables are kept per variant
     (|alphabet| × depth × K entries) and the walk skips, culls and zooms as before. Two subtrees
     with the same (symbol, depth, v) are the same picture; with the variants mixed at every level
     that is not visible. The seed is part of the system, so a picture reproduces.
   - **Parametric**: the tables are memoised per (symbol, depth, parameter values) while the
     distinct values stay within a budget, which holds for the common geometric forms
     (`F(l) → F(l/3)…`); beyond the budget the system draws at a bounded order by plain expansion,
     and says so.
   - **Context-sensitive**: a node's rewrite depends on its neighbours in the string, so the tables
     do not apply: these draw at a bounded order by expanding the string, culled only at the
     segments, and say so (as free-angle systems do, §11).
3. **Fractint `.l` files**: yes, open directly (§4.2).
4. **Colouring**: not a yes/no question; taken as "offer all of them" (as automata's Hensel rules):
   position along the curve, branch depth, heading, Fractint's colour index, and plain. Default: a
   library entry names its own; otherwise the Fractint colour index for an `.l` entry that uses `C`,
   `<` or `>`, branch depth for a system with brackets, position along the curve for one without.
5. **SVG export**: yes → phase 4 (§7).
6. **Filled shapes**: yes → phase 4: ABOP's `{` `}` — the turtle's positions between the braces
   are a polygon, triangulated on the CPU and drawn into the iteration texture under the lines;
   culled with the subtree that draws it.

## 10. Validation

- **Exactness wherever it exists.** Segment lists are compared exactly (the walk vs the reference
  expander; the culled walk vs the reference clipped to the view); positions at depth exactly
  (BigFloat vs rational arithmetic for 90° systems). Only rasterised pixels compare against a model of
  the rasteriser.
- **Facts** (each from the curve's definition): Koch order n has 4ⁿ segments, the snowflake 3·4ⁿ
  (perimeter 3·(4/3)ⁿ times the first triangle's side); Gosper 7ⁿ; Peano (edge-rewriting form) 9ⁿ; the Hilbert
  curve of order n visits each cell of a 2ⁿ × 2ⁿ grid once, each step to a neighbour; the dragon of
  order n has 2ⁿ unit segments, no edge drawn twice, and its end at distance 2^(n/2) from its start;
  the Lévy C curve's endpoints are fixed.
- **Self-similarity** (phase 3): a curve zoomed by sᵏ about a fixed point is the curve again, segment
  for segment in view coordinates — the deep-zoom gate that cannot be fooled by a plausible picture.
- **Cross-class identities** (later): the Sierpinski arrowhead's vertices against the Sierpinski digit
  automaton (design/automata.md §4.4) on the same lattice.
- **Mutations** must go red: a wrong table entry, a missed `]` reset, a culling bound one segment short,
  an off-by-one order, a swapped `+`/`-`.
- **Radeon** at the end of phase 2 (a new render pass on both drivers).

## 11. Risks

- **Bounds that are too loose** make culling useless (everything "might" be visible); too tight, and
  segments go missing. R_X(d) is computed, not estimated, and the "culled ⊇ reference ∩ view" gate
  catches a bound that is too tight.
- **Growth that is not uniform**: auto order assumes the picture converges as the order rises. For a
  system where it does not (some plants grow lopsidedly), auto order falls back to a fixed order with a
  note.
- **Free angles** (`\a`, `/a`) and `@` with irrational factors break exact headings; such systems zoom
  to f64 depth only, and say so.
- **Overdraw** at shallow zoom of dense curves (a Peano curve fills its square): LOD keeps the segment
  count bounded; the picture is the curve's density, which is what it looks like.

## 12. Licensing and references

Fractint's `.l` format is documented in Fractint's help, and is implemented from that description,
not from Fractint's code. The standard set is entered from published definitions — ABOP is freely
available from the Algorithmic Botany site; Fractint's sample `fractint.l` is not bundled (users open
their own copy).

- P. Prusinkiewicz, A. Lindenmayer, *The Algorithmic Beauty of Plants* (Springer, 1990; free PDF from
  algorithmicbotany.org) — the turtle, bracketed, stochastic, parametric and context-sensitive systems.
- A. Lindenmayer, "Mathematical models for cellular interaction in development", J. Theor. Biol. 18
  (1968).
- Fractint documentation, "L-Systems" — the `.l` format and its turtle commands.
- C. Davis, D. Knuth, "Number representations and dragon curves", J. Recreational Math. 3 (1970).
