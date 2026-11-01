use super::*;

#[test]
fn a_price_over_the_limit_refuses_only_when_enforced() {
    assert!(admits(499.0, 500.0, true));
    assert!(admits(500.0, 500.0, true));
    assert!(!admits(500.1, 500.0, true));
    assert!(admits(1.0e9, 500.0, false));
    // A pass the guard cannot price has not been shown safe.
    assert!(!admits(f64::NAN, 500.0, true));
}

/// The one test that installs the guard: it is process-wide, and tests run on parallel threads.
#[test]
fn an_installed_guard_prices_refuses_and_counts() {
    // Not installed: everything goes, nothing is priced.
    remove();
    let _ = take_peak_ms();
    assert!(admit(2, 0, u64::MAX).is_ok());
    assert_eq!(take_peak_ms(), 0.0);

    let before = refusals();
    install(Guard {
        // 1e-6 ms a step, ten times that in mode 2.
        cost_ms: Box::new(|mode, _| if mode == 2 { 1.0e-5 } else { 1.0e-6 }),
        limit_ms: 500.0,
        enforce: true,
    });
    assert!(admit(1, 0, 400_000_000).is_ok()); // 400 ms
    match admit(2, 0, 400_000_000) {
        Err(GpuError::TooSlowForThisGpu { est_ms, limit_ms }) => {
            assert!((est_ms - 4000.0).abs() < 1e-6, "{est_ms}");
            assert_eq!(limit_ms, 500.0);
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
    assert_eq!(refusals(), before + 1);
    let r = last_refusal().expect("the refusal is recorded");
    assert_eq!((r.steps, r.mode), (400_000_000, 2));
    // The peak is the dearest pass priced, refused or not, and reading it starts over.
    assert!((take_peak_ms() - 4000.0).abs() < 1e-6);
    assert_eq!(take_peak_ms(), 0.0);

    // Pricing only: the same pass goes ahead and is still priced.
    install(Guard { cost_ms: Box::new(|_, _| 1.0e-5), limit_ms: 500.0, enforce: false });
    assert!(admit(2, 0, 400_000_000).is_ok());
    assert_eq!(refusals(), before + 1);
    assert!((take_peak_ms() - 4000.0).abs() < 1e-6);

    remove();
    assert!(admit(2, 0, u64::MAX).is_ok());
}
