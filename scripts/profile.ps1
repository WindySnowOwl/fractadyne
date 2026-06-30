<#
.SYNOPSIS
  Build the optimized binary and run the Fractadyne profiling harness.

.DESCRIPTION
  Renders the benchmark regions, times the costly stages (bignum reference orbit,
  series-approximation setup, GPU iterate / full render), and writes a JSON log to logs/.
  Use scripts/profile-compare.ps1 to diff a before/after pair and validate an optimization.

.EXAMPLE
  ./scripts/profile.ps1                       # default regions, 5 reps
  ./scripts/profile.ps1 -Reps 10
  ./scripts/profile.ps1 -Regions scripts/regions.example.toml -Out logs/mine.json
#>
param(
    [int]$Reps = 5,
    [string]$Regions = "",
    [string]$Out = ""
)
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

Stop-Process -Name fractadyne -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 300
cargo build --release -p fractadyne-app -j 1

$exe = "target\release\fractadyne.exe"
$cliArgs = @("--profile", "--reps", $Reps)
if ($Regions) { $cliArgs += @("--regions", $Regions) }
if ($Out) { $cliArgs += @("--out", $Out) }
& $exe @cliArgs
