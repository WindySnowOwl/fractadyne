# Replay fixtures — recorded decisions the controller must keep making

Each `.jsonl` here is a recording of the live frame's lifted decisions (design/live-render-robustness.md
§5.4, W5): a header line, then one line per decision with every input the decision read and what it
decided. `cargo test` replays every file (`render::decide`'s
`every_recorded_session_replays_to_its_decisions`): each decision is recomputed from its recorded
inputs and compared with the recorded one, field by field. It runs on the CPU alone, so CI runs it.

A line carries its complete inputs, so any subset of a recording replays on its own; the fixtures are
trimmed recordings. Anti-vacuity: every file holds at least 200 decisions, some of them of a moving
frame (`REPLAY_MIN_DECISIONS`), and across the set every branch of every lifted decision is replayed
at least once.

Lifted so far (schema 2): `moving` (the moving frame's scale: the work budget's bound, the rate-aware refresh
cap, the AIMD, the visible-frame ladder), `present` (the present gate: the held snapshot, the
settled-cost probe, the colour-pass filter) and `iter` (the adaptive live-iteration climb: the boost's
seed from the reference, the raises, the plateau, the appetite re-pick and the exhausted revert, the
verdict, and the frame's iteration ask).

## When it goes red

A lifted decision now decides differently from the same inputs. If that was not the intent, it is a
regression, and the failure names the frame and the fields that moved. If it was — a deliberate change
to the controller — re-record the fixtures with the new build and say in the commit what changed and
why the new decisions are right.

## Recording

`FRACTADYNE_RECORD_DECIDE=<file>` makes any run record its decisions (off by default):

```sh
FRACTADYNE_CONFIG_DIR=<wiped dir> FRACTADYNE_RECORD_DECIDE=motiontest.jsonl fractadyne --motiontest
fractadyne --replay motiontest.jsonl        # 0 = every decision recomputed exactly
```

`--replay` also reads a recording made anywhere — a field session included — so a field failure can
become a regression test without anyone transcribing numbers.

| file | recorded from |
|------|---------------|
| `motiontest.jsonl` | `--motiontest` (corpus loc 07, 1.3e31, mode 2, a 1M ask): the 100× click, the dive and the Home glide |
| `autodive-2p800.jsonl` | the screen gate's fixture: `--autodive` from `scripts/dive-capture/dive-2p800.kfr` with `session-seed.toml`, 10,000 iterations |
| `uitest.jsonl` | `--uitest`: every UI screen, and the live bands (direct 1e2, df32 1e6 and 1e12, floatexp 1e30 on the seahorse) settled under auto-iterations |
| `livetest-grand-tour.jsonl` | `--livetest tours/grand-tour.toml --size 480x270`: the scripted tour's glides and holds |
| `zoomtest-climb.jsonl` | three `--zoomtest` runs (a header each) under AUTO-iterations, from `locations/`: a 12-octave glide into the 9.3e78 black-minibrot repro and 30 s there (the boost seeded from the reference, raises, a flat run that motion ends); a glide into an all-interior cardioid view (the appetite re-pick, then exhausted); the home view, a third of it interior (the plateau) |

⚠The iteration climb never raises in the other four: three run under an EXPLICIT iteration count (the
motiontest's 1M ask, the autodive's 10,000, the tour's per-keyframe budgets), and the uitest's
auto-iteration bands resolve at the depth cap. No seed, no raise, no plateau. The climb's decisions come
from `zoomtest-climb.jsonl`, and the `.fdn` files in `locations/` are its sources (the minibrot file is
`validation/minibrot-auto-iter-9.3e78.fdn` with `auto_iter=1`).

A decision must read only its inputs: the first `--uitest` recording replayed with one mismatch, because
`decide_moving` read the adapter's calibrated per-pass fixed cost, which a replay process (no adapter)
reads as the default. It is an input now, and `a_lifted_decision_reads_no_global` scans every lifted
decision and every function it reaches for a global read.
