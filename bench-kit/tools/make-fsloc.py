"""Write a FractalShark --locations file for one scene, from its .kfr, at full precision.

  python make-fsloc.py SCENE.kfr OUT.fsloc --width W --height H --iterations N [--name NAME]

The benchmark passes FractalShark a view on its command line (--center-x/--center-y/--zoom) until
the centre is too long for one: a 1e30000x centre is 30,000 digits per coordinate and a Windows
command line holds 32,767 characters. FractalSharkCli also reads a locations file, which this
writes. The view is a box of centre +/- 2/zoom in each axis, zoom being the .kfr's own Zoom (the
Kalles Fraktaler convention every lane shares: zoom 1 is a 4-unit-high view). The arithmetic is
decimal at the centre's own precision plus 50 digits, never float, which could not even represent
the zoom.
"""
import argparse
from decimal import Decimal, getcontext


def read_kfr(path):
    kv = {}
    with open(path, encoding='ascii') as f:
        for line in f:
            if ':' in line:
                k, v = line.split(':', 1)
                kv[k.strip()] = v.strip()
    return kv


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    ap.add_argument('kfr')
    ap.add_argument('out')
    ap.add_argument('--width', type=int, required=True)
    ap.add_argument('--height', type=int, required=True)
    ap.add_argument('--iterations', type=int, required=True)
    ap.add_argument('--name', default='view')
    a = ap.parse_args()
    kv = read_kfr(a.kfr)
    re_s, im_s, zoom_s = kv['Re'], kv['Im'], kv['Zoom']
    getcontext().prec = max(len(re_s), len(im_s)) + 50
    cx, cy = Decimal(re_s), Decimal(im_s)
    half = Decimal(2) / Decimal(zoom_s)
    with open(a.out, 'w', encoding='ascii', newline='\n') as f:
        f.write('%d %d\n%s\n%s\n%s\n%s\n%d 1\n%s\n' % (
            a.width, a.height, cx - half, cy - half, cx + half, cy + half, a.iterations, a.name))


if __name__ == '__main__':
    main()
