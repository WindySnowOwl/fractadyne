#!/usr/bin/env python3
"""Every message of every language in one CSV table: one row per message, the message id in the
first column, then one column per language (English first, the others by tag).

    python scripts/i18n_to_csv.py                          # writes local/fractadyne-translations.csv
    python scripts/i18n_to_csv.py -o out.csv
    python scripts/i18n_to_csv.py --lang-dir path/to/i18n  # another folder of <tag>/fractadyne.ftl

A cell holds the message's value as the language file writes it: a value over several lines keeps
its line breaks (continuation lines lose their common indent), and a plural or other selector is
kept as Fluent source (`{ $count -> [one] ... *[other] ... }`), so nothing is lost. A message a
language does not have is an empty cell. Rows follow the English file; a message only a translation
has (the checker calls it unknown) comes after them.

The file is UTF-8 with a byte-order mark, which spreadsheet programs need to read it as UTF-8
(Japanese, Chinese and Korean otherwise come out garbled); `--no-bom` leaves it out. The language
files are read with scripts/check_i18n.py's parser, so a message reads here as the checker reads
it. Standard library only.
"""

import argparse
import csv
import importlib.util
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FILE = 'fractadyne.ftl'


def checker():
    """scripts/check_i18n.py, loaded by path (works under `python -I`, which leaves the script's
    folder off the import path)."""
    path = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'check_i18n.py')
    spec = importlib.util.spec_from_file_location('check_i18n', path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def value(lines):
    """A message's value from its lines as written: the text after `=`, then the indented lines
    without their common indent; blank lines at the end (the gap before the next message) dropped."""
    first, rest = (lines[0].strip(), lines[1:]) if lines else ('', [])
    while rest and not rest[-1].strip():
        rest = rest[:-1]
    indents = [len(l) - len(l.lstrip()) for l in rest if l.strip()]
    cut = min(indents) if indents else 0
    out = ([first] if first else []) + [l[cut:].rstrip() for l in rest]
    return '\n'.join(out).strip('\n')


def languages(lang_dir):
    """The language tags with a file in `lang_dir`: English first, the others by tag."""
    tags = sorted(t for t in os.listdir(lang_dir) if os.path.isfile(os.path.join(lang_dir, t, FILE)))
    if 'en' not in tags:
        sys.exit(f'{lang_dir}: no en/{FILE} (the English file gives the rows)')
    return ['en'] + [t for t in tags if t != 'en']


def main():
    sys.stdout.reconfigure(encoding='utf-8')
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('-o', '--output', default=os.path.join(ROOT, 'local', 'fractadyne-translations.csv'))
    ap.add_argument('--lang-dir', default=os.path.join(ROOT, 'i18n'))
    ap.add_argument('--no-bom', action='store_true', help='write UTF-8 without a byte-order mark')
    args = ap.parse_args()

    parse = checker().parse
    tags = languages(args.lang_dir)
    table = {}
    for tag in tags:
        path = os.path.join(args.lang_dir, tag, FILE)
        with open(path, encoding='utf-8', newline='') as f:
            text = f.read().replace('\r\n', '\n')
        errors = []
        msgs = parse(text, f'{tag}/{FILE}', errors)
        if errors:
            print(f'{tag}: {len(errors)} parse problem(s); run scripts/check_i18n.py for them', file=sys.stderr)
        table[tag] = {mid: value(m.lines) for mid, m in msgs.items()}

    keys = list(table['en'])
    seen = set(keys)
    for tag in tags[1:]:
        for mid in table[tag]:
            if mid not in seen:
                keys.append(mid)
                seen.add(mid)

    os.makedirs(os.path.dirname(os.path.abspath(args.output)), exist_ok=True)
    with open(args.output, 'w', encoding='utf-8' if args.no_bom else 'utf-8-sig', newline='') as f:
        w = csv.writer(f)
        w.writerow(['key'] + tags)
        for mid in keys:
            w.writerow([mid] + [table[tag].get(mid, '') for tag in tags])

    print(f'{args.output}: {len(keys)} messages x {len(tags)} languages ({", ".join(tags)})')
    for tag in tags[1:]:
        missing = sum(1 for mid in table['en'] if mid not in table[tag])
        extra = sum(1 for mid in table[tag] if mid not in table['en'])
        if missing or extra:
            print(f'  {tag}: {missing} missing, {extra} not in English')
    return 0


if __name__ == '__main__':
    sys.exit(main())
