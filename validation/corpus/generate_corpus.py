#!/usr/bin/env python3
"""Generate the Fractadyne <-> Kalles Fraktaler comparison corpus.

Reads locations.toml (the canonical list) and produces, for each location:
  locations/<slug>.kfr            - Kalles Fraktaler 2 / fraktaler-3 location file
  renders/<slug>-fractadyne.png   - Fractadyne render (via --render, exact framing)
plus catalog.html (side-by-side viewer; the Fraktaler slot expects
renders/<slug>-fraktaler.png, produced by generate_fraktaler.py or manually).

Run from the repo root:  python validation/corpus/generate_corpus.py [--skip-renders]

REGRESSION GATE (run after a major change):
    python validation/corpus/generate_corpus.py --check           # every non-extreme location
    python validation/corpus/generate_corpus.py --check --only 14,15
    python validation/corpus/generate_corpus.py --check --extreme  # include EXTREME-tier rows
    python validation/corpus/generate_corpus.py --only 39          # (re)generate just row 39
  Re-renders each location to a temp file and compares it pixel-for-pixel against the committed
  renders/<slug>-fractadyne.png (the F3-confirmed baselines). Renders are deterministic, so an
  unchanged renderer prints "20/20 MATCH"; any CHANGED location must get a fresh visual F3
  comparison before re-blessing (regenerate + eyeball the catalog). Exits non-zero on any change.
  Does not modify the committed renders or catalog. See DIAGNOSTICS.md.

The Fractadyne render goes through `--render` with the committed `session-template.toml` copied
into a throwaway `FRACTADYNE_CONFIG_DIR`, so a render depends on the exe, the command line, and
that file — nothing else. It must NOT go through the tour renderer: render_tour_to_dir forces
auto_iter=true, silently re-capping an explicit iteration count — deep corpus locations whose
structure escapes above the cap rendered interior-black while Fraktaler-3 used the full count.

⚠HERMETICITY, the hard-won part (2026-08-14). This used to patch ~17 keys into the DEVELOPER'S
live session and inherit the rest, which is not hermetic at all: `cycle`/`offset` (palette phase),
`log_palette`, `normalize_live`, `use_binary`, `use_duotone`, `de_strength`, `stripe_freq` and
friends all reach the render unpinned. That is why the July 2026 baselines could not be
reproduced by ANY build — the July binary included (v0.2.20 renders `01-home` at meanD 27.98
against its own baseline). The renderer never changed; the environment did.

⚠COMPARE PIXELS, NEVER FILE BYTES. `--render` embeds metadata in the PNG, so two byte-different
files routinely hold identical images — a sha256 over the file reports "nondeterministic" for a
renderer that is bit-exact (measured: 4 runs, 4 hashes, maxD 0). `check_corpus` compares decoded
RGB, and any new determinism probe must do the same.
"""

import os
import re
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
CORPUS = os.path.join(ROOT, "validation", "corpus")
EXE = os.path.join(ROOT, "target", "release", "fractadyne.exe")
# The corpus render session, committed. Copied into a throwaway `FRACTADYNE_CONFIG_DIR` per run —
# the developer's real session is never read or written (see `stage_config_dir`).
TEMPLATE = os.path.join(CORPUS, "session-template.toml")

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
        # EXTREME tier: a row whose reference build takes tens of minutes (e18003-class). Kept
        # out of the routine --check gate and out of a full regeneration unless asked for by
        # name (--only) or with --extreme, so adding an extreme row does not turn a 3-minute
        # gate into an hour. The row is still a full corpus citizen: same files, same catalog.
        loc["extreme"] = bool(re.search(r"^extreme = true", block, re.M))
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


