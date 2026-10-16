#!/usr/bin/env python3
"""check_move_only.py [RANGE] [--each] [--rewrite OLD NEW ...] [--selfcheck] - prove a commit only
MOVES code: every line it removes comes back, in order, where it adds, and nothing else changes
(design/test-code-separation.md, section 5).

    python scripts/check_move_only.py                      # the last commit (HEAD~1..HEAD)
    python scripts/check_move_only.py main..HEAD --each    # every commit in the range, one by one
    python scripts/check_move_only.py --rewrite crate::selftest:: crate::harness::selftest::
    python scripts/check_move_only.py --selfcheck          # prove it goes red on an edit hidden in a move

How: the diff is read with rename detection OFF, so a moved file is a whole-file removal plus a
whole-file addition, and an extracted block is a removal from one file plus an addition to another.
Lines are compared with surrounding whitespace stripped (a moved block may be re-indented), after
dropping the lines a move legitimately adds or removes: blank lines, `#[cfg(test)]`,
`#[path = ...]`, `mod x;`, `mod x {` and a lone `}`. Then:

- the removed and the added lines must be the same multiset: nothing new, nothing lost;
- each run of consecutive removed lines must appear, in order, inside one run of added lines, so a
  move cannot reorder code inside a block or splice a line into it.

`--rewrite OLD NEW` (two arguments: either may contain `=`) substitutes on the removed side first,
in the order given, for a mechanical change that comes with the move (a module path);
`--rewrite-re REGEX REPL` does the same with a regular expression; `--in PATH` after either limits
it to lines removed from that file (repo-relative, `/`). Describe what the move's script did, no
more: a rewrite broader than the change it stands for can turn a line nobody touched into a
mismatch, or hide an edit. (On Git Bash, a leading `/` in an argument is rewritten into a Windows
path: set MSYS_NO_PATHCONV=1.) Binary files must move byte-identical. A move commit carries nothing
else: a content change, however small, goes in a commit of its own.

Exit 0 a pure move, 1 not (with the lines that break it). Standard library only.
"""
import collections
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile

STRUCTURAL = re.compile(r'#\[cfg\(test\)\]|#\[path\s*=\s*"[^"]*"\]|(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*[;{]|\}')


def git(*args, cwd=None):
    return subprocess.run(['git', '-c', 'core.quotepath=false', *args], capture_output=True, check=True, cwd=cwd).stdout


def diff_runs(base, head, cwd=None):
    """Maximal runs of removed ('-') and added ('+') lines per file: [(path, sign, [lines])]."""
    text = git('diff', '--no-renames', '--no-color', '--no-ext-diff', '-U0', base, head, cwd=cwd).decode('utf-8', 'replace')
    runs, path, header, cur = [], None, False, None
    for raw in text.split('\n'):
        if raw.startswith('diff --git '):
            header, cur, path = True, None, None
            continue
        if header:
            if raw.startswith('--- '):
                path = raw[6:] if raw.startswith('--- a/') else path
            elif raw.startswith('+++ '):
                path = raw[6:] if raw.startswith('+++ b/') else path
            elif raw.startswith('@@'):
                header = False
            continue
        if raw.startswith('@@'):
            cur = None
        elif raw[:1] in ('+', '-'):
            if cur is None or cur[1] != raw[0]:
                cur = (path, raw[0], [])
                runs.append(cur)
            cur[2].append(raw[1:])
        elif not raw.startswith('\\'):
            cur = None
    return runs


def binary_moves(base, head, cwd=None):
    """Blob ids of binary files removed and added: two Counters that must be equal."""
    gone, came = collections.Counter(), collections.Counter()
    numstat = git('diff', '--no-renames', '--numstat', '-z', base, head, cwd=cwd).decode('utf-8', 'replace')
    binaries = {e.split('\t', 2)[2] for e in numstat.split('\0') if e.startswith('-\t-\t')}
    raw = git('diff', '--no-renames', '--raw', '--no-abbrev', '-z', base, head, cwd=cwd).decode('utf-8', 'replace')
    fields = raw.split('\0')
    for meta, path in zip(fields[0::2], fields[1::2]):
        if path in binaries:
            _, _, old, new, status = meta.lstrip(':').split()
            if status != 'A':
                gone[old] += 1
            if status != 'D':
                came[new] += 1
    return gone, came


