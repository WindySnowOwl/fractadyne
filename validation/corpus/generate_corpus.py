#!/usr/bin/env python3
"""Generate the Fractadyne <-> Kalles Fraktaler comparison corpus.

Reads locations.toml (the canonical list) and produces, for each location:
  locations/<slug>.kfr            - Kalles Fraktaler 2 / fraktaler-3 location file
  renders/<slug>-fractadyne.png   - Fractadyne render (via --render, exact framing)
plus catalog.html (side-by-side viewer; the Fraktaler slot expects
renders/<slug>-fraktaler.png, produced by generate_fraktaler.py or manually).

Run from the repo root:  python validation/corpus/generate_corpus.py [--skip-renders]

The Fractadyne render goes through `--render` with a FULLY staged session (location,
iterations, coloring), restored afterwards. It must NOT go through the tour renderer:
render_tour_to_dir forces auto_iter=true, silently re-capping an explicit iteration
count — deep corpus locations whose structure escapes above the cap rendered
interior-black while Fraktaler-3 used the full count. And the staging must pin the
COLORING too (Ember / smooth, no DE/lighting — the catalog contract): a partial staging
inherits whatever stripe/relief config the live session happens to have, so the corpus
renders weren't reproducible (the same hermeticity lesson as --selftest).
"""

import os
import re
import shutil
import subprocess
import sys

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
        # Auto-normalize coloring: at extreme depth the smooth-iter counts are ~1e5-1e6 and a fixed
        # palette cycle aliases a correct escape field into speckle (14/15). `--normalize` maps the
        # frame's escape range to the palette instead. See the corpus README / TODO.
        loc["normalize"] = bool(re.search(r"^normalize = true", block, re.M))
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


def stage_session(loc):
    """Stage the session for one corpus render: pinned coloring + a LIGHT home view.

    Hermetic by design (the --selftest lesson): anything the render reads that is NOT staged
    here silently inherits the user's live session, making the corpus unreproducible. The
    catalog contract is 1280x720 2xSS, Ember palette, smooth iteration, no DE/lighting/HUD,
    and the location's EXACT iteration count.

    The LOCATION deliberately does NOT go into the session: it rides the `--render` command
    line (`--center`/`--zoom-log2`/`--iter`), which sets the viewport and its precision
    (`set_center_log2mag` -> `refresh_precision`) exactly. Staging the deep view into the
    session as well makes the app BOOT on it — and a live window booting an extreme-depth
    session while the export runs hung the e500 render for over an hour (the identical CLI
    render with a light session takes ~11 s). The session view is pinned to HOME with
    auto-iteration on and a small cap, so the boot frame costs nothing.
    """
    t = open(SESSION, encoding="utf-8").read()

    def setk(t, key, val):
        pat = r"^%s = .*$" % re.escape(key)
        rep = "%s = %s" % (key, val)
        if re.search(pat, t, re.M):
            return re.sub(pat, rep.replace("\\", "\\\\"), t, count=1, flags=re.M)
        return t + "\n" + rep + "\n"

    del loc  # location intentionally unused here: it rides the CLI (see docstring)
    for key, val in [
        ("center_x", "-0.5"),
        ("center_y", "0.0"),
        ("center_x_str", '"-0.5"'),
        ("center_y_str", '"0.0"'),
        ("units_per_pixel", repr(3.0 / HEIGHT)),
        ("units_per_pixel_e", "0"),
        ("max_iter", "1000"),
        ("auto_iter", "true"),
        ("show_location", "false"),
        ("color_method", '"smooth"'),
        # SA and glitch correction off: a fixed-iteration comparison render wants the fewest
        # approximations/post-passes in play — and the multi-reference glitch-correction pass goes
        # pathological (>1 h) at extreme depth (it rebuilds ~1700-bit references per glitch; see
        # TODO.md "Open bugs"). Stripe-method sessions never showed it because aux methods skip
        # correction — which is why corpus renders inherited from a stripe session looked fine
        # while hermetic smooth renders hung. BLA still accelerates the deep iterate.
        ("series_approx", "false"),
        ("glitch_correct", "false"),
        ("palette_idx", "0"),  # Ember
        ("de", "false"),
        ("light", "false"),
        ("palette_anim", '"off"'),
        ("use_custom_palette", "false"),
        ("export_ss", str(SS)),
        ("fractal", '"Mandelbrot"'),
        ("julia_mode", "false"),
        ("dual", "false"),
    ]:
        t = setk(t, key, val)
    open(SESSION, "w", encoding="utf-8", newline="\n").write(t)


