#!/usr/bin/env python3
"""check_test_layout.py [--report | --selfcheck] - fail when test code sits among the functional
source files (design/test-code-separation.md, section 4).

    python scripts/check_test_layout.py              # exit 1 on any violation
    python scripts/check_test_layout.py --report     # list the violations, exit 0 (while migrating)
    python scripts/check_test_layout.py --selfcheck  # prove every rule goes red on a planted case

The layout: `crates/<crate>/src/` holds only code the product compiles, and every unit test lives
under `crates/<crate>/tests/unit/`, at the path of the file it tests, declared from that file as

    #[cfg(test)]
    #[path = "<up to the crate>/tests/unit/<the file's path without .rs>/<module>.rs"]
    mod <module>;

The module stays a child of the code it tests (private access, unchanged test names) while its
source sits outside `src/`. A crate root's (`main.rs`, `lib.rs`) test modules sit at the top of
`tests/unit/`.

The rules (the 2026-09-02 convention allowed exceptions and had no check, and eroded):

- `inline-test`: a `#[test]`, or an inline `mod x { ... }` under `#[cfg(test)]`, in `src/`.
- `test-only-item`: any other `#[cfg(test)]` item, or a `cfg_attr(not(test), ...)`, in `src/`,
  unless ALLOWED_ITEMS names it with its reason.
- `no-path`: a test-module declaration without `#[path]` (its file sits in `src/`).
- `not-canonical`: a `#[path]` other than the canonical location above.
- `missing`: the declared file does not exist.
- `orphan`: a `.rs` under `tests/unit/` that no declaration reaches. rustc never reads a file
  nothing declares, so a dead test would sit there looking alive.
- `cargo-target`: `tests/unit/main.rs`, which Cargo would build as its own integration test.

And one about direction, for the harnesses that ship in the binary (`src/harness/`): the product
calls into them only at known hooks.

- `harness-direction`: a functional file (under `src/`, not `src/harness/`) names `harness::`
  more often than HARNESS_HOOKS allows it.
- `stale-hook`: it names it LESS often: lower the allowance, so the hook list only shrinks.

Exit 0 clean, 1 with a list. Standard library only.
"""
import pathlib
import re
import shutil
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent

# (crate-relative path, item name) -> why this test-only item must stay in `src/`.
ALLOWED_ITEMS = {}

# Functional file (repo-relative) -> how many times it may name `harness::`: the product's hooks
# into the harnesses (src/harness.rs lists them). Exact, so the list shrinks as hooks go.
HARNESS_HOOKS = {
    # logcheck at start and exit, the profiler's timings field, the four harness-mode fields of
    # FractadyneApp (type, initialiser, doc), and the windowed harness flags' parsing in new_state
    # (phase 4 moves the parsing out).
    'crates/fractadyne-app/src/main.rs': 40,
    # the headless dispatch: --torture, --logcheck, --gputest, one call per developer mode, and
    # RenderCli's --glitch-audit kind.
    'crates/fractadyne-app/src/cli.rs': 10,
    # --replay's regime accumulator (phase 4 moves --replay out).
    'crates/fractadyne-app/src/render/decide.rs': 1,
    # the export-request builder records the profiler's setup timings.
    'crates/fractadyne-app/src/render.rs': 1,
}
HARNESS_REF = re.compile(r'\bharness::')

MOD_DECL = re.compile(r'(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*([;{])')
PATH_ATTR = re.compile(r'#\[\s*path\s*=\s*"([^"]+)"\s*\]$')
ITEM_NAME = re.compile(r'\b(?:fn|const|static|struct|enum|type|trait|union|macro_rules!)\s+(\w+)|\b(impl|use)\b')


def squash(attr):
    return re.sub(r'\s+', '', attr)


def is_test_cfg(attr):
    s = squash(attr)
    return s.startswith(('#[cfg(test)', '#[cfg(all(test', '#[cfg(any(test'))


def is_test_marker(attr):
    s = squash(attr)
    return s.startswith(('#[cfg_attr(test', '#[cfg_attr(not(test)'))


def attributes_and_item(lines, i):
    """The attributes starting at line `i` and the code that follows them: (attrs, item, line)."""
    attrs, buf, j = [], '', i
    while j < len(lines):
        s = lines[j].strip()
        if not buf:
            if not s or s.startswith('//'):
                j += 1
                continue
            if not s.startswith('#['):
                return attrs, s, j
        buf = f'{buf} {s}' if buf else s
        while buf.startswith('#['):
            depth, end = 0, None
            for k, ch in enumerate(buf):
                if ch == '[':
                    depth += 1
                elif ch == ']':
                    depth -= 1
                    if depth == 0:
                        end = k
                        break
            if end is None:
                break  # the attribute continues on the next line
            attrs.append(buf[:end + 1])
            buf = buf[end + 1:].strip()
            if buf.startswith('//'):
                buf = ''  # a trailing comment on the attribute's line
        if buf and not buf.startswith('#['):
            return attrs, buf, j  # the item starts on the attribute's own line
        j += 1
    return attrs, '', j


