#!/usr/bin/env python3
"""check_ui_text.py [--report | --selfcheck] - fail when interface text is written in the code
instead of the English Fluent file (design/multilingual.md, section 7).

    python scripts/check_ui_text.py              # exit 1 on any finding
    python scripts/check_ui_text.py --report     # list the findings, exit 0
    python scripts/check_ui_text.py --selfcheck  # prove each rule goes red on a planted case

Every string the interface shows comes from `i18n/en/fractadyne.ftl` through `tr!`. This finds the
string literals (and `format!` literals) handed straight to the calls that put text on screen:
egui's labels, buttons, headings, tooltips, check boxes, radio buttons, selectable labels, window
and section titles, combo-box labels, hint texts, slider texts, file-dialog titles and filter
names, and the app's toasts (`set_toast`).

A literal passes when its words are only names and units that every language writes the same
(ALLOWED_WORDS: Fractadyne, GitHub, PNG, ms, px, Ctrl...), or when the line holding it, or the line
above, carries the marker `// i18n: keep` with a reason: text that is data (a coordinate, a token,
a file name), a name, or English on purpose (the command line, the issue report sent to GitHub).

It reads source text, not a parse tree, so it sees only literals written AT the call: a string held
in a variable or returned by a function passes unseen. `src/harness/` (test drivers, English by
design) and the generated `icons.rs` are skipped.

Exit 0 clean, 1 with a list. Standard library only.
"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, 'crates', 'fractadyne-app', 'src')
SKIP_DIRS = {os.path.join(SRC, 'harness')}
SKIP_FILES = {os.path.join(SRC, 'icons.rs')}
MARKER = 'i18n: keep'

# Calls whose FIRST argument is the text.
FIRST = ['label', 'button', 'small_button', 'heading', 'on_hover_text', 'on_disabled_hover_text',
         'hyperlink_to', 'menu_button', 'hint_text', 'set_title', 'add_filter', 'text', 'set_toast',
         'weak', 'strong', 'monospace', 'code', 'small']
# Calls whose LAST argument is the text.
LAST = ['checkbox', 'radio', 'radio_value', 'selectable_label', 'selectable_value', 'toggle_value']
# Constructors whose first (or, for the selectable/checkbox ones, last) argument is the text.
CTOR_FIRST = ['RichText::new', 'Label::new', 'Button::new', 'Window::new', 'CollapsingHeader::new',
              'ComboBox::from_label']
CTOR_LAST = ['SelectableLabel::new', 'Checkbox::new', 'RadioButton::new']

# Words every language writes as they are: names, units, file formats, key names, notation.
ALLOWED_WORDS = {
    # The product and other programs, services and libraries.
    'Fractadyne', 'GitHub', 'MPFR', 'FFmpeg', 'Fraktaler', 'Kalles', 'Imagina', 'FractalShark',
    'Fractint', 'XaoS', 'LaTeX', 'OpenEXR', 'Vulkan', 'DirectX', 'DX12', 'Metal', 'OpenGL', 'GL', 'wgpu', 'egui',
    'NVIDIA', 'AMD', 'Intel', 'Radeon', 'GeForce', 'RTX',
    # Hardware and technique acronyms.
    'GPU', 'GPUs', 'CPU', 'CPUs', 'SA', 'BLA', 'SSAA', 'AA', 'LA', 'HDR', 'SDR', 'RGB', 'RGBA', 'HSV',
    'OKLab', 'sRGB', 'ID', 'URL', 'UI', 'API', 'TDR',
    # Units.
    'ms', 'px', 'fps', 'KB', 'kB', 'MB', 'GB', 'TB', 'KiB', 'MiB', 'GiB', 'Hz', 'kHz', 'ns', 'us',
    'dpi', 'dp', 'bit', 'bits', 'bpp',
    # File formats and extensions.
    'PNG', 'png', 'EXR', 'exr', 'JPEG', 'JPG', 'jpg', 'TIFF', 'tif', 'tiff', 'WebP', 'webp', 'MP4',
    'mp4', 'RLE', 'rle', 'cells', 'fdn', 'kfr', 'kfp', 'toml', 'json', 'ftl', 'frm', 'ugr', 'map',
    'gpl', 'ggr', 'ase', 'csv', 'txt', 'zip', 'exe', 'svg', 'SVG', 'ico',
    # Notation for a colour as written in hex.
    'rrggbb',
    # Key names.
    'Ctrl', 'Shift', 'Alt', 'Cmd', 'Space', 'Esc', 'Enter', 'Tab', 'Home', 'End', 'PgUp', 'PgDn',
    'Del', 'Backspace', 'Insert',
    # Notation: complex numbers, functions of the formula language, readout symbols.
    're', 'im', 'Re', 'Im', 'sin', 'cos', 'tan', 'exp', 'log', 'abs', 'conj', 'sqrt', 'df32', 'f32',
    'f64', 'e308',
}

CALL_RE = re.compile(
    r'(?:\.(?P<m>' + '|'.join(FIRST + LAST) + r')|(?<![A-Za-z0-9_])(?P<c>'
    + '|'.join(re.escape(c) for c in CTOR_FIRST + CTOR_LAST) + r'))\s*\(')


def mask(src):
    """The source with comments blanked and string contents kept; returns (masked, literals),
    literals mapping each string's start offset to (end, text)."""
    out = list(src)
    lits = {}
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if src.startswith('//', i):
            j = src.find('\n', i)
            j = n if j < 0 else j
            for k in range(i, j):
                out[k] = ' '
            i = j
        elif src.startswith('/*', i):
            depth, j = 1, i + 2
            while j < n and depth:
                if src.startswith('/*', j):
                    depth, j = depth + 1, j + 2
                elif src.startswith('*/', j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            for k in range(i, j):
                if out[k] != '\n':
                    out[k] = ' '
            i = j
        elif c == 'r' and re.match(r'r#*"', src[i:i + 8]) and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == '_')):
            m = re.match(r'r(#*)"', src[i:])
            close = '"' + m.group(1)
            start = i + m.end()
            j = src.find(close, start)
            j = n if j < 0 else j
            lits[i] = (j + len(close), src[start:j])
            i = j + len(close)
        elif c == 'b' and src.startswith('b"', i) and (i == 0 or not (src[i - 1].isalnum() or src[i - 1] == '_')):
            j = i + 2
            while j < n and src[j] != '"':
                j += 2 if src[j] == '\\' else 1
            i = j + 1
        elif c == '"':
            j = i + 1
            buf = []
            while j < n and src[j] != '"':
                if src[j] == '\\':
                    nxt = src[j + 1] if j + 1 < n else ''
                    if nxt == '\n':
                        j += 2
                        while j < n and src[j] in ' \t\r\n':
                            j += 1
                        continue
                    buf.append({'n': '\n', 't': '\t', '"': '"', '\\': '\\'}.get(nxt, ''))
                    j += 2
                    continue
                buf.append(src[j])
                j += 1
            lits[i] = (j + 1, ''.join(buf))
            i = j + 1
        elif c == "'":
            # A char literal ('x', '\n', '\u{..}') or a lifetime ('a): skip a char literal whole.
            m = re.match(r"'(?:\\u\{[0-9a-fA-F]+\}|\\.|[^\\'])'", src[i:i + 12])
            i += m.end() if m else 1
        else:
            i += 1
    return ''.join(out), lits


