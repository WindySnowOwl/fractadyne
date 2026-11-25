//! The coloring field table (coloring plan B4): every coloring setting a view carries, one row each,
//! with what the `.fdn` writer and reader need — its key, its kind and range, the settings it hangs
//! off and when the picture depends on it. The writer ([`view_lines`]) and the reader
//! ([`apply_view`]) walk the table on a [`ColoringSpec`], the struct the session saves and a farm
//! job carries, so a setting gets its view key by gaining a row here.
//!
//! The rules are A1's (`view_coloring`, which is now a typed front on this table):
//! - ⭐**A view carries what its picture depends on, and nothing else.** A key is written only when
//!   its setting changes the picture ([`Field::shows`]) and differs from the default, so an ordinary
//!   Smooth view is written byte for byte as before these keys existed.
//! - ⭐**Reading follows `iter_div`.** A file that states its coloring (`cycle=`) and leaves a key
//!   out was written at that key's default, so the setting returns to it. A setting that hangs off
//!   others ([`Field::parents`]: the stripe density off the method, the light's angle off the light)
//!   returns to its default only when those were read from this file (or reset by it) and the
//!   picture depends on it; when one of them is unreadable (an unknown method), what hangs off it is
//!   left alone. A file that states no coloring leaves every setting it does not name alone.
//! - A value out of range is clamped and named; an unreadable one keeps the current setting.

use crate::{slider_range, ColorMethod, InteriorMode, TrapType};
use fractadyne_state::ColoringSpec;

/// How a row reads and writes its setting on a [`ColoringSpec`], by kind.
#[derive(Clone, Copy)]
pub(crate) enum Access {
    /// `1` or `0`.
    Flag(fn(&ColoringSpec) -> bool, fn(&mut ColoringSpec, bool)),
    /// A number, clamped into `lo..=hi` on reading (written with `{}`, the shortest decimal that
    /// reads back to the same bits).
    Real(fn(&ColoringSpec) -> f32, fn(&mut ColoringSpec, f32), f32, f32),
    /// A whole number, clamped into `lo..=hi` on reading.
    Count(fn(&ColoringSpec) -> u32, fn(&mut ColoringSpec, u32), u32, u32),
    /// One of the keys `choices` lists.
    Choice(fn(&ColoringSpec) -> &str, fn(&mut ColoringSpec, &str), fn() -> Vec<&'static str>),
    /// Three numbers `r,g,b` in 0..=1 (display space), clamped on reading.
    Rgb(fn(&ColoringSpec) -> [f32; 3], fn(&mut ColoringSpec, [f32; 3])),
}

/// One coloring setting a view carries.
pub(crate) struct Field {
    /// Its key in a view (`.fdn`, a bookmark, a Share text, an image's embedded view).
    pub(crate) key: &'static str,
    /// What a message calls it when its value had to be clamped.
    pub(crate) name: &'static str,
    /// What it does, for the tour schema (`TOURS.md`, `[render.coloring]`).
    pub(crate) doc: &'static str,
    pub(crate) access: Access,
    /// The rows it hangs off (earlier in the table): it is reset to its default only when they
    /// were read from the same file.
    pub(crate) parents: &'static [&'static str],
    /// Whether the picture depends on it, given the rest of the coloring.
    pub(crate) shows: fn(&ColoringSpec) -> bool,
}

/// The palette map curves' keys (C2).
fn transfer_keys() -> Vec<&'static str> {
    fractadyne_core::coloring::Transfer::ALL.iter().map(|t| t.key()).collect()
}

/// The wraps' keys (C2).
fn wrap_keys() -> Vec<&'static str> {
    fractadyne_core::coloring::Wrap::ALL.iter().map(|w| w.key()).collect()
}

/// The edge shading's curves' keys (C3).
fn edge_curve_keys() -> Vec<&'static str> {
    fractadyne_core::coloring::EdgeCurve::ALL.iter().map(|c| c.key()).collect()
}

fn method(s: &ColoringSpec) -> ColorMethod {
    ColorMethod::from_key(&s.color_method)
}

fn always(_: &ColoringSpec) -> bool {
    true
}

/// "Normalize deep colors" decides only for Smooth (`live_norm_window`).
fn uses_normalize_live(s: &ColoringSpec) -> bool {
    method(s) == ColorMethod::Smooth
}

/// The escape count colors the view: what the dual view's Julia link shifts (C11).
fn escape_count(s: &ColoringSpec) -> bool {
    method(s) == ColorMethod::Smooth
}

/// "Always fit range" and the log scale act on Smooth's escape range and on the orbit trap's values
/// (`live_norm_window`, `norm_map_for`).
fn uses_fit(s: &ColoringSpec) -> bool {
    matches!(method(s), ColorMethod::Smooth | ColorMethod::OrbitTrap)
}

/// The inside is colored by the trap (B5): chosen so, or the trap method's own.
fn interior_trap(s: &ColoringSpec) -> bool {
    InteriorMode::parse_key(&s.interior).unwrap_or(InteriorMode::of_method(method(s))) == InteriorMode::Trap
}

