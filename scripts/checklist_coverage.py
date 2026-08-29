#!/usr/bin/env python3
"""Resolve every manual-checklist step's declared enforcer against the actual tree.

    python scripts/checklist_coverage.py            # report + fail on a broken claim
    python scripts/checklist_coverage.py --cargo    # also resolve test names via `cargo test --list`

A row that CLAIMS an automated enforcer it no longer has is worse than a row claiming none: the
reviewer skims it, and nothing is checking. So every non-`planned:` enforcer must resolve to
something that exists, and this exits non-zero when one does not.

`planned:` entries are counted as OUTSTANDING, never as covered. They are a work list, not
coverage, and the summary keeps the two apart on purpose.

See design/checklist-automation.md for what each row is meant to enforce.
"""
import argparse
import importlib.util
import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def load_checklist():
    p = os.path.join(ROOT, "scripts", "release_checklist.py")
    spec = importlib.util.spec_from_file_location("release_checklist", p)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)  # its own asserts check STEPS/ENFORCERS alignment
    return m


def read(rel):
    p = os.path.join(ROOT, rel.replace("/", os.sep))
    if not os.path.exists(p):
        return None
    with open(p, encoding="utf-8", errors="replace") as f:
        return f.read()


def rust_sources():
    """Every .rs file in the workspace crates, concatenated once."""
    out = []
    for base, _dirs, files in os.walk(os.path.join(ROOT, "crates")):
        if "target" in base.split(os.sep):
            continue
        for fn in files:
            if fn.endswith(".rs"):
                with open(os.path.join(base, fn), encoding="utf-8", errors="replace") as f:
                    out.append(f.read())
    return "\n".join(out)


def cargo_test_names():
    """Authoritative test list. Needs the test binaries built; falls back to None."""
    try:
        r = subprocess.run(
            ["cargo", "test", "--workspace", "--release", "--", "--list"],
            cwd=ROOT, capture_output=True, text=True, timeout=1800,
        )
    except Exception:
        return None
    if r.returncode != 0:
        return None
    return {m.group(1) for m in re.finditer(r"^(\S+): test$", r.stdout, re.M)}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cargo", action="store_true",
                    help="resolve test names with `cargo test -- --list` instead of a source scan")
    args = ap.parse_args()

    m = load_checklist()
    rs = rust_sources()
    selftest = read("crates/fractadyne-app/src/selftest.rs") or ""
    uitest = read("crates/fractadyne-app/src/uitest.rs") or ""
    argparse_src = read("crates/fractadyne-app/src/main.rs") or ""
    listed = cargo_test_names() if args.cargo else None
    if args.cargo and listed is None:
        print("warning: `cargo test -- --list` did not run; falling back to a source scan")

    def resolve(e):
        """(ok, kind, detail). `ok` is None for rows that cannot have an enforcer."""
        if e in ("manual", "process"):
            return None, e, "no machine enforcer is possible"
        if e.startswith("planned:"):
            return None, "planned", e[len("planned:"):]
        if e.startswith("partial:"):
            ok, kind, detail = resolve(e[len("partial:"):])
            return ok, "partial/" + kind, detail
        kind, _, name = e.partition(":")
        if kind == "test":
            if listed is not None:
                hit = any(name in t for t in listed)
            else:
                hit = re.search(r"\bfn\s+%s\s*\(" % re.escape(name), rs) is not None
            return hit, "test", name
        # Anchor on the OPENING quote only: a check name is often the start of a format!
        # literal ("control changes the image \u2014 {name}"), so requiring the closing quote
        # would reject names that are genuinely there.
        if kind == "selftest":
            return ('"' + name) in selftest, "selftest", name
        if kind == "uitest":
            return ('"' + name) in uitest, "uitest", name
        if kind == "harness":
            return ('"%s"' % name) in argparse_src, "harness", name
        if kind == "script":
            return os.path.exists(os.path.join(ROOT, name.replace("/", os.sep))), "script", name
        return False, "unknown", e

    covered, partial, planned, none_possible, broken = [], [], [], [], []
    for i, ((_area, enf), step) in enumerate(zip(m.ENFORCERS, m.STEPS), start=1):
        ok, kind, detail = resolve(enf)
        row = (i, step[0], enf, detail)
        if ok is None:
            (planned if kind == "planned" else none_possible).append(row)
        elif not ok:
            broken.append((i, step[0], enf, "%s %r not found" % (kind, detail)))
        elif kind.startswith("partial"):
            partial.append(row)
        else:
            covered.append(row)

    total = len(m.STEPS)
    print("Manual checklist: %d steps" % total)
    print("  %3d fully enforced      (a machine check fails if the behaviour breaks)" % len(covered))
    print("  %3d partly enforced     (effect checked; gesture or judgement is not)" % len(partial))
    print("  %3d outstanding         (declared in the plan, not yet implemented)" % len(planned))
    print("  %3d cannot be automated (needs a person, a second machine, or a second display)"
          % len(none_possible))
    if broken:
        print("\n%d BROKEN CLAIM(S) - a row promises coverage that does not exist:" % len(broken))
        for i, area, enf, why in broken:
            print("  step %-3d %-18s %-52s %s" % (i, area, enf, why))
        return 1
    if planned:
        print("\nOutstanding, in step order:")
        for i, area, _enf, detail in planned:
            print("  step %-3d %-18s %s" % (i, area, detail))
    return 0


if __name__ == "__main__":
    sys.exit(main())
