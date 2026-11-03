#!/usr/bin/env python3
"""The minibrot-offset set: a minibrot at different places in (and just out of) the view.

    python scripts/minibrot_offsets.py views [--size WxH]
    python scripts/minibrot_offsets.py run --exe FRACTADYNE.EXE --out DIR [--rounds 2] [--size WxH]
                                       [--only SUBSTR] [--audit renorm,band,pick] [--samples 12]
    python scripts/minibrot_offsets.py compare DIR_A DIR_B

WHY
---
An optimization keyed on a minibrot (a periodic reference, the renormalized step, a nucleus
found by Newton) behaves differently depending on WHERE the minibrot sits: dead centre, off
centre, at the edge, just outside, far outside; filling the view or a speck in a crowd. Every
deep scene the bench kit had put its minibrot dead centre, and the one that did not (the deeper
Cerebral Spin, 2026-11-03) cost Fractadyne 6.8 s against 2.8 s for the same view centred on the
nucleus. This set sweeps the position on purpose, so a change that only works at the centre
shows up here, as a wrong pixel or a slow view, before it ships.

THE SET (validation/minibrot-offsets/nuclei.csv)
-----------------------------------------------
Four nuclei (periods 128, 2,066, 104,572 and 14,416 at about 1e13, 1e97, 1e153 and 1e1000),
each at three sizes - BIG (the minibrot a third of the view height), SMALL (a thirtieth) and
SPECK (a thousandth, about a pixel at 1080, among many other minibrots) - with the nucleus at
the centre and at 0.2, 0.6, 0.95, 1.2 and 3.0 of the half-view towards the right edge (x) and
towards the top-right corner (d): 11 positions, so 132 views, plus each nucleus centred and BIG
at an iteration cap of half its period (the reference cannot repeat inside the cap): 136. The
cap is otherwise 30 x the period, the bench ladder's rule.

JUDGING
-------
`run` times every view (fastest of `--rounds`, a fresh process and config folder each) and
records what its log says about the reference: the pick's time, whether the orbit escaped,
repeats (periodic) or neither, the renormalized step, and the GPU time. `--audit` adds the app's
own oracle audits per view (`--renorm-audit`, `--band-audit`, and `--pick-audit` where the build
has it): each renders the view two ways and checks the pixels that differ, and controls that do
not, against the arbitrary-precision CPU oracle. A view where an audit is VACUOUS (nothing to
compare) says so. `compare` decodes both runs' images view by view: pixels that differ, and the
time ratio. A view the change should not touch must come out identical.
"""
import argparse
import csv
import math
import os
import re
import shutil
import subprocess
import sys
import time
from decimal import Decimal, getcontext

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
NUCLEI = os.path.join(REPO, 'validation', 'minibrot-offsets', 'nuclei.csv')
SIZES = [('big', 0.0), ('small', -1.0), ('speck', -math.log10(1000 / 3))]
FRACTIONS = [0.2, 0.6, 0.95, 1.2, 3.0]
AUDIT_FLAGS = {'renorm': '--renorm-audit', 'band': '--band-audit', 'pick': '--pick-audit'}


