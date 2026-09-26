# field-agent.ps1 - run Fractadyne test jobs on a test machine, on request, through the share.
#
# Installed and started by field-agent-setup.ps1 (run THAT, not this). It runs in the logged-on
# user's own desktop session, hidden, and every 30 s looks in <share>\field\requests\ for a job.
#
# WHY SHARE POLLING: the test machine accepts no inbound connection of any kind. It only reads and
# writes the share it already uses for builds, and the dev box asks for a run by dropping a small
# JSON file there (scripts/field-request.ps1). The results come back the same way.
#
# WHAT IT WILL RUN - and nothing else:
#   battery   gpu-validate.ps1 from a PUBLISHED package
#   harness   fractadyne.exe from a PUBLISHED package, with flags from a fixed allow-list
#             (see $Allowed below); optionally several builds, interleaved A,B,A,B for an A/B;
#             optionally at a given VIEW (staged as the run's session - see New-ViewSession)
#   events    read-only: the Windows event log's display-driver resets and crash reports
# A "published package" is a zip in <share>\builds\<tag>\ whose sha256 matches that folder's
# BUILD-ID.txt. It is copied to this machine, checked, extracted under %LOCALAPPDATA%, and run from
# there - never straight off the share. Anything else in a request is refused, with the reason.
#
# WHEN IT RUNS - only when the machine is free:
#   * the workstation is unlocked (a locked session's windows are not on screen, so their timings
#     would not be a user's), and
#   * nobody has touched the keyboard or mouse for IdleMinutes (default 5; setup -IdleMinutes), and
#   * Fractadyne is not already open (it would share the GPU with the test, and it is yours), and
#   * nothing is paused: <share>\field\PAUSE, or %LOCALAPPDATA%\Fractadyne-field\PAUSE.
# Otherwise the job waits, and the heartbeat (<share>\field\agent\<COMPUTER>.json) says why. While a
# job runs the display is kept awake, and the result records whether anyone used the machine
# during it (that would make its timings suspect).
#
# It never kills a process it did not start. A job that overruns its timeout has ITS process tree
# stopped, by process id.
#
# UPDATES itself (v3+): a higher $AgentVersion in <share>\field\setup\field-agent.ps1 is copied over
# this file between jobs, and the scheduled task restarts it (see Test-SelfUpdate).
#
# ASCII-only on purpose: Windows PowerShell 5.1 reads a BOM-less .ps1 as ANSI.

[CmdletBinding()]
param(
    # Poll once and exit (for testing by hand).
    [switch]$Once,
    # Testing only: skip the idle and lock checks. Never set by the scheduled task.
    [switch]$IgnoreIdle,
    # Override the installed config (share root, idle minutes).
    [string]$Share = "",
    [int]$IdleMinutes = -1
)

$ErrorActionPreference = "Continue"
$AgentVersion = 5   # 2: screens; "used during run" only with the idle wait on. 3: request "view"; self-update. 4: request "env" (instruments); --soak-depth session. 5: --zoomtest-location session, --zoomtest-taps, --zoomtest-hold, --window (W9 motion rung)
$PollSeconds = 30
$Home_ = Split-Path -Parent $MyInvocation.MyCommand.Path
$Cache = Join-Path $Home_ "cache"
$Work = Join-Path $Home_ "work"
$LocalLog = Join-Path $Home_ "agent.log"

# --- config -------------------------------------------------------------------------------------
$cfgPath = Join-Path $Home_ "config.json"
$cfg = $null
if (Test-Path $cfgPath) { $cfg = Get-Content $cfgPath -Raw | ConvertFrom-Json }
if (-not $Share) { $Share = if ($cfg -and $cfg.share) { [string]$cfg.share } else { [string]$env:FRACTADYNE_SHARE } }
if (-not $Share) { Write-Host "no share configured: run field-agent-setup.ps1, or pass -Share"; exit 1 }
if ($IdleMinutes -lt 0) { $IdleMinutes = if ($cfg -and ($cfg.PSObject.Properties.Name -contains "idle_minutes")) { [int]$cfg.idle_minutes } else { 5 } }
$Field = Join-Path $Share "field"
$ReqDir = Join-Path $Field "requests"
$ResDir = Join-Path $Field "results"
$AgentDir = Join-Path $Field "agent"
$Computer = $env:COMPUTERNAME
# Every run gets its own config dir, and its logs must land there to be collected.
Remove-Item Env:FRACTADYNE_LOG_DIR -ErrorAction SilentlyContinue

# --- one agent per user session ---------------------------------------------------------------
$mutex = New-Object System.Threading.Mutex($false, "Local\FractadyneFieldAgent")
if (-not $mutex.WaitOne(0)) { Write-Host "another field agent is already running in this session"; exit 0 }

Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public static class FdField {
    [StructLayout(LayoutKind.Sequential)]
    struct LASTINPUTINFO { public uint cbSize; public uint dwTime; }
    [DllImport("user32.dll")] static extern bool GetLastInputInfo(ref LASTINPUTINFO plii);
    [DllImport("kernel32.dll")] static extern uint SetThreadExecutionState(uint esFlags);
    public static double IdleSeconds() {
        LASTINPUTINFO l = new LASTINPUTINFO();
        l.cbSize = (uint)Marshal.SizeOf(l);
        if (!GetLastInputInfo(ref l)) return -1;
        return unchecked((uint)Environment.TickCount - l.dwTime) / 1000.0;
    }
    // ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED, and back.
    public static void KeepAwake() { SetThreadExecutionState(0x80000003u); }
    public static void AllowSleep() { SetThreadExecutionState(0x80000000u); }
}
public class FieldRefused : Exception { public FieldRefused(string m) : base(m) {} }
"@