/// The trap is drawn: the trap method colors the outside by it, whatever colors the inside, and an
/// inside colored by the trap (any method's) is drawn by it. Its shape and place then matter.
/// (Fixed 2026-11-25: the trap's rows asked `interior_trap`, so the trap method with another
/// inside, a solid color, left them out of a view, which reopened on the session's trap.)
fn trap_drawn(s: &ColoringSpec) -> bool {
    method(s) == ColorMethod::OrbitTrap || interior_trap(s)
}

/// The trap method's inside has a trap of its own (`interior_trap`).
fn inside_own_trap(s: &ColoringSpec) -> bool {
    method(s) == ColorMethod::OrbitTrap && interior_trap(s) && TrapType::parse_key(&s.interior_trap).is_some()
}

fn two_color_on(s: &ColoringSpec) -> bool {
    s.use_binary || s.use_duotone
}

/// The table, in the order a view writes its keys. A row's parents come before it.
pub(crate) const FIELDS: [Field; 58] = [
    Field {
        key: "method",
        name: "coloring method",
        doc: "The coloring method: smooth, stripe, triangle, trap, distance, fringe or decomposition.",
        access: Access::Choice(|s| &s.color_method, |s, v| s.color_method = v.to_string(), || {
            ColorMethod::ALL.iter().map(|m| m.key()).collect()
        }),
        parents: &[],
        shows: always,
    },
    Field {
        key: "stripe_freq",
        name: "stripe density",
        doc: "Stripe average: the stripes' density (method \"stripe\").",
        access: Access::Real(
            |s| s.stripe_freq,
            |s, v| s.stripe_freq = v,
            *slider_range::STRIPE_FREQ.start(),
            *slider_range::STRIPE_FREQ.end(),
        ),
        parents: &["method"],
        shows: |s| method(s) == ColorMethod::Stripe,
    },
    Field {
        key: "stripe_tail",
        name: "stripe tail",
        doc: "Stripe average over the last `stripe_tail_len` iterations only, which keeps its contrast at depth (method \"stripe\").",
        access: Access::Flag(|s| s.stripe_tail, |s, v| s.stripe_tail = v),
        parents: &["method"],
        shows: |s| method(s) == ColorMethod::Stripe,
    },
    Field {
        key: "stripe_tail_len",
        name: "tail length",
        doc: "The tail's length in iterations (with `stripe_tail`).",
        access: Access::Count(
            |s| s.stripe_tail_len,
            |s, v| s.stripe_tail_len = v,
            *slider_range::STRIPE_TAIL_LEN.start(),
            *slider_range::STRIPE_TAIL_LEN.end(),
        ),
        parents: &["method", "stripe_tail"],
        shows: |s| method(s) == ColorMethod::Stripe && s.stripe_tail,
    },
    Field {
        key: "interior",
        name: "inside",
        doc: "What colors the inside of the set: \"solid\" (one color) or \"trap\" (how near each orbit came to the orbit trap, through `interior_palette`). Left out: the method's own (method \"trap\" colors it by its trap, every other method one color).",
        access: Access::Choice(|s| &s.interior, |s, v| s.interior = v.to_string(), || {
            InteriorMode::ALL.iter().map(|m| m.key()).collect()
        }),
        parents: &[],
        shows: always,
    },
    Field {
        key: "interior_palette",
        name: "inside palette",
        doc: "The palette the trap colors the inside with, a preset's id (when the inside is colored by the trap). Left out: the outside palette.",
        access: Access::Choice(|s| &s.interior_palette, |s, v| s.interior_palette = v.to_string(), || {
            fractadyne_color::PRESETS.iter().map(|p| p.id).collect()
        }),
        parents: &["method", "interior"],
        shows: interior_trap,
    },
    Field {
        key: "interior_fit",
        name: "inside fit",
        doc: "Live view and its exports: fit the inside's palette to the trap values measured inside the set on screen (when the inside is colored by the trap). An offline render keeps the fixed rate.",
        access: Access::Flag(|s| s.interior_fit, |s, v| s.interior_fit = v),
        parents: &["method", "interior"],
        shows: interior_trap,
    },
    Field {
        key: "trap",
        name: "orbit trap",
        doc: "The orbit trap's shape: point, cross, circle, line, square, hexagon, star, sine or s-curve (method \"trap\", or an inside colored by the trap).",
        access: Access::Choice(|s| &s.trap_type, |s, v| s.trap_type = v.to_string(), || {
            TrapType::ALL.iter().map(|t| t.key()).collect()
        }),
        parents: &["method", "interior"],
        shows: trap_drawn,
    },
    Field {
        key: "trap_x",
        name: "trap center",
        doc: "Where the orbit trap sits in the orbit's plane, its real coordinate (with the trap).",
        access: Access::Real(
            |s| s.trap_x,
            |s, v| s.trap_x = v,
            *slider_range::TRAP_CENTER.start(),
            *slider_range::TRAP_CENTER.end(),
        ),
        parents: &["method", "interior"],
        shows: trap_drawn,
    },
    Field {
        key: "trap_y",
        name: "trap center",
        doc: "Where the orbit trap sits in the orbit's plane, its imaginary coordinate (with the trap).",
        access: Access::Real(
            |s| s.trap_y,
            |s, v| s.trap_y = v,
            *slider_range::TRAP_CENTER.start(),
            *slider_range::TRAP_CENTER.end(),
        ),
        parents: &["method", "interior"],
        shows: trap_drawn,
    },
    Field {
        key: "trap_rotation",
        name: "trap rotation",
        doc: "The trap's turn, in degrees counterclockwise (the shapes that turn: not the point or the circle).",
        access: Access::Real(
            |s| s.trap_rotation,
            |s, v| s.trap_rotation = v,
            *slider_range::TRAP_ROTATION.start(),
            *slider_range::TRAP_ROTATION.end(),
        ),
        parents: &["method", "interior", "trap"],
        shows: |s| trap_drawn(s) && TrapType::from_key(&s.trap_type).spec().turns,
    },
    Field {
        key: "trap_size",
        name: "trap size",
        doc: "The trap's size: the circle's radius, the polygons' and the star's outer corners' distance from its center (the shapes with a size: not the point, the cross or the line).",
        access: Access::Real(
            |s| s.trap_size,
            |s, v| s.trap_size = v,
            *slider_range::TRAP_SIZE.start(),
            *slider_range::TRAP_SIZE.end(),
        ),
        parents: &["method", "interior", "trap"],
        shows: |s| trap_drawn(s) && TrapType::from_key(&s.trap_type).spec().sized,
    },
    Field {
        key: "trap_phase",
        name: "trap phase",
        doc: "The sine wave trap's phase, in degrees: the wave shifted along itself, a full wave every 360 (the shapes with a phase: the sine wave).",
        access: Access::Real(|s| s.trap_phase, |s, v| s.trap_phase = v, *slider_range::TRAP_PHASE.start(), *slider_range::TRAP_PHASE.end()),
        parents: &["method", "interior", "trap"],
        shows: |s| interior_trap(s) && TrapType::from_key(&s.trap_type).spec().phased,
    },
    Field {
        key: "interior_trap",
        name: "inside trap",
        doc: "The inside's own trap shape, point, cross, circle, line, square, hexagon, star, sine or s-curve, when the trap method colors the inside by a trap (the inside's own placement in the four keys after). Left out: the method's trap.",
        access: Access::Choice(|s| &s.interior_trap, |s, v| s.interior_trap = v.to_string(), || {
            TrapType::ALL.iter().map(|t| t.key()).collect()
        }),
        parents: &["method", "interior"],
        shows: |s| method(s) == ColorMethod::OrbitTrap && interior_trap(s),
    },
    Field {
        key: "interior_trap_x",
        name: "inside trap center",
        doc: "The inside's own trap's center, its real coordinate (with `interior_trap`).",
        access: Access::Real(|s| s.interior_trap_x, |s, v| s.interior_trap_x = v, *slider_range::TRAP_CENTER.start(), *slider_range::TRAP_CENTER.end()),
        parents: &["method", "interior", "interior_trap"],
        shows: inside_own_trap,
    },
    Field {
        key: "interior_trap_y",
        name: "inside trap center",
        doc: "The inside's own trap's center, its imaginary coordinate (with `interior_trap`).",
        access: Access::Real(|s| s.interior_trap_y, |s, v| s.interior_trap_y = v, *slider_range::TRAP_CENTER.start(), *slider_range::TRAP_CENTER.end()),
        parents: &["method", "interior", "interior_trap"],
        shows: inside_own_trap,
    },
    Field {
        key: "interior_trap_rotation",
        name: "inside trap rotation",
        doc: "The inside's own trap's turn, as `trap_rotation`.",
        access: Access::Real(|s| s.interior_trap_rotation, |s, v| s.interior_trap_rotation = v, *slider_range::TRAP_ROTATION.start(), *slider_range::TRAP_ROTATION.end()),
        parents: &["method", "interior", "interior_trap"],
        shows: |s| inside_own_trap(s) && TrapType::from_key(&s.interior_trap).spec().turns,
    },
    Field {
        key: "interior_trap_size",
        name: "inside trap size",
        doc: "The inside's own trap's size, as `trap_size`.",
        access: Access::Real(|s| s.interior_trap_size, |s, v| s.interior_trap_size = v, *slider_range::TRAP_SIZE.start(), *slider_range::TRAP_SIZE.end()),
        parents: &["method", "interior", "interior_trap"],
        shows: |s| inside_own_trap(s) && TrapType::from_key(&s.interior_trap).spec().sized,
    },
    Field {
        key: "interior_trap_phase",
        name: "inside trap phase",
        doc: "The inside's own trap's phase, as `trap_phase`.",
        access: Access::Real(|s| s.interior_trap_phase, |s, v| s.interior_trap_phase = v, *slider_range::TRAP_PHASE.start(), *slider_range::TRAP_PHASE.end()),
        parents: &["method", "interior", "interior_trap"],
        shows: |s| inside_own_trap(s) && TrapType::from_key(&s.interior_trap).spec().phased,
    },
    Field {
        key: "light",
        name: "relief light",
        doc: "Relief lighting from the distance estimate.",
        access: Access::Flag(|s| s.light, |s, v| s.light = v),
        parents: &[],
        shows: always,
    },
    Field {
        key: "light_angle",
        name: "light angle",
        doc: "The light's direction in radians (with `light`).",
        access: Access::Real(|s| s.light_angle, |s, v| s.light_angle = v, f32::MIN, f32::MAX),
        parents: &["light"],
        shows: |s| s.light,
    },
    Field {
        key: "light_height",
        name: "relief",
        doc: "Relief strength: smaller is sharper (with `light`).",
        access: Access::Real(
            |s| s.light_height,
            |s, v| s.light_height = v,
            *slider_range::LIGHT_HEIGHT.start(),
            *slider_range::LIGHT_HEIGHT.end(),
        ),
        parents: &["light"],
        shows: |s| s.light,
    },
    Field {
        key: "light_steep",
        name: "relief slope from the distance",
        doc: "The relief's slope from the distance: steeper near the set and flat far from it, 45 degrees `light_steep_px` pixels from the set, instead of 45 degrees everywhere (with `light`).",
        access: Access::Flag(|s| s.light_steep, |s, v| s.light_steep = v),
        parents: &["light"],
        shows: |s| s.light,
    },
    Field {
        key: "light_steep_px",
        name: "relief steep within",
        doc: "How many output pixels from the set the relief's surface is at 45 degrees, 1 to 256 (with `light_steep`).",
        access: Access::Real(
            |s| s.light_steep_px,
            |s, v| s.light_steep_px = v,
            *slider_range::RELIEF_STEEP_PX.start(),
            *slider_range::RELIEF_STEEP_PX.end(),
        ),
        parents: &["light", "light_steep"],
        shows: |s| s.light && s.light_steep,
    },
    Field {
        key: "light_highlight",
        name: "relief highlight",
        doc: "The relief's highlight, 0 to 1: a glossy rim where the surface turns toward the light (with `light_steep`).",
        access: Access::Real(
            |s| s.light_highlight,
            |s, v| s.light_highlight = v,
            *slider_range::RELIEF_HIGHLIGHT.start(),
            *slider_range::RELIEF_HIGHLIGHT.end(),
        ),
        parents: &["light", "light_steep"],
        shows: |s| s.light && s.light_steep,
    },
    Field {
        key: "light_shininess",
        name: "relief shininess",
        doc: "The relief highlight's shininess, 4 to 512: higher is a smaller, sharper gleam (with `light_steep`).",
        access: Access::Real(
            |s| s.light_shininess,
            |s, v| s.light_shininess = v,
            *slider_range::RELIEF_SHININESS.start(),
            *slider_range::RELIEF_SHININESS.end(),
        ),
        parents: &["light", "light_steep"],
        shows: |s| s.light && s.light_steep,
    },
    Field {
        key: "light_highlight_color",
        name: "relief highlight color",
        doc: "The relief highlight's color, as display-space [r, g, b] (with `light_steep`).",
        access: Access::Rgb(|s| s.light_highlight_color, |s, v| s.light_highlight_color = v),
        parents: &["light", "light_steep"],
        shows: |s| s.light && s.light_steep,
    },
    Field {
        key: "glow",
        name: "distance glow",
        doc: "Distance glow along the set's edge.",
        access: Access::Flag(|s| s.de, |s, v| s.de = v),
        parents: &[],
        shows: always,
    },
    Field {
        key: "glow_strength",
        name: "glow",
        doc: "Glow strength (with `glow`).",
        access: Access::Real(
            |s| s.de_strength,
            |s, v| s.de_strength = v,
            *slider_range::GLOW_STRENGTH.start(),
            *slider_range::GLOW_STRENGTH.end(),
        ),
        parents: &["glow"],
        shows: |s| s.de,
    },
    Field {
        key: "glow_width",
        name: "band width",
        doc: "Glow band width (with `glow`).",
        access: Access::Real(
            |s| s.de_width,
            |s, v| s.de_width = v,
            *slider_range::GLOW_WIDTH.start(),
            *slider_range::GLOW_WIDTH.end(),
        ),
        parents: &["glow"],
        shows: |s| s.de,
    },
    Field {
        key: "edge",
        name: "edge shading",
        doc: "Edge shading: the colors near the set shaded toward `edge_color` (or the inside's color, `edge_inside`), fading out `edge_width` pixels from it.",
        access: Access::Flag(|s| s.edge, |s, v| s.edge = v),
        parents: &[],
        shows: always,
    },
    Field {
        key: "edge_inside",
        name: "edge inside color",
        doc: "Shade the edges toward the inside's color instead of `edge_color`: the set's edge smoothed by the distance (with `edge`).",
        access: Access::Flag(|s| s.edge_inside, |s, v| s.edge_inside = v),
        parents: &["edge"],
        shows: |s| s.edge,
    },
    Field {
        key: "edge_width",
        name: "edge width",
        doc: "How far from the set the edge shading reaches, in output pixels, 0.25 to 32 (with `edge`).",
        access: Access::Real(
            |s| s.edge_width,
            |s, v| s.edge_width = v,
            *slider_range::EDGE_WIDTH.start(),
            *slider_range::EDGE_WIDTH.end(),
        ),
        parents: &["edge"],
        shows: |s| s.edge,
    },
    Field {
        key: "edge_strength",
        name: "edge strength",
        doc: "The edge color's share at the set, 0 to 1 (with `edge`).",
        access: Access::Real(
            |s| s.edge_strength,
            |s, v| s.edge_strength = v,
            *slider_range::EDGE_STRENGTH.start(),
            *slider_range::EDGE_STRENGTH.end(),
        ),
        parents: &["edge"],
        shows: |s| s.edge,
    },
    Field {
        key: "edge_curve",
        name: "edge curve",
        doc: "How the edge shading fades out from the set: linear, sqrt (a thin line with a soft tail), square (a bold one), smooth or hard (a solid line `edge_width` wide) (with `edge`).",
        access: Access::Choice(|s| s.edge_curve.as_str(), |s, v| s.edge_curve = v.to_string(), edge_curve_keys),
        parents: &["edge"],
        shows: |s| s.edge,
    },
    Field {
        key: "edge_color",
        name: "edge color",
        doc: "The color the edges are shaded toward, as display-space [r, g, b] (with `edge`, unless `edge_inside`).",
        access: Access::Rgb(|s| s.edge_color, |s, v| s.edge_color = v),
        parents: &["edge", "edge_inside"],
        shows: |s| s.edge && !s.edge_inside,
    },
    Field {
        key: "equipotentials",
        name: "equipotential lines",
        doc: "Equipotential lines: lines along the levels of the escape count, `equipotential_width` pixels wide and at least `equipotential_spacing` apart at every depth, in `equipotential_color`.",
        access: Access::Flag(|s| s.equipotentials, |s, v| s.equipotentials = v),
        parents: &[],
        shows: always,
    },
    Field {
        key: "equipotential_spacing",
        name: "equipotential spacing",
        doc: "The least distance between neighboring equipotential lines, in output pixels, 4 to 64; finer lines fade in as the gaps widen toward twice this (with `equipotentials`).",
        access: Access::Real(
            |s| s.equipotential_spacing,
            |s, v| s.equipotential_spacing = v,
            *slider_range::GRID_SPACING.start(),
            *slider_range::GRID_SPACING.end(),
        ),
        parents: &["equipotentials"],
        shows: |s| s.equipotentials,
    },
    Field {
        key: "equipotential_width",
        name: "equipotential width",
        doc: "The equipotential lines' width in output pixels, 0.5 to 8 (with `equipotentials`).",
        access: Access::Real(
            |s| s.equipotential_width,
            |s, v| s.equipotential_width = v,
            *slider_range::GRID_WIDTH.start(),
            *slider_range::GRID_WIDTH.end(),
        ),
        parents: &["equipotentials"],
        shows: |s| s.equipotentials,
    },
    Field {
        key: "equipotential_strength",
        name: "equipotential strength",
        doc: "The line color's share on an equipotential line, 0 to 1 (with `equipotentials`).",
        access: Access::Real(
            |s| s.equipotential_strength,
            |s, v| s.equipotential_strength = v,
            *slider_range::GRID_STRENGTH.start(),
            *slider_range::GRID_STRENGTH.end(),
        ),
        parents: &["equipotentials"],
        shows: |s| s.equipotentials,
    },
    Field {
        key: "equipotential_color",
        name: "equipotential color",
        doc: "The equipotential lines' color, as display-space [r, g, b] (with `equipotentials`).",
        access: Access::Rgb(|s| s.equipotential_color, |s, v| s.equipotential_color = v),
        parents: &["equipotentials"],
        shows: |s| s.equipotentials,
    },
    // Duotone or Binary: one choice (`palette_mode`), held in the session as two switches. Off is
    // the absence of the key, and never written; Binary wins when both are set, as
    // `chosen_gradient` reads them.
    Field {
        key: "palette_mode",
        name: "palette mode",
        doc: "Duotone or binary in place of the palette: \"duotone\" or \"binary\" (left out: the palette).",
        access: Access::Choice(
            |s| {
                if s.use_binary {
                    "binary"
                } else if s.use_duotone {
                    "duotone"
                } else {
                    "off"
                }
            },
            |s, v| {
                s.use_binary = v == "binary";
                s.use_duotone = v == "duotone";
            },
            || vec!["duotone", "binary"],
        ),
        parents: &[],
        shows: always,
    },
    Field {
        key: "palette_lo",
        name: "palette colors",
        doc: "Duotone's shadow, Binary's color inside the set, as display-space [r, g, b] (with `palette_mode`).",
        access: Access::Rgb(|s| s.duotone_lo, |s, v| s.duotone_lo = v),
        parents: &["palette_mode"],
        shows: two_color_on,
    },
    Field {
        key: "palette_hi",
        name: "palette colors",
        doc: "Duotone's highlight, Binary's color outside the set, as display-space [r, g, b] (with `palette_mode`).",
        access: Access::Rgb(|s| s.duotone_hi, |s, v| s.duotone_hi = v),
        parents: &["palette_mode"],
        shows: two_color_on,
    },
    Field {
        key: "interior_color",
        name: "inside color",
        doc: "The color of a solid inside, as display-space [r, g, b] (when the inside is solid; Duotone and Binary use `palette_lo`).",
        access: Access::Rgb(|s| s.interior_color, |s, v| s.interior_color = v),
        parents: &["method", "interior", "palette_mode"],
        shows: |s| !interior_trap(s) && !two_color_on(s),
    },
    Field {
        key: "normalize_live",
        name: "normalize deep colors",
        doc: "Live view only: Normalize deep colors (method \"smooth\"). An offline render uses `[render]` `normalize`.",
        access: Access::Flag(|s| s.normalize_live, |s, v| s.normalize_live = v),
        parents: &["method"],
        shows: uses_normalize_live,
    },
    Field {
        key: "normalize_fit",
        name: "always fit range",
        doc: "Live view only: always fit the palette to the escape range (method \"smooth\" or \"trap\").",
        access: Access::Flag(|s| s.normalize_fit, |s, v| s.normalize_fit = v),
        parents: &["method"],
        shows: uses_fit,
    },
    Field {
        key: "log_scale",
        name: "log color scale",
        doc: "Log color scale wherever the palette is normalized (method \"smooth\" or \"trap\").",
        access: Access::Flag(|s| s.log_palette, |s, v| s.log_palette = v),
        parents: &["method"],
        shows: uses_fit,
    },
    Field {
        key: "transfer",
        name: "color scale",
        doc: "The curve the palette is spread through wherever it is normalized (method \"smooth\" or \"trap\"): linear, sqrt, cbrt, root4, loglog or square; log is written `log_scale = true`, as older files have it (and `transfer = \"log\"` reads as log).",
        access: Access::Choice(|s| s.transfer.as_str(), |s, v| s.transfer = v.to_string(), transfer_keys),
        parents: &["method"],
        shows: uses_fit,
    },
    Field {
        key: "equalize",
        name: "equalize colors",
        doc: "Equalize colors, 0 to 1: the share of histogram coloring mixed into the palette map wherever it is normalized, so each color takes about the same area of the picture (method \"smooth\", and \"trap\" with normalize_fit; 0: the fitted map alone).",
        access: Access::Real(
            |s| s.equalize,
            |s, v| s.equalize = v,
            *slider_range::EQUALIZE.start(),
            *slider_range::EQUALIZE.end(),
        ),
        parents: &["method"],
        shows: uses_fit,
    },
    Field {
        key: "wrap",
        name: "palette wrap",
        doc: "How the palette continues past its end: repeat, mirror (run back the other way) or clamp (the last color held).",
        access: Access::Choice(|s| s.wrap.as_str(), |s, v| s.wrap = v.to_string(), wrap_keys),
        parents: &[],
        shows: always,
    },
    Field {
        key: "steps",
        name: "palette steps",
        doc: "Flat bands each palette cycle is cut into, each the color at its start (0: smooth; at most 64).",
        access: Access::Count(|s| s.steps, |s, v| s.steps = v, 0, fractadyne_core::coloring::MAX_STEPS),
        parents: &[],
        shows: always,
    },
    Field {
        key: "palette_aa",
        name: "palette anti-aliasing",
        doc: "Anti-alias the palette: where its bands are finer than a pixel, the pixel shows their average color, over a footprint sized by the distance to the set (where the formula has none, by the next pixels' colors).",
        access: Access::Flag(|s| s.palette_aa, |s, v| s.palette_aa = v),
        parents: &[],
        shows: always,
    },
    Field {
        key: "palette_brightness",
        name: "palette brightness",
        doc: "The palette's brightness, -1 to 1 (0: as the palette is): its midtones lighter or darker, black and white kept.",
        access: Access::Real(
            |s| s.palette_brightness,
            |s, v| s.palette_brightness = v,
            *slider_range::PALETTE_BRIGHTNESS.start(),
            *slider_range::PALETTE_BRIGHTNESS.end(),
        ),
        parents: &[],
        shows: always,
    },
    Field {
        key: "palette_contrast",
        name: "palette contrast",
        doc: "The palette's contrast, -1 to 1 (0: as the palette is): its lightness spread from the middle or drawn toward it, black and white kept.",
        access: Access::Real(
            |s| s.palette_contrast,
            |s, v| s.palette_contrast = v,
            *slider_range::PALETTE_CONTRAST.start(),
            *slider_range::PALETTE_CONTRAST.end(),
        ),
        parents: &[],
        shows: always,
    },
    Field {
        key: "palette_saturation",
        name: "palette saturation",
        doc: "The palette's saturation, 0 to 2 (1: as the palette is; 0: gray), hue and lightness kept.",
        access: Access::Real(
            |s| s.palette_saturation,
            |s, v| s.palette_saturation = v,
            *slider_range::PALETTE_SATURATION.start(),
            *slider_range::PALETTE_SATURATION.end(),
        ),
        parents: &[],
        shows: always,
    },
    Field {
        key: "palette_oklab",
        name: "palette blended in OKLab",
        doc: "Blend in OKLab: the palette's colors blended in OKLab, a perceptual space, rather than between their display (sRGB) values, so each blend steps evenly in lightness and one between two distant colors does not darken or gray in the middle (HSV segments and flat bands as they are).",
        access: Access::Flag(|s| s.palette_oklab, |s, v| s.palette_oklab = v),
        parents: &[],
        shows: always,
    },
    Field {
        key: "julia_linked",
        name: "linked Julia colors",
        doc: "Dual view: color the Julia pane with the set's palette map, one iteration on, so the Julia point z = c shows the color of the set's point c (method \"smooth\").",
        access: Access::Flag(|s| s.julia_linked, |s, v| s.julia_linked = v),
        parents: &["method"],
        shows: escape_count,
    },
];

