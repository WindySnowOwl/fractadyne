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
