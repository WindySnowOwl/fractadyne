#!/usr/bin/env python3
"""Read Fractadyne's per-frame record (design/live-render-robustness.md sections 6.3-6.6).

    python scripts/framelog.py summarize  <frames.bin | *-frames.jsonl> [--view N] [--target-ms MS]
    python scripts/framelog.py decode     <frames.bin> [-o out.jsonl]
    python scripts/framelog.py schema-check <file>
    python scripts/framelog.py compare    --a A1 A2 A3 [...] --b B1 B2 B3 [...]
    python scripts/framelog.py calibrate  <frames.bin ...> [--min-steps N] [--knee PX]
    python scripts/framelog.py selftest

Inputs: `<logs>/frames.bin` (the always-on circular file) or a `crash-*-frames.jsonl` written
beside a crash report. The encoding is `validation/frame-schema.json`, which `fractadyne
--dump-frame-schema` generates and a Rust test keeps in step with the code; this reader refuses a
file of any other schema rather than guessing.

`summarize` answers the three jobs the record exists for. It prints the session header (WHICH
code produced the records), the scorecard for one view, and then every SLOW EPISODE - a run of
frames the wall clock called slow while the app had asked for a repaint - with what the frame
budget controller was told during it. That last part discriminates the five mechanisms the
2026-09-21 RX 6800 XT budget stall could have been (finding U9); two of them were unanswerable from
the beta.112 decision ring, and all five are answerable from this record:

  (a) no reading arrived          - the timestamp readback never delivered for this view
  (b) armed on the wrong dispatch - readings price step counts no recent dispatch had
  (c) the bracket misses the work - readings are short while the frame is long, and price steps
                                    the frame really dispatched
  (d) the cost is outside the pass - the time outside any timed bracket is large and persistent
  (e) readings could not move it  - slow readings arrived and the budget did not come down

The labels are INDICATIVE: each is a pattern in the record, reported with its numbers.

`compare` is the A/B instrument for validating a fix by measurement. It requires three or more
runs per arm (single runs of this app vary 8-14%), reports every metric against its own run-to-run
spread, and SCORES THE CONTROL: arm A is split in half and compared with itself, and a metric that
"separates" A from A is noise, not evidence.

`selftest` feeds the discrimination synthetic episodes, one per mechanism, and fails if any is
mislabelled - a classifier that has never been seen to go red is not a classifier.

Standard library only. Exit codes: 0 ok, 1 a check failed, 2 VACUOUS (too little to judge).
"""
import json
import math
import os
import statistics
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
SCHEMA_PATH = os.path.join(HERE, "..", "validation", "frame-schema.json")
FMT = {"u8": "B", "u16": "H", "u32": "I", "u64": "Q", "f32": "f", "f64": "d", "bool": "B"}
PRESENT = {0: "unset", 1: "live", 2: "reproject", 3: "hold"}
SRC = {0: "-", 1: "gpu", 2: "wall"}
VERDICT = {0: "-", 1: "DISCARDED", 2: "moved", 3: "unchanged"}
# The minimum a summary needs before it will state a verdict (design 6.6.2's anti-vacuity).
MIN_FRAMES = 30


def schema_for(version, current):
    """The schema a file declares: the current one, or a kept older one
    (`validation/frame-schema-v<N>.json`, saved when the schema was bumped), so a field log from an
    older build still decodes. Anything else is refused rather than guessed at."""
    if version == current["schema"]:
        return current
    older = os.path.join(HERE, "..", "validation", f"frame-schema-v{version}.json")
    if not os.path.exists(older):
        raise SystemExit(f"schema {version}: this reader knows {current['schema']} and has no "
                         f"validation/frame-schema-v{version}.json - refusing to guess")
    return load_schema(older)


