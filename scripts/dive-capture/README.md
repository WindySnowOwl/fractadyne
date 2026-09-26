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

## The screen gate

`screengate.py` turns three such runs into a verdict (design/live-render-robustness.md §7.6):

```sh
powershell -File scripts/dive-capture/capdive.ps1 -Out run1 -Exe C:\path\fd-test.exe   # x3, or:
scripts/gpu-validate.ps1 -Label ...        # its step 08 captures screen\run1-3 into the bundle
python scripts/dive-capture/screengate.py run1 run2 run3      # 0 PASS, 1 RED, 2 VACUOUS
python scripts/dive-capture/screengate.py --selftest          # every criterion seen to fire
```

It scores **blank** (canvas stddev < 1), **flat** (>97% within 8 of the median colour), **flash**
EPISODES (the canvas switching into or out of a flat capture) and **stale** (a streak of identical
captures while the view moved; a whole run whose screen never changed while the autopilot's log
shows the view travelling is FROZEN, and RED), takes the median of three, and holds it to 7% / 7% /
3 / 2 with a blank spread of at most 10 points. ⚠`flashscore.py`'s FLASH_DIFF of 60 is kept for
recordings but is NOT the gate's flash: at 2^800 an ordinary zoom step between captures differs by
35–75 units, so it counted 26–43% of healthy transitions. VACUOUS (never a pass): fewer than three
runs, a run under 60 captures, a view that did not move, or a log naming another bignum backend
than the fixture declares (`--bignum`, default astro-float).

`capdive.ps1` is `capdive.sh` for a machine with no sh — the Radeon box and the battery. Both
capture at ~170 ms per frame (PrintWindow at 480 px). Captures before the dive's first picture are
its start, counted apart. ⚠On the RTX 3080 on 2026-09-26 the gate did NOT go red on the build before
the blank-frame fix (median 2.9% blank against the fixed build's 0%): it catches a severe
regression. The mid-dive blank-episode count it prints (pre-fix 3/1/1, fixed 0 in six runs)
separates that one here, and is reported rather than gated until the Radeon arm has been measured.

## Files

| file | what |
|------|------|
| `capdive.sh NAME EXE KFR [SEED] [T] [ITER] [PRIORITY] [TARGET]` | runs `--autodive` from a `.kfr` in a wiped scratch config, captures the window for 18 s; `PRIORITY` = `speed` / `quality` (the auto-zoom priority), `TARGET` = `detail` / `misiurewicz` (the auto-zoom target); `FRACTADYNE_BIGNUM` passes through |
| `capdive.ps1 -Out D -Exe E [-Kfr K] [-Seed S] [-TimeoutS T] [-Iter I]` | `capdive.sh` in PowerShell (defaults: `dive-2p800.kfr`, `session-seed.toml`, 26 s, 10,000 iterations); writes `exit.txt` too |
| `screengate.py RUN… [--bignum B] [--json F]` / `--selftest` | the screen gate: blank / flat / flash / stale, median of three, exit 0/1/2 (above) |
| `grab.ps1 -ProcId N -OutDir D -Seconds S -IntervalMs M` | PrintWindow capture of one process's window, DPI-aware, 480 px wide (PS 5.1) |
| `grab_full.ps1` | the same at native resolution — needed to read an 8 px toolbar glyph |
| `blankscore.py RUN…` | per run: frames, `SCREEN BLANK` count (canvas-interior stddev < 1), rung changes, empty passes, and a `#`/`.` timeline |
| `flashscore.py RUN… [--rect X0 X1 Y0 Y1]` | per run: FLASHES (captures whose canvas differs from the previous one by > 60 mean units/channel — a crisp frame giving way to a flat one, or back), flat frames, and a `!`/`F`/`.` timeline. Also runs on frames extracted from a screen recording (`--rect` names the canvas) — the 2026-09-20 recording scored 35% flat, 22 episodes |
| `motion.py RUN/stderr.txt…` | camera metrics from the `[fd-autopilot]` evals: focus-of-expansion off-centre and aim movement per look |
| `life.py RUN/stderr.txt` | octaves of runway at the goal and the aim: true distance estimate at 260 digits (mpmath), slow. Assumes a 1467×1102 panel — edit `W,H` |
| `mkpath.py RUN STEP` | every STEP-th eval as `path.tsv` (centre, l2, aim) for offline `--render` frames along the dive |
| `dive-2p800.kfr`, `dive-2p633.kfr` | the two reported locations (the 2^633 one does NOT reproduce the blank frames on this box) |
| `dive-2p584.kfr` | the start view of the 2026-09-20 "flashing / flat colour planes" recording (8.1e175×); with `session-2026-09-20.toml`, auto-iter (`ITER` 0) and `FRACTADYNE_BIGNUM=rug` it is that session |
| `session-seed.toml` | the reporting user's settings, personal paths removed |
| `session-2026-09-20.toml` | the same user's settings at the flashing report (base iterations 250,000; the auto-iter ask at 2^584 is ~152k against a picture complete by ~4,400) |

Log fields worth knowing: `norm reading EMPTY` = a pass committed no escaped pixel;
`norm range: frame [lo,hi]` = the view's escape range (hi at the iteration cap means the count,
not the fractal, ran out); `visible-res f=… → …` = the motion-resolution rung changing;
`chunk f=… cur=… step=…` = how far a walk got (`cur + step` against `lo`).
