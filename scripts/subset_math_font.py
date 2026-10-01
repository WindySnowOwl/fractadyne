"""Build Fractadyne Math: the subset of Latin Modern Math that the formula editor's textbook mode draws.

Latin Modern Math is the OpenType Computer Modern -- the face LaTeX sets mathematics in. The textbook
mode (design/formula-textbook-editor.md) lays formulas out by TeX's rules from the font's own MATH
table, so the table is kept whole. Two changes besides dropping glyphs:

- Every size variant and assembly part of the stretchy glyphs -- parentheses, bars, the radical --
  gets a Private Use Area code point (U+E000 up). egui draws text by CHARACTER, and these glyphs have
  none in the original; with one, they are drawn from egui's anti-aliased glyph atlas like any letter.
- The font is renamed. Its licence, the GUST Font License (the LaTeX Project Public License 1.3c plus
  a request to rename derived fonts), is honoured by: the new name; the original copyright kept in the
  name table; FractadyneMath-NOTICE.txt saying what changed and where the original is; and
  GUST-FONT-LICENSE.txt shipped beside it.

Run: python scripts/subset_math_font.py   (needs fonttools; downloads lm-math.zip from CTAN once)
"""
import hashlib, io, os, sys, urllib.request, zipfile

from fontTools.ttLib import TTFont
from fontTools import subset

URL = "https://mirrors.ctan.org/fonts/lm-math.zip"
SHA256 = "3d906317f27279af05eb095aa4db5e7f3f87312e69d672e8f8928b64adcd403c"  # lm-math 1.959
TMP = os.path.join(os.environ.get("TEMP", "/tmp"), "lmmath")
REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))  # this script is in <repo>/scripts
ASSETS = os.path.join(REPO, "crates", "fractadyne-app", "assets", "fonts")
OUT = os.path.join(ASSETS, "FractadyneMath.otf")

FAMILY = "Fractadyne Math"
POSTSCRIPT = "FractadyneMath-Regular"
PUA_START = 0xE000

# What the formula language can show. Upright Latin for function names (sin, Re), math italic for
# variables, digits, the operators, and the stretchy glyphs' bases.
KEEP = (
    list(range(0x20, 0x7F))  # printable ASCII
    + list(range(0x1D434, 0x1D468))  # MATHEMATICAL ITALIC A..z (U+1D455 is unassigned: h is U+210E)
    + [0x210E]  # PLANCK CONSTANT = math italic h
    + [0x03C0, 0x1D70B]  # pi, math italic pi
    + [0x2212, 0x22C5, 0x00D7, 0x221A, 0x2190, 0x2016]  # minus, dot, times, radical, left arrow, double bar
)


def fetch():
    os.makedirs(TMP, exist_ok=True)
    path = os.path.join(TMP, "lm-math.zip")
    if not os.path.exists(path):
        print("downloading", URL)
        urllib.request.urlretrieve(URL, path)
    data = open(path, "rb").read()
    digest = hashlib.sha256(data).hexdigest()
    if digest != SHA256:
        sys.exit(f"lm-math.zip has SHA-256 {digest}, expected {SHA256}: refusing an unknown font")
    z = zipfile.ZipFile(io.BytesIO(data))
    font = z.read("lm-math/opentype/latinmodern-math.otf")
    licence = z.read("lm-math/doc/GUST-FONT-LICENSE.txt")
    return font, licence


def stretchy_glyphs(font):
    """Every vertical size variant and assembly part, in the MATH table's order (deterministic)."""
    mv = font["MATH"].table.MathVariants
    out = []
    for c in mv.VertGlyphConstruction or []:
        for r in c.MathGlyphVariantRecord or []:
            out.append(r.VariantGlyph)
        if c.GlyphAssembly:
            for p in c.GlyphAssembly.PartRecords:
                out.append(p.glyph)
    seen, ordered = set(), []
    for g in out:
        if g not in seen:
            seen.add(g)
            ordered.append(g)
    return ordered