def load_schema(path=SCHEMA_PATH):
    with open(path, encoding="utf-8") as f:
        s = json.load(f)
    s["names"] = [n for n, _ in s["fields"]]
    s["types"] = dict(s["fields"])
    s["struct"] = struct.Struct("<" + "".join(FMT[t] for _, t in s["fields"]))
    if s["struct"].size != s["payload_bytes"]:
        raise SystemExit(f"schema file is inconsistent: fields pack to {s['struct'].size}, "
                         f"payload_bytes says {s['payload_bytes']}")
    return s


def _row(sch, values):
    r = dict(zip(sch["names"], values))
    for n, t in sch["fields"]:
        if t == "bool":
            if r[n] not in (0, 1):
                return None
            r[n] = bool(r[n])
    return r


def read_bin(path, sch):
    """(header dict, records oldest first, torn count). Refuses another schema."""
    with open(path, "rb") as f:
        data = f.read()
    hb, sb, sh = sch["header_bytes"], sch["slot_bytes"], sch["slot_header_bytes"]
    if len(data) < hb or data[:4] != sch["file_magic"].encode():
        raise SystemExit(f"{path}: not a frames.bin (bad magic)")
    schema = struct.unpack_from("<H", data, 4)[0]
    sch = schema_for(schema, sch)
    session = struct.unpack_from("<Q", data, 8)[0]
    hlen = struct.unpack_from("<I", data, 16)[0]
    try:
        header = json.loads(data[20:20 + min(hlen, hb - 20)].decode("utf-8") or "{}")
    except (ValueError, UnicodeDecodeError):
        header = {"header_unreadable": True}
    recs, torn = [], 0
    magic = sch["slot_magic"].encode()
    for off in range(hb, len(data) - sb + 1, sb):
        s = data[off:off + sb]
        if s[:4] != magic:
            continue
        sess = struct.unpack_from("<Q", s, 8)[0]
        if sess != session:
            continue  # an earlier session's leftover
        ln = struct.unpack_from("<H", s, 6)[0]
        crc = struct.unpack_from("<I", s, 24)[0]
        payload = s[sh:sh + ln]
        if struct.unpack_from("<H", s, 4)[0] != sch["schema"] or ln != sch["payload_bytes"] or fnv1a(payload) != crc:
            torn += 1
            continue
        r = _row(sch, sch["struct"].unpack(payload))
        if r is None:
            torn += 1
            continue
        recs.append(r)
    recs.sort(key=lambda r: r["seq"])
    header.setdefault("session", f"{session:016x}")
    return header, recs, torn


def fnv1a(b):
    h = 0x811C9DC5
    for x in b:
        h = ((h ^ x) * 0x01000193) & 0xFFFFFFFF
    return h


def read_jsonl(path, sch, strict=True):
    """(header dict, records, problems). Strict: an unknown or a missing key is a problem.
    `frames.jsonl` also carries one `"kind":"summary"` row a second; those are checked against
    the schema's `summary_keys` and returned in `header["summaries"]`."""
    header, recs, problems, summaries = {}, [], [], []
    want = set(sch["names"])
    want_summary = set(sch.get("summary_keys", []))
    with open(path, encoding="utf-8") as f:
        for i, line in enumerate(f, 1):
            line = line.strip()
            if not line:
                continue
            try:
                o = json.loads(line)
            except ValueError as e:
                problems.append(f"line {i}: not JSON ({e})")
                continue
            if o.get("kind") == "header":
                if o.get("schema") != sch["schema"]:
                    # An older build's file: check it against the schema it was written in.
                    try:
                        sch = schema_for(o.get("schema"), sch)
                        want = set(sch["names"])
                        want_summary = set(sch.get("summary_keys", []))
                    except SystemExit:
                        problems.append(f"line {i}: schema {o.get('schema')}, reader knows {sch['schema']}")
                header = dict(o.get("header") or {})
                header.setdefault("session", o.get("session"))
                continue
            if o.get("kind") == "summary":
                if strict and set(o) != want_summary:
                    problems.append(f"line {i}: summary row keys differ: unknown {sorted(set(o) - want_summary)[:5]}"
                                    f" missing {sorted(want_summary - set(o))[:5]}")
                    continue
                summaries.append(o)
                continue
            keys = set(o)
            if strict and keys != want:
                extra, missing = sorted(keys - want), sorted(want - keys)
                problems.append(f"line {i}: unknown {extra[:5]} missing {missing[:5]}")
                continue
            recs.append(o)
    recs.sort(key=lambda r: r.get("seq", 0))
    header["summaries"] = summaries
    return header, recs, problems


