//! The [`FractalKind`] domain enum — the escape-time families the app offers — and a single
//! per-family metadata table ([`FractalKind::SPECS`]) holding each one's display name, shader
//! formula id, default view, deep-zoom support, and human description.
//!
//! # Adding a new formula
//!
//! Everything the *app* knows about a family lives in one [`FractalSpec`] row, so the app side is
//! a single edit. The numeric cores and the shader are separate code paths (they can't share Rust
//! with WGSL), so they each need one arm too. The full checklist:
//!
//! 1. **This file:** add a variant to [`FractalKind`] and [`FractalKind::ALL`], then add one
//!    [`FractalSpec`] row to [`FractalKind::SPECS`] in the *same position* (a test enforces the
//!    order and that `formula_id == index`). That covers name / id / center / julia / perturbation
//!    / info in one place.
//! 2. **CPU numerics** (`fractadyne-core/src/lib.rs`), keyed on the `u32` `formula_id`:
//!    - `step_bf` — the bignum reference-orbit step (required).
//!    - `orbit_points` — the f64 orbit overlay (required).
//!    - `series_skip` — only if the family supports series approximation (polynomial `z^d + c`).
//!    - `formula_power` — the escape power, if the family has one (used by nucleus finding).
//! 3. **Shader** (`fractadyne-gpu/src/mandelbrot.wgsl`): add the branch to `fs_iterate` for each
//!    active render mode it should support — direct df32, df32 perturbation, and/or extended-range
//!    floatexp — matching `formula_id`.
//! 4. If the family is **not** deep-zoom capable, set `supports_perturbation: false`; it then runs
//!    on the direct (shallow) path only and steps 2–3 need only the direct-mode arm.

/// Per-family description shown in the info panel and Help.
#[derive(Clone, Copy)]
pub(crate) struct FractalInfo {
    pub(crate) formula: &'static str,
    pub(crate) about: &'static str,
    pub(crate) reference: &'static str,
}

/// Escape-time fractal families. `formula_id` (see [`FractalKind::SPECS`]) must match the
/// shader's `fs_iterate` in `fractadyne-gpu/src/mandelbrot.wgsl`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum FractalKind {
    Mandelbrot,
    Multibrot3,
    Multibrot4,
    Multibrot5,
    Tricorn,
    BurningShip,
    Celtic,
    Buffalo,
    Phoenix,
    Newton,
}

/// All the app-side metadata for one family, gathered in one place so adding a formula is a
/// single row rather than an edit spread across many `match` arms.
pub(crate) struct FractalSpec {
    pub(crate) kind: FractalKind,
    /// Display name (also the token used in view files / `.fdn`); must stay stable once shipped.
    pub(crate) name: &'static str,
    /// Shader / core dispatch id — MUST equal this row's index (a test enforces it).
    pub(crate) formula_id: u32,
    /// Default view center `(x, y)` when switching to this family.
    pub(crate) default_center: (f64, f64),
    /// Whether a Julia variant is meaningful (needs a parameter `c`).
    pub(crate) supports_julia: bool,
    /// Whether deep zoom (CPU reference + GPU perturbation, df32 and extended-range floatexp) is
    /// implemented. Families without it run on the direct path only (~1e6×).
    pub(crate) supports_perturbation: bool,
    pub(crate) info: FractalInfo,
}

impl FractalKind {
    pub(crate) const ALL: [FractalKind; 10] = [
        FractalKind::Mandelbrot,
        FractalKind::Multibrot3,
        FractalKind::Multibrot4,
        FractalKind::Multibrot5,
        FractalKind::Tricorn,
        FractalKind::BurningShip,
        FractalKind::Celtic,
        FractalKind::Buffalo,
        FractalKind::Phoenix,
        FractalKind::Newton,
    ];

