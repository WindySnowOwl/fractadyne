use super::mem_timeline::*;
use std::collections::VecDeque;

fn reading(t_s: f64, commit_gb: f64) -> Reading {
    let b = |gb: f64| (gb * (1u64 << 30) as f64) as u64;
    Reading {
        t_s,
        working_set: b(0.35),
        peak_working_set: b(0.40),
        commit: b(commit_gb),
        peak_commit: b(commit_gb),
        system_free: Some(b(31.0)),
        gpus: vec![(
            "main".into(),
            GpuCounts { buffers: 312, textures: 48, views: 96, bind_groups: 211, buffer_bytes: 1 << 30, texture_bytes: 2 << 30, allocations: 57 },
        )],
    }
}

/// A reading's line says each figure once, in GB, and each device by its name.
#[test]
fn a_reading_says_the_process_the_system_and_each_device() {
    let l = line(&reading(600.4, 2.59));
    assert_eq!(
        l,
        "t=600s commit 2.59 GB (peak 2.59 GB), working set 358 MB (peak 409 MB), system free 31.00 GB; \
         main: 312 buffers 1.00 GB, 48 textures 2.00 GB, 96 views, 211 bind groups, 57 allocations"
    );
    // A platform that does not say what is committed: no commit, and no device.
    let mut r = reading(60.0, 0.0);
    r.gpus.clear();
    r.system_free = None;
    assert_eq!(line(&r), "t=60s working set 358 MB (peak 409 MB)");
}

/// The ring keeps the newest readings.
#[test]
fn the_ring_keeps_the_newest() {
    let mut ring = VecDeque::new();
    for i in 0..KEEP + 7 {
        push_into(&mut ring, reading(i as f64 * 60.0, 1.0));
    }
    assert_eq!(ring.len(), KEEP);
    assert_eq!(ring.front().unwrap().t_s, 7.0 * 60.0);
    assert_eq!(ring.back().unwrap().t_s, (KEEP + 6) as f64 * 60.0);
}

/// The crash report prints the session's first readings and the latest, and says how many it left
/// out; a short history whole; none, says so.
#[test]
fn the_report_prints_the_start_and_the_end() {
    let all: Vec<Reading> = (0..100).map(|i| reading(i as f64 * 60.0, 1.0 + i as f64 * 0.1)).collect();
    let s = report_section(&all);
    let rows: Vec<&str> = s.lines().skip(1).collect();
    assert_eq!(rows.len(), REPORT_FIRST + 1 + REPORT_LAST, "{s}");
    assert!(rows[0].contains("t=0s") && rows[REPORT_FIRST - 1].contains("t=120s"));
    assert!(rows[REPORT_FIRST].contains("57 readings left out"), "{}", rows[REPORT_FIRST]);
    assert!(rows[REPORT_FIRST + 1].contains("t=3600s") && rows.last().unwrap().contains("t=5940s"));
    let few = report_section(&all[..5]);
    assert_eq!(few.lines().count(), 1 + 5);
    assert!(!few.contains("left out"));
    assert!(report_section(&[]).contains("no reading"));
}
