#!/usr/bin/env python3
"""Run a release gate and record its exit code against the binary that ran it.

    python scripts/run_ledger.py run -- --livetest tours/grand-tour.toml --size 480x270
    python scripts/run_ledger.py run -- python scripts/dive-capture/screengate.py B/screen/run1 B/screen/run2 B/screen/run3
    python scripts/run_ledger.py show [VERSION]

A checklist row whose enforcer is `run:<command>` is satisfied only by a row here: that command, from
THIS release's binary, from a clean tree, exit 0 (`scripts/checklist_coverage.py --release VERSION`).
A harness that exists is not a gate that ran, and a run nobody recorded cannot be told apart from
one that never happened.

- Arguments starting with `--` are app arguments: the release binary runs them (`--exe`, default
  `target/release/fractadyne[.exe]`) with a FRESH config directory, so a gate never reads or writes
  someone's real session and never inherits one: a saved session's own Iterations base changes what
  `--livetest` measures. `FRACTADYNE_CONFIG_DIR` from the environment is deliberately ignored;
  `--config DIR` is the explicit way to start from a prepared one. Its identity is the binary's own
  `--version` (version, build, git sha, `-dirty`), and the row also carries what the app's log
  says: the `[fd-exit]` line and logcheck's verdict.
- Anything else is a command run as given (a Python or PowerShell script). Its identity is the
  workspace version and the repository's HEAD.

Rows go to `validation/runs/<version>.tsv`, committed with the release's report.
"""
import argparse
import datetime
import os
import re
import shutil
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RUNS = os.path.join(ROOT, "validation", "runs")
COLUMNS = ["utc", "version", "identity", "exit", "verdict", "config", "command"]


def default_exe():
    name = "fractadyne.exe" if os.name == "nt" else "fractadyne"
    return os.path.join(ROOT, "target", "release", name)


def workspace_version():
    with open(os.path.join(ROOT, "Cargo.toml"), encoding="utf-8") as f:
        m = re.search(r'^version\s*=\s*"([^"]+)"', f.read(), re.M)
    return m.group(1) if m else "unknown"


def repo_identity():
    def git(*a):
        r = subprocess.run(["git", *a], cwd=ROOT, capture_output=True, text=True, encoding="utf-8")
        return r.stdout.strip() if r.returncode == 0 else None
    sha = git("rev-parse", "--short", "HEAD")
    if sha is None:
        return "repo unknown"
    dirty = git("status", "--porcelain")
    return "repo g%s%s" % (sha, "-dirty" if dirty else "")


def app_identity(exe, env):
    """`(version, identity)` from the binary's own `--version`: `fractadyne V (build N, gSHA[-dirty])`."""
    r = subprocess.run([exe, "--version"], cwd=ROOT, env=env, capture_output=True, text=True,
                       encoding="utf-8", errors="replace")
    m = re.search(r"fractadyne (\S+) \(([^)]*)\)", r.stdout)
    if not m:
        sys.exit("run_ledger: %s --version printed no version line: %r" % (exe, r.stdout[:200]))
    return m.group(1), m.group(2)


def log_path(config_dir):
    return os.path.join(config_dir, "logs", "fractadyne.log")


def log_size(config_dir):
    try:
        return os.path.getsize(log_path(config_dir))
    except OSError:
        return 0


def log_verdict(config_dir, start):
    """The app's own account of the run: the `[fd-exit]` line and logcheck's verdict it wrote past
    byte `start` (a prepared config's log already holds other sessions' lines)."""
    try:
        with open(log_path(config_dir), "rb") as f:
            f.seek(start)
            text = f.read().decode("utf-8", errors="replace")
    except OSError:
        return "no log"
    # The file writes `[+ t] [fd-exit] livetest exit 0`; the console, `[fd-exit] [+ t] …`.
    exits = re.findall(r"\[fd-exit\]\s+(?:\[\+[^\]]*\]\s+)?(.+)", text)
    checks = re.findall(r"\[fd-logcheck\]\s+(?:\[\+[^\]]*\]\s+)?(\S+)", text)
    if not exits:
        return "no [fd-exit]"
    return exits[-1].strip() + ("; logcheck " + checks[-1] if checks else "")


def clean(s):
    """One TSV field: no tabs or line breaks, forward slashes."""
    return re.sub(r"[\t\r\n]+", " ", s).replace("\\", "/")


def ledger_path(version):
    return os.path.join(RUNS, "%s.tsv" % version)


def append(row):
    os.makedirs(RUNS, exist_ok=True)
    path = ledger_path(row["version"])
    new = not os.path.exists(path)
    # Bytes, LF, UTF-8: the file is committed, and a text-mode write on Windows would store CRLF.
    with open(path, "ab") as f:
        if new:
            f.write(("\t".join(COLUMNS) + "\n").encode("utf-8"))
        f.write(("\t".join(clean(str(row[c])) for c in COLUMNS) + "\n").encode("utf-8"))
    return path


