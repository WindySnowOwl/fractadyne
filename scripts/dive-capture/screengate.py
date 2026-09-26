"""The SCREEN gate: judge what three live dives put on screen (design/live-render-robustness.md §7.6,
W6's screen-level gate).

Every per-pass metric in the log said the 2026-09-20 blank-motion-frame fix was working while the
screen got worse; only capturing the window showed it (README.md). This turns that capture into a
gate: three runs of `capdive.sh` / `capdive.ps1` at the reported regime, each scored on four things
a viewer sees, the median of the three held to a bound, and an exit code.

    python scripts/dive-capture/screengate.py RUN1 RUN2 RUN3 [--bignum astro-float] [--json out.json]
    python scripts/dive-capture/screengate.py --selftest     # every criterion seen to fire

  blank   a capture whose canvas is one colour (interior grey stddev < 1) — the 2026-09-20 report
  flat    a capture whose canvas is >97% within 8 units of its median colour (a near-flat panel)
  flash   an EPISODE: the canvas switching into a flat capture or out of one — what the eye reads as
          a flash. ⚠Not "a big frame-to-frame difference": at the 2^800 fixture an ordinary zoom step
          between captures differs by 35–75 units, so flashscore.py's FLASH_DIFF of 60 counted 26–43%
          of healthy transitions as flashes (2026-09-26). It is kept for recordings; this is the gate.
  stale   the longest run of consecutive captures with IDENTICAL canvases (mean difference < 0.5)
          in a run whose view was moving — a frozen pin or a smear (U39/U44), which a blank score
          cannot see because a frozen picture is not blank. The dive must move for this to mean
          anything: a run whose median capture-to-capture difference is under MOVING_DIFF is VACUOUS.

Verdict (exit code): 0 PASS · 1 RED — median blank or flat above 7% (the measured post-112a088 band
is 1–7%; single runs vary 8–14%, hence three), median flash episodes above 3, a median-run stale
streak of 3 or more, or a blank spread across the runs wider than 10 points · 2 VACUOUS — fewer than
three runs, a run with fewer than MIN_CAPTURES captures, a run that did not move, or a run whose log
does not name the declared bignum backend (copying fractadyne.exe out of the accelerated package
drops the MPFR DLLs and once scored 3% blank against the user's 35%).

Calibration (RTX 3080, 2026-09-26, dive-2p800.kfr + session-seed.toml, 10,000 iterations, the
startup captures before the first picture excluded; interleaved against the build before the
blank-frame fix, 112a088^1):
    fixed (beta.120, 6 runs)   blank 0% in 0 episodes, flat 0%, flash 0, stale 0 — every run
    pre-fix (3 runs)           blank 4.7 / 2.9 / 1.0% in 3 / 1 / 1 episodes, flash 6 / 2 / 2
⚠So on this box today the gate does NOT go red on the pre-fix build: it is measurably worse, but
under the design's 7%. That bound was set when the pre-fix build scored 8-14% here and the fixed one
1-7%; the regime is milder now, and the user's live sessions were milder still than theirs (35%).
The gate catches a severe regression; the episode count (reported, not gated) separates this one.
"""
import argparse, glob, json, os, re, statistics, sys

import numpy as np
from PIL import Image

BLANK_MAX_PCT = 7.0
FLAT_MAX_PCT = 7.0
FLASH_MAX = 3
STALE_MAX = 2          # a streak of 3 identical captures (~0.5 s) is RED
SPREAD_MAX_PCT = 10.0
MIN_CAPTURES = 60
MOVING_DIFF = 5.0      # median capture-to-capture difference below this: the dive did not move
SAME_DIFF = 0.5        # below this, two captures show the same picture


def frames_of(run):
    fs = glob.glob(os.path.join(run, "frames", "*.jpg")) + glob.glob(os.path.join(run, "frames", "*.png"))
    # grab.ps1 names them f<ms since start>: sort by that number, not as text.
    def ms(p):
        m = re.search(r"f(\d+)", os.path.basename(p))
        return int(m.group(1)) if m else 0
    return sorted(fs, key=ms)


def backend_of(run):
    for name in ("stderr.txt", os.path.join("cfg", "logs", "fractadyne.log")):
        p = os.path.join(run, name)
        if os.path.exists(p):
            m = re.search(r"bignum backend selected: (\S+)", open(p, encoding="utf-8", errors="replace").read())
            if m:
                return m.group(1)
    return None


def log_octaves(run):
    """How far the autopilot's recorded view travelled (its `eval … l2=` lines), or None."""
    p = os.path.join(run, "stderr.txt")
    if not os.path.exists(p):
        return None
    l2 = [float(m.group(1)) for m in re.finditer(r"\[fd-autopilot\].* eval .*?\bl2=([-0-9.]+)", open(p, encoding="utf-8", errors="replace").read())]
    return (max(l2) - min(l2)) if l2 else None


