#!/usr/bin/env python3
"""Load a `fractadyne --render-lanes DIR` folder into NumPy, and color it (coloring plan B9).

`--render-lanes` writes every lane of a view's G-buffer exactly as the GPU computed it, at any
depth, so a coloring idea can be tried in NumPy on the app's own numbers without writing an
iteration (which in double precision could not go deep, and would not be the app's arithmetic):

    fractadyne --render-lanes DIR --center RE IM --zoom 1e30 --iter 25000 --size 480x270 --method fringe

    import lanes                                    # with scripts/ on sys.path
    v = lanes.load("DIR")
    v.count, v.normal_x, v.normal_y, v.de_log2      # (H, W) float32, as --render-iter's channels
    v.counts                                        # the counts themselves, float64 (count + count_base)
    v.stripe, v.triangle, v.trap, v.angle           # the statistics accumulated (None if not)
    v.palette                                       # (N, 4) display RGBA, the baked palette
    v.view                                          # view.json: band, statistics, the palette map
    rgb = lanes.palette_at(v, t)                    # the app's palette fetch at coordinate t

From the command line:

    python -I scripts/lanes.py info DIR
    python -I scripts/lanes.py smooth DIR [--out F.png] [--compare APP.png]
    python -I scripts/lanes.py fringe DIR [--out F.png] [--compare APP.png]

`smooth` and `fringe` are worked examples: each colors the lanes in NumPy as the app's color pass
does (point sampled), and `--compare` measures the result against the app's own render of the same
view (`--render -o APP.png --ss 1 --no-palette-aa --no-watermark`, the same method and size). The
fringe is the coloring plan's C9, written here from its WGSL (`mandelbrot.wgsl`, `fringed`): the
shape a new idea takes before it goes to the GPU. Needs NumPy; `--out` and `--compare` need Pillow.
"""
import argparse
import json
import sys
from pathlib import Path

import numpy as np

MAIN = ("count", "normal_x", "normal_y", "de_log2")
STATS = ("stripe", "triangle", "trap", "angle")


class Lanes:
    """One `--render-lanes` folder: each lane an (H, W) float32 array, or None if not written."""

    def __init__(self, folder):
        self.dir = Path(folder)
        self.view = json.loads((self.dir / "view.json").read_text(encoding="utf-8"))
        if self.view.get("format") != "fractadyne-lanes":
            raise ValueError(f"{self.dir}: view.json is not a lanes folder's")
        for name in MAIN + STATS + ("palette", "inside_palette"):
            f = self.dir / f"{name}.npy"
            setattr(self, name, np.load(f) if f.exists() else None)
        self.coloring = self.view["coloring"]
        # The count lane holds each count less this (coloring plan D2; 0 in an older folder).
        self.count_base = int(self.view.get("count_base", 0))

    @property
    def counts(self):
        """The smooth counts themselves, float64: the count lane plus `count_base`, whose f32 keeps
        the fraction a count past 2^21 would lose on its own; the inside and glitches as the lane."""
        c = self.count.astype(np.float64)
        return np.where(c >= 0, c + self.count_base, c)

    @property
    def escaped(self):
        """The pixels that escaped (count > 0): inside the set reads -1, a glitched pixel -2."""
        return self.count > 0

    @property
    def shape(self):
        return self.count.shape


def load(folder):
    return Lanes(folder)


def palette_at(v, t, palette=None, smooth=None):
    """The palette at coordinate `t` (any shape), as the shader's `palette_in`: the coordinate
    wraps (fract), and a smooth palette interpolates between entry CENTERS (entry i sits at
    (i + 0.5) / N), a stepped one takes the entry t falls in. Display RGB, (..., 3)."""
    pal = v.palette if palette is None else palette
    smooth = v.coloring["palette_smooth"] if smooth is None else smooth
    n = len(pal)
    t = np.asarray(t, dtype=np.float64)
    t = t - np.floor(t)
    if not smooth:
        return pal[np.clip((t * n).astype(np.int64), 0, n - 1), :3]
    x = t * n - 0.5
    i0 = np.floor(x)
    f = (x - i0)[..., None]
    i0 = i0.astype(np.int64)
    return pal[i0 % n, :3] * (1.0 - f) + pal[(i0 + 1) % n, :3] * f


