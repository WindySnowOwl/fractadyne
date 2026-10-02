# Automata — Life-like cellular automata and Sierpinski-type digit automata

Status: **phase 1 built** (2026-10-01, §6.1): the Life core in `fractadyne-core::life`, CPU only;
nothing in the app yet. Phase 1 went before phase 0 — the two are independent, and the class layer's
state wants the engine's types. The user: "design it and plan on handling both things
like Life and Sierpinski". §9 records the user's answers to the first draft's open questions (`988f20c`):
an unbounded plane by default on a sparse structure that scales to extremely large grids; standard
sets first, custom definitions too; Fractint `.par` files open directly; non-totalistic (Hensel)
rules supported. Background (what Life, elementary automata and digit automata are) is in §1;
integration facts in §2 are from a survey of this codebase (file:line as of `7861f20`).

## 1. Goal

Two kinds of picture the app cannot make today, sharing one new layer:

- **Life-like cellular automata.** A grid of cells evolving by a local rule — Conway's Life (B3/S23)
  and its relatives (HighLife, Day & Night, Seeds; "Generations" rules with dying states;
  non-totalistic isotropic rules in Hensel notation, e.g. B2-a/S12). Played, paused, stepped and
  edited on an **unbounded plane**; standard pattern files open; colour by state, age or density.
- **Sierpinski-type sets**, two ways:
  - **Digit automata** (exact, unlimited zoom): a finite automaton reads the base-k digits of a
    point's coordinates and accepts or rejects it — the Sierpinski triangle (binary: reject the
    digit pair (1,1), i.e. `x AND y ≠ 0`), the carpet (base 3), Cantor dust, the Vicsek fractal,
    Pascal's triangle mod p (Lucas' theorem). Static pictures; every zoom level costs the same.
  - **1-D cellular automata** (simulated): Wolfram's 256 elementary rules and Fractint's `cellular`
    type (k colours, radius r, totalistic), drawn as space-time diagrams — Rule 90 draws the
    Sierpinski triangle, Rule 30 chaos, Rule 110 is Turing-complete. The two meet: an *additive*
    rule's diagram from one cell IS a digit automaton (Rule 90 = Pascal mod 2), so those rules can
    be deep-zoomed exactly (§4.4).

Every class ships a **standard set** (rules, patterns, automata — §4.6) and accepts **custom
definitions** the user writes, saves and reuses, as custom formulas do.

### Non-goals

- A Golly replacement: no rule-table files (`.rule`), no scripting, no 3-D, no larger-than-range-1
  neighbourhoods (Larger than Life), no hexagonal grids in these phases.
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
  special-case sites (view files, session, tours, menus, panels, CLI, selftest). The formula library
  and collection (`88610c5`) and the `.frm` importer (`1b8829f`) are the precedent for §4.6's
  libraries and the `.par` importer.
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
- **The viewport assumes a centre near the origin.** The centre's precision follows the
  magnification alone (`Viewport::refresh_precision`, `core/src/viewport.rs:112`, 64 bits plus one a
  zoom octave), which is right for |c| ≲ 2. A Life view 2^100 cells from the origin needs the
  centre's own magnitude counted too (§3). Zooming out past 1× has no clamp in the viewport.
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
| Extent / depth | to 1e60000× | unbounded plane: sparse tiles (±2^62 cells), Hashlife quadtree beyond | rows simulated, unbounded width (exact for additive rules) | unlimited, exact |
| Julia, perturbation, SA/BLA, DE, nucleus finder, autopilot | as today | off | off | off |

Gating is by class, not per feature: menus hide what does not apply (the Julia toggle, iteration
controls), and `caps()` answers "none" for the new ids, as it does for `CUSTOM`. Each class's state
lives beside `custom` (`FractadyneApp::automaton: Option<AutomatonState>`), and its GPU pipelines are
a set swapped in as `CustomPipelines` are.

**Every class writes the iteration-texture format** (§2): value in main.r, interior < 0, aux
`AUX_NONE`, and commits the normalization counters. So the colouring panel, palettes, export,
thumbnails, bookmarks and the gallery work without knowing what made the picture.

