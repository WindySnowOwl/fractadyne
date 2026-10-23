#!/usr/bin/env python3
"""Merge a reviewed translations table back into the language files: the counterpart of
scripts/i18n_to_csv.py. Every cell that differs from its language's file replaces that message.

    python scripts/i18n_from_csv.py REVIEW.csv                    # write the changed messages
    python scripts/i18n_from_csv.py REVIEW.csv --dry-run          # only report them
    python scripts/i18n_from_csv.py REVIEW.csv --fix-conventions  # also correct convention slips
    python scripts/i18n_from_csv.py --selfcheck                   # prove the rules on a scratch copy

The table is the one i18n_to_csv.py writes: a `key` column, then one column per language tag. A
spreadsheet may have saved it with ';' or tabs between the cells, and with or without a byte-order
mark; it must be UTF-8 (a spreadsheet's plain "CSV" can save in the system code page, which turns
Japanese, Chinese, Korean and Cyrillic into '?': save as "CSV UTF-8").

What is taken, and what is not:
- A row whose key is not an English message is skipped (the English file names every message).
- A column whose tag has no folder in --lang-dir is skipped. English is never written: it is the
  source, and its messages are constants in the code; its differing cells are only counted.
- A cell equal to the file's message (as i18n_to_csv.py reads it) is no change, and an empty cell
  never deletes a translation. A message a file lacks is added after the message before it.
- Each language's changes are checked as a whole file, by scripts/check_i18n.py's rules, before
  it is written: a language with any error is not written at all (the others still are). So a
  cell cannot bring in an argument the English lacks, a plural category the language does not
  have, a link, a control or bidirectional character, or text past the length guard.
- Conventions: where a file writes something one way at least 9 times in 10 (20 or more cases),
  a changed message that writes it the other way is reported: the dash between words (' — ' or
  ' – '), the space before ; : ! ? » and after « (a space, a no-break space or a narrow one), and
  whether a space separates Japanese, Chinese or Korean from Latin letters, digits or a
  placeable. --fix-conventions writes the file's way instead.

A message is written as the files write it: on its key's line when it is one line, else its first
line there and the rest indented by four; a value that opens with a selector starts on the next
line. Line endings are kept. Nothing is committed: review the result with `git diff`, then run
scripts/check_i18n.py. Standard library only.
"""

import argparse
import csv
import importlib.util
import io
import os
import re
import shutil
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
HERE = os.path.dirname(os.path.abspath(__file__))
FILE = 'fractadyne.ftl'
TAG = re.compile(r'^[A-Za-z]{2,3}(-[A-Za-z0-9]{2,8})*$')


def load(name):
    """A sibling script as a module, by path (works under `python -I`)."""
    spec = importlib.util.spec_from_file_location(name, os.path.join(HERE, name + '.py'))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


check = load('check_i18n')
export = load('i18n_to_csv')


# --- the table -------------------------------------------------------------------------------

def read_table(path):
    """(language tags, {key: {tag: cell}}) from a CSV in UTF-8, the delimiter found from its header."""
    raw = open(path, 'rb').read()
    try:
        text = raw.decode('utf-8-sig')
    except UnicodeDecodeError as e:
        sys.exit(f'{path}: not UTF-8 ({e}); save it as "CSV UTF-8"')
    head = text.split('\n', 1)[0]
    delim = max([',', ';', '\t'], key=head.count)
    rows = list(csv.reader(io.StringIO(text, newline=''), delimiter=delim))
    if not rows or not rows[0] or rows[0][0].strip().lower() != 'key':
        sys.exit(f'{path}: the first column must be "key" (found {rows[0][:1] if rows else "nothing"})')
    tags = [t.strip() for t in rows[0][1:]]
    table = {}
    for r in rows[1:]:
        if not r or not r[0].strip():
            continue
        table[r[0].strip()] = {t: (r[i + 1] if i + 1 < len(r) else '') for i, t in enumerate(tags)}
    return tags, table


def normalize(cell):
    """A cell as i18n_to_csv.py writes a value: line breaks as '\\n', no trailing spaces, no blank
    lines at either end, the first line not indented."""
    lines = [l.rstrip() for l in cell.replace('\r\n', '\n').replace('\r', '\n').split('\n')]
    while lines and not lines[0]:
        lines.pop(0)
    while lines and not lines[-1]:
        lines.pop()
    if lines:
        lines[0] = lines[0].lstrip()
    return '\n'.join(lines)


# --- conventions -----------------------------------------------------------------------------

CJK = '぀-ヿ㐀-䶿一-鿿가-힯'
SPACES = {' ': 'a space', ' ': 'a no-break space', ' ': 'a narrow no-break space'}