def write_fdn(loc):
    """Write locations/<slug>.fdn — Fractadyne's native "Share location" file — so the exact view
    loads straight into the app (File -> Share location -> Load .fdn..., or paste the text).

    The renderer already embeds this reloadable block in each PNG's `Fractadyne` tEXt chunk
    (full-precision center, extended-range upp_log2, the fixed iteration count, coloring), so it is
    the authoritative source — the app wrote it and reads it back verbatim. We only strip the
    volatile provenance lines (version/build + save timestamp) so the committed .fdn is stable
    across re-renders, and set a descriptive `notes`. Returns the .fdn text (or None if the render
    or its metadata is missing). Requires the render to exist (run without --skip-renders first, or
    keep the committed renders/)."""
    from PIL import Image
    png = os.path.join(CORPUS, "renders", loc["slug"] + "-fractadyne.png")
    if not os.path.exists(png):
        return None
    meta = Image.open(png).info.get("Fractadyne")
    if not meta:
        return None
    num = loc["slug"].split("-")[0]
    drop = {"version", "saved", "saved_unix"}  # provenance/timestamp: noise + churn, not reload data
    out = []
    for line in meta.splitlines():
        key = line.split("=", 1)[0]
        if key in drop:
            continue
        if key == "notes":  # <=120 ASCII, single line (the app's tEXt constraint)
            tail = ("EXTREME tier; see catalog.html" if loc.get("extreme")
                    else "matches Fraktaler-3; see catalog.html")
            line = "notes=Fractadyne reference corpus %s: %s (%s)" % (num, loc["title"], tail)
        out.append(line)
    text = "\n".join(out) + "\n"
    path = os.path.join(CORPUS, "locations", loc["slug"] + ".fdn")
    open(path, "w", encoding="utf-8", newline="\n").write(text)
    return text


def stage_config_dir():
    """Create a throwaway config dir holding ONLY the committed corpus session; return its path.

    HERMETIC BY CONSTRUCTION, which is the whole point and was the bug. This used to read the
    DEVELOPER'S live session (`%APPDATA%\\Fractadyne\\...\\session.toml`), patch ~17 keys into it,
    write it back, render, and restore — so every key it did not think to pin rode along from
    whatever the app last wrote. `cycle`/`offset` (palette phase!), `log_palette`,
    `normalize_live`, `use_binary`, `use_duotone`, `de_strength`, `stripe_freq` and the rest all
    reach the render, which is why the July 2026 baselines could not be reproduced by ANY build —
    including the July binary that made them (verified: v0.2.20 renders `01-home` at meanD 27.98
    against its own baseline). The variable was the environment, not the renderer.

    Now: `FRACTADYNE_CONFIG_DIR` points the app at a fresh temp dir containing a copy of the
    committed `session-template.toml`. The developer's real session is never read and never
    written — no backup/restore dance, and nothing to clobber if the run dies mid-render.

    The LOCATION deliberately does NOT go into the session: it rides the `--render` command line
    (`--center`/`--zoom-log2`/`--iter`), which sets the viewport and its precision
    (`set_center_log2mag` -> `refresh_precision`) exactly. Staging the deep view into the session
    as well makes the app BOOT on it — and a live window booting an extreme-depth session while
    the export runs hung the e500 render for over an hour (the identical CLI render with a light
    session takes ~11 s). The template pins HOME with auto-iteration on and a small cap, so the
    boot frame costs nothing.
    """
    import tempfile
    d = tempfile.mkdtemp(prefix="fdcfg_")
    shutil.copyfile(TEMPLATE, os.path.join(d, "session.toml"))
    return d


