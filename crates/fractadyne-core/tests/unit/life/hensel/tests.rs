use super::*;

fn class(count: u8, letter: char) -> Mask {
    let &(_, _, m) = CHART.iter().find(|&&(n, l, _)| n == count && l == letter).expect("a chart entry");
    canonical(m)
}

fn named(s: &str) -> (u8, char) {
    let mut cs = s.chars();
    let n = cs.next().unwrap().to_digit(10).unwrap() as u8;
    (n, cs.next().unwrap_or(' '))
}

/// The eight symmetries are a group acting on arrangements: each one is a bijection, the identity
/// is among them, and every class's size divides eight.
#[test]
fn the_symmetries_permute_the_arrangements() {
    for t in 0..8 {
        let mut seen = [false; 256];
        for m in 0..=255u8 {
            let u = transform(m, t);
            assert_eq!(u.count_ones(), m.count_ones(), "a symmetry keeps the count");
            assert!(!seen[u as usize], "symmetry {t} is not one-to-one");
            seen[u as usize] = true;
        }
    }
    assert!((0..=255u8).all(|m| transform(m, 0) == m));
    // A quarter turn moves N to E (y down: (0,-1) -> (1,0)), and the mirror moves W to E.
    assert_eq!(transform(M_N, 1), M_E);
    assert_eq!(transform(M_W, 4), M_E);
}

/// The chart names 51 classes that partition the 256 arrangements — 1, 2, 6, 10, 13, 10, 6, 2, 1
/// by count — and each picture has the count it is filed under.
#[test]
fn the_chart_partitions_the_arrangements() {
    let mut per_count = [0usize; 9];
    let mut reps: Vec<Mask> = Vec::new();
    for &(n, l, m) in &CHART {
        assert_eq!(m.count_ones() as u8, n, "{n}{l}: the picture has {} neighbours", m.count_ones());
        let c = canonical(m);
        assert!(!reps.contains(&c), "{n}{l} pictures a class another letter already names");
        reps.push(c);
        per_count[n as usize] += 1;
    }
    assert_eq!(per_count, [1, 2, 6, 10, 13, 10, 6, 2, 1]);
    let mut members = 0;
    for &c in &reps {
        members += (0..=255u8).filter(|&m| canonical(m) == c).count();
    }
    assert_eq!(members, 256, "every arrangement is in exactly one named class");
    for m in 0..=255u8 {
        let (n, l) = class_of(m);
        assert_eq!(n as u32, m.count_ones());
        assert_eq!(canonical(m), class(n, l));
    }
}

/// LifeWiki: "for any letter x and number n ≠ 4, nx is defined if and only if (8 − n)x is defined
/// and moreover (8 − n)x is the complement of nx". The chart's pictures for 5–8 were entered from
/// their own images, so this checks the two halves against each other: a swapped pair of letters
/// on one side shows up here.
#[test]
fn counts_above_four_are_the_complements_of_those_below() {
    for &(n, l, m) in &CHART {
        if n == 4 {
            continue;
        }
        assert!(is_letter_of(8 - n, l) || l == ' ', "{n}{l} has no partner {}{l}", 8 - n);
        assert_eq!(canonical(!m), class(8 - n, l), "{}{l} is not the complement of {n}{l}", 8 - n);
    }
}

/// LifeWiki's "von Neumann emulation" lists which classes share an arrangement of the four
/// orthogonal neighbours — text on the page, independent of the chart's pictures.
#[test]
fn the_von_neumann_groups_match_the_page() {
    // Each group: its classes and the orthogonal neighbours they all have (up to symmetry).
    let groups: [(&[&str], Mask); 6] = [
        (&["0", "1c", "2n", "2c", "3c", "4c"], 0),
        (&["1e", "2a", "2k", "3i", "3n", "3y", "3q", "4n", "4y", "5e"], M_N),
        (&["2e", "3k", "3a", "3j", "4k", "4a", "4q", "4w", "5a", "5j", "5k", "6e"], M_N | M_E),
        (&["2i", "3r", "4i", "4t", "4z", "5r", "6i"], M_N | M_S),
        (&["3e", "4j", "4r", "5i", "5n", "5y", "5q", "6a", "6k", "7e"], M_N | M_E | M_S),
        (&["4e", "5c", "6c", "6n", "7c", "8"], M_EDGES),
    ];
    let mut listed = 0;
    for (names, edges) in groups {
        for name in names {
            let (n, l) = named(name);
            let m = class(n, l);
            let e = m & M_EDGES;
            assert!((0..8).any(|t| transform(e, t) == edges), "{name}: orthogonal neighbours {e:#010b}");
            listed += 1;
        }
    }
    assert_eq!(listed, 51, "the groups cover every class once");
}

