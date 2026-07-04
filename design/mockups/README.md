# Fractadyne — UI mockups (egui, dark-first)

High-fidelity, **buildable** mockups of every surface in `UI-DESIGN.md`. Target look:
native paneled desktop app in **egui** — stock widgets, standard desktop patterns,
neutral low-chroma chrome, amber accent, canvas-as-hero. **Not** a web aesthetic.

The interactive source is `../Fractadyne.dc.html` (open in a browser; toolbar toggles
Dual / Panels, the icons open each dialog). The PNGs below are pulled from it.

## Tokens (locked — `UI-DESIGN.md` §9, §12)

| Token                       | Value                             | Use                                                          |
| --------------------------- | --------------------------------- | ------------------------------------------------------------ |
| `bg.base`                   | `#1A1B1E`                         | window / canvas background                                   |
| `bg.panel`                  | `#232428`                         | side panels, status bar, menu bar                            |
| `bg.elevated`               | `#2C2E33`                         | inputs, combos, popups, headers                              |
| `border`                    | `#3A3D44`                         | dividers, input outlines (1px hairline)                      |
| `text.primary`              | `#E6E7EA`                         | main text                                                    |
| `text.secondary`            | `#9DA1A8`                         | labels, hints                                                |
| `text.disabled`             | `#5C6069`                         | disabled / key captions                                      |
| `accent`                    | `#E0A030`                         | selection, focus, slider grab, primary button, active toggle |
| `accent.hover`              | `#ECB14A`                         | hover                                                        |
| `success / warning / error` | `#5BBF7A` / `#E0A030` / `#E0584B` | render / glitch / error                                      |

- **Fonts:** Inter (UI) · JetBrains Mono (numbers, coordinates, code/status bar).
- **Spacing:** 4 / 8 / 12 / 16 / 24.  **Radius:** 4–6px.  **Density:** comfortable.
- **Panels:** left 266px, right 298px (both collapsible). Menu bar 40px, status bar 26px.
- Primary button = filled amber, text `#1b1500`. Secondary = `bg.elevated` + hairline,
  hover lifts the border to accent. Active toolbar toggle = `rgba(224,160,48,.15)` fill +
  accent border + accent text.

## Surfaces

