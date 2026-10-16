use super::*;

#[test]
fn word_width_matches_astro_floats_own_allocation() {
    // The rule a second backend has to match. Verified against what astro-float actually
    // allocates rather than against the formula restated.
    for p in [1usize, 63, 64, 65, 127, 128, 200, 576, 1088, 3776] {
        let v = BigFloat::from_f64(0.7071067811865476, p);
        let got = v.mantissa_digits().map(|d| d.len()).unwrap_or(0);
        assert_eq!(got, p.div_ceil(64), "astro-float allocated {got} words for p={p}");
    }
}

#[test]
fn the_name_registry_agrees_with_the_trait() {
    // A backend whose BIT does not index its own name would mis-stamp every gate.
    assert_eq!(
        BACKEND_NAMES.get(<BigFloat as RefBackend>::BIT as usize),
        Some(&"astro-float")
    );
    assert!(BACKEND_NAMES.len() <= 32, "the observation mask is a u32");
}

#[test]
fn the_carrier_backend_round_trips_identically() {
    let p = 576;
    let ctx = <BigFloat as RefBackend>::ctx_for(p);
    let v = crate::parse_bf_prec("-0.743643887037158704752191506114774", p).unwrap();
    let back = <BigFloat as RefBackend>::from_carrier(&v, ctx).to_carrier(ctx);
    assert_eq!(crate::to_decimal_string(&v), crate::to_decimal_string(&back));
    assert_eq!(v.to_f64_trunc(), crate::to_f64(&v));
}

/// Startup resolution never returns a backend this build cannot run, and only warns when it had
/// to downgrade. On a build without `rug` the default is astro and there is nothing to probe, so
/// there is never a warning. (The rug fallback path is compiled only into the accelerated build;
/// it is exercised end-to-end by that package's clean-room verify, which runs the binary with
/// its MPFR DLLs removed and asserts it falls back.)
#[test]
fn resolve_startup_backend_stays_within_this_build_and_warns_only_on_downgrade() {
    // The probe is trivially true when MPFR is statically present or not compiled in.
    assert!(mpfr_runtime_available() || cfg!(all(feature = "rug", target_os = "windows")));

    let (choice, fell_back) = resolve_startup_backend(None);
    assert!(available_backends().contains(&choice), "default is a backend this build lacks");

    let (c2, fb2) = resolve_startup_backend(Some(BackendChoice::Astro));
    assert_eq!(c2, BackendChoice::Astro);
    assert!(!fb2, "asking for astro never falls back");

    #[cfg(not(all(feature = "rug", target_os = "windows")))]
    {
        // No delay-load probe here, so resolution is a pure pass-through: never a fallback.
        assert!(!fell_back);
        assert_eq!(choice, default_choice());
    }
    let _ = fell_back;

    // The fallback message names the fix and the links, and the running directory when known.
    let m = mpfr_missing_message(Some(std::path::Path::new("C:\\Fractadyne\\accel")));
    assert!(m.contains("astro-float") && m.contains("releases") && m.contains(".dll"));
    assert!(m.contains("C:\\Fractadyne\\accel"), "the running directory is named");
    assert!(m.contains("https://gmplib.org/") && m.contains("https://www.mpfr.org/"));
    // Each URL on its own line (for clean copy-paste): no two https on the same line.
    assert!(m.lines().all(|l| l.matches("https://").count() <= 1), "two URLs share a line");
    // The no-dir form still names the executable.
    assert!(mpfr_missing_message(None).contains("fractadyne.exe"));
    // The recovery message says the next launch, not this session.
    assert!(mpfr_found_message().contains("next time you start"));
    // The restored-at-startup message says it is in use now.
    assert!(mpfr_restored_message().contains("present again") && mpfr_restored_message().contains("now using"));
}
