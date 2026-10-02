use super::*;
use crate::life::dense::Dense;
use crate::life::parse_rle;

fn rule(s: &str) -> Rule {
    Rule::parse(s).unwrap_or_else(|e| panic!("{s}: {e}"))
}

fn with_cells(rule: &Rule, topology: Topology, cells: &[(i64, i64, u8)]) -> (Universe, Dense) {
    let mut u = Universe::new(rule.clone(), topology).unwrap();
    let mut d = Dense::new(rule.clone(), topology);
    for &(x, y, s) in cells {
        if u.set(x, y, s) {
            d.set(x, y, s);
        } else {
            assert!(matches!(topology, Topology::Bounded { .. }), "({x}, {y}) refused");
        }
    }
    (u, d)
}

/// A random soup of `states`-valued cells in `w × h` at `(x, y)`.
fn soup(seed: u64, x: i64, y: i64, w: i64, h: i64, density: f64, states: u16) -> Vec<(i64, i64, u8)> {
    let mut s = seed;
    let mut next = move || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        s
    };
    let mut out = Vec::new();
    for j in 0..h {
        for i in 0..w {
            if (next() % 1000) as f64 / 1000.0 < density {
                let st = 1 + (next() % u64::from(states - 1)) as u8;
                out.push((x + i, y + j, st));
            }
        }
    }
    out
}

/// Step both and compare every generation: the tile stepper must equal the reference stepper cell
/// for cell — no tolerance.
fn agree(u: &mut Universe, d: &mut Dense, generations: u32, what: &str) {
    for g in 1..=generations {
        u.step();
        d.step();
        assert_eq!(u.background(), d.background(), "{what}: background at generation {g}");
        let (a, b) = (u.cells(), d.cells());
        if a != b {
            let first = a.iter().zip(&b).position(|(p, q)| p != q).unwrap_or(a.len().min(b.len()));
            panic!(
                "{what}: generation {g} differs ({} vs {} cells; first difference at {:?} vs {:?})",
                a.len(),
                b.len(),
                a.get(first),
                b.get(first)
            );
        }
    }
    assert_eq!(u.generation(), u64::from(generations));
}

/// Random soups under every kind of rule — totalistic, Generations, Hensel, von Neumann, a random
/// MAP table, and B0 rules whose background blinks or turns alive — on all three topologies.
#[test]
fn the_tile_stepper_matches_the_reference_on_soups() {
    let rules = [
        "B3/S23",
        "B36/S23",
        "B3678/S34678",
        "B2/S",
        "B2/S/C3",
        "B2/S345/C4",
        "B3/S2-i34q",
        "B2-a/S12",
        "B2/S013V",
        "B0/S",
        "B0123478/S01234678",
        "B01/S1",
    ];
    let mut seed = 0x5EED_u64;
    let mut map_table = [0u64; 8];
    for w in &mut map_table {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *w = seed;
    }
    map_table[0] &= !1; // no B0, so it can run on the plane without blinking forever
    let random_map = Rule::from_table(map_table, 2).unwrap();
    let mut all: Vec<Rule> = rules.iter().map(|s| rule(s)).collect();
    all.push(random_map);
    let topologies = [
        Topology::Plane,
        Topology::Torus { width: 128, height: 64 },
        Topology::Bounded { x: -37, y: 11, width: 100, height: 70 },
    ];
    for (k, r) in all.iter().enumerate() {
        for (t, &topology) in topologies.iter().enumerate() {
            // Straddle tile edges and the origin: cells at negative coordinates too.
            let cells = soup(1 + k as u64 * 7 + t as u64, -20, 5, 48, 40, 0.35, r.states());
            let (mut u, mut d) = with_cells(r, topology, &cells);
            agree(&mut u, &mut d, 60, &format!("{r} on {topology:?}"));
        }
    }
}

/// A longer run of Life from a soup: debris, gliders leaving, tiles allocated and freed.
#[test]
fn a_life_soup_agrees_for_two_thousand_generations() {
    let r = Rule::life();
    let cells = soup(42, -30, -30, 60, 60, 0.4, 2);
    let (mut u, mut d) = with_cells(&r, Topology::Plane, &cells);
    agree(&mut u, &mut d, 2000, "Life soup");
    assert!(u.tile_count() > 1, "the soup spread across tiles");
}

fn glider() -> Vec<(i64, i64, u8)> {
    parse_rle("bo$2bo$3o!").unwrap().cells
}

fn shifted(cells: &[(i64, i64, u8)], dx: i64, dy: i64) -> Vec<(i64, i64, u8)> {
    let mut v: Vec<_> = cells.iter().map(|&(x, y, s)| (x + dx, y + dy, s)).collect();
    v.sort_unstable_by_key(|&(x, y, _)| (y, x));
    v
}

/// A glider started just short of a tile corner crosses it and travels 1,000 cells; the tiles
/// behind it are freed, so the stored count stays small.
#[test]
fn a_glider_crosses_tile_corners_and_leaves_no_tiles_behind() {
    let start = shifted(&glider(), 60, 60);
    let mut u = Universe::new(Rule::life(), Topology::Plane).unwrap();
    for &(x, y, s) in &start {
        u.set(x, y, s);
    }
    let mut most = 0;
    for _ in 0..1000 {
        u.step_n(4);
        most = most.max(u.tile_count());
    }
    assert_eq!(u.cells(), shifted(&start, 1000, 1000));
    assert!(most <= 4, "at most the four tiles around a corner are stored, saw {most}");
}

