#!/usr/bin/env python3
"""Build the Windows icon that is compiled INTO fractadyne.exe, from the same render as the window icon.

    python scripts/make_windows_icon.py            # writes the three files below
    python scripts/make_windows_icon.py --check    # exit 1 if they are not what this would write

Windows takes a program's icon (Explorer, a shortcut's Change Icon, the Start menu, a taskbar pin
before the program runs) from an icon RESOURCE inside the .exe. The app sets its window icon itself
at run time (`theme::brand_icon`, from `assets/icon.png`), which Explorer never sees, so until this
existed every build reported "contains no icons".

Writes, in crates/fractadyne-app/assets/:
  fractadyne.ico        the icon, 16-256 px, cropped exactly as `theme::icon_from_render` crops
  fractadyne-icon.res   it as a compiled resource, for the MSVC linker (the standard build)
  fractadyne-icon.o     the same as a COFF object, for the GNU linker (the accelerated build)

build.rs links the .res or the .o, so a build needs neither a resource compiler nor a crate for it.
The two compiled files come from MSYS2's windres (C:\\msys64\\mingw64\\bin), which only this script
needs. Run it after changing assets/icon.png, and commit what it writes.
"""
import argparse
import os
import subprocess
import sys
import tempfile

from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ASSETS = os.path.join(ROOT, "crates", "fractadyne-app", "assets")
WINDRES = os.environ.get("WINDRES", r"C:\msys64\mingw64\bin\windres.exe")
SIZES = [16, 20, 24, 32, 40, 48, 64, 96, 128, 256]


def base_icon():
    """assets/icon.png cropped and box-averaged to 256x256 as `theme::icon_from_render` does."""
    src = Image.open(os.path.join(ASSETS, "icon.png")).convert("RGBA")
    w, h = src.size
    px = src.load()
    out_n = 256
    mx, my = max(w * 3 // 100, 2), max(h * 3 // 100, 2)
    iw, ih = w - 2 * mx, h - 2 * my
    side = min(iw, ih)
    cx0, cy0 = mx + (iw - side) // 2, my + (ih - side) // 2
    out = Image.new("RGBA", (out_n, out_n))
    op = out.load()
    for oy in range(out_n):
        for ox in range(out_n):
            sx0 = cx0 + ox * side // out_n
            sx1 = min(max(cx0 + (ox + 1) * side // out_n, sx0 + 1), w)
            sy0 = cy0 + oy * side // out_n
            sy1 = min(max(cy0 + (oy + 1) * side // out_n, sy0 + 1), h)
            r = g = b = n = 0
            for sy in range(sy0, sy1):
                for sx in range(sx0, sx1):
                    p = px[sx, sy]
                    r, g, b, n = r + p[0], g + p[1], b + p[2], n + 1
            n = max(n, 1)
            op[ox, oy] = (r // n, g // n, b // n, 255)
    return out


def build(dest):
    ico = os.path.join(dest, "fractadyne.ico")
    base_icon().save(ico, format="ICO", sizes=[(s, s) for s in SIZES])
    rc = os.path.join(dest, "fractadyne-icon.rc")
    with open(rc, "w", newline="\n") as f:
        # Resource 1: Explorer shows a program's first icon group.
        f.write('1 ICON "fractadyne.ico"\n')
    for fmt, name, extra in (("res", "fractadyne-icon.res", []), ("coff", "fractadyne-icon.o", ["-F", "pe-x86-64"])):
        # windres preprocesses the .rc with MinGW's gcc, which must be on PATH with its DLLs.
        env = dict(os.environ, PATH=os.path.dirname(WINDRES) + os.pathsep + os.environ.get("PATH", ""))
        subprocess.run([WINDRES, *extra, "-i", rc, "-O", fmt, "-o", os.path.join(dest, name)], check=True, cwd=dest, env=env)
    os.remove(rc)
    return ["fractadyne.ico", "fractadyne-icon.res", "fractadyne-icon.o"]


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--check", action="store_true", help="compare with the committed files instead of writing")
    a = ap.parse_args()
    if not a.check:
        for n in build(ASSETS):
            print("wrote", os.path.join("crates", "fractadyne-app", "assets", n))
        return 0
    with tempfile.TemporaryDirectory() as t:
        names = build(t)
        stale = [n for n in names if open(os.path.join(t, n), "rb").read() != open(os.path.join(ASSETS, n), "rb").read()]
    if stale:
        print("stale:", ", ".join(stale), "- run scripts/make_windows_icon.py and commit the result")
        return 1
    print("make_windows_icon: up to date")
    return 0


if __name__ == "__main__":
    sys.exit(main())