def canonical(crate, f, module):
    """The `#[path]` a test module `module` declared in `f` must carry."""
    up = '../' * (len(f.relative_to(crate).parts) - 1)
    inner = f.relative_to(crate / 'src')
    if len(inner.parts) == 1 and inner.name in ('main.rs', 'lib.rs'):
        mirror = ''
    elif inner.name == 'mod.rs':
        mirror = inner.parent.as_posix() + '/'
    else:
        mirror = inner.with_suffix('').as_posix() + '/'
    return f'{up}tests/unit/{mirror}{module}.rs'


def default_location(f, module):
    base = f.parent if f.name in ('main.rs', 'lib.rs', 'mod.rs') else f.parent / f.stem
    return base / f'{module}.rs' if (base / f'{module}.rs').exists() else base / module / 'mod.rs'


def nested_declarations(f):
    """Files a test file declares (`mod x;`, with or without `#[path]`), resolved as rustc does
    for a path-loaded file: beside it."""
    lines = f.read_text(encoding='utf-8').splitlines()
    out, i = [], 0
    while i < len(lines):
        s = lines[i].strip()
        attrs, item, j = attributes_and_item(lines, i) if s.startswith('#[') else ([], s, i)
        m = MOD_DECL.match(item)
        if m and m.group(2) == ';':
            p = next((PATH_ATTR.match(a).group(1) for a in attrs if PATH_ATTR.match(a)), None)
            out.append((f.parent / p) if p else (f.parent / f'{m.group(1)}.rs'))
        i = max(j, i) + 1
    return out


def check(root, hooks=None):
    """Every violation under `root` (a workspace) as (rule, where, what). `hooks`: the allowed
    `harness::` references per functional file (default HARNESS_HOOKS)."""
    hooks = HARNESS_HOOKS if hooks is None else hooks
    found = []
    for crate in sorted(p for p in (root / 'crates').iterdir() if (p / 'src').is_dir()):
        reached = set()
        for f in sorted((crate / 'src').rglob('*.rs')):
            rel = f.relative_to(root).as_posix()
            text = f.read_text(encoding='utf-8')
            lines = text.splitlines()
            if f != crate / 'src' / 'harness.rs' and crate / 'src' / 'harness' not in f.parents:
                refs, allowed = len(HARNESS_REF.findall(text)), hooks.get(rel, 0)
                if refs > allowed:
                    found.append(('harness-direction', rel, f'names `harness::` {refs}x; HARNESS_HOOKS allows {allowed}'))
                elif refs < allowed:
                    found.append(('stale-hook', rel, f'names `harness::` {refs}x; lower its HARNESS_HOOKS allowance from {allowed}'))
            i = 0
            while i < len(lines):
                if not lines[i].strip().startswith('#['):
                    i += 1
                    continue
                attrs, item, j = attributes_and_item(lines, i)
                where = f'{rel}:{i + 1}'
                if any(squash(a) == '#[test]' for a in attrs):
                    found.append(('inline-test', where, 'a #[test] in a functional file'))
                if any(is_test_cfg(a) or is_test_marker(a) for a in attrs):
                    m = MOD_DECL.match(item)
                    if m and m.group(2) == '{' and any(is_test_cfg(a) for a in attrs):
                        found.append(('inline-test', where, f'inline test module `{m.group(1)}`'))
                    elif m and m.group(2) == ';' and any(is_test_cfg(a) for a in attrs):
                        module = m.group(1)
                        want = canonical(crate, f, module)
                        p = next((PATH_ATTR.match(a).group(1) for a in attrs if PATH_ATTR.match(a)), None)
                        target = (f.parent / p) if p else default_location(f, module)
                        if p is None:
                            found.append(('no-path', where, f'`mod {module};` sits in src/; want #[path = "{want}"]'))
                        elif p != want:
                            found.append(('not-canonical', where, f'#[path = "{p}"]; want "{want}"'))
                        if not target.exists():
                            found.append(('missing', where, f'`mod {module};` -> {target.relative_to(root).as_posix()} does not exist'))
                        reached.add(target.resolve())
                    else:
                        n = ITEM_NAME.search(item)
                        name = (n.group(1) or n.group(2)) if n else item[:40]
                        key = (f.relative_to(crate).as_posix(), name)
                        if key not in ALLOWED_ITEMS:
                            found.append(('test-only-item', where, f'test-only `{name}` in a functional file'))
                i = max(j, i) + 1
        unit = crate / 'tests' / 'unit'
        todo = [t for t in reached if unit in t.parents]
        while todo:
            t = todo.pop()
            if t.exists():
                for n in nested_declarations(t):
                    n = n.resolve()
                    if n not in reached:
                        reached.add(n)
                        todo.append(n)
        if unit.is_dir():
            for f in sorted(unit.rglob('*.rs')):
                if f.resolve() not in reached:
                    found.append(('orphan', f.relative_to(root).as_posix(), 'no declaration reaches it'))
        if (unit / 'main.rs').exists():
            found.append(('cargo-target', (unit / 'main.rs').relative_to(root).as_posix(),
                          'Cargo builds tests/*/main.rs as an integration test'))
    for rel in sorted(hooks):
        if not (root / rel).exists():
            found.append(('stale-hook', rel, 'HARNESS_HOOKS names a file that no longer exists'))
    return found


