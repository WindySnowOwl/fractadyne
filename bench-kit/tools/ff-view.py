#!/usr/bin/env python3
"""A bench-kit scene's view, in the form Fracturing Fog's batch command line takes.

    tools/ff-view.py SCENE.kfr --width W --height H

Prints one JSON object: {"x": ..., "y": ..., "zoom": ..., "zoom_log10": ...}.

THREE CONVERSIONS, each measured against the Fractadyne render of the same scene (2026-10-31,
Fracturing Fog 0.7.1):

  center  Past about 1e15 a single double cannot hold the center, so --x/--y take it as up to
          eight double limbs, "hi|lo|...", whose exact sum is the center (Fracturing Fog's own
          lossless form). The .kfr decimal is split exactly, with rational arithmetic, never via
          a float: the first limb is the nearest double, the next the nearest double to what is
          left, and so on. Eight limbs hold about 124 digits.
  y       Fracturing Fog's imaginary axis points DOWN the image. Given the .kfr's Im, scene 03
          came out flipped top-to-bottom at the right magnification. The set is symmetric about
          the real axis, so the conjugate center, -Im, drawn y-down is exactly the scene drawn
          y-up, as every other renderer here draws it.
  zoom    Fracturing Fog's pixel size is 3.5 / (max(width, height) * zoom): its zoom fixes the
          WIDTH of a landscape image at 3.5/zoom. The .kfr's is Kalles Fraktaler's, the HEIGHT
          4/zoom. So zoom_ff = zoom_kfr * 3.5 * (height / max(width, height)) / 4, 0.4921875 at
          3840x2160. Scene 03 then has 42.1% interior against Fractadyne's 42.7%.

"zoom" is null when it does not fit a double (past about 1e308); "zoom_log10" is always given,
so the lane can refuse a scene past Fracturing Fog's ceiling without parsing a number it cannot
hold.

Exit 0 on success, 1 if the scene file cannot be read, 2 on a usage error.
"""
import argparse
import json
import math
import sys
from fractions import Fraction

LIMBS = 8


def limbs(value):
    """value (a Fraction) as up to LIMBS doubles whose exact sum is closest to it."""
    out = []
    r = value
    while r != 0 and len(out) < LIMBS:
        f = float(r)
        out.append(repr(f))
        r -= Fraction(f)
    return "|".join(out or ["0.0"])


def log10_fraction(q):
    """log10 of a positive Fraction of any size, without converting it to a float."""
    return math.log10(q.numerator) - math.log10(q.denominator)


def main():
    # The deepest scenes carry centers of 10,000 to 30,000 digits, past the 4,300-digit cap Python
    # 3.11 put on converting a string to an integer, which Fraction does. Older Pythons have no cap.
    if hasattr(sys, "set_int_max_str_digits"):
        sys.set_int_max_str_digits(0)
    ap = argparse.ArgumentParser()
    ap.add_argument("kfr")
    ap.add_argument("--width", type=int, required=True)
    ap.add_argument("--height", type=int, required=True)
    args = ap.parse_args()
    if args.width < 1 or args.height < 1:
        sys.stderr.write("ff-view: --width and --height must be positive\n")
        return 2
    kv = {}
    try:
        with open(args.kfr, encoding="utf-8") as fh:
            for line in fh:
                if ":" in line:
                    k, v = line.split(":", 1)
                    kv[k.strip()] = v.strip()
        re_, im = Fraction(kv["Re"]), Fraction(kv["Im"])
        zoom_kfr = Fraction(kv["Zoom"].replace("E", "e"))
    except (OSError, KeyError, ValueError) as e:
        sys.stderr.write("ff-view: cannot read the view from %s: %s\n" % (args.kfr, e))
        return 1
    zoom = zoom_kfr * Fraction(35, 10) * Fraction(args.height, max(args.width, args.height)) / 4
    try:
        zoom_s = repr(float(zoom))
    except OverflowError:
        zoom_s = None
    print(json.dumps({"x": limbs(re_), "y": limbs(-im), "zoom": zoom_s,
                      "zoom_log10": round(log10_fraction(zoom), 4)}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
