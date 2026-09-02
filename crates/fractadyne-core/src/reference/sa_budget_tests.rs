use super::sa_step_budget;

/// ⭐Every blessed fixture must clear the budget with margin, or a corpus/golden re-bless is
/// being smuggled in as a "tuning" change. The rows are the corpus's deep half — (working
/// precision, iteration count), iteration count being a HARD upper bound on the SA walk — so
/// this fails before `--check` would, and names the row.
#[test]
fn sa_budget_clears_every_blessed_fixture() {
    // (slug, prec bits, iterations) — from validation/corpus/locations.toml; prec is the
    // octaves+64 the SA pass receives, and the assertion adds the 128-bit orbit headroom on
    // top so the bound holds under either precision reading. Measured actual walks are far
    // smaller (row 20: 78,231 steps where this ceiling says 600,008) — the ceiling is the
    // contract precisely so the test never depends on how early an orbit happens to escape.
    let rows: [(&str, usize, u32); 8] = [
        ("15-deep-3.7e163", 607, 1_600_000),
        ("16-deep-2.1e250", 895, 600_008),
        ("17-deep-4.2e275", 979, 600_008),
        ("09-deep-6.1e500", 1_727, 150_000),
        ("18-deep-4.1e508", 1_753, 600_008),
        ("19-deep-1.3e726", 2_476, 600_008),
        ("20-deep-1.2e1008", 3_413, 600_008),
        ("10-deep-4.6e1105", 3_737, 250_000),
    ];
    for (slug, prec, iters) in rows {
        let budget = sa_step_budget(prec + 128);
        assert!(
            budget >= iters,
            "{slug}: SA budget {budget} steps at {prec} bits is below its {iters}-iteration \
             ceiling — the budget would change a BLESSED render; that needs a deliberate \
             re-bless, not a constant edit"
        );
    }
}

/// ...and it actually bites where it exists to bite: at the measured 2.37e4000× build the
/// natural walk was 439,915 steps at 13,353 bits (= 258 s); the budget must land well under
/// that, or it is a no-op wearing a comment.
#[test]
fn sa_budget_bites_at_extreme_depth() {
    let b = sa_step_budget(13_353);
    assert!(
        b < 439_915 / 2,
        "budget at 13,353 bits is {b} steps — not meaningfully below the 439,915-step walk \
         that cost 258 s"
    );
    assert!(b >= 8, "the budget must never sit below MIN_SKIP, or SA silently dies entirely");
}
