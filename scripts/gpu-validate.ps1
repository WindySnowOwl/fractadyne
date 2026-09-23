# gpu-validate.ps1 - run the whole hardware-validation battery on one machine/GPU and leave a
# single bundle to send back. The Windows half of the B6 "validate on real hardware across the
# swappable GPUs" item; scripts/gpu-validate.sh is the Linux mirror (same steps, same layout).
#
#   .\gpu-validate.ps1 -Label rx6800xt-windows
#   .\gpu-validate.ps1 -Label rtx3070-win -Quick        # skip the two long steps (~15 min -> ~3)
#   .\gpu-validate.ps1 -Label gtx1060 -Backend vulkan   # pin one backend instead of the default
#
# Why a script rather than "run these six commands": the steps must be run the SAME way on every
# card or the results are not comparable, several are easy to run subtly wrong (wrong working
# directory, inherited settings, UTF-16 redirection), and the interesting output is spread across
# stdout, the app log and the crash folder. This collects all of it.
#
# HERMETIC BY DESIGN: everything runs against a private config directory created inside the
# bundle, so (a) your own session/settings/bookmarks are never touched, and (b) every machine
# renders with identical settings, which is the whole point of a cross-GPU comparison. (Learned
# the hard way: the F3 corpus check inherits the developer's live session, and its baselines
# drifted into meaninglessness because of it.)
#
# LOCAL BY DESIGN: the battery runs in a folder on this machine's own disk (%TEMP%) and the
# finished bundle is copied to -Out at the end. Every step logs into the bundle's config folder,
# and the app writes its frame record (frames.bin) on the UI thread every frame - so a bundle
# built ON a network share measures the share. The RX 6800 XT's beta.113 run did exactly that:
# its recordtest read a network round trip as the frame record's cost and failed.
#
# NOTE: kept deliberately ASCII-only. Windows PowerShell 5.1 reads .ps1 files as ANSI unless they
# carry a UTF-8 BOM, so any em-dash or curly quote in here becomes a parse error on a stranger's
# machine. Do not "improve" the punctuation.
#
# Produces  validate-<label>-<timestamp>/
#   summary.txt          step | exit code | duration - read this first
#   system.txt           OS, CPU, RAM, GPU + driver version, app version
#   adapter.txt          the adapter + capability line the app itself resolved (the B6 ask)
#   00-build-id.txt      is this binary the one the share published? (vs BUILD-ID.txt)
#   01-gputest.txt       df32/floatexp primitives vs CPU oracles, swept over every backend
#   02-selftest.txt      full suite + 17 goldens
#   03-live-res.txt      --selftest-filter live-res - the settled-resolution invariant
#   04-bench-matrix.txt  22-segment perf + determinism vs the blessed baseline
#   05-livetest.txt      live-vs-offline truth at every tour hold       (skipped by -Quick)
#   uitest-*/            25-step UI + live-render screenshot bundle     (skipped by -Quick)
#   07-recordtest.txt    the per-frame record's own gate (~20 s)
#   app.log              the app's own log across all steps
#   frames/              each step's frame record (<step>.frames.bin, <step>.frames.jsonl) -
#                        read with scripts/framelog.py summarize
#   crash/               any crash reports produced during the run, with their -frames.jsonl
#   ...and a .zip of the whole thing. Both are built under %TEMP%\fractadyne-validate\ and then
#   copied to -Out (left there, and said so, if the copy fails).

[CmdletBinding()]
param(
    # Short machine/GPU identifier used in the bundle name, e.g. "rx6800xt-windows".
    [Parameter(Mandatory = $true)][string]$Label,
    # Skip the two long steps (livetest ~13 min, uitest ~2 min).
    [switch]$Quick,
    # Force one wgpu backend (vulkan | dx12 | gl). Default: whatever the app picks.
    [string]$Backend = "",
    # Where to deliver the bundle when it is done. Default: the share if reachable, else beside
    # this script. The battery itself always runs on this machine's disk (see LOCAL BY DESIGN).
    [string]$Out = "",
    # The BUILD-ID.txt to check the binary against (step 00). Default: searched for beside this
    # script and in the share's builds\<tag>\ folder.
    [string]$BuildId = ""
)