def render_location(loc):
    """Render one location via `--render`; returns the output PNG path.

    The location goes on the COMMAND LINE (`--center`/`--zoom-log2`/`--iter`): `--render` is a
    one-shot CLI renderer that always resets center+zoom from its flags (defaulting to the HOME
    view when they are absent — a session-staged location is silently ignored, which once rendered
    the corpus as twenty copies of the full set). `--zoom-log2` carries arbitrary depth; `--iter`
    is honored verbatim (auto-iter off). The session staging above still pins what has no CLI
    flag: coloring method, DE/lighting, HUD.
    """
    import math
    out_png = os.path.join(CORPUS, "renders", loc["slug"] + "-fractadyne.png")
    stage_session(loc)
    cmd = [
        EXE, "--render", "--out", out_png, "--size", "%dx%d" % (WIDTH, HEIGHT),
        "--center", loc["center_x"], loc["center_y"],
        "--zoom-log2", "%.10f" % (loc["mag_log10"] * math.log2(10.0)),
        "--iter", str(loc["iterations"]),
        "--ss", str(SS),
        "--palette", "0",  # Ember
    ]
    if loc.get("normalize"):
        cmd.append("--normalize")  # escape-range -> palette (deep dense fields; see read_locations)
    r = subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT, timeout=3600)
    if not os.path.exists(out_png):
        sys.exit("no render for %s\nstdout: %s\nstderr: %s" % (loc["slug"], r.stdout[-2000:], r.stderr[-2000:]))
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
    # Catalog cards in MAGNIFICATION order (locations.toml keeps its stable numbering — slugs,
    # files, and cross-references don't move; only the browsing order is depth-sorted).
    locs = sorted(locs, key=lambda l: l["mag_log10"])
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
<p>Twenty locations, ordered by magnification from the full-set overview to 4.6e1105&times;, rendered by both apps from the exact same
center / magnification / iteration cap for side-by-side structural comparison. Fractadyne renders (all
twenty) come from <code>generate_corpus.py</code>; the Fraktaler-3 side from <code>generate_fraktaler.py</code>.
With <code>maximum_reference_iterations</code> set in each F3 param (see README &mdash; F3&rsquo;s batch
default is far too low for a zoomed view and blanks silently, which long masqueraded as a ~1e13&times;
ceiling), <strong>F3 renders deep correctly here</strong>: <strong>19 of 20</strong> locations have
clean, arm-for-arm F3 counterparts, out to 4.60e1105&times; (over a thousand orders of magnitude)
(location 15 is a known partial &mdash; see the note below). The
former 1e30&times; gap (location 07, once a too-coarse 34-digit seahorse placeholder that F3 rounded onto
different sub-structure) is now a genuine cross-app match, re-imported from a user Fraktaler-3 save with a
43-digit center. Locations 07 and 11&ndash;20 are user-saved Fraktaler-3 finds (7.5e29&times; to
1.2e1008&times;), imported from their .exr headers with center, zoom, and iteration count taken verbatim.
<strong>Coloring note for 14 (1.2e148&times;):</strong> this long rendered as speckle and was
mis-diagnosed as glitches, then a perturbation bug &mdash; both wrong. The escape VALUES are correct
(a faithful CPU transcription of the mode-2 shader kernel reproduces them exactly); the problem is
that its smooth-iter counts are huge (~3e5&ndash;8e5) and vary steeply, so the fixed palette cycle
(0.27) aliases a correct field into speckle. Location 14&rsquo;s render uses <em>auto-normalized</em>
coloring (the frame's escape range mapped to the palette) and then matches F3 arm-for-arm. The general
fix &mdash; an adaptive-cycle coloring mode for extreme depth &mdash; is in TODO.md.
<strong>Known limit at 15 (3.7e163&times;):</strong> the smooth bulb matches F3, but the right-side
dendrites are MISSING (smooth-orange where F3 shows dense dark spirals). Those pixels escape at
928k&ndash;1.6M &mdash; <em>past</em> the reference orbit (which escapes at ~918k) &mdash; so the GPU
caps them at the reference&rsquo;s end (~919k) instead of rebasing deeper. The faithful CPU df32 sim
recovers the dendrites with that same reference (by rebasing), so it is a GPU past-reference rebasing
limitation, not coloring or iteration count; a longer reference would fix it but overflows the GPU
128&nbsp;MB buffer-binding limit (~932k samples). Tracked in TODO.md &ldquo;Open bugs&rdquo;.</p>
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
