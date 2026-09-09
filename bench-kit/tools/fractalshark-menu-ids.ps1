# fractalshark-menu-ids.ps1 - discover the FractalShark GUI popup menu tree with its WM_COMMAND ids
# from the LIVE menu (MN_GETHMENU; the menu is built in code, there is no menu resource), then
# exercise Show Rendering Details (clipboard) and Benchmark (5x, full recalc). 2026-09-08.
#
# menu-walk.ps1 - launch the FractalShark GUI, open its popup, read the live HMENU tree with
# command IDs (MN_GETHMENU), then drive it by posting WM_COMMAND: Show Rendering Details
# (clipboard), Benchmark (5x, full recalc) (dialog / BenchmarkResults.txt), capture the window
# with PrintWindow, and close the instance. Every wait is bounded. Windows PowerShell 5.1.
param(
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$OutDir,
    [int]$SettleS = 8,
    [int]$BenchWaitS = 180
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
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindow(string cls, string title);
    [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr h, uint msg, IntPtr w, IntPtr l);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint msg, IntPtr w, IntPtr l);
    [DllImport("user32.dll")] public static extern int GetMenuItemCount(IntPtr m);
    [DllImport("user32.dll")] public static extern IntPtr GetSubMenu(IntPtr m, int pos);
    [DllImport("user32.dll")] public static extern uint GetMenuItemID(IntPtr m, int pos);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetMenuString(IntPtr m, uint item, StringBuilder s, int max, uint flag);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc p, IntPtr l);
    [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr h, EnumProc p, IntPtr l);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetClassName(IntPtr h, StringBuilder s, int max);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowText(IntPtr h, StringBuilder s, int max);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr dc, uint flags);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L; public int T; public int R; public int B; }
    public static List<IntPtr> Found = new List<IntPtr>();
    public static bool Collect(IntPtr h, IntPtr l) { Found.Add(h); return true; }
    public static string Cls(IntPtr h) { var sb = new StringBuilder(256); GetClassName(h, sb, 256); return sb.ToString(); }
    public static string Txt(IntPtr h) { var sb = new StringBuilder(4096); GetWindowText(h, sb, 4096); return sb.ToString(); }
}
"@
$WM_COMMAND = 0x0111; $WM_CLOSE = 0x0010; $MN_GETHMENU = 0x01E1; $MF_BYPOSITION = 0x400
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $OutDir 'BenchmarkResults.txt')

function Windows-Of($pid, $cls) {
    [W]::Found.Clear()
    [W]::EnumWindows([W+EnumProc]{ param($h, $l) [W]::Collect($h, $l) }, [IntPtr]::Zero) | Out-Null
    $out = @()
    foreach ($h in [W]::Found) {
        $p = 0; [W]::GetWindowThreadProcessId($h, [ref]$p) | Out-Null
        if ($p -eq $pid -and [W]::IsWindowVisible($h) -and (-not $cls -or [W]::Cls($h) -eq $cls)) { $out += $h }
    }
    return $out
}
function Dialog-Text($h) {
    [W]::Found.Clear()
    [W]::EnumChildWindows($h, [W+EnumProc]{ param($c, $l) [W]::Collect($c, $l) }, [IntPtr]::Zero) | Out-Null
    $lines = @()
    foreach ($c in [W]::Found) { $t = [W]::Txt($c); if ($t) { $lines += ('[' + [W]::Cls($c) + '] ' + $t) } }
    return $lines
}
function Capture($h, $path) {
    $r = New-Object W+RECT
    [W]::GetWindowRect($h, [ref]$r) | Out-Null
    $bmp = New-Object System.Drawing.Bitmap(($r.R - $r.L), ($r.B - $r.T))
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $dc = $g.GetHdc()
    [W]::PrintWindow($h, $dc, 2) | Out-Null   # PW_RENDERFULLCONTENT
    $g.ReleaseHdc($dc)
    $bmp.Save($path)
    $g.Dispose(); $bmp.Dispose()
}
$script:tree = @()
$script:ids = @{}
function Walk($menu, $depth, $path) {
    $n = [W]::GetMenuItemCount($menu)
    for ($i = 0; $i -lt $n; $i++) {
        $sb = New-Object System.Text.StringBuilder 256
        [W]::GetMenuString($menu, [uint32]$i, $sb, 256, $MF_BYPOSITION) | Out-Null
        $text = $sb.ToString(); if ($text -eq '') { $text = '---' }
        $sub = [W]::GetSubMenu($menu, $i)
        if ($sub -ne [IntPtr]::Zero) {
            $script:tree += (('  ' * $depth) + '+ ' + $text)
            Walk $sub ($depth + 1) ($path + $text + ' > ')
        } else {
            $id = [W]::GetMenuItemID($menu, $i)
            $script:tree += (('  ' * $depth) + '- ' + $text + '  [id ' + $id + ']')
            if ($text -ne '---') { $script:ids[$text] = $id }
        }
    }
}

