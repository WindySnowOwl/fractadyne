#!/usr/bin/env python3
"""Inspection sheets: every renderer on ONE stepped palette, and where it disagrees with Fractadyne.

    tools/inspect-sheet.py prepare RESULTS_DIR
    tools/inspect-sheet.py sheet RESULTS_DIR [--out FILE.jpg]

WHY
---
Each renderer draws with its own palette, so two correct renders of one scene can look nothing
alike, and a real difference (a glitch, a blank patch, a wrong count) hides among the colouring
differences. The view check (verify-views.py) can say "same scene" and no more. An inspection
sheet puts the renderers that allow it on one palette made for spotting differences, and says
WHERE a renderer that writes its raw counts disagrees with Fractadyne. (User request, 2026-11-03:
"a standard color map ... high contrast cycling colors ... something that would make it easier to
identify aberrations and deviations between renderers".) It is for looking at, not for
publishing: the published images keep each renderer's own look.

THE PALETTE
-----------
Four colours in a fixed order, light and dark alternating (white, blue, yellow, vermillion),
never blended; interior black. A pixel's colour is band(n) = floor(n / k) mod 4 for its whole
iteration count n. k, the band width in iterations, is a power of two chosen per scene from
Fractadyne's own counts, so bands come out about BAND_PX pixels wide at 640x360. Hard edges are
the point: an off-by-a-few count moves a band edge, which a smooth ramp (grayscale included)
would hide.

WHO GETS IT
-----------
- Fractadyne and Fraktaler-3 write raw iteration counts (fd-raw/, f3-raw/, both 640x360); they
  are coloured HERE, so their palette is identical by construction.
- FractalShark 0.545 takes --palette-map FILE --palette-depth D. Measured 2026-11-03 at bench
  scene 03: it resamples the file to 2^D entries and advances ONE ENTRY PER ITERATION, blending
  neighbouring entries; its count runs about one behind Fractadyne's (offset -1 for most pixels,
  -3..+1 overall). `prepare` writes each scene a file of exactly 2^D entries, each colour repeated
  k times, so blends fall only at band edges (inspect/<scene>.fs.map, depth in inspect/plan.json).
- Imagina, through imagina-cli: its counts come back ENCODED IN COLOUR and are coloured here like
  Fraktaler-3's. `--iterations` makes it store each pixel's smooth iteration count instead of its
  default value, 8*log(1/DE + 1) from the distance estimate (which ran 600..738 on scene 62 where
  Fractadyne counted 1,029..1,856, and lit filaments rather than bands). With no --palette-cycle,
  imagina-cli colours a pixel with map entry floor(count) mod n, unblended, so `prepare` writes
  inspect/imagina-decode.map, entry i = (i mod 256, i / 256, 77), and `im_counts` reads the count
  back mod 65,536. Measured 2026-11-03 on scenes 61-64: its whole count is Fractadyne's plus 1 to
  3 (the median), within 1-2 at the median pixel; Cerebral Spin (63, 2M iterations) spreads far
  wider (p90 445), which is Imagina's own Julia-sampling approximation. imagina-cli is not
  deterministic: two runs of one build differed at 82% of pixels of scene 64.
- Fractal Zoomer and Fracturing Fog: their own images for now, marked "own palette".
  Fraktaler-3 3.1 has no colour option, which is why its raw counts are used.

THE DEVIATION MAP (Fraktaler-3 and Imagina against Fractadyne)
--------------------------------------------------------------
Over pixels where both escaped, the median count difference is taken as the two renderers'
convention offset (they count escapes from different radii) and removed. A pixel then deviates
when its count lies outside the RANGE of Fractadyne's 3x3 neighbourhood by more than DEV_SLACK
(a renderer samples a pixel somewhere inside it), and counts only as part of a connected patch of
DEV_MIN_PATCH pixels or more; isolated ones are sampling speckle (see `deviation`). Escaped in one
renderer only is its own class. A pixel whose neighbourhood spans more than a band is CHAOTIC: no
count there is stable to a pixel, so it is drawn grey and not judged. Colours: agreeing pixels
dimmed, deviating patches magenta, speckle dull purple, escaped-in-one-only red, chaotic grey.

Exit 0 on success, 1 if a scene's raw counts could not be read, 2 on a usage error.
Needs numpy and Pillow.
"""
import argparse
import glob
import json
import math
import os
import struct
import sys
import zlib

