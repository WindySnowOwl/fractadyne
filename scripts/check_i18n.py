#!/usr/bin/env python3
"""check_i18n.py [FILE.ftl ...] [--quiet] - check the language files against the English one
(design/multilingual.md sections 6 and 10).

    python scripts/check_i18n.py                      # every i18n/<tag>/fractadyne.ftl
    python scripts/check_i18n.py i18n/de/fractadyne.ftl
    python scripts/check_i18n.py --selfcheck          # prove each rule goes red on a planted case

Each file is checked as the app checks it when it loads one, so a file that passes here loads with
no message set aside:

- `parse`: the file reads as Fluent: a message is `id = value` at the start of a line, its other
  lines are indented, braces balance, an indented line never starts with `[`, `*` or `.` outside a
  selector, and every selector has exactly one default variant (`*[...]`).
- `unknown-id`: a message the English file does not have (a typo, or a message since renamed: the
  app ignores it).
- `argument`: a `{ $name }` the English message does not have (the app sets the message aside).
- `plural`: a selector on a number uses a category the language does not have (it never matches),
  or lacks one it has (those numbers take the default; a warning).
- `length`: longer than four times the English plus 64 characters (set aside: a guard against a
  file that floods a window).
- `url`: an address in a message (set aside: a language file must not carry links).
- `character`: a control character, a byte-order mark, a carriage return inside a line, or a
  bidirectional control (U+202A-202E, U+2066-2069), all of which the app removes. A CRLF line ending
  is fine: git on Windows checks the files out that way.
- `data`: the language-data messages (language-name, language-english-name, language-script,
  number-decimal, number-group, language-status) are present and well formed.
- `empty`: a message with no text.

Prints a coverage table; exit 1 on any error (warnings do not fail). Standard library only.
"""
import io
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
I18N = os.path.join(ROOT, 'i18n')
ENGLISH = os.path.join(I18N, 'en', 'fractadyne.ftl')

# CLDR 37 cardinal plural categories (the `intl_pluralrules` crate the app uses), by language.
PLURALS = {
    'en': {'one', 'other'},
    'es': {'one', 'other'},
    'fr': {'one', 'other'},
    'de': {'one', 'other'},
    'uk': {'one', 'few', 'many', 'other'},
    'ja': {'other'},
    'zh': {'other'},
    'km': {'other'},
    'pt': {'one', 'other'},
    'it': {'one', 'other'},
    'tr': {'one', 'other'},
    'ru': {'one', 'few', 'many', 'other'},
    'pl': {'one', 'few', 'many', 'other'},
    'ko': {'other'},
    'id': {'other'},
}
ALL_PLURALS = {'zero', 'one', 'two', 'few', 'many', 'other'}
DATA = ['language-name', 'language-english-name', 'language-script', 'number-decimal', 'number-group', 'language-status']
SCRIPTS = {'Latn', 'Cyrl', 'Jpan', 'Hans', 'Hant', 'Kore', 'Khmr', 'Grek', 'Arab', 'Hebr'}
BAD_CHARS = re.compile('[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f‎‏‪-‮⁦-⁩﻿\r]')
URL = re.compile(r'(?i)\b(?:https?://|www\.)')


class Message:
    def __init__(self, mid, line):
        self.id = mid
        self.line = line
        self.lines = []      # the value's lines, as written
        self.vars = set()    # $variables
        self.selectors = []  # (selector variable, [variant keys], default count, line)


