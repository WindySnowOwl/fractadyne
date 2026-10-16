//! `--oomtest`: force a real allocation failure, to prove the out-of-memory path writes a crash
//! report (an out-of-memory abort skips the panic hook). Dispatched from `main()`.

pub(crate) fn run() -> ! {
    crate::diag::breadcrumb("oomtest: requesting an impossible allocation".into());
    let n = usize::MAX / 2;
    // Deliberately unsatisfiable: the allocator returns null, `ReportingAlloc` reports, and
    // the runtime aborts on it exactly as a real exhaustion would.
    let v: Vec<u8> = Vec::with_capacity(n);
    std::hint::black_box(&v);
    crate::exit(0);
}
