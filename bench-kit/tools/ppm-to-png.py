#!/usr/bin/env python3
"""Convert a renderer's PPM output to PNG.

    tools/ppm-to-png.py IN.ppm OUT.png

imagina-cli writes binary PPM, which is a perfectly good image and useless to everything
downstream here: the lane's structure guard reads images through System.Drawing, which has no PPM
decoder, and no browser will display one in the report. Converting once at the point of capture
keeps every later stage uniform rather than teaching each of them a second format.

Exit 0 on success, 1 if the input cannot be read, 2 on a usage error. Never raises: a conversion
failure has to degrade into a DNF row, not take the run down.
"""
import os
import sys


def main():
    if len(sys.argv) != 3:
        sys.stderr.write("usage: ppm-to-png.py IN.ppm OUT.png\n")
        return 2
    src, dst = sys.argv[1], sys.argv[2]
    try:
        from PIL import Image
    except ImportError:
        sys.stderr.write("ppm-to-png: needs Pillow (pip install pillow)\n")
        return 1
    try:
        if not os.path.isfile(src):
            sys.stderr.write("ppm-to-png: no such file: %s\n" % src)
            return 1
        with Image.open(src) as im:
            im.convert("RGB").save(dst, "PNG")
    except Exception as e:
        sys.stderr.write("ppm-to-png: %s\n" % e)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
