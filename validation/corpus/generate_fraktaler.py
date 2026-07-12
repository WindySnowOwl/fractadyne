#!/usr/bin/env python3
"""Generate the Kalles Fraktaler / Fraktaler-3 side of the comparison corpus.

For each location in locations.toml, writes an F3 batch param file, runs the vendored
Fraktaler-3 in batch mode, and copies the resulting PNG to renders/<slug>-fraktaler.png
(so it pairs with <slug>-fractadyne.png in catalog.html).

Convention: F3 zoom = 1 shows a vertical extent of 4; Fractadyne mag = 1 shows 3. To frame
the SAME view, f3_zoom = our_mag * 4/3 (see validation/crosscheck-fraktaler3.md).

DEPTH NOTE: Fraktaler-3's batch mode defaults maximum_reference_iterations far too low for a
zoomed view, so without it F3 silently renders uniform/blank at ANY depth. For a long time that
looked like a hard "F3 blanks past ~1e13x" extended-type ceiling (softfloat/floatexp kernels vs
this GPU) — but every deep test shared the same missing setting, so they all blanked identically.
It was a config gap, not a kernel wall. write_param now sets maximum_reference_iterations /
maximum_perturb_iterations / maximum_bla_steps, and with those F3 renders deep correctly here —
real, arm-for-arm matches verified out to 4.6e1105x (location 10, ~4 min). Two more things must
hold: the iteration cap must be high enough for the depth (else the reference truncates to blank),
and the center must carry enough digits for the depth PLUS margin for F3's internal reference
rounding (coarser than Fractadyne's full bignum). The former location 07 (1e30x) gap illustrated the
latter: a 34-digit seahorse center was too coarse there, so F3 rounded onto different sub-structure.
It was replaced by a user F3 save (me30.exr) whose 43-digit center matches arm-for-arm. (F3 renders
all twenty cleanly; the two that are NOT clean cross-app matches are 14 and 15, where Fractadyne's
side is glitch-limited — its multi-reference glitch correction is disabled there because it goes
pathologically slow at the deep-interior dark cores, so its interior renders as uncorrected-glitch
speckle. See validation/corpus/README.md and TODO.md.)

Run from repo root:  python validation/corpus/generate_fraktaler.py
"""

import math
import os
import re
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CORPUS = os.path.join(ROOT, "validation", "corpus")
F3 = os.path.join(ROOT, "diag", "fraktaler", "fraktaler-3-3.1.x86_64.exe")
PARAMS = os.path.join(CORPUS, "locations")  # .f3.toml go beside the .kfr files
WIDTH, HEIGHT, SUBFRAMES = 1280, 720, 4
BLANK_BYTES = 15000  # a 1280x720 PNG below this at these depths is F3's all-interior blank


def read_locations():
    text = open(os.path.join(CORPUS, "locations.toml"), encoding="utf-8").read()
    locs = []
    for block in text.split("[[location]]")[1:]:
        loc = {}
        for key in ("slug", "center_x", "center_y"):
            loc[key] = re.search(r'%s = "(.*?)"' % key, block).group(1)
        loc["mag_log10"] = float(re.search(r"mag_log10 = ([\d.eE+-]+)", block).group(1))
        loc["iterations"] = int(re.search(r"iterations = (\d+)", block).group(1))
        locs.append(loc)
    return locs


def f3_zoom(mag_log10):
    """F3 zoom string from log10(our magnification): f3_zoom = 10^mag_log10 * 4/3."""
    l = mag_log10 + math.log10(4.0 / 3.0)
    exp = int(l // 1)
    mant = 10 ** (l - exp)
    return "%.6fe%d" % (mant, exp) if exp >= 4 else "%.6g" % (10 ** l)


def write_param(loc):
    path = os.path.join(PARAMS, loc["slug"] + ".f3.toml")
    body = (
        "[location]\n"
        'real = "%s"\n' % loc["center_x"] +
        'imag = "%s"\n' % loc["center_y"] +
        'zoom = "%s"\n' % f3_zoom(loc["mag_log10"]) +
        "[image]\n"
        "width = %d\nheight = %d\nsubframes = %d\n" % (WIDTH, HEIGHT, SUBFRAMES) +
        "[bailout]\n"
        "iterations = %d\n" % loc["iterations"] +
        # F3 batch's default maximum_reference_iterations is far too low for zoomed views — without
        # these it silently renders uniform/blank at ANY depth (this, not an extended-type kernel
        # ceiling, is why the deep corpus renders were blank before). Set them to the iteration
        # budget; with a depth-appropriate cap F3 renders the whole practical range (through 6.6e43x
        # here). The cap must scale with depth: 1e30x needs >=30k or the reference truncates to blank.
        "maximum_reference_iterations = %d\n" % loc["iterations"] +
        "maximum_perturb_iterations = %d\n" % loc["iterations"] +
        "maximum_bla_steps = 8192\n"
        "escape_radius = 256\n" +
        "[transform]\nexponential_map = false\n"
        "[render]\n"
        # Relative filename (resolved against cwd=PARAMS in main): an absolute Windows path here
        # would put backslash escapes into the TOML string and F3 silently writes nothing.
        'filename = "%s"\nsave_png = true\n' % (loc["slug"] + "-f3")
    )
    open(path, "w", newline="\n").write(body)
    return path


def main():
    if not os.path.exists(F3):
        sys.exit("Fraktaler-3 binary not found: %s" % F3)
    # `--max-log10 X` skips RUNNING F3 beyond depth X (param files are still written) — handy for
    # regenerating params without re-running the slow deep renders (09 ~9 min, 10 ~4 min).
    # Default: render everything.
    max_log10 = float("inf")
    if "--max-log10" in sys.argv:
        max_log10 = float(sys.argv[sys.argv.index("--max-log10") + 1])
    locs = read_locations()
    kept, blank = [], []
    for loc in locs:
        param = write_param(loc)
        raw = os.path.join(PARAMS, loc["slug"] + "-f3.png")
        if os.path.exists(raw):
            os.remove(raw)
        if loc["mag_log10"] > max_log10:
            print("skipping %s (deeper than --max-log10; param written)" % loc["slug"])
            blank.append(loc["slug"])
            continue
        print("rendering %s (f3_zoom %s) ..." % (loc["slug"], f3_zoom(loc["mag_log10"])), flush=True)
        try:
            # cwd=PARAMS so the relative `filename` in the param resolves beside the param file.
            subprocess.run([F3, "-b", os.path.basename(param)], capture_output=True, text=True,
                           cwd=PARAMS, timeout=1800)
        except subprocess.TimeoutExpired:
            print("  timed out")
        if not os.path.exists(raw):
            print("  no output")
            blank.append(loc["slug"])
            continue
        size = os.path.getsize(raw)
        dest = os.path.join(CORPUS, "renders", loc["slug"] + "-fraktaler.png")
        if size < BLANK_BYTES:
            print("  blank (%d B) - reference truncated (raise iterations) or beyond practical range; skipped" % size)
            os.remove(raw)
            blank.append(loc["slug"])
        else:
            shutil.move(raw, dest)
            print("  -> %s (%.2f MB)" % (os.path.basename(dest), size / 1e6))
            kept.append(loc["slug"])
    print("\n%d Fraktaler renders written, %d blank/skipped." % (len(kept), len(blank)))
    if blank:
        print("blank (render on a working F3 install): " + ", ".join(blank))


if __name__ == "__main__":
    main()