# Deliberately NOT "Stop". The app writes its startup banner to stderr, and with Stop in force
# any `2>&1` from a native command is promoted to a terminating error - which aborts the battery
# on a perfectly healthy run. Failures here are handled explicitly, via exit codes per step.
$ErrorActionPreference = "Continue"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path

# --- locate the binary: beside the script (extracted zip) or in a repo checkout ---------------
$exe = $null
foreach ($c in @(
        (Join-Path $root "fractadyne.exe"),
        (Join-Path $root "..\fractadyne.exe"),
        (Join-Path $root "..\target\release\fractadyne.exe"))) {
    if (Test-Path $c) { $exe = (Resolve-Path $c).Path; break }
}
if (-not $exe) { throw "fractadyne.exe not found next to this script or in ..\target\release\" }

# --- bundle location --------------------------------------------------------------------------
if (-not $Out) {
    $share = "\\vger\share\Fractadyne"
    $Out = if (Test-Path $share) { $share } else { $root }
}
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$name = "validate-$Label-$stamp"
# Built locally, delivered at the end (LOCAL BY DESIGN, above).
$work = Join-Path ([IO.Path]::GetTempPath()) "fractadyne-validate"
$dir = Join-Path $work $name
# [IO.Path]::Combine, not Join-Path: Join-Path refuses a drive this machine does not have.
$final = [IO.Path]::Combine($Out, $name)
$cfg = Join-Path $dir "config"
New-Item -ItemType Directory -Force -Path $cfg | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $dir "crash") | Out-Null

# A share, by name (\\host\...) or by a mapped drive letter.
function Test-NetworkPath([string]$p) {
    if ($p.StartsWith("\\") -or $p.StartsWith("//")) { return $true }
    try { return ([IO.DriveInfo]::new([IO.Path]::GetPathRoot($p))).DriveType -eq [IO.DriveType]::Network }
    catch { return $false }
}
$workNote = if (Test-NetworkPath $dir) { "WARNING: this is a network location - timings will measure it" } else { "this machine's disk" }

Write-Host "Fractadyne hardware validation" -ForegroundColor Cyan
Write-Host "  binary : $exe"
Write-Host "  runs in: $dir ($workNote)"
Write-Host "  bundle : $final (copied there when the battery is done)"
if ($Backend) { Write-Host "  backend: $Backend (pinned)" }
Write-Host ""

# Private config dir: hermetic, and leaves the user's real settings untouched.
$env:FRACTADYNE_CONFIG_DIR = $cfg
if ($Backend) { $env:WGPU_BACKEND = $Backend }

# --- system inventory -------------------------------------------------------------------------
$sys = Join-Path $dir "system.txt"
$lines = @("Fractadyne validation bundle - $Label - $stamp", "")
try {
    $os = Get-CimInstance Win32_OperatingSystem
    $cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
    $lines += "OS      : $($os.Caption) $($os.Version)"
    $lines += "CPU     : $($cpu.Name) ($($cpu.NumberOfCores)C/$($cpu.NumberOfLogicalProcessors)T)"
    $lines += "RAM     : $([math]::Round($os.TotalVisibleMemorySize / 1MB, 1)) GB"
    foreach ($g in Get-CimInstance Win32_VideoController) {
        $lines += "GPU     : $($g.Name) - driver $($g.DriverVersion) ($($g.DriverDate))"
    }
} catch {
    $lines += "system inventory failed: $_"
}
$appVersion = ((& $exe --version 2>$null | Where-Object { $_ -match '^fractadyne ' } | Select-Object -Last 1) -replace '\s+$', '')
$lines += "App     : " + $appVersion
if ($Backend) { $lines += "Backend : pinned to $Backend" }
$lines | Out-File $sys -Encoding utf8

