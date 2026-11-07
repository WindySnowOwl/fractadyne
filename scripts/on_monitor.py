#!/usr/bin/env python3
"""Start a command so its first window opens on a chosen monitor (Windows).

    python scripts/on_monitor.py right -- target/release/fractadyne.exe --motiontest --worker-gpu same
    python scripts/on_monitor.py --list

A test window on the monitor someone is working on catches their clicks and keystrokes, and theirs
catch its focus. This puts it on another monitor from the start.

It does not move the window. Windows gives a program's first window the position in the
STARTUPINFO it was started with (STARTF_USEPOSITION) when the program asks for the default
position, which is what winit does. So the window is created on that monitor, at that monitor's
scaling, and never crosses a scaling boundary. Moving it after it opens would: winit creates its
window at the default spot and moves it there afterward, and a move from a 150% monitor to a 100%
one is the WM_DPICHANGED path where the monitor-drag growth bug lives (GitHub #2).

Monitors: `right` (default), `left`, `primary`, or a device name as `--list` prints it
(`DISPLAY3` or `\\\\.\\DISPLAY3`). The exit code is the command's. On other systems the command
just runs.
"""
import os
import subprocess
import sys

# Distance from the monitor's work-area corner, in that monitor's pixels: clear of the edge, and
# of a taskbar placed at the top or left.
MARGIN = 40


def monitors():
    """`[(device, (left, top, right, bottom) work area in physical px, dpi, primary)]`."""
    import ctypes
    from ctypes import wintypes

    user32 = ctypes.WinDLL("user32", use_last_error=True)
    shcore = ctypes.WinDLL("shcore")

    class MONITORINFOEXW(ctypes.Structure):
        _fields_ = [("cbSize", wintypes.DWORD), ("rcMonitor", wintypes.RECT),
                    ("rcWork", wintypes.RECT), ("dwFlags", wintypes.DWORD),
                    ("szDevice", wintypes.WCHAR * 32)]

    # Physical pixels on every monitor, whatever this process's own DPI awareness.
    user32.SetThreadDpiAwarenessContext.restype = ctypes.c_void_p
    user32.SetThreadDpiAwarenessContext.argtypes = [ctypes.c_void_p]
    old = user32.SetThreadDpiAwarenessContext(ctypes.c_void_p(-4))  # PER_MONITOR_AWARE_V2
    out = []
    proc_t = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HMONITOR, wintypes.HDC,
                                ctypes.POINTER(wintypes.RECT), wintypes.LPARAM)

    def each(hmon, _hdc, _rect, _l):
        mi = MONITORINFOEXW()
        mi.cbSize = ctypes.sizeof(mi)
        user32.GetMonitorInfoW(hmon, ctypes.byref(mi))
        dx, dy = wintypes.UINT(), wintypes.UINT()
        shcore.GetDpiForMonitor(hmon, 0, ctypes.byref(dx), ctypes.byref(dy))
        w = mi.rcWork
        out.append((mi.szDevice, (w.left, w.top, w.right, w.bottom), dx.value, bool(mi.dwFlags & 1)))
        return True

    try:
        user32.EnumDisplayMonitors(None, None, proc_t(each), 0)
    finally:
        if old:
            user32.SetThreadDpiAwarenessContext(ctypes.c_void_p(old))
    return out


def pick(where, mons):
    if not mons:
        sys.exit("on_monitor: no monitors found")
    if where == "right":
        return max(mons, key=lambda m: m[1][2])
    if where == "left":
        return min(mons, key=lambda m: m[1][0])
    if where == "primary":
        return next(m for m in mons if m[3])
    want = where.upper().lstrip("\\.")
    for m in mons:
        if m[0].upper().lstrip("\\.") == want:
            return m
    sys.exit("on_monitor: no monitor %r (have: %s)" % (where, ", ".join(m[0] for m in mons)))


def describe(m):
    dev, (l, t, r, b), dpi, primary = m
    return "%s %dx%d work area at (%d, %d), %d%%%s" % (
        dev, r - l, b - t, l, t, round(dpi * 100 / 96), ", primary" if primary else "")


