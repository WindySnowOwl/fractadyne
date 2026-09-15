//! Failure-path coverage for the bounded GPU-completion wait (Phase A hang-safety). These exercise
//! the decision logic — a lost device (dropped callback) resolving to `DeviceLost` instead of a
//! hang, cancel interrupting the wait, a finished readback never being discarded as "canceled",
//! and the deadline backstop — WITHOUT a GPU, by testing the two pure helpers the poll loop is
//! built from (`recv_readback_outcome`, `readback_step`).

use super::*;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::TryRecvError;
use std::time::{Duration, Instant};

// ---- recv_readback_outcome: the readback channel → outcome mapping ----

#[test]
fn recv_maps_each_channel_state() {
    assert_eq!(recv_readback_outcome(Ok(Ok(()))), Some(ReadbackWait::Ready));
    // A dropped sender = the map_async callback was discarded (the device-loss shape). It must read
    // as DeviceLost, NOT as "still waiting" — that is what lets the worker escape instead of hang.
    assert_eq!(
        recv_readback_outcome(Err(TryRecvError::Disconnected)),
        Some(ReadbackWait::DeviceLost)
    );
    // Nothing delivered yet → keep polling.
    assert_eq!(recv_readback_outcome(Err(TryRecvError::Empty)), None);
}

#[test]
fn dropped_readback_sender_reads_as_device_lost() {
    // The realistic path: the callback's sender is dropped without ever firing (device lost /
    // buffer destroyed). `try_recv` then reports Disconnected.
    let (tx, rx) = std::sync::mpsc::channel::<Result<(), wgpu::BufferAsyncError>>();
    drop(tx);
    assert_eq!(recv_readback_outcome(rx.try_recv()), Some(ReadbackWait::DeviceLost));
}

#[test]
fn delivered_ok_reads_as_ready() {
    let (tx, rx) = std::sync::mpsc::channel::<Result<(), wgpu::BufferAsyncError>>();
    tx.send(Ok(())).unwrap();
    assert_eq!(recv_readback_outcome(rx.try_recv()), Some(ReadbackWait::Ready));
    // Drained → next poll is Empty (keep polling), never a spurious terminal state.
    assert_eq!(recv_readback_outcome(rx.try_recv()), None);
}

// ---- readback_step: completion / cancel / deadline precedence ----

#[test]
fn completion_is_reported_even_when_cancel_is_set() {
    // ⭐Load-bearing: a finished readback (or a real map error) is returned even if cancel was set
    // in the same breath — a completed tile's result must never be thrown away as "canceled".
    let cancel = AtomicBool::new(true);
    let mut ready = || Some(ReadbackWait::Ready);
    assert_eq!(readback_step(&mut ready, Some(&cancel), None), Some(ReadbackWait::Ready));
    let mut mapfail = || Some(ReadbackWait::MapError("boom".into()));
    assert_eq!(
        readback_step(&mut mapfail, Some(&cancel), None),
        Some(ReadbackWait::MapError("boom".into()))
    );
}

#[test]
fn cancel_interrupts_a_pending_wait() {
    let cancel = AtomicBool::new(true);
    let mut pending = || None::<ReadbackWait>;
    assert_eq!(readback_step(&mut pending, Some(&cancel), None), Some(ReadbackWait::Canceled));
}

#[test]
fn cancel_takes_precedence_over_the_deadline() {
    let cancel = AtomicBool::new(true);
    let past = Instant::now() - Duration::from_secs(1);
    let mut pending = || None::<ReadbackWait>;
    assert_eq!(
        readback_step(&mut pending, Some(&cancel), Some(past)),
        Some(ReadbackWait::Canceled)
    );
}

#[test]
fn deadline_fires_when_pending_and_not_canceled() {
    let past = Instant::now() - Duration::from_secs(1);
    let mut pending = || None::<ReadbackWait>;
    let cancel = AtomicBool::new(false);
    assert_eq!(
        readback_step(&mut pending, Some(&cancel), Some(past)),
        Some(ReadbackWait::Deadline)
    );
    // No cancel flag supplied at all, past deadline → still Deadline.
    assert_eq!(readback_step(&mut pending, None, Some(past)), Some(ReadbackWait::Deadline));
}

#[test]
fn keeps_polling_while_pending_with_no_terminal_signal() {
    let future = Instant::now() + Duration::from_secs(3600);
    let cancel = AtomicBool::new(false);
    let mut pending = || None::<ReadbackWait>;
    assert_eq!(readback_step(&mut pending, Some(&cancel), Some(future)), None);
    // No cancel, no deadline, not ready → keep polling.
    assert_eq!(readback_step(&mut pending, None, None), None);
}