# --- the battery ------------------------------------------------------------------------------
# Each step: run, capture stdout+stderr to its own UTF-8 file (never '>', which writes UTF-16),
# record exit code + duration, and KEEP GOING on failure. A failing step is data, not a reason to
# abandon the rest: a card can fail the goldens and still pass the live-resolution check, and
# knowing both is the point.
$results = @()
function Invoke-Step {
    param([string]$Name, [string]$File, [string[]]$Arguments, [string]$Why)
    Write-Host ("-> {0}" -f $Name) -ForegroundColor Yellow
    if ($Why) { Write-Host ("   {0}" -f $Why) -ForegroundColor DarkGray }
    $path = Join-Path $dir $File
    $errPath = "$path.err"
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $stepStart = Get-Date
    $code = 0
    # Start-Process with file redirection, NOT `& $exe ... 2>&1 | Out-File`. Piping a native
    # command's stderr through PowerShell wraps every line in an ErrorRecord, so the captured log
    # ends up interleaved with "NativeCommandError" blocks and script line numbers - noise in the
    # one artifact a tester is asked to read. Redirecting at the process level keeps the app's own
    # output verbatim. (stdout and stderr cannot share one file here, so they are merged after.)
    try {
        $p = Start-Process -FilePath $exe -ArgumentList $Arguments -NoNewWindow -Wait -PassThru `
            -RedirectStandardOutput $path -RedirectStandardError $errPath
        $code = $p.ExitCode
    }
    catch {
        "$_" | Out-File $path -Encoding utf8 -Append
        $code = 1
    }
    # Fold stderr in after stdout so the diagnostics/banner lines are present but not interleaved.
    if ((Test-Path $errPath) -and (Get-Item $errPath).Length -gt 0) {
        # -Encoding utf8 on the READ too: the app emits UTF-8, and re-reading it as ANSI before
        # writing it back out double-encodes every non-ASCII character (the selftest's em-dashes
        # came through as mojibake before this).
        "", "--- stderr ---" | Out-File $path -Encoding utf8 -Append
        Get-Content $errPath -Encoding utf8 | Out-File $path -Encoding utf8 -Append
    }
    Remove-Item $errPath -ErrorAction SilentlyContinue
    # The step's own frame record. Harvested NOW, per step: every launch truncates frames.bin and
    # rotates frames.jsonl, so collecting once at the end would keep only the last step's. Only a
    # file WRITTEN during this step is taken - a step that records no frames (--gputest) would
    # otherwise hand in the previous step's under its own name. (Not deleted beforehand: the next
    # launch's unclean-exit report reads the previous frames.bin.)
    $frames = Join-Path $dir "frames"
    New-Item -ItemType Directory -Force -Path $frames | Out-Null
    $base = [IO.Path]::GetFileNameWithoutExtension($File)
    foreach ($f in @("frames.bin", "frames.jsonl")) {
        $src = Join-Path (Join-Path $cfg "logs") $f
        if ((Test-Path $src) -and ((Get-Item $src).LastWriteTime -ge $stepStart)) {
            Copy-Item $src (Join-Path $frames "$base.$f") -Force
        }
    }
    $sw.Stop()
    $secs = [math]::Round($sw.Elapsed.TotalSeconds, 1)
    $script:results += [pscustomobject]@{ Step = $Name; Exit = $code; Seconds = $secs; File = $File }
    $colour = if ($code -eq 0) { "Green" } else { "Red" }
    Write-Host ("   exit {0} in {1}s" -f $code, $secs) -ForegroundColor $colour
}

# --- step 00: WHICH binary is this? ------------------------------------------------------------
# Every result below is only as good as knowing which build produced it: a 2026-08-16 run measured
# the previous binary and nothing could tell. publish-share.ps1 writes BUILD-ID.txt into the share
# folder a package came from; this checks the binary under test against it.
#   exit 0  the binary is one the share published (its --version is an `exe:` line there), or a
#           build of the published source (g<commit>-archive, the Linux tarball route)
#   exit 1  BUILD-ID.txt was found and this binary is NOT in it: stale or foreign. Every other
#           result in this bundle is then unattributed.
#   exit 2  no BUILD-ID.txt found: cannot verify. Deliberately NOT exit 0 - an unverifiable
#           identity must not read as a verified one.
$idFile = Join-Path $dir "00-build-id.txt"
$idSw = [Diagnostics.Stopwatch]::StartNew()
$idOut = @("binary  : $exe", "version : $appVersion")
$tag = if ($appVersion -match '^fractadyne (\S+) ') { "v" + $Matches[1] } else { "" }
$candidates = @()
if ($BuildId) { $candidates += $BuildId }
$candidates += (Join-Path $root "BUILD-ID.txt"), (Join-Path $root "..\BUILD-ID.txt"), (Join-Path $root "..\..\BUILD-ID.txt")
if ($tag) {
    # [IO.Path]::Combine, not Join-Path: Join-Path fails on a drive this machine does not have
    # ("Cannot find drive. A drive with the name 'D' does not exist." - the RX 6800 XT box, beta.113),
    # and every candidate is only ever tested with Test-Path below.
    foreach ($s in @($Out, "\\vger\share\Fractadyne", "D:\share\Fractadyne")) {
        $candidates += [IO.Path]::Combine($s, "builds", $tag, "BUILD-ID.txt")
    }
}
$candidates = $candidates | Select-Object -Unique
$idPath = $candidates | Where-Object { Test-Path -LiteralPath $_ -ErrorAction SilentlyContinue } | Select-Object -First 1
$idCode = 2
if (-not $idPath) {
    $idOut += "verdict : CANNOT VERIFY - no BUILD-ID.txt found. Searched:"
    $idOut += ($candidates | ForEach-Object { "            $_" })
}
else {
    # .ProviderPath, not .Path: on a share .Path reads "Microsoft.PowerShell.Core\FileSystem::\\host\...".
    $idOut += "against : $((Resolve-Path -LiteralPath $idPath).ProviderPath)"
    $id = Get-Content $idPath
    $commit = ($id | Where-Object { $_ -match '^commit: ' } | Select-Object -First 1) -replace '^commit: ', ''
    $ver = $appVersion -replace '^fractadyne ', ''
    $hit = $id | Where-Object { $_ -match '^exe: ' -and ($_ -split ' \| ', 2)[1] -eq $ver } | Select-Object -First 1
    if ($hit) {
        $idCode = 0
        $idOut += "verdict : OK - this is the published " + ((($hit -replace '^exe: ', '') -split ' \| ', 2)[0])
    }
    elseif ($commit -and $ver -match '\(build \d+, g([0-9a-f]{7,40})-archive\)$' -and $commit.StartsWith($Matches[1])) {
        $idCode = 0
        $idOut += "verdict : OK - built from the published source of commit $commit"
    }
    else {
        $idCode = 1
        $idOut += "verdict : MISMATCH - this binary is not one the share published. Published:"
        $idOut += ($id | Where-Object { $_ -match '^(tag|commit|exe): ' } | ForEach-Object { "            $_" })
    }
}
$idOut | Out-File $idFile -Encoding utf8
$idSw.Stop()
$results += [pscustomobject]@{ Step = "build-id"; Exit = $idCode; Seconds = [math]::Round($idSw.Elapsed.TotalSeconds, 1); File = "00-build-id.txt" }
$idColour = if ($idCode -eq 0) { "Green" } else { "Red" }
Write-Host "-> build-id" -ForegroundColor Yellow
Write-Host ("   {0}" -f (($idOut | Where-Object { $_ -match '^verdict' }) -replace '^verdict : ', '')) -ForegroundColor $idColour

Invoke-Step "gputest" "01-gputest.txt" @("--gputest") `
    "df32/floatexp primitives vs CPU oracles, every backend"
Invoke-Step "selftest" "02-selftest.txt" @("--selftest") `
    "full suite + 17 goldens (goldens blessed on an RTX 3080; deltas elsewhere are expected)"
Invoke-Step "live-res" "03-live-res.txt" @("--selftest", "--selftest-filter", "live-res") `
    "settled-resolution invariant - the B6 core, never yet run on hardware truly lacking TIMESTAMP_QUERY"
Invoke-Step "bench-matrix" "04-bench-matrix.txt" @("--bench-matrix") `
    "22-segment perf + determinism; exit 2 signals algorithmic drift, not merely slower"

if (-not $Quick) {
    Invoke-Step "livetest" "05-livetest.txt" `
        @("--livetest", "tours/grand-tour.toml", "--size", "480x270") `
        "live view vs an offline render of the same view ON THIS MACHINE"
    Invoke-Step "uitest" "06-uitest.txt" @("--uitest", $dir) `
        "25-step UI + live-render walk with screenshots"
}
else {
    Write-Host "-> skipping livetest + uitest (-Quick)" -ForegroundColor DarkGray
}
# ~20 s, so it runs even under -Quick. It is the one step that proves the frame record works on
# THIS card and filesystem - including that it survives a process abort - which is what makes the
# next field failure diagnosable.
Invoke-Step "recordtest" "07-recordtest.txt" @("--recordtest") `
    "the per-frame record: every frame recorded, survives an abort, watchdog + blind tripwire fire"

# --- harvest the app's own evidence -------------------------------------------------------------
$log = Join-Path $cfg "logs\fractadyne.log"
if (Test-Path $log) { Copy-Item $log (Join-Path $dir "app.log") -Force }
Get-ChildItem (Join-Path $cfg "logs") -Filter "crash-*.txt" -ErrorAction SilentlyContinue |
Copy-Item -Destination (Join-Path $dir "crash") -Force
# ...and each crash's whole frame record (crash-<stamp>-frames.jsonl), and the view it died at.
Get-ChildItem (Join-Path $cfg "logs") -Filter "crash-*" -ErrorAction SilentlyContinue |
Where-Object { $_.Name -like "*-frames.jsonl" -or $_.Name -like "crash-view-*.fdn" } |
Copy-Item -Destination (Join-Path $dir "crash") -Force

# The adapter + capability line the app resolved for ITSELF - the "record adapter and resolved
# tunables per card" half of B6, taken from the app rather than from the Windows device list.
if (Test-Path (Join-Path $dir "app.log")) {
    $adapters = Select-String -Path (Join-Path $dir "app.log") `
        -Pattern 'adapter:|capability:|TIMESTAMP_QUERY' -ErrorAction SilentlyContinue |
    ForEach-Object { $_.Line.Trim() } | Select-Object -Unique
    if ($adapters) { $adapters | Out-File (Join-Path $dir "adapter.txt") -Encoding utf8 }
}

