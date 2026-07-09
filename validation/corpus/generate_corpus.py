#!/usr/bin/env python3
"""Generate the Fractadyne <-> Kalles Fraktaler comparison corpus.

Reads locations.toml (the canonical list) and produces, for each location:
  locations/<slug>.kfr            - Kalles Fraktaler 2 / fraktaler-3 location file
  renders/<slug>-fractadyne.png   - Fractadyne render (via --render-tour, exact framing)
plus catalog.html (side-by-side viewer; the Fraktaler slot expects
renders/<slug>-fraktaler.png, which you produce manually in KF and drop in).

Run from the repo root:  python validation/corpus/generate_corpus.py [--skip-renders]

The Fractadyne render goes through the tour renderer rather than --render because a
tour keyframe takes the exact center strings + mag_log10 and the frame size comes from
--size (no dependence on the live window/panel geometry). Session settings that the
tour engine still reads (iteration cap) are staged into session.toml per location and
the original session is restored afterwards.
"""

import os
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CORPUS = os.path.join(ROOT, "validation", "corpus")
EXE = os.path.join(ROOT, "target", "release", "fractadyne.exe")
SESSION = os.path.expandvars(r"%APPDATA%\Fractadyne\Fractadyne\config\session.toml")

# Render geometry — identical for both apps (set KF's image size to match).
WIDTH, HEIGHT, SS = 1280, 720, 2


def read_locations():
    """Minimal [[location]] table parser (stdlib only; the file is ours and regular)."""
    text = open(os.path.join(CORPUS, "locations.toml"), encoding="utf-8").read()
    locs = []
    for block in text.split("[[location]]")[1:]:
        loc = {}
        for key in ("slug", "title", "center_x", "center_y", "note"):
            m = re.search(r'%s = "(.*?)"' % key, block)
            loc[key] = m.group(1) if m else ""
        loc["mag_log10"] = float(re.search(r"mag_log10 = ([\d.eE+-]+)", block).group(1))
        loc["iterations"] = int(re.search(r"iterations = (\d+)", block).group(1))
        locs.append(loc)
    return locs


def zoom_string(l10):
    """KF-style zoom string from log10(magnification), e.g. 500.7846 -> '6.0954E500'."""
    exp = int(l10 // 1)
    mant = 10 ** (l10 - exp)
    if exp < 3:  # shallow: plain number reads better
        return "%.6g" % (10 ** l10)
    return "%.6fE%d" % (mant, exp)


def write_kfr(loc):
    """Kalles Fraktaler 2 .kfr location (minimal, defaults elsewhere; fraktaler-3 reads it too)."""
    path = os.path.join(CORPUS, "locations", loc["slug"] + ".kfr")
    with open(path, "w", newline="\r\n") as f:  # KF is a Windows app; CRLF is its native line ending
        f.write("Re: %s\n" % loc["center_x"])
        f.write("Im: %s\n" % loc["center_y"])
        f.write("Zoom: %s\n" % zoom_string(loc["mag_log10"]))
        f.write("Iterations: %d\n" % loc["iterations"])
    return path


def stage_session(iterations):
    """Point session.toml at fixed iterations so the tour renders with the same cap as KF.

    Also forces the location HUD off: the tour engine ORs the session's `show_location`
    with the script's, so a session with the HUD on would burn it into every frame.
    """
    t = open(SESSION, encoding="utf-8").read()
    t = re.sub(r"max_iter = \d+", "max_iter = %d" % iterations, t, count=1)
    t = re.sub(r"auto_iter = \w+", "auto_iter = false", t, count=1)
    t = re.sub(r"show_location = \w+", "show_location = false", t, count=1)
    open(SESSION, "w", encoding="utf-8").write(t)


def render_location(loc):
    """Render one location through the tour renderer; returns the output PNG path."""
    out_png = os.path.join(CORPUS, "renders", loc["slug"] + "-fractadyne.png")
    stage_session(loc["iterations"])
    with tempfile.TemporaryDirectory() as td:
        tour = os.path.join(td, "loc.toml")
        with open(tour, "w", newline="\n") as f:
            f.write('name = "corpus %s"\npalette = "Ember"\n\n' % loc["slug"])
            f.write("[[keyframe]]\n")
            f.write('center_x = "%s"\ncenter_y = "%s"\n' % (loc["center_x"], loc["center_y"]))
            f.write("mag_log10 = %.10f\n" % loc["mag_log10"])
            f.write('fractal = "Mandelbrot"\ndual = false\nsecs = 0.0\nhold = 1.0\n')
        frames = os.path.join(td, "frames")
        os.makedirs(frames)
        cmd = [
            EXE, "--render-tour", tour,
            "--size", "%dx%d" % (WIDTH, HEIGHT), "--ss", str(SS),
            "--fps", "1", "--prefix", "frame", "--out", frames, "-y",
        ]
        r = subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT, timeout=1800)
        pngs = sorted(f for f in os.listdir(frames) if f.endswith(".png"))
        if not pngs:
            sys.exit("no frames rendered for %s\nstdout: %s\nstderr: %s" % (loc["slug"], r.stdout[-2000:], r.stderr[-2000:]))
        shutil.copyfile(os.path.join(frames, pngs[-1]), out_png)
    return out_png


