# Deep-view PIXELLATION repro — the settle grid needs more tiles than it is allowed.
#
# Reproduces the 2026-08-14 field report (TODO.md "Open bugs"): park at a deep view with an
# EXPLICIT iteration count, click to zoom 100x, and the destination settles as a coarse mosaic that
# only a window resize clears. The stuck frame reports the cause directly:
#
#   can_tile=true tiling=false fresh=85x49 geo=Some(([1920,1102], 1, 110)) max_tiles=16
#
# 110 tiles needed, 16 allowed => the tiled settle refuses to start and the frame falls back to the
# budget-derived resolution. A healthy run reports max_tiles=512 and tiles to native in under a
# second.
#
# WHY A SCRIPT. This bug lives in the GUI settle path, which no existing harness reaches: --livetest
# plays a TOUR, and `res_scale = 1.0` (the settled full-resolution branch) is explicitly gated on
# `!tour_playing()`. So the only way to measure it is to drive the real window. This script does
# that headlessly enough to run unattended:
#   1. writes a session pinned at the reporter's exact coordinates, in a THROWAWAY config dir
#      (FRACTADYNE_CONFIG_DIR — never touches your own session),
#   2. boots and waits for the first reference (a cold 4M-iteration build at ~550 bits is ~45 s),
#   3. injects a real left-click on the canvas through user32 (click-to-zoom, factor from session),
#   4. captures the trace and prints the resolution timeline plus the tiling verdict.
#
#   .\validation\pixellation-repro.ps1                 # full run (~4 min)
#   .\validation\pixellation-repro.ps1 -SettleSec 30   # shorter waits on a fast machine
#
# ⚠Read the RESOLUTION TIMELINE, not the verdict: a frame at 85x49 upscaled to the panel IS the
# bug, and nothing in the app calls that an error.

param(
    [string]$Exe = "$PSScriptRoot\..\target\release\fractadyne.exe",
    [int]$SettleSec = 55,      # time for the cold reference build before the click
    [int]$WatchSec = 100       # time to watch after the click
)

$ErrorActionPreference = "Stop"
$cfg = Join-Path $env:TEMP "fdcfg_pixrepro"
Remove-Item -LiteralPath $cfg -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $cfg | Out-Null

# The reporter's point A (the click's origin). Full precision matters: at 1e104 the elided middle
# digits of a status-bar readout ARE the structure, so a screenshot cannot seed this.
$re = "-1.01096363845622131810062384757351929938361014185318540959576769264716835033666295089126713641247178090131019571879772223879089263721208e-1"
$im = "9.5628651080914147131604703998237075557983304380930462483482733212267499793490593467836270525491775144889554436475852768427866482512100200000000000000000003e-1"
$upp = 3.77446476099229243e-108

# Session = the committed corpus template (already pins every image-affecting key) + this view.
$tmpl = Get-Content -LiteralPath "$PSScriptRoot\corpus\session-template.toml" -Raw
$e2 = [math]::Floor([math]::Log($upp, 2))
$m = $upp / [math]::Pow(2, $e2)
$pins = @{
    center_x_str = '"' + $re + '"'; center_y_str = '"' + $im + '"'
    center_x = "-0.10109636384562213"; center_y = "0.9562865108091415"
    units_per_pixel = $m.ToString("R"); units_per_pixel_e = "$e2"
    max_iter = "4000000"; auto_iter = "false"; aa = "2"
    click_zoom = "true"; click_zoom_factor = "100.0"
    min_motion_res = "0.75"; prefer_detail = "true"; work_budget_scale = "8.0"
}
foreach ($k in $pins.Keys) {
    $tmpl = [regex]::Replace($tmpl, "(?m)^$k = .*$", "$k = $($pins[$k])")
}
Set-Content -LiteralPath (Join-Path $cfg "session.toml") -Value $tmpl -NoNewline

Add-Type @'
using System; using System.Runtime.InteropServices;
public class FdClick {
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, IntPtr e);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out Rect r);
  public struct Rect { public int L, T, R, B; }
}
'@

$env:FRACTADYNE_CONFIG_DIR = $cfg
$env:FRACTADYNE_TRACE = 'tile,ref'
Write-Host "booting at 9.612e104x (config $cfg)"
$p = Start-Process -FilePath $Exe -PassThru
Start-Sleep -Seconds $SettleSec

$h = (Get-Process -Id $p.Id).MainWindowHandle
[FdClick]::SetForegroundWindow($h) | Out-Null
$r = New-Object 'FdClick+Rect'
[FdClick]::GetWindowRect($h, [ref]$r) | Out-Null
# Left of centre so the click lands on the canvas, clear of the ~430 px controls panel.
$cx = [int](0.5 * ($r.L + $r.R) - 220)
$cy = [int](0.5 * ($r.T + $r.B))
Write-Host "click-to-zoom 100x at $cx,$cy"
[FdClick]::SetCursorPos($cx, $cy) | Out-Null
Start-Sleep -Milliseconds 400
[FdClick]::mouse_event(0x02, 0, 0, 0, [IntPtr]::Zero)   # left down
Start-Sleep -Milliseconds 60
[FdClick]::mouse_event(0x04, 0, 0, 0, [IntPtr]::Zero)   # left up

Start-Sleep -Seconds $WatchSec
Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2
Remove-Item Env:\FRACTADYNE_CONFIG_DIR, Env:\FRACTADYNE_TRACE

$log = Join-Path $cfg "logs\fractadyne.log"
Write-Host "`nRESOLUTION TIMELINE (every change):"
$prev = ""
foreach ($line in Get-Content -LiteralPath $log) {
    if ($line -match '\[\+ *([0-9.]+)s\].*res=(\d+x\d+)') {
        if ($Matches[2] -ne $prev) {
            Write-Host ("  {0,8}s  res {1}" -f $Matches[1], $Matches[2])
            $prev = $Matches[2]
        }
    }
}
Write-Host "`nTILING VERDICT at the coarse frames (the bug is geo > max_tiles):"
Select-String -LiteralPath $log -Pattern 'tiling=false' |
    Select-Object -Last 1 |
    ForEach-Object { if ($_.Line -match '(can_tile=\S+ tiling=\S+).*(geo=\S+ ).*(max_tiles=\d+)') { Write-Host "  $($Matches[1]) $($Matches[2])$($Matches[3])" } }
Write-Host "`nfull log: $log"