def _space_forms(before=None, after=None):
    """The ways of writing a space before `before` or after `after`: {name: (regex, text)}."""
    out = {}
    for ch, name in SPACES.items():
        if before:
            out[name] = (re.escape(ch) + '(?=' + re.escape(before) + ')', ch)
        else:
            out[name] = ('(?<=' + re.escape(after) + ')' + re.escape(ch), ch)
    return out


# Each: (what, {form: (regex matching one case written that way, the text that writes it so)}).
CONVENTIONS = [('the dash between words', {"' — '": ('(?<= )—(?= )', '—'), "' – '": ('(?<= )–(?= )', '–')})]
CONVENTIONS += [(f"the space before '{p}'", _space_forms(before=p)) for p in ';:!?»']
CONVENTIONS += [("the space after '«'", _space_forms(after='«'))]
CONVENTIONS += [(
    'a space between CJK text and Latin letters, digits or a placeable',
    {
        'a space': (f'(?<=[{CJK}]) (?=[A-Za-z0-9.{{])|(?<=[A-Za-z0-9}}]) (?=[{CJK}])', ' '),
        'no space': (f'(?<=[{CJK}])(?=[A-Za-z0-9.{{])|(?<=[A-Za-z0-9}}])(?=[{CJK}])', ''),
    },
)]


def house_style(text):
    """The conventions `text` keeps: [(what, its form, the other forms, how many cases each)]."""
    out = []
    for what, forms in CONVENTIONS:
        counts = {name: len(re.findall(rx, text)) for name, (rx, _) in forms.items()}
        total = sum(counts.values())
        name, top = max(counts.items(), key=lambda kv: kv[1])
        if total >= 20 and top >= 0.9 * total:
            out.append((what, name, forms, counts))
    return out


def conventions(value, style, fix):
    """(value, [complaints]): where `value` writes a convention of `style` the other way."""
    notes = []
    for what, name, forms, counts in style:
        for other, (rx, _) in forms.items():
            n = len(re.findall(rx, value)) if other != name else 0
            if n:
                tally = ', '.join(f'{k} {v}' for k, v in counts.items() if v)
                notes.append(f'{what}: {other} ({n}) where the file writes {name} ({tally})' + (' - fixed' if fix else ''))
                if fix:
                    value = re.sub(rx, forms[name][1], value)
    return value, notes


# --- the files -------------------------------------------------------------------------------

def blocks(lines):
    """{message id: (first line, line after its last)} of a file's lines."""
    out = {}
    for i, line in enumerate(lines):
        m = re.match(r'^([A-Za-z][A-Za-z0-9_-]*)\s*=', line)
        if not m:
            continue
        last, j = i, i + 1
        while j < len(lines):
            if not lines[j].strip():
                j += 1
                continue
            if not lines[j][0].isspace():
                break
            last = j
            j += 1
        out[m.group(1)] = (i, last + 1)
    return out


def written(mid, value):
    """The file's lines for a message."""
    lines = value.split('\n')
    indent = lambda ls: ['    ' + l if l else '' for l in ls]
    if len(lines) == 1:
        return [f'{mid} = {value}']
    if re.match(r'^\{\s*\$[A-Za-z][A-Za-z0-9_-]*\s*->', lines[0]):
        return [f'{mid} ='] + indent(lines)
    return [f'{mid} = {lines[0]}'] + indent(lines[1:])


def merge(text, changes, order):
    """`text` (a file, '\\n' line ends) with each message of `changes` ({id: value}) replaced, or
    added after the message before it in `order` (the English ids)."""
    lines = text.split('\n')
    for mid, value in changes.items():
        found = blocks(lines)
        if mid in found:
            i, j = found[mid]
            lines[i:j] = written(mid, value)
            continue
        prev = [k for k in order[: order.index(mid)] if k in found]
        at = found[prev[-1]][1] if prev else len(lines)
        lines[at:at] = written(mid, value)
    return '\n'.join(lines)