    /// The single source of truth for every family's app-side metadata. Row order MUST match the
    /// `FractalKind` declaration order and each `formula_id` MUST equal its index — the
    /// `specs_cover_all_kinds_in_order` test enforces both, so `spec()` can index in O(1).
    pub(crate) const SPECS: &'static [FractalSpec] = &[
        FractalSpec {
            kind: FractalKind::Mandelbrot,
            name: "Mandelbrot",
            formula_id: 0,
            default_center: (-0.5, 0.0),
            supports_julia: true,
            supports_perturbation: true,
            info: FractalInfo {
                formula: "z -> z^2 + c    (z0 = 0)",
                about: "The canonical escape-time fractal: the set of c for which the \
                        orbit of 0 stays bounded. Its boundary is infinitely intricate.",
                reference: "https://en.wikipedia.org/wiki/Mandelbrot_set",
            },
        },
        FractalSpec {
            kind: FractalKind::Multibrot3,
            name: "Multibrot 3",
            formula_id: 1,
            default_center: (0.0, 0.0),
            supports_julia: true,
            supports_perturbation: true,
            info: FractalInfo {
                formula: "z -> z^3 + c",
                about: "A Multibrot set - the Mandelbrot construction at a higher power. \
                        Power d gives (d-1)-fold rotational symmetry.",
                reference: "https://en.wikipedia.org/wiki/Multibrot_set",
            },
        },
        FractalSpec {
            kind: FractalKind::Multibrot4,
            name: "Multibrot 4",
            formula_id: 2,
            default_center: (0.0, 0.0),
            supports_julia: true,
            supports_perturbation: true,
            info: FractalInfo {
                formula: "z -> z^4 + c",
                about: "Multibrot at power 4: threefold symmetry, broad bulbs.",
                reference: "https://en.wikipedia.org/wiki/Multibrot_set",
            },
        },
        FractalSpec {
            kind: FractalKind::Multibrot5,
            name: "Multibrot 5",
            formula_id: 3,
            default_center: (0.0, 0.0),
            supports_julia: true,
            supports_perturbation: true,
            info: FractalInfo {
                formula: "z -> z^5 + c",
                about: "Multibrot at power 5: fourfold symmetry.",
                reference: "https://en.wikipedia.org/wiki/Multibrot_set",
            },
        },
        FractalSpec {
            kind: FractalKind::Tricorn,
            name: "Tricorn",
            formula_id: 4,
            default_center: (0.0, 0.0),
            supports_julia: true,
            supports_perturbation: true,
            info: FractalInfo {
                formula: "z -> conj(z)^2 + c",
                about: "The Tricorn (Mandelbar): conjugates z each step. This \
                        anti-holomorphic map yields a three-cornered shape.",
                reference: "https://en.wikipedia.org/wiki/Tricorn_(mathematics)",
            },
        },
        FractalSpec {
            kind: FractalKind::BurningShip,
            name: "Burning Ship",
            formula_id: 5,
            default_center: (-0.5, -0.5),
            supports_julia: true,
            supports_perturbation: true,
            info: FractalInfo {
                formula: "z -> (|Re z| + i|Im z|)^2 + c",
                about: "Absolute values of z's parts are taken before squaring; the \
                        result resembles a ship in flames.",
                reference: "https://en.wikipedia.org/wiki/Burning_Ship_fractal",
            },
        },
        FractalSpec {
            kind: FractalKind::Celtic,
            name: "Celtic",
            formula_id: 6,
            default_center: (-0.5, 0.0),
            supports_julia: true,
            supports_perturbation: true,
            info: FractalInfo {
                formula: "Re -> |Re(z^2)| + cx;  Im -> Im(z^2) + cy",
                about: "A Burning-Ship relative that takes the absolute value of only \
                        the real part of z^2, producing celtic-knot / heart motifs.",
                reference: "https://paulbourke.net/fractals/burnship/",
            },
        },
        FractalSpec {
            kind: FractalKind::Buffalo,
            name: "Buffalo",
            formula_id: 7,
            default_center: (-0.5, -0.5),
            supports_julia: true,
            supports_perturbation: true,
            info: FractalInfo {
                formula: "Re -> |Re(z^2)| + cx;  Im -> |Im(z^2)| + cy",
                about: "An abs-variant taking absolute values of both components of z^2.",
                reference: "https://paulbourke.net/fractals/burnship/",
            },
        },
        FractalSpec {
            kind: FractalKind::Phoenix,
            name: "Phoenix",
            formula_id: 8,
            default_center: (0.0, 0.0),
            supports_julia: true,
            supports_perturbation: true,
            info: FractalInfo {
                formula: "z' = z^2 + c + p*z_prev    (p = -0.5)",
                about: "The Phoenix uses the previous iterate too, giving flame-like \
                        filaments. Try its Julia form via Julia mode.",
                reference: "https://paulbourke.net/fractals/phoenix/",
            },
        },
        FractalSpec {
            kind: FractalKind::Newton,
            name: "Newton",
            formula_id: 9,
            default_center: (0.0, 0.0),
            supports_julia: false,
            supports_perturbation: false,
            info: FractalInfo {
                formula: "z -> z - (z^3 - 1)/(3 z^2)",
                about: "Newton's root-finding iteration for z^3 = 1, colored by how fast \
                        each point converges. A convergence (not escape) fractal.",
                reference: "https://en.wikipedia.org/wiki/Newton_fractal",
            },
        },
    ];

    /// This family's metadata row. O(1): rows are ordered to match the enum (test-enforced).
    pub(crate) fn spec(self) -> &'static FractalSpec {
        &Self::SPECS[self as usize]
    }

    pub(crate) fn name(self) -> &'static str {
        self.spec().name
    }

    pub(crate) fn from_name(name: &str) -> Option<FractalKind> {
        FractalKind::SPECS
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.kind)
    }

    pub(crate) fn formula_id(self) -> u32 {
        self.spec().formula_id
    }

    /// Default view center (x, y) for this fractal.
    pub(crate) fn default_center(self) -> (f64, f64) {
        self.spec().default_center
    }

    /// Whether a Julia variant is meaningful (Newton has no parameter `c`).
    pub(crate) fn supports_julia(self) -> bool {
        self.spec().supports_julia
    }

    /// Whether deep zoom (CPU reference + GPU perturbation, both the df32 and the extended-range
    /// floatexp paths) is implemented. See [`FractalSpec::supports_perturbation`].
    pub(crate) fn supports_perturbation(self) -> bool {
        self.spec().supports_perturbation
    }

    pub(crate) fn info(self) -> FractalInfo {
        self.spec().info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `spec()` O(1) index and `formula_id`-as-index invariant both depend on `SPECS` being in
    /// exact `FractalKind` declaration order. Guard it so a mis-ordered/forgotten row can't ship.
    #[test]
    fn specs_cover_all_kinds_in_order() {
        assert_eq!(
            FractalKind::SPECS.len(),
            FractalKind::ALL.len(),
            "every FractalKind needs exactly one SPECS row"
        );
        for (i, kind) in FractalKind::ALL.iter().enumerate() {
            let spec = &FractalKind::SPECS[i];
            assert_eq!(spec.kind, *kind, "SPECS row {i} is out of declaration order");
            assert_eq!(
                spec.formula_id as usize, i,
                "{}: formula_id must equal its index",
                spec.name
            );
        }
    }

    /// Names are used as stable tokens in view files; they must round-trip and be unique.
    #[test]
    fn names_round_trip_and_are_unique() {
        for k in FractalKind::ALL {
            assert_eq!(FractalKind::from_name(k.name()), Some(k));
        }
        let mut names: Vec<&str> = FractalKind::ALL.iter().map(|k| k.name()).collect();
        names.sort_unstable();
        let n = names.len();
        names.dedup();
        assert_eq!(names.len(), n, "duplicate fractal name");
    }
}
