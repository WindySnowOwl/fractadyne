# View rotation

The user, 2026-11-28: "work on the image rotation and then the coloring work" (TODO.md, "Rotation for
views", requested 2026-11-01). Today the view model has no rotation: a pixel's offset from the
screen's center is the complex offset, x along Re and y along Im. This note fixes the conventions,
lists every place the mapping is assumed, and stages the work so that every stage renders the
unrotated view exactly as before, bit for bit.

Inventory: two read-only sweeps on 2026-11-28 (the GPU crate; the app, core and state crates).
Line numbers below are from main `2cb2d2fc`.

## 1. Conventions

| Item                | Decision                                                                                                                                                                                                                                                                                                             |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| The angle           | `Viewport::rotation`, degrees, f64. **Positive turns the picture counterclockwise on the screen**, like turning a photo; the same sense as the light's angle and the trap's rotation (both counterclockwise degrees).                                                                                                |
| The mapping         | A screen offset `(dx, dy)` in pixels, y up, maps to the plane as `R(-θ)·(dx, dy)·upp`: the screen's x axis points along `e^{-iθ}` in the plane. At θ = 0 it is today's mapping.                                                                                                                                      |
| Exact quarter turns | `(cos, sin)` are computed once per view in f64 and snapped to exact 0/±1 at multiples of 90°, so a quarter turn is an exact permutation of the unrotated pixels (the shader's oracle, §4).                                                                                                                           |
| Range               | Kept in (−180°, 180°] when set by the user; a tour may carry any value and interpolates unwrapped (several turns in one glide are allowed).                                                                                                                                                                          |
| Screen-fixed things | The relief light, its highlight, the fringe's direction, the statistic bump and gloss stay fixed **to the screen** (as the zoom video's assembly already does, `FramePlacement.turns`): the shader writes the slope normal in screen axes. The orbit statistics, the trap and its outline stay in the orbit's plane. |
| Each pane its own   | The dual view's Julia pane is a viewport of its own and has its own rotation.                                                                                                                                                                                                                                        |
| Not rotated         | The minimap (north up; its marker becomes a turned quad), Life and L-system views (their own renderers; the rotation is 0 there), the zoom-video strips (rendered at 0; the view's angle reaches the video as a constant phase of `FramePlacement.turns`).                                                           |

⚠The zoom video's own rotation (`--assemble-rotate`, turns per octave) is **clockwise**-positive; a
view's angle enters it with its sign flipped. `.kfr` `RotateAngle`'s sense is not yet checked against
Kalles Fraktaler: stage 2 pins it with a test that names the assumption, as the `.ugr` importer did.

## 2. Where the mapping is assumed

### GPU crate

- **The iterate prologue**, four copies: `iterate_at` (mandelbrot.wgsl:1955-1966, behind
  `fs_iterate`, the gather pass, the split refresh, the probe), `fs_iterate_chunk` (3097-3106),
  `chunk_fe_at` (3494-3506, also the export's packed pass) and a custom formula's own `chunk_fe_at`
  (custom.rs:1334-1341, no slots: easy to miss). Each computes `off = (step.xy·(gx − W/2),
  step.zw·(H/2 − gy))`. The rotation goes after that per-axis scaling and before `ref_offset` or
  `center` is added (rotating the texel first would shear a frame whose x and y steps differ by
  rounding). `ref_offset`, `center` and `julia_c` are plane quantities: never rotated.
- **The uniforms**: `IterU`/`IterUniforms` gain a row `(cos, sin, on, pad)` (the layout gate
  `iter_uniform_matches_the_shader`); built at the live `prepare` (lib.rs:5408-5480), the export's
  `TileRenderer::new`/`tile` (export.rs:2127-2231), `render_iter_tiled_impl` (3331-3387), the
  gather `run` (3906-3951), `render_iter_passes` (4282-4294) and the test kit. The twin runs the
  live `prepare`. `IterKey` (lib.rs:1103-1145) gains the angle.
- **`de_log2`** takes the texel size as `|step.x|`: unchanged while the rotation follows the scaling.
- **The slope normal** (`slope_normal`, 673-690, every writer goes through it): turned by `R(+θ)`
  into screen axes (the picture's direction on screen). Then the light (5268, 5334), the highlight,
  the bump (screen axes already), the fringe (`fringe_turn`) and the coloring hooks' `normal_x/y`
  need nothing.
- **Reprojection** (fs_color 5099-5113; `uv_offset`, `uv_scale`): a held or reprojected frame
  gains the rotation between it and the view on screen, applied in aspect-correct pixel units.
  `ColorU`'s free `_pad_ly0/_pad_ly1` take `(cos, sin)`. The present gate's hold snapshot goes
  through the same path. Assembled video frames pass 0 (the assembly rotates them itself).
- **Unchanged**: BLA and series validity radii (Euclidean), the reference offset, `GpuScale`'s
  per-axis magnitudes, the diagnostic grids, aprons and the vignette, seed and adopt (guarded by
  the view stamp, which gains the angle).

### Core, app and state crates

- **`Viewport`** (core viewport.rs:48-58) gains `rotation`. Five hand-written copies of the mapping
  move onto one private helper: `pixel_to_complex` 122-127, `complex_to_pixel` 133-139,
  `pan_pixels` 141-147, `zoom_at` 174-189, `complex_at_pixel_f64` 338-345 (and `pixel_offset`
  117 becomes two-dimensional). `recenter_and_zoom`, `zoom_by`, `zoom_to_rect` inherit them;
  `pan_complex` needs nothing. `gpu_scale` carries `(cos, sin)` out to every request builder.
- **Copies outside `Viewport`** (each moved onto the helper): the Julia pin `complex_screen_pos`
  (main.rs:12390-12399), the orbit overlay's projection (central.rs:519-527), the minimap marker
  (central.rs:843-856), the autopilot's edge projection (autopilot.rs:1438: becomes right once the
  shader's normals are in screen axes), the zoom snap's axis anchor (zoom_snap.rs:203), the hold
  transform (frame_plan.rs:336-341), `view_inside` (motion_worker.rs:69), the drift checks
  (render.rs:1305, 9977), the reference pick's candidate grid and `nucleus_in_view`
  (reference.rs:3518-3534, 3833), the status bar's precision check (menus.rs:1643-1650).
- **Fresh viewports built without an angle** (they would silently render unrotated): the reticle's
  preview (main.rs:12682-12686), the tour renderer (scripting.rs:4376-4406) and its helpers
  (`measure_norm_anchor` 3698, `tour_deep_reference` 3739, the prefetch 4285, 4328, the bench
  5325). Each gets the angle as an explicit argument.
- **Identity keys** that must include it, or a rotation alone resumes an old walk or accumulation:
  `pos_sig` (render.rs:9809-9830), `view_key` (9411), the walk's `view_bits` (decide.rs:1969-1975),
  `WalkSig` (frame_plan.rs:195-209), the onset key (6685), `norm_signature` (12552), the
  accumulation key (main.rs:18363-18510), `OrbitKey` (central.rs:404-415), `snap_view_sig`
  (main.rs:12745), the method sheet's key (sheet_render.rs:440-466), the GPU `IterKey`; and a
  rotation must stamp `settle_t` as a pan does.
- **Requests**: `build_params` (render.rs:8285) takes unpacked center and span: it takes the angle
  too; `build_export_request` (4704-4900) and `params_to_request*` (4916, 5006) copy it.
- **Glitch correction**: the seed pixel's coordinate (render.rs:5174-5176) must be the shader's
  rotated texel center exactly, or the loop does not converge.
- **Files and state**: `.fdn` key `rotation=` (degrees) written only when it is not 0, **format 7**
  (`VIEW_FORMAT_ROTATION`, written only by a turned view: an older build would show it unturned and
  say only "unknown key"), and `julia_view_rotation=` for a turned Julia pane; ⚠the reader puts
  a missing key back to 0 (today a missing key keeps the current value, export.rs:1449). The
  session (`SessionState`, additive, `serde(default)`), the undo snapshot (`ViewSnapshot`,
  main.rs:6436), the crash view's text (main.rs:982-1003), bookmarks and share text (the view
  text), tours (keyframe field `rotate`, degrees, interpolated unwrapped; the generator writes it),
  the CLI (`--rotate DEG`), the `.kfr` importer (`RotateAngle`, dropped today: bignum.rs:1077).
- **Jump sites** decide keep or reset: Go to (keeps unless the location states one), a bookmark or
  a file (the file's, 0 when absent), random location, home and reset (0), the formula library and
  the Misiurewicz explorer (keep).

## 3. Stages

Each stage is its own branch and merge; each leaves the unrotated view bit for bit as it was.

| Stage | Scope                                                                                                                                                                               | Visible                                                                                           |
| ----- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| R1    | `Viewport::rotation` and the one mapping helper; every CPU copy and identity key; files, session, undo, tours, CLI, `.kfr`; requests carry the angle                                | Nothing yet: the GPU still renders unrotated, so the angle is forced to 0 at the request boundary |
| R2    | The shader: the prologue's rotation on every path, the normal in screen axes, `IterKey`; reprojection's rotation term                                                               | `--rotate` renders, exports, tours                                                                |
| R3    | Live interaction: drag, wheel, click-to-zoom and its reticle, the zoom box, the minimap's turned marker, the zoom snap, the cursor readout, holds                                   | A rotated live view that behaves                                                                  |
| R4    | The control: a rotation field and reset in the Navigate panel, a gesture (Ctrl+drag) and keys; then the autopilot's counter-rotation of a Misiurewicz dive (`twist_deg` per repeat) | The feature                                                                                       |

## 4. Tests

- **R1**: unit tests of the helper (pixel → plane → pixel round trips at 0, 30, 45, 90, 180 and
  −135 degrees; the point under the cursor stays put through `zoom_at`; a drag moves the picture
  with the pointer at any angle; at 0 every mapping returns the old values to the bit); `.fdn`
  round trips (format 7 only when turned; a missing key reads 0); a tour's `rotate` interpolated;
  the session; `--rotate`; `.kfr` `RotateAngle`.
- **R2**: ⭐an exact oracle: a square view turned by 90° is the unrotated view's pixels permuted, bit
  for bit, on every path (direct, df32, floatexp, chunked, tiled, the gather) — the snapped
  `(cos, sin)` make the offsets the same numbers. At 30° and 45°: the CPU escape oracle at the
  shader's rotated pixel centers. The relief: a view turned by θ with the light turned by θ draws
  the unturned view's picture turned (the normal in screen axes). Speed: the prologue runs once per
  pixel, not per iteration, but the strip note (strip.rs:9-13) measured a shared-prologue change at
  1-2%: A/B at rotation 0, interleaved builds (`--bench-matrix`, ABABAB). Goldens unchanged.
- **R3**: UI walk steps (a turned view: drag, wheel at a point, click-to-zoom landing within a pixel,
  the reticle's preview turned, the minimap's marker a turned quad), a livetest of a turned glide.
- **R4**: the control's steps; the Tan Lei invariant (TODO, "Tan Lei invariant goldens"): a
  Misiurewicz view at m and at m·|λ| turned by arg λ converge, needing no stored image; the
  autopilot's dive holds the spiral still (its screen angle drift measured).

## 5. Risks

1. The hold and reprojection transform: hand-flipped signs; a mistake shows only while a reference
   builds, as a held frame sliding or turning the wrong way.
2. A missed copy of the mapping: a drag pans wrongly or the zoom anchor drifts, only when turned.
3. The glitch-correction seed's coordinate: the only symptom is leftover glitches.
4. A missed identity key: a rotation alone resumes an old walk or accumulation (a ghost of the
   old orientation).
5. Axis-aligned box tests (the pick grid, `nucleus_in_view`, drift): performance churn, rebasing.
6. Sign conventions meeting: the video's clockwise turns, the screen's y down, `RotateAngle`.
7. Fresh viewports built without the angle: tours and previews silently unturned.
8. The shared prologue's speed (measured, not assumed).

## 6. Decided while building R1 + R2 (2026-11-28)

R1 and R2 went in as one stage: the shader was cheap to turn once the view model had the angle,
and the quarter-turn oracle tests both together.

- **The reference's region stays the plane-axis box** of the view's spans about its center: the
  pick's candidate grid, `nucleus_in_view`, the drift and the reuse tests all keep it. The series
  and BLA bounds are radii (`|r| + half diagonal`, `2.5·diagonal`), so they hold on a turned view.
  Turning only the drift tests would make a plane-grid pick read as off screen on a turned
  non-square view, which is the per-frame recompute loop the drift site warns about. A turned
  view also keeps the unturned view's reference, so a turn alone does not rebuild it.
- **The screen's own questions are asked in screen axes**: the held frame's offset
  (`frame_plan::plane_spans_on_screen`, inside `hold_transform_for`), the pin frame's
  inside-check (`view_inside`), the status bar's precision reach (the turned box's bounding box),
  the zoom snap's axis anchor (the cursor's foot on a slanted axis), the minimap's turned quad,
  the Julia pin and the orbit overlay (`Viewport::pixel_of_f64`).
- **Identity keys fold `rotation_bits`** (0 for an unturned view, so every key of an unturned view
  is the number it was): `pos_sig`, `settings_hash` (with it `view_key`, the onset key and the
  walk's `view_bits`), the GPU `IterKey`, the normalization signature, the fold tripwire, the
  orbit overlay's key, the click snap's key and the method sheet's key.
- **`settle_t`** is stamped by gestures only; a turn gets its stamp with the control (R4).
- **Home** keeps the angle while it glides and unturns at its end (`reset_view`); turning back
  along the glide is R4. A family switch, toggling the dual view (its pane), a random location
  and every file without the key unturn.
- **The glitch seed** goes through `pixel_to_complex`: exact at quarter turns, and elsewhere the
  GPU's f32 `(cos, sin)` leave it a δc under 1e-7 px, far too small to glitch.
- **Tours**: `rotate` is inherited forward from 0, so a tour that never states it plays unturned;
  the dual view's Julia pane in a tour is unturned. Kept, not restored, when a tour ends (it is
  part of where the tour left the view).
- **The CPU oracle runs on the overview**: at the seahorse at 2e3x a fifth of the pixels miss the
  CPU unturned too (84 of 400; the direct path's f32 orbit near the boundary), which would hide a
  turn's error. The quarter-turn oracle covers `iterate_at`, `fs_iterate_chunk` (500,000
  iterations, every tile chunked), `chunk_fe_at` and the custom module's copy, each row with its
  tile and chunk counts as the control that the path ran.
- **Until R3** a held frame and the view share an angle: a jump to a file at another angle shows
  the old frame unturned against the new one until the new picture lands.

## 7. Decided while building R3 (2026-11-28)

Most of the live interaction needed nothing: a drag pans by the pointer's screen pixels
(`pan_px`), and the wheel, click-to-zoom, the zoom box and the reticle all go through `Viewport`.

- **A held frame keeps its own angle** (`RefCache::frozen_rot`, set wherever `frozen_center` is:
  a real frame, an adopted pin, a worker frame from its ticket). The hold transform turns the view
  center's offset into the HELD frame's screen axes, and the color pass turns each pixel's offset
  from the center by `held − view` in the held frame's pixels before the scale
  (`frame_plan::hold_turn` → `uv_turn`, in what were `ColorU`'s pad words; `(0, 0)` is no turn).
  Oracle: `hold_turn_tests` mirrors `fs_color` against the two viewports' mappings (a flipped
  sign fails by half a pixel). Not yet seen on screen: a turn while a frame is held needs the R4
  gesture.
- **A turn stops a walk in progress** (`PinStop::Turn`, a split pin too) rather than pin frames
  shadowing the angle: a turn is a short gesture and the hold covers it. A finished walk still
  adopts, held turned.
- **The zoom video**: the strips stay at 0; `--rotate` reaches the assembly as a constant
  `-deg/360` of every frame's turns (clockwise there), on top of `--assemble-rotate`; a tour's
  timeline turns by its keyframes' `rotate`. The dialog passes `--rotate`, the farm forwards it.
- **The walk's live check compares 12-pixel block averages**: per pixel, the view's sub-pixel dust
  alone differs by ~25 levels between the live drawing and an export; by blocks 2.6 unturned (the
  control), 3.0 turned, and 43.5 against an export at the wrong angle.
