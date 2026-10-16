use super::*;
use std::io::ErrorKind as K;
use std::time::Duration;

#[test]
fn a_vanished_destination_retries_with_an_escalating_ladder() {
    // The field case: the SMB host rebooted, the path read as NotFound for minutes.
    let d = |a| match write_retry_policy(K::NotFound, Duration::ZERO, a) {
        WriteVerdict::RetryAfter(d) => d.as_secs(),
        WriteVerdict::Fatal => panic!("attempt {a} must retry"),
    };
    assert_eq!((d(0), d(1), d(4), d(7)), (1, 2, 30, 300));
    assert_eq!(d(20), 600); // past the ladder: 10 min, forever (the cap bounds it)
}

#[test]
fn conditions_that_cannot_fix_themselves_are_fatal_immediately() {
    for k in [K::StorageFull, K::PermissionDenied, K::ReadOnlyFilesystem, K::QuotaExceeded] {
        assert_eq!(write_retry_policy(k, Duration::ZERO, 0), WriteVerdict::Fatal);
    }
}

#[test]
fn unknown_kinds_retry_because_windows_hides_smb_errors_in_them() {
    // ERROR_BAD_NETPATH and friends surface as uncategorized kinds; retry-biased on purpose.
    assert!(matches!(
        write_retry_policy(K::Other, Duration::ZERO, 0),
        WriteVerdict::RetryAfter(_)
    ));
}

#[test]
fn the_total_wait_is_capped_and_then_it_gives_up_cleanly() {
    assert_eq!(
        write_retry_policy(K::NotFound, Duration::from_secs(30 * 60), 9),
        WriteVerdict::Fatal
    );
}
