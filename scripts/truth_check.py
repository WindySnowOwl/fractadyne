#!/usr/bin/env python3
"""truth_check.py - grade a renderer's raw iteration counts against a reference-grade truth.

    python scripts/truth_check.py score TRUTH_STEM RENDER [--kind fd|f3|imagina|fz] [--png OUT.png]

TRUTH_STEM is a `fractadyne --truth` output without its extension (DIR/NAME: NAME.truth.json and
NAME.truth.bin). RENDER is the renderer's raw counts at the truth's size and view:
    fd       Fractadyne --render-iter EXR (smooth count, R channel; < 0 interior)
    f3       Fraktaler-3 raw EXR (N or N0+N1, bias 1024; 0xFFFFFFFF never escaped)
    imagina  imagina-cli --iterations through the bench kit's decode map (PPM/PNG)
    fz       Fractal Zoomer direct colour through FzsWriter --counts (PNG)
The kind is guessed from the file name when not given.

WHY. A render compared with another render can agree with it on a shared fault (2026-11-03: twelve
views drew a flat disc for every reference pick, and the audit comparing two picks called them
VACUOUS). The truth (`fractadyne --truth`, see `crates/fractadyne-core/src/harness/reference/truth.rs`)
is every pixel's escape count with no approximation, checked against plain arbitrary-precision
iteration.

HOW IT JUDGES.
- WHICH COUNT. Renderers count escapes from different radii (Fractal Zoomer 2, Fractadyne 256,
  Imagina 4096) and add their own constants, and the steps between radii vary across a view, so
  no one offset maps one radius onto another. The truth holds all three; the scorer takes the one
  this renderer's counts follow most closely (most pixels within 1 of the median difference) and
  says which. A Fractadyne smooth count is compared with the truth's own smooth count at 256.
- EVERY PIXEL against its RANGE: the truth's stencil evaluated each pixel's centre and four
  points 1/1000 px away, and kept the lowest and highest count (format 2). A count outside that
  range by more than SLACK (after the offset) is wrong wherever in that neighbourhood the
  renderer sampled. On a STABLE pixel (all five agree) the range is one value, and a Fractadyne
  smooth count is held to the truth's smooth count within 0.5. On a SENSITIVE pixel (structure
  finer than the pixel, such as a thin filament) the range is wider, and only a pixel whose
  structure is finer than 1/1000 px is beyond judging. Escaping where every point stayed inside
  (or the reverse) is its own class. Deviations count as connected patches (8-neighbour,
  PATCH_MIN px or more): a real fault - a lost filament, a glitch, a blank - is a region.
  (The first scorer judged sensitive pixels only by block medians and its self-check MISSED a
  planted 2-px filament: a thin line moves no median.)
- BLOCKS, additionally: the median signed difference over each BLOCK x BLOCK block's sensitive
  pixels. Sub-pixel disagreement has no sign; a broad bias does.
Without a stencil in the truth every range is a single value and every pixel counts as stable.

Exit 0 clean, 1 when anything deviates, 2 on a usage error. Needs numpy (and Pillow for images).
"""
import argparse
import importlib.util
import json
import os
import sys

import numpy as np

SLACK = 2          # iterations of disagreement tolerated after the offset (escape conventions)
PATCH_MIN = 12     # a deviation counts as a patch from this many connected pixels
BLOCK = 8          # the sensitive-pixel test's block size
BLOCK_MIN = 16     # escaped sensitive pixels a block needs to be judged

_here = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location(
    "inspect_sheet", os.path.join(_here, "..", "bench-kit", "tools", "inspect-sheet.py"))
INSP = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(INSP)

FLAG_PLAIN, FLAG_STENCIL, FLAG_STABLE, FLAG_CHECKED, FLAG_CHECK_OK = 1, 2, 4, 8, 16