/// Every key the table writes and reads, in its order. `export.rs`'s reader lists them too
/// (`KNOWN_VIEW_KEYS`, `TYPED_VIEW_KEYS`), and a unit test holds the lists together.
pub(crate) const KEYS: [&str; FIELDS.len()] = {
    let mut k = [""; FIELDS.len()];
    let mut i = 0;
    while i < FIELDS.len() {
        k[i] = FIELDS[i].key;
        i += 1;
    }
    k
};

impl Access {
    /// The setting as a view writes it.
    fn text(&self, s: &ColoringSpec) -> String {
        match *self {
            Access::Flag(get, _) => if get(s) { "1" } else { "0" }.to_string(),
            Access::Real(get, ..) => get(s).to_string(),
            Access::Count(get, ..) => get(s).to_string(),
            Access::Choice(get, ..) => get(s).to_string(),
            Access::Rgb(get, _) => {
                let c = get(s);
                format!("{},{},{}", c[0], c[1], c[2])
            }
        }
    }

    /// Whether `a` and `b` hold different values of this setting.
    fn differs(&self, a: &ColoringSpec, b: &ColoringSpec) -> bool {
        match *self {
            Access::Flag(get, _) => get(a) != get(b),
            Access::Real(get, ..) => get(a) != get(b),
            Access::Count(get, ..) => get(a) != get(b),
            Access::Choice(get, ..) => get(a) != get(b),
            Access::Rgb(get, _) => get(a) != get(b),
        }
    }

