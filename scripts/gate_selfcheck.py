#!/usr/bin/env python3
"""The meta-gate: every gate the release checklist relies on must go RED when what it guards breaks.

    python scripts/gate_selfcheck.py              # every arm (about 5 minutes on an RTX 3080)
    python scripts/gate_selfcheck.py --only livetest
    python scripts/gate_selfcheck.py --list       # the inventory: which gates have an arm

A gate that cannot go red is not a gate, and this workstream's product IS gates
(design/live-render-robustness.md 7.8). Each ARM runs one gate twice in a scratch copy of the data
it reads (validation/, benchmarks/, tours/ - the app prefers the working directory's copy, so the
repository is never touched):
  control   unmodified: must PASS (exit 0). A sabotage only proves something against a control
            that passed - otherwise the gate may simply be red already.
  sabotage  one deliberate breakage of exactly what the gate guards (a corrupted golden, a
            corrupted baseline, an override known to break the run): must exit NON-ZERO, must have
            reached its end (an [fd-exit] line in its log - a crash is not a catch), and must name
            the breakage in its output (a gate red for the WRONG reason proves nothing).
Gates the checklist names that have no arm yet are listed as GAPS.

Exit: 0 every arm correct and no gaps; 2 an arm went wrong (a sabotage passed, a control failed,
or the reason is missing); 3 every arm correct but gaps remain. Not a --set on --selftest: the
self-test fails ANY override on its own "tunables are stock" check, so it would go red for the
wrong reason (7.8) - its arms corrupt data instead.
"""
import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
EXE = os.path.join(ROOT, "target", "release", "fractadyne.exe" if os.name == "nt" else "fractadyne")


# ------------------------------------------------------------------------------------- sabotages
def replace_once(path, old, new):
    """Byte-exact single replacement; the anchor must occur exactly once."""
    b = open(path, "rb").read()
    n = b.count(old)
    if n != 1:
        raise RuntimeError("%s: anchor occurs %d times, expected 1: %r" % (path, n, old[:80]))
    open(path, "wb").write(b.replace(old, new))


def swap_golden(sb):
    """The home golden replaced by another view's image: the render no longer matches it."""
    g = os.path.join(sb, "validation", "golden")
    shutil.copyfile(os.path.join(g, "seahorse.png"), os.path.join(g, "home.png"))


def corrupt_replay(sb):
    """One recorded decision changed by one iteration: the replay recomputes a different plan.
    Only the FIRST decision line (line 2; line 1 is the header) is touched."""
    p = os.path.join(sb, "validation", "replays", "escaped-ref.jsonl")
    lines = open(p, "rb").read().split(b"\n")
    old, new = b'{"decision":{"budget_now":4627,', b'{"decision":{"budget_now":4628,'
    if not lines[1].startswith(old):
        raise RuntimeError("%s: line 2 does not begin %r" % (p, old))
    lines[1] = new + lines[1][len(old):]
    open(p, "wb").write(b"\n".join(lines))


def corrupt_bench_baseline(sb):
    """The reference baseline's first segment claims one rebase more than the path performs."""
    p = os.path.join(sb, "benchmarks", "bench-matrix-baseline.json")
    t = open(p, encoding="utf-8").read()
    m = re.search(r'"counters": \[\s*(\d+)', t)
    if not m:
        raise RuntimeError("no counters in %s" % p)
    t = t[:m.start(1)] + str(int(m.group(1)) + 1) + t[m.end(1):]
    open(p, "wb").write(t.encode("utf-8"))


def corrupt_livetest_baseline(sb):
    """The blessed 'open' checkpoint recorded 30 points blacker than the view really is."""
    p = os.path.join(sb, "benchmarks", "livetest-grand-tour-480x270.json")
    t = open(p, encoding="utf-8").read()
    m = re.search(r'"open": \{[^}]*?"live_black": ([0-9.eE+-]+)', t, re.S)
    if not m:
        raise RuntimeError("no 'open' checkpoint in %s" % p)
    t = t[:m.start(1)] + repr(float(m.group(1)) + 0.30) + t[m.end(1):]
    open(p, "wb").write(t.encode("utf-8"))


