#!/usr/bin/env python3
"""Turn a benchmark results folder into a single readable HTML report.

    tools/make-report.py RESULTS_DIR [--no-thumbs]

WHY THIS EXISTS
---------------
A results folder is a CSV, a markdown summary and thirty 4K PNGs. Everything needed to check a
number is in there, and none of it is in one place: the time is in results.csv, the picture is a
file you have to go and open, and the ARGUMENTS THAT PRODUCED BOTH were, until the manifest, not
recorded anywhere at all.

That last gap is not hypothetical. The FractalShark lane spent its whole life sending a zoom the
renderer silently truncated, rendering the wrong view at every depth, and no artifact on disk
recorded the argument responsible. A reader could not have found it, because a reader was never
shown it.

So this report puts the three together per scene: what was run, what came out, and how long it
took - with the images inline, because the fastest way to catch a wrong render is to look at it.

It reads run-manifest.json (written by run-all.ps1), results.csv, and, when present,
fs-view-check.json, sysinfo.txt and zoomseq/results.csv. Nothing here is required: a missing
input costs you that section, not the report.
"""
import argparse
import csv
import html
import json
import math
import os
import re
import sys
from collections import defaultdict

THUMB_W = 480

try:
    from PIL import Image
    HAVE_PIL = True
except ImportError:
    HAVE_PIL = False


# ---------------------------------------------------------------- loading

def load_json(path, default=None):
    try:
        with open(path) as fh:
            return json.load(fh)
    except Exception:
        return default


def load_csv(path):
    try:
        with open(path, newline="") as fh:
            return list(csv.DictReader(fh))
    except Exception:
        return []


def load_text(path, limit=None):
    try:
        with open(path, errors="replace") as fh:
            t = fh.read()
        return t[:limit] if limit else t
    except Exception:
        return ""


def fnum(v):
    try:
        return float(v)
    except (TypeError, ValueError):
        return None


# ---------------------------------------------------------------- thumbnails

def make_thumb(src, outdir, name):
    """Return the thumbnail's relative path, or the original if one cannot be made.

    The originals are 4K and several megabytes each; a page embedding thirty of them is not a
    report, it is a denial of service on the browser. Thumbnails are generated once and linked
    to the full-size file.
    """
    if not HAVE_PIL:
        return None
    try:
        os.makedirs(outdir, exist_ok=True)
        dst = os.path.join(outdir, name)
        if not os.path.exists(dst) or os.path.getmtime(dst) < os.path.getmtime(src):
            im = Image.open(src)
            im.thumbnail((THUMB_W, THUMB_W), Image.LANCZOS)
            im.convert("RGB").save(dst, "JPEG", quality=82)
        return "thumbs/" + name
    except Exception:
        return None


# ---------------------------------------------------------------- findings

