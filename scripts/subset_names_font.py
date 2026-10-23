#!/usr/bin/env python3
"""subset_names_font.py [--cache DIR] - build the tiny fonts that draw the languages' own names in
the language menu (design/multilingual.md section 8, "the names font").

The language menu lists every language in its own name, in every interface language, so a reader
of Japanese can find 日本語 in an English interface. The interface fonts have no Han characters,
and a whole CJK font is 5-30 MB, so the names are drawn from subsets of Noto Sans JP and Noto Sans
SC holding only the characters of those names (a few kilobytes). Japanese and Chinese each get the
font of their own typography (Han unification: the same character is drawn differently).

Reads `language-name` and `language-script` from every i18n/<tag>/fractadyne.ftl (and always
includes 日本語 and 简体中文), downloads the two Noto fonts (SIL Open Font License 1.1) into a cache
folder once, and writes:

    crates/fractadyne-app/assets/fonts/NamesJP.otf   (script Jpan)
    crates/fractadyne-app/assets/fonts/NamesSC.otf   (script Hans)

Run it after a language with one of those scripts is added or renamed; the font-coverage test
fails until then. Needs fontTools (pip install fonttools).
"""
import io
import os
import re
import sys
import urllib.request

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FONTS = os.path.join(ROOT, 'crates', 'fractadyne-app', 'assets', 'fonts')
BASE = 'https://cdn.jsdelivr.net/gh/notofonts/noto-cjk@main/Sans/SubsetOTF'
SOURCES = {
    'Jpan': ('JP', 'NamesJP.otf', '日本語'),
    'Hans': ('SC', 'NamesSC.otf', '简体中文'),
}


def names_by_script():
    out = {k: set(v[2]) for k, v in SOURCES.items()}
    i18n = os.path.join(ROOT, 'i18n')
    for tag in sorted(os.listdir(i18n)):
        p = os.path.join(i18n, tag, 'fractadyne.ftl')
        if not os.path.isfile(p):
            continue
        text = io.open(p, encoding='utf-8').read()
        name = re.search(r'^language-name\s*=\s*(.+)$', text, re.M)
        script = re.search(r'^language-script\s*=\s*(\S+)', text, re.M)
        if name and script and script.group(1) in out:
            out[script.group(1)] |= {c for c in name.group(1).strip() if ord(c) > 0x2E7F}
    return out


def fetch(region, cache):
    os.makedirs(cache, exist_ok=True)
    path = os.path.join(cache, f'NotoSans{region}-Regular.otf')
    if not os.path.isfile(path):
        url = f'{BASE}/{region}/NotoSans{region}-Regular.otf'
        print('downloading', url)
        with urllib.request.urlopen(url, timeout=120) as r, open(path + '.part', 'wb') as f:
            f.write(r.read())
        os.replace(path + '.part', path)
    return path


def main():
    from fontTools import subset
    from fontTools.ttLib import TTFont
    cache = os.path.join(ROOT, 'target', 'names-font-cache')
    if '--cache' in sys.argv:
        cache = sys.argv[sys.argv.index('--cache') + 1]
    for script, chars in names_by_script().items():
        region, out_name, _ = SOURCES[script]
        src = fetch(region, cache)
        opts = subset.Options()
        opts.layout_features = []
        opts.hinting = False
        opts.name_IDs = ['*']
        opts.notdef_outline = True
        opts.drop_tables += ['DSIG', 'vhea', 'vmtx', 'VORG', 'BASE']
        font = TTFont(src)
        s = subset.Subsetter(options=opts)
        s.populate(unicodes=[ord(c) for c in sorted(chars)])
        s.subset(font)
        out = os.path.join(FONTS, out_name)
        font.save(out)
        have = set(TTFont(out).getBestCmap())
        missing = [c for c in chars if ord(c) not in have]
        if missing:
            print(f'{out_name}: the font lacks {missing}', file=sys.stderr)
            return 1
        print(f'{out_name}: {"".join(sorted(chars))} ({os.path.getsize(out):,} bytes)')
    return 0


if __name__ == '__main__':
    sys.stdout.reconfigure(encoding='utf-8')
    sys.exit(main())
