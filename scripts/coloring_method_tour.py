#!/usr/bin/env python3
"""The live method tour (coloring plan B1b): `--livetest tours/coloring-methods.toml` once per
coloring method, each against its own baseline, and one line per method at the end.

    python -I scripts/coloring_method_tour.py --exe target/release/fractadyne.exe --out DIR
        [--size 480x270] [--methods smooth,stripe,...] [--bless] [--monitor right]

Every run gets a scratch config folder of its own under DIR (never the real one) and no sound, and
its window goes on the named monitor through scripts/on_monitor.py. A run's whole output is kept as
DIR/<method>.txt. The exit status is the worst run's (`crate::verdict`: 0 pass, 2 drift or a failing
checkpoint with no baseline, 3 nothing judged).
"""
import argparse
import os
import subprocess
import sys
from pathlib import Path

# The method keys, in the method table's order (fractadyne_core::coloring). The app refuses an
# unknown key, so a stale list fails loudly rather than skipping a method.
METHODS = ["smooth", "stripe", "triangle", "trap", "distance", "fringe", "decomposition"]
TOUR = "tours/coloring-methods.toml"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--exe", default="target/release/fractadyne.exe")
    ap.add_argument("--out", required=True, help="a folder for the configs, logs and outputs")
    ap.add_argument("--size", default="480x270")
    ap.add_argument("--methods", default=",".join(METHODS))
    ap.add_argument("--bless", action="store_true", help="record each method's baseline on this card")
    ap.add_argument("--monitor", default="right", help="scripts/on_monitor.py's monitor, or 'none'")
    a = ap.parse_args()

    repo = Path(__file__).resolve().parent.parent
    out = Path(a.out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    rows, worst = [], 0
    for m in [s.strip() for s in a.methods.split(",") if s.strip()]:
        cfg = out / f"cfg-{m}"
        cfg.mkdir(exist_ok=True)
        cmd = [a.exe, "--livetest", TOUR, "--method", m, "--size", a.size, "--out", str(out / m)]
        if a.bless:
            cmd.append("--bless")
        if a.monitor != "none":
            cmd = [sys.executable, "-I", str(repo / "scripts" / "on_monitor.py"), a.monitor, "--"] + cmd
        env = dict(os.environ, FRACTADYNE_CONFIG_DIR=str(cfg), FRACTADYNE_NO_SOUND="1")
        print(f"== {m}", flush=True)
        r = subprocess.run(cmd, cwd=repo, env=env, capture_output=True, text=True, encoding="utf-8", errors="replace")
        text = r.stdout + r.stderr
        (out / f"{m}.txt").write_text(text, encoding="utf-8")
        summary = next((l.strip() for l in text.splitlines() if "checkpoint(s):" in l), "no summary line")
        verdict = next((l.strip() for l in text.splitlines() if l.startswith("--livetest: VERDICT")), "no verdict")
        rows.append((m, r.returncode, summary, verdict))
        worst = max(worst, r.returncode)
        print(f"   exit {r.returncode}  {summary}  {verdict}", flush=True)
    print()
    for m, code, summary, verdict in rows:
        print(f"{m:<14} exit {code}  {summary}  {verdict}")
    return worst


if __name__ == "__main__":
    sys.exit(main())
