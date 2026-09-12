"""Validate a cutoff-crossing tour three ways, to catch bugs in the arithmetic-mode switch and in
the optimizations that reuse work across frames of a dive.

For each tour (see gen-crossing-tours.py):

  1. COLD    -- render the tour with cross-frame reference reuse OFF
               (FRACTADYNE_NO_PREFETCH=1, --no-orbit-cache). Each frame is built from scratch.
               This is the ground truth.
  2. WARM    -- render the same tour with reuse ON (the sequencing/caching path that a real dive
               uses). Diff every frame against COLD: they must be BIT-IDENTICAL. Any difference is
               a caching/sequencing optimization changing the picture, which is a bug.
  3. LIVE    -- run --livetest, which plays the tour through the live, multithreaded pipeline and
               compares each checkpoint against its own offline oracle. Reports drift.

Both COLD and WARM go through the identical --render-tour code path, so the only variable is the
reuse, which is what isolates the optimization under test.

Usage:
  python bench-kit/tools/cutoff-validate.py TOUR.toml [--exe PATH] [--size WxH] [--out DIR] [--quick]

Exit 0 if COLD==WARM for every frame (and LIVE has no drift unless --quick); non-zero otherwise.
"""
import argparse
import os
import subprocess
import sys
import tempfile

from PIL import Image, ImageChops

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))


def render_tour(exe, tour, out_dir, size, reuse, cfg_dir):
    os.makedirs(out_dir, exist_ok=True)
    env = dict(os.environ)
    env["FRACTADYNE_NO_SOUND"] = "1"
    env["FRACTADYNE_CONFIG_DIR"] = cfg_dir
    args = [exe, "--render-tour", tour, "--out", out_dir]
    if size:
        args += ["--size", size]
    if reuse:
        args += ["--orbit-cache"]          # disk cache on; prefetch is on by default for tours
    else:
        env["FRACTADYNE_NO_PREFETCH"] = "1"
        args += ["--no-orbit-cache"]
    t = subprocess.run(args, env=env, capture_output=True, text=True, timeout=14400)
    if t.returncode != 0:
        sys.stderr.write(t.stdout[-2000:] + t.stderr[-2000:])
    return t.returncode


def frames_in(d):
    return sorted(f for f in os.listdir(d) if f.endswith(".png"))


def diff_frame(a, b):
    """Return (max_channel_delta, differing_pixel_count) between two PNGs."""
    ia = Image.open(a).convert("RGB")
    ib = Image.open(b).convert("RGB")
    if ia.size != ib.size:
        return (255, ia.size[0] * ia.size[1])
    d = ImageChops.difference(ia, ib)
    bbox = d.getbbox()
    if bbox is None:
        return (0, 0)
    extrema = d.getextrema()               # per-channel (min,max)
    maxd = max(hi for _lo, hi in extrema)
    # count differing pixels
    gray = d.convert("L")
    diffpx = sum(1 for p in gray.getdata() if p != 0)
    return (maxd, diffpx)


def run_livetest(exe, tour, size, cfg_dir, quick):
    env = dict(os.environ)
    env["FRACTADYNE_NO_SOUND"] = "1"
    env["FRACTADYNE_CONFIG_DIR"] = cfg_dir
    args = [exe, "--livetest", tour]
    if size:
        args += ["--size", size]
    if quick:
        args += ["--quick"]
    t = subprocess.run(args, env=env, capture_output=True, text=True, timeout=14400)
    tail = (t.stdout + t.stderr).strip().splitlines()
    verdict = next((l for l in reversed(tail) if any(k in l.lower()
                    for k in ("pass", "fail", "drift", "checkpoint", "max"))), tail[-1] if tail else "")
    return t.returncode, verdict


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("tour")
    ap.add_argument("--exe", default=os.path.join(REPO, "target", "release", "fractadyne.exe"))
    ap.add_argument("--size", default="1920x1080")
    ap.add_argument("--out", default=None, help="work dir (default: a temp dir)")
    ap.add_argument("--quick", action="store_true", help="skip the live pass")
    a = ap.parse_args()

    work = a.out or tempfile.mkdtemp(prefix="cutoff-validate-")
    name = os.path.splitext(os.path.basename(a.tour))[0]
    cold_dir = os.path.join(work, name, "cold")
    warm_dir = os.path.join(work, name, "warm")
    cfg = os.path.join(work, name, "cfg")
    os.makedirs(cfg, exist_ok=True)

    print(f"[{name}] COLD (reuse off) -> {cold_dir}")
    if render_tour(a.exe, a.tour, cold_dir, a.size, reuse=False, cfg_dir=cfg + "-cold") != 0:
        print(f"[{name}] FAIL: cold render exited non-zero"); return 2

    print(f"[{name}] WARM (reuse on)  -> {warm_dir}")
    if render_tour(a.exe, a.tour, warm_dir, a.size, reuse=True, cfg_dir=cfg + "-warm") != 0:
        print(f"[{name}] FAIL: warm render exited non-zero"); return 2

    cold = frames_in(cold_dir)
    warm = frames_in(warm_dir)
    n = min(len(cold), len(warm))
    if n == 0:
        print(f"[{name}] FAIL: no frames rendered"); return 2

    worst = (0, 0)
    bad = []
    for i in range(n):
        maxd, diffpx = diff_frame(os.path.join(cold_dir, cold[i]), os.path.join(warm_dir, warm[i]))
        if maxd > worst[0]:
            worst = (maxd, i)
        if maxd != 0:
            bad.append((i, maxd, diffpx))
    if len(cold) != len(warm):
        print(f"[{name}] WARN: frame count differs cold={len(cold)} warm={len(warm)} (comparing {n})")

    if bad:
        print(f"[{name}] COLD != WARM on {len(bad)}/{n} frames "
              f"(worst frame {worst[1]}: max channel delta {worst[0]}). "
              f"Reuse changed the picture.")
        for i, maxd, dpx in bad[:8]:
            print(f"    frame {i}: max delta {maxd}, {dpx} px differ")
        cw_ok = False
    else:
        print(f"[{name}] COLD == WARM on all {n} frames (reuse is bit-identical). OK")
        cw_ok = True

    live_ok = True
    if not a.quick:
        rc, verdict = run_livetest(a.exe, a.tour, a.size, cfg + "-live", quick=False)
        live_ok = (rc == 0)
        print(f"[{name}] LIVE: exit {rc} — {verdict}")

    return 0 if (cw_ok and live_ok) else 1


if __name__ == "__main__":
    sys.exit(main())