try:
    import numpy as np
    from PIL import Image, ImageDraw, ImageFont
except ImportError:
    sys.stderr.write("inspect: needs numpy and Pillow (pip install numpy pillow)\n")
    sys.exit(2)

PALETTE = np.array([[255, 255, 255], [0, 114, 178], [240, 228, 66], [213, 94, 0]], dtype=np.uint8)
INTERIOR = np.array([0, 0, 0], dtype=np.uint8)
BAND_PX = 10
K_MAX = 1 << 18
FS_DEPTHS = (5, 6, 8, 12, 16, 20)
NEVER_ESCAPED = 0xFFFFFFFF
# Imagina's counts come back through colour: entry i of the decode map is (i mod 256, i / 256,
# IM_MARK), so a pixel gives floor(count) mod IM_N. The blue lane marks a decodable pixel.
IM_N = 65536
IM_MARK = 77
# A count may sit this far outside the reference's 3x3 range before it deviates (escape-count
# conventions differ by a step or two even after the median offset).
DEV_SLACK = 2
# A deviation counts only as part of a connected patch (8-neighbour) of at least this many pixels.
# Measured on bench scenes 62-64 (2026-11-03): in smooth areas Fraktaler-3 and Fractadyne agree to
# +-1 iteration everywhere, and where the field has structure finer than a pixel the two disagree
# by up to the local spread in ISOLATED pixels (no sub-pixel sampling shift between them). A real
# aberration - a glitch blob, a blank patch, a shifted band - is a coherent region.
DEV_MIN_PATCH = 12


# ---- reading OpenEXR (scanline or single-level tiled; none, RLE, ZIPS, ZIP) ----
def _unpredict(buf):
    t = np.frombuffer(buf, dtype=np.uint8).astype(np.int64)
    t = ((np.cumsum(t - 128) + 128) & 0xFF).astype(np.uint8)
    half = (len(t) + 1) // 2
    out = np.empty(len(t), dtype=np.uint8)
    out[0::2] = t[:half]
    out[1::2] = t[half:]
    return out.tobytes()


def _unrle(src):
    out, i = bytearray(), 0
    while i < len(src):
        c = src[i] - 256 if src[i] > 127 else src[i]
        i += 1
        if c < 0:
            out += src[i:i - c]
            i -= c
        else:
            out += bytes([src[i]]) * (c + 1)
            i += 1
    return bytes(out)


def read_exr(path):
    """{channel: 2-D array} from an OpenEXR file."""
    with open(path, "rb") as fh:
        b = fh.read()
    if len(b) < 8 or struct.unpack("<I", b[:4])[0] != 20000630:
        raise ValueError("not an OpenEXR file")
    p, hdr = 8, {}
    while True:
        e = b.index(b"\0", p)
        name = b[p:e].decode("latin1")
        p = e + 1
        if not name:
            break
        e = b.index(b"\0", p)
        p = e + 1
        n = struct.unpack("<I", b[p:p + 4])[0]
        p += 4
        hdr[name] = b[p:p + n]
        p += n
    types = {0: ("<u4", 4), 1: ("<f2", 2), 2: ("<f4", 4)}
    v, q, chans = hdr["channels"], 0, []
    while v[q] != 0:
        e = v.index(b"\0", q)
        cn = v[q:e].decode("latin1")
        q = e + 1
        pt = struct.unpack("<i", v[q:q + 4])[0]
        q += 16
        chans.append((cn,) + types[pt])
    chans.sort()
    comp = hdr["compression"][0]
    if comp not in (0, 1, 2, 3):
        raise ValueError("compression %d is not supported" % comp)
    x0, y0, x1, y1 = struct.unpack("<4i", hdr["dataWindow"])
    W, H = x1 - x0 + 1, y1 - y0 + 1
    out = {c: np.zeros((H, W), dtype=dt) for c, dt, _s in chans}

    def decode(raw, expect):
        if len(raw) >= expect:
            return raw
        if comp == 1:
            return _unpredict(_unrle(raw))
        return _unpredict(zlib.decompress(raw))

    if "tiles" in hdr:
        tw, th, mode = struct.unpack("<IIB", hdr["tiles"])
        if mode & 0xF:
            raise ValueError("multi-level tiled EXR is not supported")
        nx, ny = (W + tw - 1) // tw, (H + th - 1) // th
        for off in struct.unpack("<%dQ" % (nx * ny), b[p:p + 8 * nx * ny]):
            tx, ty, _lx, _ly, size = struct.unpack("<5i", b[off:off + 20])
            cw, ch = min(tw, W - tx * tw), min(th, H - ty * th)
            data = decode(b[off + 20:off + 20 + size], cw * ch * sum(s for _c, _d, s in chans))
            k = 0
            for row in range(ch):
                for c, dt, s in chans:
                    out[c][ty * th + row, tx * tw:tx * tw + cw] = np.frombuffer(data[k:k + cw * s], dtype=dt)
                    k += cw * s
        return out
    lines = {0: 1, 1: 1, 2: 1, 3: 16}[comp]
    nchunks = (H + lines - 1) // lines
    stride = W * sum(s for _c, _d, s in chans)
    for off in struct.unpack("<%dQ" % nchunks, b[p:p + 8 * nchunks]):
        y, size = struct.unpack("<ii", b[off:off + 8])
        nl = min(lines, y1 - y + 1)
        data = decode(b[off + 8:off + 8 + size], nl * stride)
        k = 0
        for row in range(nl):
            for c, dt, s in chans:
                out[c][y - y0 + row] = np.frombuffer(data[k:k + W * s], dtype=dt)
                k += W * s
    return out


