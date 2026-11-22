# Custom coloring: writing a coloring hook

A coloring hook is a short function, written in WGSL (the graphics language Fractadyne's shaders
are written in), that colors the view in place of the coloring method. It runs on the graphics
card for every sample of the picture, at any depth, and it recolors the view the moment you save
its file: a way to try a coloring idea without building Fractadyne, and to keep colorings of your
own.

## Getting started

1. Open **Color ▸ Custom coloring…**.
2. Pick one of the **Examples**, then **Save as…** to write it to a file of your own. From then on
   that file is watched.
3. Open the file in any text editor, change it, and save. The view recolors at once.

If a save does not build, the window (and a line under the method in the control panel) says why,
with the line and column, and the last version that built stays on screen until you fix it.

**Open file…** loads a `.wgsl` file you already have. **Turn off** goes back to the method's own
colors. From the command line, `--coloring-hook FILE` colors a render or an export with a hook
(`--hook-params P1,P2,P3,P4` sets its parameters).

## The function

```wgsl
fn custom_color(l: Lanes) -> vec3<f32> {
    if (l.inside) {
        return l.color; // the view's own inside coloring
    }
    return palette(palette_map(l.count));
}
```

That is the Smooth method written out. `custom_color` gets one sample of the picture in `l` and
returns its color: red, green and blue, each from 0 to 1, as the screen shows them (sRGB). A value
outside 0 to 1 is clamped, and a value that is not a number (a division by zero, the log of a
negative) is drawn black.

### What a sample carries: `l`

| Field                      | What it holds                                                                                     |
| -------------------------- | ------------------------------------------------------------------------------------------------- |
| `l.count`                  | The smooth iteration count. Below 0 in the set (-1) and on a texel that glitched (-2).            |
| `l.normal_x`, `l.normal_y` | The slope's direction, z / (dz/dc), as a unit vector where there is one.                          |
| `l.de_log2`                | log2 of the distance to the set, in output pixels; 1e20 or more where there is none.              |
| `l.stripe`                 | The stripe average (0 to 1).                                                                      |
| `l.triangle`               | The triangle inequality average (0 to 1).                                                         |
| `l.trap`                   | The orbit's nearest approach to the trap's shape.                                                 |
| `l.angle`                  | The angle at which the orbit escaped, in turns (0 to 1).                                          |
| `l.inside`                 | Whether the sample is in the set.                                                                 |
| `l.coord`                  | Where the view's method puts this sample on the palette (1e30 inside).                            |
| `l.color`                  | The color the view's method gives this sample, inside included, with the palette's anti-aliasing. |

These are the lanes `--render-lanes` writes as NumPy arrays, by the same names, so an idea tried
in Python on a view's lanes carries over.

⚠A hook that reads a statistic (`l.stripe`, `l.triangle`, `l.trap` or `l.angle`) makes the view
collect it while iterating, whatever the method, as the method that colors by it would. That can
calculate the view again when the hook starts reading one, and it can be slower at depth. Zoom
videos cannot use a hook that reads a statistic.

### What you can call

| Function                       | What it does                                                                                                                                                                                        |
| ------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `palette(t)`                   | The view's palette at position `t`; it wraps every 1. A point sample: no anti-aliasing.                                                                                                             |
| `palette_map(v)`               | `v` mapped as the view's method maps its own value: its rate (from Cycle), Offset, and the Scale's curve when the palette is fitted to the view's range. `palette(palette_map(l.count))` is Smooth. |
| `palette_offset()`             | The Offset alone, for a value you map at a rate of your own: `palette(l.stripe * 2.0 + palette_offset())`.                                                                                          |
| `palette_inside(t)`            | The inside palette at `t`, when the view has one; the palette otherwise.                                                                                                                            |
| `with_lightness(c, lightness)` | `c` at that OKLab lightness (0 dark to 1 light), its hue and chroma kept.                                                                                                                           |
| `p1()`, `p2()`, `p3()`, `p4()` | The hook's four parameters (below).                                                                                                                                                                 |

⚠`palette_map` uses the rate of the view's method. Under Smooth that rate is in palettes per
iteration, a few hundredths, so a value from 0 to 1 (a statistic, a direction) hardly moves
through the palette under it: map such a value with `palette_offset()` and a rate of your own.

WGSL's own functions are all there: `sin`, `atan2`, `exp2`, `mix`, `clamp`, `smoothstep`, and so
on.

### Parameters

A hook has four parameters, `p1()` to `p4()`, each a slider in the window. A comment line names
one and gives its range and default:

```wgsl
// @param p1 "Line width (pixels)" 0.25 4 1
```

The window shows a slider for each parameter the hook names or calls; a double-click on a slider
gives back its default. A view, a session and a farm job keep the values.

## The rules

The color pass runs once for every sample of every pixel, and the whole picture has to finish
within the graphics driver's time limit, so a hook:

- has **no loops** (`for`, `while`, `loop`). Write the steps out.
- keeps its values in `let` and `var` inside its functions: no `var` at module scope, no
  `override` (use `const`).
- has no entry points (`@fragment`, `@vertex`, `@compute`): the color pass calls it.
- can define functions and constants of its own, under names the color pass does not use (an
  error says when a name is taken).
- is at most 64 KB of UTF-8 text.

## Where a hook applies

- The live view, in both panes of the dual view, and every export of it.
- Views saved as files, bookmarks and Share text carry the hook's text and its parameters (view
  format 5), never the file's path: a view you send to someone colors the same for them. An older
  build opens such a view in its method's colors and says it was saved by a newer Fractadyne.
- A render farm job carries it the same way.
- Your session keeps the hook and the file it came from: the next start watches that file again
  (and uses its newest text, if you saved it while Fractadyne was closed).
- Not Life or L-systems, which color their own way.
- The Effects (relief lighting, the distance glow, edge shading, equipotential lines) draw over
  your colors as they draw over a method's: edge shading and the equipotential lines on each sample
  your function returns, the others on the finished pixel.

A hook's text is its identity: the same text with Windows or Unix line ends is the same hook.

## The examples

- **Smooth iteration, written out** (`smooth.wgsl`): the Smooth method, exactly. A starting point.
- **Fringe, written out** (`fringe.wgsl`): the Fringe method: the palette swept around the slope's
  direction (`p2` sweeps a turn, from the view's Offset), at a lightness from the distance to the
  set (`p1` is the width of the dark line, in pixels).
- **Transfer curves** (`transfer-curves.wgsl`): the smooth count bent by a curve before the
  palette (`p1`: linear, square root, cube root, log, atan), then optionally cut into bands (`p2`).
- **Darken near the set** (`darken-near-set.wgsl`): any method's own colors (`l.color`), darkened
  within a few pixels of the set (`p1` the width, `p2` the strength), so the filaments stand out.

## From a hook to a built-in method

A hook is spliced into the same color pass the built-in methods run in (`mandelbrot.wgsl`), so a
coloring that proves itself as a hook becomes a method mostly by moving its body into that pass.
The hook interface itself is `crates/fractadyne-gpu/src/color_hook.wgsl`.
