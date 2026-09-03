# SA coefficient walk through the bignum backend — design (2026-09-02)

> ✅**SHIPPED 0.2.41-beta.12 (2026-09-03), as designed** — with one simplification the survey
> missed: at d=2 every `mul_u32_bf` factor is 1 or 2, i.e. EXACT (identity / doubling), so no
> shift-and-add mirror was needed at all. Measured: accelerated e4000 SA 47.5 → 13.6 s, wall
> 82.2 → **47.5 s vs F3's 60.3 s** — the projection ("~45-50 s") held. All proof obligations
> met (1,152-case identity matrix; default build byte-inert; corpus cross-backend maxD 0).

The measured next lever from the beta.11 F3-gap close (TODO.md "ROUTE THE SERIES-APPROXIMATION
COEFFICIENT WALK THROUGH THE BIGNUM BACKEND"): `series_skip` is astro-hardcoded on both builds
(e4000: sa_ms 47,386 default vs 47,511 accelerated — 58% of the accelerated 82.2 s wall), and the
orbit's own conversion precedent gained 4.4x. Projected accelerated e4000 wall ~46 s, ahead of
F3's 60.3 s at that location. Design surveyed 2026-09-02 while the corpus row-39 renders ran;
implementation NOT started.

## What the survey found (all load-bearing)

- **`step_bf` is already backend-generic**: `step_gen::<B: RefBackend>` routes through
  `crate::fractal::trait_step`, documented as reproducing the hand-written arms bit-for-bit
  (guarded by the exact SA cross-check tests + goldens). The SA walk's orbit-advance line
  needs no new twin — call `step_gen::<Float>` with the rug ctx.
- **The rounding discipline is already established** (backend_rug.rs): every rounded op
  truncates toward zero (`RZ = Round::Zero`, matching astro `RoundingMode::None`); precision
  is word-rounded ONCE by `RefBackend::ctx_for` (nothing may re-derive it from `p`);
  `from_carrier`/`to_carrier` round-trip exactly. The named tests
  (`round_to_nearest_would_break_bit_identity`,
  `an_operand_wider_than_the_working_precision_is_not_truncated_first`) document why.
- **`cmul_bf` sequence to mirror literally** (reference.rs): rx = ax*bx sub ay*by,
  ry = ax*by add ay*bx — each of the 4 muls and 2 add/subs rounds at p with RM. Use
  `Field::fmul`/`fadd`/`fsub` in the same operand order.
- **`mul_u32_bf` is shift-and-add** (bignum.rs:651): acc += x*2^k for each set bit of n, each
  ADD rounding at p; base doubles per bit. ⚠The rug mirror must reproduce the SAME
  decomposition — `fdouble` (exact <<1) + `fadd` (RZ) per set bit — NOT a single MPFR mul_ui,
  whose one-rounding result can differ from the chained truncations.
- **`cpow_bf`** = repeated cmul (e<=3 here); mirror trivially. `log2_cmag` reads component
  exponents (max; None,None -> -inf) — the rug mirror must reproduce astro's exponent
  semantics INCLUDING zero (astro `exponent()` is `Some(0)` for zero — the [[topic-logspace-solve]]
  trap); `to_floatexp`'s existing convention mapping shows how the exponents align.
- **Escape check** uses `to_f64` (truncating) — mirror with `to_f64_trunc`.
- **The final coefficients** can round-trip `to_carrier` back to astro and reuse
  `coeff_to_fe` verbatim (conversion exact) — no second floatexp conversion path to prove.
- **Gates stay where they are**: `do_sa || short_escaper` (render.rs build_params reference
  stage) decides WHETHER SA runs; the twin sits underneath. Explains the e18003 reading
  (sa_ms=0 there: BLA eligibility cleared do_sa; e4000 runs both — 47 s).

## Shape of the change

1. `try_series_skip_inplace(cx, cy, log2_max_dc, max_iter, orbit_len, formula, p)
   -> Option<SeriesSkip>` in backend_rug.rs — the beta.2 `try_orbit_length_inplace` pattern:
   scope to `formula::MANDELBROT` first (d=2: 5 cmuls + step per iteration), `None` for the
   multibrots (astro fallback; extend later if their depth traffic ever matters). In-place
   rug scratch, allocated once.
2. Dispatch inside `series_skip` (or at its caller) exactly the way the pick walks dispatch:
   selected backend = rug AND formula supported -> twin; anything else -> the astro loop
   UNTOUCHED. The astro path must not change by a single op — blessed corpus output pins it.
3. `sa_step_budget(p)` / EPS_LOG2 / MIN_SKIP / the once-invalid-stays-invalid break: same
   constants, same control flow, mirrored literally.

## Proof obligations (the beta.2 bar)

- `the_sa_walk_is_backend_identical`: case matrix (shallow/deep log2_max_dc x precisions
  incl. multi-word x escaping/surviving/budget-bound centres) pinning `skip` equal AND all
  six coefficient outputs (`a/b/c` f64s via to_bits, `*_exp` i32s) equal astro vs twin.
- `sa_budget_clears_every_blessed_fixture` untouched and green.
- Corpus 38/38 byte-identical on the DEFAULT build (astro path untouched by construction);
  crosscheck_backends.py astro-vs-MPFR on a deep SA-active row (09/10/16-19 are e250-e1105
  with SA on) after the twin lands.
- Timing: re-run the e4000 command (validation/extreme-depth.md) on the accelerated build;
  expect SA ~47.5 -> ~9-12 s, wall ~82 -> ~45-50 s. Record per-config only (the 4K-table
  scope discipline; no "owns extreme" claims).

## Traps already paid for elsewhere

- msys bash `-lc` starts in HOME (cd inside the command); gnu-target exes need
  C:/msys64/mingw64/bin on PATH. Build: `cargo +stable-x86_64-pc-windows-gnu test
  -p fractadyne-core --features rug --target x86_64-pc-windows-gnu` (-j1).
- A backend compiled in but never selected never runs — the identity test must SELECT the
  twin explicitly, not assume it engaged (assert-the-variable-under-test).
- `-j1` builds; never rebuild target/release (msvc) while a corpus check is mid-run — the
  check re-spawns that exe per row.
