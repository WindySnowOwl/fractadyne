#!/usr/bin/env python3
"""test_names.py dump OUT [cargo test args] | compare BEFORE AFTER [--map OLD=NEW ...] - the full
list of test names per test target, to prove a refactor kept every test
(design/test-code-separation.md, section 5).

    python scripts/test_names.py dump before.txt                    # cargo test --workspace --release -j 1 -- --list
    python scripts/test_names.py dump before-rug.txt -p fractadyne-core --features rug
    python scripts/test_names.py parse transcript.txt before.txt    # a `-- --list` run made elsewhere
    python scripts/test_names.py compare before.txt after.txt       # exit 1 on any difference
    python scripts/test_names.py compare before.txt after.txt --map selftest::=harness::selftest::

A count can stay the same while one test is lost and another duplicated; a list of names cannot.
`--list` builds the tests but runs none. Ignored tests are listed too. A doc-test's name carries
its line number, which a move may change, so the number is dropped.

Exit 0 identical (or dumped), 1 different, 2 cargo failed. Standard library only.
"""
import re
import subprocess
import sys

RUNNING = re.compile(r'^\s*Running (?:unittests )?(\S+) \(.*?[\\/]deps[\\/](\w+?)-[0-9a-f]+(?:\.exe)?\)')
DOCTESTS = re.compile(r'^\s*Doc-tests (\w+)')
NAME = re.compile(r'^(.*): (?:test|bench)$')


def dump(out, args):
    cmd = ['cargo', 'test', *(args or ['--workspace']), '--release', '-j', '1', '--', '--list']
    p = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    text = p.stdout.decode('utf-8', 'replace')
    if p.returncode != 0:
        print(text[-3000:])
        print(f'test_names: `{" ".join(cmd)}` failed ({p.returncode})')
        return 2
    return write(out, text)


def write(out, text):
    """The names in a `cargo test ... -- --list` transcript (stdout and stderr together)."""
    names, target = [], None
    for line in text.splitlines():
        m = RUNNING.match(line)
        if m:
            target = f'{m.group(2)}:{m.group(1).replace(chr(92), "/")}'
            continue
        m = DOCTESTS.match(line)
        if m:
            target = f'{m.group(1)}:doc'
            continue
        m = NAME.match(line)
        if m and target:
            names.append(f'{target}\t{re.sub(r" [(]line [0-9]+[)]$", "", m.group(1))}')
    names.sort()
    with open(out, 'w', encoding='utf-8', newline='\n') as f:
        f.write(''.join(n + '\n' for n in names))
    per = {}
    for n in names:
        t = n.split('\t', 1)[0]
        per[t] = per.get(t, 0) + 1
    for t, n in sorted(per.items()):
        print(f'  {n:5} {t}')
    print(f'test_names: {len(names)} tests in {len(per)} targets -> {out}')
    return 0


def load(path, maps=()):
    out = []
    with open(path, encoding='utf-8') as f:
        for line in f:
            target, name = line.rstrip('\n').split('\t', 1)
            for old, new in maps:
                if name.startswith(old):
                    name = new + name[len(old):]
            out.append(f'{target}\t{name}')
    return out


def compare(before, after, maps):
    a, b = load(before, maps), load(after)
    sa, sb = set(a), set(b)
    dup = sorted({n for n in b if b.count(n) > 1}) if len(sb) != len(b) else []
    lost, new = sorted(sa - sb), sorted(sb - sa)
    for n in lost:
        print(f'  lost: {n}')
    for n in new:
        print(f'  new:  {n}')
    for n in dup:
        print(f'  twice: {n}')
    if lost or new or dup or len(a) != len(b):
        print(f'test_names: DIFFERENT - {len(a)} before, {len(b)} after, {len(lost)} lost, {len(new)} new')
        return 1
    print(f'test_names: identical - {len(b)} tests')
    return 0


def main(argv):
    if len(argv) >= 2 and argv[0] == 'dump':
        return dump(argv[1], argv[2:])
    if len(argv) == 3 and argv[0] == 'parse':
        with open(argv[1], encoding='utf-8', errors='replace') as f:
            return write(argv[2], f.read())
    if len(argv) >= 3 and argv[0] == 'compare':
        maps, rest = [], argv[3:]
        while rest:
            if rest[0] == '--map' and len(rest) > 1:
                old, _, new = rest[1].partition('=')
                maps.append((old, new))
                rest = rest[2:]
            else:
                print(f'test_names: unknown argument {rest[0]}')
                return 2
        return compare(argv[1], argv[2], maps)
    print(__doc__)
    return 2


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
