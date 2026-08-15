<#
.SYNOPSIS
  Pull Fractadyne diagnostics bundles from the Linux test rig to this Windows dev box via scp.

.DESCRIPTION
  The companion to scripts/linux-report.sh. That script gathers each run into
  ~/fractadyne-reports/<timestamp>/ on the Linux box; this one scp's those folders down into
  local/linux-reports/ here (which is gitignored), where Claude Code / you can read them.

  Requires an SSH client on this machine (Windows 10/11's built-in OpenSSH provides scp + ssh)
  and an SSH server on the Linux box. Key-based auth is smoothest; otherwise scp prompts for a
  password per transfer.

.PARAMETER From
  The Linux box as user@host (e.g. rhong@fractrig or rhong@192.168.1.50). Required.

.PARAMETER Latest
  Pull only the newest run folder (asks the rig which one). Default pulls every run present.

.PARAMETER RemoteDir
  Remote staging dir, relative to the Linux home or absolute. Default: fractadyne-reports.

.PARAMETER Dest
  Local destination. Default: local/linux-reports under the repo root.

.PARAMETER Port
  SSH port, if not 22.

.PARAMETER Identity
  Path to a private key file, passed to scp/ssh as -i.

.EXAMPLE
  .\scripts\pull-linux-reports.ps1 -From rhong@fractrig -Latest

.EXAMPLE
  .\scripts\pull-linux-reports.ps1 -From rhong@192.168.1.50 -Port 2222
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$From,
    [switch]$Latest,
    [string]$RemoteDir = "fractadyne-reports",
    [string]$Dest,
    [int]$Port,
    [string]$Identity
)

$ErrorActionPreference = "Stop"

# Resolve the repo root from this script's location, so -Dest defaults sensibly regardless of cwd.
$repoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
if (-not $Dest) { $Dest = Join-Path $repoRoot "local\linux-reports" }

# scp/ssh must exist (built into Windows 10/11; else install the OpenSSH Client optional feature).
if (-not (Get-Command scp -ErrorAction SilentlyContinue)) {
    throw "scp not found. Enable it: Settings > Apps > Optional features > OpenSSH Client (or 'Add-WindowsCapability -Online -Name OpenSSH.Client~~~~0.0.1.0')."
}

# Common flags for both ssh and scp (port / identity are optional).
$common = @()
if ($Identity) { $common += @("-i", $Identity) }
$sshExtra  = @(); if ($Port) { $sshExtra  += @("-p", "$Port") }   # ssh uses -p
$scpExtra  = @(); if ($Port) { $scpExtra  += @("-P", "$Port") }   # scp uses -P (capital)

New-Item -ItemType Directory -Force -Path $Dest | Out-Null
Write-Host "==> Destination: $Dest" -ForegroundColor Cyan

if ($Latest) {
    # Ask the rig for the newest run folder name, then pull just that one.
    Write-Host "==> Asking $From for the latest run under $RemoteDir/ …" -ForegroundColor Cyan
    $lsCmd = "ls -1 '$RemoteDir' 2>/dev/null | grep -E '^[0-9]{8}-[0-9]{6}$' | sort | tail -1"
    $newest = (& ssh @common @sshExtra $From $lsCmd | Select-Object -Last 1).Trim()
    if (-not $newest) {
        throw "No timestamped run folders found in ${From}:$RemoteDir/ (has linux-report.sh been run there?)."
    }
    Write-Host "==> Latest run: $newest" -ForegroundColor Cyan
    & scp @common @scpExtra -r "${From}:$RemoteDir/$newest" "$Dest"
    $landed = Join-Path $Dest $newest
}
else {
    # Pull the whole staging tree (scp copies the folder itself into Dest).
    Write-Host "==> Pulling all runs from ${From}:$RemoteDir/ …" -ForegroundColor Cyan
    & scp @common @scpExtra -r "${From}:$RemoteDir" "$Dest"
    $landed = Join-Path $Dest (Split-Path -Leaf $RemoteDir)
}

if ($LASTEXITCODE -ne 0) { throw "scp failed (exit $LASTEXITCODE). Check -From, connectivity, and auth." }

Write-Host "==> Done." -ForegroundColor Green
if (Test-Path $landed) {
    Write-Host "Landed under: $landed"
    Get-ChildItem -Recurse -File $landed |
        Select-Object @{n = "file"; e = { Resolve-Path -Relative $_.FullName } },
                      @{n = "KB";   e = { [math]::Round($_.Length / 1KB, 1) } } |
        Format-Table -AutoSize
}
