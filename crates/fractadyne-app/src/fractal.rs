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
//! 5. **Capabilities** (`fractadyne_core::formula::caps`): series approximation, BLA, resumable
//!    passes, the finders and the export glitch-correction policy. Code asks [`FractalKind::caps`],
//!    never the id, so a new family only needs its row there.

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
    // The power families (design/power-families.md): ids 10–24, `formula::family`.
    Multibrot6,
    Multibrot7,
    Multibrot8,
    BurningShip3,
    BurningShip4,
    BurningShip5,
    Tricorn3,
    Tricorn4,
    Tricorn5,
    Celtic3,
    Celtic4,
    Celtic5,
    Buffalo3,
    Buffalo4,
    Buffalo5,
    /// The session's custom formula (`FractadyneApp::custom`, design/custom-formulas.md): its step
    /// comes from a generated shader module. Outside [`FractalKind::ALL`], which lists the built-in
    /// escape-time families.
    Custom,
    /// A Life-like cellular automaton (`FractadyneApp::life`, design/automata.md): not escape time —
    /// [`FractalClass::Life`]. Outside [`FractalKind::ALL`]; the pickers list it under
    /// [`FractalKind::AUTOMATA`].
    Life,
    /// An L-system (`FractadyneApp::lsystem`, design/lsystems.md): turtle-drawn segments —
    /// [`FractalClass::LSystem`]. Outside [`FractalKind::ALL`]; the pickers list it under
    /// [`FractalKind::LSYSTEMS`].
    LSystem,
}

/// What kind of picture a family makes (design/automata.md §3). Everything that only makes sense
/// for an escape-time formula — iterations, perturbation, Julia, the finders, the autopilot — asks
/// this before it applies.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum FractalClass {
    /// Iterate a formula per pixel: every family but the automata, `Custom` included.
    EscapeTime,
    /// A Life-like cellular automaton on the GPU tile stepper.
    Life,
    /// An L-system: a grammar's turtle drawing, walked for the view and drawn as segments.
    LSystem,
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
    /// Escape time, or an automaton.
    pub(crate) class: FractalClass,
    pub(crate) info: FractalInfo,
}

const MULTIBROT_REF: &str = "https://en.wikipedia.org/wiki/Multibrot_set";
const SHIP_REF: &str = "https://en.wikipedia.org/wiki/Burning_Ship_fractal";
const TRICORN_REF: &str = "https://en.wikipedia.org/wiki/Tricorn_(mathematics)";
const CELTIC_REF: &str = "https://paulbourke.net/fractals/burnship/";

/// A power family's row: deep zoom and a Julia form, as every one has.
const fn power_spec(
    kind: FractalKind,
    name: &'static str,
    formula_id: u32,
    default_center: (f64, f64),
    formula: &'static str,
    about: &'static str,
    reference: &'static str,
) -> FractalSpec {
    FractalSpec {
        kind,
        name,
        formula_id,
        default_center,
        supports_julia: true,
        supports_perturbation: true,
        class: FractalClass::EscapeTime,
        info: FractalInfo { formula, about, reference },
    }
}