def load_any(path, sch):
    with open(path, "rb") as f:
        head = f.read(4)
    if head == sch["file_magic"].encode():
        header, recs, torn = read_bin(path, sch)
        return header, recs, ([f"{torn} torn slot(s)"] if torn else [])
    return read_jsonl(path, sch)


def pct(xs, p):
    if not xs:
        return float("nan")
    xs = sorted(xs)
    return xs[min(len(xs) - 1, max(0, int(round((len(xs) - 1) * p))))]


# ---------------------------------------------------------------------------------------------
# Slow episodes and the five mechanisms (finding U9).

def episodes(recs, target_ms):
    """Runs of consecutive frames the WALL called slow while a repaint was asked for. Without the
    repaint, eframe's ~1 Hz idle tick reads as a 1 s frame (the discriminator the app's own
    tripwire uses, and the one two earlier wrong diagnoses were missing)."""
    out, cur = [], []
    for r in recs:
        if r["last_dt_ms"] > target_ms and r["repaint_requested"]:
            cur.append(r)
        else:
            if len(cur) >= 3:
                out.append(cur)
            cur = []
    if len(cur) >= 3:
        out.append(cur)
    return out


def classify(ep, recs_by_frame, target_ms):
    """Indicators for one slow episode, and the mechanism(s) they fit."""
    reads = [r for r in ep if r["read_n"] > 0]
    slow_reads = [r for r in reads if r["read_ms"] > target_ms]
    moved_down = [r for r in reads if r["read_verdict"] == 2 and r["read_budget_after"] < r["read_budget_before"]]
    dt = [r["last_dt_ms"] for r in ep]
    # The time no bracket covered: the wall interval minus the longest reading judged in it.
    uncovered = [r["last_dt_ms"] - (r["read_ms"] if r["read_n"] else 0.0) for r in ep]
    # (b): does each reading's step count match a dispatch in the preceding few frames?
    mismatched = 0
    for r in reads:
        near = [recs_by_frame.get(r["frame"] - k) for k in range(0, 8)]
        steps = {x["nominal_steps"] for x in near if x and x["dispatched"]}
        if steps and r["read_steps"] not in steps:
            mismatched += 1
    # (d): a slow frame that submitted NO iterate spent its time somewhere no iterate timestamp
    # can see (resolve, colour, hold copy, accumulation, present). The beta.112 field log read
    # exactly this way: "body 0-4 ms + outside(acquire/present/idle) 208-1025 ms".
    undispatched = sum(1 for r in ep if not r["dispatched"])
    ind = {
        "frames": len(ep),
        "first": ep[0]["frame"],
        "last": ep[-1]["frame"],
        "dt_max": max(dt),
        "dt_p50": pct(dt, 0.5),
        "readings": len(reads),
        "slow_readings": len(slow_reads),
        "budget_moved_down": len(moved_down),
        "discarded": sum(1 for r in reads if r["read_verdict"] == 1),
        "since_reading_max": max(r["frames_since_reading"] for r in ep),
        "uncovered_p50": pct(uncovered, 0.5),
        "undispatched": undispatched,
        "read_ms_p50": pct([r["read_ms"] for r in reads], 0.5) if reads else float("nan"),
        "mismatched_readings": mismatched,
        "budget_first": ep[0]["fe_budget"],
        "budget_last": ep[-1]["fe_budget"],
        "chunked": sum(1 for r in ep if r["chunked"]),
        "blind_warned": any(r["blind_warned"] for r in ep),
    }
    fits = []
    if not reads and ind["since_reading_max"] >= len(ep):
        fits.append("(a) no reading arrived")
    if reads and mismatched * 2 > len(reads):
        fits.append("(b) armed on the wrong dispatch")
    if reads and not slow_reads and not mismatched * 2 > len(reads) and undispatched * 2 < len(ep):
        # Short readings of the steps these frames dispatched, while the wall says long.
        fits.append("(c) the bracket misses the work")
    if undispatched * 2 >= len(ep):
        fits.append("(d) the cost is outside the timed pass")
    if slow_reads and not moved_down:
        fits.append("(e) slow readings did not move the budget")
    if not fits:
        fits.append("none of the five - the controller saw it and reacted")
    return ind, fits


