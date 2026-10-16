#!/usr/bin/env python3
"""check_text_hygiene.py [--staged] - fail on the encoding, escaping and line-ending damage that tool
edits leave in text files.

    python scripts/check_text_hygiene.py           # every tracked text file, as checked out
    python scripts/check_text_hygiene.py --staged  # only what is staged, as it will be committed

Each check is the signature of a failure this repo has paid for:

- **A blob stored with CRLF or mixed line endings.** `.gitattributes` (`* text=auto`) stores every
  text file with LF; a CRLF or mixed blob means the attribute was overridden. Before it existed,
  staging a file with the wrong `core.autocrlf` flipped all of its lines (thousand-line phantom
  diffs, `git blame` destroyed).
- **Control bytes** (anything below 0x20 but tab, LF and CR, and 0x7F). A backslash sequence passed
  through a shell heredoc or `python -c` into a file lands as a form feed or vertical tab (`\\f`,
  `\\v`): seen in memory notes, twice.
- **A UTF-8 byte-order mark** outside `.ps1` files (rustc, TOML and most tools treat it as content).
- **Text that is not valid UTF-8** (a tool that wrote the console's code page instead).
- **A `.ps1` that is neither pure ASCII nor marked with a BOM**: Windows PowerShell 5.1 reads a
  BOM-less file as CP1252, and an em-dash's third byte becomes a quote that swallows the script.

Exit 0 clean, 1 with a list of what is wrong. Standard library only.
"""
import subprocess
import sys

BOM = b'\xef\xbb\xbf'
CONTROL = set(range(0x00, 0x20)) - {0x09, 0x0A, 0x0D} | {0x7F}


def git(*args):
    return subprocess.run(['git', *args], capture_output=True, check=True).stdout


def text_files(staged):
    """(path, index-EOL class) of every tracked file git treats as text."""
    if staged:
        names = set(git('diff', '--cached', '--name-only', '--diff-filter=ACMR', '-z').decode('utf-8').split('\0'))
    out = []
    for line in git('ls-files', '--eol', '-z').decode('utf-8').split('\0'):
        if not line:
            continue
        meta, path = line.split('\t', 1)
        cls = meta.split()[0]  # i/lf, i/crlf, i/mixed, i/-text, i/none
        if cls in ('i/-text', 'i/none'):
            continue  # binary, or empty
        if staged and path not in names:
            continue
        out.append((path, cls))
    return out


def content(path, staged):
    if staged:
        return git('cat-file', 'blob', f':{path}')
    with open(path, 'rb') as f:
        return f.read()


def check(staged):
    problems = []
    for path, cls in text_files(staged):
        if cls in ('i/crlf', 'i/mixed'):
            problems.append(f'{path}: stored with {cls[2:].upper()} line endings (every text blob must be LF)')
        try:
            data = content(path, staged)
        except (OSError, subprocess.CalledProcessError):
            continue  # deleted in the worktree
        is_ps1 = path.lower().endswith('.ps1')
        if data.startswith(BOM) and not is_ps1:
            problems.append(f'{path}: starts with a UTF-8 byte-order mark')
        body = data[len(BOM):] if data.startswith(BOM) else data
        try:
            body.decode('utf-8')
        except UnicodeDecodeError as e:
            problems.append(f'{path}: not valid UTF-8 at byte {e.start}')
            continue
        if is_ps1 and not data.startswith(BOM) and any(b > 0x7F for b in body):
            problems.append(f'{path}: non-ASCII without a BOM (Windows PowerShell 5.1 misreads it)')
        bad = [i for i, b in enumerate(body) if b in CONTROL]
        if bad:
            line = body.count(b'\n', 0, bad[0]) + 1
            problems.append(f'{path}: {len(bad)} control byte(s), first 0x{body[bad[0]]:02x} on line {line}')
    return problems


def main(argv):
    staged = '--staged' in argv
    problems = check(staged)
    scope = 'staged files' if staged else 'tracked text files'
    if problems:
        print(f'check_text_hygiene: {len(problems)} problem(s) in the {scope}:')
        for p in problems:
            print(f'  {p}')
        return 1
    print(f'check_text_hygiene: clean ({scope})')
    return 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