# --- summary -------------------------------------------------------------------------------------
$sum = Join-Path $dir "summary.txt"
$head = @("Fractadyne hardware validation - $Label", "$stamp", "ran in $dir ($workNote)", "")
$head | Out-File $sum -Encoding utf8
($results | Format-Table Step, Exit, Seconds, File -AutoSize | Out-String).TrimEnd() |
Out-File $sum -Encoding utf8 -Append

@"

How to read this
----------------
build-id     Read this line FIRST. 0 = the binary tested is the one the share published. 1 = it is
             not (a stale extraction, or a different build): every other result is then about an
             unknown binary. 2 = no BUILD-ID.txt was found, so nobody can say which build this is -
             pass -BuildId <path> to the one in the share's builds\<tag>\ folder.
gputest      A failing two_sum/two_prod means this stack's shader compiler folds the error-free
             transforms, so every extended-precision path silently degrades to plain f32. Known:
             all NVIDIA backends fold them; AMD Vulkan/OpenGL do not; AMD DX12 fails differently
             (fma not fused).
selftest     Should now pass on any card. The 17 goldens were blessed on an RTX 3080 and are
             compared with a wider, measured tolerance on other hardware; the path-signature
             checks likewise report cross-GPU differences rather than failing them. So a FAILURE
             here is a real signal - it is no longer expected noise. (If your build predates
             beta.94 you will instead see "goldens 0/17" and a dozen DRIFT lines, all of which
             were expected cross-vendor differences rather than defects.)
