//! Verify the deep-dive reference-reuse plumbing end-to-end at the orbit level: `recompute_worker`
//! extends a cached orbit instead of rebuilding, and `try_reuse_reference`'s gates reject any
//! reference that would be invalid to reuse. (The extended orbit's byte-identity to a fresh build,
//! and render invariance to the chosen valid reference, are proven separately in core + selftest.)
use super::*;
use fractadyne_core::{parse_bf, Viewport};

// A Mandelbrot recompute for a view at (cx, cy, log2mag) — no BLA/SA, so the orbit is isolated.
fn inputs_for(cx: &str, cy: &str, log2mag: f64, gpu_iter: u32, reuse: Option<ReuseRef>) -> RecomputeInputs {
    let mut vp = Viewport::new(256.0, 256.0);
    vp.set_center_log2mag(parse_bf(cx).unwrap(), parse_bf(cy).unwrap(), log2mag);
    let scale = vp.gpu_scale();
    RecomputeInputs {
        origin: "test",
        center_bf: [vp.center_x.clone(), vp.center_y.clone()],
        span: vp.complex_span_fe(),
        span_mantissa: scale.span_mantissa,
        delta_exp: scale.delta_exp,
        gpu_iter,
        orbit_len_cap: u32::MAX,
        precision: vp.precision,
        julia: false,
        formula: 0,
        julia_c: (0.0, 0.0),
        do_sa: false,
        bla_dc_max: None,
        stripe_freq: 1.0,
        trap_type: 0,
        reuse,
        spawn_orbit_id: 0,
    }
}

// Seahorse boundary point: survives thousands of iters, so a short build is truncated/extendable.
const SX: &str = "-0.7436438870371587047521915061147707";
const SY: &str = "0.131825904205311970493132056385139";
const L2: f64 = 26.6; // ~1e8× (mode-0 df32 perturbation)

/// ⭐The guard the original test never had. `precomputed_matches` compares a
/// `RecomputeResult` against the inputs it was built from, so a fresh build MUST satisfy it —
/// otherwise reference pipelining is silently dead, which is exactly what happened when the
/// predicate compared the derived `prec` (request + 128 headroom) against the request itself.
/// Asserts the real production predicate, not a restatement of it, so the two cannot drift.
#[test]
fn precomputed_matches_a_fresh_build() {
    let inp = inputs_for(SX, SY, L2, 3000, None);
    let (want_iter, want_prec) = (inp.gpu_iter, inp.precision);
    let r = recompute_worker(inp);
    assert!(
        precomputed_matches(&r, want_iter, want_prec),
        "a reference built for (iter={want_iter}, prec={want_prec}) must be accepted for that \
         same frame — got iter={} req_prec={} (built at prec={})",
        r.iter, r.req_prec, r.prec,
    );
    // And it must still REJECT a frame it was not built for, or the gate is a rubber stamp.
    assert!(!precomputed_matches(&r, want_iter + 1, want_prec), "a different iter must reject");
    assert!(!precomputed_matches(&r, want_iter, want_prec + 1), "a different prec must reject");
    // The headroom is what made the old test unsatisfiable; pin the relationship it broke on.
    assert_eq!(r.prec, r.req_prec + REF_PREC_HEADROOM, "fresh build carries the headroom");
}

#[test]
fn reuse_extends_cached_orbit_in_place() {
    let a = recompute_worker(inputs_for(SX, SY, L2, 3000, None));
    assert!(a.partial, "short seahorse orbit should be truncated (extendable)");
    let tail = a.orbit_tail.clone().expect("a truncated orbit carries a tail");
    let reuse = ReuseRef { point: a.rp.clone(), prefix: a.orbit.clone(), tail, prec: a.prec };
    // A deeper-iter rebuild at the same view must EXTEND a's orbit, not rebuild it.
    let b = recompute_worker(inputs_for(SX, SY, L2, 6000, Some(reuse)));
    assert!(b.orbit_len > a.orbit_len, "reuse should have extended the orbit");
    assert_eq!(b.rp, a.rp, "reuse must keep the cached reference point");
    assert_eq!(b.prec, a.prec, "extend must stay at the cached (headroom) precision");
    // Byte-identical prefix ⇒ it truly extended (a fresh build at this depth would differ).
    assert_eq!(&b.orbit[..a.orbit.len()], &a.orbit[..], "extended orbit must preserve the prefix");
}

#[test]
fn reuse_gates_reject_invalid_references() {
    let a = recompute_worker(inputs_for(SX, SY, L2, 3000, None));
    let tail = a.orbit_tail.clone().expect("tail");
    let mk = |r: ReuseRef| inputs_for(SX, SY, L2, 6000, Some(r));

    // Valid reference → reuse fires.
    let ok = ReuseRef { point: a.rp.clone(), prefix: a.orbit.clone(), tail: tail.clone(), prec: a.prec };
    assert!(try_reuse_reference(&mk(ok)).is_some(), "a valid in-view reference must reuse");

    // Escaped (complete) orbit → reused AS-IS, not extended: there's nothing past the escape,
    // but keeping the same reference avoids a re-pick "jump" on rebuild (deep-dive reuse policy
    // since v0.1.64/65). So reuse fires (Some) and the orbit length is unchanged.
    let mut esc_tail = tail.clone();
    esc_tail.escaped = true;
    let esc = ReuseRef { point: a.rp.clone(), prefix: a.orbit.clone(), tail: esc_tail, prec: a.prec };
    let re = try_reuse_reference(&mk(esc)).expect("an escaped orbit is reused as-is");
    assert_eq!(re.orbit_len, a.orbit_len, "an escaped orbit is reused unchanged, not extended");

    // Cached precision below this depth's need → headroom exhausted.
    let lowp = ReuseRef { point: a.rp.clone(), prefix: a.orbit.clone(), tail: tail.clone(), prec: 8 };
    assert!(try_reuse_reference(&mk(lowp)).is_none(), "insufficient precision must not reuse");

    // Point far off-centre (origin is ~0.75 away, ≫ a 1e8× span) → drifted out of validity.
    let far = [parse_bf("0.0").unwrap(), parse_bf("0.0").unwrap()];
    let drift = ReuseRef { point: far, prefix: a.orbit.clone(), tail, prec: a.prec };
    assert!(try_reuse_reference(&mk(drift)).is_none(), "a drifted point must not reuse");
}

