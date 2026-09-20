# dive-capture — score what a live dive put ON SCREEN

The tooling behind the blank-motion-frame fix of 2026-09-20 (`112a088`), kept because the
lesson it taught is easy to forget: **every per-pass metric in the log said the fix was working
while the screen got worse.** Resolution is part of the iteration walk's signature, so a
controller that re-chose it every few frames restarted the walk every few frames — walks
reached further, and 36% of frames were a flat colour. Only capturing the live window and
testing the canvas for uniformity showed it. Measure the screen.

## Reproduction

`dive-2p800.kfr` with the reporting user's settings (`session-seed.toml`: `prefer_detail`,
`min_motion_res` 0.83, zoom rate 4×, `work_budget_scale` 7.5) and a fixed 10,000 iterations
puts an RTX 3080 in the regime: ~1% of motion walks reach an escape, ~76% of passes commit no
pixel. Every shallower replay (2^485–2^633) was far milder — go to the reported depth.

```sh
cp target/release/fractadyne.exe /tmp/fd-test.exe      # a copy; never the running app
scripts/dive-capture/capdive.sh before /tmp/fd-test.exe scripts/dive-capture/dive-2p800.kfr
python scripts/dive-capture/blankscore.py before       # % of captured frames that are blank
```

Single runs vary (the baseline scored 8%, 13% and 14% on three runs): **compare at least three
pairs** before believing a difference.

## Files

| file | what |
|------|------|
| `capdive.sh NAME EXE KFR [SEED] [T] [ITER]` | runs `--autodive` from a `.kfr` in a wiped scratch config, captures the window for 18 s |
| `grab.ps1 -ProcId N -OutDir D -Seconds S -IntervalMs M` | PrintWindow capture of one process's window, DPI-aware, 480 px wide (PS 5.1) |
| `grab_full.ps1` | the same at native resolution — needed to read an 8 px toolbar glyph |
| `blankscore.py RUN…` | per run: frames, `SCREEN BLANK` count (canvas-interior stddev < 1), rung changes, empty passes, and a `#`/`.` timeline |
| `motion.py RUN/stderr.txt…` | camera metrics from the `[fd-autopilot]` evals: focus-of-expansion off-centre and aim movement per look |
| `life.py RUN/stderr.txt` | octaves of runway at the goal and the aim: true distance estimate at 260 digits (mpmath), slow. Assumes a 1467×1102 panel — edit `W,H` |
| `mkpath.py RUN STEP` | every STEP-th eval as `path.tsv` (centre, l2, aim) for offline `--render` frames along the dive |
| `dive-2p800.kfr`, `dive-2p633.kfr` | the two reported locations (the 2^633 one does NOT reproduce the blank frames on this box) |
| `session-seed.toml` | the reporting user's settings, personal paths removed |

Log fields worth knowing: `norm reading EMPTY` = a pass committed no escaped pixel;
`norm range: frame [lo,hi]` = the view's escape range (hi at the iteration cap means the count,
not the fractal, ran out); `visible-res f=… → …` = the motion-resolution rung changing;
`chunk f=… cur=… step=…` = how far a walk got (`cur + step` against `lo`).