# --- small helpers ------------------------------------------------------------------------------
$Utf8 = New-Object System.Text.UTF8Encoding($false)

function Write-Log([string]$msg) {
    try {
        if ((Test-Path $LocalLog) -and (Get-Item $LocalLog).Length -gt 1MB) { Move-Item $LocalLog "$LocalLog.1" -Force }
        Add-Content -Path $LocalLog -Value ("{0:yyyy-MM-dd HH:mm:ss} {1}" -f (Get-Date), $msg)
    }
    catch {}
}

# Write JSON without a BOM, via a temp file and a rename, so a reader never sees half a file.
function Write-JsonFile([string]$path, $obj) {
    $tmp = "$path.tmp-$PID"
    [IO.File]::WriteAllText($tmp, ($obj | ConvertTo-Json -Depth 8), $Utf8)
    Move-Item -LiteralPath $tmp -Destination $path -Force
}

function Get-UtcStamp { (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ") }

# A request this agent will not run (state "rejected"), as opposed to one that failed while running.
function Stop-Refused([string]$why) { throw (New-Object FieldRefused $why) }

# Start-Process joins its arguments with spaces and quotes nothing.
function Format-Arg([string]$a) { if ($a -match '\s') { return "`"$a`"" } return $a }

function Test-Locked { [bool](Get-Process -Name LogonUI -ErrorAction SilentlyContinue) }

# The screens this session has, as Windows reports them (logical pixels). Behind a KVM switch that
# does not emulate the monitor, switching away can leave the session with no real display - and a
# test then renders to a desktop no one sees, with timings that are not a user's.
function Get-Screens {
    try {
        Add-Type -AssemblyName System.Windows.Forms -ErrorAction Stop
        return @([System.Windows.Forms.Screen]::AllScreens | ForEach-Object {
                "{0} {1}x{2}{3}" -f $_.DeviceName, $_.Bounds.Width, $_.Bounds.Height, $(if ($_.Primary) { " primary" } else { "" }) })
    }
    catch { return @("unknown: $($_.Exception.Message)") }
}

function Write-Heartbeat([string]$state, [string]$detail, [string]$current = "") {
    try {
        if (-not (Test-Path $AgentDir)) { New-Item -ItemType Directory -Force -Path $AgentDir | Out-Null }
        $pending = @(Get-ChildItem -LiteralPath $ReqDir -Filter "*.json" -ErrorAction SilentlyContinue).Count
        Write-JsonFile (Join-Path $AgentDir "$Computer.json") ([ordered]@{
                computer      = $Computer
                agent_version = $AgentVersion
                pid           = $PID
                last_poll_utc = Get-UtcStamp
                poll_seconds  = $PollSeconds
                state         = $state
                detail        = $detail
                current       = $current
                pending       = $pending
                idle_seconds  = [math]::Round([FdField]::IdleSeconds())
                idle_required = $IdleMinutes * 60
                locked        = (Test-Locked)
                screens       = @(Get-Screens)
            })
    }
    catch { Write-Log "heartbeat failed: $_" }
}

# Stop a process and everything it started - only ever called on a process this agent started.
function Stop-Tree([int]$Id) {
    foreach ($k in @(Get-CimInstance Win32_Process -Filter "ParentProcessId=$Id" -ErrorAction SilentlyContinue)) {
        Stop-Tree ([int]$k.ProcessId)
    }
    Stop-Process -Id $Id -Force -ErrorAction SilentlyContinue
}

# Start a process, wait up to $TimeoutMin, return @{ exit; seconds; timed_out }.
function Invoke-Bounded([string]$File, [string[]]$Arguments, [string]$Cwd, [string]$Out, [string]$Err, [int]$TimeoutMin, [switch]$Hidden) {
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $sp = @{ FilePath = $File; ArgumentList = $Arguments; WorkingDirectory = $Cwd; PassThru = $true
        RedirectStandardOutput = $Out; RedirectStandardError = $Err }
    # fractadyne.exe is a CONSOLE-subsystem program: it must share this (hidden) console, or it
    # opens a console window of its own on the tester's screen. Its render window is unaffected.
    if ($Hidden) { $sp.WindowStyle = "Hidden" } else { $sp.NoNewWindow = $true }
    $p = Start-Process @sp
    $null = $p.Handle   # so ExitCode is populated after a timed wait
    $timedOut = -not $p.WaitForExit($TimeoutMin * 60 * 1000)
    if ($timedOut) { Stop-Tree $p.Id; $p.WaitForExit(10000) | Out-Null }
    $sw.Stop()
    $code = if ($timedOut) { $null } else { $p.ExitCode }
    return @{ exit = $code; seconds = [math]::Round($sw.Elapsed.TotalSeconds, 1); timed_out = $timedOut }
}

# --- builds: resolve, fetch, verify, extract --------------------------------------------------------
function Get-BuildId([string]$tag) {
    $p = Join-Path (Join-Path (Join-Path $Share "builds") $tag) "BUILD-ID.txt"
    if (-not (Test-Path -LiteralPath $p)) { throw "no BUILD-ID.txt for $tag on the share" }
    $lines = Get-Content -LiteralPath $p
    $sha = @{}
    foreach ($l in $lines) { if ($l -match '^sha256: ([0-9a-f]{64})  (\S+)$') { $sha[$Matches[2]] = $Matches[1] } }
    $pub = ($lines | Where-Object { $_ -match '^published_utc: ' } | Select-Object -First 1) -replace '^published_utc: ', ''
    return @{ path = $p; sha = $sha; published = $pub }
}

function Resolve-Tag([string]$want) {
    if ($want -and $want -ne "latest") {
        if ($want -notmatch '^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$') { Stop-Refused "bad build tag: $want" }
        return $want
    }
    $best = $null; $bestPub = ""
    foreach ($d in @(Get-ChildItem -LiteralPath (Join-Path $Share "builds") -Directory -ErrorAction SilentlyContinue)) {
        if ($d.Name -notmatch '^v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?$') { continue }
        try { $id = Get-BuildId $d.Name } catch { continue }
        if ($id.published -gt $bestPub) { $best = $d.Name; $bestPub = $id.published }
    }
    if (-not $best) { throw "no published build found under $Share\builds" }
    return $best
}

# Returns the extracted package's root folder (the one holding fractadyne.exe).
function Get-Package([string]$tag, [string]$package) {
    $zipName = if ($package -eq "accelerated") { "fractadyne-$tag-windows-x64-accelerated.zip" } else { "fractadyne-$tag-windows-x64.zip" }
    $dest = Join-Path (Join-Path $Cache $tag) $package
    $root = if ($package -eq "accelerated") { $dest } else { Join-Path $dest "fractadyne-$tag-windows-x64" }
    $marker = Join-Path $dest ".verified"
    $id = Get-BuildId $tag
    if (-not $id.sha.ContainsKey($zipName)) { throw "BUILD-ID.txt for $tag lists no $zipName" }
    if ((Test-Path $marker) -and ((Get-Content $marker -Raw).Trim() -eq $id.sha[$zipName]) -and (Test-Path (Join-Path $root "fractadyne.exe"))) {
        return $root
    }
    Write-Log "fetching $zipName"
    $src = Join-Path (Join-Path (Join-Path $Share "builds") $tag) $zipName
    New-Item -ItemType Directory -Force -Path (Join-Path $Cache $tag) | Out-Null
    $zip = Join-Path (Join-Path $Cache $tag) $zipName
    Copy-Item -LiteralPath $src -Destination $zip -Force -ErrorAction Stop
    $got = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLower()
    if ($got -ne $id.sha[$zipName]) {
        Remove-Item -LiteralPath $zip -Force -ErrorAction SilentlyContinue
        throw "$zipName sha256 $got does not match BUILD-ID.txt ($($id.sha[$zipName]))"
    }
    if (Test-Path $dest) { Remove-Item -LiteralPath $dest -Recurse -Force }
    Expand-Archive -LiteralPath $zip -DestinationPath $dest -Force
    Remove-Item -LiteralPath $zip -Force
    if (-not (Test-Path (Join-Path $root "fractadyne.exe"))) { throw "$zipName has no fractadyne.exe where expected" }
    Set-Content -LiteralPath $marker -Value $id.sha[$zipName]
    # Keep the three newest builds.
    Get-ChildItem -LiteralPath $Cache -Directory | Sort-Object LastWriteTime -Descending | Select-Object -Skip 3 |
    ForEach-Object { Remove-Item -LiteralPath $_.FullName -Recurse -Force -ErrorAction SilentlyContinue }
    return $root
}

# --- the harness allow-list -------------------------------------------------------------------------
# flag -> the values it takes. "?x" = optional. Values never contain spaces or quotes, and a file is
# only ever one shipped inside the package.
$Modes = @("--recordtest", "--zoomtest", "--motiontest", "--gputest", "--selftest", "--bench-matrix", "--livetest", "--chunk-sweep", "--soak")
$Allowed = @{
    "--recordtest" = @("?int"); "--zoomtest" = @("?num"); "--zoomtest-rate" = @("num"); "--zoomtest-start-log2" = @("num")
    "--zoomtest-location" = @("loc"); "--zoomtest-taps" = @("taps"); "--zoomtest-hold" = @("num"); "--motiontest" = @(); "--gputest" = @(); "--selftest" = @()
    "--selftest-filter" = @("word"); "--bench-matrix" = @(); "--livetest" = @("pkgfile"); "--size" = @("size")
    "--chunk-sweep" = @("?int"); "--soak" = @("int"); "--soak-depth" = @("depth"); "--center" = @("num", "num")
    "--zoom" = @("num"); "--zoom-log2" = @("num"); "--iter" = @("int"); "--set" = @("assign"); "--window" = @("size")
}
$ValuePattern = @{
    "int" = '^[0-9]{1,9}$'; "num" = '^[-+0-9.eE]{1,400}$'; "word" = '^[A-Za-z0-9_.-]{1,64}$'
    "depth" = '^(session|[-+0-9.eE]{1,20})$'
    "size" = '^[0-9]{2,5}x[0-9]{2,5}$'; "assign" = '^[A-Za-z0-9_]{1,64}=[-+0-9.eE]{1,32}$'
    "pkgfile" = '^(tours|validation|benchmarks)/[A-Za-z0-9_./-]{1,160}\.(toml|fdn|kfr)$'
    # a package file, or "session" (the view the request staged)
    "loc" = '^(session|(tours|validation|benchmarks)/[A-Za-z0-9_./-]{1,160}\.(toml|fdn|kfr))$'
    "taps" = '^[0-9]{1,3},[0-9.]{1,8},[0-9.]{1,8}$'
}

# Validate and resolve a harness argument list. Package files become absolute paths.
function Resolve-HarnessArgs([string[]]$argv, [string]$pkgRoot) {
    # NOTE: PowerShell variable names ignore case - a local "$modes" here WOULD BE the script's
    # "$Modes" list (it was, and every harness request was refused).
    $out = @(); $modeCount = 0; $i = 0
    while ($i -lt $argv.Count) {
        $flag = $argv[$i]; $i++
        if (-not $Allowed.ContainsKey($flag)) { Stop-Refused "flag not allowed: $flag" }
        if ($Modes -contains $flag) { $modeCount++ }
        $out += $flag
        foreach ($kind in $Allowed[$flag]) {
            $opt = $kind.StartsWith("?"); $k = $kind.TrimStart("?")
            $v = if ($i -lt $argv.Count) { $argv[$i] } else { $null }
            $isFile = $k -eq "pkgfile" -or ($k -eq "loc" -and $v -ne "session")
            if ($null -eq $v -or $v -notmatch $ValuePattern[$k] -or ($isFile -and $v -match '\.\.')) {
                if ($opt) { continue }
                Stop-Refused "$flag needs a $k value, got '$v'"
            }
            $i++
            if ($isFile) {
                $abs = Join-Path $pkgRoot ($v -replace '/', '\')
                if (-not (Test-Path -LiteralPath $abs)) { Stop-Refused "$v is not in the package" }
                $v = $abs
            }
            $out += $v
        }
    }
    if ($modeCount -ne 1) { Stop-Refused "exactly one harness mode is required, one of: $($Modes -join ' ')" }
    return , $out
}

# --- the jobs ---------------------------------------------------------------------------------------
function Get-Field($r, [string]$name, $default) {
    if ($r.PSObject.Properties.Name -contains $name -and $null -ne $r.$name) { return $r.$name }
    return $default
}

function Invoke-Battery($r, [string]$dir, $status) {
    $tag = Resolve-Tag ([string](Get-Field $r "build" "latest"))
    $package = [string](Get-Field $r "package" "standard")
    $root = Get-Package $tag $package
    $script = Join-Path $root "scripts\gpu-validate.ps1"
    if ($package -eq "accelerated") {
        # The flat MPFR package ships no battery: take it from the standard package of the SAME
        # build, which is sha256-checked like everything else.
        $std = Get-Package $tag "standard"
        New-Item -ItemType Directory -Force -Path (Join-Path $root "scripts") | Out-Null
        Copy-Item -LiteralPath (Join-Path $std "scripts\gpu-validate.ps1") -Destination $script -Force
    }
    $label = [string](Get-Field $r "label" "")
    if (-not $label) { $label = "$($Computer.ToLower())-$tag-$package" }
    if ($label -notmatch '^[A-Za-z0-9._-]{1,64}$') { Stop-Refused "bad label: $label" }
    $status.build = $tag; $status.package = $package
    $psHost = (Get-Process -Id $PID).Path
    $argv = @("-NoProfile", "-ExecutionPolicy", "Bypass", "-File", "`"$script`"", "-Label", $label, "-Out", "`"$dir`"",
        "-BuildId", "`"$(Join-Path (Join-Path (Join-Path $Share 'builds') $tag) 'BUILD-ID.txt')`"")
    if ([bool](Get-Field $r "quick" $false)) { $argv += "-Quick" }
    $local = Join-Path $Work $status.id
    New-Item -ItemType Directory -Force -Path $local | Out-Null
    $timeout = [math]::Min([int](Get-Field $r "timeout_min" 90), 240)
    Write-JsonFile (Join-Path $dir "status.json") $status
    $res = Invoke-Bounded -File $psHost -Arguments $argv -Cwd $root -Out (Join-Path $local "console.txt") -Err (Join-Path $local "console.err") -TimeoutMin $timeout -Hidden
    Copy-Item (Join-Path $local "console.txt") (Join-Path $dir "battery-console.txt") -ErrorAction SilentlyContinue
    $status.runs = @([ordered]@{ build = $tag; package = $package; exit = $res.exit; seconds = $res.seconds; timed_out = $res.timed_out })
    if ($res.timed_out) { throw "the battery overran its $timeout min timeout and was stopped" }
    # Each step's exit code, from the bundle's own summary, so a reader need not open it to triage.
    $bundle = Get-ChildItem -LiteralPath $dir -Directory -Filter "validate-*" -ErrorAction SilentlyContinue | Select-Object -First 1
    if (-not $bundle) { throw "the battery produced no bundle in $dir (see battery-console.txt)" }
    $status.bundle = $bundle.Name
    $status.screens = @(Get-Screens)
    $steps = [ordered]@{}
    foreach ($l in @(Get-Content -LiteralPath (Join-Path $bundle.FullName "summary.txt") -ErrorAction SilentlyContinue)) {
        if ($l -match '^(build-id|gputest|selftest|live-res|bench-matrix|livetest|uitest|recordtest)\s+(-?[0-9]+)\s') { $steps[$Matches[1]] = [int]$Matches[2] }
    }
    $status.steps = $steps
    $status.detail = (@($steps.Keys) | ForEach-Object { "$_ $($steps[$_])" }) -join ", "
}

# A harness that opens a window renders the SESSION's view: --center/--zoom are read only by the
# headless modes. So a request's "view" becomes a session.toml in the run's fresh config dir, built
# from the committed corpus template (published beside this agent as session-template.toml, so it
# follows the app's schema without a new agent) with only the view keys pinned. Returns $null when
# the request has no view.
function New-ViewSession($r) {
    $v = Get-Field $r "view" $null
    if ($null -eq $v) { return $null }
    $num = '^-?[0-9]+(\.[0-9]+)?([eE][-+]?[0-9]+)?$'
    $re = [string](Get-Field $v "center_re" ""); $im = [string](Get-Field $v "center_im" "")
    if ($re -notmatch $num -or $re.Length -gt 400 -or $im -notmatch $num -or $im.Length -gt 400) { Stop-Refused "view.center_re/center_im must be plain decimals" }
    $log2 = 0.0
    if (-not [double]::TryParse([string](Get-Field $v "upp_log2" ""), [Globalization.NumberStyles]::Float, [Globalization.CultureInfo]::InvariantCulture, [ref]$log2) -or $log2 -lt -1000000 -or $log2 -gt 10) {
        Stop-Refused "view.upp_log2 must be a number (log2 of the complex units per pixel, as in a .fdn)"
    }
    $maxIter = [long](Get-Field $v "max_iter" 1000)
    if ($maxIter -lt 1 -or $maxIter -gt 100000000) { Stop-Refused "view.max_iter must be 1..100000000" }
    $auto = [bool](Get-Field $v "auto_iter" $true)
    $aa = [int](Get-Field $v "aa" 2)
    if ($aa -lt 1 -or $aa -gt 4) { Stop-Refused "view.aa must be 1..4" }
    $tmplPath = Join-Path (Join-Path $Field "setup") "session-template.toml"
    if (-not (Test-Path -LiteralPath $tmplPath)) { throw "no session template at $tmplPath" }
    $tmpl = Get-Content -LiteralPath $tmplPath -Raw
    $e2 = [math]::Floor($log2)
    $inv = [Globalization.CultureInfo]::InvariantCulture
    $pins = [ordered]@{
        center_x = ([double]::Parse($re, $inv)).ToString("R", $inv); center_y = ([double]::Parse($im, $inv)).ToString("R", $inv)
        center_x_str = "`"$re`""; center_y_str = "`"$im`""
        units_per_pixel = ([math]::Pow(2.0, $log2 - $e2)).ToString("R", $inv); units_per_pixel_e = "$e2"
        max_iter = "$maxIter"; auto_iter = $(if ($auto) { "true" } else { "false" }); aa = "$aa"
    }
    foreach ($k in $pins.Keys) {
        if ($tmpl -notmatch "(?m)^$k = ") { throw "the session template has no '$k' key - it no longer matches the app" }
        $tmpl = [regex]::Replace($tmpl, "(?m)^$k = .*$", "$k = $($pins[$k])")
    }
    return $tmpl
}

# The diagnostic INSTRUMENTS a request may arm (DIAGNOSTICS.md, "Environment variables"): each puts
# the live path in a regime that has killed devices, on purpose. Integer values only; nothing else
# from a request ever reaches the environment.
$InstrumentEnv = @("FRACTADYNE_REF_ESCAPE_AT", "FRACTADYNE_BLA_DROP_FRAMES")

function Get-RequestEnv($r) {
    $e = Get-Field $r "env" $null
    $out = [ordered]@{}
    if ($null -eq $e) { return $out }
    foreach ($p in $e.PSObject.Properties) {
        if ($InstrumentEnv -notcontains $p.Name) { Stop-Refused "env $($p.Name) is not an allowed instrument ($($InstrumentEnv -join ', '))" }
        if ([string]$p.Value -notmatch '^[0-9]{1,9}$') { Stop-Refused "env $($p.Name) must be a whole number" }
        $out[$p.Name] = [string]$p.Value
    }
    return $out
}

function Invoke-Harness($r, [string]$dir, $status) {
    $package = [string](Get-Field $r "package" "standard")
    $session = New-ViewSession $r
    $instr = Get-RequestEnv $r
    $status.env = $instr
    $builds = @(Get-Field $r "builds" @())
    if ($builds.Count -eq 0) { $builds = @([string](Get-Field $r "build" "latest")) }
    if ($builds.Count -gt 4) { Stop-Refused "at most 4 builds per request" }
    $tags = @($builds | ForEach-Object { Resolve-Tag ([string]$_) })
    $repeat = [int](Get-Field $r "repeat" 1)
    if ($repeat -lt 1 -or $repeat -gt 10) { Stop-Refused "repeat must be 1..10" }
    $argv = @(@(Get-Field $r "args" @()) | ForEach-Object { [string]$_ })
    $timeout = [math]::Min([int](Get-Field $r "timeout_min" 30), 180)
    # Validate against every build BEFORE running any, so a bad request fails fast.
    $roots = @{}
    foreach ($t in $tags) { $roots[$t] = Get-Package $t $package; $null = Resolve-HarnessArgs $argv $roots[$t] }
    $status.package = $package; $status.runs = @()
    $n = 0
    for ($rep = 1; $rep -le $repeat; $rep++) {
        foreach ($t in $tags) {   # interleaved: A, B, A, B ...
            $n++
            $name = "run-{0:D2}-{1}" -f $n, $t
            $local = Join-Path (Join-Path $Work $status.id) $name
            $cfgDir = Join-Path $local "config"
            New-Item -ItemType Directory -Force -Path $cfgDir | Out-Null
            if ($session) { [IO.File]::WriteAllText((Join-Path $cfgDir "session.toml"), $session, $Utf8) }
            $exe = Join-Path $roots[$t] "fractadyne.exe"
            $resolved = Resolve-HarnessArgs $argv $roots[$t]
            $status.detail = "$name of $($repeat * $tags.Count): fractadyne $($argv -join ' ')"
            Write-JsonFile (Join-Path $dir "status.json") $status
            $env:FRACTADYNE_CONFIG_DIR = $cfgDir
            $env:FRACTADYNE_NO_SOUND = "1"
            foreach ($k in $InstrumentEnv) { Remove-Item "Env:$k" -ErrorAction SilentlyContinue }
            foreach ($k in $instr.Keys) { Set-Item "Env:$k" $instr[$k] }
            $idle0 = [FdField]::IdleSeconds()
            $screens = @(Get-Screens)
            $res = Invoke-Bounded -File $exe -Arguments @($resolved | ForEach-Object { Format-Arg $_ }) -Cwd $local -Out (Join-Path $local "stdout.txt") -Err (Join-Path $local "stderr.txt") -TimeoutMin $timeout
            # Only meaningful when the idle wait is on: with it off (-IdleMinutes 0, e.g. behind a
            # KVM switch that sends input of its own) every run would read as "used".
            $touched = if ($IdleMinutes -gt 0 -and $idle0 -ge 0) { [FdField]::IdleSeconds() -lt $res.seconds } else { $null }
            Remove-Item Env:FRACTADYNE_CONFIG_DIR -ErrorAction SilentlyContinue
            foreach ($k in $InstrumentEnv) { Remove-Item "Env:$k" -ErrorAction SilentlyContinue }
            # Deliver: the output, and the logs (fractadyne.log, frames.bin/.jsonl, any crash report).
            $to = Join-Path $dir $name
            New-Item -ItemType Directory -Force -Path $to | Out-Null
            $o = Join-Path $to "output.txt"
            Get-Content (Join-Path $local "stdout.txt") -Encoding utf8 -ErrorAction SilentlyContinue | Out-File $o -Encoding utf8
            "", "--- stderr ---" | Out-File $o -Encoding utf8 -Append
            Get-Content (Join-Path $local "stderr.txt") -Encoding utf8 -ErrorAction SilentlyContinue | Out-File $o -Encoding utf8 -Append
            foreach ($logs in @((Join-Path $cfgDir "logs"), (Join-Path $local "logs"))) {
                if (Test-Path $logs) { Copy-Item -Path (Join-Path $logs "*") -Destination $to -Recurse -Force -ErrorAction SilentlyContinue }
            }
            # A staged view the app did not LOAD (it falls back to defaults on a file it cannot
            # parse) means the run measured the wrong view - say so, do not pass it off.
            $sessionLoaded = $null
            if ($session) {
                $sessionLoaded = [bool](Select-String -Path (Join-Path $to "fractadyne.log") -Pattern 'session: .* loaded' -Quiet -ErrorAction SilentlyContinue)
            }
            $status.runs += [ordered]@{ run = $name; build = $t; exit = $res.exit; seconds = $res.seconds; timed_out = $res.timed_out; input_during_run = $touched; screens = $screens; view_loaded = $sessionLoaded }
            if ($session -and -not $sessionLoaded) { Write-JsonFile (Join-Path $dir "status.json") $status; throw "$name did not load the staged view (no 'session: ... loaded' in its log) - its result is for the wrong view" }
            Write-JsonFile (Join-Path $dir "status.json") $status
        }
    }
    $status.detail = "$($status.runs.Count) run(s), exit codes: " + ((@($status.runs) | ForEach-Object { if ($_.timed_out) { "timeout" } else { $_.exit } }) -join " ")
}

function Invoke-Events($r, [string]$dir, $status) {
    $days = [int](Get-Field $r "days" 30)
    if ($days -lt 1 -or $days -gt 365) { Stop-Refused "days must be 1..365" }
    $since = (Get-Date).AddDays(-$days)
    # One query per provider: a provider this machine does not have (nvlddmkm on an AMD box) makes
    # a combined query fail outright.
    $queries = @(
        @{ LogName = "System"; Id = 4101 },                        # "Display driver ... stopped responding and has successfully recovered" (a TDR)
        @{ LogName = "System"; ProviderName = "Display" },
        @{ LogName = "System"; ProviderName = "amdkmdag" },
        @{ LogName = "System"; ProviderName = "amdwddmg" },
        @{ LogName = "System"; ProviderName = "nvlddmkm" },
        @{ LogName = "System"; ProviderName = "Microsoft-Windows-Kernel-Power"; Id = 41 },   # rebooted without a clean shutdown
        @{ LogName = "Application"; ProviderName = "Windows Error Reporting" },
        @{ LogName = "Application"; ProviderName = "Application Error" },
        @{ LogName = "Application"; ProviderName = "Application Hang" })
    $seen = @{}; $rows = @(); $absent = @()
    foreach ($q in $queries) {
        $q.StartTime = $since
        # A provider this machine lacks THROWS, even under SilentlyContinue ("The parameter is
        # incorrect" - measured on the dev box for amdkmdag). Skip it, and say which.
        try { $found = @(Get-WinEvent -FilterHashtable $q -ErrorAction SilentlyContinue) }
        catch { if ($q.ContainsKey("ProviderName")) { $absent += $q.ProviderName }; continue }
        foreach ($e in $found) {
            if ($null -eq $e) { continue }
            $key = "$($e.LogName)/$($e.RecordId)"
            if ($seen.ContainsKey($key)) { continue }
            $seen[$key] = $true
            # Crash reports and hangs: only a GPU reset (LiveKernelEvent) or this app's own.
            if ($e.LogName -eq "Application" -and ([string]$e.Message) -notmatch 'LiveKernelEvent|fractadyne') { continue }
            $rows += $e
        }
    }
    $rows = @($rows | ForEach-Object {
            $msg = ([string]$_.Message) -replace '\s+', ' '
            # Some driver events carry no message text on this machine (nvlddmkm 153 on the dev
            # box): their raw data is then the only content.
            if (-not $msg.Trim()) { $msg = "[data] " + ((@($_.Properties) | ForEach-Object { [string]$_.Value }) -join " | ") }
            if ($msg.Length -gt 600) { $msg = $msg.Substring(0, 600) + '...' }
            $kind = if ($_.LogName -eq "System") { "driver" } elseif ($msg -match 'LiveKernelEvent') { "gpu-reset" } else { "app-crash" }
            [ordered]@{ time = $_.TimeCreated.ToString("yyyy-MM-dd HH:mm:ss"); kind = $kind; log = $_.LogName; provider = $_.ProviderName; id = $_.Id; level = $_.LevelDisplayName; message = $msg }
        })
    $rows = @($rows | Sort-Object { $_.time })
    Write-JsonFile (Join-Path $dir "events.json") $rows
    $fmt = { "{0}  {1} {2}  {3}" -f $_.time, $_.provider, $_.id, $_.message }
    $drv = @($rows | Where-Object { $_.kind -eq "driver" })
    $lke = @($rows | Where-Object { $_.kind -eq "gpu-reset" })
    $app = @($rows | Where-Object { $_.kind -eq "app-crash" })
    $txt = @("Display-driver and crash events, last $days days, on $Computer")
    if ($absent.Count -gt 0) { $txt += "(no such event provider on this machine: $($absent -join ', '))" }
    $txt += "", "Display-driver events (System log): $($drv.Count)"
    $txt += ($drv | ForEach-Object $fmt)
    # Windows Error Reporting logs a LiveKernelEvent (a GPU reset, among others) once per report
    # ATTEMPT, so one reset can appear many times: by code first (141 = a GPU engine timeout), then
    # the newest in full.
    $txt += "", "LiveKernelEvent reports (Windows Error Reporting): $($lke.Count)"
    $txt += ($lke | Group-Object { if ($_.message -match 'P1: (\S+)') { $Matches[1] } else { "?" } } | ForEach-Object {
            "  code {0,-5} x{1,-6} first {2}  last {3}" -f $_.Name, $_.Count, ($_.Group | Select-Object -First 1).time, ($_.Group | Select-Object -Last 1).time })
    $txt += "  newest 20:"
    $txt += ($lke | Select-Object -Last 20 | ForEach-Object $fmt)
    # Fractadyne's own crash and hang reports. On a machine that runs --recordtest these are mostly
    # its deliberate abort child, one per run - so the count, and only the newest in full.
    $txt += "", "Fractadyne crash/hang reports: $($app.Count) (newest 20 below; all are in events.json)"
    $txt += ($app | Select-Object -Last 20 | ForEach-Object $fmt)
    # The driver's own dumps of a GPU reset. Usually readable only by an administrator - said so
    # rather than reported as "none".
    $lkr = Join-Path $env:SystemRoot "LiveKernelReports"
    try {
        $dumps = @(Get-ChildItem -LiteralPath $lkr -Recurse -Filter "*.dmp" -ErrorAction Stop | Where-Object { $_.LastWriteTime -ge $since })
        $txt += "", "LiveKernelReports dumps since then: $($dumps.Count)"
        $txt += ($dumps | ForEach-Object { "  {0:yyyy-MM-dd HH:mm:ss}  {1}  {2:N0} bytes" -f $_.LastWriteTime, $_.FullName.Substring($lkr.Length + 1), $_.Length })
        $dumpNote = "$($dumps.Count) live-kernel dump(s)"
    }
    catch {
        $txt += "", "LiveKernelReports: not readable from this account ($($_.Exception.Message))"
        $dumpNote = "live-kernel dumps not readable"
    }
    $txt | Out-File (Join-Path $dir "events.txt") -Encoding utf8
    $status.detail = "$($drv.Count) driver event(s), $($lke.Count) LiveKernelEvent report(s), $($app.Count) Fractadyne crash report(s), $dumpNote"
}

# --- self-update ----------------------------------------------------------------------------------------
# A newer agent published in <share>\field\setup\ replaces this one: copied over it, then this process
# exits and the scheduled task's 5-minute trigger starts the new one. This widens nothing: whoever can
# write the share can already publish a "build" this agent will run. Only ever between jobs.
function Test-SelfUpdate {
    $src = Join-Path (Join-Path $Field "setup") "field-agent.ps1"
    $self = Join-Path $Home_ "field-agent.ps1"
    if (-not (Test-Path -LiteralPath $src) -or -not (Test-Path -LiteralPath $self)) { return }
    $m = Select-String -LiteralPath $src -Pattern '^\$AgentVersion = ([0-9]+)' | Select-Object -First 1
    if (-not $m) { return }
    $v = [int]$m.Matches[0].Groups[1].Value
    if ($v -le $AgentVersion) { return }
    Copy-Item -LiteralPath $src -Destination $self -Force
    Write-Log "updated v$AgentVersion -> v$v from the share; exiting for the task to restart it"
    Write-Heartbeat "updating" "v$AgentVersion -> v$v; the task restarts it within 5 minutes"
    $mutex.ReleaseMutex()
    exit 0
}

# --- one poll -----------------------------------------------------------------------------------------
function Invoke-Poll {
    if (-not (Test-Path -LiteralPath $Share)) { Write-Log "share $Share not reachable"; return }
    if (-not $Once) { Test-SelfUpdate }
    foreach ($d in @($ReqDir, $ResDir, $AgentDir)) { if (-not (Test-Path $d)) { New-Item -ItemType Directory -Force -Path $d | Out-Null } }
    $pending = @(Get-ChildItem -LiteralPath $ReqDir -Filter "*.json" -ErrorAction SilentlyContinue | Sort-Object Name)
    if ((Test-Path (Join-Path $Field "PAUSE")) -or (Test-Path (Join-Path $Home_ "PAUSE"))) { Write-Heartbeat "paused" "a PAUSE file is present"; return }
    if ($pending.Count -eq 0) { Write-Heartbeat "idle" "no requests"; return }
    if (-not $IgnoreIdle) {
        if (Test-Locked) { Write-Heartbeat "waiting" "the workstation is locked"; return }
        $idle = [FdField]::IdleSeconds()
        if ($idle -ge 0 -and $idle -lt $IdleMinutes * 60) { Write-Heartbeat "waiting" ("in use - idle {0:N0} s of the {1} s required" -f $idle, ($IdleMinutes * 60)); return }
    }
    if (@(Get-Process -Name fractadyne -ErrorAction SilentlyContinue).Count -gt 0) { Write-Heartbeat "waiting" "Fractadyne is open"; return }

    $f = $pending[0]
    $id = $f.BaseName
    $claimed = Join-Path $ReqDir "$id.claimed-$Computer"
    try { Move-Item -LiteralPath $f.FullName -Destination $claimed -ErrorAction Stop } catch { return }   # another agent took it
    $dir = Join-Path $ResDir $id
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    Copy-Item -LiteralPath $claimed -Destination (Join-Path $dir "request.json") -Force
    $status = [ordered]@{ id = $id; state = "running"; agent = $Computer; agent_version = $AgentVersion; action = ""; detail = ""
        claimed_utc = Get-UtcStamp; finished_utc = ""; build = ""; package = ""; runs = @() }
    Write-Heartbeat "running" "" $id
    Write-Log "job $id claimed"
    [FdField]::KeepAwake()
    try {
        if ($id -notmatch '^[0-9]{8}-[0-9]{6}-[a-z0-9]{4}$') { Stop-Refused "bad request id: $id" }
        $r = Get-Content -LiteralPath $claimed -Raw | ConvertFrom-Json
        if ([int](Get-Field $r "schema" 0) -ne 1) { Stop-Refused "unknown request schema" }
        $status.action = [string](Get-Field $r "action" "")
        Write-JsonFile (Join-Path $dir "status.json") $status
        switch ($status.action) {
            "battery" { Invoke-Battery $r $dir $status }
            "harness" { Invoke-Harness $r $dir $status }
            "events" { Invoke-Events $r $dir $status }
            default { Stop-Refused "unknown action '$($status.action)'" }
        }
        $status.state = "done"
    }
    catch {
        $status.state = if ($_.Exception -is [FieldRefused]) { "rejected" } else { "failed" }
        $status.detail = "$($_.Exception.Message)"
        if ($status.state -eq "failed") { $status.where = "$($_.InvocationInfo.PositionMessage)`n$($_.ScriptStackTrace)" }
        Write-Log "job $id $($status.state): $_ | $($_.InvocationInfo.PositionMessage -replace '\s+', ' ')"
    }
    finally {
        [FdField]::AllowSleep()
        $status.finished_utc = Get-UtcStamp
        Write-JsonFile (Join-Path $dir "status.json") $status
        Remove-Item -LiteralPath $claimed -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath (Join-Path $Work $id) -Recurse -Force -ErrorAction SilentlyContinue
        Write-Heartbeat "idle" "finished $id ($($status.state))"
        Write-Log "job $id $($status.state)"
    }
}

Write-Log "agent v$AgentVersion started (pid $PID, share $Share, idle $IdleMinutes min$(if ($IgnoreIdle) { ', IGNORING IDLE' }))"
while ($true) {
    try { Invoke-Poll } catch { Write-Log "poll failed: $_" }
    if ($Once) { break }
    Start-Sleep -Seconds $PollSeconds
}
