# fractalshark-basic-test.ps1 - launch the FractalShark GUI with stdout/stderr captured, post its
# "Run Basic Test (saves files in local dir)" command (WM_COMMAND 40900; one image per render
# algorithm into TestBasic/), report what it wrote, close it. Message-only: no keystrokes, no mouse.
# 2026-09-08: on an RTX 3080 every GPU-algorithm image was flat and every CPU one correct - the
# release binaries carry no code for sm_86 (see cuda-load-check.py).
#
# diag-probe.ps1 - launch the GUI with stdout/stderr captured, run its built-in "Run Basic Test
# (saves files in local dir)", and report what it printed and what it wrote. Windows PowerShell 5.1.
param(
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$OutDir,
    [int]$SettleS = 8,
    [int]$TestWaitS = 180
)
$ErrorActionPreference = 'Continue'
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class W {
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint msg, IntPtr w, IntPtr l);
}
"@
$WM_COMMAND = 0x0111; $ID_BASIC_TEST = 40900
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Get-ChildItem $OutDir -Include '*.png', '*.bmp', '*.txt' -Recurse | Remove-Item -Force -ErrorAction SilentlyContinue
$out = Join-Path $OutDir 'gui-stdout.txt'; $err = Join-Path $OutDir 'gui-stderr.txt'
$p = Start-Process -FilePath $Exe -WorkingDirectory $OutDir -PassThru -RedirectStandardOutput $out -RedirectStandardError $err
Start-Sleep -Seconds $SettleS
$p.Refresh(); $h = $p.MainWindowHandle
"pid=$($p.Id) handle=$h"
$before = @(Get-ChildItem $OutDir -File | Select-Object -ExpandProperty Name)
[W]::PostMessage($h, $WM_COMMAND, [IntPtr]$ID_BASIC_TEST, [IntPtr]::Zero) | Out-Null
$t0 = Get-Date
$seen = @()
while (((Get-Date) - $t0).TotalSeconds -lt $TestWaitS) {
    Start-Sleep -Seconds 3
    $now = @(Get-ChildItem $OutDir -File | Where-Object { $_.Extension -in '.png', '.bmp', '.txt' } | Select-Object -ExpandProperty Name)
    $new = $now | Where-Object { $before -notcontains $_ }
    if ($new.Count -gt 0 -and $new.Count -eq $seen.Count) { break }
    $seen = $new
}
"new files after $([int]((Get-Date) - $t0).TotalSeconds)s:"
Get-ChildItem $OutDir -File | Where-Object { $before -notcontains $_.Name } | ForEach-Object { "  " + $_.Name + "  " + $_.Length }
Stop-Process -Id $p.Id -Force
Start-Sleep -Seconds 1
"--- stderr (tail) ---"
Get-Content $err -ErrorAction SilentlyContinue | Select-Object -Last 40
"--- stdout (tail) ---"
Get-Content $out -ErrorAction SilentlyContinue | Select-Object -Last 20
Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $OutDir 'HeapFile.bin')
Get-ChildItem $OutDir -Filter '*.iters' | Remove-Item -Force -ErrorAction SilentlyContinue
