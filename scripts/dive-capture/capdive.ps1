# capdive.ps1 - the PowerShell twin of capdive.sh, for a machine with no sh (the Radeon box, and
# the gpu-validate battery's screen step). Runs the real autopilot from a .kfr location in a WIPED
# scratch config and captures its window by PrintWindow (grab.ps1; no input is sent anywhere), so
# what was ON SCREEN can be scored by screengate.py - see README.md.
#
#   powershell -File capdive.ps1 -Out RUNDIR -Exe fractadyne.exe [-Kfr dive-2p800.kfr]
#       [-Seed session-seed.toml] [-TimeoutS 26] [-Iter 10000]
#
# RUNDIR is wiped first and gets: cfg\ (the scratch config and its logs), frames\ (the captures,
# f<ms since capture start>.jpg), stdout.txt, stderr.txt and exit.txt (the app's exit code).
# Capture starts 4 s after launch and lasts 18 s at ~100 ms, as capdive.sh's does.
#
# ASCII-only on purpose: Windows PowerShell 5.1 reads a BOM-less .ps1 as ANSI.

param(
    [Parameter(Mandatory = $true)][string]$Out,
    [Parameter(Mandatory = $true)][string]$Exe,
    [string]$Kfr = "",
    [string]$Seed = "",
    [int]$TimeoutS = 26,
    [int]$Iter = 10000
)

$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
if (-not $Kfr) { $Kfr = Join-Path $here "dive-2p800.kfr" }
if (-not $Seed) { $Seed = Join-Path $here "session-seed.toml" }
foreach ($f in @($Exe, $Kfr, $Seed, (Join-Path $here "grab.ps1"))) {
    if (-not (Test-Path -LiteralPath $f)) { throw "capdive: not found: $f" }
}

if (Test-Path -LiteralPath $Out) { Remove-Item -Recurse -Force -LiteralPath $Out }
$cfg = Join-Path $Out "cfg"
$frames = Join-Path $Out "frames"
New-Item -ItemType Directory -Force -Path $cfg, $frames | Out-Null
Copy-Item -LiteralPath $Seed -Destination (Join-Path $cfg "session.toml")

# The child inherits these; put back whatever was there after.
$saved = @{}
$set = @{ FRACTADYNE_CONFIG_DIR = $cfg; FRACTADYNE_NO_SOUND = "1"; FRACTADYNE_TRACE = "autopilot,tile,gpu,ref" }
foreach ($k in $set.Keys) { $saved[$k] = [Environment]::GetEnvironmentVariable($k); [Environment]::SetEnvironmentVariable($k, $set[$k]) }
try {
    $argv = @("--import-kfr", "`"$Kfr`"", "--show-timestamp", "--autodive", "300", "--autodive-iter", "$Iter",
        "--autodive-home", "0", "--autodive-timeout", "$TimeoutS")
    $p = Start-Process -FilePath $Exe -ArgumentList $argv -PassThru -WorkingDirectory $Out `
        -RedirectStandardOutput (Join-Path $Out "stdout.txt") -RedirectStandardError (Join-Path $Out "stderr.txt")
    $null = $p.Handle   # Windows PowerShell 5.1 loses ExitCode unless the handle is taken now
}
finally {
    foreach ($k in $set.Keys) { [Environment]::SetEnvironmentVariable($k, $saved[$k]) }
}

Start-Sleep -Seconds 4
# grab.ps1 needs System.Drawing, which only Windows PowerShell 5.1 has - whatever runs this script.
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File (Join-Path $here "grab.ps1") `
    -ProcId $p.Id -OutDir $frames -Seconds 18 -IntervalMs 100 | Out-Null

# The dive ends itself at -TimeoutS; bound the wait in case it does not.
if (-not $p.WaitForExit(($TimeoutS + 60) * 1000)) {
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
    "timeout" | Out-File (Join-Path $Out "exit.txt") -Encoding ascii
    Write-Host "capdive: the dive did not end within $($TimeoutS + 60) s - stopped (pid $($p.Id))"
}
else {
    "$($p.ExitCode)" | Out-File (Join-Path $Out "exit.txt") -Encoding ascii
}
$n = @(Get-ChildItem -LiteralPath $frames -Filter "*.jpg" -ErrorAction SilentlyContinue).Count
Write-Host "capdive: $Out - $n captures, dive exit $(Get-Content (Join-Path $Out 'exit.txt'))"
