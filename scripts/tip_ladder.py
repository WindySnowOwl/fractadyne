#!/usr/bin/env python3
"""The tip ladder: one view of the tip of the Mandelbrot set's antenna, at every depth.

    python scripts/tip_ladder.py views [--size WxH] [--deep]
    python scripts/tip_ladder.py run --exe FRACTADYNE.EXE --out DIR [--size WxH] [--rungs L,L,...]
                                     [--confirm L,L,...] [--deep] [--png]

WHY
---
c = -2, the left end of the antenna, is a Misiurewicz point: its orbit (-2, 2, 2, ...) lands on the
repelling fixed point z = 2, whose multiplier is 4. Near -2 the set is asymptotically self-similar
with factor 4 and no rotation, and so are the escape counts: zooming 4x about -2 adds exactly one
iteration to every pixel's count. (Exact in the limit, with an error that shrinks 4x a step;
checked 2026-11-03 on a 64x36 frame at 4^6, 4^20 and 4^60: every pixel +1, none off.) At radius
256 and 4096, that is: every point left of -2 has |c| > 2 and passes radius 2 at the first step at
every depth, so radius-2 counts are not judged here. So one truth
at a shallow depth is the truth at EVERY depth, shifted by the number of 4x steps: a check that
needs no reference at the depth it checks, through every change of arithmetic a renderer makes on
the way down (f32's exponent range near 1e38x, f64's near 1e308x, extended formats beyond).

THE VIEW
--------
Rung L is zoom 2^L (L even: two of L per 4x step). The view's height is 4 / zoom, and its center is
-2 + (U_RE + U_IM i) x height. The offset is a fixed fraction of the frame, so it shrinks with the
zoom and every rung shows the same picture: the tip U_RE heights left of the center and U_IM below
it. Not centered on -2 exactly, because
- -2 is a two-bit number and its orbit is exact in any arithmetic, so a center there tests none of
  the deep machinery; the offset center needs every digit of the depth;
- that orbit sits at |z| = 2, ON the radius-2 escape test, where > against >= decides it;
- the real axis there is the antenna, wholly inside the set: a pixel row on it would never escape,
  and nothing could prove it interior. The offset keeps every pixel center off the axis (the
  nearest is 0.1 px from it at 360 rows).
The pixels next to the antenna are a filament test of their own: the antenna is a line of zero
width, and the counts climb towards it.

JUDGING
-------
`run` computes the truth (`fractadyne --truth`, with the stencil) at the base rung, then
- CONFIRMS the law in the truth itself: recomputes the truth directly at the --confirm rungs and
  requires it to match the base shifted (equal counts wherever both are stable, overlapping ranges
  elsewhere, smooth counts within SMOOTH_TOL). That checks the truth engine at depth too: two
  independent routes to one answer. It found the engine's first fault on its first run
  (2026-11-04): both of the engine's routes finished each orbit in f64 from radius 2, and near -2
  an orbit hovers just outside |z| = 2 for as many steps as the zoom is deep.
- renders every rung with Fractadyne (`--render-iter`, a fresh process and config folder each) and
  scores it against the base truth shifted (`truth_check.py score --shift`); a rung shallower than
  LAW_MIN against a truth of its own. Each rung is also rendered as a PNG (`--render`), to look at
  and for the log line naming the GPU arithmetic (the mode column).
Coordinates go through an args file (`@FILE`): at 1e10000x the center has 10,000 digits, and a
Windows command line holds 32,767 characters.
Exit 0 when every confirmation holds and every rung passes, 1 otherwise, 2 on a usage error.
"""
import argparse
import json
import math
import os
import subprocess
import sys
import time
from decimal import Decimal, localcontext

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import minibrot_offsets as MO  # noqa: E402  (fresh_env, parse_log)
import truth_check as TC  # noqa: E402

U_RE = Decimal("0.4937")
U_IM = Decimal("0.0711")
BASE = 40
# Through f32's exponent range (pixels 2^-126 apart near L = 120) and f64's (near L = 1016), then
# 1e1000x, 1e3010x and 1e10000x.
# Fractadyne's arithmetic on the RTX 3080 (2026-11-04): direct, no reference (mode 1) at 1e2-1e3x;
# df32 perturbation (mode 0) from 1e4x to 1e27x; floatexp perturbation with BLA (mode 2) from 1e28x.
RUNGS = [8, 12, 16, 20, 40, 42, 92, 94, 96, 100, 116, 120, 124, 128, 132, 200, 332, 600, 1000, 1012,
         1016, 1020, 1024, 1028, 2000, 3322, 10000, 33220]