    /// Copy this setting from `from` into `to`.
    fn copy(&self, from: &ColoringSpec, to: &mut ColoringSpec) {
        match *self {
            Access::Flag(get, set) => set(to, get(from)),
            Access::Real(get, set, ..) => set(to, get(from)),
            Access::Count(get, set, ..) => set(to, get(from)),
            Access::Choice(get, set, _) => {
                let v = get(from).to_string();
                set(to, &v)
            }
            Access::Rgb(get, set) => set(to, get(from)),
        }
    }

    /// Read `text` into `to`: `Some(true)` when it was usable (clamped into range if it had to
    /// be, and named in `clamped` then), `Some(false)` when it was not (a number that is not
    /// finite is named too, as the cycle's reader does), and the setting is left alone.
    fn read(&self, text: &str, name: &'static str, to: &mut ColoringSpec, clamped: &mut Vec<&'static str>) -> bool {
        match *self {
            Access::Flag(_, set) => match text {
                "1" => {
                    set(to, true);
                    true
                }
                "0" => {
                    set(to, false);
                    true
                }
                _ => false,
            },
            Access::Real(_, set, lo, hi) => match text.parse::<f32>() {
                Ok(x) if x.is_finite() => {
                    let c = x.clamp(lo, hi);
                    if c != x {
                        clamped.push(name);
                    }
                    set(to, c);
                    true
                }
                Ok(_) => {
                    clamped.push(name);
                    false
                }
                Err(_) => false,
            },
            Access::Count(_, set, lo, hi) => match text.parse::<u64>() {
                Ok(n) => {
                    let c = n.clamp(u64::from(lo), u64::from(hi)) as u32;
                    if u64::from(c) != n {
                        clamped.push(name);
                    }
                    set(to, c);
                    true
                }
                Err(_) => false,
            },
            Access::Choice(_, set, choices) => {
                if choices().contains(&text) {
                    set(to, text);
                    true
                } else {
                    false
                }
            }
            Access::Rgb(_, set) => {
                let parts: Vec<Option<f32>> =
                    text.split(',').map(|t| t.trim().parse::<f32>().ok().filter(|x| x.is_finite())).collect();
                match parts.as_slice() {
                    [Some(r), Some(g), Some(b)] => {
                        let c = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
                        if c != [*r, *g, *b] {
                            clamped.push(name);
                        }
                        set(to, c);
                        true
                    }
                    _ => false,
                }
            }
        }
    }
}