# ---------------------------------------------------------------------------------------------
# summarize

def summarize(path, view=0, target_ms=400.0, out=sys.stdout):
    sch = load_schema()
    header, recs_all, problems = load_any(path, sch)
    p = lambda *a: print(*a, file=out)  # noqa: E731
    p(f"== {os.path.basename(path)}")
    for k in ("version", "adapter", "backend", "timestamp_query", "attach_bytes_granted", "tunables",
              "bignum", "window", "max_iter", "auto_iter", "session"):
        if k in header:
            p(f"  {k:22} {header[k]}")
    for pr in problems:
        p(f"  PROBLEM: {pr}")
    # Frame rows only: a STALL row is the watchdog's, carries the LAST recorded frame's index, and
    # would read as a duplicate frame in any per-view count.
    recs = [r for r in recs_all if r["view"] == view and r["kind"] == 1]
    stalls = [r for r in recs_all if r["kind"] == 2]
    p(f"  records                {len(recs_all)} ({len(recs)} frames of view {view}, {len(stalls)} watchdog stall rows)")
    for s in stalls:
        p(f"  STALL                  {s['stall_ms'] / 1000:.1f}s with nothing recorded after frame {s['frame']}"
          f" (at +{s['t_ms'] / 1000:.1f}s)")
    if header.get("summaries"):
        sm = header["summaries"]
        p(f"  summary rows           {len(sm)} covering {sum(s['frames'] for s in sm)} view-0 frames;"
          f" events written {sum(s['events'] for s in sm)}, dropped by the rate limit {sum(s['events_dropped'] for s in sm)}")
    if len(recs) < MIN_FRAMES:
        p(f"VACUOUS: {len(recs)} records for view {view}, fewer than {MIN_FRAMES} - no verdict")
        return 2
    frames = [r["frame"] for r in recs]
    gaps = (frames[-1] - frames[0] + 1) - len(set(frames))
    span = (recs[-1]["t_ms"] - recs[0]["t_ms"]) / 1000.0
    p(f"  frames                 {frames[0]}..{frames[-1]} over {span:.1f}s; {gaps} missing, "
      f"{len(frames) - len(set(frames))} duplicated")
    built = [r for r in recs if r["plan_calls"] > 0]
    dt = [r["last_dt_ms"] for r in recs if r["last_dt_ms"] > 0]
    p(f"  interval ms            p50 {pct(dt, .5):.1f}  p95 {pct(dt, .95):.1f}  max {max(dt, default=0):.0f}")
    pres = {}
    for r in built:
        pres[PRESENT.get(r["present"], "?")] = pres.get(PRESENT.get(r["present"], "?"), 0) + 1
    p(f"  presented              {pres}  (not built: {len(recs) - len(built)})")
    p(f"  dispatched             {sum(r['dispatched'] for r in recs)}  chunked {sum(r['chunked'] for r in built)}"
      f"  tiled {sum(r['tiled'] for r in built)}")
    reads = [r for r in recs if r["read_n"] > 0]
    vh = {}
    for r in reads:
        k = f"{SRC.get(r['read_src'], '?')}:{VERDICT.get(r['read_verdict'], '?')}"
        vh[k] = vh.get(k, 0) + 1
    p(f"  readings judged        {len(reads)} {vh}  lethal {sum(r['read_lethal'] for r in reads)}"
      f"  growth refused (building) {sum(1 for r in reads if r['refusal'] == 1)}")
    fb = [r["fe_budget"] for r in recs]
    p(f"  budget                 first {fb[0]:.3e}  last {fb[-1]:.3e}  min {min(fb):.3e}  max {max(fb):.3e}")
    ctr = [r for r in recs if r["ctr_new"]]
    if ctr:
        p(f"  counter readings       {len(ctr)}  rebase max {max(r['ctr_rebase'] for r in ctr):,}"
          f"  bla_skip=0 with rebase>0: {sum(1 for r in ctr if r['ctr_bla_skip'] == 0 and r['ctr_rebase'] > 0)}")
    # The regime that has killed every device (design 6.7): a complete (escaped) reference far
    # shorter than the ask, with BLA skipping nothing.
    regime = [r for r in built if not r["ref_partial"] and r["ref_len"] > 0 and r["iter"] >= 4 * r["ref_len"]]
    if regime:
        p(f"  ESCAPED-REFERENCE SHAPE {len(regime)} frames (iter >= 4 x ref_len, reference complete),"
          f" first f{regime[0]['frame']}: iter {regime[0]['iter']} vs ref_len {regime[0]['ref_len']}")
    cost = [r["rec_us"] for r in recs if r["seq"] >= 2 and r["rec_us"] > 0]
    if cost:
        p(f"  record cost            p50 {pct(cost, .5):.1f} us  p99 {pct(cost, .99):.1f} us")
    eps = episodes(recs, target_ms)
    by_frame = {r["frame"]: r for r in recs}
    p(f"  slow episodes (> {target_ms:.0f} ms, repaint asked, >= 3 frames): {len(eps)}")
    for ep in eps:
        ind, fits = classify(ep, by_frame, target_ms)
        p(f"    f{ind['first']}..f{ind['last']} ({ind['frames']} frames, dt p50 {ind['dt_p50']:.0f} max {ind['dt_max']:.0f} ms)"
          f"  readings {ind['readings']} (slow {ind['slow_readings']}, discarded {ind['discarded']},"
          f" moved down {ind['budget_moved_down']})  budget {ind['budget_first']:.3e} -> {ind['budget_last']:.3e}")
        p(f"      frames since a reading <= {ind['since_reading_max']}, uncovered ms p50 {ind['uncovered_p50']:.0f},"
          f" mismatched readings {ind['mismatched_readings']}, chunked frames {ind['chunked']},"
          f" blind tripwire {'fired' if ind['blind_warned'] else 'silent'}")
        p(f"      fits: {'; '.join(fits)}  [indicative]")
    return 1 if problems else 0


