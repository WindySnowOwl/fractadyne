use super::*;
use crate::life::hensel::{canonical as class_rep, class_of};

/// Life's MAP string, as LifeWiki gives it.
const LIFE_MAP: &str = "MAPARYXfhZofugWaH7oaIDogBZofuhogOiAaIDogIAAgAAWaH7oaIDogGiA6ICAAIAAaIDogIAAgACAAIAAAAAAAA";

fn neighbours(idx: usize) -> u32 {
    (idx & !CENTRE).count_ones()
}

fn rng(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

/// Life from its definition — born on three, survives on two or three — with no table in sight.
#[test]
fn life_is_born_on_three_and_survives_on_two_or_three() {
    let life = Rule::life();
    for idx in 0..512 {
        let n = neighbours(idx);
        let want = n == 3 || (idx & CENTRE != 0 && n == 2);
        assert_eq!(life.alive(idx), want, "index {idx:#011b}");
    }
    assert_eq!(life.states(), 2);
    assert!(!life.has_b0());
}

/// Every spelling of Life is Life: B/S, the lower-case run-together form, the older S/B, Hensel
/// with every letter written out, and LifeWiki's MAP string — which also pins the MAP bit order
/// (index 0 first, most significant bit first) and the centre's place in the index.
#[test]
fn every_spelling_of_life_is_life() {
    let life = Rule::life();
    for s in ["B3/S23", "b3s23", "B3S23", "23/3", "S23/B3", " B3/S23 ", "B3cekainyqjr/S2cekain3cekainyqjr", LIFE_MAP] {
        assert_eq!(Rule::parse(s).as_ref(), Ok(&life), "{s}");
    }
    assert_eq!(life.canonical(), "B3/S23");
    assert_eq!(format!("MAP{}", encode_map(&life.table())), LIFE_MAP);
}

/// tlife (LifeWiki: "B3/S2-i34q") is Life except that a cell with two opposite orthogonal
/// neighbours (2i) does not survive and one with the 4q arrangement does.
#[test]
fn a_hensel_rule_changes_exactly_the_classes_it_names() {
    let life = Rule::life();
    let tlife = Rule::parse("B3/S2-i34q").unwrap();
    let mut changed = 0;
    for idx in 0..512 {
        let (n, l) = class_of(mask_of(idx));
        let survival = idx & CENTRE != 0;
        let want = match (survival, n, l) {
            (true, 2, 'i') => false,
            (true, 4, 'q') => true,
            _ => life.alive(idx),
        };
        assert_eq!(tlife.alive(idx), want, "index {idx:#011b} ({n}{l})");
        changed += (tlife.alive(idx) != life.alive(idx)) as u32;
    }
    // 2i has 2 arrangements (vertical, horizontal); 4q has 4 (it is its own mirror image across a
    // diagonal, so only the four turns differ).
    assert_eq!(changed, 2 + 4);
    assert_eq!(tlife.canonical(), "B3/S2-i34q");
    assert_eq!(Rule::parse("B2-a/S12").unwrap().canonical(), "B2-a/S12");
    // `-` with the letters it lacks, or the letters it has: whichever is shorter.
    assert_eq!(Rule::parse("B2cekin/S").unwrap().canonical(), "B2-a/S");
    assert_eq!(Rule::parse("B3-cekainyqj/S").unwrap().canonical(), "B3r/S");
}

/// A von Neumann rule ignores the corners and counts the four orthogonal neighbours.
#[test]
fn von_neumann_rules_ignore_the_corners() {
    let r = Rule::parse("B2/S013V").unwrap();
    for idx in 0..512 {
        let orth = (idx & (N | W | E | S)).count_ones();
        let want = if idx & CENTRE != 0 { [0, 1, 3].contains(&orth) } else { orth == 2 };
        assert_eq!(r.alive(idx), want, "index {idx:#011b}");
    }
    assert!(r.is_von_neumann() && !r.is_totalistic());
    assert_eq!(r.canonical(), "B2/S013V");
    // LifeWiki: B1/SV in Hensel letters.
    assert_eq!(Rule::parse("B1/SV").unwrap(), Rule::parse("B1e2ak3inqy4ny5e/S").unwrap());
}

/// Generations: state 1 alive, 2…C−1 dying in order, then dead; only state 1 counts as a
/// neighbour, which the caller's index already encodes.
#[test]
fn generations_rules_age_their_dying_cells() {
    let brain = Rule::parse("B2/S/C3").unwrap();
    assert_eq!(brain.states(), 3);
    for spelling in ["/2/3", "B2/S/3", "b2/s/g3", "B2/S/C3"] {
        assert_eq!(Rule::parse(spelling).unwrap(), brain, "{spelling}");
    }
    let two = NW | N; // two live neighbours
    assert_eq!(brain.next(0, two), 1, "born on two");
    assert_eq!(brain.next(0, NW), 0);
    assert_eq!(brain.next(1, two | CENTRE), 2, "S is empty: every live cell starts dying");
    assert_eq!(brain.next(2, two), 0, "the last dying state dies");
    let wars = Rule::parse("345/2/4").unwrap(); // Star Wars
    assert_eq!(wars.canonical(), "B2/S345/C4");
    assert_eq!(wars.next(1, N | S | E | CENTRE), 1, "survives on three");
    assert_eq!(wars.next(1, N | CENTRE), 2);
    assert_eq!(wars.next(2, 0), 3);
    assert_eq!(wars.next(3, 511), 0);
}

/// The background of an unbounded plane changes only under B0.
#[test]
fn the_background_changes_only_under_b0() {
    let life = Rule::life();
    assert_eq!((life.next_background(0), life.next_background(1)), (0, 0));
    let blink = Rule::parse("B0/S").unwrap();
    assert_eq!((blink.next_background(0), blink.next_background(1)), (1, 0));
    let stay = Rule::parse("B0/S8").unwrap();
    assert_eq!((stay.next_background(0), stay.next_background(1)), (1, 1));
    assert!(Rule::parse("B0/S/C3").is_err(), "Generations with B0 are refused");
}

/// The canonical string reads back to the same rule, whichever form it takes: totalistic,
/// von Neumann, Hensel (random isotropic rules), MAP (random tables), with and without states.
#[test]
fn canonical_strings_round_trip() {
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    let check = |r: &Rule| {
        let s = r.canonical();
        assert_eq!(Rule::parse(&s).as_ref(), Ok(r), "{s}");
    };
    for s in ["B3/S23", "B36/S23", "B3678/S34678", "B2/S", "B/S", "B012345678/S012345678", "B2/S013V", "B2/S345/C4"] {
        check(&Rule::parse(s).unwrap());
    }
    let classes: Vec<u8> = {
        let mut v: Vec<u8> = (0..=255u8).map(class_rep).collect();
        v.sort_unstable();
        v.dedup();
        v
    };
    assert_eq!(classes.len(), 51);
    for _ in 0..200 {
        // A random isotropic rule: one coin per (class, centre).
        let coins: Vec<u64> = (0..2).map(|_| rng(&mut seed)).collect();
        let mut table = [0u64; 8];
        for idx in 0..512usize {
            let k = classes.iter().position(|&c| c == class_rep(mask_of(idx))).unwrap();
            let c = (idx & CENTRE != 0) as usize;
            if (coins[c] >> k) & 1 != 0 {
                table[idx >> 6] |= 1 << (idx & 63);
            }
        }
        if table[0] & 1 != 0 {
            table[0] &= !1; // keep B0 off so the state count may vary
        }
        let states = 2 + (rng(&mut seed) % 3) as u16;
        let r = Rule::from_table(table, states).unwrap();
        assert!(r.is_isotropic());
        check(&r);
        // A random table: almost surely not isotropic, so a MAP string.
        let t: [u64; 8] = std::array::from_fn(|_| rng(&mut seed));
        let r = Rule::from_table(t, 2).unwrap();
        assert!(r.canonical().starts_with("MAP"));
        check(&r);
    }
}

#[test]
fn malformed_rules_are_refused_with_a_reason() {
    for (s, says) in [
        ("", "empty"),
        ("Life", "not a rule"),
        ("B9/S23", "0-8"),
        ("B3/S23H", "hexagonal"),
        ("B2x/S", "not a letter"),
        ("B3-/S23", "followed by letters"),
        ("B0c/S", "no letters"),
        ("B2a/S1V", "no Hensel letters"),
        ("B5/S1V", "4 neighbours"),
        ("B3/S23/C1", "states"),
        ("B3/S23/C999", "states"),
        ("B3/B3/S23", "twice"),
        ("MAPABC", "86"),
        ("B3 /S23", "spaces"),
        ("B0/S/C3", "B0"),
    ] {
        let e = Rule::parse(s).expect_err(s);
        assert!(e.0.contains(says), "'{s}': {e}");
    }
}

/// The GPU's 16 words are the table, low word first.
#[test]
fn the_gpu_words_are_the_table() {
    let r = Rule::parse("B2-a/S12").unwrap();
    let w = r.table_words();
    for idx in 0..512 {
        assert_eq!((w[idx / 32] >> (idx % 32)) & 1 != 0, r.alive(idx));
    }
}

/// A table index and its arrangement convert both ways.
#[test]
fn indices_and_arrangements_convert_both_ways() {
    for idx in 0..512 {
        assert_eq!(index_of(mask_of(idx), idx & CENTRE != 0), idx);
        assert_eq!(mask_of(idx).count_ones(), neighbours(idx));
    }
    assert_eq!(mask_of(NW), crate::life::hensel::M_NW);
    assert_eq!(mask_of(SE), crate::life::hensel::M_SE);
}