def render_location(loc, out_png=None, config_dir=None):
    """Render one location via `--render`; returns the output PNG path.

    The location goes on the COMMAND LINE (`--center`/`--zoom-log2`/`--iter`): `--render` is a
    one-shot CLI renderer that always resets center+zoom from its flags (defaulting to the HOME
    view when they are absent — a session-staged location is silently ignored, which once rendered
    the corpus as twenty copies of the full set). `--zoom-log2` carries arbitrary depth; `--iter`
    is honored verbatim (auto-iter off). The staged session still supplies what has no CLI
    flag: coloring method + phase, DE/lighting, HUD, SA/glitch correction.

    `out_png` overrides the destination (the corpus check renders to a temp file so it never
    clobbers the committed render it is comparing against). `config_dir` is a staged throwaway
    config dir (see `stage_config_dir`); one is made per call when omitted.
    """
    import math
    if out_png is None:
        out_png = os.path.join(CORPUS, "renders", loc["slug"] + "-fractadyne.png")
    own_dir = config_dir is None
    if own_dir:
        config_dir = stage_config_dir()
    args = [
        "--render", "--out", out_png, "--size", "%dx%d" % (WIDTH, HEIGHT),
        "--center", loc["center_x"], loc["center_y"],
        "--zoom-log2", "%.10f" % (loc["mag_log10"] * math.log2(10.0)),
        "--iter", str(loc["iterations"]),
        "--ss", str(SS),
        "--palette", "0",  # Ember
    ]
    if loc.get("normalize"):
        args.append("--normalize")  # escape-range -> palette (deep dense fields; see read_locations)
    if sum(len(a) for a in args) > 30000:
        # An e18000-class centre is ~18k digits per ordinate — two of them overflow the Windows
        # 32,767-char command-line ceiling (CreateProcess fails with WinError 206 before the app
        # even starts). The app expands @response-file arguments itself (main.rs), so route the
        # whole argument list through one; the parsed values are identical either way.
        rf = os.path.join(config_dir, loc["slug"] + ".args")
        open(rf, "w", encoding="ascii", newline="\n").write("\n".join(args) + "\n")
        cmd = [EXE, "@" + rf]
    else:
        cmd = [EXE] + args
    env = dict(os.environ, FRACTADYNE_CONFIG_DIR=config_dir)
    try:
        # Extreme-tier rows (e18000-class) hold a ~2 h reference build on the standard exe;
        # the ordinary 1 h ceiling exists for the 38 routine rows and must not kill them.
        cap = 14400 if loc.get("extreme") else 3600
        r = subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT, timeout=cap, env=env)
        if not os.path.exists(out_png):
            sys.exit("no render for %s\nstdout: %s\nstderr: %s"
                     % (loc["slug"], r.stdout[-2000:], r.stderr[-2000:]))
        # ⚠VERIFY THE STAGING TOOK. The app falls back to DEFAULTS on a session file it cannot
        # parse (`parse_with_status`) and says nothing to the caller, so a template that goes stale
        # against the schema would silently render the whole corpus with default coloring — the
        # same unreproducible-corpus failure in a new costume. The app logs the outcome; insist on
        # it rather than trusting the file we just wrote.
        blob = (r.stderr or "") + (r.stdout or "")
        line = next((l for l in blob.splitlines() if "session:" in l), None)
        # Match the WORD, never the punctuation: the app writes an em dash and this pipe is
        # decoded with the console codepage on Windows, so any dash-matching check fails on a
        # perfectly good render (it did, first try). "loaded" also covers "loaded, newer format";
        # "none (defaults)" and "UNREADABLE, ignored (defaults)" are the two that must fail.
        if line is not None and "loaded" not in line:
            sys.exit("staged session was NOT loaded for %s -- the render used app defaults, so it "
                     "is not the corpus contract.\n  %s\n  template: %s"
                     % (loc["slug"], line.strip(), TEMPLATE))
    finally:
        if own_dir:
            shutil.rmtree(config_dir, ignore_errors=True)
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


def fdn_text(loc):
    """The committed locations/<slug>.fdn text (for the catalog copy button), or a hint if absent."""
    path = os.path.join(CORPUS, "locations", loc["slug"] + ".fdn")
    if os.path.exists(path):
        return open(path, encoding="utf-8").read().rstrip("\n")
    return "(.fdn not generated yet — run: python validation/corpus/generate_corpus.py)"


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
  <h2><span class="num" title="location {num} (slug {slug}) — matches --check / logs / --only {num}">{num}</span> {title} <span class="zoom">{zoom}&times;</span></h2>
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
  <div class="repro">
    <a class="fdn-dl" href="locations/{slug}.fdn" download>&#x2b07;&#xfe0e; {slug}.fdn</a>
    <button class="copybtn" type="button">&#x1f4cb;&#xfe0e; Copy .fdn</button>
    <span class="repro-hint">Reproduce in Fractadyne: <b>File &#9656; Share location &#9656; Load .fdn&hellip;</b> (or paste the copied text into Share location).</span>
  </div>
  <details><summary>Exact location &mdash; .fdn reload snippet, Kalles Fraktaler (.kfr), full-precision center</summary>
    <pre class="fdn">{fdn}</pre>
    <pre>{kfr}</pre>
  </details>