/// Read one key of the table into `spec`: `None` when the table has no such key, else whether the
/// value was usable (clamped into range if it had to be, and named in `clamped` then).
pub(crate) fn read_key(spec: &mut ColoringSpec, key: &str, text: &str, clamped: &mut Vec<&'static str>) -> Option<bool> {
    FIELDS.iter().find(|f| f.key == key).map(|f| f.access.read(text, f.name, spec, clamped))
}

/// A TOML value as the text a row reads: a flag from `true`/`false`, a number as written, a choice
/// from a string, three numbers `[r, g, b]` as `r,g,b`. `None` for any other shape.
pub(crate) fn toml_text(v: &toml::Value) -> Option<String> {
    match v {
        toml::Value::Boolean(b) => Some(if *b { "1" } else { "0" }.to_string()),
        toml::Value::Integer(i) => Some(i.to_string()),
        toml::Value::Float(x) => Some(x.to_string()),
        toml::Value::String(t) => Some(t.clone()),
        toml::Value::Array(a) if a.len() == 3 => {
            let n: Vec<String> = a
                .iter()
                .filter_map(|x| match x {
                    toml::Value::Integer(i) => Some(i.to_string()),
                    toml::Value::Float(f) => Some(f.to_string()),
                    _ => None,
                })
                .collect();
            (n.len() == 3).then(|| n.join(","))
        }
        _ => None,
    }
}

