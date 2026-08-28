"""Subset Lucide to the icons Fractadyne actually uses, and emit the Rust constants.

The full font is ~840 KB for 2035 icons; we use a few dozen. Subsetting keeps the binary small and, more
usefully, makes the set we depend on EXPLICIT: adding an icon means editing this table, which is
also where the name -> codepoint mapping is checked against Lucide's own manifest.
"""
import io, json, os, subprocess, sys

TMP = os.path.join(os.environ["TEMP"], "lucide")
REPO = r"c:\Users\rhong\Documents\Claude\Code\FractEx"
ASSETS = os.path.join(REPO, r"crates\fractadyne-app\assets\fonts")

# (RUST_CONST, lucide-name, what it replaces / where it is used)
ICONS = [
    ("OPEN",        "folder-open",  "File > Open view or location, toolbar"),
    ("GALLERY",     "image",        "File > Gallery, toolbar"),
    ("SAVE",        "save",         "File > Export image, toolbar"),
    ("SNAPSHOT",    "camera",       "File > Snapshot, toolbar"),
    ("SETTINGS",    "settings",     "File > Settings"),
    ("RESET_APP",   "eraser",       "File > Reset application state"),
    ("QUIT",        "log-out",      "File > Quit"),
    ("HOME",        "house",        "Navigate > Zoom out to full view, toolbar"),
    ("RESET_VIEW",  "rotate-ccw",   "Navigate > Reset to default view, toolbar"),
    ("RANDOM",      "dices",        "Navigate > Random location"),
    ("SHARE",       "link",         "Navigate > Share location"),
    ("ZOOM_IN",     "zoom-in",      "toolbar"),
    ("ZOOM_OUT",    "zoom-out",     "toolbar"),
    ("CLICK_ZOOM",  "crosshair",    "toolbar: click-to-zoom toggle"),
    ("AUTOPILOT",   "bot",          "toolbar: auto-zoom toggle"),
    ("PALETTE",     "palette",      "toolbar: next palette"),
    ("PERF",        "chart-column", "toolbar: performance panel"),
    ("PLAY",        "play",         "toolbar + tour transport"),
    ("PAUSE",       "pause",        "tour transport"),
    ("STOP",        "square",       "tour transport"),
    ("SKIP_BACK",   "skip-back",    "tour transport: restart"),
    ("REWIND",      "rewind",       "tour transport: back 10s"),
    ("FORWARD",     "fast-forward", "tour transport: forward 10s"),
    ("LOOP",        "repeat",       "tour transport: loop"),
    ("TOUR",        "clapperboard", "tour transport: record/tour"),
    ("FULLSCREEN",  "maximize",     "toolbar: fullscreen toggle"),
    ("BOOKMARK",    "star",         "Navigate > Bookmarks, toolbar"),
    ("EDIT",        "pencil",       "Color > Palette > Custom"),
    ("CLOSE",       "x",            "tour transport close, gradient stop remove"),
    ("ADD",         "plus",         "gradient editor: add stop"),
    ("DELETE",      "trash-2",      "bookmarks browser: delete"),
    ("DUAL",        "columns-2",    "toolbar: dual linked view toggle"),
]

info = json.load(io.open(os.path.join(TMP, "info.json"), encoding="utf-8"))
cps = {}
missing = [n for _, n, _ in ICONS if n not in info]
assert not missing, "not in Lucide: %s" % missing
for const, name, _ in ICONS:
    cps[const] = int(info[name]["unicode"][2:-1])

# --- subset -------------------------------------------------------------------------------
os.makedirs(ASSETS, exist_ok=True)
out = os.path.join(ASSETS, "Lucide.ttf")
uni = ",".join("U+%04X" % c for c in sorted(cps.values()))
subprocess.run(
    [sys.executable, "-m", "fontTools.subset", os.path.join(TMP, "lucide.ttf"),
     "--unicodes=" + uni, "--output-file=" + out, "--no-hinting", "--desubroutinize"],
    check=True, capture_output=True,
)
full = os.path.getsize(os.path.join(TMP, "lucide.ttf"))
print("subset %d icons: %d KB -> %d KB" % (len(ICONS), full // 1024, os.path.getsize(out) // 1024))

# Verify the subset really contains them all - a subset that silently dropped a glyph would
# reintroduce exactly the tofu bug this replaces.
from fontTools.ttLib import TTFont
have = set()
f = TTFont(out, lazy=True)
for t in f["cmap"].tables:
    have |= set(t.cmap.keys())
lost = {k: v for k, v in cps.items() if v not in have}
assert not lost, "subset dropped: %s" % lost
print("verified: all %d codepoints present in the subset" % len(cps))

ICONS_RS = os.path.join(REPO, r"crates\fractadyne-app\src\icons.rs")

# This script rewrites icons.rs WHOLE: anything hand-written in it is destroyed on the next
# run. That is not hypothetical - the glyph-coverage test lived at the bottom of that file
# and adding one icon deleted it, silently, with the suite still green. It now lives in
# src/icons_coverage.rs. Refuse rather than repeat that.
if os.path.exists(ICONS_RS):
    prev = io.open(ICONS_RS, encoding="utf-8").read()
    if "#[cfg(test)]" in prev or "mod " in prev:
        sys.exit("icons.rs has hand-written code in it; this script would delete it. "
                 "Move it to its own file (see src/icons_coverage.rs) first.")

# --- Rust constants -----------------------------------------------------------------------
lines = [
    "//! Lucide icon glyphs, by name.",
    "//!",
    "//! The UI used Unicode emoji as icons, which had two problems: the bundled fonts cover an",
    "//! arbitrary SUBSET of them (four glyphs shipped as tofu squares before this), and the ones",
    "//! that did render came from different families and did not look like one set.",
    "//!",
    "//! These are Private Use Area codepoints, so they are meaningless on sight - hence names.",
    "//! NEVER write a raw `\\u{e...}` escape at a call site; add a constant here instead, and",
    "//! regenerate with `scripts/subset_lucide.py` so the font subset gains the glyph too. A",
    "//! codepoint with no glyph in the subset renders as a blank box, which is the failure mode",
    "//! this module exists to end.",
    "//!",
    "//! Generated by `scripts/subset_lucide.py` - edit the ICONS table there, not this file.",
    "//! Nothing hand-written survives here; the coverage test is in `icons_coverage.rs`.",
    "",
    "#![allow(dead_code)] // the full set is deliberately available; not every icon is placed yet",
    "",
]
for const, name, use in ICONS:
    lines.append("/// Lucide `%s` - %s" % (name, use))
    lines.append('pub(crate) const %s: &str = "\\u{%x}";' % (const, cps[const]))
    lines.append("")
io.open(ICONS_RS, "w", encoding="utf-8", newline="\n").write("\n".join(lines))
print("wrote crates/fractadyne-app/src/icons.rs")