# ---------------------------------------------------------------------------------------------
# compare

METRICS = [
    ("interval p50 ms", lambda rs: pct([r["last_dt_ms"] for r in rs if r["last_dt_ms"] > 0], .5)),
    ("interval p95 ms", lambda rs: pct([r["last_dt_ms"] for r in rs if r["last_dt_ms"] > 0], .95)),
    ("slow frames %", lambda rs: 100.0 * sum(r["last_dt_ms"] > 100 for r in rs) / max(1, len(rs))),
    ("not live %", lambda rs: 100.0 * sum(r["present"] not in (0, 1) for r in rs if r["plan_calls"]) / max(1, sum(1 for r in rs if r["plan_calls"]))),
    ("readings discarded %", lambda rs: 100.0 * sum(r["read_verdict"] == 1 for r in rs if r["read_n"]) / max(1, sum(1 for r in rs if r["read_n"]))),
    ("record cost p99 us", lambda rs: pct([r["rec_us"] for r in rs if r["seq"] >= 2 and r["rec_us"] > 0], .99)),
]


def _arm(paths, sch, view):
    out = []
    for p in paths:
        _, recs, _ = load_any(p, sch)
        out.append([r for r in recs if r["view"] == view and r["kind"] == 1])
    return out


def _separates(a, b):
    """True when every value of b lies outside a's range, on one side."""
    return (min(b) > max(a)) or (max(b) < min(a))


