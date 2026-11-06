#!/usr/bin/env python3
"""The share of pixels that never escaped, read from a renderer's own raw EXR output.

    tools/exr-interior.py RAW_DIR --json OUT.json

WHY THIS EXISTS
---------------
verify-views.py judges magnification by the interior fraction: the share of pixels that never
escaped. It reads that off the image as the share of BLACK pixels, which assumes a renderer
draws only never-escaped pixels black. Fraktaler-3's default coloring does not. It shades by
distance estimate, so every pixel close to the boundary is black too, escaped or not. On scene 59
(2.1e68x, 28.5 million iterations) its image is 62% black at 4K, while only 6.6% of its pixels
never escaped (measured at 640x360 from this output, 2026-10-31); the other 91% of its black
pixels escaped after 4 to 10 million iterations. Read off the image, that looks like a
different magnification, and the check called a correct render the wrong view.

Whether a pixel escaped is not a coloring decision, so this reads it from the renderer itself.
Fraktaler-3 writes raw iteration counts to EXR when asked (`save_exr = true`,
`exr_channels = ["N0", "N1"]`, one subframe), and marks a pixel that never escaped with the
all-ones count 0xFFFFFFFF. The Fraktaler-3 lane renders each scene once more for this, untimed,
at the view check's own 640x360.

Every `*.exr` in RAW_DIR is read; the scene is the file name without `.exr`. Writes
{scene: share} to OUT.json, for verify-views.py --interior-json.

Exit 0 if every file was read, 1 if any could not be, 2 on a usage error. Needs numpy.
"""
import argparse
import glob
import json
import os
import struct
import sys
import zlib

try:
    import numpy as np
except ImportError:
    sys.stderr.write("exr-interior: needs numpy (pip install numpy)\n")
    sys.exit(2)

NEVER_ESCAPED = 0xFFFFFFFF
# The OpenEXR compressions this reads. Fraktaler-3 writes ZIP; the others are here because they
# cost nothing. Anything else is refused by name rather than misread.
COMPRESSIONS = {0: ("none", 1), 2: ("zips", 1), 3: ("zip", 16)}
PIXEL_TYPES = {0: ("<u4", 4), 1: ("<f2", 2), 2: ("<f4", 4)}


def read_exr(path):
    """{channel: 2-D array} from a scanline OpenEXR file."""
    with open(path, "rb") as fh:
        b = fh.read()
    if len(b) < 8 or struct.unpack("<I", b[:4])[0] != 20000630:
        raise ValueError("not an OpenEXR file")
    if struct.unpack("<I", b[4:8])[0] & 0x200:
        raise ValueError("tiled EXR is not supported")
    p = 8

    def cstr(p):
        e = b.index(b"\0", p)
        return b[p:e].decode("latin1"), e + 1

    hdr = {}
    while True:
        name, p = cstr(p)
        if not name:
            break
        _typ, p = cstr(p)
        n = struct.unpack("<I", b[p:p + 4])[0]
        p += 4
        hdr[name] = b[p:p + n]
        p += n
    v = hdr["channels"]
    q = 0
    chans = []
    while v[q] != 0:
        e = v.index(b"\0", q)
        cn = v[q:e].decode("latin1")
        q = e + 1
        pt, _pl, _xs, ys = struct.unpack("<iiii", v[q:q + 16])
        q += 16
        if ys != 1:
            raise ValueError("subsampled channel %s is not supported" % cn)
        chans.append((cn,) + PIXEL_TYPES[pt])
    comp = hdr["compression"][0]
    if comp not in COMPRESSIONS:
        raise ValueError("compression %d is not supported (none, zips and zip are)" % comp)
    lines = COMPRESSIONS[comp][1]
    x0, y0, x1, y1 = struct.unpack("<4i", hdr["dataWindow"])
    W, H = x1 - x0 + 1, y1 - y0 + 1
    nchunks = (H + lines - 1) // lines
    offsets = struct.unpack("<%dQ" % nchunks, b[p:p + 8 * nchunks])
    out = {c: np.zeros((H, W), dtype=dt) for c, dt, _s in chans}
    stride = W * sum(s for _c, _d, s in chans)
    for off in offsets:
        y, size = struct.unpack("<ii", b[off:off + 8])
        raw = b[off + 8:off + 8 + size]
        nl = min(lines, y1 - y + 1)
        if comp != 0 and size < nl * stride:
            # ZIP: zlib, then a byte predictor, then the two halves interleaved. A chunk that
            # would not shrink is stored as it is, which the size test above catches.
            t = np.frombuffer(zlib.decompress(raw), dtype=np.uint8).astype(np.int64)
            t = ((np.cumsum(t - 128) + 128) & 0xFF).astype(np.uint8)
            half = (len(t) + 1) // 2
            data = np.empty(len(t), dtype=np.uint8)
            data[0::2] = t[:half]
            data[1::2] = t[half:]
        else:
            data = np.frombuffer(raw, dtype=np.uint8)
        q = 0
        for row in range(nl):
            for c, dt, s in chans:
                out[c][y - y0 + row] = np.frombuffer(data[q:q + W * s].tobytes(), dtype=dt)
                q += W * s
    return out


def never_escaped(ch):
    """Boolean mask of never-escaped pixels. A count that fits 32 bits arrives as one channel N;
    a wider one as N0 (low word) and N1 (high word), all ones in both for a pixel that never
    escaped."""
    if "N" in ch:
        return ch["N"] == NEVER_ESCAPED
    if "N0" in ch and "N1" in ch:
        return (ch["N0"] == NEVER_ESCAPED) & (ch["N1"] == NEVER_ESCAPED)
    raise ValueError("no iteration-count channel (N, or N0 and N1); channels are %s"
                     % ", ".join(sorted(ch)))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("raw_dir")
    ap.add_argument("--json", metavar="PATH", required=True,
                    help="write {scene: never-escaped share} here, for verify-views.py --interior-json")
    args = ap.parse_args()
    if not os.path.isdir(args.raw_dir):
        sys.stderr.write("exr-interior: no such directory: %s\n" % args.raw_dir)
        return 2
    shares = {}
    bad = 0
    for path in sorted(glob.glob(os.path.join(args.raw_dir, "*.exr"))):
        scene = os.path.basename(path)[:-4]
        try:
            m = never_escaped(read_exr(path))
        except Exception as e:
            print("%-28s could not be read: %s" % (scene, e))
            bad += 1
            continue
        shares[scene] = round(float(m.mean()), 4)
        print("%-28s %8.4f never escaped  (%dx%d)" % (scene, shares[scene], m.shape[1], m.shape[0]))
    if not shares and not bad:
        print("exr-interior: no .exr files in %s" % args.raw_dir)
    with open(args.json, "w") as fh:
        json.dump(shares, fh, indent=1, sort_keys=True)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
