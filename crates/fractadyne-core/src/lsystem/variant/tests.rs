use super::*;
use crate::lsystem::reference;

fn plant(seed: u64) -> LSystem {
    LSystem::parse(&format!(
        "angle 25.7\nheading 90\nseed {seed}\naxiom F\nF (1) = F[+F]F[-F]F\nF (1) = F[+F]F\nF (2) = F[-F]F\n"
    ))
    .unwrap()
}

#[test]
fn a_deterministic_system_has_one_variant() {
    let s = LSystem::parse("angle 60\naxiom F\nF = F-F++F-F\n").unwrap();
    let v = Variants::of(&s);
    assert_eq!((v.k, v.root), (1, 0));
    assert_eq!(v.child(0, 3), 0);
    assert_eq!(v.choose(b'F', 0, &[1.0]), 0);
}

/// For each place in a word, the children's variants are a permutation of the parents': a line
/// of descent through one place (a stem) cannot collapse onto a few variants, as it would through a
/// hash of (variant, place) — a random map of 64 values falls into a cycle of about 5.
#[test]
fn each_place_permutes_the_variants() {
    let v = Variants::of(&plant(1));
    assert_eq!(v.k, VARIANTS);
    for j in 0..11 {
        let mut seen = vec![false; v.k as usize];
        for p in 0..v.k {
            let c = v.child(p, j);
            assert!(!std::mem::replace(&mut seen[c as usize], true), "place {j}: variant {c} twice");
        }
    }
}

/// Over many seeds, each alternative is chosen in proportion to its weight (1 : 1 : 2).
#[test]
fn the_choices_follow_the_weights() {
    let mut count = [0u32; 3];
    for seed in 0..400 {
        let v = Variants::of(&plant(seed));
        for p in 0..v.k {
            count[v.choose(b'F', p, &[1.0, 1.0, 2.0])] += 1;
        }
    }
    let total: u32 = count.iter().sum();
    for (i, want) in [0.25, 0.25, 0.5].into_iter().enumerate() {
        let got = f64::from(count[i]) / f64::from(total);
        assert!((got - want).abs() < 0.01, "alternative {i}: {got} of the choices, weight {want}");
    }
}

/// The seed is what the picture follows: the same seed, the same word; another seed, another.
#[test]
fn the_seed_decides_the_picture() {
    let w = |seed| reference::expand(&plant(seed), 5, 1 << 20).unwrap();
    assert_eq!(w(4), w(4));
    assert_ne!(w(4), w(5));
}

/// Every alternative is drawn somewhere: the stochastic plant at order 4 holds all three shapes.
#[test]
fn every_alternative_is_taken() {
    let s = plant(2);
    let v = Variants::of(&s);
    let taken: std::collections::HashSet<usize> = (0..v.k).map(|p| v.choose(b'F', p, &[1.0, 1.0, 2.0])).collect();
    assert_eq!(taken.len(), 3);
}