$p = Start-Process -FilePath $Exe -WorkingDirectory $OutDir -PassThru
Start-Sleep -Seconds $SettleS
$p.Refresh()
$h = $p.MainWindowHandle
"pid=$($p.Id) title='$($p.MainWindowTitle)' handle=$h"
if ($h -eq [IntPtr]::Zero) { "no main window"; Stop-Process -Id $p.Id -Force; exit 1 }
Capture $h (Join-Path $OutDir 'pw-launch.png')
# Open the popup by MESSAGES addressed to the window - no cursor movement, no focus change. An
# earlier version synthesised a real right-click and dismissed the menu with an Escape keystroke;
# on a desktop someone is using, input like that lands wherever THEIR focus is (2026-09-09).
# The app tracks its popup on a right-button release in the client area. UNTESTED since the rewrite.
$cr = New-Object W+RECT; [W]::GetClientRect($h, [ref]$cr) | Out-Null
$lp = [IntPtr](([int](($cr.B - $cr.T) / 2) -shl 16) -bor [int](($cr.R - $cr.L) / 2))
[W]::PostMessage($h, 0x0204, [IntPtr]2, $lp) | Out-Null    # WM_RBUTTONDOWN, MK_RBUTTON
[W]::PostMessage($h, 0x0205, [IntPtr]0, $lp) | Out-Null    # WM_RBUTTONUP
Start-Sleep -Milliseconds 900
$popup = [W]::FindWindow('#32768', $null)
"popup window: $popup"
if ($popup -ne [IntPtr]::Zero) {
    $hmenu = [W]::SendMessage($popup, $MN_GETHMENU, [IntPtr]::Zero, [IntPtr]::Zero)
    "hmenu: $hmenu"
    if ($hmenu -ne [IntPtr]::Zero) { Walk $hmenu 0 '' }
}
[W]::SendMessage($h, 0x001F, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null   # WM_CANCELMODE ends the menu
Start-Sleep -Milliseconds 400
"=== MENU TREE ($($script:tree.Count) lines) ==="
$script:tree
$script:tree | Set-Content -Encoding UTF8 (Join-Path $OutDir 'menu-tree.txt')

function Post-Command($name) {
    $id = $script:ids[$name]
    if ($null -eq $id) { "command '$name' not in the menu"; return $false }
    "posting WM_COMMAND $id ('$name')"
    [W]::PostMessage($h, $WM_COMMAND, [IntPtr]$id, [IntPtr]::Zero) | Out-Null
    return $true
}

# 1. Show Rendering Details -> modal dialog + clipboard.
[System.Windows.Forms.Clipboard]::Clear()
if (Post-Command 'Show Rendering Details') {
    Start-Sleep -Seconds 3
    "--- clipboard after Show Rendering Details ---"
    try { [System.Windows.Forms.Clipboard]::GetText() } catch { "(clipboard unreadable: $_)" }
    foreach ($d in (Windows-Of $p.Id '#32770')) {
        "--- dialog '$([W]::Txt($d))' ---"
        Dialog-Text $d
        [W]::PostMessage($d, $WM_CLOSE, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
    }
    Start-Sleep -Milliseconds 500
}

# 2. Benchmark (5x, full recalc) -> wait for a result dialog or BenchmarkResults.txt.
$benchName = ($script:ids.Keys | Where-Object { $_ -like 'Benchmark (5x, full*' } | Select-Object -First 1)
if ($benchName) {
    $t0 = Get-Date
    Post-Command $benchName | Out-Null
    $done = $false
    while (((Get-Date) - $t0).TotalSeconds -lt $BenchWaitS) {
        Start-Sleep -Seconds 2
        $dlgs = Windows-Of $p.Id '#32770'
        $file = Join-Path $OutDir 'BenchmarkResults.txt'
        if ($dlgs.Count -gt 0 -or (Test-Path $file)) { $done = $true; break }
    }
    "benchmark: done=$done after $([int]((Get-Date) - $t0).TotalSeconds)s"
    foreach ($d in (Windows-Of $p.Id '#32770')) {
        "--- dialog '$([W]::Txt($d))' ---"
        Dialog-Text $d
        [W]::PostMessage($d, $WM_CLOSE, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
    }
    "--- clipboard after benchmark ---"
    try { [System.Windows.Forms.Clipboard]::GetText() } catch { "(clipboard unreadable)" }
    Get-ChildItem $OutDir | Where-Object { $_.Name -notlike 'pw-*' } | ForEach-Object { "  file: " + $_.Name + " " + $_.Length }
    if (Test-Path (Join-Path $OutDir 'BenchmarkResults.txt')) { "--- BenchmarkResults.txt ---"; Get-Content (Join-Path $OutDir 'BenchmarkResults.txt') }
}
Start-Sleep -Seconds 1
Capture $h (Join-Path $OutDir 'pw-final.png')
Stop-Process -Id $p.Id -Force
"closed pid $($p.Id)"