# ----------------------------------------------------------------------------------------- arms
LIVETEST = ["--livetest", "tours/grand-tour.toml", "--segment", "whole-set", "--size", "480x270"]
ARMS = [
    {
        "gate": "selftest", "name": "selftest/golden",
        "args": ["--selftest", "--selftest-filter", "home", "--out", "{sb}/st/report.md"],
        "sabotage": swap_golden, "why": "the home golden replaced by the seahorse image",
        "expect": r"\[FAIL\] golden home",
    },
    {
        "gate": "selftest", "name": "selftest/replay",
        "args": ["--selftest", "--selftest-filter", "replay", "--out", "{sb}/st/report.md"],
        "sabotage": corrupt_replay, "why": "one recorded decision off by one iteration",
        "expect": r"\[FAIL\] escaped-ref\.jsonl replays to its decisions",
    },
    {
        "gate": "bench-matrix", "name": "bench-matrix/signature",
        "args": ["--bench-matrix", "--reps", "1"],
        "sabotage": corrupt_bench_baseline, "why": "the baseline's first segment claims one more rebase",
        "expect": r"ALGORITHMIC DRIFT",
    },
    {
        "gate": "livetest", "name": "livetest/baseline",
        "args": LIVETEST,
        "sabotage": corrupt_livetest_baseline, "why": "the blessed 'open' checkpoint 30 points blacker",
        "expect": r"live black .* →|DRIFT",
    },
    {
        # 7.8's named red-check: a lethal band of 1 ms sheds on every pass, and the run must go
        # red THROUGH LOGCHECK, not merely be noisy. ⚠On the landmarks chapter, not whole-set: at
        # the home view no pass reaches even 1 ms, so the first version of this arm never entered
        # the regime it tests and the run passed (2026-10-15) - an arm can be vacuous too.
        "gate": "livetest", "name": "livetest/lethal-band",
        "args": ["--livetest", "tours/grand-tour.toml", "--segment", "landmarks", "--size", "480x270"],
        "extra": ["--set", "TDR_LETHAL_MS=1"],
        "sabotage": None, "why": "--set TDR_LETHAL_MS=1 on the landmarks chapter (passes of a few ms)",
        "expect": r"logcheck: FAIL lethal-band",
    },
    {
        # A reachability fixture for the soak's crash criterion: a crash report appears in the run's
        # own log folder while it runs (the soak judges reports that appeared DURING it). 45 s: two
        # 20 s liveness windows, so the run is judged at all rather than VACUOUS.
        "gate": "soak", "name": "soak/crash-report",
        "args": ["--soak", "45"],
        "midrun": (15, lambda cfg: open(os.path.join(cfg, "logs", "crash-gate-selfcheck.txt"), "w").write(
            "a crash report planted by scripts/gate_selfcheck.py\n")),
        "sabotage": None, "why": "a crash report written into the run's log folder 15 s in",
        "expect": r"soak-liveness: FAIL .*crash report\(s\) appeared: crash-gate-selfcheck\.txt",
    },
    {
        # The same fixture for the UI walk's no-crash-files criterion, which fails the RUN (a crash
        # report from a worker thread while every step passes is exactly what it exists to catch).
        "gate": "uitest", "name": "uitest/crash-report",
        "args": ["--uitest", "{sb}/ui"],
        "midrun": (30, lambda cfg: open(os.path.join(cfg, "logs", "crash-gate-selfcheck.txt"), "w").write(
            "a crash report planted by scripts/gate_selfcheck.py\n")),
        "sabotage": None, "why": "a crash report written into the run's log folder 30 s into the walk",
        "expect": r"no-crash-files: FAIL .*crash-gate-selfcheck\.txt",
    },
    {
        # A 1 ms lethal band: every reading is lethal and no retreat can bring a pass under it, so
        # the dive reaches the regime and the retreat does not hold - --autodive's RED. The control
        # is the checklist's row 32 run, which on a card that never reaches the regime is VACUOUS
        # (3), the exit that row accepts.
        "gate": "autodive", "name": "autodive/retreat-cannot-hold",
        "args": ["--autodive", "12", "--autodive-home", "0"], "control_ok": [0, 3],
        "extra": ["--set", "TDR_LETHAL_MS=1"],
        "sabotage": None, "why": "--set TDR_LETHAL_MS=1 (no retreat can leave the lethal band)",
        "expect": r"REACHED THE REGIME AND THE RETREAT DID NOT HOLD",
    },
    {
        # The step budget starved to 1: no pinned refresh can complete, so detail stops streaming
        # during motion - exactly A2's "option-B freeze shape". (TDR_STEPS_CEIL alone is refused at
        # startup, below TDR_MIN_STEPS: an argument error, which this script never counts as a catch.)
        "gate": "motiontest", "name": "motiontest/starved-refresh",
        "args": ["--motiontest"], "extra": ["--set", "TDR_MIN_STEPS=1", "--set", "TDR_STEPS_CEIL=1"],
        "sabotage": None, "why": "--set TDR_MIN_STEPS=1 TDR_STEPS_CEIL=1 (no refresh can complete)",
        "expect": r"A2: no complete refresh was adopted during motion",
    },
    {
        # A NaN scale planted after the autopilot's step at frame 300: the shape of three field
        # sessions, which reached one with nothing naming the cause. The guard must catch it the
        # frame it appears and put back the last usable view; logcheck's unusable-view rule fails
        # the run (the motiontest's own verdict passes, because the view was put back).
        "gate": "motiontest", "name": "motiontest/unusable-view",
        "args": ["--motiontest"], "env": {"FRACTADYNE_POISON_VIEW_AT": "300"},
        "sabotage": None, "why": "FRACTADYNE_POISON_VIEW_AT=300 (the view's scale set to NaN at frame 300)",
        "expect": r"logcheck: FAIL unusable-view",
    },
    {
        # Coarse: the frame record switched off altogether (FRACTADYNE_LOG=0), not a field left
        # unset - the app has no knob for the subtler breakage. It proves the gate reads the record
        # it guards rather than passing on its absence.
        "gate": "recordtest", "name": "recordtest/no-record",
        "args": ["--recordtest"], "env": {"FRACTADYNE_LOG": "0"},
        "sabotage": None, "why": "FRACTADYNE_LOG=0: no log directory, so no frame record",
        "expect": r"no logs directory, so no frames\.bin",
    },
    {
        # The record-cost limit at 1 ppm of a 60 Hz frame: over in the first window, re-measured,
        # over in the second. Proves the near-miss re-measure still lets the check go red.
        "gate": "recordtest", "name": "recordtest/cost-both-windows",
        "args": ["--recordtest"], "extra": ["--set", "RECORD_COST_PPM=1"],
        "sabotage": None, "why": "--set RECORD_COST_PPM=1 (every emit is over the limit)",
        "expect": r"p99 costs [0-9.]+% of a 60 Hz frame \(limit [0-9.]+%\), in both windows",
    },
    {
        # A SCRIPT gate, in a copy of exactly the files it reads (it resolves everything from its
        # own location): one row's enforcer renamed to a test that does not exist.
        "gate": "checklist_coverage.py", "name": "checklist/broken-claim", "kind": "script",
        # (scripts/ whole: script: and run: rows name scripts that must exist)
        "tree": ["scripts", "design/checklist-automation.md", "validation/golden/BLESSED-GPU.txt", "crates"],
        "cmd": [sys.executable, "scripts/checklist_coverage.py"],
        "sabotage": lambda t: replace_once(os.path.join(t, "scripts", "release_checklist.py"),
                                           b'"test:title_string_matches_version"', b'"test:no_such_test_anywhere"'),
        "why": "a row's enforcer renamed to a test that does not exist",
        "expect": r"BROKEN CLAIM.*\n.*no_such_test_anywhere",
    },
    {
        # The Fraktaler-3 corpus check on one cheap row, in a copy of what it reads (it resolves
        # everything, the binary included, from its own location): row 01's committed render
        # replaced by row 02's, same size. Possible since the df32 tail fix (2026-10-15): before it
        # the check's own control was red (28/38).
        "gate": "corpus --check", "name": "corpus/swapped-baseline", "kind": "script",
        "tree": ["validation/corpus/generate_corpus.py", "validation/corpus/locations.toml",
                 "validation/corpus/session-template.toml",
                 "validation/corpus/renders/01-home-fractadyne.png",
                 "validation/corpus/renders/02-seahorse-1e4-fractadyne.png",
                 "target/release/" + os.path.basename(EXE)],
        "cmd": [sys.executable, "validation/corpus/generate_corpus.py", "--check", "--only", "01"],
        "sabotage": lambda t: shutil.copyfile(
            os.path.join(t, "validation", "corpus", "renders", "02-seahorse-1e4-fractadyne.png"),
            os.path.join(t, "validation", "corpus", "renders", "01-home-fractadyne.png")),
        "why": "row 01's committed render replaced by row 02's",
        "expect": r"01-home\s+CHANGED",
    },
    {
        # SELF-ARMED: the screen gate's own --selftest trips every criterion on synthetic captures
        # beside a control that must pass, and checks each exit code - the per-assertion fixtures
        # 7.8 asks for. Run as is: it must exit 0 with its PASS line.
        "gate": "screengate.py", "name": "screengate/own-fixtures", "kind": "self-armed",
        "cmd": [sys.executable, "scripts/dive-capture/screengate.py", "--selftest"],
        "why": "its --selftest: every criterion seen to fire on a synthetic capture",
        "expect": r"(?m)^selftest: PASS$",
    },
]

