# fractalshark-gui-scene.ps1 - PROTOTYPE of a FractalShark GUI lane (2026-09-08). Drives one scene
# through the GUI with no mouse: sizes the window, fills the Enter Location dialog (real, imaginary,
# zoom, iterations) by WM_SETTEXT + a posted OK, selects the render algorithm and antialiasing by
# WM_COMMAND, runs Benchmark (5x, full recalc) which writes BenchmarkResults.txt in the working
# directory, and saves the bitmap through the Save As dialog by typing the path.
#
# WARNING: it only yields GPU pictures on a card the release was built for. The 0.532 and 0.54
# binaries embed CUDA code for sm_89 (RTX 40xx) and sm_120 (RTX 50xx) ONLY - see
# cuda-arch-inventory.py - so on an RTX 3080 (sm_86) every GPU algorithm renders flat, in the GUI
# as in the CLI. Check the saved image for structure before believing any time.
#
# scene-probe.ps1 - one scene through the FractalShark GUI without a mouse: size the window, enter
# the location (real, imaginary, zoom, iterations) through the Enter Location dialog, choose the
# render algorithm and antialiasing, run Benchmark (5x, full recalc), save the bitmap through the
# Save dialog (typed path), report. Windows PowerShell 5.1. Bounded waits; instance killed at end.
param(
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$OutDir,
    [Parameter(Mandatory = $true)][string]$Kfr,
    [Parameter(Mandatory = $true)][double]$MagLog10,
    [Parameter(Mandatory = $true)][int]$Iters,
    [string]$Size = '1920x1080',
    [int]$AlgoId = 41403,       # HDRx32 GPU - LAv2
    [string]$Tag = 'scene',
    [int]$SettleS = 8,
    [int]$RenderWaitS = 600
)
$ErrorActionPreference = 'Continue'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Text;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class W {
    public delegate bool EnumProc(IntPtr h, IntPtr l);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr SendMessage(IntPtr h, uint msg, IntPtr w, string l);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint msg, IntPtr w, IntPtr l);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc p, IntPtr l);
    [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr h, EnumProc p, IntPtr l);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetClassName(IntPtr h, StringBuilder s, int max);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowText(IntPtr h, StringBuilder s, int max);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll")] public static extern int GetDlgCtrlID(IntPtr h);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L; public int T; public int R; public int B; }
    public static List<IntPtr> Found = new List<IntPtr>();
    public static bool Collect(IntPtr h, IntPtr l) { Found.Add(h); return true; }
    public static string Cls(IntPtr h) { var sb = new StringBuilder(256); GetClassName(h, sb, 256); return sb.ToString(); }
    public static string Txt(IntPtr h) { var sb = new StringBuilder(4096); GetWindowText(h, sb, 4096); return sb.ToString(); }
}
"@
$WM_COMMAND = 0x0111; $WM_CLOSE = 0x0010; $WM_SETTEXT = 0x000C; $BM_CLICK = 0x00F5
$ID_DETAILS = 40015; $ID_ENTER_LOCATION = 40831; $ID_AA1 = 40300; $ID_BENCH = 40820; $ID_SAVE_BMP = 40803
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $OutDir 'BenchmarkResults.txt')

