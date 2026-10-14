# Replay fixtures — recorded decisions the controller must keep making

Each `.jsonl` here is a recording of the live frame's lifted decisions (design/live-render-robustness.md
§5.4, W5): a header line, then one line per decision with every input the decision read and what it
decided. `cargo test` replays every file (`render::decide`'s
`every_recorded_session_replays_to_its_decisions`): each decision is recomputed from its recorded
inputs and compared with the recorded one, field by field. It runs on the CPU alone, so CI runs it.

A line carries its complete inputs, so any subset of a recording replays on its own; the fixtures are
trimmed recordings. Anti-vacuity: every file holds at least 200 decisions, some of them of a moving
frame (`REPLAY_MIN_DECISIONS`).

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
