//! Palettes and coloring for Fractadyne (M1).
//!
//! A palette is a small list of gradient stops in `0..1`. The renderer interpolates
//! between them in-shader, cycled by the smooth iteration value. Stops are chosen
//! to loop seamlessly (first and last colors match). A data-driven LUT / custom
//! gradient editor comes later (UI-DESIGN.md §6, §8); these presets are the start.

/// Max stops the GPU uniform carries (must match `fractadyne-gpu`).
pub const MAX_STOPS: usize = 8;

/// A named gradient palette: ascending `(position 0..1, linear RGB 0..1)` stops.
pub struct Palette {
    pub name: &'static str,
    pub stops: &'static [(f32, [f32; 3])],
}

impl Palette {
    /// Pack into a fixed `[[r,g,b,pos]; MAX_STOPS]` array plus the active count, for
    /// GPU upload. Unused trailing slots repeat the last stop.
    pub fn packed(&self) -> ([[f32; 4]; MAX_STOPS], u32) {
        let mut out = [[0.0f32; 4]; MAX_STOPS];
        let n = self.stops.len().clamp(1, MAX_STOPS);
        for (i, slot) in out.iter_mut().enumerate() {
            let (pos, c) = self.stops[i.min(n - 1)];
            *slot = [c[0], c[1], c[2], pos];
        }
        (out, n as u32)
    }
}

/// Built-in palettes (names match the design mockups).
pub const PRESETS: &[Palette] = &[
    Palette {
        name: "Ember",
        stops: &[
            (0.00, [0.00, 0.00, 0.00]),
            (0.15, [0.45, 0.02, 0.02]),
            (0.40, [0.90, 0.30, 0.02]),
            (0.62, [1.00, 0.70, 0.12]),
            (0.82, [1.00, 1.00, 0.82]),
            (1.00, [0.00, 0.00, 0.00]),
        ],
    },
    Palette {
        name: "Ice",
        stops: &[
            (0.00, [0.00, 0.02, 0.10]),
            (0.30, [0.00, 0.30, 0.60]),
            (0.55, [0.20, 0.70, 0.92]),
            (0.80, [0.82, 0.95, 1.00]),
            (1.00, [0.00, 0.02, 0.10]),
        ],
    },
    Palette {
        name: "Nebula",
        stops: &[
            (0.00, [0.05, 0.00, 0.10]),
            (0.25, [0.40, 0.00, 0.50]),
            (0.50, [0.90, 0.20, 0.50]),
            (0.70, [0.20, 0.60, 0.70]),
            (0.86, [0.85, 0.92, 0.72]),
            (1.00, [0.05, 0.00, 0.10]),
        ],
    },
    Palette {
        name: "Grayscale",
        stops: &[
            (0.00, [0.00, 0.00, 0.00]),
            (0.50, [1.00, 1.00, 1.00]),
            (1.00, [0.00, 0.00, 0.00]),
        ],
    },
];