/// LifeWiki's duality table, checkerboard columns: for odd-parity cells the input is XORed with the
/// four edges, for even-parity cells with the four corners (and the centre, which the table records
/// as a birth/survival swap). Each row maps a class to a class — 51 more facts from the page's
/// text, not its pictures.
#[test]
fn the_checkerboard_dual_matches_the_page() {
    // (class, its image under XOR corners, its image under XOR edges)
    let table: [(&str, &str, &str); 51] = [
        ("0", "4c", "4e"),
        ("1c", "3c", "5c"),
        ("1e", "5e", "3e"),
        ("2c", "2c", "6c"),
        ("2e", "6e", "2e"),
        ("2k", "4n", "4r"),
        ("2a", "4y", "4j"),
        ("2i", "6i", "2i"),
        ("2n", "2n", "6n"),
        ("3c", "1c", "7c"),
        ("3e", "7e", "1e"),
        ("3k", "5a", "3a"),
        ("3a", "5k", "3k"),
        ("3i", "3y", "5y"),
        ("3n", "3n", "5n"),
        ("3y", "3i", "5i"),
        ("3q", "3q", "5q"),
        ("3j", "5j", "3j"),
        ("3r", "5r", "3r"),
        ("4c", "0", "8"),
        ("4e", "8", "0"),
        ("4k", "4a", "4a"),
        ("4a", "4k", "4k"),
        ("4i", "4i", "4t"),
        ("4n", "2k", "6k"),
        ("4y", "2a", "6a"),
        ("4q", "4w", "4q"),
        ("4j", "6a", "2a"),
        ("4r", "6k", "2k"),
        ("4t", "4t", "4i"),
        ("4w", "4q", "4w"),
        ("4z", "4z", "4z"),
        ("5c", "7c", "1c"),
        ("5e", "1e", "7e"),
        ("5k", "3a", "5a"),
        ("5a", "3k", "5k"),
        ("5i", "5y", "3y"),
        ("5n", "5n", "3n"),
        ("5y", "5i", "3i"),
        ("5q", "5q", "3q"),
        ("5j", "3j", "5j"),
        ("5r", "3r", "5r"),
        ("6c", "6c", "2c"),
        ("6e", "2e", "6e"),
        ("6k", "4r", "4n"),
        ("6a", "4j", "4y"),
        ("6i", "2i", "6i"),
        ("6n", "6n", "2n"),
        ("7c", "5c", "3c"),
        ("7e", "3e", "5e"),
        ("8", "4e", "4c"),
    ];
    for (from, by_corners, by_edges) in table {
        let (n, l) = named(from);
        let m = class(n, l);
        let (cn, cl) = named(by_corners);
        let (en, el) = named(by_edges);
        assert_eq!(canonical(m ^ M_CORNERS), class(cn, cl), "{from} XOR corners should be {by_corners}");
        assert_eq!(canonical(m ^ M_EDGES), class(en, el), "{from} XOR edges should be {by_edges}");
    }
}

#[test]
fn letters_are_listed_in_chart_order_per_count() {
    let got: Vec<String> = (0..=8).map(|n| letters_of(n).collect()).collect();
    assert_eq!(got, ["", "ce", "cekain", "cekainyqjr", "cekainyqjrtwz", "cekainyqjr", "cekain", "ce", ""]);
}
