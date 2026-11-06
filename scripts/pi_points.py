#!/usr/bin/env python3
"""The pi points: escape counts that pi predicts, beside the main cardioid's two pinch points.

    python scripts/pi_points.py points [--size N]
    python scripts/pi_points.py run --exe FRACTADYNE.EXE --out DIR [--size N] [--only SUBSTR]
                                    [--max-count N] [--checks N]

WHY
---
Two points of the Mandelbrot set's boundary are known exactly and come with a law for the escape
count beside them (Dave Boll, 1991; Aaron Klebanoff, "pi in the Mandelbrot set", 2001):
- the NECK c = -3/4, where the main cardioid meets the period-2 disc (they touch at this one point
  with a shared vertical tangent): on the vertical through it, c = -3/4 + i eps, the count is
  pi/eps plus about 1 (1e-6 above it: 3,141,593);
- the CUSP c = 1/4, the cardioid's cusp: on the real axis to its right, c = 1/4 + eps, the count
  is pi/sqrt(eps) minus about 2.
Neither is a deep test: the count grows with the zoom (10x closer is 10x the count at the neck,
100x closer at the cusp), and a view centered on either goes black past about 1e3x (neck) or 1e6x
(cusp), as the escaping gap narrows below a pixel. What they test is the high-iteration regime -
counts in the millions whose answer is known in advance - and the precision of c itself: at the
cusp a relative error in eps is half that relative error in the count (f64 already gets the 1e-12
point wrong by about 32: 0.25 + 1e-12 is not a double), and in f32 every point closer than about
1.5e-8 is 1/4 exactly and never escapes.

THE VIEWS
---------
Each point is the center of a SIZE x SIZE frame (SIZE odd, so the center pixel is the point
exactly; 17, as Fractadyne renders nothing under 16), small enough to lie inside the escaping gap: at the neck the gap at height eps runs from
-3/4 - 2 eps^2 (the disc) to -3/4 + 0.75 eps^2 (the cardioid); at the cusp it is the wedge
|y| < 2 eps^1.5. The frame's half-span is FILL of the nearer side.

JUDGING
-------
The truth is `fractadyne --truth --truth-direct` (plain double-double iteration, no reference: a
31-million-sample reference would not fit; checked against arbitrary precision at --checks
pixels), with the stencil. Every STABLE pixel of Fractadyne's render (`--render-iter`) must hold
the truth's smooth count within 0.5 plus one f32 step at that count: both store it as an f32,
which above 2^24 (16.7 million) holds only even numbers. A SENSITIVE pixel (its count changes
within 1/1000 px: the seahorse valley's filaments at eps = 0.1) is held to the stencil's range, as
truth_check.py does, and reported, not failed: no renderer can be held to a point there. The
center pixel - the pi point itself - must be stable, and the truth's own count there must meet the
pi law within PI_SLACK: a check on the truth, not on Fractadyne.
Exit 0 when every point passes, 1 otherwise, 2 on a usage error.
"""
import argparse
import json
import math
import os
import subprocess
import sys
import time
from decimal import Decimal

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import minibrot_offsets as MO  # noqa: E402  (fresh_env, parse_log)
import truth_check as TC  # noqa: E402

FILL = 0.25
PI_SLACK = 3   # the law's own offset: +0.4 to +1.6 at the neck, -1.4 to -2.3 at the cusp (radius 2)
NECK = range(1, 8)          # eps = 1e-1 .. 1e-7: 33 .. 31,415,9xx
CUSP = range(2, 15, 2)      # eps = 1e-2 .. 1e-14: 30 .. 31,415,9xx
# ⛔Points above this are skipped unless --max-count says otherwise. The neck at 1e-6 (3.1 million
# iterations a pixel, 17x17) LOST THE DEVICE on an RTX 3080, twice (2026-11-03: once, then again when
# Fractadyne's recovery relaunch re-ran the same command-line render). The cause: an odd-sized
# tile's helper lanes re-iterated from 0 on every resumed chunk pass (fixed in d9e7443e, branch
# fix/helper-lane-rewalk). With that fix, --max-count 4e6 ran safely on the RTX 3080 and on PLUTO's
# RTX 3070 and RX 6800 XT (longest GPU pass 71 ms); a build WITHOUT it can still lose the device.
# Above 7,452,444 iterations (these cards' reference-orbit limit, 1 GiB of samples) Fractadyne caps
# the count and says so in its log, so the two 31-million points cannot be rendered there at all.
SAFE_MAX_COUNT = 4e5


