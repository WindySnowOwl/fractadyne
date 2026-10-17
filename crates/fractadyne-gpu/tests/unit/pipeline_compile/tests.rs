use super::*;

/// A compile is in flight exactly while it runs, is counted when it ends, and its time is kept.
/// One test, because the account is process-wide.
#[test]
fn a_compile_is_in_flight_while_it_runs_and_counted_when_it_ends() {
    let done = compiles_done();
    let total = compile_ms_total();
    assert_eq!(compile_in_flight_ms(), None, "nothing compiles before the test");
    let seen = compiling(|| {
        std::thread::sleep(std::time::Duration::from_millis(30));
        compile_in_flight_ms()
    });
    assert!(seen.is_some_and(|ms| ms >= 25), "in flight while it runs, with its age: {seen:?}");
    assert_eq!(compile_in_flight_ms(), None, "not in flight once it returns");
    assert_eq!(compiles_done(), done + 1, "counted when it ends");
    assert!(compile_ms_total() >= total + 25, "its time is kept");
    assert!(compile_ms_max() >= 25);

    // A compile that panics does not stay in flight: the watchdog would wait on it forever.
    let r = std::panic::catch_unwind(|| compiling(|| panic!("a driver error")));
    assert!(r.is_err());
    assert_eq!(compile_in_flight_ms(), None, "a panicking compile is not left in flight");
}

/// Only a REPEAT can say the driver's cache did not keep a pipeline: a first compile is slow on any
/// cold cache. A repeat a working cache answers (~7 ms) says nothing.
#[test]
fn only_a_slow_repeat_says_the_driver_cache_is_not_keeping() {
    assert!(!slow_repeat(false, 9_500), "a first compile, however slow");
    assert!(!slow_repeat(true, 7), "a repeat the cache answered");
    assert!(!slow_repeat(true, SLOW_REPEAT_MS - 1));
    assert!(slow_repeat(true, SLOW_REPEAT_MS), "a repeat that compiled");
    assert!(slow_repeat(true, 7_500), "the gather pipeline, measured 2026-10-16 with the cache full");
}

/// The record keeps the slowest repeat and counts them all; it is reported from the second.
#[test]
fn the_record_keeps_the_slowest_repeat_and_counts_them() {
    let a = worse(None, "gather.pipeline fs_iterate_gather", 7_500);
    assert_eq!((a.ms, a.count), (7_500, 1));
    let b = worse(Some(a), "export.iter_pipeline fs_iterate", 1_100);
    assert_eq!((b.pipeline.as_str(), b.ms, b.count), ("gather.pipeline fs_iterate_gather", 7_500, 2));
    let c = worse(Some(b), "export.iter_pipeline fs_iterate", 9_500);
    assert_eq!((c.pipeline.as_str(), c.ms, c.count), ("export.iter_pipeline fs_iterate", 9_500, 3));
    assert!(SLOW_REPEATS_TO_REPORT >= 2, "one slow repeat could be a disk stall");
}