| PNG  | Surface                                                    | Tier | egui implementation                                                                                                                                                                                                                                                                                                                                                  |
| ---- | ---------------------------------------------------------- | ---- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `01` | **Explore — Tier 1 (uncluttered)** + status "Refining 80%" | 1    | `egui::menu::bar` + canvas over the `wgpu` surface. On-canvas zoom/home buttons. Status bar shows phase + spinner (`egui-notify`-style), depth/iter/ref-orbits/coords/GPU/zoom in mono.                                                                                                                                                                              |
| `02` | **Tier 2 — panels open**                                   | 2    | `egui_dock` SidePanels. Left = Parameters/Location; right = Coloring/Palette.                                                                                                                                                                                                                                                                                        |
| `03` | **Dual linked view** (Mandelbrot ↔ Julia)                  | —    | Two `wgpu` views split 50/50; right pane `c` driven by hover over the left pane (cheap low-res while moving, full on settle; click to pin). `PlaneLinkage`.                                                                                                                                                                                                          |
| `04` | **Coloring + gradient stop editor** (per-stop popup)       | 2    | The one custom widget: stop strip with draggable diamond handles, double-click → color popup (`color_picker`: SV square + hue strip + hex/pos). Right-click deletes. Everything else stock combos/sliders.                                                                                                                                                           |
| `05` | **High-res export** dialog                                 | —    | `egui::Window` modal. Preset `ComboBox`, W/H `DragValue`, supersample combo, PNG/EXR radio, path + Browse, live time/size/tile estimate, determinate progress + Cancel. Runs on the export worker; view stays live.                                                                                                                                                  |
| `06` | **Library** (Bookmarks / Presets / Custom defs)            | 2–3  | `egui::Window`; tab bar; `ScrollArea` of selectable rows w/ thumbnails (`egui_extras`); detail pane + Load / Import / Export.                                                                                                                                                                                                                                        |
| `07` | **Formula editor** (Guided ⇆ Raw WGSL)                     | 3    | `egui::Window`; `egui_code_editor` with syntax highlight + error gutter (line-8 marker mirrored in footer). Tabs share one compiler. Live preview + typed param widgets. "Open built-in as sample" forks the editable built-in def.                                                                                                                                  |
| `08` | **Command palette** (Ctrl+P)                               | 3    | Small modal: mono text field + filtered list, grouped (Fractals / Actions), selected row accent-filled, shortcut hints right-aligned. Primary discoverability path for Tier-3 features.                                                                                                                                                                              |
| `09` | **Fractal Info drawer**                                    | 1    | Right drawer rendering `FractalInfo` (description, formula, history, params, refs) from bundled metadata.                                                                                                                                                                                                                                                            |
| `10` | **Preferences**                                            | 3    | `egui::Window`; left section nav + right content. GPU detected (RTX 3080 · Vulkan), fallback tier, tile RAM budget slider, tile size, bignum backend, SA toggle.                                                                                                                                                                                                     |
| `11` | **L-system — main window** (Tier 2)                        | 2    | Left panel reconfigures: Preset / Axiom (`TextEdit`) / Angle° (`DragValue`) / Iterations `Slider` with an amber string-explosion caution / inline Rules + "Edit rules…" / Symbols legend. Right = Color mode combo, Stroke `Slider`, depth gradient strip, Background swatch. **No deep-zoom controls** (vector/turtle pipeline). Status: `segments · iter · order`. |
| `12` | **L-system rule editor** (modal)                           | 3    | Mirrors `07`. `egui::Window`; Axiom field; **Rules table** (`egui_extras`: Symbol \| → \| Production, add-row + per-row trash); Angle°/Iterations; right rail = live preview + Commands legend + "Open built-in as sample"; footer validation (`✓ valid · n rules · max depth`) + Revert / Compile & apply.                                                          |
| `13` | **Cellular automaton — 1-D elementary** (space-time)       | 2    | Left: `[1-D \| 2-D]` segmented Mode toggle; Rule family combo; Rule number `DragValue` 0–255; **8-pattern transition editor** (neighborhood triples + toggleable output cells, updates rule # live); Initial / Generations / Cell size. Right = Alive/Dead swatches, Color-by-age toggle + palette. **No precision wall.** Status: `rule · gen · cells`.             |
| `14` | **Cellular automaton — 2-D life-like**                     | 2    | Mode toggle on 2-D; Preset combo; **B/S rule editor** (two rows of 0–8 toggle chips → `B3/S23` notation); Initial pattern (density `Slider` + Clear/Randomize/Draw); Grid size; Toroidal-wrap toggle; **playback bar** (⏮ ⏯ ⏭ + speed `Slider` + gen counter). Right = swatches, Age-heatmap + Grid-lines toggles, palette. Status: `B3/S23 · gen · live · grid`.    |

### Family switching

The top selector and left-panel **Type** combo now offer **Mandelbrot / L-system / Cellular
automaton**; selecting one reconfigures the left Parameters panel, the right Coloring panel,
the canvas label, and the status-bar metrics. CA additionally carries a 1-D/2-D mode toggle.
L-systems and CA are **separate pipelines** (`DESIGN.md` §4 / §4.1) — they expose **no
perturbation, reference-orbit, bignum, or `1e-N` depth controls** (escape-time only).
The three new custom pieces — the **1-D transition grid** (`13`), the **B/S toggle rows**
(`14`), and the **L-system rules table** (`12`) — are all assembled from stock primitives;
playback is stock buttons + slider.

## Notes for implementation

- **Compute/coloring split is visible in the UI:** palette, cycle, offset and algorithm
  changes must re-shade instantly from cache (no re-iteration) — design the panels to
  invite live dragging.
- **Never block.** Export, glitch passes and deep-zoom refine run async; the status bar +
  canvas progress shimmer (see `01`) communicate state; toasts (`egui-notify`) for transient
  events ("Bookmark saved").
- **Coordinates** are arbitrary-precision: the Location fields are multiline monospace
  `TextEdit` (hundreds of digits) with copy buttons — not numeric spinners.
- **Stock-first.** Only the gradient stop strip (`04`) and the command-palette filter are
  custom, and both are assembled from stock primitives. Everything else maps 1:1 to a
  maintained crate or built-in widget (see `UI-DESIGN.md` §8).
- Canvas art in these mockups is a **striped placeholder** — the real `wgpu` fractal
  surface renders there.

> Mockups are ~0.71× scale of the 1280×800 design window; exact pixel values live in
> `UI-DESIGN.md` and the inline styles of `Fractadyne.dc.html`.
