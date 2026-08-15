#!/usr/bin/env pwsh
<#
  windows-build.ps1 - pull the latest Fractadyne source from GitHub and build it locally, so you
  can test a fresh commit on a Windows box WITHOUT triggering a GitHub release build.

  First run:   .\windows-build.ps1 -Deps        # install prerequisites + clone + build
  Every after: .\windows-build.ps1              # fetch latest main + rebuild
  With checks: .\windows-build.ps1 -SelfTest    # build, then run the GPU self-test
  And launch:  .\windows-build.ps1 -Run         # build, then start the app

  Options:
    -Deps           install the prerequisites: git, the Rust toolchain, and the MSVC C++ build
                    tools the Rust `msvc` toolchain links against. Git and Rust are installed with
                    the vendors' OWN unattended installers, not winget - see the note in the code
                    for why (winget stalled at "0%" twice, including from an elevated shell).
                    !!RUN THIS ONE FROM AN ELEVATED POWERSHELL: Git and the build tools install
                    machine-wide. The script checks first and tells you, rather than starting work
                    it cannot finish. Every other switch works fine unelevated.
    -SelfTest       after building, run --selftest (needs a real GPU; see the note below).
    -Run            after building, launch the app.
    -Branch NAME    build a branch or tag other than main.
    -Dir PATH       where the working copy lives (default: %USERPROFILE%\fractadyne).
    -Release        build the optimized release profile (default; matches the shipped binary).
    -DebugBuild     build the debug profile instead (faster compile, much slower renders).
                    (Not "-Debug": that name is reserved by PowerShell's common parameters.)
    -Clean          `cargo clean` first (use when a build is wedged; costs a full rebuild).
    -Force          allow the update to DISCARD local changes in the working copy (see below).
    -Yes            never prompt (unattended install).

  The repo is public, so no credentials are needed for a read-only clone/pull.

  This is the Windows counterpart of `linux-build.sh` and mirrors its flags. Two deliberate
  differences, both because Windows behaves differently:

    * The Linux script hard-resets the checkout unconditionally. This one REFUSES to discard
      uncommitted changes unless you pass -Force, because on Windows the default location is
      inside your profile and is easy to confuse with a working tree you actually edit.
    * It closes a running fractadyne.exe before building. Windows holds an executable open while
      it runs, so `cargo build` fails with "failed to remove file ...\fractadyne.exe" - a confusing
      error for what is really "the app you are rebuilding is still on screen".

  Written for Windows PowerShell 5.1 (what a fresh Windows install has) as well as PowerShell 7,
  so it avoids 7-only syntax.
#>
[CmdletBinding()]
param(
    [switch]$Deps,
    [switch]$SelfTest,
    [switch]$Run,
    [switch]$Clean,
    [switch]$Release,
    [switch]$DebugBuild,
    [switch]$Force,
    [switch]$Yes,
    [string]$Branch = 'main',
    [string]$Dir = (Join-Path $env:USERPROFILE 'fractadyne')
)

$ErrorActionPreference = 'Stop'
$RepoUrl = 'https://github.com/WindySnowOwl/fractadyne.git'
$Profile_ = if ($DebugBuild) { 'debug' } else { 'release' }

function Say($msg)  { Write-Host "==> $msg" -ForegroundColor Cyan }
function Ok($msg)   { Write-Host "    [ok] $msg" -ForegroundColor Green }
function Warn2($msg){ Write-Host "    [!]  $msg" -ForegroundColor Yellow }
function Die($msg)  { Write-Host "error: $msg" -ForegroundColor Red; exit 1 }
function Have($name){ return [bool](Get-Command $name -ErrorAction SilentlyContinue) }
function Confirm2($msg) {
    if ($Yes) { return $true }
    return ((Read-Host "$msg [y/N]") -match '^(y|yes)$')
}

function Test-Admin {
    $id = [Security.Principal.WindowsIdentity]::GetCurrent()
    return (New-Object Security.Principal.WindowsPrincipal($id)).IsInRole(
        [Security.Principal.WindowsBuiltInRole]::Administrator)
}

# !!NO WINGET ANYWHERE IN THIS SCRIPT, deliberately. Every install attempted through it stalled at
# "0%" on the machine this was written for: Git unelevated (a UAC prompt that never surfaced), Git
# again from an ELEVATED shell with --source winget --silent --disable-interactivity - flags that
# should make waiting impossible - and finally ffmpeg. Three stalls, three different packages, two
# elevation states. Whatever it waits on is invisible from here, and a bootstrap script that hangs
# with no output is worse than one that fails, because a failure tells you what to do next.
# The vendors' own unattended installers have never done this, so they are the only path used.

# Git for Windows embeds its version in the asset name, so the download URL comes from the API
# rather than being hardcoded to a version that will age out.
function Install-GitDirect {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Say "Fetching the current Git for Windows release"
    $rel = Invoke-RestMethod -Uri 'https://api.github.com/repos/git-for-windows/git/releases/latest' `
                             -Headers @{ 'User-Agent' = 'fractadyne-setup' } -TimeoutSec 90
    $asset = $rel.assets | Where-Object { $_.name -match '^Git-.*-64-bit\.exe$' } | Select-Object -First 1
    if (-not $asset) { Die "could not find a 64-bit Git installer in the latest release" }
    $mb = [math]::Round($asset.size / 1MB, 1)
    $exe = Join-Path $env:TEMP $asset.name
    Say "Downloading $($asset.name) ($mb MB) - this takes a minute and shows no progress"

    # PowerShell 5.1's Invoke-WebRequest renders a progress bar that costs FAR more than the
    # download itself (a documented ~10x slowdown on large files), which on a 62 MB installer looks
    # exactly like the stall we are here to eliminate. Silencing it is a speed fix, not cosmetic.
    $prev = $ProgressPreference
    $ProgressPreference = 'SilentlyContinue'
    try { Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $exe -TimeoutSec 1800 }
    finally { $ProgressPreference = $prev }
    if (-not (Test-Path $exe)) { Die "the Git installer did not download" }
    Ok "downloaded $([math]::Round((Get-Item $exe).Length / 1MB, 1)) MB"

    Say "Running the installer silently (no dialogs; it does not report progress either)"
    $p = Start-Process -FilePath $exe -ArgumentList '/VERYSILENT', '/NORESTART', '/NOCANCEL', '/SP-',
                                                    '/CLOSEAPPLICATIONS', '/RESTARTAPPLICATIONS' -PassThru
    if (-not $p.WaitForExit(900 * 1000)) {
        try { $p.Kill() } catch { }
        Die "the Git installer did not finish in 15 minutes - install it by hand from https://git-scm.com/download/win"
    }
    if ($p.ExitCode -ne 0) { Die "the Git installer exited $($p.ExitCode)" }
    Remove-Item $exe -ErrorAction SilentlyContinue
}

# Native commands (git, cargo) signal failure through the exit code, not an exception, so every
# call goes through here - a silent non-zero is how a "successful" build ends up not existing.
function Invoke-Native {
    param([Parameter(Mandatory)][string]$What, [Parameter(Mandatory)][scriptblock]$Cmd)
    & $Cmd
    if ($LASTEXITCODE -ne 0) { Die "$What failed (exit $LASTEXITCODE)" }
}

# rustup and the Git installer put their binaries in places that are not on PATH in an
# already-open session. Add them for THIS process so a first run continues into the build instead
# of telling you to open a new terminal.
function Add-ToolPaths {
    # Installers write PATH to the registry; this process still holds the value it started with.
    # Re-read both scopes wholesale rather than diffing entry by entry (a path containing `[`
    # breaks a `-like` comparison, and duplicates in a process-local PATH are harmless).
    $machine = [Environment]::GetEnvironmentVariable('Path', 'Machine')
    $user    = [Environment]::GetEnvironmentVariable('Path', 'User')
    $env:Path = (@($machine, $user, $env:Path) | Where-Object { $_ }) -join ';'
    # rustup's bin dir is not always in that registry value yet on a first install.
    foreach ($p in @(
        (Join-Path $env:USERPROFILE '.cargo\bin'),
        (Join-Path $env:ProgramFiles 'Git\cmd'),
        (Join-Path $env:LOCALAPPDATA 'Programs\Git\cmd')
    )) {
        if (Test-Path $p) { $env:Path = "$p;$env:Path" }
    }
}

Write-Host "Fractadyne - Windows source build" -ForegroundColor White
Add-ToolPaths

# ---------------------------------------------------------------------- prerequisites ----
if ($Deps) {
    Say "Installing prerequisites"

    # Git, Rust and the MSVC build tools all install MACHINE-WIDE, which needs administrator
    # rights. Without them the installers wait on an elevation prompt that may never appear in this
    # window, and the install sits at "0%" indefinitely with no error - so refuse up front and say
    # what to do, rather than letting the script appear to work and then stop forever.
    if (-not (Test-Admin)) {
        Warn2 "-Deps installs machine-wide software (Git, the MSVC build tools) and this window"
        Warn2 "is NOT elevated. Unelevated, those installers wait on a UAC prompt that may never"
        Warn2 "surface here, which looks exactly like a hang."
        Write-Host ""
        Write-Host "  Start an elevated PowerShell and run:" -ForegroundColor White
        Write-Host "    cd $PSScriptRoot" -ForegroundColor White
        Write-Host "    .\windows-build.ps1 -Deps" -ForegroundColor White
        Write-Host ""
        Write-Host "  (Right-click PowerShell -> Run as administrator. Everything EXCEPT -Deps"
        Write-Host "   works fine unelevated, so once the tools are installed you never need this"
        Write-Host "   again.)"
        exit 1
    }

    # !!WINGET IS NOT USED FOR THE TWO TOOLS THAT MATTER, and that is the fix rather than an
    # omission. It stalled at "0%" installing Git twice - once unelevated (a UAC prompt that never
    # surfaced) and again from an ELEVATED shell even with --source winget --silent
    # --disable-interactivity, which should have made waiting impossible. Whatever it is waiting on
    # is not something this script can see or control, and the vendors both ship unattended
    # installers that need no package manager at all. Fewer moving parts, and each one observable.
    if (Have 'git') { Ok "git present" }
    else {
        Install-GitDirect
        Add-ToolPaths
        if (-not (Have 'git')) { Die "Git installed but is not on PATH - open a new terminal and re-run." }
        Ok "git installed"
    }

    if (Have 'rustup') { Ok "rustup present" }
    else {
        Say "Installing the Rust toolchain (rustup-init, the vendor's own unattended installer)"
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        $init = Join-Path $env:TEMP 'rustup-init.exe'
        Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile $init -TimeoutSec 600
        & $init -y --default-toolchain stable --profile default
        Remove-Item $init -ErrorAction SilentlyContinue
        Add-ToolPaths
        if (-not (Have 'rustup')) { Die "rustup installed but is not on PATH - open a new terminal and re-run." }
        Ok "rustup installed"
    }

    # ffmpeg is OPTIONAL - it is used only by `--render-tour --mp4`, which shells out to it after
    # all the frames exist. This script does NOT install it: every winget install attempted here
    # stalled at "0%", elevated or not, and ffmpeg ships as a zip rather than an installer, so
    # fetching it would mean unpacking it somewhere and editing PATH - too invasive for a
    # convenience nobody has asked for yet. A one-line pointer costs nothing and cannot hang.
    if (Have 'ffmpeg') { Ok "ffmpeg present (tour --mp4 assembly)" }
    else {
        Warn2 "ffmpeg not found - only needed for `--render-tour --mp4`; frames render without it."
        Warn2 "  Get it from https://www.gyan.dev/ffmpeg/builds/ and put ffmpeg.exe on PATH."
    }
}

if (-not (Have 'git')) {
    Die "git not found - run with -Deps, or install from https://git-scm.com/download/win"
}
if (-not (Have 'cargo')) {
    Die "cargo not found - run with -Deps, or install Rust from https://rustup.rs (then open a new terminal)"
}

# ------------------------------------------------------------------ fetch / update ----
if (Test-Path (Join-Path $Dir '.git')) {
    Say "Updating $Dir (branch $Branch)"
    # A dirty tree means this is somebody's working copy, not a throwaway test checkout. The
    # update below is a hard reset, so stop rather than destroy uncommitted work.
    $dirty = & git -C $Dir status --porcelain
    if ($LASTEXITCODE -ne 0) { Die "git status failed in $Dir" }
    if ($dirty -and -not $Force) {
        Write-Host ($dirty | Select-Object -First 10 | Out-String)
        Die ("$Dir has uncommitted changes and updating it would DISCARD them. " +
             "Commit/stash them, pass -Force to throw them away, or use -Dir <other path>.")
    }
    Invoke-Native "git fetch" { git -C $Dir fetch --prune origin }
    Invoke-Native "git checkout $Branch" { git -C $Dir checkout $Branch }
    # Hard-sync to origin so a local build never diverges from what was pushed.
    Invoke-Native "git reset" { git -C $Dir reset --hard "origin/$Branch" }
}
else {
    if (Test-Path $Dir) {
        $entries = Get-ChildItem -LiteralPath $Dir -Force -ErrorAction SilentlyContinue
        if ($entries) { Die "$Dir exists, is not a git checkout, and is not empty. Use -Dir <other path>." }
    }
    Say "Cloning $RepoUrl -> $Dir"
    Invoke-Native "git clone" { git clone --branch $Branch $RepoUrl $Dir }
}

$commit  = (& git -C $Dir rev-parse --short HEAD).Trim()
$verLine = Select-String -LiteralPath (Join-Path $Dir 'Cargo.toml') -Pattern '^version = "(.*)"' |
           Select-Object -First 1
$version = if ($verLine) { $verLine.Matches[0].Groups[1].Value } else { 'unknown' }
Ok "At $commit - version $version"

# ---------------------------------------------------------------------- toolchain ----
# The repo's own bootstrap owns this: it installs/defaults the stable MSVC toolchain and checks
# for the C++ build tools the linker needs. Calling it keeps ONE description of the toolchain
# requirement instead of two that can drift apart.
$setup = Join-Path $Dir 'scripts\setup.ps1'
if (Test-Path $setup) {
    Say "Checking the Rust/MSVC toolchain (scripts\setup.ps1)"
    # A HASHTABLE splat, not an array: array splatting passes its elements POSITIONALLY, so
    # `@('-SkipBuild')` reaches a [CmdletBinding()] script as a positional argument and it errors
    # with "A positional parameter cannot be found that accepts argument '-SkipBuild'".
    $setupArgs = @{ SkipBuild = $true }
    if ($Yes -or $Deps) { $setupArgs['Yes'] = $true }
    # A PowerShell script only sets $LASTEXITCODE when it calls `exit`, so clear it first -
    # otherwise a success here would be judged by the previous native command's code.
    $global:LASTEXITCODE = 0
    & $setup @setupArgs
    if ($LASTEXITCODE -ne 0) { Die "toolchain setup reported a problem (see above)" }
    Add-ToolPaths
}

# ------------------------------------------------------------------------- build ----
Push-Location $Dir
try {
    if ($Clean) { Say "cargo clean"; Invoke-Native "cargo clean" { cargo clean } }

    # Windows keeps a running executable locked, so `cargo build` cannot replace it: the error is
    # "failed to remove file ...\fractadyne.exe", which reads like a permissions problem and is
    # really "the app is still open". Close it rather than failing three minutes into a build.
    $running = Get-Process fractadyne -ErrorAction SilentlyContinue
    if ($running) {
        Warn2 "fractadyne.exe is running and would lock the output binary - closing it."
        $running | Stop-Process -Force
        Start-Sleep -Milliseconds 300
    }

    if ($Profile_ -eq 'release') {
        Say "Building release binary (the first build fetches wgpu/egui - several minutes)"
        Invoke-Native "cargo build" { cargo build --release --bin fractadyne }
        $bin = Join-Path $Dir 'target\release\fractadyne.exe'
    }
    else {
        Say "Building debug binary"
        Invoke-Native "cargo build" { cargo build --bin fractadyne }
        $bin = Join-Path $Dir 'target\debug\fractadyne.exe'
    }
    if (-not (Test-Path $bin)) { Die "build reported success but $bin is missing" }
    Ok "Built: $bin"

    # Cheapest possible proof the binary actually runs on this machine (a link that succeeded can
    # still produce something that dies on a missing runtime DLL).
    & $bin --version
    if ($LASTEXITCODE -ne 0) { Die "the built binary did not run (--version exited $LASTEXITCODE)" }
}
finally {
    Pop-Location
}

# --------------------------------------------------------------------- self-test ----
# Opens a real GPU device, so it needs a desktop session - over RDP or as a service it may find
# no adapter. The goldens are blessed on the developer's RTX 3080: an exact match confirms the
# GPU produces identical output, and small per-pixel deltas on other hardware are expected FP32
# variance rather than failures (the suite applies a cross-GPU tolerance when the blessing marker
# says the card differs).
if ($SelfTest) {
    Say "Running --selftest"
    & $bin --selftest
    if ($LASTEXITCODE -ne 0) {
        Die "the self-test reported failures (see the report path printed above)"
    }
    Ok "Self-test passed"
}

if ($Run) {
    Say "Launching"
    & $bin
    exit $LASTEXITCODE
}

Write-Host ""
Ok "Done. Run it with:  $bin"
Write-Host "     Useful next:  $bin --selftest        # GPU validation suite" -ForegroundColor DarkGray
Write-Host "                   $bin --help            # every CLI mode" -ForegroundColor DarkGray