def exact(d):
    """A coordinate as 60 decimal places: `--center` reads a short decimal at the parser's natural
    precision (~64 bits) whatever the zoom, which put 0.25000000000001 (the cusp at 1e-14) about 40
    pixels from itself at 2^71.8 and moved its count by 5 (2026-11-03: 31,415,920, padded
    31,415,925, the law's 31,415,926.5 - 1.5). Padding the text carries the digits."""
    return format(Decimal(d), '.60f')


def points(size):
    out = []
    for k in NECK:
        eps = 10.0 ** -k
        half = FILL * 0.75 * eps * eps
        out.append({"id": "neck-1e-%d" % k, "kind": "neck", "eps": "1e-%d" % k, "re": exact("-0.75"),
                    "im": exact("1e-%d" % k), "law": math.pi / eps, "half": half})
    for k in CUSP:
        eps = 10.0 ** -k
        half = FILL * 2 * eps ** 1.5
        out.append({"id": "cusp-1e-%d" % k, "kind": "cusp", "eps": "1e-%d" % k,
                    "re": exact(Decimal("0.25") + Decimal("1e-%d" % k)), "im": exact("0"),
                    "law": math.pi / math.sqrt(eps), "half": half})
    for p in out:
        height = 2 * p["half"]                 # the frame is square: its height spans the gap
        p["zoom_log2"] = math.log2(4 / height)  # Fractadyne's view height is 4 / zoom
        p["cap"] = int(2 * p["law"]) + 1000
    out.sort(key=lambda p: p["law"])
    return out


def view_args(p, size):
    return ["--center", p["re"], p["im"], "--zoom-log2", repr(p["zoom_log2"]), "--size", "%dx%d" % (size, size),
            "--iter", str(p["cap"])]


