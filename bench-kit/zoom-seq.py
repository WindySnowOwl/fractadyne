#!/usr/bin/env python3
"""zoom-seq.py - the bench kit's ZOOM SEQUENCE lane.

WHY THIS LANE EXISTS
--------------------
Every other scene in this kit is a SINGLE FRAME, and a single frame is blind to the entire
class of optimisation that matters most for zoom video: a dive toward a FIXED CENTRE keeps the
same reference orbit valid across many frames, so the expensive part can be amortised instead
of paid per frame. What is amortisable is exactly the part our own bench-matrix measured as
dominant - at a period-145 nucleus the reference + BLA build costs 60.97 ms against 17.04 ms of
GPU iterate, a 58x setup-to-render ratio - plus the BLA/LA tables and the series-approximation
coefficients derived from it.

This is not hypothetical for Fractadyne: beta.130/132 took a 45-frame deep tour from 25
main-thread reference rebuilds to 0 (18 s -> 14 s wall, output byte-identical). A single-frame
benchmark scores that work as exactly zero.

THE METRIC
----------
    amortisation = (frames x single_frame_wall) / sequence_wall

1.0 means the app rebuilds everything every frame. Higher means it reuses. It is directly
comparable across renderers because it is each app measured against ITSELF - no cross-app
calibration, no shared assumption about what a "frame" costs.

WHY A MISIUREWICZ LADDER IS THE RIGHT PATH
------------------------------------------
The sequence descends by |lambda| per frame at a Misiurewicz point, so every frame is the SAME
PICTURE at a different scale (verified: |c - c_M| / span held 1.205 across 37 rungs). Per-frame
cost therefore SHOULD be flat, and any ramp is the renderer failing to amortise rather than the
scene getting harder. On an arbitrary dive the two are hopelessly confounded.

FAIRNESS NOTE, AND IT IS THE IMPORTANT ONE
------------------------------------------
Fractadyne renders the sequence IN ONE PROCESS via --render-tour, which is the mode that owns
the reference prefetch. Fraktaler-3 3.1's batch CLI (-b) renders ONE image per invocation, so
its sequence is N invocations and its amortisation is ~1.0 BY CONSTRUCTION. That is a real
difference in what the two CLIs offer, not a measurement of F3's engine: F3 has zoom-sequence
and exponential-map/zoomasm machinery that this lane does not reach. Report it as "CLI batch
mode, per-frame" and do NOT quote it as F3's amortisation ceiling.
"""
import argparse, csv, os, re, subprocess, sys, time
from decimal import Decimal, getcontext

getcontext().prec = 400

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)

# M(4,3), Newton-refined and verified; |lambda| = 3.49331342397577021 (0.5432 decades/rung).
M43_RE = ("-0.173006716092090164776138289467703724588887640121379597443935476430201803535625204197715376537058"
          "855405166925649896461064949139318317341128327473274939564013861471073570747748645616285570542524292")
M43_IM = ("1.062752280849242560235197612681136690229320617854029710218025383997229644009518733109036931633002903"
          "184739525797020239071889093827262374176800484266893692470946462136107956050977216146050440014861")
LAM_LOG10 = Decimal("0.543182")   # log10|lambda|


def rung_log10(n):
    return LAM_LOG10 * n


def zoom_str(n, digits=20):
    """The rung's magnification as a PLAIN scientific literal: mantissa e INTEGER-exponent.

    This lane used to write "1.0e%s" % rung_log10(n), i.e. a FRACTIONAL exponent like
    "1.0e23.900008". Fractadyne's tour parser accepts that (it parses the exponent as f64), but
    two other readers did not, and neither said so:

      * `fractadyne --zoom` was a bare f64 parse with `unwrap_or(1.0)`, so the SINGLE-FRAME arm
        - the denominator of the amortisation ratio - rendered the whole Mandelbrot set at 1x
        and exited 0. Fixed in beta.135 (it is now the real parser, and fatal on garbage), but
        the kit must not depend on that: 3.1's Fraktaler-3 is not ours to fix.
      * Fraktaler-3 rejects the value and falls back to the settings PERSISTED in
        %APPDATA%/uk.co.mathr/fraktaler-3/persistence.f3.toml - i.e. it silently renders the
        LAST scene someone rendered, at full cost, and reports success. Here that was a 1e1105
        location: four minutes per "frame" of a benchmark that thought it was at 1e23.9.

    Both failures produce a plausible number and a wrong picture, which is the worst shape a
    benchmark defect can have. An integer exponent is understood by every reader involved.
    """
    l10 = rung_log10(n)
    e = int(l10)                                  # rungs are positive; int() is floor here
    m = Decimal(10) ** (l10 - e)                  # 1 <= m < 10
    return "%se%d" % (round(m, digits).normalize(), e)


