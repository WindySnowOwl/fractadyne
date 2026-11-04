#!/usr/bin/env python3
"""Score a kit run of the validation points against reference-grade truth.

    python bench-kit/tools/points-score.py RESULTS_DIR --exe FRACTADYNE.EXE

For a run of `-SceneSet tip` or `-SceneSet pi` with `-Inspect` (see tools/make-points.py). Every
renderer whose raw counts the inspection pass collected - Fractadyne (fd-raw), Fraktaler-3
(f3-raw), Imagina (im-inspect) and Fractal Zoomer (fz-inspect/out) - is judged by
scripts/truth_check.py against a truth computed here (`fractadyne --truth`, cached in
RESULTS_DIR/truth):
- a TIP scene against the truth of the 1e12x rung, shifted by the number of 4x steps between them:
  each 4x of zoom about -2 adds exactly one iteration at radius 256 and 4096 (not at 2, so a
  renderer that counts from radius 2 is judged on radius 256/4096 planes only);
- a PI scene against its own truth (`--truth-direct`), and the truth's centre against pi's law.
Writes RESULTS_DIR/points-score.json and points-score.md, and a map per renderer and scene in
RESULTS_DIR/points-maps (grey agrees, magenta a deviating patch, red escaped in one only).
"""
import argparse
import json
import math
import os
import subprocess
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
import minibrot_offsets as MO  # noqa: E402  (fresh_env)
import tip_ladder as TL  # noqa: E402
import truth_check as TC  # noqa: E402

SOURCES = [  # renderer, kind, path in the results folder
    ("fractadyne", "fd", "fd-raw/{s}.exr"),
    ("fraktaler3", "f3", "f3-raw/{s}.exr"),
    ("imagina", "imagina", "im-inspect/{s}.ppm"),
    ("fractalzoomer", "fz", "fz-inspect/out/{s}.png"),
]


def read_kfr(path):
    out = {}
    for line in open(path, encoding="ascii"):
        k, _, v = line.partition(":")
        out[k.strip()] = v.strip()
    return out


def truth(exe, out_dir, name, args, extra):
    stem = os.path.join(out_dir, name)
    if os.path.exists(stem + ".truth.json"):
        return stem
    cmd = [exe, "--truth", out_dir] + args + ["--truth-name", name, "--truth-stencil"] + extra
    p = subprocess.run(cmd, env=MO.fresh_env(os.path.join(out_dir, "cfg")), capture_output=True, text=True,
                       timeout=14400)
    if p.returncode != 0:
        raise SystemExit("points-score: --truth %s failed (exit %d):\n%s" % (name, p.returncode, (p.stdout + p.stderr)[-1200:]))
    return stem


def main():
    ap = argparse.ArgumentParser(description="score a kit run of the validation points against the truth")
    ap.add_argument("results")
    ap.add_argument("--exe", required=True, help="a fractadyne.exe with --truth (and --truth-direct)")
    a = ap.parse_args()
    res = os.path.abspath(a.results)
    man = json.load(open(os.path.join(res, "run-manifest.json"), encoding="utf-8"))["meta"]
    kit = os.path.dirname(HERE)
    tdir = os.path.join(res, "truth")
    mdir = os.path.join(res, "points-maps")
    os.makedirs(tdir, exist_ok=True)
    os.makedirs(mdir, exist_ok=True)
    rows = []
    for s in man["scenes"]:
        fd_raw = os.path.join(res, "fd-raw", s + ".exr")
        if not os.path.exists(fd_raw):
            print("%s: no Fractadyne raw counts (run with -Inspect)" % s)
            continue
        h, w = TC.INSP.read_exr(fd_raw)["R"].shape
        size = ["--size", "%dx%d" % (w, h)]
        mag_log10 = float(man["scene_mag"][s])
        zl2 = mag_log10 * math.log2(10)
        kfr = read_kfr(os.path.join(kit, "scenes", s + ".kfr"))
        law = None
        if "-tip-" in s:
            level = int(round(zl2))
            re, im = TL.centre(TL.BASE)
            stem = truth(a.exe, tdir, "tip-base-%dx%d" % (w, h),
                         ["--center", re, im, "--zoom-log2", str(TL.BASE), "--iter", str(TL.cap(TL.BASE))] + size,
                         ["--truth-check", "200"])
            shift = (level - TL.BASE) // 2
        elif "-pi-" in s:
            stem = truth(a.exe, tdir, s, ["--center", kfr["Re"], kfr["Im"], "--zoom-log2", repr(zl2),
                                          "--iter", kfr["Iterations"]] + size,
                         ["--truth-direct", "--truth-check", "1"])
            shift = 0
            kind, k = s.split("-pi-")[1].split("-1e-")  # e.g. "neck", "6"
            eps = 10.0 ** -int(k)
            law = math.pi / eps if kind == "neck" else math.pi / math.sqrt(eps)
        else:
            print("%s: not a validation point (tip or pi); skipped" % s)
            continue
        t = TC.load_truth(stem, shift)
        cy, cx = h // 2, w // 2
        for renderer, kind, rel in SOURCES:
            path = os.path.join(res, rel.format(s=s))
            row = {"scene": s, "renderer": renderer, "shift": shift}
            if not os.path.exists(path):
                row["verdict"] = "no counts"
                rows.append(row)
                continue
            try:
                sc = TC.score(stem, path, kind, os.path.join(mdir, "%s-%s.png" % (renderer, s)), shift=shift)
            except Exception as e:  # a render at the wrong size, an unreadable file
                row["verdict"] = "unreadable: %s" % e
                rows.append(row)
                continue
            row.update({k: sc[k] for k in ("verdict", "follows", "offset", "judged", "deviating", "deviating_patches",
                                           "escaped_in_one", "escaped_in_one_patches", "interior_truth",
                                           "interior_render", "median_abs_diff_stable")})
            if law is not None:
                v, _ = TC.read_render(path, kind, t)
                tv = t[sc["follows"]][cy, cx]
                row["centre"] = float(v[cy, cx])
                row["centre_truth"] = int(tv)
                row["centre_vs_truth"] = (float(np.floor(v[cy, cx])) - sc["offset"] - tv) if v[cy, cx] >= 0 else None
                row["law"] = round(law, 1)
                row["truth_n2_centre"] = int(t["n2"][cy, cx])
            rows.append(row)
        print("%s: scored (%s, shift %+d)" % (s, os.path.basename(stem), shift), flush=True)
    with open(os.path.join(res, "points-score.json"), "w", encoding="utf-8", newline="\n") as f:
        json.dump(rows, f, indent=1)
    cols = ["scene", "renderer", "verdict", "follows", "offset", "deviating", "deviating_patches", "escaped_in_one",
            "escaped_in_one_patches", "interior_truth", "interior_render", "centre_vs_truth"]
    with open(os.path.join(res, "points-score.md"), "w", encoding="utf-8", newline="\n") as f:
        f.write("| " + " | ".join(cols) + " |\n|" + "---|" * len(cols) + "\n")
        for r in rows:
            f.write("| " + " | ".join(str(r.get(c, "")) for c in cols) + " |\n")
    print(open(os.path.join(res, "points-score.md"), encoding="utf-8").read())
    return 0


if __name__ == "__main__":
    sys.exit(main())
