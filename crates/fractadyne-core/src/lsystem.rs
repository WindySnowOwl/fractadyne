//! L-systems (design/lsystems.md): a grammar rewrites a word, and a turtle draws the result. The word
//! after n rewrites is never built — [`walk`] descends the derivation tree, skipping every subtree
//! that is off the view or under a pixel by what [`Tables`] know of it, so a view costs what its
//! pixels cost at any order. CPU only; [`reference`] (build the word, run the turtle) is the check.

mod system;
pub use system::*;

mod fractint;
pub use fractint::*;

mod tables;
pub use tables::*;

mod walk;
pub use walk::*;

mod deep;
pub use deep::*;

pub mod reference;

pub mod library;

#[cfg(test)]
mod exercises;