def load_truth(stem):
    with open(stem + ".truth.json", encoding="utf-8") as f:
        head = json.load(f)
    w, h = head["width"], head["height"]
    n = w * h
    raw = np.fromfile(stem + ".truth.bin", dtype=np.uint8)
    if head.get("format") != 2 or raw.size != 41 * n:
        raise ValueError("%s: format %s, %d bytes; this scorer reads format 2 (%dx%d: %d bytes)" % (
            stem, head.get("format"), raw.size, w, h, 41 * n))
    plane = lambda off, t: raw[off:off + 4 * n].view(t).reshape(h, w)
    out = {
        "head": head,
        "n2": plane(0, "<i4").astype(np.int64),
        "n256": plane(4 * n, "<i4").astype(np.int64),
        "n4096": plane(8 * n, "<i4").astype(np.int64),
        "smooth256": plane(12 * n, "<f4").astype(np.float64),
        "flags": raw[16 * n:17 * n].reshape(h, w),
    }
    for j, name in enumerate(("n2", "n256", "n4096")):
        out[name + "_lo"] = plane(17 * n + 8 * j * n, "<i4").astype(np.int64)
        out[name + "_hi"] = plane(17 * n + (8 * j + 4) * n, "<i4").astype(np.int64)
    return out


def guess_kind(path):
    b = os.path.basename(path).lower()
    if b.endswith(".exr"):
        return "f3" if "f3" in b else "fd"
    if "im" in b.split("-")[0] or b.endswith(".ppm"):
        return "imagina"
    return "fz"


def read_render(path, kind, truth):
    """The render's counts as (values, is_smooth): float smooth counts for fd, whole counts else;
    -1 where it never escaped."""
    ref = truth["n256"]
    if kind == "fd":
        r = INSP.read_exr(path)["R"].astype(np.float64)
        return np.where(r < 0, -1.0, r), True
    if kind == "f3":
        n = INSP.f3_counts(path).astype(np.float64)
        return np.where(n < 0, -1.0, n - 1024.0), False
    if kind == "imagina":
        t, _ = INSP.im_counts(path, ref)
        return t.astype(np.float64), False
    if kind == "fz":
        return INSP.fz_counts(path, ref).astype(np.float64), False
    raise ValueError("unknown kind " + kind)


def patches(mask, min_px):
    keep, count = INSP.patches_of(mask, min_px)
    return keep, count