</section>""".format(
            slug=loc["slug"], num=loc["slug"].split("-")[0], title=esc(loc["title"]), note=esc(loc["note"]),
            fdn=esc(fdn_text(loc)),
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
  h2 .num {{ display:inline-block; min-width:1.5em; text-align:center; color:#0d0f12; background:#E0A030;
    border-radius:5px; padding:0 .35em; margin-right:.45em; font-size:.82em; font-weight:700; vertical-align:.08em; }}
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
  pre.fdn {{ color:#cfd6df; }}
  .repro {{ display:flex; gap:.6rem; align-items:center; flex-wrap:wrap; margin:.2rem 0 .5rem; }}
  .repro a.fdn-dl, .repro button.copybtn {{ font:inherit; font-size:.85em; color:#E0A030; background:#20242b;
    border:1px solid #3a404a; border-radius:6px; padding:.2rem .6rem; cursor:pointer; text-decoration:none; }}
  .repro a.fdn-dl:hover, .repro button.copybtn:hover {{ border-color:#E0A030; }}
  .repro-hint {{ color:#8a929d; font-size:.82em; }}
</style>
<h1>Fractadyne vs Kalles Fraktaler &mdash; reference render corpus</h1>
<p>Twenty locations, ordered by magnification from the full-set overview to 6.13e1105&times;, rendered by both apps from the exact same
center / magnification / iteration cap for side-by-side structural comparison. Fractadyne renders (all
twenty) come from <code>generate_corpus.py</code>; the Fraktaler-3 side from <code>generate_fraktaler.py</code>.
With <code>maximum_reference_iterations</code> set in each F3 param (see README &mdash; F3&rsquo;s batch
default is far too low for a zoomed view and blanks silently, which long masqueraded as a ~1e13&times;
ceiling), <strong>F3 renders deep correctly here</strong>: <strong>all 20</strong> locations have
clean, arm-for-arm F3 counterparts, out to 6.13e1105&times; (over a thousand orders of magnitude). The
former 1e30&times; gap (location 07, once a too-coarse 34-digit seahorse placeholder that F3 rounded onto
different sub-structure) is now a genuine cross-app match, re-imported from a user Fraktaler-3 save with a
43-digit center. Locations 07 and 11&ndash;20 are user-saved Fraktaler-3 finds (1e30&times; to
1.58e1008&times;), imported from their .exr headers with center, zoom, and iteration count taken verbatim.
<strong>Coloring note for the deep dendrite / minibrot views (13, 14, 15, 16&ndash;20):</strong> at these
depths the smooth-iter counts are huge (~3e5&ndash;1.6e6+) and vary steeply pixel-to-pixel, so the fixed
palette cycle (0.27) aliases a correct escape field into speckle (long mis-diagnosed as glitches, then a
perturbation bug &mdash; both wrong; a faithful CPU transcription of the mode-2 shader kernel reproduces
the escape VALUES exactly). These renders use <em>auto-normalized</em> coloring (<code>--normalize</code>:
the frame's escape range mapped to the palette), which colours the smooth exterior cleanly and matches F3
arm-for-arm. The spiral views (09&ndash;12) have slowly-varying exteriors that don't alias, so they keep
the standard cycle. The general fix &mdash; an adaptive-cycle coloring mode for the live view &mdash; is in
TODO.md. <strong>15 also needed a rendering fix (v0.2.18):</strong> its right-side dendrites escape at
928k&ndash;1.6M &mdash; <em>past</em> the reference orbit (~918k) &mdash; so they must rebase at the
reference&rsquo;s near-zero orbit dips. The GPU&rsquo;s BLA-skip path was skipping those dips without a
rebase check (it assumed &delta;z stays small in the BLA regime, which fails where |Z|&asymp;0),
marching the deep pixels to the reference end and capping them at ~919k so the dendrites vanished. A
rebase check at the BLA landing restored the full range (this also sharpened the deep-centre detail of
11&ndash;13); goldens stayed byte-identical.</p>
<p><strong>Reading the comparison:</strong> palettes and smooth-coloring curves differ between the apps by
design &mdash; compare <em>structure</em> (feature placement, spiral arm counts, escape-boundary shape,
minibrot positions), not colors. Zoom note: Fractadyne&rsquo;s magnification <strong>equals</strong> the
Kalles Fraktaler / Fraktaler-3 zoom (both reference a 4-unit vertical extent at value 1; aligned in
v0.2.21) &mdash; the same zoom number frames the same view in either app.</p>
<p><strong>Reproduce any location in Fractadyne:</strong> each card has a <code>.fdn</code> download and a
<em>Copy .fdn</em> button &mdash; Fractadyne&rsquo;s native &ldquo;Share location&rdquo; format. Load it via
<b>File &#9656; Share location &#9656; Load .fdn&hellip;</b>, or paste the copied text into that dialog; the
exact fractal, full-precision center, zoom, and iteration count come with it. The files also live at
<code>locations/&lt;slug&gt;.fdn</code>.</p>
{cards}
<script>
document.querySelectorAll('.copybtn').forEach(function (b) {{
  b.addEventListener('click', function () {{
    var pre = b.closest('.card').querySelector('pre.fdn');
    navigator.clipboard.writeText(pre.textContent).then(function () {{
      var t = b.textContent; b.textContent = '✓ Copied'; setTimeout(function () {{ b.textContent = t; }}, 1200);
    }});
  }});
}});
</script>
""".format(w=WIDTH, h=HEIGHT, cards="\n".join(cards))
    open(os.path.join(CORPUS, "catalog.html"), "w", encoding="utf-8", newline="\n").write(html)