impl Field {
    /// Its type as a tour writes it, for the schema.
    pub(crate) fn tour_type(&self) -> String {
        match self.access {
            Access::Flag(..) => "bool".to_string(),
            Access::Real(_, _, lo, _) if lo == f32::MIN => "float".to_string(),
            Access::Real(_, _, lo, hi) => format!("float, {lo} to {hi}"),
            Access::Count(_, _, lo, hi) => format!("int, {lo} to {hi}"),
            // The palettes' ids are too many to list in a table cell (the interior palette, B5).
            Access::Choice(_, _, choices) if choices().iter().copied().eq(fractadyne_color::PRESETS.iter().map(|p| p.id)) => {
                format!("string: a palette id, such as \"{}\"", fractadyne_color::PRESETS[0].id)
            }
            Access::Choice(_, _, choices) => format!("string: {}", choices().join(" \\| ")),
            Access::Rgb(..) => "[r, g, b], 0 to 1".to_string(),
        }
    }

    /// Its default as a tour writes it, for the schema.
    pub(crate) fn tour_default(&self) -> String {
        let d = ColoringSpec::default();
        match self.access {
            Access::Flag(get, _) => get(&d).to_string(),
            // A default no file writes (Duotone or Binary's "off") is the key left out.
            Access::Choice(get, _, choices) => {
                let v = get(&d);
                if choices().contains(&v) { format!("\"{v}\"") } else { "(left out)".to_string() }
            }
            Access::Rgb(get, _) => {
                let c = get(&d);
                format!("[{}, {}, {}]", c[0], c[1], c[2])
            }
            _ => self.access.text(&d),
        }
    }
}

