//! The standard set (design/lsystems.md §6), entered from published definitions: Paul Bourke's
//! L-system pages, *The Algorithmic Beauty of Plants* figure 1.24, and the curves' own literature.
//! What can be checked is, in `library/tests.rs` — segment counts, the grids the space-filling curves
//! visit, the dragon's end — so a mistyped symbol fails a test rather than shipping.

use super::system::{LSystem, ParseError};

/// Which shelf of the library a system sits on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Curve,
    SpaceFilling,
    Tiling,
    Plant,
}

impl Category {
    pub const ALL: [Category; 4] = [Category::Curve, Category::SpaceFilling, Category::Tiling, Category::Plant];

    pub fn label(self) -> &'static str {
        match self {
            Category::Curve => "Curves",
            Category::SpaceFilling => "Space-filling curves",
            Category::Tiling => "Islands and tilings",
            Category::Plant => "Plants",
        }
    }
}

/// A named system, in the native format.
#[derive(Clone, Copy, Debug)]
pub struct NamedSystem {
    pub name: &'static str,
    pub category: Category,
    pub text: &'static str,
    pub about: &'static str,
}

impl NamedSystem {
    pub fn system(&self) -> Result<LSystem, ParseError> {
        let mut s = LSystem::parse(self.text)?;
        s.name = self.name.to_string();
        Ok(s)
    }
}

/// Finds a system by name (ignoring case).
pub fn find(name: &str) -> Option<&'static NamedSystem> {
    SYSTEMS.iter().find(|s| s.name.eq_ignore_ascii_case(name))
}

