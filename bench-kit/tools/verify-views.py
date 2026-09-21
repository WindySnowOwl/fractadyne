#!/usr/bin/env python3
"""Did the other renderer render the SAME VIEW, or just A view?

    tools/verify-views.py RESULTS_DIR [--prefix fs] [--quiet]

WHY THIS EXISTS
---------------
The structure guard in the bench lane asks one question: "is this a picture, or a flat field?"
It has never claimed to ask "is this the RIGHT picture", and on 2026-09-21 that gap cost us a
whole benchmark run.

FractalSharkCli accepts `--zoom 1e6.1249387366083` and silently TRUNCATES the fractional
exponent: that render is byte-identical to `--zoom 1e6`. Every scene in the run came out
under-zoomed by up to 10x. Every one exited 0. Every one was a real, structured, plausible
picture of the wrong place. And because an under-zoomed frame is a CHEAPER frame, the error did
not look like an error -- it looked like FractalShark being 20-50x faster than everything else,
which is exactly the shape a result takes when it is wrong in your favour.

A time is only meaningful if it is the time for the work you asked for. This checks that.

HOW
---
Palettes differ between renderers, so pixels cannot be compared and a plain grayscale
correlation is defeated by a palette inversion alone (it scores a perfect structural match as
-0.26). Two palette-independent signals are used instead:

  interior fraction  -- the share of pixels that never escaped. Black in every renderer's
                        palette, and strongly monotonic in magnification, so it catches a wrong
                        zoom directly. This is the signal that settled the 0.543 case.
  edge correlation   -- correlation of gradient MAGNITUDE, which survives inversion and most
                        remapping, and catches a wrong centre at the same zoom.

THE CONTROL IS NOT OPTIONAL. Each render is also scored against every OTHER scene's reference.
A metric that cannot tell the right scene from the wrong one is measuring nothing, so a match
must beat the best wrong scene, not merely score highly. That is reported on every line.

Exit 0 if every pair agrees, 1 if any disagrees, 2 on a usage error.
"""
import os
import sys
import glob
import argparse

try:
    import numpy as np
    from PIL import Image
except ImportError:
    sys.stderr.write("verify-views: needs numpy and Pillow (pip install pillow numpy)\n")
    sys.exit(2)

# Downsample before measuring: this compares FRAMING, not resampling detail, and a 4K pair costs
# nothing to load at this size.
RES = (640, 360)
# Interior is "did not escape" = black in every palette these renderers ship. The threshold is on
# the RGB sum, so it stays black-ish rather than exactly zero after a Lanczos resize.
INTERIOR_SUM = 40
# Tolerances. Interior fraction moves fast with magnification -- scene 03 goes 0.427 -> 0.251 for
# a single truncated mantissa -- so 0.06 is loose enough for palette and antialiasing differences
# and far tighter than any real zoom error.
INTERIOR_TOL = 0.06
EDGE_MIN = 0.35
EDGE_MARGIN = 0.10
# A DECISIVE edge match settles it on its own, and the interior test is not allowed to veto it.
# Scene 17 is why: FractalShark draws the same dendrite on a near-black background, so 16.7% of
# its frame reads as "interior" where ours reads 0%, and the interior test called a correct render
# wrong. Edge correlation there was 0.779 against 0.028 for the best wrong scene - not a close
# call. Interior fraction earns its place on the scenes where edge correlation is ambiguous, which
# is exactly where a wrong zoom lands; in the truncated-exponent run no scene reached even 0.28.
EDGE_DECISIVE = 0.50
EDGE_DECISIVE_MARGIN = 0.15


def load(path):
    a = Image.open(path).convert("RGB").resize(RES, Image.LANCZOS)
    return np.asarray(a, dtype=np.float64)


def interior_fraction(rgb):
    return float((rgb.sum(axis=2) < INTERIOR_SUM).mean())


