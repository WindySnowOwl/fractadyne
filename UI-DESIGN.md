# Fractadyne — UI/UX Design Brief

**Status:** Draft v0.1
**Date:** 2026-06-25
**Companion to:** [`DESIGN.md`](DESIGN.md) (engineering/architecture spec)

> This is the **UI/UX brief**: design principles, information architecture,
> layout, theme tokens, a stock-widget component map, and per-screen specs. Hand
> *this* to a designer or design tool for UI work; hand `DESIGN.md` alongside it
> for feature/technical context. Items only the product owner can decide are
> marked **[DECIDE]**.

---

## 1. Direction (from kickoff answers)

Three decisions anchor every choice below:

1. **Attractive, familiar, easy — and cheap to maintain.** Favor conventional
   desktop patterns over novelty. **Avoid custom controls and hacks**; build from
   stock widgets and well-maintained ecosystem crates. "Attractive" comes from
   restraint, spacing, and a good dark theme — not bespoke flourishes.
2. **Progressive disclosure.** Uncluttered and approachable by default; detailed
   and technical features are *easily discoverable*, not removed.
3. **Dark theme first.** Design for dark; a light theme can follow later.

### 1.1 Toolkit decision — egui (confirmed)

Given the priorities above, the UI is built in **`egui`** (already the engine's
choice, rendering through the same `wgpu` device as the fractal canvas):

- Immediate-mode with **familiar stock widgets**; fast to build, update, maintain.
- Trivial to composite over the live `wgpu` fractal surface.
- A **web-UI shell (Tauri) was considered and rejected** — more design freedom,
  but it invites custom components and extra integration, against the "familiar +
  maintainable, avoid custom controls" goal.

**Implication for designers/tools:** target a *native, paneled desktop app* look
(think VS Code / OBS / Blender-modern), **not** a marketing-site aesthetic.
Designs must be expressible with standard widgets + theming. See the component
map (§8) for what's stock vs. the few justified exceptions.

### 1.2 Lean on the egui ecosystem (not custom code)

| Need                                               | Use (maintained crate / stock)                     |
| -------------------------------------------------- | -------------------------------------------------- |
| Dockable/collapsible panels                        | `egui_dock`                                        |
| Icons                                              | `egui_phosphor` (Phosphor icon set)                |
| Code editor (formula/coloring) w/ syntax highlight | `egui_code_editor`                                 |
| Tables / grids (rules, library lists)              | `egui_extras`                                      |
| Toasts / non-blocking notifications                | `egui-notify`                                      |
| Color picking                                      | egui built-in `color_edit_button` / `color_picker` |

Only **one** genuinely custom widget is expected (the gradient/palette stop
editor, §8) — and it's assembled from stock primitives. Everything else is
off-the-shelf.

---

## 2. Design Principles

1. **The canvas is the hero.** The fractal fills the window; chrome recedes and
   stays out of the way. Panels are collapsible so the art can go full-bleed.
2. **Neutral chrome so colors read true.** Because users judge *fractal* color,
   the UI itself is low-chroma, consistent dark gray (like pro photo/video
   tools). The UI accent is muted and used sparingly so it never biases color
   perception.
3. **Familiar first.** Standard menu bar, side panels, status bar, dialogs,
   right-click context menus, conventional shortcuts. Nothing to "figure out."
4. **Uncluttered default, deep on demand.** Three disclosure tiers (§4). A new
   user explores immediately; a power user reaches formulas and high-res export
   without hunting.
5. **Never block; always show state.** Long work (deep-zoom refine, glitch
   passes, export) runs async with clear, non-intrusive feedback (§7). The window
   stays interactive mid-render.
6. **Instant recolor is a feature.** Coloring is decoupled from compute
   (`DESIGN.md` §2.1); palette/algorithm tweaks update live — design controls to
   invite experimentation (drag a slider, see it immediately).
7. **Maintainable = consistent.** One spacing scale, one type scale, one token
   set (§9). Reuse layouts across screens.

---

## 3. Target User & Disclosure Model

**Primary persona — the Explorer.** Wants to dive in, zoom somewhere beautiful,
recolor, and save/export — with little setup. **Reachable without a mode switch:**
the *Artist* (palettes, high-res export) and the *Power User* (custom formulas,
coloring code, CA/L-system rules).

**Three disclosure tiers:**

- **Tier 1 — Default (uncluttered).** Canvas + top bar (fractal picker, presets,
  theme) + minimal on-canvas zoom controls + status bar. Great defaults; explore
  instantly.