def run(csv_path, lang_dir, dry_run=False, fix=False, out=print):
    """Merges the table into the files; returns (exit code, {tag: [changed ids]})."""
    tags, table = read_table(csv_path)
    errs = []
    with open(os.path.join(lang_dir, 'en', FILE), encoding='utf-8', newline='') as f:
        eng = check.parse(f.read().replace('\r\n', '\n'), 'en', errs)
    if errs:
        out(f'the English file does not parse ({len(errs)} problem(s)); run scripts/check_i18n.py')
        return 1, {}
    pv = check.plural_vars(eng)
    order = list(eng)
    english = export.read_values(lang_dir, 'en', check.parse)

    unknown = [k for k in table if k not in eng]
    if unknown:
        out(f'skipped {len(unknown)} row(s) whose key is not an English message: {", ".join(unknown[:8])}{" ..." if len(unknown) > 8 else ""}')
    if 'en' in tags:
        n = sum(1 for k, row in table.items() if k in english and row.get('en', '') and normalize(row['en']) != english[k])
        if n:
            out(f'en: {n} cell(s) differ from the English file; not imported (English is the source: edit i18n/en/{FILE})')

    code, done = 0, {}
    scratch = tempfile.mkdtemp(prefix='i18n-import-')
    try:
        for tag in tags:
            if tag == 'en':
                continue
            path = os.path.join(lang_dir, tag, FILE)
            if not TAG.match(tag) or not os.path.isfile(path):
                out(f'{tag}: no {tag}/{FILE} in {lang_dir}; column skipped')
                continue
            raw = open(path, encoding='utf-8', newline='').read()
            nl = '\r\n' if '\r\n' in raw else '\n'
            text = raw.replace('\r\n', '\n')
            current = export.read_values(lang_dir, tag, check.parse)
            style = house_style(text)
            changes, notes = {}, []
            for mid in order:
                cell = table.get(mid, {}).get(tag, '')
                new = normalize(cell)
                if not new or new == current.get(mid):
                    continue
                new, complaints = conventions(new, style, fix)
                notes += [f'  {mid}: {c}' for c in complaints]
                if new != current.get(mid):
                    changes[mid] = new
            if not changes:
                for n in notes:
                    out(f'{tag}{n}')
                continue
            candidate = merge(text, changes, order)
            probe = os.path.join(scratch, tag, FILE)
            os.makedirs(os.path.dirname(probe), exist_ok=True)
            with open(probe, 'w', encoding='utf-8', newline='') as f:
                f.write(candidate)
            buf = io.StringIO()
            old_stdout, sys.stdout = sys.stdout, buf
            try:
                _, _, n_err, n_warn = check.check_file(probe, eng, pv, quiet=False)
            finally:
                sys.stdout = old_stdout
            listed = ', '.join(list(changes)[:6]) + (' ...' if len(changes) > 6 else '')
            if n_err:
                out(f'{tag}: NOT WRITTEN - {n_err} error(s) in the file as it would be ({len(changes)} message(s): {listed}):')
                for line in buf.getvalue().splitlines():
                    if 'ERROR' in line:
                        out('  ' + line.split(': ', 1)[-1])
                code = 1
                continue
            out(f'{tag}: {len(changes)} message(s) {"would change" if dry_run else "changed"}: {listed}' + (f' ({n_warn} checker warning(s))' if n_warn else ''))
            for n in notes:
                out(n)
            if not dry_run:
                with open(path, 'w', encoding='utf-8', newline='') as f:
                    f.write(candidate.replace('\n', nl))
            done[tag] = list(changes)
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    if not done and code == 0:
        out('no changes')
    return code, done


# --- selfcheck -------------------------------------------------------------------------------

