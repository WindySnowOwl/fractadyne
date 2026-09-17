#!/usr/bin/env python3
"""Analyse one or two `fractadyne --zoomtest` JSON logs.

    python scripts/zoomtest_report.py logs/zoomtest-A.json [logs/zoomtest-B.json] [--worst N]

Prints the run summary, a frame-interval histogram, and the WORST stalls with their context —
where in the dive (1e{depth}), whether the frame shown was a real re-iterate or a held
reprojection, the reference-pipeline depth lag, and whether a reference install landed on that
frame (the usual suspect for a hitch). With two files it prints the summaries side by side, so a
before/after (or lookahead on/off via FRACTADYNE_NO_PREFETCH=1) can be read in one screen.
Standard library only.

Reading `held oct` / `gap_oct`: the octaves by which the frame ON SCREEN lags the view (the
display magnifies the last complete frame by 2^gap). It applies to `real` frames too: a pinned
refresh (mode 2, from the floatexp hand-over at ~1e28) re-iterates every frame but the display
keeps serving the snapshot of the last complete frame until the pin adopts, and every reference
install aborts the pin. At a fast zoom rate the installs starve the pin, so the hand-over frame
stays on screen while it is magnified 20-50x (the 1e28-1e29.5 band at 4.0x).
"""
import json
import math
import os
import sys


def load(path):
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def summary_rows(d):
    s = d["summary"]
    return [
        ("octaves / secs", f"{d['octaves']:.1f} / {d['secs']:.1f}"),
        ("oct/s · fps", f"{s['oct_s']:.3f} · {s['fps']:.1f}"),
        ("zoom rate · prefetch", f"{d['zoom_rate']:.2f}x · {'on' if d['prefetch'] else 'OFF'}"),
        ("frames", f"{s['frames']}"),
        ("interval mean/p50", f"{s['dt_mean']:.1f} / {s['dt_p50']:.1f} ms"),
        ("interval p95/p99", f"{s['dt_p95']:.1f} / {s['dt_p99']:.1f} ms"),
        ("interval max", f"{s['dt_max']:.1f} ms"),
        (">33 / >50 / >100 ms", f"{s['gt33']} / {s['gt50']} / {s['gt100']}"),
        ("longest stall", f"{s['longest_ms']:.0f} ms @ t={s['longest_t']:.1f}s 1e{s['longest_l2'] / math.log2(10):.1f}"),
        ("real refreshes", f"{s['reals']} ({100 - s['hold_pct']:.0f}% of frames)"),
        ("real interval mean/p95/max", f"{s['real_gap_mean']:.0f} / {s['real_gap_p95']:.0f} / {s['real_gap_max']:.0f} ms"),
        ("held frame max", f"{s['gap_oct_max']:.2f} oct ({2 ** s['gap_oct_max']:.2f}x)"),
        ("zoom step p95/max", f"{s['step_p95']:.4f} / {s['step_max']:.3f} oct"),
        ("installs (lookahead)", f"{s['installs']} ({s['lookahead']})"),
        ("depth lag max", f"{s['lag_max']:.2f}"),
        ("pacer throttled", f"{s['paced_pct']:.1f}% of frames"),
    ]


def histogram(frames, edges=(0, 17, 20, 25, 33, 50, 100, 200, 500, 1e9)):
    counts = [0] * (len(edges) - 1)
    for fr in frames:
        dt = fr["dt_ms"]
        for i in range(len(edges) - 1):
            if edges[i] <= dt < edges[i + 1]:
                counts[i] += 1
                break
    total = max(1, len(frames))
    out = []
    for i, c in enumerate(counts):
        hi = "∞" if edges[i + 1] >= 1e9 else f"{edges[i + 1]:.0f}"
        bar = "#" * int(round(60 * c / total))
        out.append(f"  {edges[i]:>4.0f}–{hi:<4} ms {c:>6} {100 * c / total:5.1f}% {bar}")
    return "\n".join(out)


BAND_EDGES = (0, 2, 4, 6, 8, 12, 20, 40, 70, 100, 150, 250, 1e9)  # log10 magnification