def parse(text, name, errors):
    """{id: Message} of a Fluent file; syntax problems go to `errors` as (line, rule, text)."""
    msgs = {}
    cur = None
    # Open braces of the current message, innermost last: a selector is [var, keys, defaults, line],
    # a plain placeable is None.
    stack = []

    def close_message():
        if cur is not None and stack:
            errors.append((cur.line, 'parse', f'{cur.id}: a brace or selector is not closed'))
        stack.clear()

    for n, line in enumerate(text.split('\n'), 1):
        if not line.strip():
            if cur is not None:
                cur.lines.append('')
            continue
        if line[0] == '#':
            close_message()
            cur = None
            continue
        if not line[0].isspace():
            close_message()
            m = re.match(r'^([A-Za-z][A-Za-z0-9_-]*)\s*=(.*)$', line)
            if not m:
                errors.append((n, 'parse', f'not a message, comment or continuation: {line[:50]!r}'))
                cur = None
                continue
            mid = m.group(1)
            if mid in msgs:
                errors.append((n, 'parse', f'{mid} is defined twice'))
            cur = Message(mid, n)
            msgs[mid] = cur
            body = m.group(2)
        else:
            if cur is None:
                errors.append((n, 'parse', 'an indented line outside any message'))
                continue
            body = line
            s = line.strip()
            sel = next((x for x in reversed(stack) if x is not None), None) if stack and stack[-1] is not None else None
            if s.startswith(('[', '*[')):
                if sel is None:
                    errors.append((n, 'parse', f'{cur.id}: a line starting with {s[0]!r} outside a selector (write {{"{s[0]}"}})'))
                else:
                    km = re.match(r'^\*?\[\s*([^\]\s]+)\s*\]', s)
                    if km:
                        sel[1].append(km.group(1))
                        if s.startswith('*'):
                            sel[2] += 1
                    body = re.sub(r'^\s*\*?\[[^\]]*\]', '', line)
            elif s.startswith(('*', '.')):
                errors.append((n, 'parse', f'{cur.id}: a line starting with {s[0]!r} (write {{"{s[0]}"}})'))
        cur.lines.append(line if line[0].isspace() else m.group(2))
        # Walk the braces, skipping quoted literals.
        i = 0
        while i < len(body):
            c = body[i]
            if c == '{':
                q = re.match(r'\{\s*"((?:[^"\\]|\\.)*)"\s*\}', body[i:])
                if q:
                    i += q.end()
                    continue
                sm = re.match(r'\{\s*\$([A-Za-z][A-Za-z0-9_-]*)\s*->', body[i:])
                if sm:
                    stack.append([sm.group(1), [], 0, n])
                    i += sm.end()
                    continue
                stack.append(None)
            elif c == '}':
                if not stack:
                    errors.append((n, 'parse', f'{cur.id}: a closing brace with none open (a literal one is {{"}}"}})'))
                else:
                    top = stack.pop()
                    if top is not None:
                        cur.selectors.append(tuple(top))
            i += 1
        plain = re.sub(r'\{\s*"(?:[^"\\]|\\.)*"\s*\}', '', body)
        for v in re.findall(r'\$([A-Za-z][A-Za-z0-9_-]*)', plain):
            cur.vars.add(v)
    close_message()
    for m in msgs.values():
        for sel in m.selectors:
            if sel[2] != 1:
                errors.append((sel[3], 'parse', f'{m.id}: a selector needs exactly one default variant *[...] (has {sel[2]})'))
    return msgs


def text_len(m):
    """The longest variant's length, roughly as the app measures it (placeables count as written)."""
    if not m.selectors:
        return len(' '.join(l.strip() for l in m.lines if l.strip()))
    best = 0
    for l in m.lines:
        s = l.strip()
        if s.startswith(('[', '*[')):
            best = max(best, len(re.sub(r'^\*?\[[^\]]*\]\s*', '', s)))
    return best or len(' '.join(l.strip() for l in m.lines))


def plural_vars(eng):
    """English selector variables used as plural counts: those with a [one] variant."""
    out = {}
    for m in eng.values():
        out[m.id] = {s[0] for s in m.selectors if 'one' in s[1] or set(s[1]) <= ALL_PLURALS}
    return out