# Every gate the release checklist relies on (its run: rows and its "must also be green" list),
# and the arm(s) that prove it can go red. An entry with no arm is a GAP, with the reason.
INVENTORY = [
    ("selftest", None),
    ("bench-matrix", None),
    ("livetest", None),
    ("motiontest", None),
    ("uitest", None),
    ("recordtest", None),
    ("autodive", None),
    ("soak", None),
    ("screengate.py", None),
    ("corpus --check", None),
    ("checklist_coverage.py", None),
]


# Gates the checklist only PLANS (its planned: rows): listed, not counted as gaps, armed when they
# become gates.
PLANNED = [
    ("zoomtest-gate", "the P-G matrix: RED on every Smooth row on beta.19 too, and its fixture "
                      "misi-49-3.fdn is not in the tree"),
    ("torture increment 2", "the rung passed three runs running on the RX 6800 XT and runs in every field battery (step 09); the checklist cannot read a field battery"),
]


# ---------------------------------------------------------------------------------------- running
def make_sandbox():
    sb = tempfile.mkdtemp(prefix="gate-selfcheck-")
    for d in ("validation", "benchmarks", "tours"):
        shutil.copytree(os.path.join(ROOT, d), os.path.join(sb, d),
                        ignore=shutil.ignore_patterns("current", "report.md", "runs"))
    return sb


