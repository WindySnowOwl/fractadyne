use super::*;

#[test]
fn proc_stat_gives_one_idle_and_total_per_core_and_skips_the_aggregate() {
    let text = "cpu  100 0 50 800 50 0 0 0 0 0\n\
                cpu0 10 0 5 80 5 0 0 0 0 0\n\
                cpu1 60 0 30 10 0 0 0 0 0 0\n\
                intr 12345\n\
                cpufreq is not a core\n";
    let cores = parse_proc_stat_cores(text);
    // idle = idle + iowait; total = every field.
    assert_eq!(cores, vec![(85, 100), (10, 100)]);
}

#[test]
fn a_core_is_as_busy_as_the_share_of_its_time_not_idle() {
    let prev = [(100, 1000), (500, 1000), (0, 0)];
    let now = [(150, 1100), (500, 1100), (0, 0)];
    let busy = core_busy(&prev, &now);
    assert!((busy[0] - 0.5).abs() < 1e-6, "{busy:?}");
    assert!((busy[1] - 1.0).abs() < 1e-6, "{busy:?}");
    // No time passed: idle, not a division by zero.
    assert_eq!(busy[2], 0.0);
}

#[test]
fn the_local_clock_reads_a_plausible_date_where_it_is_supported() {
    if let Some((y, mo, d, h, mi, s, ms)) = local_now() {
        assert!((2020..2200).contains(&y));
        assert!((1..=12).contains(&mo) && (1..=31).contains(&d));
        assert!(h < 24 && mi < 60 && s < 61 && ms < 1000);
    }
}