DEEP = [332192]  # 1e100000x: a 332,000-bit reference
MIN_RUNG = 8
# Shallower than this the law is not exact enough to judge by (its error shrinks 4x a step: against
# the base shifted, Fractadyne's 1e2x render had 2,483 pixels off and its 1e3x 139, but against a
# truth computed at their own depth 0 and 1). Those rungs are judged by their own truth, which is
# cheap there.
LAW_MIN = 24
CONFIRM = [42, 332, 1024, 3322]
# Smooth counts at a pixel stable in both truths must agree within this plus two f32 steps at the
# count (the file holds them as f32: 2^-10 near 16,000). The law's own error is far below.
SMOOTH_TOL = 1e-4


def cap(level):
    # Counts run about L/2 plus a frame-shaped part under 30; the pixels nearest the antenna most.
    return 2 * level + 1000


def centre(level):
    """-2 + (U_RE + U_IM i) * 4 / 2^L as decimal text, to about 2^-(L+48): pixels are 2^-(L+6.5)
    apart at 360 rows."""
    with localcontext() as ctx:
        ctx.prec = int((level + 48) * 0.30103) + 12
        height = Decimal(4) / (Decimal(2) ** level)
        re = Decimal(-2) + U_RE * height
    with localcontext() as ctx:
        ctx.prec = 40
        im = U_IM * (Decimal(4) / (Decimal(2) ** level))
    return MO.fmt(re), MO.fmt(im)


def rungs_of(a):
    if a.rungs:
        out = [int(x) for x in a.rungs.split(",")]
    else:
        out = RUNGS + (DEEP if a.deep else [])
    bad = [x for x in out if x < MIN_RUNG or (x - BASE) % 2]
    if bad:
        raise SystemExit("tip_ladder: rungs must be even and at least %d: %s" % (MIN_RUNG, bad))
    return out


def write_args(path, level, size):
    re, im = centre(level)
    with open(path, "w", encoding="ascii", newline="\n") as f:
        f.write("# tip ladder rung L=%d: zoom 2^%d, center -2 + (%s + %si) x height\n" % (level, level, U_RE, U_IM))
        f.write("--center %s %s\n--zoom-log2 %d\n--size %dx%d\n--iter %d\n" % (re, im, level, size[0], size[1], cap(level)))


def run_truth(exe, out, level, size, env_dir, checks):
    args = os.path.join(out, "args", "tip-L%d.args" % level)
    cmd = [exe, "--truth", os.path.join(out, "truth"), "@" + args, "--truth-name", "tip-L%d" % level,
           "--truth-stencil", "--truth-check", str(checks)]
    t0 = time.time()
    p = subprocess.run(cmd, env=MO.fresh_env(env_dir), capture_output=True, text=True, timeout=7200)
    with open(os.path.join(out, "logs", "truth-L%d.log" % level), "w", encoding="utf-8") as f:
        f.write(p.stdout + p.stderr)
    if p.returncode != 0:
        raise SystemExit("tip_ladder: --truth at L=%d failed (exit %d); see logs/truth-L%d.log\n%s" % (
            level, p.returncode, level, (p.stdout + p.stderr)[-1500:]))
    return os.path.join(out, "truth", "tip-L%d" % level), time.time() - t0


