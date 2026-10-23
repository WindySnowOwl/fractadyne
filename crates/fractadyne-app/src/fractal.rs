//! The [`FractalKind`] domain enum — the escape-time families the app offers — and a single
//! per-family metadata table ([`FractalKind::SPECS`]) holding each one's display name, shader
//! formula id, default view and deep-zoom support; [`FractalKind::about`] gives its description.
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

/// Per-family facts the info panel shows. The description is translated text
/// ([`FractalKind::about`]); `formula` is written in the formula language and stays as it is, and
/// is empty for the families whose "formula" is a sentence ([`FractalKind::formula`]).
#[derive(Clone, Copy)]
pub(crate) struct FractalInfo {
    pub(crate) formula: &'static str,
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
        info: FractalInfo { formula, reference },
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
                reference: "https://en.wikipedia.org/wiki/Newton_fractal",
            },
        },
        // The power families (design/power-families.md): every one deep-zooms and has a Julia form.
        power_spec(FractalKind::Multibrot6, "Multibrot 6", 10, (0.0, 0.0), "z -> z^6 + c", MULTIBROT_REF),
        power_spec(FractalKind::Multibrot7, "Multibrot 7", 11, (0.0, 0.0), "z -> z^7 + c", MULTIBROT_REF),
        power_spec(FractalKind::Multibrot8, "Multibrot 8", 12, (0.0, 0.0), "z -> z^8 + c", MULTIBROT_REF),
        power_spec(FractalKind::BurningShip3, "Burning Ship 3", 13, (0.0, 0.0), "z -> (|Re z| + i|Im z|)^3 + c", SHIP_REF),
        power_spec(FractalKind::BurningShip4, "Burning Ship 4", 14, (0.0, 0.0), "z -> (|Re z| + i|Im z|)^4 + c", SHIP_REF),
        power_spec(FractalKind::BurningShip5, "Burning Ship 5", 15, (0.0, 0.0), "z -> (|Re z| + i|Im z|)^5 + c", SHIP_REF),
        power_spec(FractalKind::Tricorn3, "Tricorn 3", 16, (0.0, 0.0), "z -> conj(z)^3 + c", TRICORN_REF),
        power_spec(FractalKind::Tricorn4, "Tricorn 4", 17, (0.0, 0.0), "z -> conj(z)^4 + c", TRICORN_REF),
        power_spec(FractalKind::Tricorn5, "Tricorn 5", 18, (0.0, 0.0), "z -> conj(z)^5 + c", TRICORN_REF),
        power_spec(FractalKind::Celtic3, "Celtic 3", 19, (0.0, 0.0), "Re -> |Re(z^3)| + cx;  Im -> Im(z^3) + cy", CELTIC_REF),
        power_spec(FractalKind::Celtic4, "Celtic 4", 20, (0.0, 0.0), "Re -> |Re(z^4)| + cx;  Im -> Im(z^4) + cy", CELTIC_REF),
        power_spec(FractalKind::Celtic5, "Celtic 5", 21, (0.0, 0.0), "Re -> |Re(z^5)| + cx;  Im -> Im(z^5) + cy", CELTIC_REF),
        power_spec(FractalKind::Buffalo3, "Buffalo 3", 22, (0.0, 0.0), "Re -> |Re(z^3)| + cx;  Im -> |Im(z^3)| + cy", CELTIC_REF),
        power_spec(FractalKind::Buffalo4, "Buffalo 4", 23, (0.0, 0.0), "Re -> |Re(z^4)| + cx;  Im -> |Im(z^4)| + cy", CELTIC_REF),
        power_spec(FractalKind::Buffalo5, "Buffalo 5", 24, (0.0, 0.0), "Re -> |Re(z^5)| + cx;  Im -> |Im(z^5)| + cy", CELTIC_REF),
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
                formula: "",
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
                formula: "",
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
                formula: "",
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

    /// The family's TOKEN: what sessions, view files, scripts, logs and file names hold. English,
    /// and never translated: the interface shows [`Self::display_name`].
    pub(crate) fn name(self) -> &'static str {
        self.spec().name
    }

    /// The family's name as the interface shows it, in the language in use: "Burning Ship 4".
    pub(crate) fn display_name(self) -> String {
        match self.shown_power() {
            Some(d) => tr!(fractal_with_power, family = self.family_name(), power = d.to_string()),
            None => self.family_name(),
        }
    }

    /// The family without its power ("Burning Ship"), in the language in use: the pickers' row for
    /// a power family, whose powers follow it as buttons.
    pub(crate) fn family_name(self) -> String {
        use FractalKind::*;
        match self {
            Mandelbrot => tr!(fractal_family_mandelbrot),
            Multibrot3 | Multibrot4 | Multibrot5 | Multibrot6 | Multibrot7 | Multibrot8 => tr!(fractal_family_multibrot),
            Tricorn | Tricorn3 | Tricorn4 | Tricorn5 => tr!(fractal_family_tricorn),
            BurningShip | BurningShip3 | BurningShip4 | BurningShip5 => tr!(fractal_family_burning_ship),
            Celtic | Celtic3 | Celtic4 | Celtic5 => tr!(fractal_family_celtic),
            Buffalo | Buffalo3 | Buffalo4 | Buffalo5 => tr!(fractal_family_buffalo),
            Phoenix => tr!(fractal_family_phoenix),
            Newton => tr!(fractal_family_newton),
            Custom => tr!(fractal_family_custom),
            Life => tr!(fractal_family_life),
            LSystem => tr!(fractal_family_lsystem),
        }
    }

    /// The power the name carries ("Multibrot 3"), as the token writes it.
    fn shown_power(self) -> Option<u32> {
        use FractalKind::*;
        match self {
            Multibrot3 | BurningShip3 | Tricorn3 | Celtic3 | Buffalo3 => Some(3),
            Multibrot4 | BurningShip4 | Tricorn4 | Celtic4 | Buffalo4 => Some(4),
            Multibrot5 | BurningShip5 | Tricorn5 | Celtic5 | Buffalo5 => Some(5),
            Multibrot6 => Some(6),
            Multibrot7 => Some(7),
            Multibrot8 => Some(8),
            _ => None,
        }
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
        let mut s = format!("{}\n\n{}", self.formula(), self.about());
        // Not for Custom: whether a formula deep-zooms depends on the formula (the dialog says).
        if !self.supports_perturbation() && self != FractalKind::Custom {
            // The one difference a picker cannot show but a user feels immediately.
            s.push_str("\n\n");
            s.push_str(&tr!(fractal_no_deep_zoom));
        }
        s
    }

    /// The iteration as the info panel and the pickers show it: written in the formula language,
    /// or, for the families whose "formula" is a sentence, that sentence in the language in use.
    pub(crate) fn formula(self) -> String {
        match self {
            FractalKind::Custom => tr!(fractal_custom_formula),
            FractalKind::Life => tr!(fractal_life_formula),
            FractalKind::LSystem => tr!(fractal_lsystem_formula),
            _ => self.info().formula.to_string(),
        }
    }

    /// What the family is, in a sentence or two, in the language in use.
    pub(crate) fn about(self) -> String {
        match self {
            FractalKind::Mandelbrot => tr!(fractal_mandelbrot_about),
            FractalKind::Multibrot3 => tr!(fractal_multibrot3_about),
            FractalKind::Multibrot4 => tr!(fractal_multibrot4_about),
            FractalKind::Multibrot5 => tr!(fractal_multibrot5_about),
            FractalKind::Tricorn => tr!(fractal_tricorn_about),
            FractalKind::BurningShip => tr!(fractal_burning_ship_about),
            FractalKind::Celtic => tr!(fractal_celtic_about),
            FractalKind::Buffalo => tr!(fractal_buffalo_about),
            FractalKind::Phoenix => tr!(fractal_phoenix_about),
            FractalKind::Newton => tr!(fractal_newton_about),
            FractalKind::Multibrot6 => tr!(fractal_multibrot6_about),
            FractalKind::Multibrot7 => tr!(fractal_multibrot7_about),
            FractalKind::Multibrot8 => tr!(fractal_multibrot8_about),
            FractalKind::BurningShip3 => tr!(fractal_burning_ship3_about),
            FractalKind::BurningShip4 => tr!(fractal_burning_ship4_about),
            FractalKind::BurningShip5 => tr!(fractal_burning_ship5_about),
            FractalKind::Tricorn3 => tr!(fractal_tricorn3_about),
            FractalKind::Tricorn4 => tr!(fractal_tricorn4_about),
            FractalKind::Tricorn5 => tr!(fractal_tricorn5_about),
            FractalKind::Celtic3 => tr!(fractal_celtic3_about),
            FractalKind::Celtic4 => tr!(fractal_celtic4_about),
            FractalKind::Celtic5 => tr!(fractal_celtic5_about),
            FractalKind::Buffalo3 => tr!(fractal_buffalo3_about),
            FractalKind::Buffalo4 => tr!(fractal_buffalo4_about),
            FractalKind::Buffalo5 => tr!(fractal_buffalo5_about),
            FractalKind::Custom => tr!(fractal_custom_about),
            FractalKind::Life => tr!(fractal_life_about),
            FractalKind::LSystem => tr!(fractal_lsystem_about),
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/fractal/tests.rs"]
mod tests;