- **Tier 2 — One click away.** Side panels: *Parameters* (left) and
  *Coloring/Palette* (right). Toggled from the toolbar or by dragging panel edges.
- **Tier 3 — Discoverable depth.** Formula editor, advanced coloring, high-res
  export, CA/L-system rule editors, preferences — behind **menus**, **"Advanced"
  expanders**, and a **Command Palette** (Ctrl+P / Ctrl+Shift+P) for searchable
  access to every action.

---

## 4. Information Architecture

Single main window; secondary surfaces are panels or modal dialogs (familiar,
low-maintenance) rather than separate windows.

```
Main Window
├─ Menu bar + toolbar (fractal picker, view toggles, theme, settings)
├─ Canvas area
│   ├─ Single view  (default)
│   └─ Dual linked view  (parameter plane ↔ dynamical plane)
├─ Left panel  · Parameters / Location   (collapsible)
├─ Right panel · Coloring / Palette       (collapsible)
├─ Status bar (render state, depth, iterations, coordinates, zoom %)
└─ Modal/secondary surfaces
    ├─ Formula editor            (Tier 3)
    ├─ Coloring-algorithm editor (Tier 3)
    ├─ High-res Export dialog
    ├─ Library: bookmarks / presets / custom defs
    ├─ Fractal Info panel/drawer
    └─ Preferences
```

---

## 5. Layout (low-fidelity)

Conventional, dockable (`egui_dock`), panels collapsible to maximize canvas.

> A clickable static HTML proxy of these layouts (dark tokens, panel/dual-view
> toggles) is in [`mockups/fractadyne-main-window.html`](mockups/fractadyne-main-window.html) —
> open it in a browser. It approximates the egui look; it is not the real toolkit.

**Tier 1 — default, uncluttered**

```
┌────────────────────────────────────────────────────────────┐
│ ☰ File  Fractal  View  Tools  Help     Mandelbrot ▾   ⚙  ◐ │
├────────────────────────────────────────────────────────────┤
│                                                            │
│                                                            │
│                    [ fractal canvas ]                 ⊕    │
│                                                       ⊟    │
│                                                            │
│  ‹ Parameters                              Coloring ›      │  ← collapsed panel tabs (discoverable)
├────────────────────────────────────────────────────────────┤
│ ⟳ Rendering 80%   depth 1e‑42   iter 5,000   −0.743,0.131  100% │
└────────────────────────────────────────────────────────────┘
```

**Tier 2 — panels open**

```
┌────────────────────────────────────────────────────────────┐
│ ☰ File  Fractal  View  Tools  Help     Mandelbrot ▾   ⚙  ◐ │
├───────────────┬──────────────────────────┬─────────────────┤
│ PARAMETERS    │                          │ COLORING        │
│ Type    ▾     │                          │ Algorithm  ▾    │
│ Power   [2]   │                          │ Palette    ▾    │
│ Max iter ▭▭▭  │     [ fractal canvas ]   │ ┌───gradient──┐ │
│               │                          │ │▮▮▮▮▮▮▮▮▮▮▮▮│ │
│ LOCATION      │                          │ └────────────┘ │
│ x −0.7436…    │                          │ Cycle  ▭▭      │
│ y  0.1318…    │                          │ ▸ Advanced     │
│ zoom ▭▭▭▭ +/− │                          │                │
├───────────────┴──────────────────────────┴─────────────────┤
│ ⟳ Done   depth 1e‑42   iter 5,000   −0.7436,0.1318     100% │
└────────────────────────────────────────────────────────────┘
```

**Dual linked view** (toggle in toolbar / View menu)

```
┌────────────────────────────────────────────────────────────┐
│ … toolbar …                              Dual view: [ on ]  │
├───────────────┬───────────────────┬────────────────────────┤
│ panels        │ Parameter plane   │ Dynamical plane (Julia) │
│ (as above)    │   (Mandelbrot)    │  live-linked to cursor  │
│               │     [ canvas ]    │      [ canvas ]         │
├───────────────┴───────────────────┴────────────────────────┤
│ status …                                                   │
└────────────────────────────────────────────────────────────┘
```

---

## 6. Navigation & Core Interactions

- **Pan:** left-drag. **Zoom:** mouse wheel (cursor-centered). **Box-zoom:**
  right-drag or modifier-drag a rectangle. **Reset/Home:** toolbar + shortcut.