def confirm(base_stem, stem, shift):
    """The truth at a deeper rung against the base truth shifted: the law, and the engine at depth."""
    a, b = TC.load_truth(base_stem, shift), TC.load_truth(stem)
    st_a = (a["flags"] & TC.FLAG_STABLE) != 0
    st_b = (b["flags"] & TC.FLAG_STABLE) != 0
    both = st_a & st_b
    names = ("n256", "n4096")  # not n2: see WHY
    # A whole count at 256 may flip only where the smooth count sits on a whole number.
    on_edge = np.abs(a["smooth256"] - np.round(a["smooth256"])) <= SMOOTH_TOL
    differ = both & (a["n4096"] != b["n4096"])
    differ |= both & ~on_edge & (a["n256"] != b["n256"])
    # Elsewhere the ranges must meet (lo -1: some point never escaped, no lower bound).
    apart = np.zeros(both.shape, dtype=bool)
    for n in names:
        lo_a = np.where(a[n + "_lo"] < 0, -np.inf, a[n + "_lo"])
        lo_b = np.where(b[n + "_lo"] < 0, -np.inf, b[n + "_lo"])
        hi_a = np.where(a[n + "_hi"] < 0, -np.inf, a[n + "_hi"])
        hi_b = np.where(b[n + "_hi"] < 0, -np.inf, b[n + "_hi"])
        # Both ranges empty of escapes (hi -1) meet; otherwise [lo, hi] intervals must overlap.
        none_a, none_b = a[n + "_hi"] < 0, b[n + "_hi"] < 0
        overlap = (np.maximum(lo_a, lo_b) <= np.minimum(hi_a, hi_b)) | (none_a & none_b)
        apart |= ~both & ~overlap
    esc = both & (a["smooth256"] >= 0) & (b["smooth256"] >= 0)
    diff = np.abs(a["smooth256"] - b["smooth256"])
    tol = SMOOTH_TOL + 2 * np.spacing(np.abs(b["smooth256"]).astype(np.float32)).astype(np.float64)
    sm = float(np.max(diff[esc])) if esc.any() else 0.0
    smooth_bad = int((esc & (diff > tol)).sum())
    return {"stable_both": int(both.sum()), "stable_differ": int(differ.sum()),
            "ranges_apart": int(apart.sum()), "smooth_max_diff_stable": sm, "smooth_off": smooth_bad,
            "ok": int(differ.sum()) == 0 and int(apart.sum()) == 0 and smooth_bad == 0}


def render_fd(exe, out, level, env_dir):
    args = os.path.join(out, "args", "tip-L%d.args" % level)
    exr = os.path.join(out, "fd", "tip-L%d.exr" % level)
    cmd = [exe, "--render-iter", "--out", exr, "@" + args, "--ss", "1"]
    t0 = time.time()
    p = subprocess.run(cmd, env=MO.fresh_env(env_dir), capture_output=True, text=True, timeout=3600)
    secs = time.time() - t0
    # The log FILE has every line (the render's mode and GPU time among them); the console does not.
    log_file = os.path.join(env_dir, "logs", "fractadyne.log")
    text = open(log_file, encoding="utf-8", errors="replace").read() if os.path.exists(log_file) else p.stdout + p.stderr
    with open(os.path.join(out, "logs", "fd-L%d.log" % level), "w", encoding="utf-8") as f:
        f.write(text)
    info = MO.parse_log(text)
    # The same view as a colored PNG: something to look at, and its log names the arithmetic the
    # GPU used (the `cli-render ... mode=` line, which the --render-iter path does not write).
    shot = os.path.join(out, "png", "tip-L%d.png" % level)
    # ⛔--render: `--out` alone opens the WINDOWED app (2026-11-04, twice, on the user's desktop).
    q = subprocess.run([exe, "--render", "--out", shot, "@" + args, "--ss", "1"], env=MO.fresh_env(env_dir),
                       capture_output=True, text=True, timeout=3600)
    if q.returncode == 0 and os.path.exists(log_file):
        shot_info = MO.parse_log(open(log_file, encoding="utf-8", errors="replace").read())
        info["mode"], info["gpu_ms"] = shot_info["mode"], shot_info["gpu_ms"]
    return exr, p.returncode, secs, info


COLS = ["L", "zoom", "judged_by", "verdict", "judged", "deviating", "deviating_patches", "escaped_in_one",
        "escaped_in_one_patches", "median_abs_diff_stable", "mode", "ref_len", "escaped", "periodic",
        "pick_offset", "gpu_ms", "seconds", "exit"]