def views(size):
    """Every view of the set, in order: dicts with id, nucleus, period, centre, zoom, cap."""
    w, h = size
    out = []
    with open(NUCLEI, encoding='ascii', newline='') as f:
        for n in csv.DictReader(f):
            getcontext().prec = max(len(n['re']), len(n['im'])) + 40
            nre, nim, period = Decimal(n['re']), Decimal(n['im']), int(n['period'])
            for sname, dmag in SIZES:
                mag = float(n['big_mag_log10']) + dmag
                half_h = 2 * Decimal(10) ** (-Decimal(repr(mag)))  # view height is 4 / zoom
                half_w = half_h * w / h
                pos = [('c', 0.0, 0.0)] + [('x%g' % fr, fr, 0.0) for fr in FRACTIONS] + [('d%g' % fr, fr, fr) for fr in FRACTIONS]
                for pname, fx, fy in pos:
                    # The view centre sits so the nucleus appears fx half-widths right of it and fy
                    # half-heights up.
                    cre = nre - Decimal(repr(fx)) * half_w
                    cim = nim - Decimal(repr(fy)) * half_h
                    caps = [30 * period] + ([period // 2] if (sname, pname) == ('big', 'c') else [])
                    for cap in caps:
                        vid = '%s-%s-%s' % (n['name'], sname, pname) + ('' if cap == 30 * period else '-cap%d' % cap)
                        out.append({'id': vid, 'nucleus': n['name'], 'period': period, 'size': sname,
                                    'pos': pname, 'mag_log10': mag, 'cap': cap,
                                    're': fmt(cre), 'im': fmt(cim)})
    return out


def fmt(d):
    return str(d).replace('E', 'e')


def parse_log(text):
    """What a render's log says about its reference and its GPU time."""
    g = lambda pat, conv=str, default='': (lambda m: conv(m.group(1)) if m else default)(re.search(pat, text))
    ref = re.search(r'\[export\] len=(\d+) iter=\d+ prec=(\d+) partial=(\w+) escaped=(\w+)(?: periodic=(\d+))?', text)
    return {
        'pick_ms': g(r'pick_reference \(candidate scoring\) took (\d+)ms', int, 0),
        'pick_offset': g(r'pick \[export\]:[^\n]*offset=\(([^)]*)\)'),
        'pick_rescued': g(r'pick \[export\]:[^\n]*RESCUED=(\w+)'),
        'ref_len': int(ref.group(1)) if ref else '',
        'ref_prec': int(ref.group(2)) if ref else '',
        'partial': ref.group(3) if ref else '',
        'escaped': ref.group(4) if ref else '',
        'periodic': ref.group(5) or '' if ref else '',
        'renorm': g(r'\[export\] len=[^\n]* renorm=(\d+)'),
        'orbit_ms': g(r'\[export\] len=[^\n]*orbit_ms=(\d+)', int, ''),
        'ref_wait_ms': g(r'which then waited (\d+) ms', int, 0),
        'mode': g(r'cli-render: \S+ ss=\d+ mode=(\d+)', int, ''),
        'gpu_ms': g(r'gpu_iterate=([\d.]+)ms', float, ''),
        'maxiter_px': g(r'maxiter=(\d+)', int, ''),
        'renorm_px': g(r'renormalized px=(\d+)', int, ''),
        'failed': g(r'\[render\] FAILED: ([^\n]*)'),
    }


def fresh_env(cfg):
    if os.path.exists(cfg):
        shutil.rmtree(cfg)
    os.makedirs(cfg)
    return dict(os.environ, FRACTADYNE_CONFIG_DIR=cfg, FRACTADYNE_NO_SOUND='1', FRACTADYNE_TRACE='ref')


def view_args(v, size):
    zl2 = v['mag_log10'] * math.log2(10)
    return ['--size', '%dx%d' % size, '--center', v['re'], v['im'], '--zoom-log2', repr(zl2),
            '--iter', str(v['cap']), '--ss', '1']


def run(a):
    size = tuple(int(x) for x in a.size.split('x'))
    out = os.path.abspath(a.out)
    for d in ('png', 'logs', 'audits'):
        os.makedirs(os.path.join(out, d), exist_ok=True)
    cfg = os.path.join(out, 'cfg')
    vs = [v for v in views(size) if not a.only or a.only in v['id']]
    ver = subprocess.run([a.exe, '--version'], capture_output=True, text=True).stdout.strip()
    with open(os.path.join(out, 'run.txt'), 'w', encoding='utf-8') as f:
        f.write('exe: %s\nversion: %s\nsize: %dx%d\nrounds: %d\nviews: %d\nstarted: %s\n'
                % (os.path.basename(a.exe), ver, size[0], size[1], a.rounds, len(vs), time.strftime('%Y-%m-%d %H:%M:%S')))
    res_path = os.path.join(out, 'results.csv')
    cols = ['id', 'nucleus', 'size', 'pos', 'cap', 'round', 'exit', 'wall_s'] + list(parse_log('').keys())
    with open(res_path, 'w', encoding='utf-8', newline='') as f:
        csv.writer(f).writerow(cols)
    for i, v in enumerate(vs):
        for rnd in range(1, a.rounds + 1):
            png = os.path.join(out, 'png', v['id'] + '.png')
            env = fresh_env(cfg)
            t0 = time.perf_counter()
            try:
                p = subprocess.run([a.exe, '--render', '--out', png] + view_args(v, size) + ['--palette', '0'],
                                   env=env, capture_output=True, text=True, timeout=a.timeout)
                code = p.returncode
            except subprocess.TimeoutExpired:
                code = 'timeout'
            wall = time.perf_counter() - t0
            log = os.path.join(cfg, 'logs', 'fractadyne.log')
            text = open(log, encoding='utf-8', errors='replace').read() if os.path.exists(log) else ''
            shutil.copy(log, os.path.join(out, 'logs', '%s-r%d.log' % (v['id'], rnd))) if text else None
            info = parse_log(text)
            row = [v['id'], v['nucleus'], v['size'], v['pos'], v['cap'], rnd, code, '%.2f' % wall] + [info[k] for k in parse_log('')]
            with open(res_path, 'a', encoding='utf-8', newline='') as f:
                csv.writer(f).writerow(row)
            print('[%3d/%d] %-28s r%d exit=%s %6.2fs pick=%sms wait=%sms ref=%s/%s%s gpu=%sms' % (
                i + 1, len(vs), v['id'], rnd, code, wall, info['pick_ms'], info['ref_wait_ms'], info['ref_len'],
                'periodic' if info['periodic'] else ('escaped' if info['escaped'] == 'true' else 'partial'),
                ' renorm' if info['renorm'] else '', info['gpu_ms']), flush=True)
    if a.audit:
        aud_path = os.path.join(out, 'audits.csv')
        with open(aud_path, 'w', encoding='utf-8', newline='') as f:
            csv.writer(f).writerow(['id', 'kind', 'exit', 'wall_s', 'controls', 'summary'])
        for i, v in enumerate(vs):
            for kind in a.audit.split(','):
                if kind == 'band' and v['mag_log10'] > 28:
                    continue  # the df32 band ends at 1e28
                env = fresh_env(cfg)
                t0 = time.perf_counter()
                try:
                    p = subprocess.run([a.exe, AUDIT_FLAGS[kind], str(a.samples)] + view_args(v, size),
                                       env=env, capture_output=True, text=True, timeout=a.timeout * 4)
                    code, text = p.returncode, p.stdout + p.stderr
                except subprocess.TimeoutExpired:
                    code, text = 'timeout', ''
                wall = time.perf_counter() - t0
                with open(os.path.join(out, 'audits', '%s-%s.txt' % (v['id'], kind)), 'w', encoding='utf-8') as f:
                    f.write(text)
                summ = ([l.strip() for l in text.splitlines() if l.strip().startswith('audit:')] or [text.strip().splitlines()[-1] if text.strip() else ''])[-1]
                ctl = ([l.strip() for l in text.splitlines() if l.strip().startswith('controls:')] or [''])[-1]
                with open(aud_path, 'a', encoding='utf-8', newline='') as f:
                    csv.writer(f).writerow([v['id'], kind, code, '%.1f' % wall, ctl, summ])
                print('[%3d/%d] %-28s %-6s exit=%s %6.1fs %s' % (i + 1, len(vs), v['id'], kind, code, wall, summ[:150]), flush=True)
    with open(os.path.join(out, 'run.txt'), 'a', encoding='utf-8') as f:
        f.write('finished: %s\n' % time.strftime('%Y-%m-%d %H:%M:%S'))


def fastest(path):
    best = {}
    with open(path, encoding='utf-8', newline='') as f:
        for r in csv.DictReader(f):
            if str(r['exit']) != '0':
                best.setdefault(r['id'], r)
                continue
            if r['id'] not in best or str(best[r['id']]['exit']) != '0' or float(r['wall_s']) < float(best[r['id']]['wall_s']):
                best[r['id']] = r
    return best


def compare(a):
    import numpy as np
    from PIL import Image
    A, B = fastest(os.path.join(a.a, 'results.csv')), fastest(os.path.join(a.b, 'results.csv'))
    print('%-28s %8s %8s %6s %10s  %s' % ('view', 'A s', 'B s', 'B/A', 'diff px', 'reference A -> B'))
    for vid in A:
        if vid not in B:
            continue
        pa, pb = (os.path.join(d, 'png', vid + '.png') for d in (a.a, a.b))
        if os.path.exists(pa) and os.path.exists(pb):
            x = np.asarray(Image.open(pa).convert('RGB')).astype(np.int16)
            y = np.asarray(Image.open(pb).convert('RGB')).astype(np.int16)
            nd = int((np.abs(x - y).sum(axis=2) > 0).sum()) if x.shape == y.shape else -1
        else:
            nd = -1
        ra, rb = A[vid], B[vid]
        kind = lambda r: ('periodic' if r['periodic'] else ('escaped' if r['escaped'] == 'true' else 'partial')) + ('+rn' if r['renorm'] else '')
        ta, tb = float(ra['wall_s']), float(rb['wall_s'])
        print('%-28s %8.2f %8.2f %6.2f %10d  %s -> %s' % (vid, ta, tb, tb / ta if ta else 0, nd, kind(ra), kind(rb)))


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = ap.add_subparsers(dest='cmd', required=True)
    s = sub.add_parser('views')
    s.add_argument('--size', default='1920x1080')
    s = sub.add_parser('run')
    s.add_argument('--exe', required=True)
    s.add_argument('--out', required=True)
    s.add_argument('--rounds', type=int, default=2)
    s.add_argument('--size', default='1920x1080')
    s.add_argument('--only', default='')
    s.add_argument('--audit', default='')
    s.add_argument('--samples', type=int, default=12)
    s.add_argument('--timeout', type=int, default=900)
    s = sub.add_parser('compare')
    s.add_argument('a')
    s.add_argument('b')
    a = ap.parse_args()
    if a.cmd == 'views':
        for v in views(tuple(int(x) for x in a.size.split('x'))):
            print('%-28s mag 1e%.3f cap %d' % (v['id'], v['mag_log10'], v['cap']))
    elif a.cmd == 'run':
        run(a)
    else:
        compare(a)


if __name__ == '__main__':
    sys.exit(main())