/// The same in the negative quadrant (a glider flipped to travel up and left): floor division of
/// negative coordinates into tiles.
#[test]
fn a_glider_travels_through_negative_coordinates() {
    let flipped: Vec<_> = glider().iter().map(|&(x, y, s)| (-x, -y, s)).collect();
    let start = shifted(&flipped, -3, -3);
    let mut u = Universe::new(Rule::life(), Topology::Plane).unwrap();
    for &(x, y, s) in &start {
        u.set(x, y, s);
    }
    u.step_n(4 * 300);
    assert_eq!(u.cells(), shifted(&start, -300, -300));
}

/// On a 64 × 64 torus a glider comes back to where it started after 4 · 64 generations.
#[test]
fn a_glider_wraps_round_a_torus() {
    let start = shifted(&glider(), 30, 30);
    let (mut u, _) = with_cells(&Rule::life(), Topology::Torus { width: 64, height: 64 }, &start);
    u.step_n(4 * 64);
    assert_eq!(u.cells(), start);
    u.step_n(4 * 10);
    assert_eq!(u.cells(), shifted(&start, 10, 10));
}

/// B0/S: from a single live cell the plane turns alive except for a 3 × 3 hole, then back to the
/// single cell — period 2, the background blinking with it.
#[test]
fn a_blinking_background_is_exact() {
    let (mut u, _) = with_cells(&rule("B0/S"), Topology::Plane, &[(5, -7, 1)]);
    u.step();
    assert_eq!(u.background(), 1);
    let hole: Vec<_> = (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (5 + dx, -7 + dy, 0))).collect();
    assert_eq!(u.cells(), hole);
    u.step();
    assert_eq!(u.background(), 0);
    assert_eq!(u.cells(), vec![(5, -7, 1)]);
}

/// Life and AntiLife (B0123478/S01234678, its live/dead reversal) are the same universe with the
/// colours swapped: a pattern run in one is the complement of its complement run in the other.
#[test]
fn antilife_is_life_with_the_colours_swapped() {
    let cells = soup(7, 0, 0, 30, 30, 0.4, 2);
    let (mut life, _) = with_cells(&Rule::life(), Topology::Plane, &cells);
    let mut anti = Universe::new(rule("B0123478/S01234678"), Topology::Plane).unwrap();
    assert!(anti.set_background(1));
    for &(x, y, _) in &cells {
        anti.set(x, y, 0);
    }
    for _ in 0..200 {
        life.step();
        anti.step();
        assert_eq!(anti.background(), 1);
        let a: Vec<_> = anti.cells().into_iter().map(|(x, y, _)| (x, y)).collect();
        let l: Vec<_> = life.cells().into_iter().map(|(x, y, _)| (x, y)).collect();
        assert_eq!(a, l);
    }
}

#[test]
fn cells_are_read_set_and_counted() {
    let mut u = Universe::new(rule("B2/S/C3"), Topology::Plane).unwrap();
    assert!(u.set(-1, -1, 2));
    assert!(u.set(100, 3, 1));
    assert!(!u.set(0, 0, 3), "Brian's Brain has states 0-2");
    assert!(!u.set(LIMIT, 0, 1), "past the plane's extent");
    assert_eq!(u.get(-1, -1), 2);
    assert_eq!(u.population(), 2);
    assert_eq!(u.bounding_box(), Some((-1, -1, 100, 3)));
    assert_eq!(u.tile_count(), 2);
    assert!(u.set(100, 3, 0));
    assert_eq!(u.tile_count(), 1, "a tile back to background is dropped");
    u.clear();
    assert_eq!((u.population(), u.tile_count(), u.bounding_box()), (0, 0, None));

    let mut t = Universe::new(Rule::life(), Topology::Torus { width: 64, height: 128 }).unwrap();
    assert!(t.set(-1, 130, 1));
    assert_eq!(t.get(63, 2), 1, "a torus wraps");
    assert_eq!(t.cells(), vec![(63, 2, 1)]);

    let mut b = Universe::new(Rule::life(), Topology::Bounded { x: 10, y: 10, width: 5, height: 5 }).unwrap();
    assert!(!b.set(9, 10, 1), "outside a bounded plane");
    assert!(b.set(14, 14, 1));
    assert!(!b.set_background(1), "only an unbounded plane has a background to set");
}

#[test]
fn topologies_are_validated() {
    assert!(Universe::new(Rule::life(), Topology::Torus { width: 100, height: 64 }).is_err());
    assert!(Universe::new(Rule::life(), Topology::Torus { width: 0, height: 64 }).is_err());
    assert!(Universe::new(Rule::life(), Topology::Bounded { x: 0, y: 0, width: 0, height: 3 }).is_err());
    assert!(Universe::new(Rule::life(), Topology::Bounded { x: LIMIT - 2, y: 0, width: 5, height: 3 }).is_err());
}

/// A random fill is the same soup for the same seed, and its density is what was asked.
#[test]
fn random_fill_is_reproducible() {
    let fill = |seed| {
        let mut u = Universe::new(Rule::life(), Topology::Plane).unwrap();
        u.random_fill(-50, -50, 200, 100, 0.3, seed);
        u
    };
    let (a, b, c) = (fill(1), fill(1), fill(2));
    assert_eq!(a.cells(), b.cells());
    assert_ne!(a.cells(), c.cells());
    let frac = a.population() as f64 / 20_000.0;
    assert!((frac - 0.3).abs() < 0.02, "density {frac}");
    let mut full = Universe::new(Rule::life(), Topology::Plane).unwrap();
    full.random_fill(0, 0, 10, 10, 1.0, 9);
    assert_eq!(full.population(), 100);
}