def write_tour(path, frames, start_rung, size, iters, fps=1):
    """A tour whose keyframes ARE the ladder rungs, one frame per rung.

    fps=1 with hold=1 per keyframe gives exactly one rendered frame per rung: the tour clock is
    an accumulator, so a paced tour would interpolate BETWEEN rungs and render intermediate
    views that are not on the ladder.
    """
    L = ['# generated by bench-kit/zoom-seq.py - do not edit', 'format_version = 2', '',
         '[render]', 'size = "%s"' % size, 'fps = %d' % fps, '']
    for i in range(frames):
        n = start_rung + i
        L += ['[[keyframe]]',
              'id = "rung%d"' % n,
              't = %d' % i,
              're = "%s"' % M43_RE,
              'im = "%s"' % M43_IM,
              'zoom = "%s"' % zoom_str(n),
              'max_iter = %d' % iters,
              'hold = 1', '']
    with open(path, 'w', newline='\n') as f:
        f.write("\n".join(L))
    return path


def write_f3_frame(path, template, n, size):
    w, h = size.split('x')
    zoom = zoom_str(n)
    out = []
    for line in open(template):
        s = line.rstrip('\n')
        if re.match(r'^\s*width\s*=', s):    s = 'width = %s' % w
        elif re.match(r'^\s*height\s*=', s): s = 'height = %s' % h
        # SAMPLING PARITY: the corpus template carries subframes = 4 (F3's antialiasing sample
        # count, paired with the corpus's --ss 2 on our side). This lane renders one sample per
        # pixel on both arms, and 4x the samples on one of them is not a comparison.
        elif re.match(r'^\s*subframes\s*=', s): s = 'subframes = 1'
        elif re.match(r'^\s*zoom\s*=', s):   s = 'zoom = "%s"' % zoom
        out.append(s)
    with open(path, 'w', newline='\n') as f:
        f.write("\n".join(out) + "\n")
    return path


def f3_out_name(toml_path):
    """The PNG `[render] filename` in a generated F3 config promises to write."""
    for line in open(toml_path):
        m = re.match(r'^\s*filename\s*=\s*"(.+)"\s*$', line)
        if m:
            return m.group(1) + '.png'
    return ''