def _img_rgb(path):
    """Decode a PNG to an int32 H×W×3 array (stdlib PIL/numpy — both ship with the dev env)."""
    from PIL import Image
    import numpy as np
    return np.asarray(Image.open(path).convert("RGB"), dtype=np.int32)


def check_corpus(locs, only=None):
    """REGRESSION GATE — run after a major change: re-render every location to a temp file and
    compare it PIXEL-FOR-PIXEL against the committed renders/<slug>-fractadyne.png, which were
    visually confirmed to match the Fraktaler-3 references. Renders are deterministic (same
    pipeline ⇒ byte-identical, like the --selftest goldens), so an unchanged renderer gives
    maxΔ 0 for every location; any CHANGED location must get a fresh visual F3 comparison before
    re-blessing. Does NOT touch the committed renders or catalog.html.

    `only` = list of slug prefixes to check a subset (e.g. ["14", "15"]). Returns True iff every
    checked location matched. Exit code is wired from this in main()."""
    import numpy as np
    import tempfile
    if only:
        locs = [l for l in locs if any(l["slug"].startswith(o) for o in only)]
        if not locs:
            sys.exit("no locations match --only %s" % only)
    elif "--extreme" not in sys.argv:
        skipped = [l["slug"] for l in locs if l["extreme"]]
        locs = [l for l in locs if not l["extreme"]]
        if skipped:
            print("skipping %d EXTREME location(s) (%s) -- their reference builds take tens of "
                  "minutes; pass --extreme, or name them with --only, to check them too"
                  % (len(skipped), ", ".join(skipped)))
    if not os.path.exists(EXE):
        sys.exit("build first: cargo build --release -p fractadyne-app")
    cfg = stage_config_dir()
    tmpdir = tempfile.mkdtemp(prefix="fdcorpus_")
    results = []
    try:
        for loc in locs:
            slug = loc["slug"]
            committed = os.path.join(CORPUS, "renders", slug + "-fractadyne.png")
            if not os.path.exists(committed):
                results.append((slug, "MISSING", "no committed render to compare against"))
                continue
            print("checking %s ..." % slug, flush=True)
            tmp = os.path.join(tmpdir, slug + ".png")
            render_location(loc, out_png=tmp, config_dir=cfg)
            a, b = _img_rgb(committed), _img_rgb(tmp)
            if a.shape != b.shape:
                results.append((slug, "SIZE", "shape %s vs committed %s" % (b.shape, a.shape)))
                continue
            d = np.abs(a - b)
            maxd, meand, std = int(d.max()), float(d.mean()), float(b.std())
            blank = " BLANK?" if std < 1.0 else ""  # near-uniform frame -> suspect (defensive)
            status = "MATCH" if maxd == 0 else "CHANGED"
            results.append((slug, status, "maxD %d meanD %.2f std %.1f%s" % (maxd, meand, std, blank)))
    finally:
        shutil.rmtree(tmpdir, ignore_errors=True)
        shutil.rmtree(cfg, ignore_errors=True)
    ok = sum(1 for _, s, _ in results if s == "MATCH")
    print("\ncorpus check -- re-render vs committed (F3-confirmed) renders, %dx%d %dxSS:" % (WIDTH, HEIGHT, SS))
    for slug, status, detail in results:
        flag = "" if status == "MATCH" else "   <-- REVIEW vs F3"
        print("  %-22s %-8s %s%s" % (slug, status, detail, flag))
    verdict = ("unchanged (still matches F3)" if ok == len(results)
               else "CHANGED -- review the flagged locations against their F3 reference")
    print("\n%d/%d MATCH -- corpus %s" % (ok, len(results), verdict))
    return ok == len(results)