**Coordinates** stay the app's `Viewport` (BigFloat centre, extended-range scale): a Life cell
`(i, j)` — column i, row j, rows growing downward as pattern files read — is the unit square
`[i, i+1) × [−j−1, −j)`; a digit automaton's picture is the unit square `[0, 1)²`. Pan, zoom,
click-to-zoom, bookmarks and tours then need nothing new. Two class-aware pieces: the centre's
precision counts `log₂|centre|` as well as the magnification (escape-time views keep today's bits, so
goldens are unchanged), and the zoom readout for Life shows cells per pixel ("1 px = 4,096 cells")
rather than a magnification relative to the Mandelbrot home.

## 4. Design: the engines

### 4.1 Life: rules

One representation for every rule the engine runs: a **512-bit transition table** indexed by the
3×3 neighbourhood (the cell and its eight neighbours, one bit each: alive or not), plus a state count
C (2 for binary rules, C ≥ 3 for Generations). The steppers (CPU, GPU, Hashlife) only ever see the
table, so every notation below costs the parser, not the engines.

- **Outer totalistic**: `B3/S23`, also `23/3` (S/B, the older order). Moore neighbourhood.
- **Von Neumann**: suffix `V` (`B2/S013V`) — the table ignores the corners.
- **Non-totalistic isotropic (Hensel notation)**: `B2-a/S12`, `B3/S2-i34q`. Each count 0–8 splits
  into the configurations that are equal under the square's symmetries (rotations and reflections) —
  1, 2, 6, 10, 13, 10, 6, 2, 1 classes, 51 in all — named by letters (c, e, k, a, i, n, y, q, j, r, t,
  w, z); `2-a` is every two-neighbour class except `a`. Letters are entered from LifeWiki's chart
  as one representative neighbourhood each; the parser expands each to its symmetry orbit.
- **MAP rules**: `MAP` + base64 of the 512-bit table itself — any binary range-1 rule, isotropic or
  not; also the export format for "save this rule exactly".
- **Generations**: suffix `/C` or `/3` style (`B2/S/C3` Brian's Brain; `B2-a/S12/3` with Hensel
  letters). State 1 is alive; states 2…C−1 are dying and count as dead to their neighbours; a live
  cell the table does not keep alive starts dying.
- **B0 rules** (birth with no neighbours) flip the infinite background of an unbounded plane each
  generation. The universe carries a **background state** (§4.2): unstored cells equal it, and the
  next background is the table applied to an all-background neighbourhood. B0 rules are then exact
  on the plane with no special case in the stepper. Generations rules with B0 are refused with a
  message.