# ---- counts ----
def fd_counts(path):
    """Fractadyne's --render-iter EXR: whole counts (int64), -1 for interior."""
    r = read_exr(path)["R"].astype(np.float64)
    n = np.floor(r).astype(np.int64)
    n[r < 0] = -1
    return n


def f3_counts(path):
    """Fraktaler-3's raw EXR (N, or N0 + N1): whole counts (int64), -1 for never escaped."""
    ch = read_exr(path)
    if "N" in ch:
        n = ch["N"].astype(np.int64)
        n[ch["N"] == NEVER_ESCAPED] = -1
        return n
    lo, hi = ch["N0"].astype(np.int64), ch["N1"].astype(np.int64)
    n = lo | (hi << 32)
    n[(ch["N0"] == NEVER_ESCAPED) & (ch["N1"] == NEVER_ESCAPED)] = -1
    return n


def im_counts(path, ref):
    """Imagina's whole counts from its decode-map render (im-inspect/<scene>.ppm), -1 for interior,
    plus the share of pixels that did not decode. The image holds each count mod IM_N; it is put
    back at the value nearest Fractadyne's count there (its 3x3 maximum where Fractadyne's pixel is
    interior), after the median offset between the two, so a disagreement of IM_N / 2 iterations or
    more is misread in size: it still deviates, unless it lands by chance inside Fractadyne's 3x3
    range. (Scene 63, 2026-11-03: a 2^21-entry map showed Imagina's ring by the minibrot 3,448 to
    91,987 below Fractadyne's - Fraktaler-3 agrees with Fractadyne there - and 929 of its 3,066
    deviating pixels past the IM_N / 2 this map can size; every one was flagged.)"""
    rgb = np.asarray(Image.open(path).convert("RGB")).astype(np.int64)
    if rgb.shape[:2] != ref.shape:
        raise ValueError("Imagina image is %dx%d, Fractadyne's counts %dx%d" % (
            rgb.shape[1], rgb.shape[0], ref.shape[1], ref.shape[0]))
    interior = rgb.sum(axis=2) == 0
    bad = (rgb[..., 2] != IM_MARK) & ~interior
    dec = rgb[..., 0] + (rgb[..., 1] << 8)
    anchor = ref.copy()
    near = np.max(np.stack(neighbours(ref, -1)), axis=0)
    anchor[ref < 0] = near[ref < 0]
    if (anchor < 0).any():
        anchor[anchor < 0] = int(np.median(ref[ref >= 0])) if (ref >= 0).any() else 0

    def unwrap(target):
        return dec + IM_N * np.round((target - dec) / IM_N).astype(np.int64)

    v = unwrap(anchor)
    both = (ref >= 0) & ~interior & ~bad
    off = int(np.median((v - ref)[both])) if both.any() else 0
    v = unwrap(anchor + off)
    v[interior | bad] = -1
    return v, float(bad.mean())


def imagina_decode_lines():
    return ["%d %d %d" % (i & 255, i >> 8, IM_MARK) for i in range(IM_N)]


