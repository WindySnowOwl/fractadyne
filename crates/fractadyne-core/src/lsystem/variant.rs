//! Stochastic productions (design/lsystems.md §9.2): a symbol with weighted alternatives rewrites
//! by one of them, chosen at random — reproducibly, and so that the walk can still skip, cull and
//! zoom.
//!
//! Every node of the derivation carries a **variant**, one of [`VARIANTS`]. Its production is chosen
//! by its symbol and variant alone, through a hash of the system's seed; its children's variants
//! follow from its own and their places in the word. So everything about a node is a function of
//! (symbol, depth, variant): the tables are kept per variant, and two nodes that share all three are
//! the same picture (with 64 variants mixed at every level, not visibly so).
//!
//! The choice does not depend on the depth. A node keeps its variant as the order rises (only its
//! depth changes), so it keeps its choice: order n + 1 is order n with one more level of detail, and
//! a plant does not reshuffle itself every time the zoom gains an order.

use super::system::LSystem;

/// How many variants a stochastic system's nodes take.
pub const VARIANTS: u32 = 64;

/// SplitMix64's finaliser: a well-mixed 64-bit hash of `x`.
fn mix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

/// How a system's nodes get their variants and choose their productions.
#[derive(Clone, Debug, PartialEq)]
pub struct Variants {
    /// How many there are: 1 for a deterministic system (every node is variant 0).
    pub k: u32,
    /// The axiom's variant (its symbols are its children).
    pub root: u32,
    seed: u64,
    /// A child's variant is `perm[(v + shift[j]) mod k]`, `j` its place in its parent's word: for
    /// each place a permutation of the variants, so a line of descent through the same place (a
    /// stem) cycles through about half of them rather than collapsing onto a few.
    perm: Vec<u16>,
    shift: Vec<u16>,
}

impl Variants {
    /// The variants of `sys`: [`VARIANTS`] if any symbol has alternatives, else one.
    pub fn of(sys: &LSystem) -> Variants {
        let seed = sys.seed;
        if !sys.is_stochastic() {
            return Variants { k: 1, root: 0, seed, perm: vec![0], shift: Vec::new() };
        }
        let k = VARIANTS;
        let h = |a: u64, b: u64| mix64(mix64(seed ^ a.wrapping_mul(0x51_7cc1_b727_220a)) ^ b);
        // Fisher–Yates, from the seed.
        let mut perm: Vec<u16> = (0..k as u16).collect();
        for i in (1..k as usize).rev() {
            let j = (h(1, i as u64) % (i as u64 + 1)) as usize;
            perm.swap(i, j);
        }
        let longest = sys.words().map(Vec::len).max().unwrap_or(0);
        let shift = (0..longest).map(|j| (h(2, j as u64) % u64::from(k)) as u16).collect();
        Variants { k, root: (h(3, 0) % u64::from(k)) as u32, seed, perm, shift }
    }

    /// The variant of the child at place `j` of a word rewritten from a node of variant `v`.
    #[inline]
    pub fn child(&self, v: u32, j: usize) -> u32 {
        if self.k == 1 {
            return 0;
        }
        let s = u32::from(self.shift[j]);
        u32::from(self.perm[((v + s) % self.k) as usize])
    }

    /// Which of `weights` (a symbol's alternatives) a node of symbol `c` and variant `v` rewrites by.
    pub fn choose(&self, c: u8, v: u32, weights: &[f64]) -> usize {
        if weights.len() <= 1 {
            return 0;
        }
        let total: f64 = weights.iter().sum();
        let u = (mix64(mix64(self.seed ^ 0xa076_1d64_78bd_642f) ^ (u64::from(c) << 32 | u64::from(v))) >> 11) as f64
            / (1u64 << 53) as f64
            * total;
        let mut acc = 0.0;
        for (i, w) in weights.iter().enumerate() {
            acc += w;
            if u < acc {
                return i;
            }
        }
        weights.len() - 1
    }
}

#[cfg(test)]
mod tests;
