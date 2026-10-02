//! Hensel's notation for non-totalistic isotropic rules (design/automata.md §4.1): the eight
//! neighbours' 256 arrangements fall into 51 classes under the square's symmetries (rotations and
//! reflections), named by a count and a letter — `2a`, `3q`, `4z`.
//!
//! The letters come from LifeWiki's chart ("Isotropic non-totalistic rule"), one picture per class,
//! entered here as one representative arrangement each — every count's, 5–8 included, so the chart
//! can be checked against itself (`(8−n)x` is the complement of `nx`) and against the page's
//! symmetry tables (`hensel/tests.rs`).

/// An arrangement of the eight neighbours, one bit each: NW is the most significant bit and SE the
/// least, row by row — the neighbour bits of a table index (`rule.rs`) with the centre taken out.
pub(crate) type Mask = u8;

pub(crate) const M_NW: Mask = 1 << 7;
pub(crate) const M_N: Mask = 1 << 6;
pub(crate) const M_NE: Mask = 1 << 5;
pub(crate) const M_W: Mask = 1 << 4;
pub(crate) const M_E: Mask = 1 << 3;
pub(crate) const M_SW: Mask = 1 << 2;
pub(crate) const M_S: Mask = 1 << 1;
pub(crate) const M_SE: Mask = 1;

/// The orthogonal neighbours: what a von Neumann rule counts.
pub(crate) const M_EDGES: Mask = M_N | M_W | M_E | M_S;
/// The diagonal neighbours.
#[cfg(test)]
pub(crate) const M_CORNERS: Mask = M_NW | M_NE | M_SW | M_SE;

/// The letters in the chart's order, which is also the order a canonical rule string lists them in.
pub(crate) const LETTERS: [char; 13] = ['c', 'e', 'k', 'a', 'i', 'n', 'y', 'q', 'j', 'r', 't', 'w', 'z'];

/// LifeWiki's chart: `(count, letter, one arrangement of the class)`. Counts 0 and 8 have one class
/// each and no letter (`' '`).
pub(crate) const CHART: [(u8, char, Mask); 51] = [
    (0, ' ', 0),
    (1, 'c', M_NW),
    (1, 'e', M_N),
    (2, 'c', M_NW | M_NE),
    (2, 'e', M_N | M_W),
    (2, 'k', M_N | M_SE),
    (2, 'a', M_NW | M_N),
    (2, 'i', M_N | M_S),
    (2, 'n', M_NW | M_SE),
    (3, 'c', M_NW | M_NE | M_SE),
    (3, 'e', M_N | M_W | M_E),
    (3, 'k', M_N | M_W | M_SE),
    (3, 'a', M_NW | M_N | M_W),
    (3, 'i', M_NW | M_W | M_SW),
    (3, 'n', M_NW | M_NE | M_W),
    (3, 'y', M_NW | M_NE | M_S),
    (3, 'q', M_NW | M_W | M_SE),
    (3, 'j', M_NE | M_E | M_S),
    (3, 'r', M_N | M_NE | M_S),
    (4, 'c', M_NW | M_NE | M_SW | M_SE),
    (4, 'e', M_N | M_W | M_E | M_S),
    (4, 'k', M_N | M_NE | M_W | M_SE),
    (4, 'a', M_NW | M_W | M_SW | M_S),
    (4, 'i', M_NW | M_NE | M_W | M_E),
    (4, 'n', M_NW | M_W | M_SW | M_SE),
    (4, 'y', M_NW | M_NE | M_SW | M_S),
    (4, 'q', M_NW | M_N | M_W | M_SE),
    (4, 'j', M_NE | M_W | M_E | M_S),
    (4, 'r', M_N | M_NE | M_E | M_S),
    (4, 't', M_NW | M_N | M_NE | M_S),
    (4, 'w', M_NW | M_W | M_S | M_SE),
    (4, 'z', M_NW | M_N | M_S | M_SE),
    (5, 'c', M_N | M_W | M_E | M_SW | M_S),
    (5, 'e', M_NW | M_NE | M_SW | M_S | M_SE),
    (5, 'k', M_NW | M_NE | M_E | M_SW | M_S),
    (5, 'a', M_NE | M_E | M_SW | M_S | M_SE),
    (5, 'i', M_N | M_NE | M_E | M_S | M_SE),
    (5, 'n', M_N | M_E | M_SW | M_S | M_SE),
    (5, 'y', M_N | M_W | M_E | M_SW | M_SE),
    (5, 'q', M_N | M_NE | M_E | M_SW | M_S),
    (5, 'j', M_NW | M_N | M_W | M_SW | M_SE),
    (5, 'r', M_NW | M_W | M_E | M_SW | M_SE),
    (6, 'c', M_N | M_W | M_E | M_SW | M_S | M_SE),
    (6, 'e', M_NW | M_NE | M_E | M_SW | M_S | M_SE),
    (6, 'k', M_NW | M_NE | M_W | M_E | M_SW | M_S),
    (6, 'a', M_NE | M_W | M_E | M_SW | M_S | M_SE),
    (6, 'i', M_NW | M_NE | M_W | M_E | M_SW | M_SE),
    (6, 'n', M_N | M_NE | M_W | M_E | M_SW | M_S),
    (7, 'c', M_N | M_NE | M_W | M_E | M_SW | M_S | M_SE),
    (7, 'e', M_NW | M_NE | M_W | M_E | M_SW | M_S | M_SE),
    (8, ' ', 0xFF),
];