- **Dual-view hover preview:** moving the cursor over the parameter plane updates
  the dynamical (Julia) plane live (cheap low-res while moving, full quality on
  settle); click to pin. Standard, discoverable, no custom gesture.
- **Coordinates:** shown in the status bar; clicking opens the Location panel for
  precise entry/paste (§8 — a plain monospace text field, not a custom widget).
- **Keyboard:** conventional set — Ctrl+S save, Ctrl+E export, Ctrl+Z/Y, +/−
  zoom, arrows pan, F11 full-bleed canvas, Ctrl+P command palette.
- **Context menus:** right-click canvas → "Set as Julia c", "Copy coordinates",
  "Add bookmark", "Export view…".
- **Command palette:** searchable list of all actions/fractals/palettes — the
  primary way advanced features stay discoverable without cluttering Tier 1.

---

## 7. Render & Async States

The UI must always communicate what the engine is doing, without blocking:

- **Status bar render indicator:** phase label + progress —
  `Rendering → Refining → Correcting glitches → Done`, with % and a spinner.
- **Canvas during big jumps:** show the reprojected previous frame as an instant
  placeholder; overlay a subtle progress shimmer/coarse preview until refined.
- **Glitch correction:** a quiet status note (and optional debug overlay in
  Tools) — never a blocking dialog.
- **Export:** modal dialog with determinate progress bar, ETA, output-size
  estimate, and **Cancel**; runs in the background so the main view stays live.
- **Errors/empties:** inline, friendly messages (e.g., formula compile error
  shown in the editor gutter); toasts (`egui-notify`) for transient events
  ("Bookmark saved", "Export complete").

---

## 8. Component Map (stock-first)

Reaffirming "avoid custom controls" — nearly everything maps to a stock widget:

| UI need                                    | Implementation                                                       | Custom?                                        |
| ------------------------------------------ | -------------------------------------------------------------------- | ---------------------------------------------- |
| Numeric parameter (power, bailout, iter)   | `DragValue` + `Slider`                                               | stock                                          |
| Zoom (huge range)                          | log-scaled `Slider` + `+/−` buttons + wheel                          | stock                                          |
| High-precision coordinate (100s of digits) | multiline monospace `TextEdit` + copy/paste buttons                  | stock                                          |
| Fractal type / algorithm / rule family     | `ComboBox`                                                           | stock                                          |
| Toggles (dual view, options)               | `Checkbox` / toggle                                                  | stock                                          |
| Single color                               | `color_edit_button`                                                  | stock                                          |
| **Gradient / palette stops**               | horizontal bar with draggable stop handles + `color_picker` per stop | **small custom** (built from stock primitives) |
| Formula / coloring code                    | `egui_code_editor` (syntax highlight, error gutter)                  | crate                                          |
| L-system rules                             | `egui_extras` table of `TextEdit` rows                               | crate                                          |
| CA rule                                    | `DragValue` (0–255 elementary) or B/S text field + mini preview      | stock                                          |
| Library lists (bookmarks/presets)          | `ScrollArea` + selectable rows / `egui_extras` table                 | stock/crate                                    |
| Panels & docking                           | `egui_dock`                                                          | crate                                          |
| Menus / toolbar                            | `egui::menu::bar`                                                    | stock                                          |
| Dialogs (export, prefs)                    | `egui::Window` (modal)                                               | stock                                          |
| Command palette                            | text field + filtered list                                           | small custom                                   |
| Icons                                      | `egui_phosphor`                                                      | crate                                          |

> The **gradient stop editor** is the one piece worth designing carefully — it's
> the most-touched custom surface. Keep it conventional (Photoshop/Inkscape-style
> stop strip): draggable stops, double-click to edit color, right-click to delete.

---

## 9. Visual System / Theme Tokens (dark-first)

Concrete starting values; **[DECIDE]** the accent. Neutral, low-chroma chrome.

**Color (dark)**

| Token                           | Value                             | Use                              |
| ------------------------------- | --------------------------------- | -------------------------------- |
| `bg.base`                       | `#1A1B1E`                         | window/canvas background         |
| `bg.panel`                      | `#232428`                         | panels                           |
| `bg.elevated`                   | `#2C2E33`                         | inputs, popups, headers          |
| `border`                        | `#3A3D44`                         | dividers, input outlines         |
| `text.primary`                  | `#E6E7EA`                         | main text                        |
| `text.secondary`                | `#9DA1A8`                         | labels, hints                    |
| `text.disabled`                 | `#5C6069`                         | disabled                         |
| `accent`                        | `#E0A030` (amber)                 | selection, focus, primary action |
| `accent.hover`                  | lighten 8%                        | hover                            |
| `success` / `warning` / `error` | `#5BBF7A` / `#E0A030` / `#E0584B` | render/glitch/error states       |

