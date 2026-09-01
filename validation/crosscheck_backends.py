#!/usr/bin/env python3
"""Three-way corpus cross-check: astro-float build vs MPFR (accelerated) build vs Fraktaler-3.

For every location in the comparison corpus (validation/corpus/locations.toml), this:
  * renders it with the standard build (astro-float) and the accelerated build (MPFR/rug),
    each timed, each compared DECODED-RGB (never file hash — --render embeds metadata)
    against the committed, F3-confirmed corpus render — so both backends are held to the
    same blessed pixels — and against each other;
  * runs the vendored Fraktaler-3 in batch mode on the committed .f3.toml for the location,
    timed, with the blank-output guard (F3 reports success even when it silently renders
    nothing usable — the OUTPUT FILE is the verdict, never the exit code).

Work equalization (why these timings are comparable): the corpus geometry is 1280x720 with
Fractadyne at --ss 2 (4 samples/pixel) and F3 at subframes = 4 (4 samples/pixel) — the same
sampling budget both sides, unlike the curated 4K bench table whose .f3.toml carried
subframes=4 against --ss 1. Correctness pairing with F3 itself is the corpus's standing
arm-for-arm verification; F3's own output is NOT pixel-diffed here (different coloring
pipeline, and F3 is not reproducible run to run — measured maxD 218 across re-renders).

Timing is single-run wall clock per (location, engine), machine otherwise idle, engines
strictly sequential. Run from the repo root:

    python validation/crosscheck_backends.py [--only 06,08] [--out report.md]
"""

import math
import os
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "validation", "corpus"))
import generate_corpus as gc  # noqa: E402

ASTRO = os.path.join(ROOT, "target", "release", "fractadyne.exe")
MPFR = os.path.join(ROOT, "target", "x86_64-pc-windows-gnu", "release", "fractadyne.exe")
F3 = os.path.join(ROOT, "diag", "fraktaler", "fraktaler-3-3.1.x86_64.exe")
PARAMS = os.path.join(ROOT, "validation", "corpus", "locations")
MINGW_BIN = r"C:\msys64\mingw64\bin"
BLANK_BYTES = 15000  # generate_fraktaler.py's guard: smaller PNGs are F3's silent blank
F3_TIMEOUT = 2400  # row 09 historically ~9 min


def exe_identity(exe, expect_rug):
    """Assert the binary is the build under test: version string + compiled-in backends.
    A cross-check that silently ran a stale exe (or the astro exe twice) would produce a
    clean, confident, meaningless table — assert the variable under test instead."""
    v = subprocess.run([exe, "--version"], capture_output=True, text=True, timeout=60)
    version = (v.stdout or "").strip()
    # The fd-start banner names the compiled-in backends; any cheap invocation emits it.
    b = subprocess.run(
        [exe, "--pickcheck", "definitely-not-a-file.fdn"],
        capture_output=True, text=True, timeout=120,
        env=dict(os.environ, FRACTADYNE_NO_SOUND="1"),
    )
    blob = (b.stdout or "") + (b.stderr or "")
    line = next((l for l in blob.splitlines() if "backends compiled in" in l), "")
    has_rug = "rug" in line
    if expect_rug != has_rug:
        sys.exit("%s: expected rug=%s but banner says: %s" % (exe, expect_rug, line or "<absent>"))
    return version, line.split("backends compiled in:")[-1].strip()


def f3_run(loc):
    """Timed F3 batch render of the COMMITTED param file; returns (seconds, png_bytes)."""
    param = os.path.join(PARAMS, loc["slug"] + ".f3.toml")
    if not os.path.exists(param):
        sys.exit("missing committed param %s" % param)
    out = os.path.join(PARAMS, loc["slug"] + "-f3.png")
    if os.path.exists(out):
        os.remove(out)
    t0 = time.monotonic()
    subprocess.run([F3, "-b", os.path.basename(param)], capture_output=True, text=True,
                   cwd=PARAMS, timeout=F3_TIMEOUT)
    dt = time.monotonic() - t0
    if not os.path.exists(out):
        return dt, 0
    size = os.path.getsize(out)
    os.remove(out)  # keep the committed locations dir clean
    return dt, size