live-res     Must pass everywhere. This is the invariant that a GPU without TIMESTAMP_QUERY still
             settles at native resolution instead of being stuck at ~1/3 forever.
bench-matrix Timings vary by card and mean nothing across machines. Signature differences against
             a baseline recorded on another GPU are EXPECTED and are reported, not failed - the
             escape decisions (and the rebase/skip counts that follow) legitimately differ between
             vendors. Exit 2 means drift on the baseline's OWN card, which does matter.
livetest     Self-contained: compares the live view against an offline render on THIS machine, so
             its pass/fail is meaningful here. "drift" lines compare against an RTX 3080 baseline
             and can be ignored on other hardware; FAIL lines cannot.
uitest       Screenshots for eyeballing. The deep floatexp band is WARN-not-FAIL by design.
recordtest   Must pass everywhere: 0 pass, 1 fail, 2 VACUOUS (nothing exercised - not a pass). The
             per-frame record is what diagnoses the next crash, so a failure here matters even if
             every other step is green. The frames/ folder holds each step's own record.

Send back the whole folder (or the .zip beside it).
"@ | Out-File $sum -Encoding utf8 -Append

Remove-Item Env:\FRACTADYNE_CONFIG_DIR -ErrorAction SilentlyContinue
if ($Backend) { Remove-Item Env:\WGPU_BACKEND -ErrorAction SilentlyContinue }