def bands(frames):
    """Per-depth-band cadence: WHERE in a dive the stutter lives (a 1x -> 1e100 dive crosses the
    direct -> df32 -> floatexp hand-overs, and each regime has its own failure shape)."""
    lg = math.log2(10)
    rows = [f"  {'band (1e)':>10} {'frames':>6} {'secs':>6} {'mean':>6} {'p95':>6} {'max':>7} {'>33':>4} {'>100':>5} {'held%':>5} {'held oct max':>12} {'installs':>8} {'res w':>6} {'oct/s':>5}"]
    for i in range(len(BAND_EDGES) - 1):
        lo, hi = BAND_EDGES[i], BAND_EDGES[i + 1]
        sel = [fr for fr in frames if lo <= fr["l2"] / lg < hi]
        if not sel:
            continue
        dts = sorted(fr["dt_ms"] for fr in sel)
        held = sum(1 for fr in sel if not fr["real"])
        inst = sel[-1]["orbit_id"] - sel[0]["orbit_id"]
        hi_s = "∞" if hi >= 1e9 else f"{hi:.0f}"
        # Newer logs carry the dispatched iterate width and the observed zoom speed (the inputs
        # of the rate-aware refresh sizing); older ones print a dash.
        res_w = [fr["res_px"][0] for fr in sel if "res_px" in fr]
        oct_s = [fr["oct_s"] for fr in sel if "oct_s" in fr]
        rows.append(
            f"  {f'{lo:.0f}–{hi_s}':>10} {len(sel):>6} {sum(dts) / 1000:6.1f} {sum(dts) / len(dts):6.1f} "
            f"{dts[int(len(dts) * 0.95)] if len(dts) > 1 else dts[-1]:6.1f} {dts[-1]:7.1f} "
            f"{sum(1 for d in dts if d > 33):>4} {sum(1 for d in dts if d > 100):>5} "
            f"{100 * held / len(sel):5.0f} {max(fr['gap_oct'] for fr in sel):12.2f} {inst:>8} "
            f"{(sum(res_w) / len(res_w)) if res_w else float('nan'):6.0f} {(sum(oct_s) / len(oct_s)) if oct_s else float('nan'):5.2f}"
        )
    return "\n".join(rows)


def worst(frames, n):
    idx = sorted(range(len(frames)), key=lambda i: -frames[i]["dt_ms"])[:n]
    lines = [f"  {'t s':>7} {'ms':>7} {'depth':>8} {'shown':>6} {'lag':>5} {'held oct':>8} {'vel%':>5} {'install':>8}"]
    for i in sorted(idx):
        fr = frames[i]
        prev = frames[i - 1] if i > 0 else fr
        install = "orbit" if fr["orbit_id"] != prev["orbit_id"] else ("look" if fr["look"] != prev["look"] else "")
        lines.append(
            f"  {fr['t']:7.2f} {fr['dt_ms']:7.1f} 1e{fr['l2'] / math.log2(10):6.1f} {'real' if fr['real'] else 'held':>6} "
            f"{fr['lag']:5.2f} {fr['gap_oct']:8.2f} {100 * fr['vel_frac']:5.0f} {install:>8}"
        )
    return "\n".join(lines)


def main(argv):
    files = [a for a in argv if not a.startswith("--")]
    n_worst = 12
    if "--worst" in argv:
        n_worst = int(argv[argv.index("--worst") + 1])
        files = [f for f in files if f != argv[argv.index("--worst") + 1]]
    if not files:
        print(__doc__)
        return 2
    missing = [p for p in files if not os.path.exists(p)]
    if missing:
        # A VACUOUS run (exit 2: fewer than 120 glide frames) writes no JSON — say so instead of a
        # traceback, so a chained A/B reads as "A did not measure" rather than "the script broke".
        print("no such zoomtest log (a vacuous run, exit 2, writes none): " + ", ".join(missing))
        return 2
    runs = [load(p) for p in files]
    if len(runs) == 1:
        d = runs[0]
        print(f"{d['tool']} {d['version']} — {d['location']}")
        for k, v in summary_rows(d):
            print(f"  {k:<28} {v}")
        print("\nframe-interval histogram:")
        print(histogram(d["frames"]))
        print("\nby depth band:")
        print(bands(d["frames"]))
        print(f"\nworst {n_worst} frames:")
        print(worst(d["frames"], n_worst))
    else:
        a, b = runs[0], runs[1]
        ra, rb = summary_rows(a), summary_rows(b)
        print(f"{'':<28} {'A: ' + files[0]:<34} {'B: ' + files[1]}")
        for (k, va), (_, vb) in zip(ra, rb):
            flag = "" if va == vb else "  ◀▶"
            print(f"  {k:<26} {va:<34} {vb}{flag}")
        print("\nby depth band — A:")
        print(bands(a["frames"]))
        print("\nby depth band — B:")
        print(bands(b["frames"]))
        print(f"\nworst {n_worst} frames — A:")
        print(worst(a["frames"], n_worst))
        print(f"\nworst {n_worst} frames — B:")
        print(worst(b["frames"], n_worst))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