def main():
    font_bytes, licence = fetch()
    font = TTFont(io.BytesIO(font_bytes))

    opts = subset.Options()
    opts.layout_features = []  # egui shapes no OpenType features; TeX layout needs only MATH
    opts.drop_tables += ["GSUB", "GPOS", "GDEF", "FFTM"]
    opts.name_IDs = ["*"]
    opts.name_languages = ["*"]
    opts.notdef_outline = True
    opts.hinting = False
    sub = subset.Subsetter(opts)
    sub.populate(unicodes=KEEP)
    sub.subset(font)  # keeps MATH and the glyphs its variants and assemblies name

    # Private Use Area code points for the stretchy glyphs that have no character of their own.
    cmap_tables = [t for t in font["cmap"].tables if t.isUnicode()]
    mapped = {g for t in cmap_tables for g in t.cmap.values()}
    pua = {}
    for g in stretchy_glyphs(font):
        if g not in mapped:
            pua[PUA_START + len(pua)] = g
    for t in cmap_tables:
        t.cmap.update(pua)

    # The new name (GUST Font License, clause 1), the original copyright kept.
    name = font["name"]
    old_full = name.getDebugName(4)
    for rec in name.names:
        if rec.nameID in (1, 4, 16, 18):
            rec.string = FAMILY
        elif rec.nameID == 3:
            rec.string = f"{FAMILY}; derived from {old_full}"
        elif rec.nameID == 6:
            rec.string = POSTSCRIPT
    name.setName(
        f"A subset of {old_full} 1.959 by B. Jackowski, P. Strzelczyk and P. Pianowski, made for "
        "Fractadyne's formula editor: glyphs outside the formula language removed, size variants "
        "and assembly parts given Private Use Area code points, renamed. Original: " + URL,
        10, 3, 1, 0x409,
    )
    name.setName("GUST Font License (LPPL 1.3c): see GUST-FONT-LICENSE.txt", 13, 3, 1, 0x409)
    cff = font["CFF "].cff
    cff.fontNames = [POSTSCRIPT]
    top = cff.topDictIndex[0]
    top.FullName = FAMILY
    top.FamilyName = FAMILY

    os.makedirs(ASSETS, exist_ok=True)
    font.save(OUT)
    open(os.path.join(ASSETS, "GUST-FONT-LICENSE.txt"), "wb").write(licence)
    with open(os.path.join(ASSETS, "FractadyneMath-NOTICE.txt"), "w", encoding="utf-8", newline="\n") as f:
        f.write(NOTICE.format(url=URL, sha=SHA256, glyphs=len(font.getGlyphOrder()), pua=len(pua)))

    print(f"{OUT}: {os.path.getsize(OUT)} bytes, {len(font.getGlyphOrder())} glyphs, "
          f"{len(pua)} stretchy glyphs at U+{PUA_START:04X}..U+{PUA_START + len(pua) - 1:04X}")


NOTICE = """Fractadyne Math (FractadyneMath.otf)

A derived work of Latin Modern Math, version 1.959 (5 IX 2014), by Boguslaw Jackowski, Piotr
Strzelczyk and Piotr Pianowski, Copyright 2012--2014 for the Latin Modern math extensions (on behalf
of TeX Users Groups). It is distributed under the GUST Font License (GUST-FONT-LICENSE.txt), an
instance of the LaTeX Project Public License 1.3c (http://www.latex-project.org/lppl.txt).

It is NOT Latin Modern Math, and its authors do not support it. As the licence asks, it has a new
name. Changes, all made by scripts/subset_math_font.py in the Fractadyne repository:

- Subset to the {glyphs} glyphs the Fractadyne formula language can show (ASCII, mathematical italic
  Latin, pi, the minus, dot, times and radical signs, a left arrow, the double bar) together with
  the size variants and assembly parts its MATH table names for them. The MATH table is unchanged.
- {pua} of those size variants and assembly parts, which have no Unicode code point in the original,
  are given Private Use Area code points from U+E000, so that a text renderer can draw them.
- OpenType layout tables (GSUB, GPOS, GDEF) and hinting removed; the font renamed "Fractadyne Math".

The unmodified original: {url}
(SHA-256 of the archive this was made from: {sha})
"""

if __name__ == "__main__":
    main()