CURVES = {
    "log": lambda d: np.log(d + 1.0),
    "sqrt": np.sqrt,
    "cbrt": np.cbrt,
    "root4": lambda d: np.sqrt(np.sqrt(d)),
    "loglog": lambda d: np.log(np.log(d + 1.0) + 1.0),
    "square": lambda d: d * d,
}


def palette_coord(v, value):
    """A lane's value under the view's palette map (`coord_under`): linear, or a curve (norm_mode:
    log, sqrt, cbrt, root4, loglog, square) of the height above norm_lo, then times cycle plus
    offset."""
    c = v.coloring
    value = np.asarray(value, dtype=np.float64)
    if c["norm_mode"] in CURVES:
        value = CURVES[c["norm_mode"]](np.maximum(value - c["norm_lo"], 0.0))
    return value * c["cycle"] + c["offset"]


# ---- OKLab (Bjorn Ottosson's matrices, as the shader's oklab_of / oklab_linear) ----

def srgb_decode(c):
    c = np.clip(c, 0.0, 1.0)
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)


def srgb_encode(c):
    c = np.clip(c, 0.0, 1.0)
    return np.where(c <= 0.0031308, 12.92 * c, 1.055 * c ** (1.0 / 2.4) - 0.055)


_M1 = np.array([[0.4122214708, 0.5363325363, 0.0514459929],
                [0.2119034982, 0.6806995451, 0.1073969566],
                [0.0883024619, 0.2817188376, 0.6299787005]])
_M2 = np.array([[0.2104542553, 0.7936177850, -0.0040720468],
                [1.9779984951, -2.4285922050, 0.4505937099],
                [0.0259040371, 0.7827717662, -0.8086757660]])
_M3 = np.array([[1.0, 0.3963377774, 0.2158037573],
                [1.0, -0.1055613458, -0.0638541728],
                [1.0, -0.0894841775, -1.2914855480]])
_M4 = np.array([[4.0767416621, -3.3077115913, 0.2309699292],
                [-1.2684380046, 2.6097574011, -0.3413193965],
                [-0.0041960863, -0.7034186147, 1.7076147010]])


def oklab_of(rgb):
    """Display (sRGB-encoded) RGB, (..., 3), to OKLab."""
    lms = srgb_decode(rgb) @ _M1.T
    return np.cbrt(np.maximum(lms, 0.0)) @ _M2.T


def oklab_linear(lab):
    """OKLab to linear-light sRGB, not clamped (outside 0..1 is out of gamut)."""
    return ((lab @ _M3.T) ** 3) @ _M4.T


def with_lightness(rgb, lightness):
    """Display colors `rgb` (..., 3) at OKLab lightness `lightness` (...), hue and chroma kept, the
    chroma bisected down (12 steps) until the color fits the gamut, as the shader's with_lightness."""
    lab = oklab_of(rgb)
    lightness = np.broadcast_to(lightness, lab.shape[:-1])

    def at(scale):
        return oklab_linear(np.stack([lightness, lab[..., 1] * scale, lab[..., 2] * scale], axis=-1))

    def fits(lin):
        return np.all((lin >= -1.0e-5) & (lin <= 1.0 + 1.0e-5), axis=-1)

    full = at(1.0)
    lo, hi = np.zeros(lightness.shape), np.ones(lightness.shape)
    for _ in range(12):
        mid = 0.5 * (lo + hi)
        ok = fits(at(mid))
        lo, hi = np.where(ok, mid, lo), np.where(ok, hi, mid)
    return srgb_encode(np.where(fits(full)[..., None], full, at(lo)))


# ---- the worked examples ----