def compare(a_paths, b_paths, view=0, out=sys.stdout):
    p = lambda *x: print(*x, file=out)  # noqa: E731
    if len(a_paths) < 3 or len(b_paths) < 3:
        p(f"VACUOUS: compare needs at least 3 runs per arm (got {len(a_paths)} and {len(b_paths)}):"
          " single runs vary 8-14%, and one pair cannot tell a change from that")
        return 2
    sch = load_schema()
    A, B = _arm(a_paths, sch, view), _arm(b_paths, sch, view)
    half = len(A) // 2
    A1, A2 = A[:half], A[half:]
    p(f"{'metric':24} {'A mean [min..max]':>28} {'B mean [min..max]':>28} {'B-A':>9}  verdict   control(A vs A)")
    noisy = 0
    for name, f in METRICS:
        a = [f(rs) for rs in A]
        b = [f(rs) for rs in B]
        a = [x for x in a if not math.isnan(x)]
        b = [x for x in b if not math.isnan(x)]
        if len(a) < 3 or len(b) < 3:
            p(f"{name:24} (not measured in enough runs)")
            continue
        sep = _separates(a, b)
        c1 = [x for x in (f(rs) for rs in A1) if not math.isnan(x)]
        c2 = [x for x in (f(rs) for rs in A2) if not math.isnan(x)]
        ctrl = bool(c1 and c2 and _separates(c1, c2))
        noisy += ctrl
        fmt = lambda xs: f"{statistics.mean(xs):9.2f} [{min(xs):.2f}..{max(xs):.2f}]"  # noqa: E731
        verdict = ("CHANGED" if sep else "same") + ("?" if ctrl else "")
        p(f"{name:24} {fmt(a):>28} {fmt(b):>28} {statistics.mean(b) - statistics.mean(a):9.2f}  {verdict:8}  "
          f"{'SEPARATES - this metric is noise here' if ctrl else 'holds'}")
    p("CHANGED = every B run lies outside A's range. A '?' means the control split A from A on that metric,"
      " so its CHANGED is not evidence.")
    return 0


# ---------------------------------------------------------------------------------------------
# selftest: the discrimination must be seen to go red.

def _synthetic(kind, target=400.0):
    sch = load_schema()
    base = {n: (False if t == "bool" else 0) for n, t in sch["fields"]}
    recs = []
    for i in range(40):
        r = dict(base)
        r.update(seq=i, kind=1, frame=100 + i, t_ms=1000 * i, plan_calls=1, present=1, res_w=100, res_h=100, ss=1,
                 last_dt_ms=16.0, repaint_requested=True, fe_budget=int(1.5e11), dispatched=True,
                 nominal_steps=4_000_000_000, frames_since_reading=0)
        recs.append(r)
    ep = recs[10:30]
    for k, r in enumerate(ep):
        r["last_dt_ms"] = 800.0
        if kind == "a":
            r["frames_since_reading"] = 5 + k
        elif kind == "d":
            # Nothing dispatched in the episode; readings of the EARLIER dispatches keep arriving,
            # short, and the frames are slow anyway.
            r["dispatched"] = False
            if k % 3 == 0:
                r.update(read_n=1, read_src=1, read_ms=12.0, read_steps=4_000_000_000, read_verdict=1,
                         read_budget_before=int(1.5e11), read_budget_after=int(1.5e11))
        elif kind in ("b", "c", "e"):
            if k % 2 == 0:
                r.update(read_n=1, read_src=1, read_budget_before=int(1.5e11))
                if kind == "b":
                    r.update(read_ms=12.0, read_steps=123, read_verdict=1, read_budget_after=int(1.5e11))
                elif kind == "c":
                    r.update(read_ms=12.0, read_steps=4_000_000_000, read_verdict=1, read_budget_after=int(1.5e11))
                else:
                    r.update(read_ms=900.0, read_steps=4_000_000_000, read_verdict=3, read_budget_after=int(1.5e11))
        elif kind == "healthy":
            if k % 2 == 0:
                r.update(read_n=1, read_src=1, read_ms=800.0, read_steps=4_000_000_000, read_verdict=2,
                         read_budget_before=int(1.5e11), read_budget_after=int(7e10))
    return recs