def fractadyne_mode(l10):
    if l10 < 4.0:
        return "direct (df32 in-shader)"
    if l10 < 28.0:
        return "perturbation, df32 delta (mode 0) + SA"
    return "perturbation, floatexp delta (mode 2) + BLA"


def kfr_text(loc):
    return "Re: %s\nIm: %s\nZoom: %s\nIterations: %d" % (
        loc["center_x"], loc["center_y"], zoom_string(loc["mag_log10"]), loc["iterations"])


def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def write_catalog(locs):
    cards = []
    for loc in locs:
        octaves = loc["mag_log10"] * 3.321928
        precision = max(int(octaves) + 64, 64)
        digits = max(len(loc["center_x"]), len(loc["center_y"])) - 2
        cards.append("""
<section class="card" id="{slug}">
  <h2>{title} <span class="zoom">{zoom}&times;</span></h2>
  <p class="note">{note}</p>
  <div class="pair">
    <figure>
      <img src="renders/{slug}-fractadyne.png" alt="Fractadyne render">
      <figcaption>Fractadyne &mdash; {w}&times;{h}, {ss}&times;SS, Ember / smooth iteration</figcaption>
    </figure>
    <figure>
      <img src="renders/{slug}-fraktaler.png" alt="Fraktaler render"
           onerror="this.closest('figure').classList.add('missing')">
      <figcaption>Fraktaler-3 &mdash; from <code>locations/{slug}.f3.toml</code>. Missing = no
        reproducible cross-app match at this center&rsquo;s precision for the depth (see README).</figcaption>
    </figure>
  </div>
  <table class="meta">
    <tr><th>Magnification</th><td>{zoom}&times; (log10 = {l10:.6f})</td>
        <th>Iterations</th><td>{iters:,} (fixed, both apps)</td></tr>
    <tr><th>Fractadyne path</th><td>{mode}</td>
        <th>Precision</th><td>~{precision} bits ({digits}-digit center)</td></tr>
  </table>
  <details><summary>Exact location &mdash; Kalles Fraktaler (.kfr) and full-precision center</summary>
    <pre>{kfr}</pre>
  </details>
</section>""".format(
            slug=loc["slug"], title=esc(loc["title"]), note=esc(loc["note"]),
            zoom=zoom_string(loc["mag_log10"]).replace("E", "e"), l10=loc["mag_log10"],
            iters=loc["iterations"], mode=fractadyne_mode(loc["mag_log10"]),
            precision=precision, digits=digits, w=WIDTH, h=HEIGHT, ss=SS,
            kfr=esc(kfr_text(loc))))
    html = """<!doctype html>
<meta charset="utf-8">
<title>Fractadyne vs Kalles Fraktaler &mdash; reference render corpus</title>
<style>
  body {{ background:#14161a; color:#ddd; font:15px/1.5 system-ui, sans-serif; max-width:1400px; margin:2rem auto; padding:0 1rem; }}
  h1 {{ color:#E0A030; }} h2 {{ margin:0 0 .2rem; }} h2 .zoom {{ color:#E0A030; font-weight:normal; font-size:.8em; }}
  .card {{ border:1px solid #2a2e35; border-radius:10px; padding:1rem 1.2rem; margin:1.4rem 0; background:#191c21; }}
  .note {{ color:#9aa2ad; margin:.2rem 0 .8rem; }}
  .pair {{ display:grid; grid-template-columns:1fr 1fr; gap:.8rem; }}
  figure {{ margin:0; }} figure img {{ width:100%; border-radius:6px; display:block; background:#0d0f12; }}
  figure.missing img {{ min-height:180px; visibility:hidden; }}
  figure.missing {{ position:relative; border:1px dashed #3a404a; border-radius:6px; }}
  figure.missing::before {{ content:"Fraktaler render not added yet"; position:absolute; inset:0; display:flex;
    align-items:center; justify-content:center; color:#6b7380; }}
  figcaption {{ font-size:.82em; color:#8a929d; margin-top:.3rem; }}
  table.meta {{ border-collapse:collapse; margin:.8rem 0 .4rem; font-size:.9em; }}
  table.meta th {{ text-align:left; color:#9aa2ad; font-weight:600; padding:.15rem 1rem .15rem 0; }}
  table.meta td {{ padding:.15rem 2rem .15rem 0; }}
  details {{ margin-top:.4rem; }} summary {{ cursor:pointer; color:#E0A030; }}
  pre {{ background:#0d0f12; border:1px solid #2a2e35; border-radius:6px; padding:.7rem; overflow-x:auto;
         font-size:.78em; white-space:pre-wrap; word-break:break-all; }}
</style>
<h1>Fractadyne vs Kalles Fraktaler &mdash; reference render corpus</h1>
<p>Ten locations from the full-set overview to 4.6e1105&times;, rendered by both apps from the exact same
center / magnification / iteration cap for side-by-side structural comparison. Fractadyne renders (all
ten) come from <code>generate_corpus.py</code>; the Fraktaler-3 side from <code>generate_fraktaler.py</code>.
With <code>maximum_reference_iterations</code> set in each F3 param (see README &mdash; F3&rsquo;s batch
default is far too low for a zoomed view and blanks silently, which long masqueraded as a ~1e13&times;
ceiling), <strong>F3 renders deep correctly here</strong>: locations 01&ndash;06 and 08&ndash;10 have
real, arm-for-arm F3 counterparts, out to 4.60e1105&times; (over a thousand orders of magnitude). The
lone gap, 07 (1e30&times;), is a center-precision placeholder &mdash; its 34-digit seahorse center is
too coarse for a reproducible cross-app match there, while 08 (83-digit), 09 (526-digit) and 10
(1141-digit) carry far more digits and match far deeper.</p>
<p><strong>Reading the comparison:</strong> palettes and smooth-coloring curves differ between the apps by
design &mdash; compare <em>structure</em> (feature placement, spiral arm counts, escape-boundary shape,
minibrot positions), not colors. Framing note: Fractadyne&rsquo;s magnification is referenced to a
3-unit-high view; KF&rsquo;s zoom is referenced to its own (wider) home frame, so KF may frame the same
zoom value slightly wider &mdash; the <em>center</em> feature and structure must still match exactly.</p>
{cards}
""".format(w=WIDTH, h=HEIGHT, cards="\n".join(cards))
    open(os.path.join(CORPUS, "catalog.html"), "w", encoding="utf-8", newline="\n").write(html)


def main():
    skip_renders = "--skip-renders" in sys.argv
    locs = read_locations()
    for loc in locs:
        write_kfr(loc)
    print("wrote %d .kfr files" % len(locs))
    if not skip_renders:
        if not os.path.exists(EXE):
            sys.exit("build first: cargo build --release -p fractadyne-app")
        backup = SESSION + ".corpusbak"
        shutil.copyfile(SESSION, backup)
        try:
            for loc in locs:
                print("rendering %s ..." % loc["slug"], flush=True)
                out = render_location(loc)
                print("  -> %s (%.1f MB)" % (os.path.basename(out), os.path.getsize(out) / 1e6))
        finally:
            shutil.copyfile(backup, SESSION)
            os.remove(backup)
            print("session restored")
    write_catalog(locs)
    print("catalog.html written")


if __name__ == "__main__":
    main()