def selfcheck():
    """Export a scratch copy of the files, edit the table as a reviewer would, import it, and check
    every rule: what is taken, what is refused, conventions, a ';' table, the English column."""
    ok = True
    d = tempfile.mkdtemp(prefix='i18n-selfcheck-')
    try:
        lang = os.path.join(d, 'i18n')
        for tag in ['en', 'de', 'es', 'fr', 'ja']:
            os.makedirs(os.path.join(lang, tag))
            shutil.copy(os.path.join(ROOT, 'i18n', tag, FILE), os.path.join(lang, tag, FILE))
        before = {t: open(os.path.join(lang, t, FILE), 'rb').read() for t in os.listdir(lang)}
        table_csv = os.path.join(d, 'table.csv')
        base = export.export(lang, table_csv)
        quiet = lambda *a: None

        def case(name, good):
            nonlocal ok
            print(f'selfcheck: {name}: {"PASS" if good else "FAIL"}')
            ok = ok and good

        def table_with(edits, delim=','):
            path = os.path.join(d, f'edit{len(os.listdir(d))}.csv')
            tags = list(base)
            with open(path, 'w', encoding='utf-8-sig', newline='') as f:
                w = csv.writer(f, delimiter=delim)
                w.writerow(['key'] + tags)
                for mid in base['en']:
                    w.writerow([mid] + [edits.get((mid, t), base[t].get(mid, '')) for t in tags])
            return path

        code, done = run(table_csv, lang, out=quiet)
        case('the unedited table changes nothing', code == 0 and not done and all(open(os.path.join(lang, t, FILE), 'rb').read() == b for t, b in before.items()))

        julia = base['fr']['menu-show-julia-hover']
        edits = {
            ('menu-show-both', 'es'): 'Ambos, enlazados (prueba)',
            ('menu-show-julia-hover', 'fr'): julia + '\n\nUne ligne de plus.',
        }
        code, done = run(table_with(edits), lang, out=quiet)
        after = export.export(lang, os.path.join(d, 'after.csv'))
        same_elsewhere = all(after[t][k] == base[t][k] for t in base for k in base[t] if (k, t) not in edits)
        case('two edited cells are written, and nothing else changes',
             code == 0 and done == {'es': ['menu-show-both'], 'fr': ['menu-show-julia-hover']}
             and after['es']['menu-show-both'] == 'Ambos, enlazados (prueba)'
             and after['fr']['menu-show-julia-hover'] == julia + '\n\nUne ligne de plus.' and same_elsewhere)
        base = after

        de = open(os.path.join(lang, 'de', FILE), 'rb').read()
        code, done = run(table_with({('menu-show-both', 'de'): 'Beide { $bogus }'}), lang, out=quiet)
        case('a cell with an argument the English lacks is refused, its file untouched', code == 1 and 'de' not in done and open(os.path.join(lang, 'de', FILE), 'rb').read() == de)

        code, done = run(table_with({('menu-show-both', 'de'): '', ('menu-show-both', 'en'): 'Changed'}), lang, out=quiet)
        case('an empty cell deletes nothing, and English is never written', code == 0 and not done and open(os.path.join(lang, 'de', FILE), 'rb').read() == de)

        lossy_fr = base['fr']['app-gradient-ggr-saved-lossy'].replace(' ;', ' ;')
        lossy_ja = base['ja']['app-gradient-ggr-saved-lossy'].replace('{ $count }個', '{ $count } 個')
        slips = {('app-gradient-ggr-saved-lossy', 'fr'): lossy_fr + ' X', ('app-gradient-ggr-saved-lossy', 'ja'): lossy_ja + 'X'}
        lines = []
        run(table_with(slips), lang, dry_run=True, out=lines.append)
        case('convention slips are reported (French space before ;, Japanese space before a counter)',
             any('narrow no-break space' in l for l in lines) and any('no space' in l for l in lines))
        code, done = run(table_with(slips, delim=';'), lang, fix=True, out=quiet)
        after = export.export(lang, os.path.join(d, 'after2.csv'))
        case("--fix-conventions writes the file's way, from a ';' table",
             code == 0 and after['fr']['app-gradient-ggr-saved-lossy'] == base['fr']['app-gradient-ggr-saved-lossy'] + ' X'
             and after['ja']['app-gradient-ggr-saved-lossy'] == base['ja']['app-gradient-ggr-saved-lossy'] + 'X')

        bad = os.path.join(d, 'cp1252.csv')
        with open(bad, 'wb') as f:
            f.write('key,en\nmenu-show-both,Both é\n'.encode('cp1252'))
        try:
            run(bad, lang, out=quiet)
            case('a table not in UTF-8 is refused', False)
        except SystemExit:
            case('a table not in UTF-8 is refused', True)

        errs = []
        for tag in ['de', 'es', 'fr', 'ja']:
            buf = io.StringIO()
            old_stdout, sys.stdout = sys.stdout, buf
            try:
                eng = check.parse(open(os.path.join(lang, 'en', FILE), encoding='utf-8').read().replace('\r\n', '\n'), 'en', [])
                _, _, n_err, _ = check.check_file(os.path.join(lang, tag, FILE), eng, check.plural_vars(eng), quiet=True)
            finally:
                sys.stdout = old_stdout
            errs.append(n_err)
        case('every file written passes check_i18n', not any(errs))
    finally:
        shutil.rmtree(d, ignore_errors=True)
    print('selfcheck', 'PASS' if ok else 'FAIL')
    return 0 if ok else 1


def main():
    sys.stdout.reconfigure(encoding='utf-8')
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('csv', nargs='?', help='the reviewed table')
    ap.add_argument('--lang-dir', default=os.path.join(ROOT, 'i18n'))
    ap.add_argument('--dry-run', action='store_true', help='report the changes, write nothing')
    ap.add_argument('--fix-conventions', action='store_true', help="write a file's own conventions where a cell slips")
    ap.add_argument('--selfcheck', action='store_true')
    args = ap.parse_args()
    if args.selfcheck:
        return selfcheck()
    if not args.csv:
        ap.error('a CSV file is required (or --selfcheck)')
    code, _ = run(args.csv, args.lang_dir, args.dry_run, args.fix_conventions)
    return code


if __name__ == '__main__':
    sys.exit(main())
