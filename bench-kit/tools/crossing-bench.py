"""Cross-renderer benchmark of the arithmetic-cutoff CROSSING tours (tours/crossing/).

For each crossing tour it drives every available renderer through the SAME per-frame views (the tour's
keyframes: fixed hard center, zoom sweeping the band across the cutoff), at one sample per pixel, and
reports per-tour timing plus a self-amortisation figure. Every frame is structure-checked, so a blank
render is DNF-blank, never a time (the "144x on an empty frame" trap the kit exists to avoid).

Lanes:
  * fractadyne  -- SEQUENCE via --render-tour (one process, owns the cross-frame reference reuse),
                   plus N single --render frames as the amortisation denominator.
  * fraktaler3  -- one --f3.toml per frame (its batch CLI renders one image per call, so its
                   amortisation is 1.0 BY CONSTRUCTION -- a CLI property, not an engine limit).
  * fractalshark-- one FractalSharkCli GPU render per frame (needs 0.541+ on RTX 20/30/40/50).

All three receive the identical zoom string per frame, so this is a like-for-like sequence.

Usage:
  python bench-kit/tools/crossing-bench.py --fractadyne target/release/fractadyne.exe \
      [--fraktaler3 PATH] [--f3-wisdom PATH] [--fractalshark-cli PATH] \
      [--tours tours/crossing] [--size 3840x2160] [--out DIR] [--timeout 7200]
"""
import argparse
import csv
import os
import re
import subprocess
import time

from PIL import Image

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
F3_BASE = os.path.join(REPO, "validation", "corpus", "locations", "21-m43-spar-1e27.7.f3.toml")


def parse_tour(path):
    """Return (size, [(re, im, zoom, iters), ...]) from a generated crossing tour."""
    size = "3840x2160"
    frames, cur = [], {}
    for line in open(path):
        s = line.strip()
        m = re.match(r'size\s*=\s*"([^"]+)"', s)
        if m:
            size = m.group(1)
        if s == "[[keyframe]]":
            if cur:
                frames.append(cur)
            cur = {}
        for key in ("re", "im", "zoom"):
            m = re.match(key + r'\s*=\s*"([^"]+)"', s)
            if m:
                cur[key] = m.group(1)
        m = re.match(r'max_iter\s*=\s*(\d+)', s)
        if m:
            cur["iters"] = int(m.group(1))
    if cur:
        frames.append(cur)
    return size, [(f["re"], f["im"], f["zoom"], f["iters"]) for f in frames if "zoom" in f]


def has_structure(png):
    try:
        im = Image.open(png).convert("RGB")
    except Exception:
        return False
    px = list(im.getdata())
    from collections import Counter
    c = Counter(px)
    modal = c.most_common(1)[0][1] / len(px)
    return len(c) > 16 and modal < 0.98


