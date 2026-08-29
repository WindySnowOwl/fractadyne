## Which download?

| File | For |
|---|---|
| `fractadyne-<version>-windows-x64.zip` | **Windows.** Start here if you are unsure. |
| `fractadyne-<version>-linux-x64.tar.gz` | **Linux.** Expect it to be less tested than the Windows build, because it is. |
| `fractadyne-<version>-windows-x64-accelerated.zip` | Windows, **optional** — faster at depth. See below. |

Each has a `.sha256` beside it. Nothing is installed: unpack anywhere, run `fractadyne.exe`
(or `fractadyne` on Linux), and delete the folder to remove it. You need 64-bit Windows 10/11 or a
current Linux, and a GPU with Vulkan, DirectX 12 or OpenGL.

## What `-accelerated.zip` is

The **same** Fractadyne as the standard Windows download, with one difference: it computes
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
- **Keep the `.dll` files next to `fractadyne.exe`.** The program will not start without them.
- To see which arithmetic you are running: **Help → About**, the "Deep-zoom arithmetic" line.

**Why it is a separate download** — two reasons, neither going away:

1. MPFR cannot be built with the Microsoft compiler the standard Windows binary uses, so this build
   comes from a different toolchain.
2. MPFR and GMP are licensed under the **GNU LGPL v3**, while Fractadyne itself is MIT OR
   Apache-2.0. Keeping them in a separate, clearly labelled download keeps the standard build free
   of those terms. The full notices, and the licence texts, are inside the zip.

If you don't need it, take the plain `windows-x64.zip`. The app will offer you the accelerated
build later from **Help → Faster deep zoom**, linked to the version you are running.

---
