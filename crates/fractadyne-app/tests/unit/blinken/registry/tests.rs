use super::*;
use std::sync::Mutex;

// The counts are process-wide and tests run in parallel: these assert on differences, in the
// Export role, which nothing else in the test binary marks, one test at a time.
static SERIAL: Mutex<()> = Mutex::new(());

#[test]
fn a_guard_counts_its_threads_while_it_lives() {
    let _serial = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    let before = snapshot()[Role::Export as usize];
    {
        let _one = busy(Role::Export);
        let _pool = busy_n(Role::Export, 16);
        assert_eq!(snapshot()[Role::Export as usize], before + 17);
    }
    assert_eq!(snapshot()[Role::Export as usize], before);
}

#[test]
fn guards_dropped_out_of_order_leave_the_count_right() {
    let _serial = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
    let before = snapshot()[Role::Export as usize];
    let a = busy_n(Role::Export, 3);
    let b = busy(Role::Export);
    drop(a);
    assert_eq!(snapshot()[Role::Export as usize], before + 1);
    drop(b);
    assert_eq!(snapshot()[Role::Export as usize], before);
}