def selfcheck():
    """Plant one violation of each rule in a scratch workspace; each must be reported, and the
    clean workspace must report nothing."""
    clean = {
        'crates/demo/src/lib.rs': 'pub mod a;\n\n#[cfg(test)]\n#[path = "../tests/unit/root_tests.rs"]\nmod root_tests;\n',
        'crates/demo/src/a.rs': 'pub fn f() {}\n\n#[cfg(test)]\n#[path = "../tests/unit/a/tests.rs"]\nmod tests;\n',
        'crates/demo/tests/unit/root_tests.rs': '#[test]\nfn t() {}\n',
        'crates/demo/tests/unit/a/tests.rs': 'use super::*;\nmod helper;\n#[test]\nfn t() { f() }\n',
        'crates/demo/tests/unit/a/helper.rs': 'pub fn h() {}\n',
    }
    a = clean['crates/demo/src/a.rs']
    call = 'pub fn g() { crate::harness::x::run() }\n'
    hooked = {'crates/demo/src/a.rs': 1}
    cases = {
        'clean': ({}, None, {}),
        'inline-test': ({'crates/demo/src/a.rs': a + '#[cfg(test)]\nmod more {\n    #[test]\n    fn t() {}\n}\n'}, 'inline-test', {}),
        'test-only-item': ({'crates/demo/src/a.rs': a + '#[cfg(test)]\nfn helper() {}\n'}, 'test-only-item', {}),
        'test-only-marker': ({'crates/demo/src/a.rs': a + '#[cfg_attr(not(test), allow(dead_code))]\nfn only_tests_call() {}\n'}, 'test-only-item', {}),
        'no-path': ({'crates/demo/src/a.rs': a + '#[cfg(test)]\nmod beside;\n', 'crates/demo/src/a/beside.rs': '#[test]\nfn t() {}\n'}, 'no-path', {}),
        'not-canonical': ({'crates/demo/src/a.rs': a.replace('a/tests.rs', 'a_tests.rs'), 'crates/demo/tests/unit/a_tests.rs': '#[test]\nfn t() {}\n'}, 'not-canonical', {}),
        'missing': ({'crates/demo/src/a.rs': a + '#[cfg(test)]\n#[path = "../tests/unit/a/gone.rs"]\nmod gone;\n'}, 'missing', {}),
        'orphan': ({'crates/demo/tests/unit/a/stray.rs': '#[test]\nfn t() {}\n'}, 'orphan', {}),
        'cargo-target': ({'crates/demo/tests/unit/main.rs': 'fn main() {}\n'}, 'cargo-target', {}),
        'harness-direction': ({'crates/demo/src/a.rs': a + call}, 'harness-direction', {}),
        'an allowed hook': ({'crates/demo/src/a.rs': a + call}, None, hooked),
        'stale-hook': ({}, 'stale-hook', hooked),
        'a harness naming harness::': ({'crates/demo/src/harness/x.rs': call}, None, {}),
    }
    failures = 0
    for name, (extra, rule, hooks) in cases.items():
        tmp = pathlib.Path(tempfile.mkdtemp(prefix='layout-'))
        try:
            for path, text in {**clean, **extra}.items():
                (tmp / path).parent.mkdir(parents=True, exist_ok=True)
                (tmp / path).write_text(text, encoding='utf-8', newline='\n')
            rules = sorted({r for r, _, _ in check(tmp, hooks)})
            ok = rules == [] if rule is None else rule in rules
            print(f'  {"ok  " if ok else "FAIL"} {name}: reported {rules or "nothing"}')
            failures += not ok
        finally:
            shutil.rmtree(tmp)
    print(f'check_test_layout --selfcheck: {"every rule goes red" if not failures else f"{failures} case(s) wrong"}')
    return 1 if failures else 0


def main(argv):
    if '--selfcheck' in argv:
        return selfcheck()
    found = check(ROOT)
    if not found:
        print('check_test_layout: clean (no test code under any src/)')
        return 0
    counts = {}
    for rule, _, _ in found:
        counts[rule] = counts.get(rule, 0) + 1
    print(f'check_test_layout: {len(found)} violation(s): ' + ', '.join(f'{r} {n}' for r, n in sorted(counts.items())))
    for rule, where, what in found:
        print(f'  {rule:15} {where}: {what}')
    return 0 if '--report' in argv else 1


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
