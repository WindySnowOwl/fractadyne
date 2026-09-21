#!/usr/bin/env python3
"""Strip your user-profile path out of files that are about to leave the machine.

    tools/redact-home.py FILE [FILE ...]          rewrite in place
    tools/redact-home.py --check FILE [FILE ...]  report only, change nothing

WHY
---
A Windows profile directory is an ACCOUNT NAME, and an account name is often a real name. A
benchmark run records absolute paths for every command it executes, so one results folder can
repeat it hundreds of times: the run manifest for a ten-scene, four-lane matrix carried it 188
times and the HTML report 313. That is fine on your own disk and not fine on a public URL,
particularly if you publish under a handle rather than your name.

`make-report.py --redact-home` handles the HTML it generates. This handles everything else you
might publish beside it - the manifest, a CSV, a log.

Both the plain and the JSON-escaped spelling are replaced (`C:\\Users\\you` appears in JSON as
`C:\\\\Users\\\\you`), because missing one is how a redaction quietly fails.

Exit 0 when nothing identifying remains, 1 when something does, 2 on a usage error. The check is
against the LEAF name, not the full path, so a stray mention that did not sit inside a full path
is still caught and reported rather than assumed absent.
"""
import argparse
import os
import sys


def variants(home):
    """Every spelling of the home path a tool might have written."""
    fwd = home.replace("\\", "/")
    out = []
    for h in (home, fwd):
        out.append((h, "C:\\Users\\<user>" if "\\" in h else "C:/Users/<user>"))
    # JSON escapes each separator, so the on-disk bytes are doubled.
    out.append((home.replace("\\", "\\\\"), "C:\\\\Users\\\\<user>"))
    return out


def prefix_variants(prefix):
    """Every spelling of a directory prefix to cut away, longest form first.

    Stripping the checkout's own root turns an absolute path into the relative one a reader
    actually needs - `bench-kit\\apps\\...` rather than someone's whole directory tree. The
    trailing separator goes with it so the result does not start with a stray slash.
    """
    p = prefix.rstrip("\\/")
    out = []
    for sep in ("\\", "/"):
        base = p.replace("\\", sep).replace("/", sep)
        out.append((base + sep, ""))
    out.append((p.replace("\\", "\\\\") + "\\\\", ""))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("files", nargs="+")
    ap.add_argument("--check", action="store_true", help="report only; do not modify")
    ap.add_argument("--home", default=None, help="override the profile path to strip")
    ap.add_argument("--strip-prefix", action="append", default=[], metavar="DIR",
                    help="cut this directory prefix off any path, leaving the rest relative. "
                         "Repeatable; longest match wins. Use it for the checkout root so "
                         "published paths read 'bench-kit\\apps\\...' instead of exposing the "
                         "whole directory tree above it.")
    args = ap.parse_args()

    home = args.home or os.path.expanduser("~")
    leaf = os.path.basename(home.rstrip("\\/"))
    if not leaf:
        sys.stderr.write("redact-home: could not determine the profile directory\n")
        return 2

    # Longest prefix first, so a nested root is cut before its parent and never leaves a
    # half-stripped path behind.
    subs = []
    for p in sorted(args.strip_prefix, key=len, reverse=True):
        subs.extend(prefix_variants(p))
    subs.extend(variants(home))
    dirty = 0
    for path in args.files:
        if not os.path.isfile(path):
            print("  %-28s SKIP (not a file)" % os.path.basename(path))
            continue
        try:
            with open(path, encoding="utf-8", errors="surrogateescape") as fh:
                text = fh.read()
        except Exception as e:
            print("  %-28s SKIP (%s)" % (os.path.basename(path), e))
            continue

        before = text.lower().count(leaf.lower())
        if not args.check:
            for old, new in subs:
                text = text.replace(old, new)
                if old.lower() != old:
                    text = text.replace(old.lower(), new)
            with open(path, "w", encoding="utf-8", errors="surrogateescape", newline="") as fh:
                fh.write(text)
        after = text.lower().count(leaf.lower())
        if after:
            dirty += 1
        print("  %-28s %4d -> %4d  %s"
              % (os.path.basename(path), before, after,
                 "STILL PRESENT" if after else "clean"))

    if dirty:
        print("\n%d file(s) still mention %r. Do not publish them as they are." % (dirty, leaf))
        return 1
    print("\nAll files clean of %r." % leaf)
    return 0


if __name__ == "__main__":
    sys.exit(main())