def args_of(masked, lits, open_paren):
    """The top-level argument spans of the call whose '(' is at `open_paren`."""
    spans, depth, start, i, n = [], 0, open_paren + 1, open_paren + 1, len(masked)
    while i < n:
        if i in lits:
            i = lits[i][0]
            continue
        c = masked[i]
        if c in '([{':
            depth += 1
        elif c in ')]}':
            if depth == 0:
                spans.append((start, i))
                return spans
            depth -= 1
        elif c == ',' and depth == 0:
            spans.append((start, i))
            start = i + 1
        i += 1
    return spans


def literal_at(masked, lits, a, b):
    """The text of the literal an argument starts with: a plain or raw string, `&"..."`, or a
    `format!("...")`'s template. None when it starts with anything else."""
    i = a
    while i < b and masked[i] in ' \t\r\n&':
        i += 1
    if i in lits:
        return i, lits[i][1]
    m = re.match(r'format!\s*\(\s*', masked[i:b])
    if m and i + m.end() in lits:
        return i + m.end(), lits[i + m.end()][1]
    return None


def words(text):
    text = re.sub(r'\{[^{}]*\}', ' ', text)  # format placeholders
    return [w for w in re.findall(r"[A-Za-z][A-Za-z']+", text) if w not in ALLOWED_WORDS]


