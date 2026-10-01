# Automata — Life-like cellular automata and Sierpinski-type digit automata

Status: **design** (2026-10-01), nothing built. The user: "design it and plan on handling both things
like Life and Sierpinski". Background (what Life, elementary automata and digit automata are) is in
§1; integration facts in §2 are from a survey of this codebase (file:line as of `7861f20`).

## 1. Goal

Two kinds of picture the app cannot make today, sharing one new layer:

- **Life-like cellular automata.** A grid of cells evolving by a local rule — Conway's Life (B3/S23)
  and its relatives (HighLife, Day & Night, Seeds; "Generations" rules with dying states). Played,
  paused, stepped and edited; standard pattern files open; colour by state, age or density.
- **Sierpinski-type sets**, two ways:
  - **Digit automata** (exact, unlimited zoom): a finite automaton reads the base-k digits of a
    point's coordinates and accepts or rejects it — the Sierpinski triangle (binary: reject the
    digit pair (1,1), i.e. `x AND y ≠ 0`), the carpet (base 3), Cantor dust, the Vicsek fractal,
    Pascal's triangle mod p (Lucas' theorem). Static pictures; every zoom level costs the same.
  - **1-D cellular automata** (simulated, bounded): Wolfram's 256 elementary rules and Fractint's
    `cellular` type (k colours, radius r, totalistic), drawn as space-time diagrams — Rule 90 draws
    the Sierpinski triangle, Rule 30 chaos, Rule 110 is Turing-complete. The two meet: an
    *additive* rule's diagram from one cell IS a digit automaton (Rule 90 = Pascal mod 2), so those
    rules can be deep-zoomed exactly (§4.3).

### Non-goals

- A Golly replacement: no rule-table files (`.rule`), no scripting, no 3-D, no hexagonal grids, no
  non-totalistic (Hensel) isotropic rules in the first phases (§9).
