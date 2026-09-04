## Which download?

| File | For |
|---|---|
| `fractadyne-<version>-windows-x64.zip` | **Windows.** Start here if you are unsure. |
| `fractadyne-<version>-linux-x64.tar.gz` | **Linux.** Expect it to be less tested than the Windows build, because it is. |
| `fractadyne-<version>-windows-x64-accelerated.zip` | Windows, **optional** — faster at depth. See below. |
| `fractadyne-<version>-linux-x64-accelerated.tar.gz` | Linux, **optional** — faster at depth. See below. |

Each has a `.sha256` beside it. Nothing is installed: unpack anywhere, run `fractadyne.exe`
(or `./fractadyne` on Linux), and delete the folder to remove it. You need 64-bit Windows 10/11 or a
current Linux, and a GPU with Vulkan, DirectX 12 or OpenGL.

## What the `-accelerated` downloads are

The **same** Fractadyne as the standard download for your platform, with one difference: it computes
deep-zoom **reference orbits** using MPFR/GMP instead of the pure-Rust library the standard build
uses. That is the CPU pause before a deep view starts resolving, and it is roughly **2.5× to 6.4×
faster** there — more so the deeper you go.

It is not a general speed-up: it does not raise the frame rate, and it does not change the picture.

- **The images are byte-identical.** Verified, not assumed: the same reference orbits across every
  fractal formula, at arithmetic widths from 64 bits to 132,000 bits, plus the full 38-location
  deep-zoom comparison corpus. A difference between the two builds would be a bug worth reporting.
- **Your settings, session and saved locations are shared.** They live in your user profile, not
  beside the executable, so you can switch between the two builds freely and nothing needs
  importing.
- To see which arithmetic you are running: **Help → About**, the "Deep-zoom arithmetic" line.

**Windows:** keep the `.dll` files next to `fractadyne.exe` — the program will not start without
them. GMP and MPFR ship inside the zip.

**Linux:** the tarball does *not* bundle GMP and MPFR; it links the ones on your system
(`libgmp10`, `libmpfr6`), which nearly every desktop already has — `sudo apt-get install libgmp10
libmpfr6` if yours does not. ⚠It also needs a **newer distribution than the standard Linux
download**: glibc 2.39+ (Ubuntu 24.04+, Debian 13+, Fedora 40+) against the standard build's 2.35+,
because the MPFR bindings require GMP 6.3.0 and Ubuntu 22.04 ships 6.2.1. If it will not start with
a glibc message, take the standard `linux-x64.tar.gz` — same program, just slower at reference
orbits.

**Why it is a separate download** — the reason that applies everywhere, plus one that is
Windows-only:

1. MPFR and GMP are licensed under the **GNU LGPL v3**, while Fractadyne itself is MIT OR
   Apache-2.0. Keeping them in a separate, clearly labelled download keeps the standard build free
   of those terms. The full notices, and the licence texts, are inside the package.
2. On Windows only: MPFR cannot be built with the Microsoft compiler the standard Windows binary
   uses, so that build comes from a different toolchain. (The standard Linux build already uses the
   same compiler as its accelerated sibling.)

If you don't need it, take the plain `windows-x64.zip` or `linux-x64.tar.gz`. The app will offer you
the accelerated build later from **Help → Faster deep zoom**, linked to the version *and platform*
you are running.

---