def run(a):
    size = tuple(int(x) for x in a.size.split("x"))
    out = os.path.abspath(a.out)
    exe = os.path.abspath(a.exe)
    for d in ("args", "truth", "fd", "png", "logs", "maps", "cfg"):
        os.makedirs(os.path.join(out, d), exist_ok=True)
    levels = rungs_of(a)
    if a.confirm is None:
        conf = CONFIRM
    else:
        conf = [int(x) for x in a.confirm.split(",") if x]
    for level in sorted(set(levels + conf + [BASE])):
        write_args(os.path.join(out, "args", "tip-L%d.args" % level), level, size)

    base, secs = run_truth(exe, out, BASE, size, os.path.join(out, "cfg", "truth"), a.checks)
    head = TC.load_truth(base)["head"]
    print("base truth L=%d: %.1f s; %s" % (BASE, secs, json.dumps(head["pixels"])), flush=True)

    ok = True
    report = {"base": BASE, "size": list(size), "u": [str(U_RE), str(U_IM)], "confirm": {}, "rungs": []}
    for level in conf:
        if level == BASE:
            continue
        stem, secs = run_truth(exe, out, level, size, os.path.join(out, "cfg", "truth"), a.checks)
        c = confirm(base, stem, (level - BASE) // 2)
        c["seconds"] = round(secs, 1)
        report["confirm"][level] = c
        ok &= c["ok"]
        print("confirm L=%-6d %s  stable in both %d, differ %d, ranges apart %d, smooth max diff %.2g (%d off)  (%.1f s)" % (
            level, "OK " if c["ok"] else "BAD", c["stable_both"], c["stable_differ"], c["ranges_apart"],
            c["smooth_max_diff_stable"], c["smooth_off"], secs), flush=True)

    print("\t".join(COLS), flush=True)
    for level in levels:
        exr, code, secs, log = render_fd(exe, out, level, os.path.join(out, "cfg", "fd"))
        if level < LAW_MIN:
            judge, _ = run_truth(exe, out, level, size, os.path.join(out, "cfg", "truth"), a.checks)
            shift = 0
        else:
            judge, shift = base, (level - BASE) // 2
        row = {"L": level, "zoom": "1e%d" % int(level * math.log10(2)), "shift": shift, "exit": code,
               "seconds": round(secs, 1), "judged_by": "own truth" if level < LAW_MIN else "base +%d" % shift}
        row.update({k: log.get(k, "") for k in ("mode", "ref_len", "escaped", "periodic", "pick_offset", "gpu_ms")})
        if code == 0 and os.path.exists(exr):
            png = os.path.join(out, "maps", "tip-L%d.png" % level) if a.png else None
            s = TC.score(judge, exr, "fd", png, shift=shift)
            row.update({k: s[k] for k in COLS if k in s and k not in row})
            row["verdict"] = s["verdict"]
        else:
            row["verdict"] = "FAILED"
        ok &= row["verdict"] == "PASS"
        report["rungs"].append(row)
        print("\t".join(str(row.get(k, "")) for k in COLS), flush=True)

    with open(os.path.join(out, "tip-ladder.json"), "w", encoding="utf-8", newline="\n") as f:
        json.dump(report, f, indent=1)
    with open(os.path.join(out, "tip-ladder.tsv"), "w", encoding="utf-8", newline="\n") as f:
        f.write("\t".join(COLS) + "\n")
        for row in report["rungs"]:
            f.write("\t".join(str(row.get(k, "")) for k in COLS) + "\n")
    print("tip ladder: %s" % ("PASS" if ok else "FAIL"))
    return 0 if ok else 1


def views(a):
    size = tuple(int(x) for x in a.size.split("x"))
    for level in rungs_of(a):
        re, im = centre(level)
        print("L=%-6d zoom 1e%-6d +%-6d iter %-7d center %s... %s" % (
            level, int(level * math.log10(2)), (level - BASE) // 2, cap(level), re[:24], im))
    print("(%dx%d; truth at L=%d)" % (size[0], size[1], BASE))
    return 0


def main():
    ap = argparse.ArgumentParser(description="the tip ladder: one view of the antenna tip at every depth")
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name in ("views", "run"):
        s = sub.add_parser(name)
        s.add_argument("--size", default="640x360")
        s.add_argument("--rungs", help="comma-separated zoom-log2 levels (even, >= %d)" % MIN_RUNG)
        s.add_argument("--deep", action="store_true", help="add 1e100000x (%s)" % DEEP)
        if name == "run":
            s.add_argument("--exe", required=True)
            s.add_argument("--out", required=True)
            s.add_argument("--confirm", help="rungs to recompute the truth at directly (default %s; '' for none)"
                           % ",".join(map(str, CONFIRM)))
            s.add_argument("--checks", type=int, default=200, help="plain-iteration checks per truth")
            s.add_argument("--png", action="store_true", help="write each rung's scorer map")
    a = ap.parse_args()
    return run(a) if a.cmd == "run" else views(a)


if __name__ == "__main__":
    sys.exit(main())