# --- zip + report ----------------------------------------------------------------------------------
$zip = "$dir.zip"
try {
    Compress-Archive -Path $dir -DestinationPath $zip -Force
}
catch {
    Write-Host "  (zip failed: $_)" -ForegroundColor DarkYellow
    $zip = $null
}

# --- deliver: copy the finished bundle (and its zip) to -Out ---------------------------------------
# The local copy is removed only once the copy has succeeded; otherwise it stays, and says where.
$bundle = $dir
if ([IO.Path]::GetFullPath($Out).TrimEnd('\') -ne [IO.Path]::GetFullPath($work).TrimEnd('\')) {
    try {
        New-Item -ItemType Directory -Force -Path $Out -ErrorAction Stop | Out-Null
        Copy-Item -LiteralPath $dir -Destination $Out -Recurse -Force -ErrorAction Stop
        if ($zip) { Copy-Item -LiteralPath $zip -Destination $Out -Force -ErrorAction Stop }
        Remove-Item -LiteralPath $dir -Recurse -Force -ErrorAction SilentlyContinue
        if ($zip) { Remove-Item -LiteralPath $zip -Force -ErrorAction SilentlyContinue; $zip = "$final.zip" }
        $bundle = $final
    }
    catch {
        Write-Host "  (copy to $Out failed: $_ - the bundle is still at $dir)" -ForegroundColor DarkYellow
    }
}

Write-Host ""
Write-Host "Summary" -ForegroundColor Cyan
$results | Format-Table Step, Exit, Seconds -AutoSize
Write-Host "bundle : $bundle"
if ($zip) { Write-Host "zip    : $zip" }
Write-Host ""
Write-Host "Read summary.txt first - it explains which failures are expected off the reference card." -ForegroundColor Cyan