def derive_findings(scenes, rows_by, views, meta, zs_rows):
    """Say what the run actually shows, computed rather than asserted.

    Only things the data supports. No adjectives that a number does not earn.
    """
    out = []

    # Wins per scene, among rows that are 'ok'. A DNF is not a slow result, so it never competes.
    wins = defaultdict(int)
    gaps = []
    for s in scenes:
        best, second = None, None
        for ren in ("fractadyne", "fraktaler3", "fractalshark", "imagina"):
            t = best_time(rows_by, ren, s)
            if t is None:
                continue
            if best is None or t < best[1]:
                second, best = best, (ren, t)
            elif second is None or t < second[1]:
                second = (ren, t)
        if best:
            wins[best[0]] += 1
            if second:
                gaps.append((second[1] / best[1], s, best[0], best[1], second[0], second[1]))
    if wins:
        parts = ", ".join("%s %d" % (r, n) for r, n in
                          sorted(wins.items(), key=lambda kv: -kv[1]))
        out.append(("Fastest per scene", "%s (of %d scenes with a valid time)"
                    % (parts, sum(wins.values()))))
    if gaps:
        gaps.sort(reverse=True)
        r, s, w, wt, l, lt = gaps[0]
        out.append(("Largest margin",
                    "%s on %s: %.1fs against %.1fs for %s, a factor of %.1f"
                    % (w, s, wt, lt, l, r)))

    # DNFs, with the reason attached. A DNF is a result and belongs in the findings.
    dnf = []
    for (ren, sc), rs in sorted(rows_by.items()):
        bad = [r for r in rs if r["status"] != "ok"]
        if bad and not any(r["status"] == "ok" for r in rs):
            dnf.append((ren, sc, bad[0]["status"], bad[0].get("note", "")))
    for ren, sc, st, note in dnf:
        out.append(("DNF: %s on %s" % (ren, sc), "%s%s" % (st, (" - " + note) if note else "")))

    # Run-to-run spread: the reader needs to know what counts as noise here before reading any
    # difference as a result.
    spreads = []
    for (ren, sc), rs in rows_by.items():
        ts = [fnum(r["wall_s"]) for r in rs if r["status"] == "ok" and fnum(r["wall_s"])]
        if len(ts) > 1:
            spreads.append((max(ts) - min(ts), ren, sc, min(ts), max(ts)))
    if spreads:
        spreads.sort(reverse=True)
        d, ren, sc, lo, hi = spreads[0]
        med = sorted(x[0] for x in spreads)[len(spreads) // 2]
        out.append(("Run-to-run spread",
                    "median %.1fs across %d repeated measurements; worst %.1fs "
                    "(%s on %s, %.1f to %.1f). Treat smaller differences as noise."
                    % (med, len(spreads), d, ren, sc, lo, hi)))

    # WORDING IS THE WHOLE POINT HERE. These states are not degrees of the same thing, and
    # reporting them as one is how this report came to say another project's renderer had failed
    # when it had not. A wrong MAGNIFICATION is a defect and voids a time. An unconfirmed
    # COMPARISON is a limit of the comparison and says nothing about the renderer at all, so it
    # is phrased as something we could not do, not something they got wrong.
    for ren, vv in sorted(views.items()):
        if not vv:
            continue
        name = {"fractalshark": "FractalShark", "imagina": "Imagina"}.get(ren, ren)
        wrong = [k for k, v in vv.items() if not v.get("magnification_ok", True)]
        unconf = [k for k, v in vv.items()
                  if v.get("magnification_ok", True) and not v.get("same_view")]
        if wrong:
            out.append(("%s: WRONG VIEW" % name,
                        "%d of %d renders are at a different magnification than the scene asks "
                        "for: %s. A time for the wrong picture is void, not slow."
                        % (len(wrong), len(vv), ", ".join(sorted(wrong)))))
        if unconf:
            agree = len(vv) - len(wrong)
            out.append(("%s: view" % name,
                        "magnification agrees with our own render on %d of %d scenes. %d could "
                        "not be confirmed by image (%s) - that is a limit of comparing two "
                        "renderers by picture, almost always a palette difference, and NOT a "
                        "fault in the renderer. The times stand."
                        % (agree, len(vv), len(unconf),
                           ", ".join(sorted(unconf)) if len(unconf) < len(vv) else "all of them")))
        if not wrong and not unconf:
            out.append(("%s: view" % name,
                        "all %d renders depict the right scene (checked against the Fractadyne "
                        "render)" % len(vv)))

    if meta.get("fractalshark_shape"):
        out.append(("FractalShark mode", meta["fractalshark_shape"]))
    if meta.get("zoomseq_amortisation"):
        out.append(("Zoom-sequence amortisation",
                    "Fractadyne %sx over single frames. Fraktaler-3's batch CLI renders one "
                    "image per invocation, so its 1.0 is a property of the CLI, not its engine."
                    % meta["zoomseq_amortisation"]))
    return out


def parse_zoom_log10(z):
    """log10 of a zoom string, computed TEXTUALLY.

    The corpus reaches 6.1e1105, which is +inf in a double, so the mantissa and the exponent are
    never multiplied back together. Accepts the Kalles Fraktaler spelling the scenes use
    ("1.333333E6") and the bare-exponent one ("1e6.12").
    """
    if z is None:
        return None
    s = str(z).strip()
    m = re.match(r"^([0-9]*\.?[0-9]+)[eE]\+?(-?[0-9]*\.?[0-9]+)$", s)
    if m:
        try:
            return math.log10(float(m.group(1))) + float(m.group(2))
        except ValueError:
            return None
    try:
        return math.log10(float(s))
    except (ValueError, OverflowError):
        return None


def scene_magnitudes(meta, runs_by, scenes):
    """log10 magnification per scene, from the manifest where possible.

    Preference order matters. meta['scene_mag'] is authoritative because run-all.ps1 takes it
    straight from scenes.csv. Falling back to a run record's own inputs keeps older folders
    sortable. The SLUG is never parsed: it leads with a scene id, not a magnitude, which is the
    very reason filename order is wrong here.
    """
    sm = meta.get("scene_mag") or {}
    out = {}
    for s in scenes:
        if s in sm:
            try:
                out[s] = float(sm[s])
                continue
            except (TypeError, ValueError):
                pass
        val = None
        for ren in ("fractadyne", "fractalshark", "fraktaler3"):
            for rec in runs_by.get((ren, s), []):
                inp = rec.get("inputs") or {}
                if not isinstance(inp, dict):
                    continue
                if inp.get("zoom_log2") not in (None, ""):
                    try:
                        val = float(inp["zoom_log2"]) * math.log10(2.0)
                    except (TypeError, ValueError):
                        val = None
                if val is None:
                    val = parse_zoom_log10(inp.get("kfr_zoom") or inp.get("zoom"))
                if val is not None:
                    break
            if val is not None:
                break
        if val is not None:
            out[s] = val
    return out


def fmt_mag(l10):
    """'1.3e6' from a log10. Textual for the same overflow reason as parse_zoom_log10."""
    if l10 is None:
        return ""
    e = math.floor(l10)
    m = 10.0 ** (l10 - e)
    if m >= 9.995:
        m, e = m / 10.0, e + 1
    return "%.1fe%d" % (m, int(e))


# One colour per lane, reused by the chart and its legend.
LANE_COLOR = {"fractadyne": "#79b8ff", "fraktaler3": "#e3b341",
              "fractalshark": "#5fd08a", "imagina": "#d48bff"}


def nice_log_ticks(lo, hi, want=6):
    """Tick positions on a log10 axis, at 1/2/5 x 10^n, covering [lo, hi]."""
    if hi <= lo:
        hi = lo + 1.0
    ticks = []
    e = math.floor(lo)
    while e <= math.ceil(hi):
        for m in (1, 2, 5):
            v = math.log10(m) + e
            if lo - 1e-9 <= v <= hi + 1e-9:
                ticks.append((v, m, e))
        e += 1
    # Thin out until the axis is readable rather than a picket fence.
    while len(ticks) > want * 2:
        ticks = [t for i, t in enumerate(ticks) if i % 2 == 0]
    return ticks


def svg_time_vs_depth(scenes, mags, rows_by, live, label):
    """Render time against magnification, both axes log.

    WHY THE X AXIS IS LOG TWICE. Magnification spans 1.3e6 to 6.1e1105, so a log scale on it
    plots the EXPONENT, 6 to 1105. Drawn linearly that is still a bad chart: this corpus samples
    1e6 to 1e77 densely and then jumps, so SEVEN OF TEN scenes pile into the first tenth of the
    width as an unreadable knot while two straight segments stretch across 800 orders of
    magnitude nobody measured - a line implying a trend through empty space. Spacing the axis by
    the order of magnitude OF THE EXPONENT separates every scene, keeps it strictly monotonic in
    zoom, and makes no claim about the gaps. Neither axis's slope is a rate, and it never was.

    Time is log for the ordinary reason: it spans 0.9 s to 65 s, and linearly one 65 s point
    flattens everything under five seconds onto the baseline, which is most of the data.

    A DNF BREAKS THE LINE rather than being drawn as a large time or interpolated across. A
    renderer that did not produce the picture has no time, and joining the points either side
    would draw a segment that never happened.
    """
    def xof(mag_log10):
        """Axis position: the order of magnitude of the exponent. See the docstring."""
        return math.log10(mag_log10) if mag_log10 and mag_log10 > 0 else None

    # Per renderer, a list of SEGMENTS: runs of consecutive scenes it completed. Splitting here
    # rather than drawing one path is what actually makes a DNF a gap. Building a single path
    # from the surviving points silently joins across the hole, which draws a segment spanning a
    # scene the renderer failed and reads as if it had simply been quick there.
    pts = {}
    for ren in live:
        segs, cur = [], []
        for s in sorted(scenes, key=lambda z: (mags.get(z, 0.0), z)):
            t, m = best_time(rows_by, ren, s), mags.get(s)
            x = xof(m) if m is not None else None
            if t and t > 0 and x is not None:
                cur.append((x, t, s, m))
            elif cur:
                segs.append(cur)
                cur = []
        if cur:
            segs.append(cur)
        if segs:
            pts[ren] = segs
    if not pts:
        return ""

    xs = [p[0] for segs in pts.values() for seg in segs for p in seg]
    ys = [p[1] for segs in pts.values() for seg in segs for p in seg]
    x0, x1 = min(xs), max(xs)
    y0, y1 = math.log10(min(ys)), math.log10(max(ys))
    # A little headroom so markers are not clipped by the frame.
    pad = (x1 - x0) * 0.04 or 1.0
    x0, x1 = x0 - pad, x1 + pad
    y0, y1 = y0 - 0.12, y1 + 0.12

    W, H = 1100, 440
    L, R, T, B = 76, 20, 18, 52
    pw, ph = W - L - R, H - T - B

    def px(v):
        return L + (v - x0) / (x1 - x0) * pw

    def py(v):
        return T + ph - (math.log10(v) - y0) / (y1 - y0) * ph

    o = ["<svg viewBox='0 0 %d %d' class='chart' role='img' "
         "aria-label='render time against magnification'>" % (W, H)]
    o.append("<rect x='0' y='0' width='%d' height='%d' fill='#0f1115'/>" % (W, H))

    # y grid: decades, plus 2 and 5 within each
    for v, m, e in nice_log_ticks(y0, y1, want=5):
        y = py(10.0 ** v)
        o.append("<line x1='%d' y1='%.1f' x2='%d' y2='%.1f' stroke='#2c313b' "
                 "stroke-width='1'/>" % (L, y, W - R, y))
        lbl = ("%g" % (m * 10 ** e)) + " s"
        o.append("<text x='%d' y='%.1f' fill='#9aa3b2' font-size='12' "
                 "text-anchor='end'>%s</text>" % (L - 8, y + 4, lbl))

    # x grid: magnifications at 1/2/5 x 10^n of the EXPONENT - 1e10, 1e20, 1e50, 1e100 ...
    # Labelled with the magnification itself so the axis reads as zoom, not as an exponent.
    for cand in (5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000):
        v = math.log10(cand)
        if not (x0 <= v <= x1):
            continue
        x = px(v)
        o.append("<line x1='%.1f' y1='%d' x2='%.1f' y2='%d' stroke='#2c313b' "
                 "stroke-width='1'/>" % (x, T, x, T + ph))
        o.append("<text x='%.1f' y='%d' fill='#9aa3b2' font-size='12' "
                 "text-anchor='middle'>1e%d</text>" % (x, H - B + 20, cand))

    o.append("<rect x='%d' y='%d' width='%d' height='%d' fill='none' stroke='#3a4150'/>"
             % (L, T, pw, ph))
    o.append("<text x='%.1f' y='%d' fill='#9aa3b2' font-size='13' text-anchor='middle'>"
             "magnification (log scale)</text>" % (L + pw / 2.0, H - 8))
    o.append("<text x='16' y='%.1f' fill='#9aa3b2' font-size='13' text-anchor='middle' "
             "transform='rotate(-90 16 %.1f)'>wall-clock seconds (log scale)</text>"
             % (T + ph / 2.0, T + ph / 2.0))

    for ren, segs in pts.items():
        c = LANE_COLOR.get(ren, "#cccccc")
        for seg in segs:
            if len(seg) > 1:
                d = " ".join(("%s%.1f,%.1f" % ("M" if i == 0 else "L", px(x), py(t)))
                             for i, (x, t, _s, _m) in enumerate(seg))
                o.append("<path d='%s' fill='none' stroke='%s' stroke-width='2' "
                         "stroke-linejoin='round'/>" % (d, c))
        for x, t, s, m in [p for seg in segs for p in seg]:
            o.append("<circle cx='%.1f' cy='%.1f' r='3.5' fill='%s'><title>%s &#183; %s "
                     "&#183; %s&#215; &#183; %.1f s</title></circle>"
                     % (px(x), py(t), c, esc(label.get(ren, ren)), esc(s), esc(fmt_mag(m)), t))

    # Legend, inside the plot so it survives being cropped into a screenshot.
    lx, ly = L + 14, T + 16
    for ren in pts:
        c = LANE_COLOR.get(ren, "#cccccc")
        o.append("<line x1='%d' y1='%d' x2='%d' y2='%d' stroke='%s' stroke-width='2'/>"
                 % (lx, ly, lx + 22, ly, c))
        o.append("<circle cx='%d' cy='%d' r='3.5' fill='%s'/>" % (lx + 11, ly, c))
        o.append("<text x='%d' y='%d' fill='#e7e9ee' font-size='13'>%s</text>"
                 % (lx + 30, ly + 4, esc(label.get(ren, ren))))
        ly += 19
    o.append("</svg>")
    return "".join(o)


def guess_output(d, renderer, scene):
    """Find a render on disk when no manifest record names one.

    Results folders written before the manifest existed still deserve a report, and a lane that
    forgets to file a record should cost a command line, not the picture. These are the naming
    conventions the lanes use; the first one that exists wins.
    """
    p = scene.replace(".", "p")
    cands = {
        "fractadyne": ["fd-%s.png" % scene],
        "fraktaler3": ["%s-f3.png" % scene],
        "fractalshark": ["fs-%s-r1.png" % p, "fs-%s.png" % p, "fs-%s.png" % scene],
        "imagina": ["im-%s.png" % scene, "%s-imagina.png" % scene],
    }.get(renderer, [])
    for c in cands:
        if os.path.exists(os.path.join(d, c)):
            return c
    return None


def best_time(rows_by, renderer, scene):
    rs = rows_by.get((renderer, scene), [])
    ts = [fnum(r["wall_s"]) for r in rs if r["status"] == "ok" and fnum(r["wall_s"])]
    return min(ts) if ts else None


def rep_stats(rows_by, renderer, scene):
    """(n, fastest, median, slowest) over the ok reps' wall_s, or None. The tables lead with the
    fastest rep (the published protocol); this is what says how steady that figure is."""
    ts = sorted(t for t in (fnum(r["wall_s"]) for r in rows_by.get((renderer, scene), [])
                            if r["status"] == "ok") if t)
    if not ts:
        return None
    n = len(ts)
    med = ts[n // 2] if n % 2 else (ts[n // 2 - 1] + ts[n // 2]) / 2.0
    return n, ts[0], med, ts[-1]


def median_of(vals):
    v = sorted(x for x in (fnum(y) for y in vals) if x is not None)
    if not v:
        return None
    n = len(v)
    return v[n // 2] if n % 2 else (v[n // 2 - 1] + v[n // 2]) / 2.0


# Where a Fractadyne render's wall went, in the order the phases happen. The kit parses each
# render's kept log into fd-phases.csv (bench-lib.ps1 Read-FdPhases documents every column);
# render = ref_wait + gpu_iterate + gpu_color + cpu_other by construction, so these segments
# tile the wall up to the few ms between the exit line and the last log line.
PHASES = [("startup_ms", "startup", "#6c7a96"), ("ref_wait_ms", "ref wait", "#e3b341"),
          ("gpu_iterate_ms", "GPU iterate", "#5fd08a"), ("gpu_color_ms", "GPU color", "#2e9e5b"),
          ("cpu_other_ms", "host other", "#ff7b72"), ("write_ms", "PNG write", "#79b8ff"),
          ("exit_ms", "exit", "#b392f0"), ("outside_ms", "outside the log", "#3d4452")]


def phase_section(ph_rows, scenes, mags, sorted_by_depth):
    """Stacked bar + table of the median phase times per scene; '' when there is nothing."""
    ok = [r for r in ph_rows if r.get("status") == "ok"]
    by = defaultdict(list)
    for r in ok:
        by[r["scene"]].append(r)
    todo = [s for s in scenes if s in by]
    if not todo:
        return ""
    med = {s: {k: median_of(r.get(k) for r in by[s]) for k in
               [p[0] for p in PHASES] + ["wall_ms", "pick_ms", "orbit_ms", "sa_ms", "bla_ms",
                                          "iters_per_step", "df32_pct", "step_executed", "mode"]}
           for s in todo}
    top = max((med[s]["wall_ms"] or 0.0) for s in todo) or 1.0
    o = []
    o.append("<h2>Fractadyne: where the wall went</h2>")
    o.append("<p class='sub'>Median over the ok reps, parsed from each render's own log "
             "(<code>fd-logs/</code>; every rep is in <code>fd-phases.csv</code>). The reference "
             "build starts at launch BESIDE startup, so only the part the render had to wait for "
             "shows as <i>ref wait</i>; its own pick/orbit/SA/BLA cost is in the table. "
             "<i>Host other</i> = render - ref wait - GPU: readback, normalization, tiling. "
             "<i>Outside the log</i> = OS launch, DLL load and teardown. Builds before b149 lack "
             "GPU iterate on the normalized path and the PNG write.</p>")
    o.append("<div class='kv'>%s</div>" % " &nbsp; ".join(
        "<span style='display:inline-block;width:10px;height:10px;background:%s'></span> %s"
        % (c, esc(n)) for _, n, c in PHASES))
    for s in todo:
        m = med[s]
        wall = m["wall_ms"] or 0.0
        segs = []
        for k, n, c in PHASES:
            v = max(m[k] or 0.0, 0.0)
            if v > 0:
                segs.append("<div title='%s: %.0f ms' style='width:%.3f%%;background:%s'></div>"
                            % (esc(n), v, 100.0 * v / top, c))
        o.append("<div style='margin:8px 0 2px;font-size:13px'>%s%s &nbsp;<span class='dim'>"
                 "%s ms</span></div>" % (esc(s), (" <span class='dim'>%s&times;</span>"
                 % esc(fmt_mag(mags.get(s)))) if sorted_by_depth else "", "{:,.0f}".format(wall)))
        o.append("<div style='display:flex;height:16px;background:#0f1115;border:1px solid "
                 "var(--line);border-radius:3px;overflow:hidden'>%s</div>" % "".join(segs))
    o.append("<table><tr><th>Scene</th><th>Wall</th>%s<th>Ref build: pick / orbit / SA / BLA</th>"
             "<th>Iterations per step</th><th>df32</th></tr>"
             % "".join("<th>%s</th>" % esc(n) for _, n, _ in PHASES))
    f0 = lambda v: "-" if v is None else "{:,.0f}".format(v)
    for s in todo:
        m = med[s]
        o.append("<tr><td>%s</td><td class='n'>%s</td>%s<td class='n'>%s</td><td class='n'>%s</td>"
                 "<td class='n'>%s</td></tr>"
                 % (esc(s), f0(m["wall_ms"]),
                    "".join("<td class='n'>%s</td>" % ("-" if m[k] is None else
                            ("%.1f" % m[k] if k == "gpu_color_ms" else f0(m[k])))
                            for k, _, _ in PHASES),
                    " / ".join(f0(m[k]) for k in ("pick_ms", "orbit_ms", "sa_ms", "bla_ms")),
                    "-" if m["iters_per_step"] is None else "%.1f" % m["iters_per_step"],
                    "-" if m["df32_pct"] is None else "%.1f%%" % m["df32_pct"]))
    o.append("</table>")
    return "".join(o)


def status_of(rows_by, renderer, scene):
    rs = rows_by.get((renderer, scene), [])
    if not rs:
        return None
    for r in rs:
        if r["status"] == "ok":
            return "ok"
    return rs[0]["status"]


# ---------------------------------------------------------------- html

CSS = """
:root{--bg:#14161a;--panel:#1c1f26;--line:#2c313b;--fg:#e7e9ee;--dim:#9aa3b2;
      --ok:#5fd08a;--bad:#ff7b72;--warn:#e3b341;--link:#79b8ff;}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--fg);
     font:15px/1.55 -apple-system,Segoe UI,Roboto,Helvetica,Arial,sans-serif}
.wrap{max-width:1180px;margin:0 auto;padding:28px 20px 80px}
h1{font-size:26px;margin:0 0 4px} h2{font-size:20px;margin:36px 0 12px;
   border-bottom:1px solid var(--line);padding-bottom:6px} h3{font-size:16px;margin:0 0 8px}
.sub{color:var(--dim);margin:0 0 20px}
table{border-collapse:collapse;width:100%;margin:10px 0 18px;font-size:14px}
th,td{border:1px solid var(--line);padding:6px 9px;text-align:left;vertical-align:top}
th{background:#232833;font-weight:600} td.n{text-align:right;font-variant-numeric:tabular-nums}
.win{color:var(--ok);font-weight:700} .dnf{color:var(--bad)} .dim{color:var(--dim)}
.panel{background:var(--panel);border:1px solid var(--line);border-radius:8px;
       padding:14px 16px;margin:0 0 14px}
.find{display:grid;grid-template-columns:230px 1fr;gap:6px 14px}
.find dt{color:var(--warn);font-weight:600} .find dd{margin:0}
.cards{display:flex;flex-wrap:wrap;gap:14px;align-items:flex-start}
.card{background:var(--panel);border:1px solid var(--line);border-radius:8px;
      padding:10px;width:340px;flex:0 0 auto}
.card img{width:100%;height:auto;border-radius:4px;display:block;background:#000}
.card .cap{font-size:13px;margin:8px 0 4px;display:flex;justify-content:space-between;gap:8px}
/* WRAP, do not scroll. These lines are the reason the report exists: a 200-digit centre and a
   full argument line must be READABLE, and inside a table cell an overflow-x box is simply cut
   off at the page edge with no scrollbar the reader will ever find. */
pre{background:#0f1115;border:1px solid var(--line);border-radius:6px;padding:9px 11px;
    white-space:pre-wrap;overflow-wrap:anywhere;word-break:break-all;
    font:12px/1.45 ui-monospace,Consolas,Menlo,monospace;color:#cfd6e4;margin:6px 0}
a{color:var(--link)} .kv{font-size:12.5px;color:var(--dim);overflow-wrap:anywhere}
.kv b{color:var(--fg);font-weight:600}
th.nw{white-space:nowrap}
/* Fixed layout so one 200-digit coordinate cannot widen the whole table past the page. */
table.cmds{table-layout:fixed} table.cmds td{overflow-wrap:anywhere}
details{margin:8px 0} summary{cursor:pointer;color:var(--link);font-size:13px}
.note{color:var(--warn);font-size:13px;margin:4px 0}
.missing{color:var(--dim);font-style:italic;padding:28px 10px;text-align:center;
         border:1px dashed var(--line);border-radius:4px}
.chart{width:100%;height:auto;border:1px solid var(--line);border-radius:8px;margin:6px 0 14px;
       background:#0f1115}
"""


def esc(x):
    return html.escape(str(x if x is not None else ""))


def cmd_block(rec):
    """The exact invocation, and the inputs it encodes. This is the point of the report."""
    exe = rec.get("exe") or ""
    args = rec.get("args") or ""
    line = '"%s" %s' % (exe, args) if exe else args
    out = ["<pre>%s</pre>" % esc(line)]
    inp = rec.get("inputs") or {}
    if isinstance(inp, dict) and inp:
        bits = []
        for k in sorted(inp):
            v = inp[k]
            if v is None or v == "":
                continue
            bits.append("<b>%s</b> %s" % (esc(k), esc(v)))
        if bits:
            out.append('<div class="kv">%s</div>' % " &nbsp;| &nbsp;".join(bits))
    src = rec.get("source")
    if src:
        out.append('<div class="kv">from <b>%s</b></div>' % esc(src))
    if rec.get("cwd"):
        out.append('<div class="kv">cwd <b>%s</b></div>' % esc(rec["cwd"]))
    return "".join(out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("results_dir")
    ap.add_argument("--no-thumbs", action="store_true")
    ap.add_argument("--self-contained", action="store_true",
                    help="inline the thumbnails and write report-standalone.html, so the page "
                         "can be sent on its own without the 4K PNGs beside it")
    ap.add_argument("--note", default="",
                    help="a short line of HTML shown under the title. For a published report, "
                         "this is where you say what this run is and link a companion run; a "
                         "page someone reaches by a bare URL has no other context.")
    ap.add_argument("--noindex", action="store_true",
                    help="add a robots noindex/nofollow meta. For an unlisted page shared by "
                         "URL: unlinked is not the same as unindexed, and a robots.txt rule "
                         "would publish the very path you are trying not to advertise.")
    ap.add_argument("--strip-prefix", action="append", default=[], metavar="DIR",
                    help="cut this directory prefix off any path in the page, leaving the rest "
                         "relative, so a published report reads 'bench-kit\\apps\\...' instead "
                         "of exposing the whole tree above the checkout. Repeatable.")
    ap.add_argument("--redact-home", action="store_true",
                    help="replace your user-profile path with a placeholder. Use it for any "
                         "report that leaves the machine: a Windows profile directory is your "
                         "ACCOUNT NAME, often your real name, and the command lines repeat it "
                         "hundreds of times. The paths stay readable and the arguments stay "
                         "complete.")
    args = ap.parse_args()
    d = os.path.abspath(args.results_dir)
    if not os.path.isdir(d):
        sys.stderr.write("make-report: no such directory: %s\n" % d)
        return 2

    manifest = load_json(os.path.join(d, "run-manifest.json"), {}) or {}
    meta = manifest.get("meta", {}) or {}
    runs = manifest.get("runs", []) or []
    rows = load_csv(os.path.join(d, "results.csv"))
    # One view-check file per automated third-party lane. `view` stays the FractalShark one so
    # the findings read as before; `views` is what the per-scene badges use.
    views = {"fractalshark": load_json(os.path.join(d, "fs-view-check.json"), {}) or {},
             "imagina": load_json(os.path.join(d, "im-view-check.json"), {}) or {}}
    view = views["fractalshark"]
    sysinfo = load_text(os.path.join(d, "sysinfo.txt"))
    zs_rows = load_csv(os.path.join(d, "zoomseq", "results.csv"))

    if not rows and not runs:
        sys.stderr.write("make-report: nothing to report in %s\n" % d)
        return 2

    rows_by = defaultdict(list)
    for r in rows:
        rows_by[(r["renderer"], r["scene"])].append(r)

    # Prefer the scene order the run used; fall back to whatever the CSV holds.
    scenes = meta.get("scenes") or []
    if not scenes:
        seen = []
        for r in rows:
            if r["scene"] not in seen and not r["scene"].startswith("zoomseq"):
                seen.append(r["scene"])
        scenes = seen

    runs_by = defaultdict(list)
    for rec in runs:
        runs_by[(rec.get("renderer"), rec.get("scene"))].append(rec)

    # SORT BY DEPTH, not by filename. scenes.csv is ordered by scene id, so the default order
    # puts 1e1105 above 1e27.7, and a benchmark whose entire axis is magnification should not be
    # read in an order that hides it. Only sort when the magnification is known for EVERY scene:
    # a partially sorted table is harder to trust than an unsorted one, because the reader cannot
    # tell which rows were placed and which were left. Execution order is untouched and still
    # shown, in order, by the command table at the bottom.
    mags = scene_magnitudes(meta, runs_by, scenes)
    sorted_by_depth = len(mags) == len(scenes) and len(scenes) > 0
    if sorted_by_depth:
        # Slug is the tiebreak: two scenes sit at exactly 5.1e27, and an unstable order would
        # make a diff between two reports of the same data look like a change.
        scenes = sorted(scenes, key=lambda s: (mags[s], s))

    thumbs = os.path.join(d, "thumbs")
    renderers = ["fractadyne", "fraktaler3", "fractalshark", "imagina"]
    label = {"fractadyne": "Fractadyne", "fraktaler3": "Fraktaler-3",
             "fractalshark": "FractalShark", "imagina": "Imagina"}

    # Folders written before the manifest existed still carry host and timestamp in their NAME
    # ("HOST-20260921-123041"). Recovering them there beats printing "?" at the reader.
    base = os.path.basename(d.rstrip("\\/"))
    host, stamp = meta.get("host"), meta.get("stamp")
    if not host or not stamp:
        parts = base.split("-")
        if len(parts) >= 3:
            host = host or parts[0]
            stamp = stamp or "-".join(parts[1:])

    H = []
    a = H.append
    title = "Benchmark report - %s" % " - ".join(x for x in (host, stamp) if x) or base
    a("<!doctype html><html><head><meta charset='utf-8'>")
    a("<meta name='viewport' content='width=device-width,initial-scale=1'>")
    if args.noindex:
        a("<meta name='robots' content='noindex,nofollow,noarchive'>")
        a("<meta name='referrer' content='no-referrer'>")
    a("<title>%s</title><style>%s</style></head><body><div class='wrap'>" % (esc(title), CSS))
    a("<h1>%s</h1>" % esc(title))
    # Say only what is known. A subtitle full of question marks tells the reader nothing except
    # that the report could not be bothered to check.
    bits = ["%d scenes" % len(scenes)]
    if meta.get("size"):
        bits.append("%s, one sample per pixel" % meta["size"])
    if meta.get("reps"):
        bits.append("%s rep(s)" % meta["reps"])
    if meta.get("lanes"):
        bits.append("lanes: %s" % ", ".join(meta["lanes"]))
    if not meta:
        bits.append("no run manifest in this folder, so command lines are unavailable "
                    "and images were matched by filename")
    a("<p class='sub'>%s.</p>" % esc(". ".join(bits)))
    # Deliberately NOT escaped: this is the author's own HTML, passed on the command line, and
    # its whole purpose is to carry a link.
    if args.note:
        a("<div class='panel'>%s</div>" % args.note)

    # ---- findings
    a("<h2>Findings</h2>")
    findings = derive_findings(scenes, rows_by, views, meta, zs_rows)
    if findings:
        a("<div class='panel'><dl class='find'>")
        for k, v in findings:
            a("<dt>%s</dt><dd>%s</dd>" % (esc(k), esc(v)))
        a("</dl></div>")
    a("<p class='note'>Every number below is wall-clock, process start to exit, fastest of the "
      "repeats; where a scene ran more than once, the median, the spread (slowest - fastest, "
      "over the median) and the rep count sit under it. A DNF is a result, not a gap: it never "
      "competes for 'fastest'.</p>")

    # ---- results table
    a("<h2>Results</h2>")
    a("<p class='sub'>%s</p>"
      % ("Shallowest first. The scene numbers are ids, not magnitudes, so filename order would "
         "put 1e1105 above 1e27.7."
         if sorted_by_depth
         else "In the order the run executed them: the magnification was not recorded for every "
              "scene in this folder, and a partly sorted table is harder to trust than an "
              "unsorted one."))
    live = [ren for ren in renderers if any((ren, s) in rows_by for s in scenes)]
    a("<table><tr><th>Scene</th>")
    if sorted_by_depth:
        a("<th class='nw'>Magnification</th>")
    for ren in live:
        a("<th>%s</th>" % esc(label[ren]))
    a("</tr>")
    for s in scenes:
        a("<tr><td>%s</td>" % esc(s))
        if sorted_by_depth:
            a("<td class='n dim'>%s</td>" % esc(fmt_mag(mags.get(s))))
        times = {ren: best_time(rows_by, ren, s) for ren in live}
        fastest = min((t for t in times.values() if t is not None), default=None)
        for ren in live:
            t, st = times[ren], status_of(rows_by, ren, s)
            if t is None:
                a("<td class='dnf'>%s</td>" % esc(st or "-"))
            else:
                cls = "n win" if t == fastest else "n"
                st4 = rep_stats(rows_by, ren, s)
                spread = ""
                if st4 and st4[0] > 1:
                    n, lo, med, hi = st4
                    spread = ("<div class='dim' style='font-size:12px;font-weight:400'>"
                              "med %.2f &middot; spread %.0f%% &middot; n=%d</div>"
                              % (med, 100.0 * (hi - lo) / med if med else 0.0, n))
                a("<td class='%s'>%.1f s%s</td>" % (cls, t, spread))
        a("</tr>")
    a("</table>")

    # ---- chart: the shape of the table above
    if sorted_by_depth:
        svg = svg_time_vs_depth(scenes, mags, rows_by, live, label)
        if svg:
            a("<h2>Render time against magnification</h2>")
            a("<p class='sub'>Both axes are logarithmic, and the magnification axis is spaced by "
              "the order of magnitude of its own exponent. It has to be: the corpus samples 1e6 "
              "to 1e77 densely and then jumps to 1e1105, so plotting the exponent linearly piles "
              "seven of the ten scenes into the first tenth of the width and stretches two "
              "straight segments across 800 orders of magnitude nobody measured. Read the "
              "ordering and the crossovers, not the slopes, which are not rates on either axis. "
              "A gap in a line is a scene that renderer did not complete, and the points are not "
              "joined across it, because a renderer that produced no picture has no time. Hover "
              "a point for its scene, magnification and figure.</p>")
            a(svg)

    # ---- Fractadyne phases (fd-phases.csv; absent in folders from before the kit kept logs)
    a(phase_section(load_csv(os.path.join(d, "fd-phases.csv")), scenes, mags, sorted_by_depth))

    # ---- per scene: images + the commands that made them
    a("<h2>Per scene: what was run, and what came out</h2>")
    if sorted_by_depth:
        a("<p class='sub'>Shallowest first, matching the table above.</p>")
    a("<p class='sub'>%sThe command line under each is the one that produced that file.</p>"
      % ("" if args.self_contained
         else "Click any image for the full-resolution render. "))
    for s in scenes:
        a("<h3>%s%s</h3>"
          % (esc(s),
             (" <span class='dim'>&nbsp;%s&times;</span>" % esc(fmt_mag(mags.get(s))))
             if sorted_by_depth else ""))
        a("<div class='cards'>")
        for ren in live:
            recs = runs_by.get((ren, s), [])
            rec = recs[0] if recs else {}
            out = rec.get("output") or guess_output(d, ren, s)
            src = os.path.join(d, out) if out else None
            a("<div class='card'>")
            if src and os.path.exists(src):
                rel = None
                if not args.no_thumbs:
                    rel = make_thumb(src, thumbs, "%s-%s.jpg" % (ren, s.replace(".", "p")))
                shown = rel or out
                a("<a href='%s'><img src='%s' alt='%s'></a>" % (esc(out), esc(shown), esc(ren)))
            else:
                a("<div class='missing'>no image on disk</div>")
            t = best_time(rows_by, ren, s)
            st = status_of(rows_by, ren, s)
            right = ("%.1f s" % t) if t is not None else ("<span class='dnf'>%s</span>" % esc(st))
            a("<div class='cap'><b>%s</b><span>%s</span></div>" % (esc(label[ren]), right))
            if s in views.get(ren, {}):
                v = views[ren][s]
                ok = v.get("same_view")
                mag_ok = v.get("magnification_ok", True)
                if ok:
                    word, cls = "depicts this scene", "kv"
                elif mag_ok:
                    # Say what is actually known: the magnification agrees and the pictures do
                    # not match. Not "does not depict this scene" - that is a different, much
                    # stronger claim, and it was wrong the one time this report made it.
                    word, cls = ("right magnification; could not confirm by image "
                                 "(usually a palette difference, not a fault)"), "kv"
                else:
                    word, cls = "a DIFFERENT magnification than this scene asks for", "note"
                a("<div class='%s'>view check: %s (edge r %.3f, best wrong %.3f)</div>"
                  % (cls, word, v.get("edge_r", 0.0), v.get("best_wrong_r", 0.0)))
            note = rec.get("note")
            if note:
                a("<div class='note'>%s</div>" % esc(note))
            if rec:
                a("<details><summary>command and inputs</summary>%s</details>" % cmd_block(rec))
                if len(recs) > 1:
                    a("<div class='kv'>%d repeats, identical arguments</div>" % len(recs))
            a("</div>")
        a("</div>")

        # Fraktaler-3 takes its view from a file, so the arguments alone do not describe it.
        toml = os.path.join(d, s + ".bench.f3.toml")
        if os.path.exists(toml):
            a("<details><summary>Fraktaler-3 input file: %s.bench.f3.toml</summary><pre>%s</pre></details>"
              % (esc(s), esc(load_text(toml, 4000))))

    # ---- any other images the run produced
    scene_outputs = {rec.get("output") for rec in runs if rec.get("output")}
    others = []
    for root, _dirs, files in os.walk(d):
        if os.path.basename(root) == "thumbs":
            continue
        for f in sorted(files):
            if not f.lower().endswith((".png", ".jpg")):
                continue
            rel = os.path.relpath(os.path.join(root, f), d).replace("\\", "/")
            if rel in scene_outputs or f in scene_outputs:
                continue
            others.append(rel)
    if others:
        a("<h2>Other images produced by this run</h2>")
        a("<p class='sub'>%d file(s) not tied to a single scene row, for example zoom-sequence "
          "frames.</p><div class='cards'>" % len(others))
        for rel in others:
            src = os.path.join(d, rel)
            th = None if args.no_thumbs else make_thumb(
                src, thumbs, "other-" + rel.replace("/", "-").replace(".", "p") + ".jpg")
            a("<div class='card'>")
            a("<a href='%s'><img src='%s' alt='%s'></a>" % (esc(rel), esc(th or rel), esc(rel)))
            a("<div class='cap'><b>%s</b></div></div>" % esc(rel))
        a("</div>")

    # ---- zoom sequence
    if zs_rows:
        a("<h2>Zoom sequence</h2>")
        a("<p class='sub'>Every frame is the same picture at a different scale, so per-frame cost "
          "should be flat; a ramp is the renderer failing to reuse its setup. The ratio is each "
          "app against itself, so it needs no cross-app calibration.</p>")
        a("<table><tr><th>Renderer</th><th>Scene</th><th>Status</th><th>Wall s</th>"
          "<th>Reported</th><th>Note</th></tr>")
        for r in zs_rows:
            a("<tr><td>%s</td><td>%s</td><td>%s</td><td class='n'>%s</td><td class='n'>%s</td>"
              "<td class='dim'>%s</td></tr>"
              % (esc(r.get("renderer")), esc(r.get("scene")), esc(r.get("status")),
                 esc(r.get("wall_s")), esc(r.get("reported_s")), esc(r.get("note"))))
        a("</table>")

    # ---- every command executed
    a("<h2>Every command this run executed</h2>")
    a("<p class='sub'>In EXECUTION order, which is not the depth order used above: the scenes ran "
      "as scenes.csv lists them, and changing that would change the measurement rather than just "
      "its presentation. Includes one-off setup such as Fraktaler-3's hardware tuning and the "
      "FractalShark server. This is the record that lets someone else reproduce or challenge a "
      "number.</p>")
    a("<table class='cmds'><colgroup><col style='width:9%'><col style='width:15%'>"
      "<col style='width:5%'><col style='width:11%'><col style='width:7%'>"
      "<col style='width:53%'></colgroup>"
      "<tr><th>Renderer</th><th>Scene</th><th class='nw'>Rep</th><th>Status</th>"
      "<th class='nw'>Wall s</th><th>Command</th></tr>")
    for rec in runs:
        st = rec.get("status", "")
        cls = "" if st == "ok" else "dnf"
        a("<tr><td>%s</td><td>%s</td><td class='n'>%s</td><td class='%s'>%s</td>"
          "<td class='n'>%s</td><td>%s</td></tr>"
          % (esc(rec.get("renderer")), esc(rec.get("scene")), esc(rec.get("rep")),
             cls, esc(st), esc(rec.get("wall_s")), cmd_block(rec)))
    a("</table>")

    # ---- provenance
    a("<h2>What produced these numbers</h2>")
    exes = meta.get("exes") or {}
    if any(exes.values()):
        a("<table><tr><th>Renderer</th><th>Binary</th><th>Bytes</th><th>sha256</th></tr>")
        for k in sorted(exes):
            e = exes[k]
            if not e:
                continue
            a("<tr><td>%s</td><td class='dim'>%s</td><td class='n'>%s</td>"
              "<td class='dim'><code>%s</code></td></tr>"
              % (esc(label.get(k, k)), esc(e.get("path")), esc(e.get("bytes")),
                 esc((e.get("sha256") or "")[:32])))
        a("</table>")
    if sysinfo:
        a("<details open><summary>sysinfo.txt</summary><pre>%s</pre></details>" % esc(sysinfo))
    # Only link what is actually there: a report that offers a dead link is claiming evidence it
    # does not have.
    raw = [f for f in ("results.csv", "run-manifest.json", "fs-view-check.json", "summary.md",
                       "sysinfo.txt", "f3-wisdom.toml")
           if os.path.exists(os.path.join(d, f))]
    if raw:
        a("<p class='sub'>Raw data beside this file: %s.</p>"
          % ", ".join("<a href='%s'>%s</a>" % (f, f) for f in raw))

    a("</div></body></html>")

    doc = "\n".join(H)

    # Prefix stripping first, then the home path: cutting the checkout root turns an absolute
    # path into the relative one a reader actually needs, and whatever is left that still sits
    # under the profile directory is caught by the pass below. Longest prefix first so a nested
    # root goes before its parent.
    for pref in sorted(args.strip_prefix, key=len, reverse=True):
        p = pref.rstrip("\\/")
        n = 0
        for old in (p + "\\", p.replace("\\", "/") + "/", p.replace("\\", "\\\\") + "\\\\"):
            n += doc.count(old)
            doc = doc.replace(old, "")
        print("stripped %d occurrence(s) of the prefix %s" % (n, p))

    # Redaction is textual and runs LAST, over the finished document, so no section can be
    # missed by forgetting to route its strings through a helper. Both slash spellings, because
    # a path can arrive either way depending on which tool printed it.
    if args.redact_home:
        home = os.path.expanduser("~")
        subs = [(home, "C:\\Users\\<user>"),
                (home.replace("\\", "/"), "C:/Users/<user>")]
        hits = 0
        for old, new in subs:
            if not old:
                continue
            hits += doc.count(old)
            doc = doc.replace(old, new)
            # Case matters on paths that different tools capitalised differently.
            if old.lower() != old:
                hits += doc.count(old.lower())
                doc = doc.replace(old.lower(), new)
        leak = os.path.basename(home)
        remaining = doc.lower().count(leak.lower()) if leak else 0
        print("redacted %d occurrence(s) of the home path" % hits)
        if remaining:
            # Say so rather than let the author assume it is clean. A redaction you cannot
            # verify is worse than none, because it buys confidence it has not earned.
            print("WARNING: %r still appears %d time(s) - check before publishing"
                  % (leak, remaining))

    # A single file someone can attach to an email. The full-size PNGs are hundreds of megabytes
    # and will not travel with it, so the thumbnails become data URIs and the links that would
    # 404 are stripped: a page that offers a dead link is worse than one that offers none.
    if args.self_contained:
        import base64
        import re as _re

        def inline(m):
            rel = m.group(1)
            p = os.path.join(d, rel.replace("/", os.sep))
            if not os.path.exists(p):
                return m.group(0)
            with open(p, "rb") as fh:
                b = base64.b64encode(fh.read()).decode("ascii")
            kind = "jpeg" if rel.lower().endswith((".jpg", ".jpeg")) else "png"
            return "src='data:image/%s;base64,%s'" % (kind, b)

        doc = _re.sub(r"src='([^']+\.(?:jpg|jpeg|png))'", inline, doc, flags=_re.I)
        doc = _re.sub(r"<a href='[^']+\.(?:png|jpg|jpeg)'>(.*?)</a>", r"\1", doc, flags=_re.I | _re.S)
        out = os.path.join(d, "report-standalone.html")
        with open(out, "w", encoding="utf-8") as fh:
            fh.write(doc)
        print("standalone report: %s (%.1f MB)" % (out, os.path.getsize(out) / 1e6))
        return 0

    out = os.path.join(d, "report.html")
    with open(out, "w", encoding="utf-8") as fh:
        fh.write(doc)
    if not HAVE_PIL and not args.no_thumbs:
        print("Pillow not installed: full-size images linked instead of thumbnailed.")
    print("report: %s" % out)
    return 0


if __name__ == "__main__":
    sys.exit(main())