// The orbit-length cap must fit the orbit + BLA (~9× the orbit at 16 B/sample) inside the GPU
// storage-binding limit, yet stay ABOVE every escaping corpus reference so those views build
// unchanged. Loc 15's 918 516-sample reference is the deepest such orbit (right at the 128 MB
// edge) and is the binding invariant: the cap must clear it, or the deep-dendrite corpus render
// regresses. (The cap only ever truncates a NON-escaping deep-interior reference.)
#[test]
fn orbit_len_cap_fits_binding_and_clears_corpus() {
    const LIMIT_128MB: u32 = 134_217_728; // wgpu default max_storage_buffer_binding_size
    const LOC15_ORBIT: u64 = 918_516; // deepest escaping corpus reference (v0.2.18 dendrites)
    init_orbit_len_cap(LIMIT_128MB);
    let cap = orbit_len_cap() as u64;
    // Orbit + BLA (~9×) at 16 B/sample must fit the binding.
    assert!(cap * 9 * 16 <= LIMIT_128MB as u64, "cap {cap} + BLA overruns the 128 MB binding");
    // …and clear the deepest escaping corpus reference so it is never truncated.
    assert!(cap > LOC15_ORBIT, "cap {cap} must exceed loc 15's {LOC15_ORBIT}-sample orbit");
    // At the 1 GiB binding the app now REQUESTS from capable adapters (the e82 spar's
    // reference does not escape within the 128 MB default's ~928k samples), the same formula
    // must yield multi-million-sample room while still fitting orbit + BLA in the binding.
    const LIMIT_1GIB: u32 = 1 << 30;
    let cap_1g = ((LIMIT_1GIB as u64) / 16 / 9).saturating_sub(4096);
    assert!(cap_1g > 7_000_000, "1 GiB binding should afford >7M samples, got {cap_1g}");
    assert!(cap_1g * 9 * 16 <= LIMIT_1GIB as u64, "1 GiB cap math overruns the binding");
}

/// The 6.3e63× spar "blobs" contract: a SETTLED live build's orbit cap must follow the
/// (boosted) iteration budget, or a non-escaping reference comes back `partial` below the
/// budget and pixels clamp to the short orbit. Motion keeps the cheap `LIVE_REF_CAP`.
#[test]
fn live_orbit_cap_follows_settled_budget() {
    // The reported failure: settled budget 351,606 + headroom > 256k must lift the cap.
    let boosted = 351_606u32 + 32 * 256;
    assert_eq!(live_orbit_cap(false, boosted, 0), boosted);
    // Below the cap, the cap still floors (escaped-short truncation guard, v0.2.26).
    assert_eq!(live_orbit_cap(false, 100_000, 0), crate::LIVE_REF_CAP);
    // Interacting with NOTHING installed: the short cap, so dive builds stay cheap.
    assert_eq!(live_orbit_cap(true, boosted, 0), crate::LIVE_REF_CAP);
    assert_eq!(live_orbit_cap(true, 100_000, 0), crate::LIVE_REF_CAP);
}

/// The 2e82× truncation (measured 2026-08-14, `FRACTADYNE_NO_PREFETCH=1`): a motion-time
/// rebuild whose reuse was refused built a FRESH 256,001-sample orbit and installed it over the
/// live 1,208,193 one, and the hold that followed rendered 100% black. A motion build may
/// decline to GROW the reference; it may never come back SHORTER than the one on screen.
#[test]
fn live_orbit_cap_never_truncates_the_installed_orbit() {
    let installed = 1_208_193u32;
    let ask = 2_008_192u32;
    // Motion at a deep view: the cap is the installed length, not `LIVE_REF_CAP`.
    assert_eq!(live_orbit_cap(true, ask, installed), installed);
    // …and it still does not let motion GROW it: `needs_quality` mins the ask with this cap.
    assert_eq!(ask.min(live_orbit_cap(true, ask, installed)), installed);
    // Settled is unchanged — growth is the settled path's job.
    assert_eq!(live_orbit_cap(false, ask, installed), ask);
    // A short installed orbit (the e21000 tip's refused extension) keeps the plain cap.
    assert_eq!(live_orbit_cap(true, ask, 100_000), crate::LIVE_REF_CAP);
    // Zooming OUT: the ask drops below what is installed, and the floor drops with it — a
    // re-anchor on the way out must not rebuild the deepest orbit the session ever held.
    assert_eq!(live_orbit_cap(true, 400_000, 4_000_000), 400_000);
}