pub const SYSTEMS: &[NamedSystem] = &[
    // ---- curves
    NamedSystem {
        name: "Koch curve",
        category: Category::Curve,
        text: "angle 60\naxiom F\nF = F+F--F+F\n",
        about: "Each segment becomes four a third as long, with a spike: dimension log 4 / log 3.",
    },
    NamedSystem {
        name: "Koch snowflake",
        category: Category::Curve,
        text: "angle 60\naxiom F++F++F\nF = F-F++F-F\n",
        about: "Three Koch curves on a triangle: a finite area inside an infinite perimeter.",
    },
    NamedSystem {
        name: "Quadratic Koch curve",
        category: Category::Curve,
        text: "angle 90\naxiom F\nF = F-F+F+F-F\n",
        about: "The Koch construction with square spikes.",
    },
    NamedSystem {
        name: "Levy C curve",
        category: Category::Curve,
        text: "angle 45\naxiom F\nF = -F++F-\n",
        about: "Paul Levy's curve: each segment becomes the two legs of a right isosceles triangle.",
    },
    NamedSystem {
        name: "Heighway dragon",
        category: Category::Curve,
        text: "angle 90\naxiom FX\nX = X+YF+\nY = -FX-Y\n",
        about: "The paper-folding curve (Davis and Knuth, 1970): it never crosses itself.",
    },
    NamedSystem {
        name: "Terdragon",
        category: Category::Curve,
        text: "angle 120\naxiom F\nF = F+F-F\n",
        about: "The dragon's three-segment cousin, turning by thirds of a circle.",
    },
    NamedSystem {
        name: "Sierpinski arrowhead",
        category: Category::Curve,
        text: "angle 60\naxiom YF\nX = YF+XF+Y\nY = XF-YF-X\n",
        about: "One unbroken curve that traces out the Sierpinski triangle.",
    },
    NamedSystem {
        name: "Sierpinski triangle",
        category: Category::Curve,
        text: "angle 120\ndraw G\naxiom F-G-G\nF = F-G+F+G-F\nG = GG\n",
        about: "The triangle with triangles cut out of it, drawn edge by edge.",
    },
    NamedSystem {
        name: "Sierpinski curve",
        category: Category::Curve,
        text: "angle 45\naxiom F--XF--F--XF\nX = XF+F+XF--F--XF+F+X\n",
        about: "Sierpinski's closed curve (rules by Chris Wallace, from Paul Bourke's collection).",
    },
    NamedSystem {
        name: "Sierpinski square curve",
        category: Category::Curve,
        text: "angle 90\naxiom F+XF+F+XF\nX = XF-F+F-XF+F+XF-F+F-X\n",
        about: "A closed curve of squares within squares.",
    },
    NamedSystem {
        name: "Krishna anklets",
        category: Category::Curve,
        text: "angle 45\naxiom -X--X\nX = XFX--XFX\n",
        about: "A pattern of loops from the kolam tradition of South India.",
    },
    // ---- space-filling curves
    NamedSystem {
        name: "Hilbert curve",
        category: Category::SpaceFilling,
        text: "angle 90\naxiom X\nX = -YF+XFX+FY-\nY = +XF-YFY-FX+\n",
        about: "David Hilbert's curve (1891): visits every cell of a 2^n by 2^n grid, each step to a neighbour.",
    },
    NamedSystem {
        name: "Moore curve",
        category: Category::SpaceFilling,
        text: "angle 90\naxiom LFL+F+LFL\nL = -RF+LFL+FR-\nR = +LF-RFR-FL+\n",
        about: "Four Hilbert curves joined into a loop: it ends next to where it began.",
    },
    NamedSystem {
        name: "Peano curve",
        category: Category::SpaceFilling,
        text: "angle 90\naxiom X\nX = XFYFX+F+YFXFY-F-XFYFX\nY = YFXFY-F-XFYFX+F+YFXFY\n",
        about: "Giuseppe Peano's curve (1890), the first space-filling curve: a 3^n by 3^n grid.",
    },
    NamedSystem {
        name: "Gosper curve",
        category: Category::SpaceFilling,
        text: "angle 60\naxiom XF\nX = X+YF++YF-FX--FXFX-YF+\nY = -FX+YFYF++YF+FX--FX-Y\n",
        about: "The flowsnake (Bill Gosper): fills a shape whose boundary is itself a fractal.",
    },
    NamedSystem {
        name: "Quadratic Gosper curve",
        category: Category::SpaceFilling,
        text: "angle 90\naxiom -YF\n\
               X = XFX-YF-YF+FX+FX-YF-YFFX+YF+FXFXYF-FX+YF+FXFX+YF-FXYF-YF-FX+FX+YFYF-\n\
               Y = +FXFX-YF-YF+FX+FXYF+FX-YFYF-FX-YF+FXYFYF-FX-YFFX+FX+YF-YF-FX+FX+YFY\n",
        about: "Dekking's (1982) square relative of the Gosper curve: a 5^n by 5^n grid.",
    },
    // ---- islands and tilings
    NamedSystem {
        name: "Quadratic Koch island",
        category: Category::Tiling,
        text: "angle 90\naxiom F+F+F+F\nF = F+F-F-FF+F+F-F\n",
        about: "A square whose sides become eight segments a quarter as long: dimension 3/2.",
    },
    NamedSystem {
        name: "Pentaplexity",
        category: Category::Tiling,
        text: "angle 36\naxiom F++F++F++F++F\nF = F++F++F|F-F++F\n",
        about: "Roger Penrose's pentagon pattern, as an edge-rewriting curve.",
    },
    NamedSystem {
        name: "Crystal",
        category: Category::Tiling,
        text: "angle 90\naxiom F+F+F+F\nF = FF+F++F+F\n",
        about: "Squares sprouting squares.",
    },
    NamedSystem {
        name: "Board",
        category: Category::Tiling,
        text: "angle 90\naxiom F+F+F+F\nF = FF+F+F+F+FF\n",
        about: "A square subdivided into a board of squares.",
    },
    NamedSystem {
        name: "Tiles",
        category: Category::Tiling,
        text: "angle 90\naxiom F+F+F+F\nF = FF+F-F+F+FF\n",
        about: "A tiling pattern: dimension log 7 / log 3.",
    },
    NamedSystem {
        name: "Rings",
        category: Category::Tiling,
        text: "angle 90\naxiom F+F+F+F\nF = FF+F+F+F+F+F-F\n",
        about: "Squares of rings.",
    },
    NamedSystem {
        name: "Cross",
        category: Category::Tiling,
        text: "angle 90\naxiom F+F+F+F\nF = F+FF++F+F\n",
        about: "Crosses within crosses.",
    },
    NamedSystem {
        name: "Koch snowflake (filled)",
        category: Category::Tiling,
        text: "angle 60\naxiom {F++F++F}\nF = F-F++F-F\n",
        about: "The snowflake as a shape: its outline inside braces is a polygon, filled.",
    },
    NamedSystem {
        name: "Snake kolam",
        category: Category::Tiling,
        text: "angle 90\naxiom F+XF+F+XF\nX = X{F-F-F}+XF+F+X{F-F-F}+X\n",
        about: "A kolam (from Paul Bourke's collection), with filled squares along its path.",
    },
    // ---- plants (The Algorithmic Beauty of Plants, figure 1.24 a-f; then Paul Bourke's)
    NamedSystem {
        name: "Plant (ABOP 1.24a)",
        category: Category::Plant,
        text: "angle 25.7\nheading 90\naxiom F\nF = F[+F]F[-F]F\n",
        about: "A branching weed: every segment grows a twig on each side.",
    },
    NamedSystem {
        name: "Plant (ABOP 1.24b)",
        category: Category::Plant,
        text: "angle 20\nheading 90\naxiom F\nF = F[+F]F[-F][F]\n",
        about: "A plant whose every segment forks three ways.",
    },
    NamedSystem {
        name: "Plant (ABOP 1.24c)",
        category: Category::Plant,
        text: "angle 22.5\nheading 90\naxiom F\nF = FF-[-F+F+F]+[+F-F-F]\n",
        about: "A bushy plant with curling side branches.",
    },
    NamedSystem {
        name: "Plant (ABOP 1.24d)",
        category: Category::Plant,
        text: "angle 20\nheading 90\naxiom X\nX = F[+X]F[-X]+X\nF = FF\n",
        about: "Branches grow from the apices (X); stems double in length each generation.",
    },
    NamedSystem {
        name: "Plant (ABOP 1.24e)",
        category: Category::Plant,
        text: "angle 25.7\nheading 90\naxiom X\nX = F[+X][-X]FX\nF = FF\n",
        about: "A symmetric plant: paired branches at every node.",
    },
    NamedSystem {
        name: "Plant (ABOP 1.24f)",
        category: Category::Plant,
        text: "angle 22.5\nheading 90\naxiom X\nX = F-[[X]+X]+F[+FX]-X\nF = FF\n",
        about: "The fractal plant: a leaning frond.",
    },
    NamedSystem {
        name: "Weed",
        category: Category::Plant,
        text: "angle 22.5\nheading 90\naxiom F\nF = FF-[XY]+[XY]\nX = +FY\nY = -FX\n",
        about: "A weed of zigzag stems (from Paul Bourke's collection).",
    },
    NamedSystem {
        name: "Bush",
        category: Category::Plant,
        text: "angle 25.7\nheading 90\naxiom Y\nX = X[-FFF][+FFF]FX\nY = YFX[+Y][-Y]\n",
        about: "A bush with straight side shoots (from Paul Bourke's collection).",
    },
    NamedSystem {
        name: "Mango leaf",
        category: Category::Plant,
        text: "angle 60\nheading 90\norder 18\naxiom Y---Y\nX = {F-F}{F-F}--[--X]{F-F}{F-F}--{F-F}{F-F}--\nY = f-F+X+F-fY\n",
        about: "A leaf of filled diamonds (from Paul Bourke's collection). It grows by a step an order, so it is drawn at a fixed one.",
    },
    NamedSystem {
        name: "Saupe's bush",
        category: Category::Plant,
        text: "angle 20\nheading 90\norder 9\naxiom VZFFF\nV = [+++W][---W]YV\nW = +X[-W]Z\nX = -W[+X]Z\nY = YZ\nZ = [-FFF][+FFF]F\n",
        about: "Dietmar Saupe's bush (from Paul Bourke's collection). Its stem grows by a step each generation, not by a factor, so it is drawn at a fixed order.",
    },
];

#[cfg(test)]
mod tests;