def score(run):
    fs = frames_of(run)
    out = {"run": run, "captures": len(fs), "backend": backend_of(run), "log_octaves": log_octaves(run)}
    if not fs:
        return out
    W, H = Image.open(fs[0]).size
    x0, x1, y0, y1 = int(W * 0.28), int(W * 0.72), int(H * 0.25), int(H * 0.85)
    blank, flat, diffs = [], [], []
    prev = None
    for f in fs:
        im = np.asarray(Image.open(f).convert("RGB"), dtype=np.int16)[y0:y1, x0:x1]
        grey = np.asarray(Image.open(f).convert("L"), dtype=np.float32)[y0:y1, x0:x1]
        blank.append(float(grey.std()) < 1.0)
        med = np.median(im.reshape(-1, 3), axis=0)
        flat.append(float((np.abs(im - med).sum(axis=2) <= 8).mean()) > 0.97)
        if prev is not None:
            diffs.append(float(np.abs(im - prev).mean()))
        prev = im
    timeline = "".join("#" if b else ("F" if f else ".") for b, f in zip(blank, flat))
    # The captures before the dive's first picture are its START, not a motion frame: a capture that
    # begins before the first frame is drawn (the 88-capture runs) scores them as blank. Counted apart.
    start = len(timeline) - len(timeline.lstrip("#F"))
    blank, flat, diffs = blank[start:], flat[start:], diffs[start:]
    streak = best = 0
    for d in diffs:
        streak = streak + 1 if d < SAME_DIFF else 0
        best = max(best, streak)
    n = max(len(blank), 1)
    out.update(
        startup=start,
        blank_pct=100.0 * sum(blank) / n,
        flat_pct=100.0 * sum(flat) / n,
        flash=sum(1 for a, b in zip(flat, flat[1:]) if a != b),
        # Separate stretches of blank captures mid-dive. REPORTED, not gated: on the RTX 3080 on
        # 2026-09-26 it separated the pre-112a088 build (3, 1, 1) from the fixed one (0 in six
        # runs), but the fixed build scored 1-7% blank in the conditions it was fixed under, so a
        # bound of one episode is calibrated in today's mild regime (P12).
        episodes=len(re.findall(r"#+", timeline[start:])),
        stale=best,
        moving=bool(diffs) and statistics.median(diffs) >= MOVING_DIFF,
        diff_median=statistics.median(diffs) if diffs else 0.0,
        timeline=timeline,
    )
    return out


def selftest():
    """Every RED and VACUOUS criterion, each fed a synthetic run built to trip it, beside a healthy
    control that must PASS. A gate that has never been seen to go red is not a gate."""
    import tempfile
    rng = np.random.default_rng(7)

    def run(root, name, kinds, backend="astro-float", travelled=0.0):
        d = os.path.join(root, name)
        os.makedirs(os.path.join(d, "frames"))
        with open(os.path.join(d, "stderr.txt"), "w", encoding="utf-8") as f:
            f.write(f"[fd-start] [+ 0.02s] bignum backend selected: {backend}\n")
            for t, l2 in ((1.0, 700.0), (20.0, 700.0 + travelled)):
                f.write(f"[fd-autopilot] [+ {t:.3f}s] eval t={t:.3f} l2={l2:.4f} goal_was=None speed=1.0\n")
        last = None
        for i, k in enumerate(kinds):
            if k == "blank":
                im = np.full((60, 80, 3), 40, np.uint8)
            elif k == "same" and last is not None:
                im = last
            elif k == "still":
                im = np.full((60, 80, 3), 90, np.uint8)
                im[::2] = 160  # structure, but the same every capture
            else:
                im = rng.integers(0, 255, (60, 80, 3), dtype=np.uint8)
            Image.fromarray(im).save(os.path.join(d, "frames", f"f{i * 170}.png"))
            last = im
        return d

    healthy = ["move"] * 80
    cases = {
        # name: (three runs' frame kinds, expected code)
        "control": ([healthy] * 3, 0),
        "blank": ([["move"] * 70 + ["blank"] * 10] * 3, 1),
        "flash": ([["move", "move", "blank"] * 27] * 3, 1),
        "stale": ([["move"] * 40 + ["same"] * 5 + ["move"] * 35] * 3, 1),
        "spread": ([healthy, healthy, ["move"] * 68 + ["blank"] * 12], 1),
        "two runs": ([healthy] * 2, 2),
        "too few captures": ([["move"] * 20] * 3, 2),
        "did not move": ([["still"] * 80] * 3, 2),
    }
    ok = True
    with tempfile.TemporaryDirectory() as root:
        for name, (kind_lists, want) in cases.items():
            dirs = [run(root, f"{name.replace(' ', '_')}{i}", k) for i, k in enumerate(kind_lists)]
            got = subprocess_gate(dirs)
            flag = "PASS" if got == want else "FAIL"
            ok &= got == want
            print(f"selftest: {flag} — {name}: exit {got} (want {want})")
        wrong = [run(root, f"wrongbackend{i}", healthy, backend="rug") for i in range(3)]
        got = subprocess_gate(wrong)
        ok &= got == 2
        print(f"selftest: {'PASS' if got == 2 else 'FAIL'} — an undeclared bignum backend: exit {got} (want 2)")
        # The screen never changed while the autopilot's view travelled 20 octaves: frozen, RED.
        frozen = [run(root, f"frozen{i}", ["still"] * 80, travelled=20.0) for i in range(3)]
        got = subprocess_gate(frozen)
        ok &= got == 1
        print(f"selftest: {'PASS' if got == 1 else 'FAIL'} — frozen while the view moved: exit {got} (want 1)")
    print("selftest:", "PASS" if ok else "FAIL")
    return 0 if ok else 1