# ---------------------------------------------------------------------------------------------
# Dispatch-ceiling calibration (crates/fractadyne-app/src/calibration.rs).

# The GPU's occupancy knee on the RTX 3080 (2026-09-17). Below it a dispatch is not saturated and
# costs more per step, so a reading there is not the SATURATED cost the ceiling is sized from.
CAL_KNEE_PX = 262144
CAL_MODES = {0: "df32_pert", 1: "direct", 2: "floatexp"}


def calibrate(paths, min_steps=5e9, knee=CAL_KNEE_PX, target_ms=400.0, out=sys.stdout):
    """Per-step cost of each arithmetic mode from ALL-INTERIOR zoomtests (the calibration views in
    `validation/calibration/`): every pixel walks every step, so nominal == real and the cost is
    the card's worst for that mode. Only GPU-timed readings of at least `min_steps` (where the
    per-dispatch fixed cost is negligible) on at least `knee` pixels count. Prints the
    `*_ms_per_step` line for `ceilings.toml`, which takes the MAX, not a typical value."""
    sch = load_schema()
    bad = 0
    for path in paths:
        header, recs, _ = load_any(path, sch)
        print(f"{path}\n  adapter {header.get('adapter', '?')} · {header.get('version', '?')} · "
              f"tunables {header.get('tunables', '?')}", file=out)
        if header.get("tunables", "stock") != "stock":
            print("  ⚠NOT STOCK: a calibration from an overridden run does not describe the card", file=out)
        for m in sorted({r["mode"] for r in recs}):
            mode = [r for r in recs if r["mode"] == m]
            esc = [r["ctr_escaped"] / r["ctr_px"] for r in mode if r.get("ctr_new") and r.get("ctr_px", 0) > 0]
            costs = []
            for r in mode:
                if not r.get("read_n") or r.get("read_src") != 1 or r.get("read_steps", 0) < min_steps:
                    continue
                px = (r["tile_w"] * r["tile_h"] if r["tiled"] else r["res_w"] * r["res_h"]) * r["ss"] ** 2
                if px >= knee:
                    costs.append(r["read_ms"] / r["read_steps"])
            name = CAL_MODES.get(m, f"mode{m}")
            if not costs:
                print(f"  mode {m} ({name}): VACUOUS - no GPU reading of >= {min_steps:.0e} steps on "
                      f">= {knee} px", file=out)
                bad += 1
                continue
            worst = max(costs)
            escaped = max(esc) if esc else float("nan")
            print(f"  mode {m} ({name}): {len(costs)} readings, ms/step p50 {statistics.median(costs):.3g} "
                  f"max {worst:.3g}; escaped fraction max {escaped:.3f}", file=out)
            if not escaped <= 0.001:
                print(f"    ⚠NOT ALL-INTERIOR: escaping pixels stop early, so this UNDERSTATES the worst "
                      f"case", file=out)
                bad += 1
            print(f"    {name}_ms_per_step = {worst:.3g}   # ceiling {target_ms / worst:.3g} nominal steps "
                  f"at {target_ms:.0f} ms", file=out)
    return 1 if bad else 0


