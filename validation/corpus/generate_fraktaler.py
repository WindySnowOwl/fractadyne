#!/usr/bin/env python3
"""Generate the Kalles Fraktaler / Fraktaler-3 side of the comparison corpus.

For each location in locations.toml, writes an F3 batch param file, runs the vendored
Fraktaler-3 in batch mode, and copies the resulting PNG to renders/<slug>-fraktaler.png
(so it pairs with <slug>-fractadyne.png in catalog.html).

Convention: F3 zoom = 1 shows a vertical extent of 4; Fractadyne mag = 1 shows 3. To frame
the SAME view, f3_zoom = our_mag * 4/3 (see validation/crosscheck-fraktaler3.md).

KNOWN LIMITATION on this machine: Fraktaler-3 renders correctly through the double-precision
regime (~1e13x) but its extended-exponent number types (softfloat / floatexp / doubleexp),
engaged past that, render blank (all-interior) here — confirmed on GPU and forced-CPU, in both
F3 3.0 and 3.1, on the 3080 alone, AND after a full NVIDIA driver update (still blank). It is
F3's OpenCL extended-type kernels vs this GPU arch, not a driver-version issue. So the double-regime locations produce real images; deeper ones are detected as
blank and skipped, leaving the catalog's placeholder. The .kfr / param files are written for
every location so the deep renders can be produced once F3 works (driver update / other machine).

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
        "iterations = %d\nescape_radius = 256\n" % loc["iterations"] +
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
    # This machine's F3 renders blank past ~1e13x (extended-type limitation). `--max-log10 X`
    # skips RUNNING F3 beyond depth X (param files are still written) so we don't spend minutes per
    # confirmed-blank deep render; pass a large value on a working F3 install to render everything.
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
            print("  blank (%d B) - F3 extended-type limitation past ~1e13x; skipped" % size)
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