def check(base, head, rewrites=(), cwd=None):
    """Problems that make base..head more than a move (empty: a pure move). A rewrite is
    (OLD, NEW) or (kind, OLD, NEW, scope): kind 'str' (substring) or 're' (regex), scope a path the
    removed line's file must equal, or None for every file."""
    rules = [('str', r[0], r[1], None) if len(r) == 2 else r for r in rewrites]

    def norm(line, sign, path):
        s = line.strip()
        if sign == '-':
            for kind, old, new, scope in rules:
                if scope is None or scope == path:
                    s = re.sub(old, new, s) if kind == 're' else s.replace(old, new)
        return s

    removed, added = [], []
    for path, sign, lines in diff_runs(base, head, cwd):
        kept = [norm(l, sign, path) for l in lines]
        kept = [s for s in kept if s and not STRUCTURAL.fullmatch(s)]
        if kept:
            (removed if sign == '-' else added).append((path, kept))
    problems = []
    lost = collections.Counter(l for _, ls in removed for l in ls)
    new = collections.Counter(l for _, ls in added for l in ls)
    for line, n in (lost - new).items():
        where = next(p for p, ls in removed if line in ls)
        problems.append(f'removed, not added back ({n}x, {where}): {line[:110]}')
    for line, n in (new - lost).items():
        where = next(p for p, ls in added if line in ls)
        problems.append(f'added, not moved from anywhere ({n}x, {where}): {line[:110]}')
    if not problems:
        targets = ['\n' + '\n'.join(ls) + '\n' for _, ls in added]
        for path, ls in removed:
            run = '\n' + '\n'.join(ls) + '\n'
            if not any(run in t for t in targets):
                problems.append(f'a block from {path} ({len(ls)} lines from "{ls[0][:60]}") does not reappear in one piece, in order')
    gone, came = binary_moves(base, head, cwd)
    if gone != came:
        problems.append(f'binary files changed, not moved: removed {sorted(gone - came)}, added {sorted(came - gone)}')
    return problems