def subprocess_gate(dirs):
    import subprocess
    p = subprocess.run([sys.executable, os.path.abspath(__file__), *dirs], capture_output=True, text=True)
    return p.returncode


def main():
    if sys.argv[1:] == ["--selftest"]:
        return selftest()
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("runs", nargs="+")
    ap.add_argument("--bignum", default="astro-float", help="the backend the fixture declares (default astro-float)")
    ap.add_argument("--json", help="write the per-run scores and the verdict here")
    a = ap.parse_args()

    runs = [score(r) for r in a.runs]
    vacuous, red = [], []
    if len(runs) < 3:
        vacuous.append(f"{len(runs)} run(s); the gate needs three (single runs vary 8-14%)")
    for r in runs:
        name = os.path.basename(os.path.normpath(r["run"]))
        if r["captures"] < MIN_CAPTURES:
            vacuous.append(f"{name}: {r['captures']} captures (< {MIN_CAPTURES})")
            continue
        if r["backend"] != a.bignum:
            vacuous.append(f"{name}: bignum backend {r['backend']!r}, the fixture declares {a.bignum!r}")
        if not r["moving"]:
            # The canvas alone cannot tell a dive that never moved from a screen that froze while
            # it did; the autopilot's own record of the view can.
            if (r["log_octaves"] or 0.0) >= 1.0:
                red.append(f"{name}: FROZEN — the view travelled {r['log_octaves']:.1f} octaves and the screen did not change "
                           f"(median capture difference {r['diff_median']:.1f})")
            else:
                vacuous.append(f"{name}: the view did not move (median capture difference {r['diff_median']:.1f} < {MOVING_DIFF})")
        print(f"{name:14s} captures {r['captures']:3d} (startup {r['startup']}) | blank {r['blank_pct']:5.1f}% "
              f"in {r['episodes']} episode(s) | flat {r['flat_pct']:5.1f}% | flash {r['flash']:2d} | "
              f"stale streak {r['stale']:2d} | diff median {r['diff_median']:5.1f} | {r['backend']}")
        print("               " + r["timeline"])

    scored = [r for r in runs if r["captures"] >= MIN_CAPTURES]
    verdict = {}
    if scored:
        med = lambda k: statistics.median(r[k] for r in scored)
        verdict = {k: med(k) for k in ("blank_pct", "flat_pct", "flash", "episodes")}
        # A streak of identical captures means "frozen" only where the view was moving.
        moving = [r["stale"] for r in scored if r["moving"]]
        verdict["stale"] = statistics.median(moving) if moving else 0
        spread = max(r["blank_pct"] for r in scored) - min(r["blank_pct"] for r in scored)
        verdict["blank_spread"] = spread
        if verdict["blank_pct"] > BLANK_MAX_PCT:
            red.append(f"median blank {verdict['blank_pct']:.1f}% > {BLANK_MAX_PCT}%")
        if verdict["flat_pct"] > FLAT_MAX_PCT:
            red.append(f"median flat {verdict['flat_pct']:.1f}% > {FLAT_MAX_PCT}%")
        if verdict["flash"] > FLASH_MAX:
            red.append(f"median flash episodes {verdict['flash']} > {FLASH_MAX}")
        if verdict["stale"] > STALE_MAX:
            red.append(f"median stale streak {verdict['stale']} captures > {STALE_MAX} (a frozen picture while the view moved)")
        if spread > SPREAD_MAX_PCT:
            red.append(f"blank spread {spread:.1f} points across the runs > {SPREAD_MAX_PCT} (the measurement itself is unstable)")
        print(f"\nmedian of {len(scored)}: blank {verdict['blank_pct']:.1f}% in {verdict['episodes']} episode(s) (reported, "
              f"not gated) · flat {verdict['flat_pct']:.1f}% · flash {verdict['flash']} · stale streak {verdict['stale']} · "
              f"blank spread {spread:.1f} points")

    # A set that fails the richness test gets no verdict (§6.6.2): VACUOUS outranks RED, and both
    # are non-zero. Every reason is printed either way.
    code = 2 if vacuous else (1 if red else 0)
    word = {0: "PASS", 1: "RED", 2: "VACUOUS"}[code]
    for v in red:
        print(f"screengate: RED — {v}")
    for v in vacuous:
        print(f"screengate: VACUOUS — {v}")
    print(f"screengate: {word}")
    if a.json:
        with open(a.json, "w", encoding="utf-8") as f:
            json.dump({"verdict": word, "code": code, "red": red, "vacuous": vacuous, "median": verdict, "runs": runs}, f, indent=1)
    return code


if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except (AttributeError, ValueError):
            pass
    sys.exit(main())
