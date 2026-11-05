//! A background thread's standing reaches every thread it spawns, and the priority hook runs once
//! on each of them and on no other thread.

use super::*;

thread_local! {
    /// How many times the hook ran on this thread. Per thread, so tests running side by side in
    /// one process cannot disturb each other's counts.
    static LOWERED: Cell<u32> = const { Cell::new(0) };
}

fn count_lowering() {
    LOWERED.with(|n| n.set(n.get() + 1));
}

fn lowered() -> u32 {
    LOWERED.with(Cell::get)
}

#[test]
fn a_thread_starts_in_the_foreground() {
    install(count_lowering);
    std::thread::spawn(|| {
        assert!(!is_background());
        assert_eq!(current(), Standing(false));
        assert_eq!(lowered(), 0);
    })
    .join()
    .unwrap();
}

#[test]
fn entering_the_background_lowers_the_thread_once() {
    install(count_lowering);
    std::thread::spawn(|| {
        enter_background();
        assert!(is_background());
        assert_eq!(lowered(), 1);
        enter_background();
        assert_eq!(lowered(), 1, "a second call must not lower the thread again");
    })
    .join()
    .unwrap();
}

#[test]
fn the_standing_reaches_every_thread_a_background_thread_spawns() {
    install(count_lowering);
    std::thread::spawn(|| {
        enter_background();
        let standing = current();
        std::thread::scope(|s| {
            for _ in 0..3 {
                s.spawn(move || {
                    standing.adopt();
                    assert!(is_background());
                    assert_eq!(lowered(), 1);
                    // And from there one level further, as the pick's pool does inside a walk.
                    let inner = current();
                    std::thread::scope(|s2| {
                        s2.spawn(move || {
                            inner.adopt();
                            assert!(is_background());
                            assert_eq!(lowered(), 1);
                        });
                    });
                });
            }
        });
    })
    .join()
    .unwrap();
}

#[test]
fn a_foreground_thread_spawns_foreground_workers() {
    // The control: the same spawning from a thread that never entered the background.
    install(count_lowering);
    std::thread::spawn(|| {
        let standing = current();
        std::thread::scope(|s| {
            s.spawn(move || {
                standing.adopt();
                assert!(!is_background());
                assert_eq!(lowered(), 0);
            });
        });
    })
    .join()
    .unwrap();
}
