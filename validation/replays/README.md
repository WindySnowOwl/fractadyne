# Replay fixtures — recorded decisions the controller must keep making

Each `.jsonl` here is a recording of the live frame's lifted decisions (design/live-render-robustness.md
§5.4, W5): a header line, then one line per decision with every input the decision read and what it
decided, and beside them each frame's §6.7 regime facts (`"kind":"regime"`: the frame-record fields the
soak's escaped-reference predicate reads). `cargo test` replays every file (`render::decide`'s
`every_recorded_session_replays_to_its_decisions`): each decision is recomputed from its recorded
inputs and compared with the recorded one, field by field. It runs on the CPU alone, so CI runs it.

A line carries its complete inputs, so any subset of a recording replays on its own; the fixtures are
trimmed recordings. Anti-vacuity: every file holds at least 200 decisions, some of them of a moving
frame (`REPLAY_MIN_DECISIONS`); across the set every branch of every lifted decision is replayed at
least once (`decide::fixture_coverage`, 52 terms); and at least one recording ENTERED the §6.7
escaped-reference regime, judged by the soak's own predicate over its regime lines; and the set's
recordings come from more than one card (their headers name the adapter), so a hidden input that
differs between cards turns a replay red on the CPU. The same verdict
runs in the app as `--selftest-filter replay` (a group of the full self-test too).

Lifted so far (schema 2): `moving` (the moving frame's scale: the work budget's bound, the rate-aware refresh
cap, the AIMD, the visible-frame ladder), `present` (the present gate: the held snapshot, the
settled-cost probe, the colour-pass filter) and `iter` (the adaptive live-iteration climb: the boost's
seed from the reference, the raises, the plateau, the appetite re-pick and the exhausted revert, the
verdict, and the frame's iteration ask) and `probe` (the budget-climb probe: whether a settled view whose
work budget has not converged, with nothing else dispatching, forces a re-measure) and `walk-price` (the
chunk walk's pricing and sizing: the wall-aware floor, the in-flight pass's price — the lethal-band shed,
the drain, the band ledger — the pass's step budget and the growth-limited iteration step) and `walk-id`
(the walk's identity: the signature composed from everything it folds, the restart and whether it clears
the band ledger, the counted lost walk, the SA-skip floor and a partial reference's capped end) and
`walk-dispatch` (the walk's dispatch: the serialized hold, the pin gate and its wall-clock escape, the
regional licence, the running-pixel charge, the window, the cursor, the pin pass's bookkeeping, the
finished walk's tail and its read retry). With the last three, `bp_chunk_tiling`'s decisions are all
lifted; what stays imperative is its state resets, the frame's cost trace and the clock reads (§5.4).

§7.5's companion red-check: set `PRICE_REPRESENTATIVE_FRAC` to 0.5 in a scratch build (never blessed,
thrown away) and the gate goes red on 35 probe decisions of `uitest.jsonl`, at the default home view.

## When it goes red

A lifted decision now decides differently from the same inputs. If that was not the intent, it is a
regression, and the failure names the frame and the fields that moved. If it was — a deliberate change
to the controller — re-record the fixtures with the new build and say in the commit what changed and
why the new decisions are right.

## Recording

On a field machine (agent v24+): `scripts/field-request.ps1 -Action harness -Run "<harness flags>" -Build <a
published build with the recorder> -RecordDecide -Wait` — the run's `decide.jsonl` comes back with its results.

`FRACTADYNE_RECORD_DECIDE=<file>` makes any run record its decisions (off by default):

```sh
FRACTADYNE_CONFIG_DIR=<wiped dir> FRACTADYNE_RECORD_DECIDE=motiontest.jsonl fractadyne --motiontest
fractadyne --replay motiontest.jsonl        # 0 = every decision recomputed exactly
```

`--replay` also reads a recording made anywhere — a field session included — so a field failure can
become a regression test without anyone transcribing numbers.

| file | recorded from |
|------|---------------|
| `motiontest.jsonl` | `--motiontest` (corpus loc 07, 1.3e31, mode 2, a 1M ask): the 100× click, the dive and the Home glide; recorded on the RTX 3080 through the field agent (dev.8). It also ENTERS the §6.7 regime, uninstrumented: a 317-sample escaped reference against the 1M ask during motion |
| `field-rx6800xt.jsonl` | a field capture on the AMD Radeon RX 6800 XT (PLUTO) through the field agent (v24, `field-request.ps1 -RecordDecide`; re-recorded with W6's dev.9 field build): `--motiontest`, and the auto-iteration climb at the 9.3e78 minibrot (`--zoomtest 12 --zoomtest-location session --zoomtest-start-log2 250 --zoomtest-hold 30 -ViewFdn locations/minibrot-9.3e78-auto.fdn`). Recorded on one card, replayed exactly on another |
| `autodive-2p800.jsonl` | the screen gate's fixture: `--autodive` from `scripts/dive-capture/dive-2p800.kfr` with `session-seed.toml`, 10,000 iterations |
| `uitest.jsonl` | `--uitest`: every UI screen, and the live bands (direct 1e2, df32 1e6 and 1e12, floatexp 1e30 on the seahorse) settled under auto-iterations |
| `livetest-grand-tour.jsonl` | `--livetest tours/grand-tour.toml --size 480x270`: the scripted tour's glides and holds |
| `escaped-ref.jsonl` | W9's motion rung on the 2026-09-21 RX 6800 XT crash view (`locations/crash-4627.fdn`, 1.76e6×, an explicit 4,627), safe variant (no seeded budget), under `FRACTADYNE_REF_ESCAPE_AT=655`: `--zoomtest 7.45 --zoomtest-location locations/crash-4627.fdn --zoomtest-start-log2 13.3 --zoomtest-taps 40,0.08,1.0 --zoomtest-hold 15 --zoomtest-rate 4.0 --window 1676x1360`. ENTERED on the RTX 3080: 2,133 frames on the 655-sample escaped reference, 319 storm readings (up to 11.5M rebases), 0 of 327 dispatches un-chunked, no slow frame. ⚠An instrument run (its header says so): never for a golden or a baseline |
| `zoomtest.jsonl` | five `--zoomtest` runs (a header each) from `locations/`. Three under AUTO-iterations: a 12-octave glide into the 9.3e78 black-minibrot repro and 30 s there (the boost seeded from the reference, raises, a flat run that motion ends); a glide into an all-interior cardioid view (the appetite re-pick, then exhausted); the home view, a third of it interior (the plateau). The minibrot under `--set TDR_LETHAL_MS=30` (W8's recipe), for the lethal-band shed no stock run on this card reaches. And the minibrot with the settle tail (`FRACTADYNE_ZOOMTEST_SETTLE`), where on-settle supersampling runs to convergence: each sample's jittered restart, which keeps the band ledger (a `--zoomtest-hold` never supersamples) |

⚠The iteration climb never raises in the other four: three run under an EXPLICIT iteration count (the
motiontest's 1M ask, the autodive's 10,000, the tour's per-keyframe budgets), and the uitest's
auto-iteration bands resolve at the depth cap. No seed, no raise, no plateau. The climb's decisions come
from `zoomtest.jsonl`, and the `.fdn` files in `locations/` are its sources (the minibrot file is
`validation/minibrot-auto-iter-9.3e78.fdn` with `auto_iter=1`).

A decision must read only its inputs: the first `--uitest` recording replayed with one mismatch, because
`decide_moving` read the adapter's calibrated per-pass fixed cost, which a replay process (no adapter)
reads as the default. It is an input now, and `a_lifted_decision_reads_no_global` scans every lifted
decision and every function it reaches for a global read.