def read_ledger(version):
    """Every recorded row for `version`, as dicts (empty when nothing was recorded)."""
    path = ledger_path(version)
    if not os.path.exists(path):
        return []
    with open(path, encoding="utf-8") as f:
        lines = [l.rstrip("\n") for l in f if l.strip()]
    if not lines or lines[0].split("\t") != COLUMNS:
        sys.exit("run_ledger: %s does not start with the header %s" % (path, "\t".join(COLUMNS)))
    return [dict(zip(COLUMNS, l.split("\t"))) for l in lines[1:]]


def matches(command, needle):
    """Does a recorded command contain the enforcer's tokens, in order? (`--livetest tours/grand-tour.toml`
    matches `fractadyne --livetest tours/grand-tour.toml --size 480x270`.)"""
    have = clean(command).split()
    want = clean(needle).split()
    i = 0
    for tok in have:
        if i < len(want) and tok == want[i]:
            i += 1
    return i == len(want)


def split_accepted(needle):
    """`--autodive-home [exit 0,3]` -> (`--autodive-home`, {"0", "3"}). Without the suffix only 0
    passes; a row that accepts more says so, and why, in design/checklist-automation.md."""
    m = re.match(r"^(.*?)\s*\[exit ([0-9, ]+)\]\s*$", needle)
    if not m:
        return needle, {"0"}
    return m.group(1), {c.strip() for c in m.group(2).split(",") if c.strip()}


def satisfies(row, version, needle):
    """Why this row does or does not satisfy `run:<needle>` for `version`: `(ok, reason)`."""
    cmd, accepted = split_accepted(needle)
    if row["version"] != version or not matches(row["command"], cmd):
        return False, "not this gate"
    if "-dirty" in row["identity"] or "unknown" in row["identity"]:
        return False, "ran from an uncommitted tree (%s)" % row["identity"]
    if row["exit"] not in accepted:
        return False, "exit %s (%s)" % (row["exit"], row["verdict"])
    return True, "exit %s, %s, %s" % (row["exit"], row["identity"], row["utc"])


def cmd_run(args):
    argv = args.command
    if argv and argv[0] == "--":
        argv = argv[1:]
    if not argv:
        sys.exit("run_ledger run: no command (put it after --)")
    env = dict(os.environ)
    env.setdefault("FRACTADYNE_NO_SOUND", "1")
    app = argv[0].startswith("--")
    config = "-"
    if app:
        exe = args.exe or default_exe()
        if not os.path.exists(exe):
            sys.exit("run_ledger: no binary at %s (build it, or pass --exe)" % exe)
        if args.config:
            env["FRACTADYNE_CONFIG_DIR"] = os.path.abspath(args.config)
            config = "given"
        else:
            env["FRACTADYNE_CONFIG_DIR"] = tempfile.mkdtemp(prefix="fractadyne-gate-")
            config = "fresh"
        version, identity = app_identity(exe, env)
        if config == "fresh":
            # The `--version` probe wrote its own log line; the gate's log starts clean.
            shutil.rmtree(os.path.join(env["FRACTADYNE_CONFIG_DIR"], "logs"), ignore_errors=True)
        cmd = [exe] + argv
        shown = "fractadyne " + " ".join(argv)
    else:
        version, identity = workspace_version(), repo_identity()
        cmd = argv
        shown = " ".join(argv)
    print("run_ledger: %s  [%s %s]" % (shown, version, identity), flush=True)
    start = log_size(env["FRACTADYNE_CONFIG_DIR"]) if app else 0
    code = subprocess.run(cmd, cwd=ROOT, env=env).returncode
    verdict = log_verdict(env["FRACTADYNE_CONFIG_DIR"], start) if app else "-"
    row = {
        "utc": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d %H:%M:%S"),
        "version": version,
        "identity": identity,
        "exit": code,
        "verdict": verdict,
        "config": config,
        "command": shown,
    }
    path = append(row)
    print("run_ledger: exit %d (%s) recorded in %s" % (code, verdict, os.path.relpath(path, ROOT)))
    if app and config == "fresh":
        print("run_ledger: the run's config and log: %s" % env["FRACTADYNE_CONFIG_DIR"])
    return code


def cmd_show(args):
    version = args.version or workspace_version()
    rows = read_ledger(version)
    if not rows:
        print("run_ledger: nothing recorded for %s (%s)" % (version, os.path.relpath(ledger_path(version), ROOT)))
        return 0
    for r in rows:
        print("%s  exit %-3s %-28s %s  [%s]" % (r["utc"], r["exit"], r["identity"], r["command"], r["verdict"]))
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    sub = ap.add_subparsers(dest="what", required=True)
    r = sub.add_parser("run", help="run a gate and record its exit code")
    r.add_argument("--exe", help="the app binary for app arguments (default: target/release)")
    r.add_argument("--config", metavar="DIR",
                   help="start the app from this config directory instead of a fresh one (recorded)")
    r.add_argument("command", nargs=argparse.REMAINDER, help="-- then the app arguments or a command")
    s = sub.add_parser("show", help="list what was recorded for a version")
    s.add_argument("version", nargs="?", help="default: the workspace version")
    args = ap.parse_args()
    return cmd_run(args) if args.what == "run" else cmd_show(args)


if __name__ == "__main__":
    sys.exit(main())