def band_width(n):
    """A power of two k so that bands are about BAND_PX pixels wide: BAND_PX times the median
    count step between escaped neighbours (the 75th percentile when most steps are zero)."""
    steps = []
    for a, b in ((n[:, 1:], n[:, :-1]), (n[1:, :], n[:-1, :])):
        both = (a >= 0) & (b >= 0)
        steps.append(np.abs(a - b)[both])
    s = np.concatenate(steps) if steps else np.zeros(0)
    if s.size == 0:
        return 1
    g = float(np.percentile(s, 50)) or float(np.percentile(s, 75)) or 1.0
    k = 2 ** round(math.log2(max(1.0, BAND_PX * g)))
    return int(min(K_MAX, k))


def colour(n, k):
    img = PALETTE[(np.maximum(n, 0) // k) % len(PALETTE)]
    img[n < 0] = INTERIOR
    return img


def neighbours(a, fill):
    """The 3x3 neighbourhood of every pixel, as 9 shifted copies (edges padded with `fill`)."""
    p = np.pad(a, 1, constant_values=fill)
    h, w = a.shape
    return [p[dy:dy + h, dx:dx + w] for dy in range(3) for dx in range(3)]


def deviation(ref, test, k):
    """The deviation image and its statistics. `test`'s counts, less their median offset from
    `ref` (escape conventions: Fraktaler-3's EXR adds 1024 and counts from another radius), are
    compared with the RANGE of `ref`'s 3x3 neighbourhood, since a renderer samples a pixel
    somewhere inside it: a pixel deviates when it lies outside that range by more than DEV_SLACK.
    Deviations count only in connected patches of DEV_MIN_PATCH pixels or more; isolated ones are
    sampling speckle where the field has structure finer than a pixel. (Tried and measured on
    bench scenes 61-64, 2026-11-03: a fixed tolerance on the pixel flagged 5 to 15% of every scene,
    evenly through the fine bands; judging only smooth pixels left 59 to 81% unjudged; the range
    test alone flagged 0.7 to 6.6% as isolated pixels in the decoration chains.)"""
    both = (ref >= 0) & (test >= 0)
    offset = int(np.median((test - ref)[both])) if both.any() else 0
    t = np.where(test >= 0, test - offset, -1)
    near = neighbours(ref, -1)
    lo = np.full(ref.shape, np.iinfo(np.int64).max, dtype=np.int64)
    hi = np.full(ref.shape, -1, dtype=np.int64)
    ref_int = np.zeros(ref.shape, dtype=bool)
    ref_esc = np.zeros(ref.shape, dtype=bool)
    for q in near:
        lo = np.minimum(lo, np.where(q >= 0, q, lo))
        hi = np.maximum(hi, q)
    for q in neighbours(ref, -2):
        ref_int |= q == -1
        ref_esc |= q >= 0
    # Escaped in one only: the test's state matches no reference pixel in the neighbourhood.
    one_raw = ((t < 0) & ~ref_int) | ((t >= 0) & ~ref_esc)
    raw_dev = (t >= 0) & ref_esc & ((t < lo - DEV_SLACK) | (t > hi + DEV_SLACK)) & ~one_raw
    dev, patches = patches_of(raw_dev, DEV_MIN_PATCH)
    one, one_patches = patches_of(one_raw, 3)
    speckle = raw_dev & ~dev
    # Where the neighbourhood spans more than a whole band the range test is near vacuous.
    chaos = (hi - lo > k) & (ref >= 0) & ~raw_dev & ~one_raw
    img = (colour(ref, k).astype(np.float32) * 0.35).astype(np.uint8)
    img[chaos] = (128, 128, 128)
    img[speckle] = (150, 60, 150)
    img[dev] = (255, 0, 255)
    img[one] = (255, 0, 0)
    return img, {
        "offset": offset,
        "slack": DEV_SLACK,
        "deviating_patches": patches,
        "deviating_share": round(float(dev.sum()) / n_px(ref), 5),
        "escaped_in_one_only_patches": one_patches,
        "escaped_in_one_only_share": round(float(one.sum()) / n_px(ref), 5),
        "speckle_share": round(float(speckle.sum()) / n_px(ref), 5),
        "chaotic_share": round(float(chaos.sum()) / n_px(ref), 5),
    }


def patches_of(mask, min_px):
    """(the pixels of `mask` in 8-connected patches of at least `min_px`, how many such patches)."""
    h, w = mask.shape
    keep = np.zeros_like(mask)
    seen = np.zeros_like(mask)
    count = 0
    for y0, x0 in zip(*np.nonzero(mask)):
        if seen[y0, x0]:
            continue
        stack, comp = [(y0, x0)], []
        seen[y0, x0] = True
        while stack:
            y, x = stack.pop()
            comp.append((y, x))
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    yy, xx = y + dy, x + dx
                    if 0 <= yy < h and 0 <= xx < w and mask[yy, xx] and not seen[yy, xx]:
                        seen[yy, xx] = True
                        stack.append((yy, xx))
        if len(comp) >= min_px:
            count += 1
            ys, xs = zip(*comp)
            keep[list(ys), list(xs)] = True
    return keep, count


def n_px(a):
    return a.shape[0] * a.shape[1]


def fs_palette(k):
    """FractalShark's palette file for band width k: (depth, lines)."""
    depth = next(d for d in FS_DEPTHS if (1 << d) >= k * len(PALETTE))
    lines = ["%d %d %d" % tuple(PALETTE[(i // k) % len(PALETTE)]) for i in range(1 << depth)]
    return depth, lines


# ---- commands ----
def prepare(res):
    raw = os.path.join(res, "fd-raw")
    out = os.path.join(res, "inspect")
    os.makedirs(out, exist_ok=True)
    plan, bad = {}, 0
    for path in sorted(glob.glob(os.path.join(raw, "*.exr"))):
        scene = os.path.basename(path)[:-4]
        try:
            n = fd_counts(path)
        except Exception as e:  # report and go on; the scene is left out of the plan
            print("  %s: cannot read %s (%s)" % (scene, path, e))
            bad += 1
            continue
        k = band_width(n)
        depth, lines = fs_palette(k)
        with open(os.path.join(out, scene + ".fs.map"), "w", newline="\n") as f:
            f.write("\n".join(lines) + "\n")
        plan[scene] = {"k": k, "fs_depth": depth, "interior_share": round(float((n < 0).mean()), 5)}
        print("  %-30s band %d iterations, FractalShark depth %d" % (scene, k, depth))
    with open(os.path.join(out, "plan.json"), "w") as f:
        json.dump(plan, f, indent=1)
    with open(os.path.join(out, "imagina-decode.map"), "w", newline="\n") as f:
        f.write("\n".join(imagina_decode_lines()) + "\n")
    return 1 if bad else 0


def font(size):
    for name in ("segoeui.ttf", "DejaVuSans.ttf", "Arial.ttf"):
        try:
            return ImageFont.truetype(name, size)
        except OSError:
            pass
    return ImageFont.load_default()


def own_image(res, pattern):
    hits = sorted(glob.glob(os.path.join(res, pattern)))
    return Image.open(hits[0]).convert("RGB") if hits else None


def sheet(res, out_path):
    with open(os.path.join(res, "inspect", "plan.json")) as f:
        plan = json.load(f)
    cols = ["Fractadyne", "Fraktaler-3", "Fraktaler-3 vs Fractadyne", "FractalShark", "Imagina",
            "Imagina vs Fractadyne", "Fractal Zoomer", "Fracturing Fog"]
    TW, TH, PAD, LAB, LEFT = 320, 180, 6, 24, 190
    rows = sorted(plan)
    W = LEFT + len(cols) * (TW + PAD)
    H = LAB + len(rows) * (TH + LAB + PAD) + 40
    img = Image.new("RGB", (W, H), (30, 30, 30))
    d = ImageDraw.Draw(img)
    f, fs = font(15), font(13)
    for j, c in enumerate(cols):
        d.text((LEFT + j * (TW + PAD), 4), c, fill=(235, 235, 235), font=f)
    stats, bad = {}, 0
    for i, scene in enumerate(rows):
        k = plan[scene]["k"]
        y = LAB + i * (TH + LAB + PAD)
        d.text((6, y + TH // 2 - 18), scene, fill=(235, 235, 235), font=f)
        d.text((6, y + TH // 2 + 2), "band %d iterations" % k, fill=(180, 180, 180), font=fs)
        tiles = {}
        try:
            ref = fd_counts(os.path.join(res, "fd-raw", scene + ".exr"))
        except Exception as e:
            print("  %s: Fractadyne raw counts unreadable (%s)" % (scene, e))
            bad += 1
            continue
        tiles[0] = (Image.fromarray(colour(ref, k)), "")
        s = {"k": k}
        f3p = os.path.join(res, "f3-raw", scene + ".exr")
        if os.path.exists(f3p):
            try:
                t = f3_counts(f3p)
                if t.shape == ref.shape:
                    dimg, s["fraktaler3"] = deviation(ref, t, k)
                    # Coloured after removing its offset from Fractadyne's count (an EXR iteration
                    # bias of 1024 plus its escape radius), so its bands are in phase.
                    off = s["fraktaler3"]["offset"]
                    tiles[1] = (Image.fromarray(colour(np.where(t >= 0, t - off, -1), k)), "offset %+d removed" % off)
                    f3s = s["fraktaler3"]
                    tiles[2] = (Image.fromarray(dimg), "%d deviating patches (%.2f%%), %d escaped-in-one patches" % (
                        f3s["deviating_patches"], 100 * f3s["deviating_share"], f3s["escaped_in_one_only_patches"]))
            except Exception as e:
                print("  %s: Fraktaler-3 raw counts unreadable (%s)" % (scene, e))
        stem = scene.replace(".", "p")
        fsi = own_image(res, os.path.join("fs-inspect", stem + ".png"))
        if fsi is not None:
            tiles[3] = (fsi, "")
        else:
            o = own_image(res, "fs-%s-r1.png" % stem)
            if o is not None:
                tiles[3] = (o, "own palette")
        imp = os.path.join(res, "im-inspect", scene + ".ppm")
        if os.path.exists(imp):
            try:
                t, undecodable = im_counts(imp, ref)
                dimg, s["imagina"] = deviation(ref, t, k)
                s["imagina"]["undecodable_share"] = round(undecodable, 5)
                off = s["imagina"]["offset"]
                tiles[4] = (Image.fromarray(colour(np.where(t >= 0, t - off, -1), k)), "offset %+d removed" % off)
                ims = s["imagina"]
                tiles[5] = (Image.fromarray(dimg), "%d deviating patches (%.2f%%), %d escaped-in-one patches" % (
                    ims["deviating_patches"], 100 * ims["deviating_share"], ims["escaped_in_one_only_patches"]))
            except Exception as e:
                print("  %s: Imagina counts unreadable (%s)" % (scene, e))
        if 4 not in tiles:
            o = own_image(res, "im-%s.png" % scene)
            if o is not None:
                tiles[4] = (o, "own palette")
        for j, pat in ((6, "fz-%s.png" % scene), (7, "ff-%s.png" % scene)):
            o = own_image(res, pat)
            if o is not None:
                tiles[j] = (o, "own palette")
        for j, (tile, note) in tiles.items():
            x = LEFT + j * (TW + PAD)
            # Counts coloured here (and FractalShark's stepped palette) are hard-edged: no blending.
            img.paste(tile.resize((TW, TH), Image.LANCZOS if note == "own palette" else Image.NEAREST), (x, y))
            if note:
                d.text((x + 2, y + TH + 3), note, fill=(200, 200, 200), font=fs)
        stats[scene] = s
    d.text((6, H - 34), "Palette: band(n) = floor(n / k) mod 4 (white, blue, yellow, vermillion), interior black. "
           "Deviation: magenta = a patch of 12+ px outside Fractadyne's 3x3 count range by more than 2 (after the median offset); "
           "dull purple = isolated pixels (sampling); red = escaped in one only; grey = chaotic; dim = agrees.", fill=(200, 200, 200), font=fs)
    img.save(out_path, quality=90)
    with open(os.path.join(res, "inspect", "inspect.json"), "w") as f:
        json.dump(stats, f, indent=1)
    print("  sheet: %s (%d scenes)" % (out_path, len(rows)))
    return 1 if bad else 0


def main():
    ap = argparse.ArgumentParser(description="inspection sheets on one stepped palette")
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("prepare")
    s.add_argument("results")
    s = sub.add_parser("sheet")
    s.add_argument("results")
    s.add_argument("--out", default="")
    a = ap.parse_args()
    if not os.path.isdir(a.results):
        sys.stderr.write("inspect: no such directory: %s\n" % a.results)
        return 2
    if a.cmd == "prepare":
        return prepare(a.results)
    return sheet(a.results, a.out or os.path.join(a.results, "inspect-sheet.jpg"))


if __name__ == "__main__":
    sys.exit(main())