def edge_feature(rgb):
    """Gradient magnitude of luminance, standardised. Invariant to palette inversion."""
    lum = rgb.mean(axis=2)
    gy, gx = np.gradient(lum)
    e = np.hypot(gx, gy).ravel()
    e -= e.mean()
    n = np.linalg.norm(e)
    return e / n if n else e


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("results_dir")
    ap.add_argument("--prefix", default="fs",
                    help="filename prefix of the renderer under test (default: fs)")
    ap.add_argument("--quiet", action="store_true")
    ap.add_argument("--json", metavar="PATH",
                    help="also write per-scene verdicts here, for the lane to consume")
    args = ap.parse_args()

    d = args.results_dir
    if not os.path.isdir(d):
        sys.stderr.write("verify-views: no such directory: %s\n" % d)
        return 2

    # Fractadyne is the reference because it is the lane cross-validated pixel-for-pixel against
    # Fraktaler-3; it is the only image here already known to be the right place.
    refs = {}
    for p in sorted(glob.glob(os.path.join(d, "fd-*.png"))):
        refs[os.path.basename(p)[3:-4]] = p
    if not refs:
        sys.stderr.write("verify-views: no fd-*.png reference renders in %s\n" % d)
        return 2

    # The lane spells dots as 'p' in its output names, and may carry a -rN rep suffix.
    tests = {}
    for slug in refs:
        for cand in (
            os.path.join(d, "%s-%s-r1.png" % (args.prefix, slug.replace(".", "p"))),
            os.path.join(d, "%s-%s.png" % (args.prefix, slug.replace(".", "p"))),
            os.path.join(d, "%s-%s.png" % (args.prefix, slug)),
        ):
            if os.path.exists(cand):
                tests[slug] = cand
                break

    if not tests:
        print("verify-views: no %s-*.png renders to check (lane skipped?)" % args.prefix)
        return 0

    R = {s: load(p) for s, p in refs.items()}
    Rint = {s: interior_fraction(v) for s, v in R.items()}
    Redge = {s: edge_feature(v) for s, v in R.items()}

    print("%-28s %9s %9s %7s %8s %9s  %s"
          % ("scene", "ref int", "test int", "d_int", "edge r", "best wrong", "verdict"))
    print("-" * 96)

    failures = []
    verdicts = {}
    for slug in sorted(tests):
        t = load(tests[slug])
        ti, te = interior_fraction(t), edge_feature(t)
        dint = ti - Rint[slug]
        er = float(np.dot(Redge[slug], te))
        wrong_r, wrong_s = max(
            (float(np.dot(Redge[o], te)), o) for o in R if o != slug
        ) if len(R) > 1 else (0.0, "-")

        decisive = er >= EDGE_DECISIVE and er >= wrong_r + EDGE_DECISIVE_MARGIN
        ok = decisive or (abs(dint) <= INTERIOR_TOL
                          and er >= EDGE_MIN
                          and er >= wrong_r + EDGE_MARGIN)
        if not ok:
            failures.append((slug, dint, er, wrong_r, wrong_s))
        verdicts[slug] = {
            "same_view": bool(ok),
            "ref_interior": round(Rint[slug], 4),
            "test_interior": round(ti, 4),
            "d_interior": round(dint, 4),
            "edge_r": round(er, 3),
            "best_wrong_r": round(wrong_r, 3),
            "best_wrong_scene": wrong_s,
        }
        print("%-28s %9.4f %9.4f %+7.4f %8.3f %9.3f  %s"
              % (slug, Rint[slug], ti, dint, er, wrong_r,
                 "same scene" if ok else "** NOT THE SCENE **"))

    if args.json:
        import json
        with open(args.json, "w") as fh:
            json.dump(verdicts, fh, indent=1, sort_keys=True)

    print()
    if failures:
        print("%d of %d renders do NOT depict the scene they were asked for." % (len(failures), len(tests)))
        if not args.quiet:
            print()
            print("Two different faults land here, and the image tells them apart:")
            print("  WRONG VIEW    - a clean picture of somewhere else. Check the zoom and centre")
            print("                  actually reaching the renderer. A truncated mantissa")
            print("                  under-zooms, and an under-zoomed frame is CHEAPER, so this")
            print("                  one always arrives looking like good news.")
            print("  FAILED RENDER - the right view, not resolved: speckle or noise where the")
            print("                  structure belongs. Note that NOISE PASSES a structure guard")
            print("                  easily - many colours, no dominant one - so nothing else in")
            print("                  the lane will object to it.")
            print()
            print("Either way the time is not a slow or a fast result, it is a VOID one. Look at")
            print("the image before deciding which fault you have.")
        return 1

    print("All %d renders depict the same scene as the Fractadyne reference." % len(tests))
    return 0


if __name__ == "__main__":
    sys.exit(main())
