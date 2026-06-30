//! The [`FractalKind`] domain enum — the escape-time families the app offers, plus their
//! display names, shader formula ids, deep-zoom support, and human descriptions.
//!
//! `formula_id` MUST match the shader's `fs_iterate` in `fractadyne-gpu/src/mandelbrot.wgsl`.

/// Per-family description shown in the info panel and Help.
pub(crate) struct FractalInfo {
    pub(crate) formula: &'static str,
    pub(crate) about: &'static str,
    pub(crate) reference: &'static str,
}

/// Escape-time fractal families. `formula_id` must match the shader's `fs_iterate`.
#[derive(Clone, Copy, PartialEq, Eq)]
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

    pub(crate) fn name(self) -> &'static str {
        match self {
            FractalKind::Mandelbrot => "Mandelbrot",
            FractalKind::Multibrot3 => "Multibrot 3",
            FractalKind::Multibrot4 => "Multibrot 4",
            FractalKind::Multibrot5 => "Multibrot 5",
            FractalKind::Tricorn => "Tricorn",
            FractalKind::BurningShip => "Burning Ship",
            FractalKind::Celtic => "Celtic",
            FractalKind::Buffalo => "Buffalo",
            FractalKind::Phoenix => "Phoenix",
            FractalKind::Newton => "Newton",
        }
    }

    pub(crate) fn from_name(name: &str) -> Option<FractalKind> {
        FractalKind::ALL.into_iter().find(|k| k.name() == name)
    }

    pub(crate) fn formula_id(self) -> u32 {
        match self {
            FractalKind::Mandelbrot => 0,
            FractalKind::Multibrot3 => 1,
            FractalKind::Multibrot4 => 2,
            FractalKind::Multibrot5 => 3,
            FractalKind::Tricorn => 4,
            FractalKind::BurningShip => 5,
            FractalKind::Celtic => 6,
            FractalKind::Buffalo => 7,
            FractalKind::Phoenix => 8,
            FractalKind::Newton => 9,
        }
    }

    /// Default view center (x, y) for this fractal.
    pub(crate) fn default_center(self) -> (f64, f64) {
        match self {
            FractalKind::Mandelbrot | FractalKind::Celtic => (-0.5, 0.0),
            FractalKind::BurningShip | FractalKind::Buffalo => (-0.5, -0.5),
            _ => (0.0, 0.0),
        }
    }

    /// Whether a Julia variant is meaningful (Newton has no parameter `c`).
    pub(crate) fn supports_julia(self) -> bool {
        !matches!(self, FractalKind::Newton)
    }

    /// Whether deep zoom (CPU reference + GPU perturbation) is implemented. The
    /// analytic families plus the abs-based ones (Burning Ship / Celtic / Buffalo,
    /// via the shader's `diffabs` fold) qualify; Phoenix and Newton stay on the
    /// direct path (clean to ~1e6×).
    pub(crate) fn supports_perturbation(self) -> bool {
        matches!(
            self,
            FractalKind::Mandelbrot
                | FractalKind::Multibrot3
                | FractalKind::Multibrot4
                | FractalKind::Multibrot5
                | FractalKind::Tricorn
                | FractalKind::BurningShip
                | FractalKind::Celtic
                | FractalKind::Buffalo
        )
    }

    /// Whether the extended-range floatexp perturbation path (GPU mode 2, for
    /// magnifications past ~1e28×) is implemented. The analytic/anti-analytic
    /// families qualify; the abs-based families currently run df32 perturbation
    /// (mode 0) only, so their clean range is ~1e4× … ~1e28×.
    pub(crate) fn supports_floatexp(self) -> bool {
        self.supports_perturbation() && self.formula_id() <= 4
    }

    pub(crate) fn info(self) -> FractalInfo {
        match self {
            FractalKind::Mandelbrot => FractalInfo {
                formula: "z -> z^2 + c    (z0 = 0)",
                about: "The canonical escape-time fractal: the set of c for which the \
                        orbit of 0 stays bounded. Its boundary is infinitely intricate.",
                reference: "https://en.wikipedia.org/wiki/Mandelbrot_set",
            },
            FractalKind::Multibrot3 => FractalInfo {
                formula: "z -> z^3 + c",
                about: "A Multibrot set - the Mandelbrot construction at a higher power. \
                        Power d gives (d-1)-fold rotational symmetry.",
                reference: "https://en.wikipedia.org/wiki/Multibrot_set",
            },
            FractalKind::Multibrot4 => FractalInfo {
                formula: "z -> z^4 + c",
                about: "Multibrot at power 4: threefold symmetry, broad bulbs.",
                reference: "https://en.wikipedia.org/wiki/Multibrot_set",
            },
            FractalKind::Multibrot5 => FractalInfo {
                formula: "z -> z^5 + c",
                about: "Multibrot at power 5: fourfold symmetry.",
                reference: "https://en.wikipedia.org/wiki/Multibrot_set",
            },
            FractalKind::Tricorn => FractalInfo {
                formula: "z -> conj(z)^2 + c",
                about: "The Tricorn (Mandelbar): conjugates z each step. This \
                        anti-holomorphic map yields a three-cornered shape.",
                reference: "https://en.wikipedia.org/wiki/Tricorn_(mathematics)",
            },
            FractalKind::BurningShip => FractalInfo {
                formula: "z -> (|Re z| + i|Im z|)^2 + c",
                about: "Absolute values of z's parts are taken before squaring; the \
                        result resembles a ship in flames.",
                reference: "https://en.wikipedia.org/wiki/Burning_Ship_fractal",
            },
            FractalKind::Celtic => FractalInfo {
                formula: "Re -> |Re(z^2)| + cx;  Im -> Im(z^2) + cy",
                about: "A Burning-Ship relative that takes the absolute value of only \
                        the real part of z^2, producing celtic-knot / heart motifs.",
                reference: "https://paulbourke.net/fractals/burnship/",
            },
            FractalKind::Buffalo => FractalInfo {
                formula: "Re -> |Re(z^2)| + cx;  Im -> |Im(z^2)| + cy",
                about: "An abs-variant taking absolute values of both components of z^2.",
                reference: "https://paulbourke.net/fractals/burnship/",
            },
            FractalKind::Phoenix => FractalInfo {
                formula: "z' = z^2 + c + p*z_prev    (p = -0.5)",
                about: "The Phoenix uses the previous iterate too, giving flame-like \
                        filaments. Try its Julia form via Julia mode.",
                reference: "https://paulbourke.net/fractals/phoenix/",
            },
            FractalKind::Newton => FractalInfo {
                formula: "z -> z - (z^3 - 1)/(3 z^2)",
                about: "Newton's root-finding iteration for z^3 = 1, colored by how fast \
                        each point converges. A convergence (not escape) fractal.",
                reference: "https://en.wikipedia.org/wiki/Newton_fractal",
            },
        }
    }
}