Parsed and validated in core with an error in the panel (the custom formula dialog's pattern);
a rule round-trips through its canonical string (Hensel letters in chart order, `MAP` when nothing
shorter fits).

### 4.2 Life: the universe (unbounded, sparse)

The default universe is the **unbounded plane**. Two sparse structures, each where it is strongest:

1. **Tile map — the stepping structure.** The plane is cut into 64×64-cell tiles; only tiles that
   differ from the background are stored, in a hash map keyed by tile coordinates (i64 each, so the
   plane spans ±2^62 cells — a glider would need 10^19 generations to reach the edge). Memory follows
   the live area, not the extent: a glider gun's stream 10^6 cells long is a thin line of tiles.
   Each generation steps the stored tiles plus a one-tile **halo** around them, so growth has
   somewhere to go.
   - *Why a halo suffices*: a live cell moves at most one cell a generation (range-1 rules), so a
     halo 64 cells wide holds for 63 generations. The engine reallocates every K = 16 generations
     from occupancy flags the stepper writes, with one batch of read-back latency — 32 generations
     at most, inside the 63. A **breach counter** (a live cell on a tile edge whose neighbour is not
     stored) is a tripwire that must stay zero; the self-test asserts it.
   - *Full*: when the GPU tile pool is full (default 2 × 256 MiB = 65,536 tiles, 2.7×10^8 cells of
     live area), the run **pauses with a message** — it never drops cells silently. From phase 3 the
     run hands over to Hashlife instead.
   - Tiles free themselves after a read-back shows them equal to the background with no occupied
     neighbour.
2. **Hashlife quadtree — the structure for extremely large grids** (§4.3). The universe as a tree of
   canonical (hash-consed) nodes: identical regions are stored once, at any scale. A universe 2^200
   cells wide whose content is a few thousand distinct blocks costs a few thousand nodes. Coordinates
   are BigInts, as the BigFloat `Viewport` already allows.

The two convert exactly: tiles → tree builds the tree bottom-up; tree → tiles enumerates non-empty
tile-sized nodes, possible while the count fits the pool. The engine picks the structure: tiles
while playing a generation at a time and the live area fits the pool; the tree for "step 2^k", for
patterns past the pool, and for macrocell files.

**Bounded universes** stay available as options: a torus W×H (multiples of 64 — the same tile pool
with wrap-around neighbour links, every tile stored) and a bounded plane (dead outside a rectangle).

### 4.3 Life: engines and display

- **CPU reference stepper** (core): byte per cell, a dense array big enough to hold the pattern
  plus its light cone — slow, obvious, and the truth every other engine is tested against.
- **CPU tile stepper** (core): §4.2's tile map on the CPU — the twin of the GPU stepper, and the
  engine of headless tests and exports without a GPU.
- **GPU tile stepper** — the app's first compute pipeline: a pool of 64×64-byte tiles in two
  ping-pong storage buffers, a per-tile table of its eight neighbours' slots (or "background"), the
  rule table (16 × u32) and C in a uniform. One dispatch per generation over the stored tiles,
  workgroups of 16×16 threads loading their halo into shared memory. The step pass also writes each
  tile's population and four edge-occupancy bits; the CPU reads them back once per batch to manage
  the tile map. ⚠ Fallback if compute misbehaves on a driver: the same stepper as a fragment pass
  into `R8Uint` ping-pong textures (the chunk passes' idiom).
- **Hashlife** (core, CPU, phase 3) — Gosper's algorithm: each level-n node memoises its centre's
  future after 2^(n−2) generations (and, with the step-size variant, after any 2^j ≤ 2^(n−2)), so
  regular patterns advance exponentially far — a breeder or an OTCA-metapixel tiling (Life simulating
  Life: a literal self-similar zoom) runs 2^60 generations; chaotic soup gets no shortcut. Off the UI
  thread like reference builds; a node budget with garbage collection (the orbit cache's cost-aware
  eviction is the model). Leaves step through the same 512-bit table, so every rule runs, Generations
  included.
- **Safety.** A generation is one bounded dispatch (stored cells × 1), so the dead-man and dispatch
  ceiling price it as steps like any other; generations per frame come from measured step time
  against the frame budget, never a giant dispatch. Cost rows go in `ceilings.toml`, measured on both
  cards. Hashlife steps run on the CPU and never touch the GPU's budget.
- **Display.** A fragment pass maps pixels to cells:
  - zoomed in (≥ 1 px a cell): state → value (background-equal cells = interior; alive = age or 1;
    dying states their own values); grid lines from 8 px a cell. The CPU uploads the visible tile
    grid (pool slot per tile, or "background").
  - between (1 px covers 2…64 cells): the fraction of live cells under the pixel, from a per-tile
    population pyramid (2×2 sums down to one count a tile) built by a compute pass per displayed
    generation — density, not aliasing.
  - far out (1 px covers tiles): a density grid the CPU bins from the stored tiles' populations
    (O(stored tiles) a frame), or from Hashlife node populations, which every node carries — a
    universe 2^200 cells wide is drawn from the nodes covering each pixel.
  - Age: an optional u16 plane (saturating), stored while any view colours by age; switching it on
    starts every live cell at age 0.
- **Time.** Play / pause / step 1 / step 2^k, speed (generations a second), reset to the start pattern.
  `IterKey` carries the generation. The status bar shows generation and population in fixed-width
  fields (a conditional element reflows the bar); populations past u64 (Hashlife) as a float.
- **Editing.** Draw and erase cells with the mouse; random fill of a rectangle (density, seed); clear.
  Selection / copy / paste of regions later.
- **Patterns.** Open RLE (`x = m, y = n, rule = B3/S23` + `b`/`o`/`$`/`!` runs; multi-state letters),
  plaintext `.cells` (`.`/`O`, `!` comments), Life 1.05 / 1.06, and (phase 3) macrocell `.mc`, the
  format Hashlife programs save; paste RLE from the clipboard (the LifeWiki convention); save as RLE
  or (Hashlife) macrocell.

### 4.4 Digit automata

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
  coloured automata such as Pascal mod p or 2-D Thue–Morse). Outside the unit square: empty, or
  tiled (option).
- **Unlimited zoom at constant cost** — a reference orbit's analogue, but exact. At level L, where
  `k^−L` is just larger than the view, the view meets at most 2×2 level-L cells. The CPU reads the
  first L digits of those cells' origins exactly from the BigFloat centre and runs the automaton to
  their states (O(L) per cell, a handful of cells per frame; a k that is not a power of two converts
  by exact multiply-by-k, with precision grown by L·log₂k bits). Each pixel then knows its cell from
  its offset (df32, relative to the cell, so precision does not depend on depth) and reads only the
  next `⌈log_k(width·ss)⌉ + 2` digits on the GPU. A view at 1e1000× costs what 1× costs.
- **Renderer.** One fragment pass (`fs_digit`): the transition table and the ≤ 4 cell states in a
  small storage buffer; no stepping, no time.

### 4.5 1-D cellular automata

- **Rules.** Wolfram elementary codes (0–255); Fractint's totalistic `cellular` (k colours, radius r,
  rule as a digit string); start row = one cell, random (density, seed, over a given width) or a
  given string; outside the start row the line is background (unbounded), or periodic / fixed edges
  (options).