def selftest():
    target = 400.0
    # Each synthetic episode must be labelled with its own mechanism AND must not be labelled
    # with the ones it is not - a classifier that says "all five" every time is never wrong and
    # never useful.
    want = {
        "a": ("(a)", ("(b)", "(c)", "(d)", "(e)")),
        "b": ("(b)", ("(a)", "(c)", "(d)", "(e)")),
        "c": ("(c)", ("(a)", "(b)", "(d)", "(e)")),
        "d": ("(d)", ("(a)", "(b)", "(c)", "(e)")),
        "e": ("(e)", ("(a)", "(b)", "(c)", "(d)")),
    }
    bad = 0
    for kind, (tag, not_tags) in want.items():
        recs = _synthetic(kind, target)
        eps = episodes(recs, target)
        if len(eps) != 1:
            print(f"selftest: FAIL - synthetic '{kind}' produced {len(eps)} episodes, expected 1")
            bad += 1
            continue
        _, fits = classify(eps[0], {r["frame"]: r for r in recs}, target)
        ok = any(f.startswith(tag) for f in fits) and not any(f.startswith(n) for f in fits for n in not_tags)
        print(f"selftest: {'PASS' if ok else 'FAIL'} - mechanism {tag}: labelled {fits}")
        bad += not ok
    # The control: a controller that saw the slow readings and cut the budget fits none of them.
    recs = _synthetic("healthy", target)
    _, fits = classify(episodes(recs, target)[0], {r["frame"]: r for r in recs}, target)
    ok = fits and fits[0].startswith("none")
    print(f"selftest: {'PASS' if ok else 'FAIL'} - control (healthy reaction): labelled {fits}")
    bad += not ok
    # The schema file must pack to what it says.
    load_schema()
    print("selftest: PASS - schema file consistent" if not bad else f"selftest: {bad} FAILED")
    return 1 if bad else 0


def main(argv):
    if len(argv) < 2 or argv[1] in ("-h", "--help"):
        print(__doc__)
        return 0
    cmd, rest = argv[1], argv[2:]
    if cmd == "selftest":
        return selftest()
    if cmd == "decode":
        sch = load_schema()
        header, recs, torn = read_bin(rest[0], sch)
        out = rest[rest.index("-o") + 1] if "-o" in rest else None
        f = open(out, "w", encoding="utf-8") if out else sys.stdout
        f.write(json.dumps({"kind": "header", "schema": sch["schema"], "header": header}) + "\n")
        for r in recs:
            f.write(json.dumps(r) + "\n")
        if out:
            f.close()
        print(f"{len(recs)} records, {torn} torn", file=sys.stderr)
        return 1 if torn else 0
    if cmd == "schema-check":
        sch = load_schema()
        _, recs, problems = load_any(rest[0], sch)
        for pr in problems[:20]:
            print(f"schema-check: {pr}")
        if not recs:
            print("schema-check: VACUOUS - no records")
            return 2
        print(f"schema-check: {'FAIL' if problems else 'PASS'} - {len(recs)} records, {len(problems)} problem(s)")
        return 1 if problems else 0
    if cmd == "summarize":
        view = int(rest[rest.index("--view") + 1]) if "--view" in rest else 0
        target = float(rest[rest.index("--target-ms") + 1]) if "--target-ms" in rest else 400.0
        return summarize(rest[0], view, target)
    if cmd == "calibrate":
        min_steps = float(rest[rest.index("--min-steps") + 1]) if "--min-steps" in rest else 5e9
        knee = int(rest[rest.index("--knee") + 1]) if "--knee" in rest else CAL_KNEE_PX
        paths = [p for i, p in enumerate(rest)
                 if not p.startswith("--") and (i == 0 or rest[i - 1] not in ("--min-steps", "--knee"))]
        return calibrate(paths, min_steps, knee)
    if cmd == "compare":
        a = rest[rest.index("--a") + 1:rest.index("--b")] if "--a" in rest and "--b" in rest else []
        b = rest[rest.index("--b") + 1:] if "--b" in rest else []
        return compare(a, b)
    print(f"unknown command {cmd!r}\n{__doc__}")
    return 2


if __name__ == "__main__":
    # The session header carries non-ASCII (a tunable override reads "400000000 → 151500000000"),
    # and Windows encodes a redirected stdout as cp1252, where that is a UnicodeEncodeError half-way
    # through the header — every run with an override crashed `summarize` (2026-09-25).
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except (AttributeError, ValueError):
            pass
    sys.exit(main(sys.argv))
