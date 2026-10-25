use super::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const POLL: Duration = Duration::from_millis(5);

/// Runs `wait_alive` over a counter the work advances; returns its result and the liveness stamps.
fn run(work: impl FnOnce(Arc<AtomicU64>) -> u32 + Send + 'static) -> (Option<u32>, u32) {
    let steps = Arc::new(AtomicU64::new(0));
    let s = steps.clone();
    let mut stamps = 0u32;
    let got = wait_alive("wait-alive-test", move || work(s), || steps.load(Ordering::Relaxed), || stamps += 1, |_| {}, POLL);
    (got, stamps)
}

#[test]
fn work_that_advances_is_stamped_alive_and_returns_its_result() {
    let (got, stamps) = run(|s| {
        for _ in 0..20 {
            std::thread::sleep(Duration::from_millis(10));
            s.fetch_add(1024, Ordering::Relaxed);
        }
        7
    });
    assert_eq!(got, Some(7));
    assert!(stamps >= 5, "20 advances over 200 ms stamped {stamps} times");
}

#[test]
fn work_that_does_not_advance_is_never_stamped_alive() {
    // A wedge: the thread runs but its count stands still. Stamping it would blind the watchdog.
    let (got, stamps) = run(|_| {
        std::thread::sleep(Duration::from_millis(200));
        3
    });
    assert_eq!(got, Some(3));
    assert_eq!(stamps, 0);
}

#[test]
fn a_stall_after_progress_stops_the_stamps() {
    let (got, stamps) = run(|s| {
        for _ in 0..5 {
            std::thread::sleep(Duration::from_millis(10));
            s.fetch_add(1, Ordering::Relaxed);
        }
        std::thread::sleep(Duration::from_millis(200));
        1
    });
    assert_eq!(got, Some(1));
    assert!((1..=5).contains(&stamps), "five advances then 200 ms still: {stamps} stamps");
}

#[test]
fn a_build_that_panics_is_none() {
    let (got, _) = run(|_| panic!("the build failed"));
    assert_eq!(got, None);
}