def run(arm, sb, sabotaged):
    cfg = tempfile.mkdtemp(prefix="gate-cfg-")
    args = [a.replace("{sb}", sb) for a in arm["args"]] + (arm.get("extra", []) if sabotaged else [])
    env = dict(os.environ, FRACTADYNE_CONFIG_DIR=cfg, FRACTADYNE_NO_SOUND="1")
    if sabotaged:
        env.update(arm.get("env", {}))
    timer = None
    if sabotaged and arm.get("midrun"):
        delay, act = arm["midrun"]
        timer = threading.Timer(delay, act, args=[cfg])
        timer.start()
    t0 = time.time()
    p = subprocess.run([EXE] + args, cwd=sb, env=env, capture_output=True, text=True,
                       encoding="utf-8", errors="replace", timeout=1800)
    out = p.stdout + "\n" + p.stderr
    if timer:
        timer.cancel()
    try:
        log = open(os.path.join(cfg, "logs", "fractadyne.log"), encoding="utf-8", errors="replace").read()
    except OSError:
        log = ""
    shutil.rmtree(cfg, ignore_errors=True)
    # Reached its end: the [fd-exit] line, in the log or (when the sabotage turns the log off) on
    # the console, which carries it too.
    return p.returncode, out, "[fd-exit]" in log or "[fd-exit]" in out, time.time() - t0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--only", help="run only the arms whose name contains this")
    ap.add_argument("--list", action="store_true", help="print the inventory and exit")
    ap.add_argument("--keep", action="store_true", help="keep the scratch sandbox")
    a = ap.parse_args()

    armed = {arm["gate"] for arm in ARMS}
    gaps = [(g, why) for g, why in INVENTORY if g not in armed]
    if a.list:
        for g, why in INVENTORY:
            names = [arm["name"] for arm in ARMS if arm["gate"] == g]
            print("  %-22s %s" % (g, ", ".join(names) if names else "GAP - " + why))
        for g, why in PLANNED:
            print("  %-22s planned, not yet a gate - %s" % (g, why))
        return 0
    if not os.path.exists(EXE):
        sys.exit("gate_selfcheck: no binary at %s (cargo build --release)" % EXE)

    sb = make_sandbox()
    wrong = []
    try:
        for arm in ARMS:
            if a.only and a.only not in arm["name"]:
                continue
            print("== %s  (%s)" % (arm["name"], arm["why"]), flush=True)
            if arm.get("kind") == "script":
                tree = tempfile.mkdtemp(prefix="gate-tree-")
                for rel in arm["tree"]:
                    src, dst = os.path.join(ROOT, rel), os.path.join(tree, rel)
                    os.makedirs(os.path.dirname(dst), exist_ok=True)
                    if os.path.isdir(src):
                        shutil.copytree(src, dst, ignore=shutil.ignore_patterns("target"))
                    else:
                        shutil.copyfile(src, dst)
                env = dict(os.environ, PYTHONIOENCODING="utf-8")
                results = []
                for sabotaged in (False, True):
                    if sabotaged:
                        arm["sabotage"](tree)
                    p = subprocess.run(arm["cmd"], cwd=tree, capture_output=True, text=True,
                                       encoding="utf-8", errors="replace", timeout=1800, env=env)
                    results.append((p.returncode, p.stdout + "\n" + p.stderr))
                shutil.rmtree(tree, ignore_errors=True)
                (c_code, c_out), (s_code, s_out) = results
                named = re.search(arm["expect"], s_out) is not None
                print("   control   exit %d%s" % (c_code, "" if c_code == 0 else "  <- the CONTROL must pass"))
                print("   sabotage  exit %d  %s" % (s_code, "RED, for the named reason" if s_code != 0 and named
                                                    else "<- " + ("still exits 0" if s_code == 0 else "red without naming it")), flush=True)
                if c_code != 0 or s_code == 0 or not named:
                    wrong.append((arm["name"], "control %d, sabotage %d, named %s" % (c_code, s_code, named)))
                    print("\n".join("     | " + l for l in (c_out if c_code else s_out).strip().splitlines()[-12:]))
                continue
            if arm.get("kind") == "self-armed":
                t0 = time.time()
                p = subprocess.run(arm["cmd"], cwd=ROOT, capture_output=True, text=True,
                                   encoding="utf-8", errors="replace", timeout=1800,
                                   env=dict(os.environ, PYTHONIOENCODING="utf-8"))
                out = p.stdout + "\n" + p.stderr
                ok = p.returncode == 0 and re.search(arm["expect"], out) is not None
                print("   own fixtures  exit %d in %.0fs  %s" % (p.returncode, time.time() - t0,
                      "every criterion fired as declared" if ok else "<- its own fixtures FAILED"), flush=True)
                if not ok:
                    wrong.append((arm["name"], "its own fixtures: exit %d" % p.returncode))
                    print("\n".join("     | " + l for l in out.strip().splitlines()[-12:]))
                continue
            code, out, ended, secs = run(arm, sb, sabotaged=False)
            ok = code in arm.get("control_ok", [0]) and ended
            print("   control   exit %d in %.0fs%s" % (code, secs, "" if ok else "  <- the CONTROL must pass"), flush=True)
            if not ok:
                wrong.append((arm["name"], "control exit %d%s" % (code, "" if ended else ", no [fd-exit]")))
                print("\n".join("     | " + l for l in out.strip().splitlines()[-12:]))
                continue
            backup = None
            if arm["sabotage"]:
                backup = sb + ".pristine"
                if not os.path.exists(backup):
                    shutil.copytree(sb, backup)
                arm["sabotage"](sb)
            code, out, ended, secs = run(arm, sb, sabotaged=True)
            named = re.search(arm["expect"], out) is not None
            ok = code not in arm.get("control_ok", [0]) and ended and named
            why = []
            passing = arm.get("control_ok", [0])
            if code in passing:
                why.append("still exits %d, a passing code" % code)
            if not ended:
                why.append("no [fd-exit]: it crashed or was killed, which is not a catch")
            if code not in passing and not named:
                why.append("red, but its output does not name the breakage (/%s/)" % arm["expect"])
            print("   sabotage  exit %d in %.0fs  %s" % (code, secs, "RED, for the named reason" if ok else "<- " + "; ".join(why)), flush=True)
            if not ok:
                wrong.append((arm["name"], "; ".join(why)))
                print("\n".join("     | " + l for l in out.strip().splitlines()[-12:]))
            if backup:
                shutil.rmtree(sb)
                shutil.copytree(backup, sb)
    finally:
        if a.keep:
            print("sandbox kept: %s" % sb)
        else:
            shutil.rmtree(sb, ignore_errors=True)
            shutil.rmtree(sb + ".pristine", ignore_errors=True)

    print("\nGaps - gates the checklist relies on with no sabotage arm yet:")
    for g, why in gaps:
        print("  %-22s %s" % (g, why))
    print("Planned in the checklist, not yet gates (not counted):")
    for g, why in PLANNED:
        print("  %-22s %s" % (g, why))
    if wrong:
        print("\nWRONG: %d arm(s) - a gate that cannot go red for its reason is not a gate:" % len(wrong))
        for n, why in wrong:
            print("  %-26s %s" % (n, why))
        print("gate_selfcheck: RED")
        return 2
    if gaps:
        print("\ngate_selfcheck: every arm went red for its reason; %d gate(s) have no arm yet (exit 3)" % len(gaps))
        return 3
    print("\ngate_selfcheck: PASS - every gate went red for its reason")
    return 0


if __name__ == "__main__":
    sys.exit(main())