/// The view lines for this coloring: each setting the picture depends on that differs from the
/// default, one `key=value` per line, in the table's order. Empty for the default coloring.
pub(crate) fn view_lines(spec: &ColoringSpec) -> String {
    let d = ColoringSpec::default();
    let mut out = String::new();
    for f in &FIELDS {
        if (f.shows)(spec) && f.access.differs(spec, &d) {
            out.push_str(f.key);
            out.push('=');
            out.push_str(&f.access.text(spec));
            out.push('\n');
        }
    }
    out
}

/// Hold a coloring that did not come from a view — the session file — to the rules a view is read
/// by: each setting the table has that would not be accepted from a view (not finite, out of its
/// range, not one of its choices) is clamped into range, or put back to its default, and named in
/// `fixed`. Review 2026-11-16 (finding 9): the session restore took these as written, so a value a
/// view could never set (a NaN light angle) survived every restart.
pub(crate) fn sanitize(spec: &mut ColoringSpec, fixed: &mut Vec<&'static str>) {
    let d = ColoringSpec::default();
    for f in &FIELDS {
        let text = f.access.text(spec);
        let mut clamped = Vec::new();
        let read = f.access.read(&text, f.name, spec, &mut clamped);
        // A choice whose default is no choice at all (`interior` empty = the method's own) is not
        // "unreadable" when it holds that default.
        if !read && f.access.differs(spec, &d) {
            f.access.copy(&d, spec);
            fixed.push(f.name);
        } else if !clamped.is_empty() {
            fixed.push(f.name);
        }
    }
}

/// Apply a view's coloring keys to `spec`, this session's coloring, by the rules at the top of this
/// module. `get` reads a key (`None` when the file does not have it); `states` says the file states
/// its coloring (it has `cycle=`). A value out of range is clamped and named in `clamped`; an
/// unreadable one keeps the current setting (the view reader's type table reports it with its line).
pub(crate) fn apply_view(spec: &mut ColoringSpec, get: &dyn Fn(&str) -> Option<String>, states: bool, clamped: &mut Vec<&'static str>) {
    let d = ColoringSpec::default();
    // The rows this file set (to its value, or back to the default).
    let mut settled: Vec<&'static str> = Vec::new();
    for f in &FIELDS {
        let read = match get(f.key) {
            Some(text) => f.access.read(&text, f.name, spec, clamped),
            None => {
                let reset = if f.parents.is_empty() {
                    states
                } else {
                    f.parents.iter().all(|p| settled.contains(p)) && (f.shows)(spec)
                };
                if reset {
                    f.access.copy(&d, spec);
                }
                reset
            }
        };
        if read {
            settled.push(f.key);
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/coloring_fields/tests.rs"]
mod tests;