def timed(cmd, cwd=None, timeout=7200, env=None):
    t0 = time.time()
    try:
        p = subprocess.run(cmd, cwd=cwd, timeout=timeout, env=env,
                           stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        return ("ok" if p.returncode == 0 else "fail-%d" % p.returncode), time.time() - t0
    except subprocess.TimeoutExpired:
        return "timeout", time.time() - t0


def write_f3(path, re_, im_, zoom, size, iters):
    w, h = size.split("x")
    out = []
    for line in open(F3_BASE):
        s = line.rstrip("\n")
        if re.match(r'^\s*real\s*=', s):    s = 'real = "%s"' % re_
        elif re.match(r'^\s*imag\s*=', s):  s = 'imag = "%s"' % im_
        elif re.match(r'^\s*zoom\s*=', s):  s = 'zoom = "%s"' % zoom
        elif re.match(r'^\s*width\s*=', s): s = 'width = %s' % w
        elif re.match(r'^\s*height\s*=', s): s = 'height = %s' % h
        elif re.match(r'^\s*subframes\s*=', s): s = 'subframes = 1'
        elif re.match(r'^\s*(iterations|maximum_reference_iterations|maximum_perturb_iterations)\s*=', s):
            s = re.sub(r'=\s*\d+', '= %d' % iters, s)
        elif re.match(r'^\s*filename\s*=', s): s = 'filename = "f3frame"'
        out.append(s)
    with open(path, "w", newline="\n") as f:
        f.write("\n".join(out) + "\n")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--fractadyne")
    ap.add_argument("--fraktaler3")
    ap.add_argument("--f3-wisdom")
    ap.add_argument("--fractalshark-cli")
    ap.add_argument("--fs-algo", default="GpuHDRx32PerturbedLAv2")
    ap.add_argument("--tours", default=os.path.join(REPO, "tours", "crossing"))
    ap.add_argument("--size", default="3840x2160")
    ap.add_argument("--out", default=None)
    ap.add_argument("--timeout", type=int, default=7200)
    ap.add_argument("--only", help="comma list of tour stems to run (default: all)")
    a = ap.parse_args()

    # Absolute paths: the Fraktaler-3 lane runs with cwd = the work dir, so a relative exe or wisdom
    # path would not resolve there (WinError 2).
    for attr in ("fractadyne", "fraktaler3", "f3_wisdom", "fractalshark_cli"):
        v = getattr(a, attr)
        if v:
            setattr(a, attr, os.path.abspath(v))

    stamp = time.strftime("%Y%m%d-%H%M%S")
    out = a.out or os.path.join(REPO, "bench-kit", "results", "crossing-%s-%s"
                                % (os.environ.get("COMPUTERNAME", "host"), stamp))
    out = os.path.abspath(out)
    os.makedirs(out, exist_ok=True)
    csv_fh = open(os.path.join(out, "results.csv"), "w", newline="")
    wr = csv.writer(csv_fh)
    wr.writerow(["renderer", "tour", "status", "wall_s", "amortisation", "frames", "note"])

    tours = sorted(f for f in os.listdir(a.tours) if f.endswith(".toml"))
    if a.only:
        keep = set(a.only.split(","))
        tours = [t for t in tours if os.path.splitext(t)[0] in keep]

    for tf in tours:
        stem = os.path.splitext(tf)[0]
        tour_path = os.path.join(a.tours, tf)
        size, frames = parse_tour(tour_path)
        size = a.size or size
        n = len(frames)
        print("\n### %s  (%d frames, %s)" % (stem, n, size))
        wdir = os.path.join(out, stem)
        os.makedirs(wdir, exist_ok=True)

        # ---- fractadyne: sequence + singles ----
        if a.fractadyne:
            env = dict(os.environ); env["FRACTADYNE_NO_SOUND"] = "1"
            env["FRACTADYNE_CONFIG_DIR"] = os.path.join(wdir, "fd-cfg")
            fdir = os.path.join(wdir, "fd-seq")
            st, seq = timed([a.fractadyne, "--render-tour", tour_path, "--out", fdir, "--size", size],
                            timeout=a.timeout, env=env)
            pngs = [os.path.join(fdir, f) for f in os.listdir(fdir)] if os.path.isdir(fdir) else []
            pngs = [p for p in pngs if p.endswith(".png")]
            blank = sum(1 for p in pngs if not has_structure(p))
            if blank:
                st = "DNF-blank(%d)" % blank
            singles = []
            for i, (r, im_, z, it) in enumerate(frames):
                one = os.path.join(wdir, "fd-single-%03d.png" % i)
                s1, w1 = timed([a.fractadyne, "--render", "--out", one, "--size", size,
                                "--center", r, im_, "--zoom", z, "--iter", str(it), "--ss", "1"],
                               timeout=a.timeout, env=env)
                if s1 != "ok":
                    break
                singles.append(w1)
            ss = sum(singles)
            amort = round(ss / seq, 2) if seq > 0 and singles else ""
            wr.writerow(["fractadyne", stem, st, round(seq, 2), amort, len(pngs),
                         "sequence --render-tour; %d singles sum=%.1fs" % (len(singles), ss)])
            print("  fractadyne   %-14s seq=%.1fs  singles=%.1fs  amort=%s" % (st, seq, ss, amort))

        # ---- Fraktaler-3: one process per frame ----
        if a.fraktaler3:
            tot, ok, blank = 0.0, True, 0
            for i, (r, im_, z, it) in enumerate(frames):
                cfg = os.path.join(wdir, "f3-%03d.toml" % i)
                write_f3(cfg, r, im_, z, size, it)
                cmd = [a.fraktaler3, "-b", os.path.basename(cfg)]
                if a.f3_wisdom:
                    cmd = [a.fraktaler3, "-w", a.f3_wisdom, "-b", os.path.basename(cfg)]
                st1, w1 = timed(cmd, cwd=wdir, timeout=a.timeout)
                png = os.path.join(wdir, "f3frame.png")
                if st1 != "ok" or not os.path.exists(png):
                    ok = False; break
                if not has_structure(png):
                    blank += 1
                os.replace(png, os.path.join(wdir, "f3-%03d.png" % i))
                tot += w1
            status = ("ok" if ok else "fail") if not blank else "DNF-blank(%d)" % blank
            wr.writerow(["fraktaler3", stem, status, round(tot, 2), "1.0 (N processes)", n,
                         "one --f3.toml per frame"])
            print("  fraktaler3   %-14s %.1fs  (N processes, amort 1.0 by construction)" % (status, tot))

        # ---- FractalShark: GPU, one process per frame ----
        if a.fractalshark_cli:
            tot, ok, blank, done = 0.0, True, 0, 0
            for i, (r, im_, z, it) in enumerate(frames):
                w, h = size.split("x")
                stem_out = os.path.join(wdir, "fs-%03d" % i)
                cmd = [a.fractalshark_cli, "--render-algorithm", a.fs_algo,
                       "--center-x", r, "--center-y", im_, "--zoom", z, "--iterations", str(it),
                       "--width", w, "--height", h, "--antialiasing", "1", "--out", stem_out, "--quiet"]
                st1, w1 = timed(cmd, timeout=a.timeout)
                png = stem_out + ".png"
                if not os.path.exists(png):
                    ok = False; break
                if not has_structure(png):
                    blank += 1
                tot += w1; done += 1
            status = ("ok" if ok else "fail")
            if blank == done and done:
                status = "DNF-blank(all)"
            elif blank:
                status = "partial-blank(%d/%d)" % (blank, done)
            wr.writerow(["fractalshark", stem, status, round(tot, 2), "1.0 (N processes)", done,
                         "GPU %s; 0.541+ required" % a.fs_algo])
            print("  fractalshark %-14s %.1fs  (%d/%d structured)" % (status, tot, done - blank, done))

    csv_fh.close()
    print("\nresults -> %s" % os.path.join(out, "results.csv"))


if __name__ == "__main__":
    main()
