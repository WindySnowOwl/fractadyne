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

#[cfg(test)]
mod tests {
    use super::*;

    /// Each packed slot is `[r, g, b, pos]` — the order the GPU uniform expects.
    #[test]
    fn packed_slot_is_rgb_then_pos() {
        let p = Palette { name: "t", stops: &[(0.0, [0.1, 0.2, 0.3]), (1.0, [0.4, 0.5, 0.6])] };
        let (out, n) = p.packed();
        assert_eq!(n, 2);
        assert_eq!(out[0], [0.1, 0.2, 0.3, 0.0]);
        assert_eq!(out[1], [0.4, 0.5, 0.6, 1.0]);
    }

    /// Unused trailing slots repeat the last real stop (so the shader's `stop_count` bound and the
    /// padded array agree — reading past `n` still yields the terminal color, never garbage).
    #[test]
    fn trailing_slots_repeat_last_stop() {
        let p = Palette { name: "t", stops: &[(0.0, [0.0; 3]), (0.5, [1.0; 3]), (1.0, [0.2; 3])] };
        let (out, n) = p.packed();
        assert_eq!(n, 3);
        assert_eq!(out[2], [0.2, 0.2, 0.2, 1.0]);
        for slot in &out[n as usize..] {
            assert_eq!(*slot, out[2]);
        }
    }

    /// A single-stop palette fills every slot with that stop (count 1, no out-of-bounds).
    #[test]
    fn single_stop_fills_all_slots() {
        let p = Palette { name: "t", stops: &[(0.3, [0.7, 0.8, 0.9])] };
        let (out, n) = p.packed();
        assert_eq!(n, 1);
        assert!(out.iter().all(|s| *s == [0.7, 0.8, 0.9, 0.3]));
    }

    /// More stops than the GPU carries: the count saturates at `MAX_STOPS` and the first
    /// `MAX_STOPS` stops are kept (no panic, no overflow).
    #[test]
    fn count_clamps_to_max_stops() {
        let p = Palette {
            name: "t",
            stops: &[
                (0.0, [0.0; 3]), (0.1, [1.0; 3]), (0.2, [2.0; 3]), (0.3, [3.0; 3]),
                (0.4, [4.0; 3]), (0.5, [5.0; 3]), (0.6, [6.0; 3]), (0.7, [7.0; 3]),
                (0.8, [8.0; 3]), (0.9, [9.0; 3]), (1.0, [10.0; 3]),
            ],
        };
        let (out, n) = p.packed();
        assert_eq!(n as usize, MAX_STOPS);
        assert_eq!(out[MAX_STOPS - 1], [7.0, 7.0, 7.0, 0.7]);
    }

    /// Every shipped preset packs within bounds, fits in `MAX_STOPS`, and keeps its first stop.
    #[test]
    fn presets_pack_within_bounds() {
        for p in PRESETS {
            let (out, n) = p.packed();
            assert!((1..=MAX_STOPS as u32).contains(&n), "{}: count {n} out of range", p.name);
            assert_eq!(n as usize, p.stops.len(), "{}: all presets must fit in MAX_STOPS", p.name);
            let (pos, c) = p.stops[0];
            assert_eq!(out[0], [c[0], c[1], c[2], pos], "{}: first slot", p.name);
        }
    }
}