def run(a):
    size = a.size
    out = os.path.abspath(a.out)
    exe = os.path.abspath(a.exe)
    for d in ("truth", "fd", "logs", "cfg"):
        os.makedirs(os.path.join(out, d), exist_ok=True)
    pts = [p for p in points(size) if (not a.only or a.only in p["id"]) and p["law"] <= a.max_count]
    mid = size // 2
    ok = True
    rows = []
    cols = ["id", "law", "truth_n2", "law_ok", "truth_smooth", "fd_smooth", "max_diff", "off", "sensitive",
            "sensitive_out", "interior_truth", "interior_fd", "verdict", "ref_len", "truth_s", "fd_s"]
    print("\t".join(cols), flush=True)
    for p in pts:
        # The truth.
        t0 = time.time()
        checks = a.checks if p["law"] < 1e6 else 1
        cmd = [exe, "--truth", os.path.join(out, "truth")] + view_args(p, size) + [
            "--truth-name", p["id"], "--truth-direct", "--truth-stencil", "--truth-check", str(checks)]
        r = subprocess.run(cmd, env=MO.fresh_env(os.path.join(out, "cfg", "truth")), capture_output=True, text=True,
                           timeout=7200)
        truth_s = time.time() - t0
        with open(os.path.join(out, "logs", "truth-%s.log" % p["id"]), "w", encoding="utf-8") as f:
            f.write(r.stdout + r.stderr)
        if r.returncode != 0:
            print("%s\ttruth FAILED (exit %d): %s" % (p["id"], r.returncode, (r.stdout + r.stderr)[-600:]), flush=True)
            ok = False
            continue
        t = TC.load_truth(os.path.join(out, "truth", p["id"]))
        n2 = int(t["n2"][mid, mid])
        law_ok = abs(n2 - p["law"]) <= PI_SLACK

        # Fractadyne.
        exr = os.path.join(out, "fd", p["id"] + ".exr")
        t0 = time.time()
        r = subprocess.run([exe, "--render-iter", "--out", exr] + view_args(p, size) + ["--ss", "1"],
                           env=MO.fresh_env(os.path.join(out, "cfg", "fd")), capture_output=True, text=True,
                           timeout=7200)
        fd_s = time.time() - t0
        # The log FILE has every line (the render's mode and GPU time among them); the console does not.
        log_file = os.path.join(out, "cfg", "fd", "logs", "fractadyne.log")
        text = open(log_file, encoding="utf-8", errors="replace").read() if os.path.exists(log_file) else r.stdout + r.stderr
        with open(os.path.join(out, "logs", "fd-%s.log" % p["id"]), "w", encoding="utf-8") as f:
            f.write(text)
        log = MO.parse_log(text)
        row = {"id": p["id"], "law": "%.1f" % p["law"], "truth_n2": n2, "law_ok": law_ok,
               "truth_smooth": "%.3f" % t["smooth256"][mid, mid], "truth_s": round(truth_s, 1), "fd_s": round(fd_s, 1),
               "mode": log.get("mode", ""), "ref_len": log.get("ref_len", ""),
               "interior_truth": int((t["n256"] < 0).sum())}
        if r.returncode != 0 or not os.path.exists(exr):
            row["verdict"] = "FAILED (exit %d)" % r.returncode
        else:
            v = TC.INSP.read_exr(exr)["R"].astype(np.float64)
            tv = t["smooth256"]
            stable = (t["flags"] & TC.FLAG_STABLE) != 0
            both = (v >= 0) & (tv >= 0)
            tol = 0.5 + np.spacing(np.abs(tv).astype(np.float32)).astype(np.float64)
            diff = np.where(both & stable, np.abs(v - tv), 0.0)
            # STABLE pixels (all five stencil points agree): the smooth count, strictly; and
            # escaping where the truth did not, or the reverse.
            off = int((both & stable & (diff > tol)).sum()) + int((stable & ((v >= 0) != (tv >= 0))).sum())
            # SENSITIVE pixels (the count changes within 1/1000 px: filaments) as truth_check does:
            # the whole count against the stencil's range at radius 256 (Fractadyne's smooth count
            # sits 2 to 3 below n256), SLACK either way. Reported; they cannot be held to a point.
            x = np.floor(v) + 3
            lo, hi = t["n256_lo"], t["n256_hi"]
            sens_out = ~stable & (((v >= 0) & (((lo >= 0) & (x < lo - TC.SLACK)) | (x > hi + TC.SLACK)))
                                  | ((v < 0) & (lo >= 0)))
            row.update({"fd_smooth": "%.3f" % v[mid, mid], "max_diff": "%.3g" % float(diff.max()), "off": off,
                        "sensitive": int((~stable).sum()), "sensitive_out": int(sens_out.sum()),
                        "interior_fd": int((v < 0).sum())})
            centre_ok = bool(stable[mid, mid])
            row["verdict"] = ("PASS" if off == 0 and law_ok and centre_ok else
                              "DEVIATES" if off else "LAW?" if not law_ok else "CENTER SENSITIVE")
        ok &= row["verdict"] == "PASS"
        rows.append(row)
        print("\t".join(str(row.get(k, "")) for k in cols), flush=True)
    with open(os.path.join(out, "pi-points.json"), "w", encoding="utf-8", newline="\n") as f:
        json.dump(rows, f, indent=1)
    print("pi points: %s" % ("PASS" if ok else "FAIL"))
    return 0 if ok else 1


def main():
    ap = argparse.ArgumentParser(description="escape counts that pi predicts, at the neck and the cusp")
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("points")
    s.add_argument("--size", type=int, default=17)
    s = sub.add_parser("run")
    s.add_argument("--exe", required=True)
    s.add_argument("--out", required=True)
    s.add_argument("--size", type=int, default=17)
    s.add_argument("--only", help="points whose id contains this")
    s.add_argument("--max-count", type=float, default=SAFE_MAX_COUNT,
                   help="skip points whose law count is larger (default %g: see SAFE_MAX_COUNT)" % SAFE_MAX_COUNT)
    s.add_argument("--checks", type=int, default=3, help="plain-iteration checks per truth (1 above a million)")
    a = ap.parse_args()
    if a.size % 2 == 0:
        sys.stderr.write("pi_points: --size must be odd (the center pixel is the point)\n")
        return 2
    if a.cmd == "points":
        for p in points(a.size):
            print("%-12s c = %s + %si  law %.1f  zoom 2^%.2f  cap %d" % (
                p["id"], p["re"], p["im"], p["law"], p["zoom_log2"], p["cap"]))
        return 0
    return run(a)


if __name__ == "__main__":
    sys.exit(main())
