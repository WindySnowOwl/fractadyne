//! Global allocator wrapper that makes an OUT-OF-MEMORY death visible.
//!
//! Rust reacts to a failed allocation with [`std::alloc::handle_alloc_error`], which calls
//! `abort()`. On Windows `abort()` raises `__fastfail`, which the OS records as exception
//! `0xc0000409` — and which does NOT run the panic hook. So the app's whole crash-reporting
//! path (`diag::write_crash_report`) is skipped and the process vanishes leaving nothing but a
//! truncated log. Measured on this project: three OS-level crashes in three days, two of them
//! `0xc0000409`, and not one produced a crash report.
//!
//! `set_alloc_error_hook` would be the direct fix but is nightly-only, so this wraps the system
//! allocator instead and reports on the null return, just before the runtime aborts on it.
//!
//! **No per-allocation counters.** The obvious extra — track live bytes here — would put an
//! atomic RMW on every allocation in a program whose deep path is bignum-heavy and rayon-parallel
//! across every core; that is a contended cacheline on the hottest path in the app. Process
//! memory comes from `sysinfo::process_memory()` at report time instead, which costs nothing
//! until something actually fails.

use std::alloc::{GlobalAlloc, Layout, System};

/// Wrapper that forwards to the system allocator and reports a null return.
pub(crate) struct ReportingAlloc;

// SAFETY: every method forwards to `System` with the caller's own layout and pointer, adding
// only a null check and a (report-once, non-allocating-on-the-failure-path) side effect. The
// allocator contract is therefore exactly `System`'s.
unsafe impl GlobalAlloc for ReportingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = System.alloc(layout);
        if p.is_null() {
            crate::diag::on_alloc_fail(layout.size());
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = System.alloc_zeroed(layout);
        if p.is_null() {
            crate::diag::on_alloc_fail(layout.size());
        }
        p
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let p = System.realloc(ptr, layout, new_size);
        if p.is_null() {
            crate::diag::on_alloc_fail(new_size);
        }
        p
    }
}
