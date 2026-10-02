//! The built-in sets (design/automata.md §4.6): named rules, and classic patterns entered from their
//! definitions. Each pattern's facts — period, displacement, population, when it settles — are
//! checked in `library/tests.rs`, so a mistyped cell fails a test rather than shipping.

/// A named rule.
#[derive(Clone, Copy, Debug)]
pub struct NamedRule {
    pub name: &'static str,
    /// A rule string [`super::Rule::parse`] reads (the canonical one).
    pub rule: &'static str,
    pub about: &'static str,
}

/// Which shelf of the pattern library a pattern sits on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    StillLife,
    Oscillator,
    Spaceship,
    Methuselah,
    Gun,
}

/// A named pattern, in RLE (body only: the rule is [`NamedPattern::rule`]).
#[derive(Clone, Copy, Debug)]
pub struct NamedPattern {
    pub name: &'static str,
    pub category: Category,
    pub rule: &'static str,
    pub rle: &'static str,
    pub about: &'static str,
}

pub const RULES: &[NamedRule] = &[
    NamedRule { name: "Life", rule: "B3/S23", about: "Conway's Game of Life." },
    NamedRule { name: "HighLife", rule: "B36/S23", about: "Life with birth on six too; has a small replicator." },
    NamedRule { name: "Day & Night", rule: "B3678/S34678", about: "Symmetric under swapping live and dead cells." },
    NamedRule { name: "Seeds", rule: "B2/S", about: "Every live cell dies; explosive growth from almost anything." },
    NamedRule { name: "Life without Death", rule: "B3/S012345678", about: "Cells never die; ladders and ink blots." },
    NamedRule { name: "34 Life", rule: "B34/S34", about: "Born and survives on three or four." },
    NamedRule { name: "2x2", rule: "B36/S125", about: "Patterns made of 2x2 blocks stay made of 2x2 blocks." },
    NamedRule { name: "Diamoeba", rule: "B35678/S5678", about: "Large diamonds with chaotic edges." },
    NamedRule { name: "Maze", rule: "B3/S12345", about: "Grows into mazes." },
    NamedRule { name: "Mazectric", rule: "B3/S1234", about: "Mazes with long straight corridors." },
    NamedRule { name: "Coral", rule: "B3/S45678", about: "Slow coral-like growth." },
    NamedRule { name: "Anneal", rule: "B4678/S35678", about: "The majority vote, with the ties reversed: blobs that smooth out." },
    NamedRule { name: "Long Life", rule: "B345/S5", about: "Long-lived oscillators." },
    NamedRule { name: "Morley", rule: "B368/S245", about: "Also called Move; many spaceships." },
    NamedRule { name: "Replicator", rule: "B1357/S1357", about: "Every pattern makes copies of itself." },
    NamedRule { name: "Gnarl", rule: "B1/S1", about: "Gnarled growth from a single cell." },
    NamedRule { name: "Amoeba", rule: "B357/S1358", about: "Amoeba-like blobs." },
    NamedRule { name: "Assimilation", rule: "B345/S4567", about: "Stable diamond shapes." },
    NamedRule { name: "Brian's Brain", rule: "B2/S/C3", about: "Generations: live cells always die, through one dying state." },
    NamedRule { name: "Star Wars", rule: "B2/S345/C4", about: "Generations with two dying states." },
    NamedRule { name: "tlife", rule: "B3/S2-i34q", about: "Non-totalistic: Life where two opposite orthogonal neighbours don't keep a cell alive." },
    NamedRule { name: "Just Friends", rule: "B2-a/S12", about: "Non-totalistic: born on two neighbours unless they sit side by side (2a)." },
];