def timed(cmd, cwd=None, timeout=7200):
    t0 = time.time()
    try:
        p = subprocess.run(cmd, cwd=cwd, timeout=timeout,
                           stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        wall = time.time() - t0
        return ("ok" if p.returncode == 0 else "fail-%d" % p.returncode), wall, p.stdout.decode('utf-8', 'replace')
    except subprocess.TimeoutExpired:
        return "timeout", time.time() - t0, ""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--out', required=True, help='results dir (the run folder)')
    ap.add_argument('--fractadyne')
    ap.add_argument('--fraktaler3')
    ap.add_argument('--f3-wisdom')
    ap.add_argument('--f3-template', default=os.path.join(REPO, 'validation/corpus/locations/21-m43-spar-1e27.7.f3.toml'))
    ap.add_argument('--frames', type=int, default=8)
    ap.add_argument('--start-rung', type=int, default=44)   # ~1e23.9, mid df32
    ap.add_argument('--size', default='3840x2160')
    ap.add_argument('--iters', type=int, default=30000)
    ap.add_argument('--timeout', type=int, default=7200)
    a = ap.parse_args()

    # ABSOLUTE, before anything is built from it. The Fraktaler-3 arm runs with cwd = a.out and
    # used to hand F3 the config as a path relative to the REPO root, which does not resolve from
    # there - so F3 could not open the batch file, fell back to its persisted session, rendered a
    # completely different location at full cost, and exited 0. (generate_fraktaler.py has always
    # passed a BASENAME with a matching cwd; this lane did not.) The frame arm below now does the
    # same, and f3_out_name/the existence check catch it if either ever drifts again.
    a.out = os.path.abspath(a.out)
    os.makedirs(a.out, exist_ok=True)
    csv_path = os.path.join(a.out, 'results.csv')
    new = not os.path.exists(csv_path)
    fh = open(csv_path, 'a', newline='')
    wr = csv.writer(fh)
    if new:
        wr.writerow(['renderer', 'scene', 'rep', 'status', 'wall_s', 'reported_s', 'note'])

    span = "rungs %d..%d (1e%.2f..1e%.2f)" % (
        a.start_rung, a.start_rung + a.frames - 1,
        rung_log10(a.start_rung), rung_log10(a.start_rung + a.frames - 1))
    print("zoom-seq: %d frames, %s, %s, %d iters" % (a.frames, span, a.size, a.iters))
    scene = 'zoomseq-m43-%df' % a.frames
    results = {}

    # ---- Fractadyne: ONE process, the mode that owns the reference prefetch ----
    if a.fractadyne:
        tour = write_tour(os.path.join(a.out, 'zoomseq.toml'), a.frames, a.start_rung, a.size, a.iters)
        frames_dir = os.path.join(a.out, 'zoomseq-fd')
        os.makedirs(frames_dir, exist_ok=True)
        st, wall, _ = timed([a.fractadyne, '--render-tour', tour, '--out', frames_dir,
                             '--size', a.size], timeout=a.timeout)
        wr.writerow(['fractadyne', scene, 1, st, round(wall, 2), '', 'sequence: one process (--render-tour)'])
        print("  fractadyne  %-10s %8.1fs  (sequence)" % (st, wall))
        results['fractadyne'] = (st, wall)

        # the SINGLE-FRAME reference for the ratio: same rung, same settings, fresh process
        one = os.path.join(a.out, 'zoomseq-fd-single.png')
        st1, wall1, _ = timed([a.fractadyne, '--render', '--out', one, '--size', a.size,
                               '--center', M43_RE, M43_IM,
                               '--zoom', zoom_str(a.start_rung),
                               '--iter', str(a.iters), '--ss', '1'], timeout=a.timeout)
        wr.writerow(['fractadyne', scene + '-single', 1, st1, round(wall1, 2), '', 'one frame, fresh process'])
        print("  fractadyne  %-10s %8.1fs  (single frame)" % (st1, wall1))
        # Divide by the frames the tour ACTUALLY wrote, not by --frames. A tour of N keyframes
        # at hold=1 renders N+1 images (the last keyframe is held, not skipped), and dividing a
        # 9-frame sequence by 8 quietly inflates the ratio by 12.5%.
        rendered = len([f for f in os.listdir(frames_dir) if f.endswith('.png')])
        if st == 'ok' and st1 == 'ok' and wall > 0 and rendered:
            print("  -> amortisation %.2fx  (%d frames rendered, %.2fs/frame in sequence)"
                  % (rendered * wall1 / wall, rendered, wall / rendered))

    # ---- Fraktaler-3: CLI batch is one image per invocation (see the module docstring) ----
    if a.fraktaler3 and os.path.exists(a.f3_template):
        t0 = time.time()
        ok = True
        for i in range(a.frames):
            n = a.start_rung + i
            tp = write_f3_frame(os.path.join(a.out, 'zoomseq-f3-%02d.toml' % i), a.f3_template, n, a.size)
            cmd = [a.fraktaler3]
            if a.f3_wisdom:
                cmd += ['-w', a.f3_wisdom]
            cmd += ['-b', os.path.basename(tp)]   # cwd is a.out; see the abspath note above
            st, w, _ = timed(cmd, cwd=a.out, timeout=a.timeout)
            # PROVE it rendered the scene we asked for. F3 falls back to the settings persisted
            # in %APPDATA%/uk.co.mathr/fraktaler-3/persistence.f3.toml when it cannot read the
            # batch file, and then reports success: exit 0, a full-cost render, a PNG named
            # after somebody else's location. Without this check the lane times the wrong scene
            # and calls it a result (2026-08-22: four minutes a frame of a 1e1105 view while
            # the ladder sat at 1e23.9).
            want = os.path.join(a.out, f3_out_name(tp))
            if st == 'ok' and not os.path.exists(want):
                st = 'fail-wrong-scene'
            if st != 'ok':
                ok = False
                print("  fraktaler3  frame %d %s" % (i, st))
                if st == 'fail-wrong-scene':
                    print("    expected %s - F3 fell back to its PERSISTED settings; the batch"
                          % os.path.basename(want))
                    print("    file was rejected. Nothing timed here describes this ladder.")
                break
            if os.path.exists(want):
                os.replace(want, os.path.join(a.out, 'zoomseq-f3-%02d.png' % i))
        wall = time.time() - t0
        wr.writerow(['fraktaler3', scene, 1, 'ok' if ok else 'fail', round(wall, 2), '',
                     'sequence: N invocations (CLI batch renders one image; amortisation ~1.0 BY CONSTRUCTION, not an engine limit)'])
        print("  fraktaler3  %-10s %8.1fs  (%d invocations)" % ('ok' if ok else 'fail', wall, a.frames))
        results['fraktaler3'] = ('ok' if ok else 'fail', wall)

    fh.close()
    print("\nresults -> %s" % csv_path)
    print("NOTE: F3's figure is N separate processes. Its CLI has no sequence mode, so its\n"
          "      amortisation is 1.0 by construction - do NOT read it as F3's engine ceiling.")
    return 0


if __name__ == '__main__':
    sys.exit(main())