def color_smooth(v):
    """The Smooth method, point sampled: the palette at the smooth count's coordinate; the inside
    the interior color. (H, W, 3) display RGB."""
    out = np.empty(v.shape + (3,))
    out[...] = v.coloring["interior_color"][:3]
    e = v.escaped
    out[e] = palette_at(v, palette_coord(v, v.counts[e]))
    return out


def color_fringe(v):
    """C9's fringe, point sampled: the palette at the slope normal's direction (in turns, a whole
    `cycle` a turn) at an OKLab lightness from the distance estimate, dark at the set and light
    from a pixel away: 0.12 + 0.78 sqrt(clamp(d / 1 px)). Inside the set, or where there is no
    distance estimate, the interior color. (H, W, 3) display RGB."""
    out = np.empty(v.shape + (3,))
    out[...] = v.coloring["interior_color"][:3]
    e = v.escaped & (v.de_log2 <= 1.0e20)
    turn = np.mod(np.arctan2(v.normal_y[e], v.normal_x[e]) / (2.0 * np.pi) + 1.0, 1.0)
    d_px = np.exp2(np.minimum(v.de_log2[e].astype(np.float64), 60.0))
    lightness = 0.12 + 0.78 * np.sqrt(np.clip(d_px / 1.0, 0.0, 1.0))
    out[e] = with_lightness(palette_at(v, palette_coord(v, turn)), lightness)
    return out


METHODS = {"smooth": color_smooth, "fringe": color_fringe}


def save_png(rgb, path):
    from PIL import Image  # optional: only --out and --compare need it

    Image.fromarray(np.round(np.clip(rgb, 0.0, 1.0) * 255.0).astype(np.uint8), "RGB").save(path)


def compare(rgb, app_png, mask):
    """Levels (0..255) between `rgb` and the app's PNG, per pixel the largest channel's."""
    from PIL import Image

    app = np.asarray(Image.open(app_png).convert("RGB"), dtype=np.float64)
    if app.shape[:2] != rgb.shape[:2]:
        raise SystemExit(f"{app_png} is {app.shape[1]}x{app.shape[0]}, the lanes {rgb.shape[1]}x{rgb.shape[0]}")
    diff = np.abs(np.clip(rgb, 0.0, 1.0) * 255.0 - app).max(axis=-1)
    return diff, diff[mask]


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("what", choices=["info"] + sorted(METHODS))
    ap.add_argument("dir")
    ap.add_argument("--out", help="write the coloring as a PNG")
    ap.add_argument("--compare", help="the app's own render of the view, to measure against")
    a = ap.parse_args()
    v = load(a.dir)
    if a.what == "info":
        w = v.view
        print(f"{v.dir}: {w['width']}x{w['height']} {w['fractal']} at {w['zoom']}x ({w['band']}, BLA {'on' if w['bla'] else 'off'}), "
              f"{w['max_iter']} iterations, method {v.coloring['method']}, statistics {w['stats']}")
        for name in MAIN + STATS:
            lane = getattr(v, name)
            if lane is not None:
                x = lane[v.escaped]
                print(f"  {name:<9} escaped px {x.size}: {x.min():.6g} .. {x.max():.6g}, mean {x.mean():.6g}")
        print(f"  escaped {v.escaped.mean():.1%}; palette {len(v.palette)} entries ({'smooth' if v.coloring['palette_smooth'] else 'stepped'})")
        return 0
    rgb = METHODS[a.what](v)
    if a.out:
        save_png(rgb, a.out)
        print(f"wrote {a.out}")
    if a.compare:
        diff, esc = compare(rgb, a.compare, v.escaped)
        q = lambda x, p: float(np.percentile(x, p)) if x.size else float("nan")
        print(f"against {a.compare}: escaped px {esc.size}, mean {esc.mean():.2f} levels, 99th percentile {q(esc, 99):.2f}, "
              f"max {esc.max():.2f}; every px max {diff.max():.2f}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