def run_at(cmd, x, y, cwd=None, env=None):
    """Run `cmd` (a list) with its first window at (x, y) physical px; wait; return its exit code.
    Standard handles are inherited the way `subprocess.run` does with no redirection."""
    import ctypes
    from ctypes import wintypes

    kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)

    class STARTUPINFOW(ctypes.Structure):
        _fields_ = [("cb", wintypes.DWORD), ("lpReserved", wintypes.LPWSTR),
                    ("lpDesktop", wintypes.LPWSTR), ("lpTitle", wintypes.LPWSTR),
                    ("dwX", wintypes.DWORD), ("dwY", wintypes.DWORD),
                    ("dwXSize", wintypes.DWORD), ("dwYSize", wintypes.DWORD),
                    ("dwXCountChars", wintypes.DWORD), ("dwYCountChars", wintypes.DWORD),
                    ("dwFillAttribute", wintypes.DWORD), ("dwFlags", wintypes.DWORD),
                    ("wShowWindow", wintypes.WORD), ("cbReserved2", wintypes.WORD),
                    ("lpReserved2", ctypes.c_void_p), ("hStdInput", wintypes.HANDLE),
                    ("hStdOutput", wintypes.HANDLE), ("hStdError", wintypes.HANDLE)]

    class PROCESS_INFORMATION(ctypes.Structure):
        _fields_ = [("hProcess", wintypes.HANDLE), ("hThread", wintypes.HANDLE),
                    ("dwProcessId", wintypes.DWORD), ("dwThreadId", wintypes.DWORD)]

    STARTF_USEPOSITION = 0x4
    CREATE_UNICODE_ENVIRONMENT = 0x400
    INFINITE = 0xFFFFFFFF

    si = STARTUPINFOW()
    si.cb = ctypes.sizeof(si)
    si.dwFlags = STARTF_USEPOSITION
    # DWORD fields: a monitor left of the primary has negative coordinates, passed as their
    # two's complement, which is how Windows reads them back.
    si.dwX = x & 0xFFFFFFFF
    si.dwY = y & 0xFFFFFFFF
    pi = PROCESS_INFORMATION()

    # CreateProcessW does not take `target/release/x.exe` with forward slashes as the program to
    # find, nor search PATH for a bare `python` the way `subprocess.run` appears to: resolve it.
    import shutil
    exe = cmd[0]
    base = cwd or os.getcwd()
    if os.path.exists(os.path.join(base, exe)):
        exe = os.path.normpath(os.path.join(base, exe))
    else:
        exe = shutil.which(exe, path=(env or os.environ).get("PATH")) or exe
    cmd = [exe] + list(cmd[1:])

    block = None
    if env is not None:
        text = "".join("%s=%s\0" % kv for kv in sorted(env.items(), key=lambda kv: kv[0].upper())) + "\0"
        block = ctypes.create_unicode_buffer(text, len(text))
    cmdline = ctypes.create_unicode_buffer(subprocess.list2cmdline(cmd))

    kernel32.CreateProcessW.argtypes = [wintypes.LPCWSTR, wintypes.LPWSTR, ctypes.c_void_p,
                                        ctypes.c_void_p, wintypes.BOOL, wintypes.DWORD,
                                        ctypes.c_void_p, wintypes.LPCWSTR,
                                        ctypes.POINTER(STARTUPINFOW),
                                        ctypes.POINTER(PROCESS_INFORMATION)]
    ok = kernel32.CreateProcessW(exe, cmdline, None, None, False, CREATE_UNICODE_ENVIRONMENT,
                                 block, cwd, ctypes.byref(si), ctypes.byref(pi))
    if not ok:
        raise OSError(ctypes.get_last_error(), "CreateProcessW failed for %s" % cmd[0])
    try:
        kernel32.WaitForSingleObject(pi.hProcess, INFINITE)
        code = wintypes.DWORD()
        kernel32.GetExitCodeProcess(pi.hProcess, ctypes.byref(code))
        # Exit codes are 32-bit; Python's convention for a negative one is the signed value.
        return ctypes.c_int32(code.value).value
    finally:
        kernel32.CloseHandle(pi.hThread)
        kernel32.CloseHandle(pi.hProcess)


def run(cmd, where="right", cwd=None, env=None, say=print):
    """`subprocess.run(cmd).returncode`, with the first window on monitor `where`."""
    if os.name != "nt":
        return subprocess.run(cmd, cwd=cwd, env=env).returncode
    m = pick(where, monitors())
    l, t = m[1][0], m[1][1]
    say("on_monitor: window on %s" % describe(m))
    return run_at(cmd, l + MARGIN, t + MARGIN, cwd=cwd, env=env)


def main():
    args = sys.argv[1:]
    if args[:1] == ["--list"]:
        if os.name != "nt":
            sys.exit("on_monitor: --list is for Windows")
        for m in monitors():
            print(describe(m))
        return 0
    where = "right"
    if args and args[0] != "--":
        where, args = args[0], args[1:]
    if args[:1] == ["--"]:
        args = args[1:]
    if not args:
        sys.exit(__doc__.split("\n\n")[1])
    sys.stdout.flush()
    return run(args, where, say=lambda s: print(s, flush=True))


if __name__ == "__main__":
    sys.exit(main())
