#!/usr/bin/env python3
"""Write the kit's two validation scene sets from the views the validation scripts define.

    python bench-kit/tools/make-points.py

- `tip` (scenes-tip.csv, 71-76): rungs of the tip ladder (scripts/tip_ladder.py) - one view of the
  antenna tip c = -2 at 1e12x, 1e28x (where Fractadyne's arithmetic changes), 1e100x, 1e308x (the
  edge of a double's range), 1e1000x and 1e10000x. Each 4x of zoom about -2 adds exactly one
  iteration to every count, so one reference at any depth judges them all.
- `pi` (scenes-pi.csv, 81-86): pi points (scripts/pi_points.py) - frames beside the neck c = -3/4
  and the cusp c = 1/4 whose centre's escape count pi predicts: 3,143 to 3,141,593 iterations.
  These are 17x17 frames: run them with -Size 17x17 -InspectSize 17x17.

Every scene in all three formats (.kfr, .f3.toml, .fdn) in bench-kit/points/, with the same centre,
magnification and iteration cap; the kit's view is 4 / zoom high. Coordinates are written in
full: a short decimal is read at too low a precision by some readers (Fractadyne's own
`--center` was, until 2026-11-03).
"""
import math
import os
import sys
from decimal import Decimal, localcontext

HERE = os.path.dirname(os.path.abspath(__file__))
KIT = os.path.dirname(HERE)
REPO = os.path.dirname(KIT)
sys.path.insert(0, os.path.join(REPO, "scripts"))
import pi_points as PP  # noqa: E402
import tip_ladder as TL  # noqa: E402

OUT = os.path.join(KIT, "points")
TIP = [(71, 40, "1e12"), (72, 94, "1e28"), (73, 332, "1e100"), (74, 1024, "1e308"), (75, 3322, "1e1000"),
       (76, 33220, "1e10000")]
PI = [(81, "neck-1e-3"), (82, "neck-1e-5"), (83, "neck-1e-6"), (84, "cusp-1e-6"), (85, "cusp-1e-10"),
      (86, "cusp-1e-12")]
FDN_HEIGHT = 720  # the scene files' nominal view (the kit sets its own size)


def zoom_text(log2):
    """2^log2 as decimal text, 20 significant digits (any depth)."""
    with localcontext() as ctx:
        ctx.prec = 40
        e10 = Decimal(log2) * Decimal(2).log10()
        whole = int(e10)
        if e10 < 0 and Decimal(whole) != e10:
            whole -= 1
        mant = (Decimal(10) ** (e10 - whole))
        return "%se%d" % (format(mant.quantize(Decimal("1.0000000000000000000")), "f"), whole)


def write(slug, re, im, log2, iters, notes, size):
    z = zoom_text(log2)
    upp_log2 = 2.0 - log2 - math.log2(FDN_HEIGHT)
    upp = "%.17e" % (2.0 ** upp_log2) if upp_log2 > -1070 else "0.00000000000000000e0"
    with open(os.path.join(OUT, slug + ".kfr"), "w", encoding="ascii", newline="\n") as f:
        f.write("Re: %s\nIm: %s\nZoom: %s\nIterations: %d\n" % (re, im, z.replace("e", "E"), iters))
    with open(os.path.join(OUT, slug + ".f3.toml"), "w", encoding="ascii", newline="\n") as f:
        f.write('[location]\nreal = "%s"\nimag = "%s"\nzoom = "%s"\n[image]\nwidth = %d\nheight = %d\n'
                'subframes = 1\n[bailout]\niterations = %d\nmaximum_reference_iterations = %d\n'
                'maximum_perturb_iterations = %d\nmaximum_bla_steps = 8192\nescape_radius = 256\n'
                '[transform]\nexponential_map = false\n[render]\nfilename = "%s-f3"\nsave_png = true\n'
                % (re, im, z, size[0], size[1], iters, iters, iters, slug))
    with open(os.path.join(OUT, slug + ".fdn"), "w", encoding="ascii", newline="\n") as f:
        f.write("app=Fractadyne\nformat_version=1\nnotes=%s\nfractal=Mandelbrot\njulia=0\n"
                "julia_c_re=-8.00000000000000044e-1\njulia_c_im=1.56000000000000000e-1\n"
                "center_re=%s\ncenter_im=%s\nupp=%s\nupp_log2=%.17e\nzoom=%s\nmax_iter=%d\nauto_iter=0\n"
                "palette=0\ncycle=0.27\noffset=0.1\naa=2\n" % (notes, re, im, upp, upp_log2, z, iters))


def main():
    os.makedirs(OUT, exist_ok=True)
    rows = {"tip": [], "pi": []}
    for sid, level, name in TIP:
        slug = "%d-tip-%s" % (sid, name)
        re, im = TL.centre(level)
        write(slug, re, im, level, TL.cap(level),
              "Validation scene %d: the antenna tip c=-2 at 2^%d (tip ladder: +%d iterations on the 1e12x rung)"
              % (sid, level, (level - TL.BASE) // 2), (1280, 720))
        rows["tip"].append("%02d,%s,%.12f,%d,0," % (sid, slug, level * math.log10(2), TL.cap(level)))
    pts = {p["id"]: p for p in PP.points(17)}
    for sid, pid in PI:
        p = pts[pid]
        slug = "%d-pi-%s" % (sid, pid)
        write(slug, p["re"], p["im"], p["zoom_log2"], p["cap"],
              "Validation scene %d: pi point %s (the centre's count is %.1f by pi's law; a 17x17 frame)"
              % (sid, pid, p["law"]), (17, 17))
        # f3_limit = the cap: beside a parabolic point BLA skips almost nothing, so a pixel needs its
        # whole count in perturbation steps, and at the kit's 8192 Fraktaler-3 stops it and draws it
        # as interior (2026-11-03: 286 of 289 pixels at the neck 1e-5 and 1e-6).
        rows["pi"].append("%02d,%s,%.12f,%d,0,%d" % (sid, slug, p["zoom_log2"] * math.log10(2), p["cap"], p["cap"]))
    for name, lines in rows.items():
        with open(os.path.join(KIT, "scenes-%s.csv" % name), "w", encoding="ascii", newline="\n") as f:
            f.write("id,slug,mag_log10,iterations,normalize,f3_limit\n" + "\n".join(lines) + "\n")
    print("wrote %d tip and %d pi scenes to %s" % (len(rows["tip"]), len(rows["pi"]), OUT))


if __name__ == "__main__":
    main()