function Windows-Of($procId, $cls) {
    [W]::Found.Clear()
    [W]::EnumWindows([W+EnumProc]{ param($h, $l) [W]::Collect($h, $l) }, [IntPtr]::Zero) | Out-Null
    $out = @()
    foreach ($h in [W]::Found) {
        $q = 0; [W]::GetWindowThreadProcessId($h, [ref]$q) | Out-Null
        if ($q -eq $procId -and [W]::IsWindowVisible($h) -and (-not $cls -or [W]::Cls($h) -eq $cls)) { $out += $h }
    }
    return $out
}
function Children($h) {
    [W]::Found.Clear()
    [W]::EnumChildWindows($h, [W+EnumProc]{ param($c, $l) [W]::Collect($c, $l) }, [IntPtr]::Zero) | Out-Null
    return @([W]::Found)
}
function Wait-Dialog($procId, $seconds) {
    $t0 = Get-Date
    while (((Get-Date) - $t0).TotalSeconds -lt $seconds) {
        $d = (Windows-Of $procId '#32770') | Select-Object -First 1
        if ($d) { return $d }
        Start-Sleep -Milliseconds 300
    }
    return $null
}
function Details($h, $procId) {
    [System.Windows.Forms.Clipboard]::Clear()
    [W]::PostMessage($h, $WM_COMMAND, [IntPtr]$ID_DETAILS, [IntPtr]::Zero) | Out-Null
    Start-Sleep -Seconds 2
    $t = ''
    try { $t = [System.Windows.Forms.Clipboard]::GetText() } catch {}
    foreach ($d in (Windows-Of $procId '#32770')) { [W]::PostMessage($d, $WM_CLOSE, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null }
    Start-Sleep -Milliseconds 400
    return $t
}
function Summ($t) {
    ($t -split "`r?`n" | Where-Object { $_ -match '^(Using|Overall|Per pixel|RefOrbit \(ms\)|LA generation|ZoomFactor|SetNumIterations|Center X)' }) -join ' | '
}

$re = ((Get-Content $Kfr | Where-Object { $_ -match '^Re:' }) -replace '^Re:\s*', '').Trim()
$im = ((Get-Content $Kfr | Where-Object { $_ -match '^Im:' }) -replace '^Im:\s*', '').Trim()
$zoom = ('{0:E12}' -f [Math]::Pow(10, $MagLog10))
$wh = $Size -split 'x'; $cw = [int]$wh[0]; $ch = [int]$wh[1]

$p = Start-Process -FilePath $Exe -WorkingDirectory $OutDir -PassThru
Start-Sleep -Seconds $SettleS
$p.Refresh(); $h = $p.MainWindowHandle
"[$Tag] pid=$($p.Id) handle=$h"
if ($h -eq [IntPtr]::Zero) { "no window"; Stop-Process -Id $p.Id -Force; exit 1 }

# 1. Size the window so the render is Size (client area = frame buffer). Add the frame first, then
# correct from the measured client size.
[W]::SetWindowPos($h, [IntPtr]::Zero, 0, 0, $cw, $ch, 0x0004 -bor 0x0020) | Out-Null   # SWP_NOZORDER | SWP_FRAMECHANGED
Start-Sleep -Milliseconds 800
$cr = New-Object W+RECT; [W]::GetClientRect($h, [ref]$cr) | Out-Null
$dw = $cw - ($cr.R - $cr.L); $dh = $ch - ($cr.B - $cr.T)
if ($dw -ne 0 -or $dh -ne 0) {
    [W]::SetWindowPos($h, [IntPtr]::Zero, 0, 0, $cw + $dw, $ch + $dh, 0x0004 -bor 0x0020) | Out-Null
    Start-Sleep -Milliseconds 800
    [W]::GetClientRect($h, [ref]$cr) | Out-Null
}
"[$Tag] client area: $($cr.R - $cr.L) x $($cr.B - $cr.T)"

# 2. Enter Location: real, imaginary, zoom, iterations.
[W]::PostMessage($h, $WM_COMMAND, [IntPtr]$ID_ENTER_LOCATION, [IntPtr]::Zero) | Out-Null
$dlg = Wait-Dialog $p.Id 10
if (-not $dlg) { "[$Tag] no Enter Location dialog"; Stop-Process -Id $p.Id -Force; exit 1 }
$edits = @{}; $ok = $null
foreach ($c in (Children $dlg)) {
    $id = [W]::GetDlgCtrlID($c)
    if ([W]::Cls($c) -eq 'Edit') { $edits[$id] = $c }
    if ([W]::Cls($c) -eq 'Button' -and [W]::Txt($c) -eq 'OK') { $ok = $c }
}
[W]::SendMessage($edits[1000], $WM_SETTEXT, [IntPtr]::Zero, $re) | Out-Null
[W]::SendMessage($edits[1001], $WM_SETTEXT, [IntPtr]::Zero, $im) | Out-Null
[W]::SendMessage($edits[1002], $WM_SETTEXT, [IntPtr]::Zero, $zoom) | Out-Null
[W]::SendMessage($edits[1003], $WM_SETTEXT, [IntPtr]::Zero, ([string]$Iters)) | Out-Null
[W]::PostMessage($ok, $BM_CLICK, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
"[$Tag] location entered: zoom=$zoom iters=$Iters"

# 3. Algorithm + antialiasing, then wait for the render by polling the details' benchmark line.
[W]::PostMessage($h, $WM_COMMAND, [IntPtr]$AlgoId, [IntPtr]::Zero) | Out-Null
Start-Sleep -Seconds 1
[W]::PostMessage($h, $WM_COMMAND, [IntPtr]$ID_AA1, [IntPtr]::Zero) | Out-Null
$t0 = Get-Date
Start-Sleep -Seconds 3
"[$Tag] details after load -> " + (Summ (Details $h $p.Id))

# 4. Benchmark (5x, full recalc) -> BenchmarkResults.txt in the working directory.
$t0 = Get-Date
[W]::PostMessage($h, $WM_COMMAND, [IntPtr]$ID_BENCH, [IntPtr]::Zero) | Out-Null
$file = Join-Path $OutDir 'BenchmarkResults.txt'
while (-not (Test-Path $file) -and ((Get-Date) - $t0).TotalSeconds -lt $RenderWaitS) { Start-Sleep -Seconds 1 }
$benchS = [int]((Get-Date) - $t0).TotalSeconds
"[$Tag] benchmark wall ${benchS}s, file: $(Test-Path $file)"
if (Test-Path $file) { "[$Tag] " + ((Get-Content $file) -join ' / ') }
foreach ($d in (Windows-Of $p.Id '#32770')) { [W]::PostMessage($d, $WM_CLOSE, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null }
Start-Sleep -Seconds 1
"[$Tag] final details -> " + (Summ (Details $h $p.Id))

# 5. Save Bitmap Image through the Save dialog: bring it to the front and type the path.
$png = Join-Path $OutDir ($Tag + '.png')
Remove-Item -Force -ErrorAction SilentlyContinue $png
[W]::PostMessage($h, $WM_COMMAND, [IntPtr]$ID_SAVE_BMP, [IntPtr]::Zero) | Out-Null
$dlg = Wait-Dialog $p.Id 15
if ($dlg) {
    "[$Tag] save dialog '$([W]::Txt($dlg))'"
    [W]::SetForegroundWindow($dlg) | Out-Null
    Start-Sleep -Milliseconds 600
    [System.Windows.Forms.SendKeys]::SendWait('^a')
    [System.Windows.Forms.SendKeys]::SendWait($png.Replace('{', '{{}').Replace('}', '{}}').Replace('+', '{+}').Replace('^', '{^}').Replace('%', '{%}').Replace('~', '{~}').Replace('(', '{(}').Replace(')', '{)}'))
    Start-Sleep -Milliseconds 300
    [System.Windows.Forms.SendKeys]::SendWait('{ENTER}')
    $t0 = Get-Date
    while (-not (Test-Path $png) -and ((Get-Date) - $t0).TotalSeconds -lt 60) { Start-Sleep -Milliseconds 500 }
    Start-Sleep -Seconds 2
    "[$Tag] saved: $(Test-Path $png) $(if (Test-Path $png) { (Get-Item $png).Length })"
} else { "[$Tag] no save dialog appeared" }
foreach ($d in (Windows-Of $p.Id '#32770')) { [W]::PostMessage($d, $WM_CLOSE, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null }
Stop-Process -Id $p.Id -Force
Start-Sleep -Seconds 1
Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $OutDir 'HeapFile.bin')
Get-ChildItem $OutDir -Filter '*.iters' | Remove-Item -Force -ErrorAction SilentlyContinue
"[$Tag] closed pid $($p.Id)"