pub const PATTERNS: &[NamedPattern] = &[
    NamedPattern { name: "Block", category: Category::StillLife, rule: "B3/S23", rle: "2o$2o!", about: "The smallest still life." },
    NamedPattern { name: "Beehive", category: Category::StillLife, rule: "B3/S23", rle: "b2o$o2bo$b2o!", about: "A six-cell still life." },
    NamedPattern { name: "Loaf", category: Category::StillLife, rule: "B3/S23", rle: "b2o$o2bo$bobo$2bo!", about: "A seven-cell still life." },
    NamedPattern { name: "Boat", category: Category::StillLife, rule: "B3/S23", rle: "2o$obo$bo!", about: "A five-cell still life." },
    NamedPattern { name: "Blinker", category: Category::Oscillator, rule: "B3/S23", rle: "3o!", about: "Period 2: the smallest oscillator." },
    NamedPattern { name: "Toad", category: Category::Oscillator, rule: "B3/S23", rle: "b3o$3o!", about: "Period 2." },
    NamedPattern { name: "Beacon", category: Category::Oscillator, rule: "B3/S23", rle: "2o$2o$2b2o$2b2o!", about: "Period 2." },
    NamedPattern {
        name: "Pulsar",
        category: Category::Oscillator,
        rule: "B3/S23",
        rle: "2b3o3b3o2$o4bobo4bo$o4bobo4bo$o4bobo4bo$2b3o3b3o2$2b3o3b3o$o4bobo4bo$o4bobo4bo$o4bobo4bo2$2b3o3b3o!",
        about: "Period 3.",
    },
    NamedPattern { name: "Pentadecathlon", category: Category::Oscillator, rule: "B3/S23", rle: "2bo4bo$2ob4ob2o$2bo4bo!", about: "Period 15." },
    NamedPattern { name: "Glider", category: Category::Spaceship, rule: "B3/S23", rle: "bo$2bo$3o!", about: "Moves one cell diagonally every four generations." },
    NamedPattern { name: "Lightweight spaceship", category: Category::Spaceship, rule: "B3/S23", rle: "bo2bo$o4b$o3bo$4o!", about: "Moves two cells every four generations." },
    NamedPattern { name: "Middleweight spaceship", category: Category::Spaceship, rule: "B3/S23", rle: "3bo2b$bo3bo$o5b$o4bo$5o!", about: "Moves two cells every four generations." },
    NamedPattern { name: "Heavyweight spaceship", category: Category::Spaceship, rule: "B3/S23", rle: "3b2o2b$bo4bo$o6b$o5bo$6o!", about: "Moves two cells every four generations." },
    NamedPattern { name: "R-pentomino", category: Category::Methuselah, rule: "B3/S23", rle: "b2o$2o$bo!", about: "Five cells that settle only at generation 1103." },
    NamedPattern { name: "Diehard", category: Category::Methuselah, rule: "B3/S23", rle: "6bo$2o$bo3b3o!", about: "Vanishes completely at generation 130." },
    NamedPattern { name: "Acorn", category: Category::Methuselah, rule: "B3/S23", rle: "bo$3bo$2o2b3o!", about: "Seven cells that settle at generation 5206 with 633." },
    NamedPattern {
        name: "Gosper glider gun",
        category: Category::Gun,
        rule: "B3/S23",
        rle: "24bo$22bobo$12b2o6b2o12b2o$11bo3bo4b2o12b2o$2o8bo5bo3b2o$2o8bo3bob2o4bobo$10bo5bo7bo$11bo3bo$12b2o!",
        about: "The first known gun: a glider every 30 generations.",
    },
    NamedPattern {
        name: "Simkin glider gun",
        category: Category::Gun,
        rule: "B3/S23",
        rle: "2o5b2o$2o5b2o2$4b2o$4b2o5$22b2ob2o$21bo5bo$21bo6bo2b2o$21b3o3bo3b2o$26bo4$20b2o$20bo$21b3o$23bo!",
        about: "A glider every 120 generations.",
    },
];

/// A named rule by name, ignoring case.
pub fn rule_named(name: &str) -> Option<&'static NamedRule> {
    RULES.iter().find(|r| r.name.eq_ignore_ascii_case(name.trim()))
}

/// A named pattern by name, ignoring case.
pub fn pattern_named(name: &str) -> Option<&'static NamedPattern> {
    PATTERNS.iter().find(|p| p.name.eq_ignore_ascii_case(name.trim()))
}

#[cfg(test)]
mod tests;