- Continuous automata (Lenia, SmoothLife).
- Grid-free fractals: IFS (Barnsley fern) and L-systems need geometry, not cells (separate design;
  they reuse §3's class layer).
- Porting Golly or any GPL code (§10).

## 2. What exists today

- **Everything is an escape-time formula.** `FractalKind` (`app/src/fractal.rs:40-71`, 25 built-ins
  and `Custom`) is described by one table, `SPECS` (`:151`): name (the view-file token), `formula_id`,
  default centre, `supports_julia`, `supports_perturbation`, info. There is no exhaustive `match` on
  the kind anywhere — callers go through the table and `caps()` (`core/src/lib.rs:166`). That makes
  a per-row `class` the natural seam (§3).
- **Custom formulas are the precedent for "a family with its own GPU code".** Id 1000, outside the
  shader's ids (`core/src/lib.rs:144`); state in `FractadyneApp::custom`; an alternate pipeline set
  (`CustomPipelines`, `gpu/src/lib.rs:980-1040`) swapped in when present (`:2481-2498`); about 20
  special-case sites (view files, session, tours, menus, panels, CLI, selftest).
- **The colour pass is formula-agnostic.** Iterate passes write two `Rgba32Float` targets: main =
  (smooth value, normal.x, normal.y, DE) with interior = (−1, 0, 0, 1e30), aux = stripe/TIA/trap/
  angle (`AUX_NONE` = (0, 0, 1e30, 0)) (`mandelbrot.wgsl:756-872`). `fs_color` (`:2934`) maps main.r
  through the palette LUT, cycle/offset, optional log normalization; with normal 0 and DE 1e30, relief
  and glow switch themselves off. **A new renderer that writes main = (value, 0, 0, 1e30), interior
  < 0, aux = `AUX_NONE`, is coloured unchanged** — palettes, gradient editor, cycling, vignette, AA.
  Two obligations: live normalization reads GPU atomics the iterate pass commits (`esc_range_commit`,
  `esc_count_commit`, `:715`), so a new pass must commit them too; and the re-iterate cache is
  `IterKey` (`gpu/src/lib.rs:181-203`), which must change every generation.
- **Export can colour any buffer.** `color_iter_buffer` (`gpu/src/export.rs:3141`) colours a supplied
  main buffer through `fs_color`; `ExportRequest`'s colouring fields are reusable as they are.
- **No compute pipelines.** Every GPU pass is a fragment pass. Life stepping would add the first.
- **Time exists only in the colour pass** (palette animation, `main.rs:10517`) and in tours, which
  interpolate the camera and step the formula (`scripting.rs:1555`). Repaints: `request_repaint`.
- **View files** are `key=value` lines with an allow-list (`export.rs:279-293`), `format_version` 2 for
  custom formulas and power families; **sessions** keep `fractal` + the custom formula
  (`fractadyne-state/src/lib.rs:330-337`); **goldens** are `(name, kind, centre, zoom, iter, …)` at
  1920×1080 (`selftest.rs:8511`).

## 3. Design: the class layer

A **`FractalClass`** on each `FractalSpec` row: `EscapeTime` (every existing family, `Custom`
included), `Life`, `Cellular1D`, `DigitAutomaton`. Three new `FractalKind`s, one per class, with ids
outside the shader's range like `CUSTOM` (proposed 1100–1102), so no escape-time path can be handed
one. The class decides:

| | EscapeTime | Life | Cellular1D | DigitAutomaton |
|---|---|---|---|---|
| Renderer | `fs_iterate` & co. | compute stepper + display pass | compute rows + display pass | `fs_digit` (fragment) |
| Parameters | formula / Julia c | rule, pattern, generation | rule, start row | automaton |
| Time axis | — | generations | rows | — |
| Depth | to 1e60000× | cells (torus) / unbounded (Hashlife) | rows simulated (exact for additive rules) | unlimited, exact |
| Julia, perturbation, SA/BLA, DE, nucleus finder, autopilot | as today | off | off | off |

Gating is by class, not per feature: menus hide what does not apply (the Julia toggle, iteration
controls), and `caps()` answers "none" for the new ids, as it does for `CUSTOM`. Each class's state
lives beside `custom` (`FractadyneApp::automaton: Option<AutomatonState>`), and its GPU pipelines are
a set swapped in as `CustomPipelines` are.

**Every class writes the iteration-texture format** (§2): value in main.r, interior < 0, aux
`AUX_NONE`, and commits the normalization counters. So the colouring panel, palettes, export,
thumbnails, bookmarks and the gallery work without knowing what made the picture.

**Coordinates** stay the app's `Viewport` (BigFloat centre, extended-range scale): a Life cell `(i, j)`
is the unit square `[i, i+1) × [j, j+1)`; a digit automaton's picture is the unit square `[0, 1)²`.
Pan, zoom, click-to-zoom, bookmarks and tours then need nothing new.

## 4. Design: the three engines

### 4.1 Life-like automata

- **Rules.** `B…/S…` (B3/S23), with `/C<n>` for Generations (Brian's Brain = B2/S/C3), Moore
  neighbourhood by default and von Neumann as `V`. Parsed and validated in core, with a clear error
  in the panel (the custom formula dialog's pattern).
- **Universe.** Phase 1: a torus W×H (default 4,096², up to 16,384²; a byte per cell, two buffers =
  512 MiB at the top, inside the 1 GiB storage binding both test cards report). An unbounded plane
  comes with Hashlife (§4.4).
- **Stepping** — the app's first compute pipeline: one dispatch per generation, 16×16 workgroups
  loading an 18×18 halo into shared memory, ping-pong storage buffers. A byte per cell serves binary
  and Generations rules alike; a bit-packed binary fast path (32 cells per word, bit-sliced adders)
  is a later optimisation (§6, phase 5). A cell's age (generations alive, saturating u16) is kept
  beside it for colouring.
- **Safety.** A generation is one bounded dispatch (cells × 1), so the dead-man and dispatch ceiling
  price it as steps like any other; generations per frame come from measured step time against the
  frame budget, never a giant dispatch. Cost rows go in `ceilings.toml`, measured on both cards.
  ⚠ Fallback if compute misbehaves on a driver: the same stepper as a fragment pass into `R8Uint`
  ping-pong textures (the chunk passes' idiom).
- **Display.** A fragment pass maps pixels to cells:
  - zoomed in (≥ 1 px a cell): state → value (dead = interior; alive = age or 1; Generations' dying
    states their own values); grid lines from 8 px a cell;
  - zoomed out (< 1 px a cell): the fraction of live cells under the pixel, read from a density
    pyramid (2×2 sums, built per displayed generation), so a 16k² universe in a 1k window shows
    density, not aliasing.
- **Time.** Play / pause / step 1 / step 2^k, speed (generations a second), reset to the start pattern.
  `IterKey` carries the generation. The status bar shows generation and population in fixed-width
  fields (a conditional element reflows the bar).
- **Editing.** Draw and erase cells with the mouse; random fill (density, seed); clear. Selection /
  copy / paste of regions later.
- **Patterns.** Open RLE (`x = m, y = n, rule = B3/S23` + `b`/`o`/`$`/`!` runs; multi-state letters),
  plaintext `.cells` (`.`/`O`, `!` comments) and Life 1.06; paste RLE from the clipboard (the
  LifeWiki convention); save as RLE. A small built-in library hand-entered from definitions (glider,
  LWSS, blinker, pulsar, Gosper gun, R-pentomino, acorn, diehard). Collections are not bundled
  (§10).

### 4.2 Digit automata

- **Definition.** Base k (2–10); named states; a start state; transitions on digit pairs (x digit,
  y digit), most significant first; a `reject` (dead) state; per state a colour index. A short text
  format, parsed in core like formulas:

  ```
  # Sierpinski triangle
  base 2
  start s
  s: 11 -> reject; * -> s
  ```

  Carpet: `base 3`, `s: 11 -> reject; * -> s`. Cantor dust: `s: 1* *1 -> reject; * -> s`.
  Vicsek: `s: 01 10 11 12 21 -> s; * -> reject`. Pascal mod p is generated (states = residues,
  `r --(x,y)--> r·C(y,x) mod p`, 0 = reject; colour by residue).
- **Pixel value.** By *depth* (the digit at which the automaton rejected — the escape count's
  analogue; never rejected = interior) or by *state* (the state after the pixel's last digit, for
  coloured automata such as Pascal mod p). Outside the unit square: empty, or tiled (option).
- **Unlimited zoom at constant cost** — a reference orbit's analogue, but exact. At level L, where
  `k^−L` is just larger than the view, the view meets at most 2×2 level-L cells. The CPU reads the
  first L digits of those cells' origins exactly from the BigFloat centre and runs the automaton to
  their states (O(L) per cell, a handful of cells per frame; a k that is not a power of two converts
  by exact multiply-by-k, with precision grown by L·log₂k bits). Each pixel then knows its cell from
  its offset (df32, relative to the cell, so precision does not depend on depth) and reads only the
  next `⌈log_k(width·ss)⌉ + 2` digits on the GPU. A view at 1e1000× costs what 1× costs.
- **Renderer.** One fragment pass (`fs_digit`): the transition table and the ≤ 4 cell states in a
  small storage buffer; no stepping, no time.

### 4.3 1-D cellular automata

- **Rules.** Wolfram elementary codes (0–255); Fractint's totalistic `cellular` (k colours, radius r,
  rule as a digit string); start row = one cell, random (density, seed) or a given string; edges fixed
  or periodic.
- **Engine.** Rows are sequential, so a compute pass advances a band of rows per dispatch into a
  W×T history buffer; the display pass is §4.1's (cells in, density out). The diagram grows
  downward while playing; T is bounded like the Life torus.
- **Exact deep zoom for additive rules.** For a linear rule mod k from one cell (Rule 90, 150, 60, 102,
  and totalistic rules whose new state is the neighbourhood's sum mod k), cell (x, t) is a
  binomial/multinomial coefficient mod k — a digit
  automaton (§4.2). Such a view switches to the digit engine, with the identical picture at shallow
  depth as the gate (§7). Rule 30 and 110 have no shortcut and stay simulated.

### 4.4 Hashlife (Life's deep zoom)

Gosper's algorithm (1984): the universe as a quadtree of canonical (hash-consed) nodes, each level-n
node memoising its centre's future after 2^(n−2) generations. Regular patterns advance exponentially
far — a breeder or an OTCA-metapixel tiling (Life simulating Life: a literal self-similar zoom) runs
2^60 generations over a universe trillions of cells wide; chaotic soup gets no shortcut.

- CPU, off the UI thread like reference builds; node table with a memory budget and garbage
  collection (the orbit cache's cost-aware eviction is the model).
- Coordinates beyond i64 — the BigFloat `Viewport` already spans them.
- Rendering: each node stores its population, so a pixel's density is read from the node that covers
  it — the same pyramid as §4.1, unbounded. Uploaded as a main-buffer tile; coloured by `fs_color`.
- Macrocell (`.mc`) import, the format Hashlife programs save.

## 5. Persistence, tours, export, UI

- **View files** (`format_version` 3 for the new classes, so an older build says "newer build"
  rather than drawing Mandelbrot at the coordinates): `fractal=Life|Cellular|Automaton`, `rule=`,
  `generation=`, `pattern=` (RLE on one line — RLE needs no newlines — or `pattern_file=` beside the
  view for large ones), `start=` (1-D), `automaton=` (the text format, one line).
- **Session**: the same keys; a large pattern in `session.rle` beside `session.toml`.
- **Tours**: keyframes gain `generation` (Life, 1-D); playback advances generations monotonically
  between keyframes while the camera interpolates as now. A video of a glider gun zooming out over
  its stream is a tour.
- **Export**: stills through the class's display pass at export size, coloured by `fs_color`
  (`color_iter_buffer`); RLE save of the current universe.
- **UI**: the Fractal menu and toolbar dropdown gain an "Automata" group (Life, 1-D cellular, Digit
  automaton, each with presets). The side panel shows an "Automaton" section in place of the
  iteration controls: rule (with presets), play/pause/step/speed, generation, population, reset,
  random fill, edit mode; "Open pattern…" and "Paste RLE" in the File menu. Hidden for these
  classes: Julia, iterations, nucleus finder, autopilot.

## 6. Phases and gates

| Phase | Content | Gate |
|---|---|---|
| **0** | The class layer: `FractalClass`, the three ids, class gating in menus/panels/view files/session/CLI; no new picture yet | goldens 31/31 byte-identical; self-test unchanged; an unknown class in a view file is refused with a message |
| **1** | Life on a GPU torus: B/S + Generations rules, compute stepper, display pass (cells, age, density), play/step/speed, editing, random fill, RLE/`.cells`/Life 1.06 open, RLE save, built-in library, view files/session/tours/export | GPU = CPU stepper bit for bit over thousands of generations (random soups, torus wrap, every rule class); known facts (§7); goldens; Radeon battery |
| **2** | Digit automata: text format + presets (Sierpinski triangle, carpet, Cantor dust, Vicsek, Pascal mod p), `fs_digit`, exact prefix states, unlimited zoom | GPU = exact CPU digits per pixel at 1×, 1e10×, 1e100×, 1e1000×; membership against `x AND y` / Lucas on integer grids; self-similarity (a cell zoomed k^n is the whole picture, exactly); goldens; Radeon |
| **3** | 1-D automata: elementary + Fractint totalistic, history buffer, additive rules routed to the digit engine | GPU = CPU; Rule 90 simulated = Pascal mod 2 digit automaton pixel for pixel; goldens (Rule 30, 90, 110); Radeon |
| **4** | Hashlife: unbounded plane, step 2^k, density from node populations, macrocell open | Hashlife = plain stepper wherever both run (soups, guns, breeders); long-run facts; memory stays in budget |
| **5** | Bit-packed binary stepper; more neighbourhoods; selection/copy/paste | bit-packed = byte stepper bit for bit; measured speed-up |

Order: Life first (the user's interest, and the class whose time axis shapes the layer most), then
digit automata (cheap, and the exact-deep-zoom story), then 1-D (reuses both), then Hashlife.

## 7. Validation

- **Exactness is the gate wherever it exists.** Life, 1-D automata and digit automata are integer
  arithmetic: every GPU engine has a CPU twin and must match it bit for bit — no tolerances, no
  "smooth pixel" masks (contrast the fold families' chaos floors).
- **Known facts as anti-vacuity checks** (to be confirmed against LifeWiki when the gates are written):
  the glider's period 4 and (1, 1) displacement; the blinker's period 2; Gosper's gun (36 cells)
  emitting a glider every 30 generations; the R-pentomino settling at generation 1103 with 116 cells;
  diehard vanishing at generation 130; acorn settling at 5206 with 633 cells. A stepper that is
  subtly wrong fails these long before a golden notices.
- **Cross-engine identities**: Rule 90 simulated = Pascal mod 2; a digit automaton's picture equals
  its own k^n zoom; Hashlife = the plain stepper.
- **Mutation checks** on each gate (a wrong birth count, a missed wrap, an off-by-one digit) must go
  red, as for the power families.
- **Radeon** (field agent) at the end of each phase; the compute pipeline is new on both drivers.

## 8. Risks

- **Scope.** Golly is a large program; the phases stop at what makes these pictures, not a CA lab.
- **The first compute pipeline.** New code path on two drivers; the fragment-pass fallback (§4.1)
  keeps the phase from depending on it.
- **Time in a pipeline built for still pictures.** Reprojection, the settle logic and the frame
  budget assume a view converges; a running universe never does. The class layer must let these
  classes opt out of settle/reprojection rather than fight them (an `IterKey` that changes every
  generation is the honest signal).
- **Large patterns in view files and sessions** (megabytes of RLE): sidecar files past a size.

## 9. Open questions (the user's)

1. Torus size: default 4,096² and maximum 16,384² — or an unbounded plane in phase 1 (sparse tiles)
   rather than waiting for Hashlife?
2. Which built-in patterns and automata presets beyond the classics listed?
3. Should Fractint `.par` files with `type=cellular` open directly (the `.frm` precedent)?
4. Non-totalistic isotropic rules (Hensel notation, e.g. B2-a/S12) — wanted, or out?

## 10. Licensing

Golly (GPL) and other CA programs are not read or ported; Hashlife is implemented from Gosper's
paper and public descriptions, the stepper from the rule's definition. RLE, `.cells`, Life 1.06 and
macrocell are public formats. Built-in patterns are a handful of classics entered from their
definitions; pattern collections (LifeWiki, Golly's) are not bundled — users open their own files.

## 11. References

- M. Gardner, "Mathematical Games: The fantastic combinations of John Conway's new solitaire game
  'life'", Scientific American 223 (October 1970).
- R. Wm. Gosper, "Exploiting regularities in large cellular spaces", Physica D 10 (1984) 75–80.
- S. Wolfram, "Statistical mechanics of cellular automata", Rev. Mod. Phys. 55 (1983) 601–644.
- J.-P. Allouche, J. Shallit, *Automatic Sequences* (Cambridge, 2003) — 2-D automatic sets.
- LifeWiki (conwaylife.com/wiki): rule notation, RLE, pattern facts.