def main():
    if "--check" in sys.argv:
        only = None
        if "--only" in sys.argv:
            only = sys.argv[sys.argv.index("--only") + 1].split(",")
        sys.exit(0 if check_corpus(read_locations(), only) else 1)
    skip_renders = "--skip-renders" in sys.argv
    locs = read_locations()
    # --only N[,M]: regenerate just the named row(s) — the way to ADD a location without
    # re-rendering (and re-timestamping) every blessed render. The catalog always rebuilds
    # from the full list, so it never loses rows.
    work = locs
    if "--only" in sys.argv:
        only = sys.argv[sys.argv.index("--only") + 1].split(",")
        work = [l for l in locs if any(l["slug"].startswith(o) for o in only)]
        if not work:
            sys.exit("no locations match --only %s" % only)
    elif "--extreme" not in sys.argv:
        skipped = [l["slug"] for l in work if l["extreme"]]
        work = [l for l in work if not l["extreme"]]
        if skipped:
            print("skipping %d EXTREME location(s) (%s) -- pass --extreme, or name them with "
                  "--only, to regenerate them" % (len(skipped), ", ".join(skipped)))
    for loc in work:
        write_kfr(loc)
    print("wrote %d .kfr files" % len(work))
    if not skip_renders:
        if not os.path.exists(EXE):
            sys.exit("build first: cargo build --release -p fractadyne-app")
        cfg = stage_config_dir()
        try:
            for loc in work:
                print("rendering %s ..." % loc["slug"], flush=True)
                out = render_location(loc, config_dir=cfg)
                print("  -> %s (%.1f MB)" % (os.path.basename(out), os.path.getsize(out) / 1e6))
        finally:
            shutil.rmtree(cfg, ignore_errors=True)
    # .fdn reload files (extracted from each render's embedded metadata — needs the renders to exist)
    n_fdn = sum(1 for loc in work if write_fdn(loc))
    print("wrote %d .fdn files (of %d)" % (n_fdn, len(work)))
    write_catalog(locs)
    print("catalog.html written")


if __name__ == "__main__":
    main()
