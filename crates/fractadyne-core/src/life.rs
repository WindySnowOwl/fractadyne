//! Life-like cellular automata (design/automata.md §4.1–4.3, phase 1): rules as one 512-bit table,
//! the unbounded sparse universe and its stepper, pattern files, and the built-in sets. CPU only —
//! the GPU stepper (phase 2) is tested against [`Universe`], and [`Universe`] against
//! [`dense::Dense`], the reference stepper.

mod hensel;

mod rule;
pub use rule::*;

mod universe;
pub use universe::*;

pub mod dense;

mod formats;
pub use formats::*;

pub mod library;