def score(stem, render, kind=None, png=None, plant=None):
    t = load_truth(stem)
    kind = kind or guess_kind(render)
    v, smooth = read_render(render, kind, t)
    if plant is not None:
        v = plant(v.copy(), t)
    if v.shape != t["n256"].shape:
        raise ValueError("render is %dx%d, truth %dx%d" % (v.shape[1], v.shape[0], t["n256"].shape[1], t["n256"].shape[0]))
    fl = t["flags"]
    has_stencil = bool((fl & FLAG_STENCIL).any())
    stable = (fl & FLAG_STABLE) != 0 if has_stencil else np.ones(fl.shape, dtype=bool)
    t_esc = t["n256"] >= 0
    r_esc = v >= 0

    # Which of the truth's counts this renderer follows: a smooth count by its whole part.
    vi = np.where(r_esc, np.floor(v), -1.0)
    best = None
    # A smooth count is Fractadyne's, defined at radius 256 (a shallow view, where the steps
    # between radii barely vary, otherwise lets another plane win the tie).
    for name in (("n256",) if smooth else ("n2", "n256", "n4096")):
        p = t[name].astype(np.float64)
        both = t_esc & r_esc & stable
        if not both.any():
            continue
        dd = (vi - p)[both]
        med = float(np.median(dd))
        share = float((np.abs(dd - med) <= 1).mean())
        if best is None or share > best[3]:
            best = (name, p, med, share)
    plane_name, plane, offset, _ = best
    lo = t[plane_name + "_lo"].astype(np.float64)
    hi = t[plane_name + "_hi"].astype(np.float64)

    # The range test, every pixel escaped in both (a stencil point that never escaped has lo -1:
    # no lower bound there).
    both = t_esc & r_esc
    x = vi - offset
    below = (lo >= 0) & (x < lo - SLACK)
    above = x > hi + SLACK
    dev = both & (below | above)
    if smooth:
        # A stable pixel holds Fractadyne's smooth count to the truth's own, within 0.5.
        dev |= both & stable & (np.abs(v - t["smooth256"]) > 0.5)
    # Escaped in one only: escaped where no stencil point did, or never escaped where all did.
    one = (r_esc & (hi < 0)) | (~r_esc & (lo >= 0))
    dev_keep, dev_patches = patches(dev, PATCH_MIN)
    one_keep, one_patches = patches(one, 3)
    d = np.where(both, vi - plane - offset, 0.0)
    tol = SLACK

    # Sensitive pixels, by block.
    sens = ~stable & t_esc & r_esc
    h, w = d.shape
    blocks_judged = blocks_bad = 0
    block_mask = np.zeros(d.shape, dtype=bool)
    if has_stencil:
        for by in range(0, h, BLOCK):
            for bx in range(0, w, BLOCK):
                m = sens[by:by + BLOCK, bx:bx + BLOCK]
                if m.sum() < BLOCK_MIN:
                    continue
                blocks_judged += 1
                if abs(float(np.median(d[by:by + BLOCK, bx:bx + BLOCK][m]))) > tol:
                    blocks_bad += 1
                    block_mask[by:by + BLOCK, bx:bx + BLOCK] = True

    out = {
        "truth": os.path.basename(stem),
        "render": os.path.basename(render),
        "kind": kind,
        "follows": plane_name,
        "offset": offset,
        "pixels": int(d.size),
        "stencil": has_stencil,
        "stable": int(stable.sum()),
        "interior_truth": int((~t_esc).sum()),
        "interior_render": int((~r_esc).sum()),
        "judged": int(both.sum()),
        "deviating": int(dev.sum()),
        "deviating_stable": int((dev & stable).sum()),
        "deviating_patches": int(dev_patches),
        "deviating_in_patches": int(dev_keep.sum()),
        "escaped_in_one": int(one.sum()),
        "escaped_in_one_patches": int(one_patches),
        "sensitive_blocks_judged": blocks_judged,
        "sensitive_blocks_deviating": blocks_bad,
        "median_abs_diff_stable": float(np.median(np.abs(d[stable & both]))) if (stable & both).any() else None,
    }
    out["verdict"] = "PASS" if (dev_patches == 0 and one_patches == 0 and blocks_bad == 0) else "DEVIATES"
    if png:
        from PIL import Image
        base = np.full(d.shape + (3,), 40, dtype=np.uint8)
        base[t_esc & r_esc & stable] = (90, 90, 90)
        base[sens] = (60, 60, 110)
        base[block_mask & sens] = (230, 140, 0)
        base[dev & ~dev_keep] = (150, 60, 150)
        base[dev_keep] = (255, 0, 255)
        base[one] = (255, 0, 0)
        Image.fromarray(base).save(png)
    return out


