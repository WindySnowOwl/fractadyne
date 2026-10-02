use super::*;
use crate::life::{parse_rle, Rule, Topology, Universe};

fn load(name: &str) -> Universe {
    let p = pattern_named(name).unwrap_or_else(|| panic!("no pattern {name}"));
    let mut u = Universe::new(Rule::parse(p.rule).unwrap(), Topology::Plane).unwrap();
    for &(x, y, s) in &parse_rle(p.rle).unwrap().cells {
        u.set(x, y, s);
    }
    u
}

fn shifted(cells: &[(i64, i64, u8)], dx: i64, dy: i64) -> Vec<(i64, i64, u8)> {
    let mut v: Vec<_> = cells.iter().map(|&(x, y, s)| (x + dx, y + dy, s)).collect();
    v.sort_unstable_by_key(|&(x, y, _)| (y, x));
    v
}

/// The displacement that maps `a` onto `b`, if one does.
fn translation(a: &[(i64, i64, u8)], b: &[(i64, i64, u8)]) -> Option<(i64, i64)> {
    let (&(ax, ay, _), &(bx, by, _)) = (a.first()?, b.first()?);
    let d = (bx - ax, by - ay);
    (a.len() == b.len() && shifted(a, d.0, d.1) == b).then_some(d)
}

#[test]
fn every_library_entry_parses() {
    for r in RULES {
        let rule = Rule::parse(r.rule).unwrap_or_else(|e| panic!("{}: {e}", r.name));
        assert_eq!(rule.canonical(), r.rule, "{}: write the canonical string", r.name);
    }
    for p in PATTERNS {
        let cells = parse_rle(p.rle).unwrap_or_else(|e| panic!("{}: {e}", p.name)).cells;
        assert!(!cells.is_empty(), "{}", p.name);
        Rule::parse(p.rule).unwrap();
    }
    assert_eq!(rule_named("highlife").map(|r| r.rule), Some("B36/S23"));
    assert!(pattern_named(" glider ").is_some());
}

/// Populations as the patterns are defined (LifeWiki).
#[test]
fn patterns_have_their_populations() {
    for (name, pop) in [
        ("Block", 4),
        ("Beehive", 6),
        ("Loaf", 7),
        ("Boat", 5),
        ("Blinker", 3),
        ("Toad", 6),
        ("Beacon", 8),
        ("Pulsar", 48),
        ("Pentadecathlon", 12),
        ("Glider", 5),
        ("Lightweight spaceship", 9),
        ("Middleweight spaceship", 11),
        ("Heavyweight spaceship", 13),
        ("R-pentomino", 5),
        ("Diehard", 7),
        ("Acorn", 7),
        ("Gosper glider gun", 36),
        ("Simkin glider gun", 36),
    ] {
        assert_eq!(load(name).population(), pop, "{name}");
    }
}

/// Still lifes stay; oscillators return after exactly their period and not before.
#[test]
fn still_lifes_and_oscillators_have_their_periods() {
    for (name, period) in
        [("Block", 1), ("Beehive", 1), ("Loaf", 1), ("Boat", 1), ("Blinker", 2), ("Toad", 2), ("Beacon", 2), ("Pulsar", 3), ("Pentadecathlon", 15)]
    {
        let mut u = load(name);
        let start = u.cells();
        for g in 1..=period {
            u.step();
            assert_eq!(u.cells() == start, g == period, "{name} at generation {g}");
        }
    }
}

/// The glider moves (1, 1) every four generations; the three ordinary spaceships two cells along a
/// row, all three the same way.
#[test]
fn spaceships_travel_at_their_speeds() {
    let mut g = load("Glider");
    let start = g.cells();
    g.step_n(4);
    assert_eq!(translation(&start, &g.cells()), Some((1, 1)));
    let mut dirs = Vec::new();
    for name in ["Lightweight spaceship", "Middleweight spaceship", "Heavyweight spaceship"] {
        let mut u = load(name);
        let start = u.cells();
        for k in 1..4 {
            u.step();
            assert!(translation(&start, &u.cells()).is_none(), "{name}: period 4, not {k}");
        }
        u.step();
        let (dx, dy) = translation(&start, &u.cells()).unwrap_or_else(|| panic!("{name}: not a spaceship"));
        assert_eq!((dx.abs(), dy), (2, 0), "{name}");
        dirs.push(dx);
    }
    assert!(dirs.iter().all(|&d| d == dirs[0]));
}

/// LifeWiki: the R-pentomino settles at generation 1103 with 116 cells (six gliders among them),
/// which on an unbounded plane stay 116 as the gliders fly off.
#[test]
fn the_r_pentomino_settles_at_1103_with_116_cells() {
    let mut u = load("R-pentomino");
    u.step_n(1103);
    assert_eq!(u.population(), 116);
    for g in [1104, 1200, 2000] {
        u.step_n(g - u.generation());
        assert_eq!(u.population(), 116, "generation {g}");
    }
    let (x0, _, x1, _) = u.bounding_box().unwrap();
    assert!(x1 - x0 > 400, "the gliders are on their way (width {})", x1 - x0);
}

/// LifeWiki: diehard vanishes at generation 130.
#[test]
fn diehard_vanishes_at_130() {
    let mut u = load("Diehard");
    u.step_n(129);
    assert!(u.population() > 0);
    u.step();
    assert_eq!(u.population(), 0);
    assert_eq!(u.tile_count(), 0);
}

/// LifeWiki: acorn settles at generation 5206 with 633 cells.
#[test]
fn acorn_settles_at_5206_with_633_cells() {
    let mut u = load("Acorn");
    u.step_n(5206);
    assert_eq!(u.population(), 633);
    u.step_n(800);
    assert_eq!(u.population(), 633);
}

/// A gun adds one glider (five cells) a period, for ever.
#[test]
fn guns_emit_a_glider_each_period() {
    for (name, period) in [("Gosper glider gun", 30), ("Simkin glider gun", 120)] {
        let mut u = load(name);
        u.step_n(2 * period);
        let mut last = u.population();
        for k in 3..=12 {
            u.step_n(period);
            let pop = u.population();
            assert_eq!(pop, last + 5, "{name}: period {k}");
            last = pop;
        }
    }
}
