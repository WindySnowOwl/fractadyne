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
import os
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

def derive_findings(scenes, rows_by, view, meta, zs_rows):
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

    if view:
        bad = [k for k, v in view.items() if not v.get("same_view")]
        if bad:
            out.append(("View check FAILED",
                        "%d of %d FractalShark renders do not depict the scene asked for: %s. "
                        "A time for the wrong picture is void, not slow."
                        % (len(bad), len(view), ", ".join(sorted(bad)))))
        else:
            out.append(("View check", "all %d FractalShark renders depict the right scene "
                                      "(checked against the Fractadyne render)" % len(view)))

    if meta.get("fractalshark_shape"):
        out.append(("FractalShark mode", meta["fractalshark_shape"]))
    if meta.get("zoomseq_amortisation"):
        out.append(("Zoom-sequence amortisation",
                    "Fractadyne %sx over single frames. Fraktaler-3's batch CLI renders one "
                    "image per invocation, so its 1.0 is a property of the CLI, not its engine."
                    % meta["zoomseq_amortisation"]))
    return out


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
    args = ap.parse_args()
    d = os.path.abspath(args.results_dir)
    if not os.path.isdir(d):
        sys.stderr.write("make-report: no such directory: %s\n" % d)
        return 2

    manifest = load_json(os.path.join(d, "run-manifest.json"), {}) or {}
    meta = manifest.get("meta", {}) or {}
    runs = manifest.get("runs", []) or []
    rows = load_csv(os.path.join(d, "results.csv"))
    view = load_json(os.path.join(d, "fs-view-check.json"), {}) or {}
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

    thumbs = os.path.join(d, "thumbs")
    renderers = ["fractadyne", "fraktaler3", "fractalshark", "imagina"]
    label = {"fractadyne": "Fractadyne", "fraktaler3": "Fraktaler-3",
             "fractalshark": "FractalShark", "imagina": "Imagina"}

    # Folders written before the manifest existed still carry host and timestamp in their NAME
    # ("VGER-20260921-123041"). Recovering them there beats printing "?" at the reader.
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

    # ---- findings
    a("<h2>Findings</h2>")
    findings = derive_findings(scenes, rows_by, view, meta, zs_rows)
    if findings:
        a("<div class='panel'><dl class='find'>")
        for k, v in findings:
            a("<dt>%s</dt><dd>%s</dd>" % (esc(k), esc(v)))
        a("</dl></div>")
    a("<p class='note'>Every number below is wall-clock, process start to exit, fastest of the "
      "repeats. A DNF is a result, not a gap: it never competes for 'fastest'.</p>")

    # ---- results table
    a("<h2>Results</h2><table><tr><th>Scene</th>")
    for ren in renderers:
        if any((ren, s) in rows_by for s in scenes):
            a("<th>%s</th>" % esc(label[ren]))
    a("</tr>")
    live = [ren for ren in renderers if any((ren, s) in rows_by for s in scenes)]
    for s in scenes:
        a("<tr><td>%s</td>" % esc(s))
        times = {ren: best_time(rows_by, ren, s) for ren in live}
        fastest = min((t for t in times.values() if t is not None), default=None)
        for ren in live:
            t, st = times[ren], status_of(rows_by, ren, s)
            if t is None:
                a("<td class='dnf'>%s</td>" % esc(st or "-"))
            else:
                cls = "n win" if t == fastest else "n"
                a("<td class='%s'>%.1f s</td>" % (cls, t))
        a("</tr>")
    a("</table>")

    # ---- per scene: images + the commands that made them
    a("<h2>Per scene: what was run, and what came out</h2>")
    a("<p class='sub'>Click any image for the full-resolution render. The command line under "
      "each is the one that produced that file.</p>")
    for s in scenes:
        a("<h3>%s</h3>" % esc(s))
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
            if ren == "fractalshark" and s in view:
                v = view[s]
                ok = v.get("same_view")
                a("<div class='%s'>view check: %s (edge r %.3f, best wrong %.3f)</div>"
                  % ("kv" if ok else "note",
                     "depicts this scene" if ok else "DOES NOT depict this scene",
                     v.get("edge_r", 0.0), v.get("best_wrong_r", 0.0)))
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
    a("<p class='sub'>In order, including one-off setup such as Fraktaler-3's hardware tuning and "
      "the FractalShark server. This is the record that lets someone else reproduce or challenge "
      "a number.</p>")
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

    out = os.path.join(d, "report.html")
    with open(out, "w", encoding="utf-8") as fh:
        fh.write("\n".join(H))
    if not HAVE_PIL and not args.no_thumbs:
        print("Pillow not installed: full-size images linked instead of thumbnailed.")
    print("report: %s" % out)
    return 0


if __name__ == "__main__":
    sys.exit(main())