def findings_in(path, src):
    masked, lits = mask(src)
    lines = src.split('\n')
    out = []
    for m in CALL_RE.finditer(masked):
        name = m.group('m') or m.group('c')
        spans = args_of(masked, lits, m.end() - 1)
        if not spans or not masked[spans[0][0]:spans[0][1]].strip():
            continue
        span = spans[-1] if name in LAST or name in CTOR_LAST else spans[0]
        got = literal_at(masked, lits, *span)
        if not got:
            continue
        at, text = got
        bad = words(text)
        if not bad:
            continue
        line = src.count('\n', 0, at) + 1
        call_line = src.count('\n', 0, m.start()) + 1
        near = {line, line - 1, call_line, call_line - 1}
        if any(0 < k <= len(lines) and MARKER in lines[k - 1] for k in near):
            continue
        out.append((path, line, name, text))
    return out


def scan():
    found = []
    for d, subdirs, files in os.walk(SRC):
        if any(d == s or d.startswith(s + os.sep) for s in SKIP_DIRS):
            continue
        for f in sorted(files):
            p = os.path.join(d, f)
            if not f.endswith('.rs') or p in SKIP_FILES:
                continue
            with open(p, encoding='utf-8') as fh:
                found += findings_in(p, fh.read())
    return found


def selfcheck():
    red = {
        'label': 'ui.label("Show the grid");',
        'checkbox': 'ui.checkbox(&mut self.grid, "Show the grid");',
        'format': 'ui.label(format!("Saved {n} files", n = 3));',
        'hover': 'r.on_hover_text(\n    "Long tip text, \\\n     continued");',
        'window': 'egui::Window::new("Export image").show(ctx, |ui| {});',
        'richtext': 'ui.label(egui::RichText::new("Busy").weak());',
        'toast': 'self.set_toast("Copied to clipboard", ctx);',
        'raw': 'ui.label(r#"Raw "quoted" text"#);',
        'radio': 'ui.radio_value(&mut v, Mode::A, "Automatic");',
        'filter': 'rfd::FileDialog::new().add_filter("Images", &["png"]);',
        'ref': 'ui.button(&"Open folder");',
    }
    green = {
        'tr': 'ui.label(tr!(menu_file));',
        'symbols': 'ui.label("× — 2^93");',
        'units': 'ui.label(format!("{ms:.1} ms  {px} px"));',
        'keys': 'ui.label("Ctrl+Shift+S");',
        'names': 'ui.label("Fractadyne · GitHub · MPFR");',
        'comment': '// ui.label("Show the grid");',
        'block': '/* ui.label("Show the grid"); */ ui.label(x);',
        'marker': 'ui.label("v1.2 report"); // i18n: keep (data)',
        'marker-above': '// i18n: keep (a token)\nui.label("stable");',
        'variable': 'ui.label(text);',
        'char': "let c = '\"'; ui.label(c.to_string());",
        'checkbox-tr': 'ui.checkbox(&mut self.grid, tr!(grid));',
        'not-ui': 'let s = format!("Saved {n} files");',
    }
    ok = True
    for k, s in red.items():
        if not findings_in('<planted>', s):
            print(f'selfcheck: {k} should be flagged and is not: {s!r}')
            ok = False
    for k, s in green.items():
        f = findings_in('<planted>', s)
        if f:
            print(f'selfcheck: {k} should pass and is flagged: {f}')
            ok = False
    print('selfcheck', 'PASS' if ok else 'FAIL', f'({len(red)} red, {len(green)} green cases)')
    return 0 if ok else 1


def main():
    if '--selfcheck' in sys.argv:
        return selfcheck()
    found = scan()
    for path, line, name, text in found:
        rel = os.path.relpath(path, ROOT).replace(os.sep, '/')
        t = text.replace('\n', '\\n')
        print(f'{rel}:{line}: {name}: "{t[:90]}{"..." if len(t) > 90 else ""}"')
    print(f'{len(found)} interface literal(s) outside the English file')
    if '--report' in sys.argv:
        return 0
    return 1 if found else 0


if __name__ == '__main__':
    sys.stdout.reconfigure(encoding='utf-8')
    sys.exit(main())