> Accent (**locked**): a **muted amber/gold** (`#E0A030`) — echoes the
> "‑dyne / power" identity and stays clear of rainbow fractal palettes. Kept
> modest in saturation (neutral chrome principle, §2).

**Typography**
- **UI:** clean sans (system default, or bundle **Inter**).
- **Numeric & code:** **monospace** (e.g., **JetBrains Mono**) for coordinates,
  iteration counts, depth, and the formula/coloring editors. egui loads custom
  fonts trivially.
- Type scale: 12 / 13 / 15 / 18 / 22 px (caption → H1). Default body 13–14 px.

**Spacing & shape**
- Spacing scale: 4 / 8 / 12 / 16 / 24 px. Default density: comfortable, not cramped.
- Corner radius: 4–6 px. Hairline 1 px borders. Subtle elevation, no heavy shadows.

**States**
- Clear hover/active/focus/disabled for every interactive element; visible
  keyboard focus ring (accessibility).

---

## 10. Key Screens (specs)

For each: purpose · layout · controls · disclosure tier · states.

1. **Explore (default).** Purpose: navigate + casual recolor. Canvas-hero,
   collapsed panels, on-canvas zoom, status bar. Tier 1. States: rendering /
   refining / done.
2. **Parameters / Location panel (left).** Fractal type + typed params; location
   block with high-precision x/y and log zoom. Tier 2. Live re-render on change.
3. **Coloring / Palette panel (right).** Algorithm combo, palette picker, gradient
   stop editor, cycle/offset, "▸ Advanced" expander. Tier 2. Instant recolor.
4. **Formula editor.** Code editor (guided expression *and* raw shader tabs per
   `DESIGN.md` §8.4), live preview, error gutter, "open built-in as sample" to
   fork. Tier 3 (modal/drawer).
5. **High-res Export.** Resolution presets + free numeric entry, supersampling,
   format (**PNG / OpenEXR**), output path; live time + file-size estimate;
   determinate progress + Cancel. From toolbar/menu.
6. **Library.** Tabs: Bookmarks (locations) · Presets (full state) · Custom defs
   (formulas/palettes/algorithms) — searchable list + preview thumbnails;
   import/export. Tier 2–3.
7. **Fractal Info drawer.** Renders the fractal's bundled description, formula,
   history, parameter docs, references (`DESIGN.md` §9). Tier 1 (one click).
8. **Preferences.** Theme (dark/light later), default fractal, RAM/tile budget,
   GPU info & fallback status, shortcuts. Tier 3.

---

## 11. Accessibility

- **Contrast:** meet WCAG AA for UI text on dark surfaces (the tokens above aim
  for this; verify accent-on-dark).
- **Color-vision deficiency:** ship CVD-safe *fractal* palettes and never encode
  UI state by color alone (pair with icon/label) — important in a color-heavy app.
- **Keyboard:** full keyboard navigation + visible focus; all actions reachable
  via menus/command palette.
- **Scaling:** respect OS DPI scaling and offer a UI scale setting; the canvas is
  resolution-independent.

---

## 12. Decisions (locked) & Deferred

1. **Accent color** — **amber `#E0A030`** (locked).
2. **Icon set** — **Phosphor** via `egui_phosphor` (locked).
3. **Bundled fonts** — **Inter** (UI) + **JetBrains Mono** (numeric/code) (locked).
4. **Light theme** — **deferred to post-v1.**

---

## 13. Handing this to a design tool

To get useful output (mockups/components) from a design tool, give it this brief
**plus** these constraints restated up front:

- "Native desktop app in **egui**; **stock widgets + standard desktop patterns**
  only; no bespoke/animated web components."
- "**Dark theme**, neutral low-chroma chrome; the fractal canvas is the hero."
- "**Progressive disclosure**: design Tier 1 (uncluttered) and Tier 2/3
  (expanded) variants of the main window."
- Ask for: the **main window** in the three layouts (§5), the **coloring panel +
  gradient editor**, and the **export dialog** — in the §9 tokens.
- Provide the §9 token table so output is themed consistently.
```
