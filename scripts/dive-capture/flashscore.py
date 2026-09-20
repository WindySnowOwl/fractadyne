"""Score the FLASHING of a captured dive: how often the canvas changes abruptly frame to frame.

`blankscore.py` counts frames that are one flat colour; this counts the transitions the eye
reads as a flash — a frame whose canvas interior differs from the previous capture by more than
FLASH_DIFF mean absolute units per channel (a crisp frame giving way to a flat one, or back, is
80–130; a smooth dive step at 100 ms intervals is under 30). Run on the `frames/` of a
`capdive.sh` run, or on frames extracted from a screen recording (`ffmpeg -i rec.mp4 f%04d.png`)
with `--rect X0 X1 Y0 Y1` naming the canvas interior in that recording.

    python scripts/dive-capture/flashscore.py RUN…  [--rect 330 1240 200 790]
"""
import argparse, glob, os, sys
import numpy as np
from PIL import Image

FLASH_DIFF = 60.0

ap = argparse.ArgumentParser()
ap.add_argument("runs", nargs="+")
ap.add_argument("--rect", nargs=4, type=int, metavar=("X0", "X1", "Y0", "Y1"),
                help="canvas interior in frame pixels (default: the middle 28–72%% × 25–85%%)")
args = ap.parse_args()

for run in args.runs:
    fs = sorted(glob.glob(os.path.join(run, "frames", "*.jpg")) + glob.glob(os.path.join(run, "frames", "*.png")))
    if not fs:
        fs = sorted(glob.glob(os.path.join(run, "*.jpg")) + glob.glob(os.path.join(run, "*.png")))
    if not fs:
        print(f"{run:12s}: no frames")
        continue
    W, H = Image.open(fs[0]).size
    if args.rect:
        x0, x1, y0, y1 = args.rect
    else:
        x0, x1, y0, y1 = int(W * 0.28), int(W * 0.72), int(H * 0.25), int(H * 0.85)
    prev = None
    diffs, flats = [], []
    for f in fs:
        im = np.asarray(Image.open(f).convert("RGB"), dtype=np.int16)[y0:y1, x0:x1]
        med = np.median(im.reshape(-1, 3), axis=0)
        flats.append(float((np.abs(im - med).sum(axis=2) <= 8).mean()) > 0.97)
        diffs.append(float(np.abs(im - prev).mean()) if prev is not None else 0.0)
        prev = im
    flashes = sum(1 for d in diffs if d > FLASH_DIFF)
    n = len(fs)
    print(f"{run:12s} frames {n:3d} | FLASHES {flashes:3d} ({100 * flashes / n:3.0f}% of transitions) | "
          f"flat {sum(flats):3d} ({100 * sum(flats) / n:3.0f}%) | diff mean {np.mean(diffs):5.1f} p95 {np.percentile(diffs, 95):5.1f} max {max(diffs):5.1f}")
    print("     " + "".join("!" if d > FLASH_DIFF else ("F" if fl else ".") for d, fl in zip(diffs, flats)))