impl FractalKind {
    /// The BUILT-IN families, in order: what the pickers list and the benchmarks sweep.
    /// [`FractalKind::Custom`] is not among them — it has no step of its own until the user writes one.
    pub(crate) const ALL: [FractalKind; 25] = [
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
        FractalKind::Multibrot6,
        FractalKind::Multibrot7,
        FractalKind::Multibrot8,
        FractalKind::BurningShip3,
        FractalKind::BurningShip4,
        FractalKind::BurningShip5,
        FractalKind::Tricorn3,
        FractalKind::Tricorn4,
        FractalKind::Tricorn5,
        FractalKind::Celtic3,
        FractalKind::Celtic4,
        FractalKind::Celtic5,
        FractalKind::Buffalo3,
        FractalKind::Buffalo4,
        FractalKind::Buffalo5,
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
            class: FractalClass::EscapeTime,
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
            class: FractalClass::EscapeTime,
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
            class: FractalClass::EscapeTime,
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
            class: FractalClass::EscapeTime,
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
            class: FractalClass::EscapeTime,
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
            class: FractalClass::EscapeTime,
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
            class: FractalClass::EscapeTime,
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
            class: FractalClass::EscapeTime,
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
            class: FractalClass::EscapeTime,
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
            class: FractalClass::EscapeTime,
            info: FractalInfo {
                formula: "z -> z - (z^3 - 1)/(3 z^2)",
                about: "Newton's root-finding iteration for z^3 = 1, colored by how fast \
                        each point converges. A convergence (not escape) fractal.",
                reference: "https://en.wikipedia.org/wiki/Newton_fractal",
            },
        },
        // The power families (design/power-families.md): every one deep-zooms and has a Julia form.
        power_spec(FractalKind::Multibrot6, "Multibrot 6", 10, (0.0, 0.0), "z -> z^6 + c",
            "Multibrot at power 6: fivefold symmetry.", MULTIBROT_REF),
        power_spec(FractalKind::Multibrot7, "Multibrot 7", 11, (0.0, 0.0), "z -> z^7 + c",
            "Multibrot at power 7: sixfold symmetry, the bulbs thinning towards a circle.", MULTIBROT_REF),
        power_spec(FractalKind::Multibrot8, "Multibrot 8", 12, (0.0, 0.0), "z -> z^8 + c",
            "Multibrot at power 8: sevenfold symmetry.", MULTIBROT_REF),
        power_spec(FractalKind::BurningShip3, "Burning Ship 3", 13, (0.0, 0.0), "z -> (|Re z| + i|Im z|)^3 + c",
            "The Burning Ship at power 3: the absolute values are taken, then cubed.", SHIP_REF),
        power_spec(FractalKind::BurningShip4, "Burning Ship 4", 14, (0.0, 0.0), "z -> (|Re z| + i|Im z|)^4 + c",
            "The Burning Ship at power 4.", SHIP_REF),
        power_spec(FractalKind::BurningShip5, "Burning Ship 5", 15, (0.0, 0.0), "z -> (|Re z| + i|Im z|)^5 + c",
            "The Burning Ship at power 5.", SHIP_REF),
        power_spec(FractalKind::Tricorn3, "Tricorn 3", 16, (0.0, 0.0), "z -> conj(z)^3 + c",
            "The Tricorn (Mandelbar) at power 3: fourfold symmetry.", TRICORN_REF),
        power_spec(FractalKind::Tricorn4, "Tricorn 4", 17, (0.0, 0.0), "z -> conj(z)^4 + c",
            "The Tricorn at power 4: fivefold symmetry.", TRICORN_REF),
        power_spec(FractalKind::Tricorn5, "Tricorn 5", 18, (0.0, 0.0), "z -> conj(z)^5 + c",
            "The Tricorn at power 5: sixfold symmetry.", TRICORN_REF),
        power_spec(FractalKind::Celtic3, "Celtic 3", 19, (0.0, 0.0), "Re -> |Re(z^3)| + cx;  Im -> Im(z^3) + cy",
            "The Celtic fold at power 3: the absolute value of the real part of z^3.", CELTIC_REF),
        power_spec(FractalKind::Celtic4, "Celtic 4", 20, (0.0, 0.0), "Re -> |Re(z^4)| + cx;  Im -> Im(z^4) + cy",
            "The Celtic fold at power 4.", CELTIC_REF),
        power_spec(FractalKind::Celtic5, "Celtic 5", 21, (0.0, 0.0), "Re -> |Re(z^5)| + cx;  Im -> Im(z^5) + cy",
            "The Celtic fold at power 5.", CELTIC_REF),
        power_spec(FractalKind::Buffalo3, "Buffalo 3", 22, (0.0, 0.0), "Re -> |Re(z^3)| + cx;  Im -> |Im(z^3)| + cy",
            "The Buffalo at power 3: the absolute values of both parts of z^3.", CELTIC_REF),
        power_spec(FractalKind::Buffalo4, "Buffalo 4", 23, (0.0, 0.0), "Re -> |Re(z^4)| + cx;  Im -> |Im(z^4)| + cy",
            "The Buffalo at power 4.", CELTIC_REF),
        power_spec(FractalKind::Buffalo5, "Buffalo 5", 24, (0.0, 0.0), "Re -> |Re(z^5)| + cx;  Im -> |Im(z^5)| + cy",
            "The Buffalo at power 5.", CELTIC_REF),
        // The one row whose id is not its index: a custom formula renders under
        // `formula::CUSTOM`, which no built-in branch of the shader matches.
        FractalSpec {
            kind: FractalKind::Custom,
            name: "Custom",
            formula_id: fractadyne_core::formula::CUSTOM,
            default_center: (-0.5, 0.0),
            supports_julia: true,
            supports_perturbation: false,
            class: FractalClass::EscapeTime,
            info: FractalInfo {
                formula: "z -> the formula you write",
                about: "A formula written in Fractint-style expressions (Fractal > Custom \
                        formula...). It renders on the direct path, so it has no deep zoom yet.",
                reference: "",
            },
        },
        // The automata (design/automata.md): ids outside the escape-time range, like Custom's.
        FractalSpec {
            kind: FractalKind::Life,
            name: "Life",
            formula_id: fractadyne_core::formula::LIFE,
            // The middle of cell (0, 0): cells are unit squares, rows growing downward.
            default_center: (0.5, -0.5),
            supports_julia: false,
            supports_perturbation: false,
            class: FractalClass::Life,
            info: FractalInfo {
                formula: "B3/S23: born with 3 live neighbours, survives with 2 or 3",
                about: "Conway's Game of Life and its relatives on an unbounded plane: Generations \
                        rules, non-totalistic (Hensel) rules, pattern files. Play, step, draw.",
                reference: "https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life",
            },
        },
        // L-systems (design/lsystems.md): an id outside the escape-time range too.
        FractalSpec {
            kind: FractalKind::LSystem,
            name: "L-system",
            formula_id: fractadyne_core::formula::LSYSTEM,
            default_center: (0.0, 0.0),
            supports_julia: false,
            supports_perturbation: false,
            class: FractalClass::LSystem,
            info: FractalInfo {
                formula: "F = F+F--F+F: each symbol rewritten, then drawn by a turtle",
                about: "Lindenmayer systems: curves, space-filling curves, islands and plants drawn \
                        by a turtle from a rewriting grammar. The order follows the zoom, so the \
                        detail never runs out. Fractint .l files open directly.",
                reference: "https://en.wikipedia.org/wiki/L-system",
            },
        },
    ];

    /// The automata, as the pickers list them (after the escape-time families).
    pub(crate) const AUTOMATA: [FractalKind; 1] = [FractalKind::Life];

    /// The L-systems, as the pickers list them (after the automata).
    pub(crate) const LSYSTEMS: [FractalKind; 1] = [FractalKind::LSystem];

    /// This family's metadata row. O(1): rows are ordered to match the enum (test-enforced).
    pub(crate) fn spec(self) -> &'static FractalSpec {
        &Self::SPECS[self as usize]
    }

    /// Escape time, or an automaton.
    pub(crate) fn class(self) -> FractalClass {
        self.spec().class
    }

    /// Whether this is an escape-time family (everything but the automata).
    pub(crate) fn is_escape_time(self) -> bool {
        self.class() == FractalClass::EscapeTime
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

    /// The power families by shape, each row's kinds in power order: what the pickers show as one
    /// line per family ("Burning Ship  3 4 5") rather than fifteen more rows.
    pub(crate) const POWER_GROUPS: [(&'static str, [FractalKind; 3]); 5] = [
        ("Multibrot", [FractalKind::Multibrot6, FractalKind::Multibrot7, FractalKind::Multibrot8]),
        ("Burning Ship", [FractalKind::BurningShip3, FractalKind::BurningShip4, FractalKind::BurningShip5]),
        ("Tricorn", [FractalKind::Tricorn3, FractalKind::Tricorn4, FractalKind::Tricorn5]),
        ("Celtic", [FractalKind::Celtic3, FractalKind::Celtic4, FractalKind::Celtic5]),
        ("Buffalo", [FractalKind::Buffalo3, FractalKind::Buffalo4, FractalKind::Buffalo5]),
    ];

    /// Whether this is one of the power families (listed by [`Self::POWER_GROUPS`]), and its power.
    pub(crate) fn power_family(self) -> Option<u32> {
        fractadyne_core::formula::family(self.formula_id()).map(|(_, d)| d)
    }

    /// What this family supports beyond plain iteration (series approximation, BLA, resumable
    /// passes, the finders, the export glitch-correction policy). Ask this, not the id.
    pub(crate) fn caps(self) -> fractadyne_core::formula::FormulaCaps {
        fractadyne_core::formula::caps(self.formula_id())
    }

    /// Hover text for a formula picker: the iteration, what the family looks like, and
    /// - where it applies - that it cannot deep zoom. Derived from [`Self::info`] rather
    /// than written out again, so a new family gets its tooltip from its `SPECS` row and
    /// the two can never disagree.
    pub(crate) fn menu_hint(self) -> String {
        let info = self.info();
        let mut s = format!("{}\n\n{}", info.formula, info.about);
        if !self.supports_perturbation() {
            // The one difference a picker cannot show but a user feels immediately.
            s.push_str("\n\nDirect rendering only - this family has no deep zoom.");
        }
        s
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
            FractalKind::ALL.len() + 1 + FractalKind::AUTOMATA.len() + FractalKind::LSYSTEMS.len(),
            "every FractalKind needs exactly one SPECS row (the built-ins, Custom, the automata, the L-systems)"
        );
        let custom = &FractalKind::SPECS[FractalKind::ALL.len()];
        assert_eq!(custom.kind, FractalKind::Custom);
        assert_eq!(FractalKind::Custom as usize, FractalKind::ALL.len(), "spec() indexes by variant");
        assert_eq!(custom.formula_id, fractadyne_core::formula::CUSTOM);
        assert!(!custom.supports_perturbation, "a custom formula renders on the direct path");
        assert_eq!(custom.class, FractalClass::EscapeTime);
        for (k, kind) in FractalKind::AUTOMATA.iter().chain(&FractalKind::LSYSTEMS).enumerate() {
            let spec = &FractalKind::SPECS[FractalKind::ALL.len() + 1 + k];
            assert_eq!(spec.kind, *kind, "the automata's then the L-systems' rows follow Custom's, in list order");
            assert_eq!(*kind as usize, FractalKind::ALL.len() + 1 + k);
            assert_ne!(spec.class, FractalClass::EscapeTime);
            assert!(!spec.supports_julia && !spec.supports_perturbation);
            assert!(!fractadyne_core::is_valid_formula(spec.formula_id), "outside the escape-time ids");
        }
        assert_eq!(FractalKind::Life.formula_id(), fractadyne_core::formula::LIFE);
        assert_eq!(FractalKind::LSystem.formula_id(), fractadyne_core::formula::LSYSTEM);
        assert_eq!(FractalKind::LSystem.class(), FractalClass::LSystem);
        for (i, kind) in FractalKind::ALL.iter().enumerate() {
            let spec = &FractalKind::SPECS[i];
            assert_eq!(spec.kind, *kind, "SPECS row {i} is out of declaration order");
            assert_eq!(spec.class, FractalClass::EscapeTime, "{}", spec.name);
            assert_eq!(
                spec.formula_id as usize, i,
                "{}: formula_id must equal its index",
                spec.name
            );
        }
    }

    /// A blank `formula` or `about` would ship as an empty tooltip, which reads as a broken
    /// menu rather than a missing sentence. Guard the whole table at once.
    #[test]
    fn every_family_has_a_hint_worth_showing() {
        for kind in FractalKind::ALL {
            let info = kind.info();
            assert!(!info.formula.trim().is_empty(), "{}: no formula", kind.name());
            assert!(!info.about.trim().is_empty(), "{}: no description", kind.name());
            let hint = kind.menu_hint();
            assert!(hint.contains(info.formula), "{}: hint drops the formula", kind.name());
            assert!(hint.contains(info.about), "{}: hint drops the description", kind.name());
        }
    }

    /// Newton is the one family with no perturbation path, and this hint is the only place
    /// the interface says so. Pinned BY NAME on purpose: asserting the hint against
    /// `supports_perturbation` looks stronger but is a tautology, since the hint is derived
    /// from that same flag - flip the flag and both sides move together and the test still
    /// passes. (Tried it; it did.)
    #[test]
    fn only_the_family_without_deep_zoom_says_so() {
        assert!(FractalKind::Newton.menu_hint().contains("no deep zoom"));
        assert!(!FractalKind::Mandelbrot.menu_hint().contains("no deep zoom"));
    }

    /// The pickers list every power family through `POWER_GROUPS` and every other one by itself:
    /// together, each built-in exactly once.
    #[test]
    fn the_power_groups_list_each_power_family_once() {
        let grouped: Vec<FractalKind> = FractalKind::POWER_GROUPS.iter().flat_map(|(_, ks)| *ks).collect();
        let singles = FractalKind::ALL.iter().filter(|k| k.power_family().is_none()).count();
        assert_eq!(grouped.len() + singles, FractalKind::ALL.len());
        for (label, kinds) in FractalKind::POWER_GROUPS {
            for k in kinds {
                let d = k.power_family().expect("a power family");
                assert_eq!(k.name(), format!("{label} {d}"), "the row's label and power name the family");
                assert_eq!(fractadyne_core::formula::power(k.formula_id()), d);
            }
        }
    }

    /// Names are used as stable tokens in view files; they must round-trip and be unique.
    #[test]
    fn names_round_trip_and_are_unique() {
        for k in FractalKind::ALL.into_iter().chain([FractalKind::Custom]).chain(FractalKind::AUTOMATA).chain(FractalKind::LSYSTEMS) {
            assert_eq!(FractalKind::from_name(k.name()), Some(k));
        }
        let mut names: Vec<&str> = FractalKind::SPECS.iter().map(|s| s.name).collect();
        names.sort_unstable();
        let n = names.len();
        names.dedup();
        assert_eq!(names.len(), n, "duplicate fractal name");
    }
}
