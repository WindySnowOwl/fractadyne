# fractadyne-render (roadmap placeholder)

**This crate is intentionally empty.** It reserves the name for the future render-orchestration
layer described in DESIGN.md §3.1 / §11: an off-thread scheduler that tiles the viewport, drives
CPU reference-orbit work and GPU compute, caches raw iteration buffers, and reprojects on pan/zoom.

Today that logic lives in the app crate:

- **`crates/fractadyne-app/src/render.rs`** — reference-orbit recompute, series-skip, mode select,
  the live per-view GPU request builder, and the off-thread recompute workers.
- **`crates/fractadyne-app/src/scripting.rs`** — the keyframe-tour / camera-path renderer.

Extracting it into this crate is **deferred** until that code has a headless, method-based API
(rather than reaching into `&mut FractadyneApp` and passing raw `mpsc::Receiver`s around). Pair the
extraction with the Phase 2c overlay/annotation consolidation, which gives the tour renderer a
goldens-testable input→output boundary worth a crate. See `REFACTOR-PLAN.md`.

The two other former stubs (`fractadyne-ui`, `fractadyne-fractals`) were **retired** — the UI lives
as `impl FractadyneApp` blocks in `fractadyne-app/src/ui/`, and the fractal metadata's future home is
the Phase-5 `Fractal` trait spanning core+gpu+app, not a standalone crate.
