use super::*;
use crate::FractalKind;
use CellSource::{Recolor, Render};

/// A cell plan flattened to `(method key, shape key or density, source)`, for readable asserts.
fn flat(plan: &[(MethodGroup, Vec<(SheetCell, CellSource)>)]) -> Vec<(&'static str, String, CellSource)> {
    plan.iter()
        .flat_map(|(_, cs)| cs.iter())
        .map(|(c, s)| {
            let v = match (c.trap, c.stripe_freq) {
                (Some(t), _) => t.key().to_string(),
                (_, Some(f)) => format!("{f}"),
                _ => String::new(),
            };
            (c.method.key(), v, *s)
        })
        .collect()
}

fn row(m: &'static str, v: &str, s: CellSource) -> (&'static str, String, CellSource) {
    (m, v.to_string(), s)
}

/// A shallow Smooth frame accumulates nothing: the count and the distance recolor from the main
/// lane, which every frame has; every statistic needs a calculation of its own.
#[test]
fn a_frame_without_statistics_recolors_only_the_main_lane() {
    let frame = FrameStats { set: StatSet::NONE, stripe_freq: 2.0, trap: TrapType::Point };
    assert_eq!(
        flat(&sheet_plan(FractalKind::Mandelbrot.caps(), frame)),
        [
            row("smooth", "", Recolor),
            row("distance", "", Recolor),
            row("stripe", "2", Render),
            row("stripe", "3", Render),
            row("stripe", "6", Render),
            row("triangle", "", Render),
            row("trap", "point", Render),
            row("trap", "cross", Render),
            row("trap", "circle", Render),
            row("decomposition", "", Render),
        ]
    );
}

/// A deep stripe, triangle or trap frame carries the G-buffer (B3): the cells it holds with the
/// frame's own parameters recolor (the stripe at its density, the triangle, the trap in its shape);
/// another density or shape, and the final angle, need their own.
#[test]
fn a_gbuffer_frame_recolors_what_it_holds_with_its_own_parameters() {
    let frame = FrameStats { set: StatSet::GBUFFER, stripe_freq: 2.0, trap: TrapType::Cross };
    assert_eq!(
        flat(&sheet_plan(FractalKind::Mandelbrot.caps(), frame)),
        [
            row("smooth", "", Recolor),
            row("distance", "", Recolor),
            row("stripe", "2", Recolor),
            row("stripe", "3", Render),
            row("stripe", "6", Render),
            row("triangle", "", Recolor),
            row("trap", "point", Render),
            row("trap", "cross", Recolor),
            row("trap", "circle", Render),
            row("decomposition", "", Render),
        ]
    );
    let angle = FrameStats { set: StatSet::of(Stat::Angle), ..frame };
    let plan = flat(&sheet_plan(FractalKind::Mandelbrot.caps(), angle));
    assert!(plan.contains(&row("decomposition", "", Recolor)), "a decomposition frame holds its angle");
    assert!(plan.contains(&row("triangle", "", Render)));
}

/// A family has a cell only for what it can color by: no distance where there is no derivative, and
/// on Newton, which records no orbit statistics, only the count.
#[test]
fn a_family_has_cells_only_for_what_it_can_color_by() {
    let frame = FrameStats { set: StatSet::NONE, stripe_freq: 2.0, trap: TrapType::Point };
    let ship = flat(&sheet_plan(FractalKind::BurningShip.caps(), frame));
    assert!(!ship.iter().any(|(m, _, _)| *m == "distance"), "{ship:?}");
    assert!(ship.iter().any(|(m, _, _)| *m == "stripe"));
    let newton = sheet_plan(FractalKind::Newton.caps(), frame);
    let methods: Vec<_> = flat(&newton).into_iter().map(|(m, _, _)| m).collect();
    assert!(!methods.iter().any(|m| ["stripe", "triangle", "trap", "decomposition"].contains(m)), "{methods:?}");
    assert!(newton.iter().all(|(_, cs)| !cs.is_empty()), "no empty heading");
}

/// The stripe's cells: the density in use, and 3 and 6, each once, in order.
#[test]
fn the_stripe_densities_are_the_one_in_use_and_three_and_six_once_each() {
    let freqs = |f: f32| -> Vec<f32> {
        let frame = FrameStats { set: StatSet::NONE, stripe_freq: f, trap: TrapType::Point };
        sheet_plan(FractalKind::Mandelbrot.caps(), frame)
            .iter()
            .flat_map(|(_, cs)| cs.iter())
            .filter_map(|(c, _)| c.stripe_freq)
            .collect()
    };
    assert_eq!(freqs(3.0), [3.0, 6.0]);
    assert_eq!(freqs(4.5), [3.0, 4.5, 6.0]);
    assert_eq!(freqs(8.0), [3.0, 6.0, 8.0]);
}

/// Short groups share a row; a group longer than a row starts its own and is cut into runs, the
/// heading on the first; a short group after it joins its last run's row when it fits.
#[test]
fn short_groups_share_a_row_and_long_ones_wrap() {
    let frame = FrameStats { set: StatSet::NONE, stripe_freq: 2.0, trap: TrapType::Point };
    let plan = sheet_plan(FractalKind::Mandelbrot.caps(), frame);
    let shape = |cols: usize| -> Vec<Vec<(MethodGroup, bool, usize)>> {
        sheet_rows(&plan, cols).iter().map(|r| r.iter().map(|s| (s.group, s.first, s.cells.len())).collect()).collect()
    };
    use MethodGroup::*;
    assert_eq!(
        shape(4),
        [
            vec![(EscapeCount, true, 1), (Distance, true, 1)],
            vec![(OrbitStatistics, true, 4)],
            vec![(OrbitStatistics, false, 3), (Angle, true, 1)],
        ]
    );
    assert_eq!(shape(1).len(), 10, "one cell a row");
    let all: usize = sheet_rows(&plan, 3).iter().flatten().map(|s| s.cells.len()).sum();
    assert_eq!(all, 10, "every cell placed once");
}
