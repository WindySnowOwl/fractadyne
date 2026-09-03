//! In-crate UI surfaces: the `draw_*` methods split out of `main.rs` as `impl FractadyneApp`
//! blocks, grouped by area (REFACTOR-PLAN Phase 3, intra-crate UI split). Because these stay in
//! the same crate, they keep full `&mut self` access to the app state — no `pub` widening, no
//! borrowed-context plumbing, no crate-graph inversion (see the Phase 3 analysis for why a
//! separate `fractadyne-ui` crate was rejected). Splitting one inherent `impl` across submodule
//! files is a plain Rust capability; the moves are verbatim.

pub(crate) mod central;
mod dialogs;
mod menus;
mod panels;
pub(crate) mod diagnostics;
pub(crate) mod misiurewicz_explorer;
pub(crate) mod tour_render;
