#!/usr/bin/env python3
"""Compare a render-farm output with the same tour rendered on one machine, frame by frame and
per machine — the farm's cross-GPU measurement (design/remote-rendering.md section 9).

    python scripts/farm_compare.py FARM_OUT REFERENCE PREFIX

FARM_OUT is the farm's --out folder (frames plus farm/done.jsonl, which says which machine
rendered each frame); REFERENCE holds the single-machine render of the same tour with the same
settings, anchors and orbit cap. Frames are compared by DECODED pixels, never file bytes (a PNG
embeds a timestamp). Prints, per machine: frames, frames identical, differing pixels (total, worst
frame), and the largest channel difference. Exit 0 when every frame was compared, 1 when one is
missing from either side.
"""

import json
import os
import sys
from collections import defaultdict

import numpy as np
from PIL import Image


def main() -> int:
    if len(sys.argv) != 4:
        print(__doc__)
        return 2
    out, ref, prefix = sys.argv[1], sys.argv[2], sys.argv[3]
    machine = {}
    done = os.path.join(out, "farm", "done.jsonl")
    with open(done, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                d = json.loads(line)
                machine[d["index"]] = d["machine"]
    stats = defaultdict(lambda: {"frames": 0, "identical": 0, "px": 0, "worst": (0, None), "delta": 0})
    missing = 0
    total_px = None
    for i in sorted(machine):
        name = f"{prefix}_{i:05d}.png"
        a, b = os.path.join(out, name), os.path.join(ref, name)
        if not (os.path.exists(a) and os.path.exists(b)):
            print(f"frame {i}: missing from {'the farm' if not os.path.exists(a) else 'the reference'}")
            missing += 1
            continue
        pa = np.asarray(Image.open(a).convert("RGBA"), dtype=np.int16)
        pb = np.asarray(Image.open(b).convert("RGBA"), dtype=np.int16)
        if pa.shape != pb.shape:
            print(f"frame {i}: size {pa.shape} vs {pb.shape}")
            missing += 1
            continue
        total_px = pa.shape[0] * pa.shape[1]
        diff = np.abs(pa - pb).max(axis=2)
        n = int((diff > 0).sum())
        s = stats[machine[i]]
        s["frames"] += 1
        s["identical"] += n == 0
        s["px"] += n
        s["delta"] = max(s["delta"], int(diff.max()))
        if n > s["worst"][0]:
            s["worst"] = (n, i)
    print(f"Farm vs single-machine reference ({len(machine)} frames, {total_px or 0} px each)")
    print(f"{'machine':<28} {'frames':>6} {'identical':>9} {'differing px':>13} {'worst frame':>18} {'max delta':>9}")
    for m in sorted(stats):
        s = stats[m]
        worst = f"{s['worst'][0]} (#{s['worst'][1]})" if s["worst"][1] is not None else "-"
        print(f"{m:<28} {s['frames']:>6} {s['identical']:>9} {s['px']:>13} {worst:>18} {s['delta']:>9}")
    print("(the reference was rendered on THIS machine: a client on the same GPU should be identical; another GPU's differences are its floating-point arithmetic)")
    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main())