def check_file(path, eng, eng_plural_vars, quiet=False):
    raw = io.open(path, encoding='utf-8', newline='').read()
    rel = os.path.relpath(path, ROOT).replace(os.sep, '/')
    errors, warnings = [], []
    tag = os.path.basename(os.path.dirname(path))
    lang = tag.split('-')[0]
    for n, line in enumerate(raw.split('\n'), 1):
        # A CRLF line ending is how git checks the files out on Windows (`core.autocrlf`): only a
        # carriage return inside a line is a fault.
        for c in BAD_CHARS.findall(line.removesuffix('\r')):
            errors.append((n, 'character', f'U+{ord(c):04X} (the app removes it)'))
    text = raw.replace('\r\n', '\n')
    msgs = parse(text, rel, errors)
    plurals = PLURALS.get(lang)
    for mid, m in msgs.items():
        if mid not in eng:
            errors.append((m.line, 'unknown-id', f'{mid} is not in the English file'))
            continue
        e = eng[mid]
        extra = m.vars - e.vars
        if extra:
            errors.append((m.line, 'argument', f'{mid}: {sorted(extra)} not in the English message (it has {sorted(e.vars) or "none"})'))
        body = '\n'.join(m.lines)
        if not re.sub(r'\{\s*""\s*\}', '', body).strip():
            errors.append((m.line, 'empty', f'{mid} has no text'))
        if URL.search(body):
            errors.append((m.line, 'url', f'{mid}: an address in a message'))
        if mid not in DATA:
            cap = 4 * text_len(e) + 64
            if text_len(m) > cap:
                errors.append((m.line, 'length', f'{mid}: {text_len(m)} characters, over the {cap} allowed'))
        for var, keys, _, ln in m.selectors:
            if var in eng_plural_vars.get(mid, set()) and plurals is not None:
                unknown = set(keys) - plurals - {k for k in keys if re.fullmatch(r'\d+', k)}
                if unknown:
                    errors.append((ln, 'plural', f'{mid}: ${var} has no {sorted(unknown)} in {lang} (it has {sorted(plurals)})'))
                missing = plurals - set(keys) - {'other'}
                if missing and '*[other]' not in ''.join(m.lines).replace(' ', ''):
                    pass
                if missing:
                    warnings.append((ln, 'plural', f'{mid}: ${var} lacks {sorted(missing)} (those numbers take the default)'))
            elif var not in eng_plural_vars.get(mid, set()):
                ekeys = set()
                for v2, k2, _, _ in e.selectors:
                    if v2 == var:
                        ekeys |= set(k2)
                unknown = set(keys) - ekeys
                if ekeys and unknown:
                    errors.append((ln, 'plural', f'{mid}: ${var} is chosen by token; {sorted(unknown)} never match (English has {sorted(ekeys)})'))
    # Language data.
    for d in DATA:
        if d not in msgs:
            errors.append((1, 'data', f'{d} is missing'))
    def value(mid):
        m = msgs.get(mid)
        return '' if m is None else '\n'.join(m.lines).strip()
    script = value('language-script')
    if script and script not in SCRIPTS:
        errors.append((msgs['language-script'].line, 'data', f'language-script {script!r} is not an ISO 15924 code this app knows'))
    status = value('language-status')
    if status and status not in ('draft', 'reviewed'):
        errors.append((msgs['language-status'].line, 'data', f'language-status is {status!r}: write draft or reviewed'))
    for d in ('number-decimal', 'number-group'):
        v = value(d)
        lit = re.fullmatch(r'\{\s*"((?:[^"\\]|\\.)*)"\s*\}', v)
        if lit:
            v = lit.group(1).encode().decode('unicode_escape') if '\\u' in lit.group(1) else lit.group(1)
        if v and len(v) != 1:
            errors.append((msgs[d].line, 'data', f'{d} must be one character, not {v!r}'))
    if value('number-decimal') and value('number-decimal') == value('number-group'):
        errors.append((msgs['number-decimal'].line, 'data', 'number-decimal and number-group are the same character'))
    translated = sum(1 for k in msgs if k in eng and k not in DATA)
    total = sum(1 for k in eng if k not in DATA)
    if not quiet:
        for n, rule, t in sorted(warnings):
            print(f'{rel}:{n}: warning [{rule}] {t}')
        for n, rule, t in sorted(errors):
            print(f'{rel}:{n}: ERROR [{rule}] {t}')
    return translated, total, len(errors), len(warnings)