- **Engine.** Rows are sequential, so a compute pass advances a band of rows per dispatch. The
  history is stored in the **same sparse tile map** as Life, with time as the second axis: a
  single-cell start's light cone is a triangle, so tiles cover only the cone, and the diagram grows
  sideways and downward without a fixed W×T buffer. The display pass is §4.3's (cells in, density
  out).
- **Exact deep zoom for additive rules.** For a linear rule mod k from one cell (Rule 90, 150, 60, 102,
  and totalistic rules whose new state is the neighbourhood's sum mod k), cell (x, t) is a
  binomial/multinomial coefficient mod k — a digit automaton (§4.4). Such a view switches to the digit
  engine, with the identical picture at shallow depth as the gate (§7). Rule 30 and 110 have no
  shortcut and stay simulated.
- **Fractint `.par` files open directly.** File → Open accepts `.par`; the file's entries
  (`name { key=value … }`, `;` comments, continuation lines) are listed as the `.frm` importer lists
  formulas, and a `type=cellular` entry opens as a 1-D automaton: `params=` gives the initial row, the
  rule, the type (k and r as two digits) and the start row; `corners=`/`center-mag=` the framing;
  `colors=` (Fractint's compressed palette) a gradient. Parameter meanings are taken from Fractint's
  documentation and checked against entries whose pictures are known. Entries of other types are
  listed as not supported (later: entries naming a `.frm` formula could open through the custom
  formula importer).

### 4.6 Standard sets and custom definitions

Each class ships a standard set, and everything in it is also something a user can write, save and
reuse — the formula library's model (`88610c5`): built-in sections read-only, a user section saved in
the config directory, and import/export as text.

- **Life rules**: Life, HighLife, Day & Night, Seeds, Life without Death, 34 Life, 2×2, Diamoeba,
  Maze, Mazectric, Coral, Anneal, Long Life, Morley, Replicator, Gnarl, Amoeba, Assimilation; Generations:
  Brian's Brain, Star Wars; Hensel: tlife (B3/S2-i34q), B2-a/S12. Custom: any rule string the parser
  accepts (§4.1), named and saved.
- **Life patterns**: still lifes (block, beehive, loaf, boat), oscillators (blinker, toad, beacon,
  pulsar, pentadecathlon), spaceships (glider, LWSS, MWSS, HWSS), methuselahs (R-pentomino, diehard,
  acorn), guns (Gosper glider gun, Simkin glider gun), entered from their definitions. Custom: open,
  paste or draw, then "Save to library"; a user pattern folder.
- **Digit automata**: Sierpinski triangle, carpet, Cantor dust, Vicsek (cross and saltire), Pascal
  mod 2/3/5/7, 2-D Thue–Morse. Custom: the text format of §4.4 in an editor with parse errors in place
  (the formula dialog's), saved to the library.
- **1-D rules**: elementary 30, 90, 110, 150, 184, 18, 22, 54, 60, 102; a few Fractint totalistic
  rules. Custom: any code or `cellular` parameters; `.par` files.

## 5. Persistence, tours, export, UI

- **View files** (`format_version` 3 for the new classes, so an older build says "newer build"
  rather than drawing Mandelbrot at the coordinates): `fractal=Life|Cellular|Automaton`, `rule=`
  (canonical string), `universe=plane|torus:W×H|bounded:W×H`, `generation=` (decimal, any size),
  `pattern=` (RLE on one line — RLE needs no newlines — or `pattern_file=` beside the view for large
  ones; a macrocell sidecar for Hashlife universes), `start=` (1-D), `automaton=` (the text format,
  one line).
- **Session**: the same keys; a large pattern in `session.rle` / `session.mc` beside `session.toml`;
  the user's libraries live in the config directory as the formula library does.
- **Tours**: keyframes gain `generation` (Life, 1-D); playback advances generations monotonically
  between keyframes while the camera interpolates as now. A video of a glider gun zooming out over
  its stream is a tour.
- **Export**: stills through the class's display pass at export size, coloured by `fs_color`
  (`color_iter_buffer`); RLE / macrocell save of the current universe.
- **UI**: the Fractal menu and toolbar dropdown gain an "Automata" group (Life, 1-D cellular, Digit
  automaton). The side panel shows an "Automaton" section in place of the iteration controls: rule
  (library picker + text field), universe, play/pause/step/speed, generation, population, reset,
  random fill, edit mode; "Open pattern…", "Paste RLE" and `.par` in the File menu. Hidden for these
  classes: Julia, iterations, nucleus finder, autopilot.

## 6. Phases and gates

| Phase | Content | Gate |
|---|---|---|
| **0** | The class layer: `FractalClass`, the three ids, class gating in menus/panels/view files/session/CLI, centre precision that counts the centre's magnitude, per-class zoom readout; no new picture yet | goldens 31/31 byte-identical; self-test unchanged; an unknown class in a view file is refused with a message; a centre 2^100 out keeps its sub-cell bits through pan/zoom |
| **1** | Life core (CPU, no app): rule parser and table compiler (B/S, S/B, V, Hensel, MAP, Generations, B0 via the background state), the tile-map universe (plane, torus, bounded), the reference and tile steppers, RLE / `.cells` / Life 1.05–1.06 read and write, the built-in rule and pattern sets | compiler vs definitions (Hensel letters partition the 256 neighbourhoods into 51 symmetry orbits with the counts of §4.1; B3/S23 = "B3cekainyqjr/S2cekain3cekainyqjr"; MAP round-trips); tile stepper = reference stepper bit for bit over thousands of generations (soups, gliders crossing tile corners, torus wrap, B0, every rule class); known facts (§7); mutations go red |
| **2** | Life in the app: GPU tile stepper (the first compute pipeline), display (cells, age, density pyramid, far-out density grid), play/step/speed, editing, random fill, open/paste/save, the libraries with custom rules and patterns, view files/session/tours/export | GPU = CPU tile stepper bit for bit including the tile set (growth, churn, pool full = pause); breach counter zero; goldens; Radeon battery |
| **3** | Hashlife: quadtree universe, step 2^k, coordinates beyond i64, density from node populations, node budget + GC, macrocell read/write, tiles ⇄ tree, automatic engine choice | Hashlife = tile stepper wherever both run (soups, guns, breeders, B0); long-run facts at 2^40 generations; memory stays in budget |
| **4** | Digit automata: text format, standard set + custom editor, `fs_digit`, exact prefix states, unlimited zoom | GPU = exact CPU digits per pixel at 1×, 1e10×, 1e100×, 1e1000×; membership against `x AND y` / Lucas on integer grids; self-similarity (a cell zoomed k^n is the whole picture, exactly); goldens; Radeon |
| **5** | 1-D automata: elementary + Fractint totalistic, sparse history, `.par` import, additive rules routed to the digit engine | GPU = CPU; Rule 90 simulated = Pascal mod 2 digit automaton pixel for pixel; `.par` entries with known pictures; goldens (Rule 30, 90, 110); Radeon |
| **6** | Bit-packed binary stepper; selection/copy/paste; more neighbourhoods | bit-packed = byte stepper bit for bit; measured speed-up |

Order: Life first (the user's interest, and the class whose time axis shapes the layer most), with
Hashlife straight after it (the user's "extremely large grids"), then digit automata (cheap, and the
exact-deep-zoom story), then 1-D (reuses both).

### 6.1 Phase 1 results (2026-10-01)

`crates/fractadyne-core/src/life/`: `rule.rs` (the 512-bit table, every notation, canonical strings),
`hensel.rs` (the letter chart), `universe.rs` (the tile map: plane / torus / bounded, background
state, stepper), `dense.rs` (the reference stepper), `formats.rs` (RLE, plaintext, Life 1.05/1.06),
`library.rs` (22 rules, 18 patterns). 40 tests; core 218 (was 178); release build, about 4 s of
the core suite's time.

- **Hensel chart** decoded from LifeWiki's 51 images, then checked against the page's *text*, which
  the images do not determine: the complement rule (`(8−n)x` = complement of `nx`), the
  von Neumann groups and the checkerboard-dual table (51 class-to-class mappings each way).
- **MAP**: LifeWiki's Life string decodes to B3/S23 with the index order above — pinning the bit
  order and the centre's place.
- **Tile = reference**, cell for cell, every generation: 13 rules (totalistic, Generations, Hensel,
  von Neumann, a random MAP table, B0 blinking and B0/S8) × plane / torus / bounded soups for 60
  generations; a Life soup for 2,000; a glider across tile corners for 4,000 (tiles freed behind it)
  and through negative coordinates; a glider round a 64² torus.
- **Facts** (LifeWiki): periods of four still lifes and five oscillators (pulsar 3, pentadecathlon 15,
  minimal); glider (1, 1)/4, LWSS/MWSS/HWSS 2/4 the same way; R-pentomino 116 cells at 1103 and
  still 116 at 2000; diehard empty at 130 (129 not); acorn 633 at 5206 and 6006; Gosper and Simkin
  guns +5 cells every 30 / 120 generations; B0/S single cell period 2 through a 3×3 hole; AntiLife
  = Life with the colours swapped for 200 generations.
- **Mutations, each red**: tile ignores its NW neighbour (10 tests); Hensel 3n ↔ 3q swapped (both
  page-text checks); torus without wrap (3); Generations dying state off by one (the rule test — the
  steppers share `Rule::next`, so only it can see this); bounded edge off by one (the soup test,
  through Seeds only — the one rule whose soup reaches that wall in 60 generations); plane without
  its halo (10).
- Not yet done from §8's Hensel risk: a published pattern in a published Hensel rule (no offline
  source; the page-text checks stand in).

## 7. Validation

- **Exactness is the gate wherever it exists.** Life, 1-D automata and digit automata are integer
  arithmetic: every engine has a CPU twin and must match it bit for bit — no tolerances, no "smooth
  pixel" masks (contrast the fold families' chaos floors). The reference stepper is deliberately
  independent of the tile map, so a tiling bug cannot hide in both.
- **Known facts as anti-vacuity checks** (to be confirmed against LifeWiki when the gates are written):
  the glider's period 4 and (1, 1) displacement; the blinker's period 2; Gosper's gun (36 cells)
  emitting a glider every 30 generations; the R-pentomino settling at generation 1103 with 116 cells;
  diehard vanishing at generation 130; acorn settling at 5206 with 633 cells. A stepper that is
  subtly wrong fails these long before a golden notices. On the unbounded plane these run without
  wrap-around: the R-pentomino's escaping gliders must still exist at generation 2000.
- **Sparse-structure checks**: a glider driven across a tile corner and 10^4 tiles away keeps 5 cells
  and the stored tile count stays bounded (tiles free behind it); a pool too small for a soup pauses
  the run with the message, not a wrong picture.
- **Cross-engine identities**: Rule 90 simulated = Pascal mod 2; a digit automaton's picture equals
  its own k^n zoom; Hashlife = the tile stepper; the GPU = the CPU tile stepper.
- **Mutation checks** on each gate (a wrong birth count, a swapped Hensel letter, a missed tile
  neighbour, a missed wrap, an off-by-one digit) must go red, as for the power families.
- **Radeon** (field agent) at the end of each phase; the compute pipeline is new on both drivers.

## 8. Risks

- **Scope.** Golly is a large program; the phases stop at what makes these pictures, not a CA lab.
- **The first compute pipeline.** New code path on two drivers; the fragment-pass fallback (§4.3)
  keeps the phase from depending on it.
- **Two sparse structures.** Tiles and the tree must agree; the exact conversion both ways and the
  "Hashlife = tile stepper" gate keep them honest. The tile pool's read-back latency is bounded by
  the halo argument (§4.2), with the breach counter as its tripwire.
- **Hashlife memory.** Chaotic patterns defeat the memo table; the node budget and GC must degrade to
  "slower", never to "out of memory", and the engine choice prefers tiles for chaotic play.
- **Time in a pipeline built for still pictures.** Reprojection, the settle logic and the frame
  budget assume a view converges; a running universe never does. The class layer must let these
  classes opt out of settle/reprojection rather than fight them (an `IterKey` that changes every
  generation is the honest signal).
- **Large patterns in view files and sessions** (megabytes of RLE): sidecar files past a size.
- **Hensel letters.** A mis-entered letter is a silently different rule; the orbit-partition check
  catches a malformed chart but not a swapped pair of letters, so each letter is also tested against
  its LifeWiki picture and against a published pattern in a published Hensel rule.

## 9. Decisions (the user's, 2026-10-01)

1. **Universe**: "Default to an unbounded plane. Use an appropriate structure for sparse matrices so
   that extremely large grids can be supported." → §4.2: a tile map for stepping, a Hashlife quadtree
   for extremely large grids; torus and bounded plane as options. Hashlife moves up to phase 3.
2. **Presets**: "Start with the standard sets and allow defining custom ones." → §4.6.
3. **Fractint `.par` with `type=cellular`**: "Yes, open directly." → §4.5.
4. **Hensel non-totalistic rules**: "Yes, have those options." → §4.1 (with MAP rules, which the
   table representation gives for free).

## 10. Licensing

Golly (GPL) and other CA programs are not read or ported; Hashlife is implemented from Gosper's
paper and public descriptions, the stepper from the rule's definition. RLE, `.cells`, Life 1.05/1.06,
macrocell, Hensel notation, MAP rules and Fractint's `.par` format are public formats, implemented
from their published descriptions. Built-in patterns are classics entered from their definitions;
pattern collections (LifeWiki, Golly's) are not bundled — users open their own files.

## 11. References

- M. Gardner, "Mathematical Games: The fantastic combinations of John Conway's new solitaire game
  'life'", Scientific American 223 (October 1970).
- R. Wm. Gosper, "Exploiting regularities in large cellular spaces", Physica D 10 (1984) 75–80.
- S. Wolfram, "Statistical mechanics of cellular automata", Rev. Mod. Phys. 55 (1983) 601–644.
- J.-P. Allouche, J. Shallit, *Automatic Sequences* (Cambridge, 2003) — 2-D automatic sets.
- LifeWiki (conwaylife.com/wiki): rule notation (including isotropic non-totalistic rules and MAP
  rules), RLE, macrocell, pattern facts.
- Fractint documentation: the `cellular` fractal type and parameter files.
