# Real-window resize smoke test (LOCAL ONLY — moves the mouse for ~2 s).
#
# Launches the app in a sandboxed config dir (FRACTADYNE_CONFIG_DIR) with FRACTADYNE_PERF=1,
# grabs the window's bottom-right corner with synthetic OS input (SendInput-level, the same
# events a human drag produces), drags it through a resize, closes the app, and analyzes the
# per-resize-frame present cadence the app recorded (perf.jsonl kind:"resize").
#
# This exercises the layer no headless harness can see: the real WM_SIZING stream, the real
# swapchain reconfigure, and DWM compositing. Verdict logic:
#   - median resize-frame interval ~<= 20 ms  -> presents keep pace; any visible stretch is the
#     endemic one-frame compositor scale every wgpu app shows during live resize.
#   - median >> 20 ms                         -> the app is starving presents during resize;
#     there is an app-side pacing bug to find.
#
# Usage:  ./scripts/resize-smoke.ps1 [-Exe path\to\fractadyne.exe]
param(
    [string]$Exe = "$PSScriptRoot\..\target\release\fractadyne.exe",
    # Optional session.toml to copy into the sandbox (e.g. your real deep view) — measures the
    # resize cadence at that view instead of the default shallow Mandelbrot.
    [string]$Session = ""
)
$ErrorActionPreference = "Stop"

Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class Win32 {
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    public const uint LEFTDOWN = 0x0002, LEFTUP = 0x0004;
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
}
"@

$scratch = Join-Path $env:TEMP "fractadyne-resize-smoke"
if (Test-Path $scratch) { Remove-Item -Recurse -Force $scratch }
New-Item -ItemType Directory -Force $scratch | Out-Null
if ($Session -ne "" -and (Test-Path $Session)) {
    Copy-Item $Session (Join-Path $scratch "session.toml")
    Write-Host "resize-smoke: using session $Session"
}
$env:FRACTADYNE_CONFIG_DIR = $scratch
$env:FRACTADYNE_PERF = "1"

Write-Host "resize-smoke: launching $Exe (sandboxed config: $scratch)"
$p = Start-Process -FilePath $Exe -PassThru
try {
    # Wait for the window (boot + first frames).
    $deadline = (Get-Date).AddSeconds(20)
    while ($p.MainWindowHandle -eq 0 -and (Get-Date) -lt $deadline) {
        Start-Sleep -Milliseconds 200
        $p.Refresh()
    }
    if ($p.MainWindowHandle -eq 0) { throw "window never appeared" }
    # The first handle can be a placeholder before the real window lays out — poll until the
    # rect is a plausible app window.
    $rect = New-Object Win32+RECT
    while ((Get-Date) -lt $deadline) {
        $p.Refresh()
        [void][Win32]::GetWindowRect($p.MainWindowHandle, [ref]$rect)
        if (($rect.Right - $rect.Left) -gt 400 -and ($rect.Bottom - $rect.Top) -gt 300) { break }
        Start-Sleep -Milliseconds 250
    }
    if (($rect.Right - $rect.Left) -le 400) { throw "window rect never became sane ($($rect.Right - $rect.Left) px wide)" }
    [void][Win32]::SetForegroundWindow($p.MainWindowHandle)
    # Let the view render/settle (a deep session needs its reference built first).
    Start-Sleep -Seconds $(if ($Session -ne "") { 10 } else { 3 })
    [void][Win32]::GetWindowRect($p.MainWindowHandle, [ref]$rect)
    # Grab the bottom-right corner (2 px inside the frame edge).
    $x = $rect.Right - 2; $y = $rect.Bottom - 2
    Write-Host ("resize-smoke: window {0}x{1}; dragging corner..." -f ($rect.Right-$rect.Left), ($rect.Bottom-$rect.Top))
    [void][Win32]::SetCursorPos($x, $y)
    Start-Sleep -Milliseconds 150
    [Win32]::mouse_event([Win32]::LEFTDOWN, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 100
    # ~60 Hz drag: shrink 40 steps, then grow 40 steps (like a human wiggle-resize).
    for ($i = 0; $i -lt 40; $i++) { $x -= 6; $y -= 5; [void][Win32]::SetCursorPos($x, $y); Start-Sleep -Milliseconds 15 }
    for ($i = 0; $i -lt 40; $i++) { $x += 6; $y += 5; [void][Win32]::SetCursorPos($x, $y); Start-Sleep -Milliseconds 15 }
    [Win32]::mouse_event([Win32]::LEFTUP, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Seconds 1
} finally {
    if (-not $p.HasExited) { [void]$p.CloseMainWindow(); if (-not $p.WaitForExit(5000)) { $p.Kill() } }
}

# Analyze the recorded resize-frame cadence.
$perf = Join-Path $scratch "logs\perf.jsonl"
if (-not (Test-Path $perf)) { throw "no perf.jsonl recorded at $perf" }
$dts = Get-Content $perf | Where-Object { $_ -match '"kind":"resize"' } |
    ForEach-Object { if ($_ -match '"dt_ms":([0-9.]+)') { [double]$Matches[1] } } |
    Where-Object { $_ -gt 0 } | Sort-Object
if ($dts.Count -lt 5) { throw "only $($dts.Count) resize frames recorded — drag didn't take?" }
$median = $dts[[int]($dts.Count / 2)]
$p95 = $dts[[int]([Math]::Min($dts.Count - 1, $dts.Count * 0.95))]
$max = $dts[-1]
Write-Host ""
Write-Host ("resize-smoke: {0} resize frames — present cadence median {1:N1} ms, p95 {2:N1} ms, max {3:N1} ms" -f $dts.Count, $median, $p95, $max)
if ($median -le 20) {
    Write-Host "VERDICT: presents keep pace with the resize stream (~vsync). Any visible stretch is the endemic one-frame compositor scale of live resize (wgpu/DWM), not app-side starvation."
    exit 0
} else {
    Write-Host "VERDICT: presents are LAGGING the resize stream — app-side pacing bug; profile what a resize frame spends its time on."
    exit 1
}