def main():
    import numpy as np

    only = None
    if "--only" in sys.argv:
        only = [t.strip() for t in sys.argv[sys.argv.index("--only") + 1].split(",")]
    out_md = None
    if "--out" in sys.argv:
        out_md = sys.argv[sys.argv.index("--out") + 1]

    for exe in (ASTRO, MPFR, F3):
        if not os.path.exists(exe):
            sys.exit("missing %s" % exe)
    os.environ["PATH"] = MINGW_BIN + os.pathsep + os.environ["PATH"]  # GMP/MPFR DLLs
    os.environ["FRACTADYNE_NO_SOUND"] = "1"

    astro_id = exe_identity(ASTRO, expect_rug=False)
    mpfr_id = exe_identity(MPFR, expect_rug=True)
    print("astro: %s [%s]" % astro_id)
    print("mpfr:  %s [%s]" % mpfr_id)

    locs = gc.read_locations()
    if only:
        locs = [l for l in locs if any(l["slug"].startswith(o) for o in only)]
        if not locs:
            sys.exit("no locations match --only")

    import tempfile
    import shutil
    cfg = gc.stage_config_dir()
    tmpdir = tempfile.mkdtemp(prefix="fdxchk_")
    rows = []
    try:
        for loc in locs:
            slug = loc["slug"]
            l10 = loc["mag_log10"]
            committed = os.path.join(ROOT, "validation", "corpus", "renders",
                                     slug + "-fractadyne.png")
            ref = gc._img_rgb(committed)
            print("%-24s 1e%-7.1f iter %-8d ..." % (slug, l10, loc["iterations"]), flush=True)
            times, imgs = {}, {}
            for name, exe in (("astro", ASTRO), ("mpfr", MPFR)):
                gc.EXE = exe
                tmp = os.path.join(tmpdir, "%s-%s.png" % (slug, name))
                t0 = time.monotonic()
                gc.render_location(loc, out_png=tmp, config_dir=cfg)
                times[name] = time.monotonic() - t0
                imgs[name] = gc._img_rgb(tmp)
                os.remove(tmp)
            d_astro = int(np.abs(imgs["astro"] - ref).max())
            d_mpfr = int(np.abs(imgs["mpfr"] - ref).max())
            d_cross = int(np.abs(imgs["astro"] - imgs["mpfr"]).max())
            f3_s, f3_bytes = f3_run(loc)
            f3_ok = f3_bytes >= BLANK_BYTES
            row = dict(slug=slug, l10=l10, iters=loc["iterations"],
                       astro=times["astro"], mpfr=times["mpfr"], f3=f3_s,
                       d_astro=d_astro, d_mpfr=d_mpfr, d_cross=d_cross, f3_ok=f3_ok)
            rows.append(row)
            print("   astro %6.1fs (maxD %d)   mpfr %6.1fs (maxD %d, cross %d)   "
                  "f3 %6.1fs%s" % (times["astro"], d_astro, times["mpfr"], d_mpfr,
                                   d_cross, f3_s, "" if f3_ok else "  F3-BLANK"), flush=True)
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)
        shutil.rmtree(cfg, ignore_errors=True)

    # ---- report ----
    bad = [r for r in rows if r["d_astro"] != 0 or r["d_mpfr"] != 0 or r["d_cross"] != 0]
    blank = [r for r in rows if not r["f3_ok"]]
    lines = []
    lines.append("| # | location | zoom | iters | astro s | mpfr s | f3 s | mpfr/astro | f3/astro | f3/mpfr | pixels |")
    lines.append("|---|---|---|---|---|---|---|---|---|---|---|")
    for r in rows:
        px = "identical" if (r["d_astro"] == 0 and r["d_mpfr"] == 0) else \
            "maxD a%d m%d x%d" % (r["d_astro"], r["d_mpfr"], r["d_cross"])
        f3c = "%.1f" % r["f3"] if r["f3_ok"] else "BLANK"
        rr = (lambda a, b: "%.2f" % (a / b) if b > 0 else "-")
        lines.append("| %s | %s | 1e%.1f | %d | %.1f | %.1f | %s | %s | %s | %s | %s |" % (
            r["slug"][:2], r["slug"][3:], r["l10"], r["iters"],
            r["astro"], r["mpfr"], f3c,
            rr(r["mpfr"], r["astro"]),
            rr(r["f3"], r["astro"]) if r["f3_ok"] else "-",
            rr(r["f3"], r["mpfr"]) if r["f3_ok"] else "-",
            px))
    # Per-band geometric means of the ratios (bands = the render-mode bands).
    def band(r):
        return gc.fractadyne_mode(r["l10"])
    lines.append("")
    for b in sorted({band(r) for r in rows}):
        sel = [r for r in rows if band(r) == b and r["f3_ok"] and r["astro"] > 0]
        if not sel:
            continue
        gm = lambda pairs: math.exp(sum(math.log(x) for x in pairs) / len(pairs))
        lines.append("- **%s** (n=%d): mpfr/astro %.2f, f3/astro %.2f, f3/mpfr %.2f" % (
            b, len(sel),
            gm([r["mpfr"] / r["astro"] for r in sel]),
            gm([r["f3"] / r["astro"] for r in sel]),
            gm([r["f3"] / r["mpfr"] for r in sel])))
    lines.append("")
    lines.append("Totals: astro %.1fs, mpfr %.1fs, f3 %.1fs (F3 over %d of %d locations%s)" % (
        sum(r["astro"] for r in rows), sum(r["mpfr"] for r in rows),
        sum(r["f3"] for r in rows if r["f3_ok"]),
        len(rows) - len(blank), len(rows),
        "; blank: " + ", ".join(r["slug"] for r in blank) if blank else ""))
    report = "\n".join(lines)
    print("\n" + report)
    verdict_px = "PIXELS: all locations byte-identical across astro/mpfr and vs the blessed corpus" \
        if not bad else "PIXELS DIVERGED: " + ", ".join(r["slug"] for r in bad)
    print("\n" + verdict_px)
    if out_md:
        hdr = ("# Backend x Fraktaler-3 corpus cross-check\n\n"
               "Date: 2026-09-01 - machine: RTX 3080 / 3950X, idle - geometry 1280x720, "
               "Fractadyne --ss 2 (4 samples/px), F3 subframes=4 (4 samples/px) - single "
               "timed run per cell, engines sequential.\n\n"
               "- astro: %s [%s]\n- mpfr: %s [%s]\n- f3: vendored 3.1 x86_64\n\n%s\n\n%s\n"
               % (astro_id + mpfr_id + (report, verdict_px)))
        open(out_md, "w", encoding="utf-8", newline="\n").write(hdr)
        print("\nreport -> %s" % out_md)
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
