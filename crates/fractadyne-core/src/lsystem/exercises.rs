//! Systems that exercise every command the library does not (tests only): step factors, `!`
//! inside and outside brackets, `|` with an odd division, free turns that are not whole steps,
//! colour commands, an angle that is no division of the circle. The walk and the tables are
//! checked against the reference on these as on the library.

use super::{library, LSystem};

const TEXTS: &[(&str, &str)] = &[
    ("scaled tree", "angle 25\nheading 90\naxiom F\nF = F[@0.6+F]@0.8F[!+F]-F\n"),
    ("inverse roots", "angle /8\naxiom X\nX = F@IQ2[+X]-F@Q2X\nF = FF\n"),
    // A factor above 1 before a branch: a reach that forgot the factor would fall short here.
    ("growing steps", "angle 90\naxiom F\nF = F@1.5[+F-F]@0.5F\n"),
    ("free turns", "angle 90\naxiom F\nF = F\\30F/45F|F\n"),
    ("odd division", "angle /5\naxiom F\nF = F|F+F[|F]\n"),
    ("reversed branches", "angle 45\naxiom X\nX = F[!+X]-X!+F\nF = FF\n"),
    // `!` outside brackets: whole subtrees run mirrored, and the walk steps over them so — which
    // only shows where a subtree ends off its own axis (the dragon's and the plant's do).
    ("reversed runs", "angle 90\naxiom X\nX = !F+X-F!X+F-X\n"),
    ("mirrored dragon", "angle 90\naxiom !FX\nX = X+YF+\nY = -FX-Y\n"),
    ("mirrored plant", "angle 22.5\nheading 90\naxiom !X\nX = F-[[X]+X]+F[+FX]-X\nF = FF\n"),
    ("colours", "angle 60\naxiom F\nF = C2F<1+F>2--F[C7+F]+F\n"),
    ("not a division", "angle 25.7\naxiom F\nF = F+F+F-F[++F]\n"),
    ("moves and variables", "angle 90\nvariables D\naxiom FD\nD = fG+F-D\nF = F+G\n"),
    // Filled polygons: leaves on a branching stem (steps and `.` vertices, inside brackets), a
    // filled snowflake (its outline rewritten inside the braces), and a polygon whose outline holds
    // a symbol that opens polygons of its own.
    ("leaves", "angle 30\nheading 90\naxiom X\nX = F[+{.f-f-f.}]F[-{.f+f+f.}]X\nF = FF\n"),
    ("filled snowflake", "angle 60\naxiom {F++F++F}\nF = F-F++F-F\n"),
    ("nested polygons", "angle 90\naxiom {FAFAFAF}\nA = +[{f-f-f-f}]F\nF = F+F-F\n"),
    // Stochastic: a branching plant whose alternatives differ in length (so its subtrees differ in
    // size), a curve whose spikes go either way, a dragon whose X folds either way (alternatives
    // that turn), and leaves on either side (alternatives that open polygons). A symbol appears
    // several times in a word, so the children's variants by place are exercised.
    ("stochastic plant", "angle 25.7\nheading 90\nseed 7\naxiom F\nF (1) = F[+F]F[-F]F\nF (1) = F[+F]F\nF (1) = F[-F]F\n"),
    ("stochastic curve", "angle 90\nseed 3\naxiom F\nF (2) = F+F-F-F+F\nF (1) = F-F+F+F-F\n"),
    ("stochastic dragon", "angle 90\nseed 11\naxiom FX\nX (1) = X+YF+\nX (1) = X-YF-\nY = -FX-Y\n"),
    ("stochastic leaves", "angle 30\nheading 90\nseed 5\naxiom X\nX (1) = F[+{.f-f-f.}]X\nX (1) = F[-{.f+f+f.}]X\nX (0.5) = FX\nF = FF\n"),
];

/// The library and the exercises: every system the walk draws (the parametric and
/// context-sensitive ones are built as words instead, and checked in `expand/tests.rs`).
pub(crate) fn all() -> Vec<LSystem> {
    let mut v: Vec<LSystem> =
        library::SYSTEMS.iter().map(|e| e.system().unwrap()).filter(|s| s.expanded.is_none()).collect();
    for (name, text) in TEXTS {
        let mut s = LSystem::parse(text).unwrap_or_else(|e| panic!("{name}: {e}"));
        s.name = name.to_string();
        v.push(s);
    }
    v
}