def selfcheck():
    """Commit a module with an inline test block, then each way of moving it; the pure moves must
    pass and every hidden edit must fail."""
    a = ('pub fn f() -> u32 {\n    1\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n'
         '    #[test]\n    fn one() {\n        assert_eq!(f(), 1);\n    }\n\n'
         '    #[test]\n    fn two() {\n        let x = f();\n        assert!(x > 0);\n    }\n}\n')
    decl = 'pub fn f() -> u32 {\n    1\n}\n\n#[cfg(test)]\n#[path = "../tests/unit/a/tests.rs"]\nmod tests;\n'
    body = ('use super::*;\n\n#[test]\nfn one() {\n    assert_eq!(f(), 1);\n}\n\n'
            '#[test]\nfn two() {\n    let x = f();\n    assert!(x > 0);\n}\n')
    cases = [
        ('pure extraction', {'src/a.rs': decl, 'tests/unit/a/tests.rs': body}, (), True),
        ('edit hidden in the move', {'src/a.rs': decl, 'tests/unit/a/tests.rs': body.replace('f(), 1', 'f(), 2')}, (), False),
        ('lines reordered in the move', {'src/a.rs': decl, 'tests/unit/a/tests.rs': body.replace(
            '    let x = f();\n    assert!(x > 0);', '    assert!(x > 0);\n    let x = f();')}, (), False),
        ('a line added in the move', {'src/a.rs': decl, 'tests/unit/a/tests.rs': body + '\nfn extra() {}\n'}, (), False),
        ('a line dropped in the move', {'src/a.rs': decl, 'tests/unit/a/tests.rs': body.replace('    let x = f();\n', '')}, (), False),
        ('rewrite without --rewrite', {'src/a.rs': decl, 'tests/unit/a/tests.rs': body.replace('use super::*;', 'use crate::moved::*;')}, (), False),
        ('rewrite with --rewrite', {'src/a.rs': decl, 'tests/unit/a/tests.rs': body.replace('use super::*;', 'use crate::moved::*;')},
         (('use super::*;', 'use crate::moved::*;'),), True),
        ('a rewrite whose text holds "="', {'src/a.rs': decl, 'tests/unit/a/tests.rs': body.replace('let x = f();', 'let x = crate::moved::f();')},
         (('let x = f();', 'let x = crate::moved::f();'),), True),
        ('a regex rewrite', {'src/a.rs': decl, 'tests/unit/a/tests.rs': body.replace('let x = f();', 'let x = crate::moved::f();')},
         (('re', r'(?<![\w:])f\(\)', 'crate::moved::f()', 'src/a.rs'),), False),  # also rewrites `f(), 1`
        ('a rewrite scoped to the file it happened in', {'src/a.rs': decl, 'tests/unit/a/tests.rs': body.replace('let x = f();', 'let x = crate::moved::f();')},
         (('re', r'let x = f\(\);', 'let x = crate::moved::f();', 'src/a.rs'),), True),
        ('a rewrite scoped to another file', {'src/a.rs': decl, 'tests/unit/a/tests.rs': body.replace('let x = f();', 'let x = crate::moved::f();')},
         (('str', 'let x = f();', 'let x = crate::moved::f();', 'src/b.rs'),), False),
    ]
    failures = 0
    for name, files, rewrites, want_pure in cases:
        tmp = pathlib.Path(tempfile.mkdtemp(prefix='moveonly-'))
        try:
            ident = ['-c', 'user.name=selfcheck', '-c', 'user.email=selfcheck@example.invalid', '-c', 'commit.gpgsign=false']
            git('init', '-q', cwd=tmp)
            (tmp / 'src').mkdir()
            (tmp / 'src/a.rs').write_text(a, encoding='utf-8', newline='\n')
            git('add', '-A', cwd=tmp)
            git(*ident, 'commit', '-q', '-m', 'before', cwd=tmp)
            for path, text in files.items():
                (tmp / path).parent.mkdir(parents=True, exist_ok=True)
                (tmp / path).write_text(text, encoding='utf-8', newline='\n')
            git('add', '-A', cwd=tmp)
            git(*ident, 'commit', '-q', '-m', 'after', cwd=tmp)
            problems = check('HEAD~1', 'HEAD', rewrites, cwd=tmp)
            ok = (not problems) == want_pure
            print(f'  {"ok  " if ok else "FAIL"} {name}: {"pure move" if not problems else problems[0]}')
            failures += not ok
        finally:
            shutil.rmtree(tmp, ignore_errors=True)
    print(f'check_move_only --selfcheck: {"every hidden edit goes red" if not failures else f"{failures} case(s) wrong"}')
    return 1 if failures else 0


def main(argv):
    if '--selfcheck' in argv:
        return selfcheck()
    rewrites, rng, each, i = [], 'HEAD~1..HEAD', False, 0
    while i < len(argv):
        if argv[i] in ('--rewrite', '--rewrite-re'):
            if i + 2 >= len(argv) or not argv[i + 1]:
                print(f'check_move_only: {argv[i]} takes two arguments, OLD (not empty) and NEW')
                return 2
            rewrites.append(('re' if argv[i] == '--rewrite-re' else 'str', argv[i + 1], argv[i + 2], None))
            i += 3
            continue
        if argv[i] == '--in':
            if not rewrites or i + 1 >= len(argv):
                print('check_move_only: --in PATH follows a --rewrite or --rewrite-re')
                return 2
            kind, old, new, _ = rewrites[-1]
            rewrites[-1] = (kind, old, new, argv[i + 1])
            i += 2
            continue
        if argv[i] == '--each':
            each = True
        else:
            rng = argv[i]
        i += 1
    base, _, head = rng.partition('..')
    pairs = [(f'{c}~1', c) for c in git('rev-list', '--reverse', rng).decode().split()] if each else [(base, head or 'HEAD')]
    bad = 0
    for b, h in pairs:
        problems = check(b, h, rewrites)
        label = git('log', '-1', '--format=%h %s', h).decode().strip()[:90]
        if problems:
            bad += 1
            print(f'NOT A PURE MOVE  {label}')
            for p in problems[:40]:
                print(f'  {p}')
            if len(problems) > 40:
                print(f'  ... and {len(problems) - 40} more')
        else:
            print(f'pure move        {label}')
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
