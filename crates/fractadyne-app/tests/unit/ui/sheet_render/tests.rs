use super::*;

fn priced(secs: f64) -> CellStatus {
    CellStatus { estimate: Some(Estimate { secs, at_least: false }), ..CellStatus::default() }
}

#[test]
fn a_probe_prices_its_cell_by_the_pixels_and_adds_the_fixed_cost_once() {
    // 1,000 probe pixels took 10 ms on the GPU; the cell has 64 times the pixels.
    let p = Probe { gpu_ms: 10.0, wall_ms: 40.0, px: 1_000, stopped: false };
    let e = estimate(p, 64_000, 25.0);
    assert!(!e.at_least);
    assert!((e.secs - (640.0 + 25.0) / 1000.0).abs() < 1e-9, "{e:?}");
    // No GPU timer: the probe's whole time, scaled (it over-prices; it never under-prices).
    let e = estimate(Probe { gpu_ms: 0.0, ..p }, 64_000, 25.0);
    assert!((e.secs - 40.0 * 64.0 / 1000.0).abs() < 1e-9, "{e:?}");
}

#[test]
fn a_stopped_probe_gives_a_floor_not_an_estimate() {
    let p = Probe { gpu_ms: 0.0, wall_ms: 2_000.0, px: 1_000, stopped: true };
    let e = estimate(p, 64_000, 25.0);
    assert!(e.at_least);
    assert!((e.secs - 128.0).abs() < 1e-9, "{e:?}");
}

#[test]
fn every_cell_is_priced_before_any_is_paid_for_and_an_asked_cell_goes_first() {
    let cells = [priced(0.5), CellStatus::default(), priced(0.1)];
    assert_eq!(next_job(&cells, 2.0), Some((1, JobKind::Probe)));
    // Once all are priced, the cells within the budget, in the sheet's order.
    let cells = [priced(5.0), priced(0.5), priced(0.1)];
    assert_eq!(next_job(&cells, 2.0), Some((1, JobKind::Full)));
    // A cell asked for is calculated first, whatever its price, even before the probes.
    let cells = [CellStatus::default(), CellStatus { asked: true, ..priced(90.0) }];
    assert_eq!(next_job(&cells, 2.0), Some((1, JobKind::Full)));
}

#[test]
fn nothing_past_the_budget_or_with_a_floor_starts_by_itself() {
    let floor = CellStatus { estimate: Some(Estimate { secs: 0.5, at_least: true }), ..CellStatus::default() };
    assert_eq!(next_job(&[priced(2.5), floor], 2.0), None);
    // Rendered and failed cells are done with.
    let done = CellStatus { rendered: true, ..priced(0.1) };
    let failed = CellStatus { failed: true, ..CellStatus::default() };
    assert_eq!(next_job(&[done, failed], 2.0), None);
    // A budget of 0 (the UI test's): only probes, then nothing.
    assert_eq!(next_job(&[CellStatus::default()], 0.0), Some((0, JobKind::Probe)));
    assert_eq!(next_job(&[priced(0.01)], 0.0), None);
}

#[test]
fn times_read_as_minutes_and_seconds_rounded_up() {
    assert_eq!(fmt_eta(0.0), "0:01");
    assert_eq!(fmt_eta(0.2), "0:01");
    assert_eq!(fmt_eta(12.1), "0:13");
    assert_eq!(fmt_eta(60.0), "1:00");
    assert_eq!(fmt_eta(3_599.0), "59:59");
    assert_eq!(fmt_eta(3_600.0), "1:00:00");
    assert_eq!(fmt_eta(7_384.0), "2:03:04");
}

#[test]
fn the_probe_is_an_eighth_of_the_cell_per_side_and_never_tiny() {
    assert_eq!(probe_size([440, 280]), [55, 35]);
    assert_eq!(probe_size([40, 20]), [8, 8]);
}