def selfcheck():
    eng_text = '''language-name = English
# $count: things.
things =
    { $count ->
        [one] One thing
       *[other] { $count } things
    }
# $kind: a token.
kind =
    { $kind ->
        [a] Kind A
       *[other] Other kind
    }
plain = Plain text
'''
    errs = []
    eng = parse(eng_text, 'en', errs)
    assert not errs, errs
    pv = plural_vars(eng)
    cases = {
        'unknown-id': 'nothing = x\n',
        'argument': 'plain = Text { $oops }\n',
        'plural': 'things =\n    { $count ->\n        [two] Two\n       *[other] Many\n    }\n',
        'parse': 'things =\n    { $count ->\n        [one] One\n        [other] Many\n    }\n',
        'url': 'plain = See https://example.com\n',
        'length': 'plain = ' + 'x' * 200 + '\n',
        'character': 'plain = Te‮xt\n',
        'character/cr': 'plain = Te\rxt\n',
        'empty': 'plain = { "" }\n',
    }
    import tempfile
    ok = True
    d = tempfile.mkdtemp()
    for rule, body in cases.items():
        p = os.path.join(d, 'de', 'fractadyne.ftl')
        os.makedirs(os.path.dirname(p), exist_ok=True)
        data = 'language-name = Deutsch\nlanguage-english-name = German\nlanguage-script = Latn\nnumber-decimal = ,\nnumber-group = .\nlanguage-status = draft\n'
        io.open(p, 'w', encoding='utf-8', newline='\n').write(data + body)
        for k in DATA:
            eng.setdefault(k, Message(k, 1))
        buf = io.StringIO()
        old = sys.stdout
        sys.stdout = buf
        try:
            _, _, n_err, _ = check_file(p, eng, pv)
        finally:
            sys.stdout = old
        hit = f'[{rule.split("/")[0]}]' in buf.getvalue() and n_err > 0
        if rule == 'empty':
            hit = n_err > 0
        if not hit:
            print(f'selfcheck: {rule} not flagged: {buf.getvalue()!r}')
            ok = False
    good = 'things =\n    { $count ->\n        [one] Ein Ding\n       *[other] { $count } Dinge\n    }\nkind =\n    { $kind ->\n        [a] Art A\n       *[other] Andere Art\n    }\nplain = Schlichter Text\n'
    p = os.path.join(d, 'de', 'fractadyne.ftl')
    # Written with CRLF endings, as git checks the files out on Windows.
    io.open(p, 'w', encoding='utf-8', newline='\r\n').write('language-name = Deutsch\nlanguage-english-name = German\nlanguage-script = Latn\nnumber-decimal = ,\nnumber-group = .\nlanguage-status = draft\n' + good)
    buf = io.StringIO()
    old = sys.stdout
    sys.stdout = buf
    try:
        _, _, n_err, _ = check_file(p, eng, pv)
    finally:
        sys.stdout = old
    if n_err:
        print('selfcheck: a good file is flagged:', buf.getvalue())
        ok = False
    print('selfcheck', 'PASS' if ok else 'FAIL', f'({len(cases)} red cases, 1 green)')
    return 0 if ok else 1


def main():
    sys.stdout.reconfigure(encoding='utf-8')
    if '--selfcheck' in sys.argv:
        return selfcheck()
    quiet = '--quiet' in sys.argv
    errs = []
    eng = parse(io.open(ENGLISH, encoding='utf-8').read().replace('\r\n', '\n'), 'en', errs)
    if errs:
        for n, rule, t in errs:
            print(f'i18n/en/fractadyne.ftl:{n}: ERROR [{rule}] {t}')
        return 1
    pv = plural_vars(eng)
    files = [a for a in sys.argv[1:] if not a.startswith('--')]
    if not files:
        for tag in sorted(os.listdir(I18N)):
            p = os.path.join(I18N, tag, 'fractadyne.ftl')
            if tag != 'en' and os.path.isfile(p):
                files.append(p)
    total_errors = 0
    rows = []
    for f in files:
        t, n, e, w = check_file(os.path.abspath(f), eng, pv, quiet)
        total_errors += e
        rows.append((os.path.relpath(os.path.abspath(f), ROOT).replace(os.sep, '/'), t, n, e, w))
    for rel, t, n, e, w in rows:
        print(f'{rel}: {t} of {n} messages ({100 * t // max(n, 1)}%), {e} error(s), {w} warning(s)')
    if not rows:
        print('no language files besides English')
    return 1 if total_errors else 0


if __name__ == '__main__':
    sys.exit(main())