def selfcheck(stem, render, kind=None):
    """Plant known faults in a copy of a render the truth passes, and require each to be found.
    A gate that cannot go red is not a gate."""
    t = load_truth(stem)
    stable = (t["flags"] & FLAG_STABLE) != 0
    esc = t["n256"] >= 0
    h, w = esc.shape

    def where(mask, size):
        """The top-left of the size x size square holding the most `mask` pixels (a deep view's
        stable pixels are scattered: at 1e999x beside a minibrot, 42% of them, and no 20x20 square
        is wholly stable), if it holds enough of them to make a patch."""
        c = np.pad(mask.astype(np.int64), ((1, 0), (1, 0))).cumsum(0).cumsum(1)
        win = c[size:, size:] - c[:-size, size:] - c[size:, :-size] + c[:-size, :-size]
        y, x = np.unravel_index(int(np.argmax(win)), win.shape)
        return (int(y), int(x)) if win[y, x] >= 2 * PATCH_MIN else None

    def block(delta, size, mask):
        def f(v, _t):
            at = where(mask, size)
            if at is None:
                raise ValueError("no %dx%d region for a planted fault" % (size, size))
            y, x = at
            v[y:y + size, x:x + size] = np.where(v[y:y + size, x:x + size] >= 0, v[y:y + size, x:x + size] + delta, v[y:y + size, x:x + size])
            return v
        return f

    def filament():
        # A LOST FILAMENT: the thin ridges of the count field (a pixel at least SLACK+2 above both
        # neighbours across it, horizontally or vertically, where the truth's range is narrow) in
        # the band of rows richest in them, flattened to the lower of those neighbours - what a
        # renderer that thins a filament away draws there. Mostly sensitive or edge pixels.
        def f(v, _t):
            n = t["n256"].astype(np.float64)
            narrow = (t["n256_hi"] - t["n256_lo"]) <= 1
            pad = np.pad(n, 1, mode="edge")
            l, r, u, d = pad[1:-1, :-2], pad[1:-1, 2:], pad[:-2, 1:-1], pad[2:, 1:-1]
            m = SLACK + 2
            ridge_h = (n >= l + m) & (n >= r + m)
            ridge_v = (n >= u + m) & (n >= d + m)
            ridge = (ridge_h | ridge_v) & narrow & esc & (v >= 0)
            band = 40
            sums = np.convolve(ridge.sum(axis=1), np.ones(band), mode="valid")
            y0 = int(np.argmax(sums))
            sel = np.zeros_like(ridge)
            sel[y0:y0 + band] = ridge[y0:y0 + band]
            if sel.sum() < 2 * PATCH_MIN:
                raise ValueError("too few filament pixels (%d) for a lost-filament fault" % sel.sum())
            low = np.where(ridge_h, np.minimum(l, r), np.minimum(u, d))
            # The render's own whole count follows n256 by an offset; flatten by the same amount.
            v[sel] = v[sel] - (n[sel] - low[sel])
            print("  (lost filament: %d ridge pixels in rows %d-%d)" % (int(sel.sum()), y0, y0 + band))
            return v
        return f

    def blank(size):
        def f(v, _t):
            at = where(stable & esc, size)
            if at is None:
                raise ValueError("no region for a blank patch")
            y, x = at
            v[y:y + size, x:x + size] = -1
            return v
        return f

    cases = [
        ("unchanged", None, "PASS"),
        ("stable block 50 low (20x20)", block(-50.0, 20, stable & esc), "DEVIATES"),
        ("lost filament (ridges flattened)", filament(), "DEVIATES"),
        ("blank patch (10x10)", blank(10), "DEVIATES"),
    ]
    ok = True
    for name, plant, want in cases:
        got = score(stem, render, kind, None, plant)
        hit = got["verdict"] == want
        ok &= hit
        print("%-34s %-9s want %-9s %s  (patches %d, one-only %d, blocks %d/%d)" % (
            name, got["verdict"], want, "ok" if hit else "MISSED",
            got["deviating_patches"], got["escaped_in_one_patches"],
            got["sensitive_blocks_deviating"], got["sensitive_blocks_judged"]))
    return 0 if ok else 1


def main():
    ap = argparse.ArgumentParser(description="grade raw iteration counts against a --truth reference")
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("score")
    s.add_argument("truth", help="DIR/NAME of NAME.truth.json + NAME.truth.bin")
    s.add_argument("render")
    s.add_argument("--kind", choices=("fd", "f3", "imagina", "fz"))
    s.add_argument("--png", help="write a map: grey agrees, blue sensitive, orange a deviating block, "
                                 "magenta a deviating patch (dull purple isolated), red escaped in one only")
    s = sub.add_parser("selfcheck", help="plant faults in a copy of a passing render; each must be found")
    s.add_argument("truth")
    s.add_argument("render")
    s.add_argument("--kind", choices=("fd", "f3", "imagina", "fz"))
    a = ap.parse_args()
    try:
        if a.cmd == "selfcheck":
            return selfcheck(a.truth, a.render, a.kind)
        out = score(a.truth, a.render, a.kind, a.png)
    except (OSError, ValueError) as e:
        sys.stderr.write("truth_check: %s\n" % e)
        return 2
    print(json.dumps(out, indent=1))
    return 0 if out["verdict"] == "PASS" else 1


if __name__ == "__main__":
    sys.exit(main())