/// Neighbour positions as `(dx, dy)`, y down, in mask-bit order (bit 7 first).
const POS: [(i8, i8); 8] = [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)];

fn bit_of(dx: i8, dy: i8) -> Mask {
    let i = POS.iter().position(|&p| p == (dx, dy)).expect("a neighbour position");
    1 << (7 - i)
}

/// `mask` under symmetry `t` (0–7): `t & 3` quarter turns, then a mirror if `t & 4`.
pub(crate) fn transform(mask: Mask, t: u8) -> Mask {
    let mut out = 0;
    for (i, &(x, y)) in POS.iter().enumerate() {
        if mask & (1 << (7 - i)) == 0 {
            continue;
        }
        let (mut dx, mut dy) = (x, y);
        for _ in 0..(t & 3) {
            (dx, dy) = (-dy, dx);
        }
        if t & 4 != 0 {
            dx = -dx;
        }
        out |= bit_of(dx, dy);
    }
    out
}

/// The class's smallest member: equal for two arrangements exactly when a symmetry maps one to
/// the other.
pub(crate) fn canonical(mask: Mask) -> Mask {
    (0..8).map(|t| transform(mask, t)).min().expect("eight symmetries")
}

/// For every arrangement, its row in [`CHART`].
fn class_table() -> &'static [u8; 256] {
    static TABLE: std::sync::OnceLock<[u8; 256]> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        let reps: Vec<Mask> = CHART.iter().map(|&(_, _, m)| canonical(m)).collect();
        let mut table = [0u8; 256];
        for (mask, slot) in table.iter_mut().enumerate() {
            let c = canonical(mask as Mask);
            *slot = reps.iter().position(|&r| r == c).expect("the chart names every class") as u8;
        }
        table
    })
}

/// The `(count, letter)` naming `mask`'s class (letter `' '` for counts 0 and 8).
pub(crate) fn class_of(mask: Mask) -> (u8, char) {
    let (n, l, _) = CHART[class_table()[mask as usize] as usize];
    (n, l)
}

/// Whether `letter` names a class of `count` neighbours.
pub(crate) fn is_letter_of(count: u8, letter: char) -> bool {
    letter != ' ' && CHART.iter().any(|&(n, l, _)| n == count && l == letter)
}

/// The letters naming `count`'s classes, in chart order (none for 0 and 8).
pub(crate) fn letters_of(count: u8) -> impl Iterator<Item = char> {
    LETTERS.into_iter().filter(move |&l| is_letter_of(count, l))
}

#[cfg(test)]
mod tests;
