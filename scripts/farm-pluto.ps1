# farm-pluto.ps1 - run the render farm across this machine and a test machine (PLUTO), through the
# field agent, and measure what the farm is for: whether frames rendered on another GPU match.
#
#   .\scripts\farm-pluto.ps1 -Check            # every precondition, and how to fix each; files nothing
#   .\scripts\farm-pluto.ps1                   # the gate tour: this machine (controller + a local
#                                              #   client) and PLUTO render it together; then every
#                                              #   frame is compared with the tour rendered HERE alone
#   .\scripts\farm-pluto.ps1 -NoLocal          # PLUTO renders every frame (a pure cross-GPU comparison)
#   .\scripts\farm-pluto.ps1 -ShareMode        # share mode: the clients write frames to the share,
#                                              #   this machine checks each one there
#   .\scripts\farm-pluto.ps1 -Tour tours\grand-tour.toml -Size 1280x720 -Ss 1
#   .\scripts\farm-pluto.ps1 -Farmtest         # only --farmtest ON PLUTO: a whole farm on that machine
#   .\scripts\farm-pluto.ps1 -AddFirewallRule  # (asks for elevation) let the test machine reach the
#                                              #   controller: inbound TCP <Port>, Private profile,
#                                              #   local subnet only
#
# WHAT RUNS WHERE. Here: the controller (--farm-render) and, unless -NoLocal, a local client, both
# from target\release\fractadyne.exe, which must BE the build published on the share (the farm
# refuses any other: exact version and commit; publish-share.ps1). There: one render client (agent action
# farm-client, agent v14+), from that published package, which dials this machine.
#
# NOTHING HERE TOUCHES YOUR REAL CONFIGURATION. Every process gets a throw-away config folder under
# local\farm-runs\<stamp>\ (gitignored), and every run a fresh farm key - the key travels to the test
# machine in the request file on the share (the results copy is redacted), so it is used once.
#
# THE OUTPUT. local\farm-runs\<stamp>\: farm-out\ (the frames and the job's farm\ folder: done log,
# events, metrics, status), reference\ (the same tour, same settings, rendered here alone),
# compare.txt (per machine: frames, frames identical, differing pixels), controller.txt, and the
# field result folder on the share (the client's output and every render process's log).
#
# ASCII-only on purpose: Windows PowerShell 5.1 reads a BOM-less .ps1 as ANSI.

[CmdletBinding()]
param(
    [string]$Agent = "PLUTO",
    [string]$Tour = "",
    [string]$Size = "",
    [int]$Ss = 0,
    [int]$Port = 46733,
    # The address the test machine dials. Default: this machine's IPv4 on the default route.
    [string]$Address = "",
    [switch]$NoLocal,
    [switch]$ShareMode,
    [switch]$Farmtest,
    [switch]$Check,
    [switch]$AddFirewallRule,
    [int]$TimeoutMin = 60,
    [string]$Share = ""
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $root
if (-not $Share) { $Share = if (Test-Path "D:\share\Fractadyne") { "D:\share\Fractadyne" } else { [string]$env:FRACTADYNE_SHARE } }
if (-not $Share) { throw "no share: pass -Share, or set FRACTADYNE_SHARE" }
$exe = Join-Path $root "target\release\fractadyne.exe"
$problems = @()
function Ok([string]$m) { Write-Host "  ok    $m" -ForegroundColor Green }
function Bad([string]$m, [string]$fix) { Write-Host "  FIX   $m" -ForegroundColor Yellow; if ($fix) { Write-Host "        -> $fix" }; $script:problems += $m }

# --- the firewall rule, on request ----------------------------------------------------------------
$ruleName = "Fractadyne render farm (TCP $Port, local subnet)"
if ($AddFirewallRule) {
    $cmd = "New-NetFirewallRule -DisplayName '$ruleName' -Direction Inbound -Protocol TCP -LocalPort $Port -Action Allow -Profile Private -RemoteAddress LocalSubnet"
    Write-Host "Adding (elevated): $cmd"
    Start-Process powershell.exe -Verb RunAs -Wait -ArgumentList "-NoProfile", "-Command", $cmd
    Write-Host "Done. Run -Check to confirm."
    return
}

Write-Host "Render farm with $Agent - preconditions" -ForegroundColor Cyan

# --- the build: target\release (the controller) and the published package (the test machine's
# client) must be the SAME clean build - the farm admits only an exact version-and-commit match.
# Not necessarily HEAD: a later commit that touches no code (a script, a document) changes nothing.
$head = (& git rev-parse HEAD).Trim()
$rebuild = "cargo build --release -j 1 -p fractadyne-app; pwsh -File scripts\publish-share.ps1 -SkipSource"
function Get-ExeVersion([string]$path) {
    $scratch = Join-Path ([IO.Path]::GetTempPath()) ("fd-version-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Force -Path $scratch | Out-Null
    $prev = $env:FRACTADYNE_CONFIG_DIR
    try {
        $env:FRACTADYNE_CONFIG_DIR = $scratch
        $ErrorActionPreference = "Continue"
        return (& $path --version 2>$null | Where-Object { $_ -match '^fractadyne ' } | Select-Object -Last 1)
    }
    finally {
        if ($null -eq $prev) { Remove-Item Env:FRACTADYNE_CONFIG_DIR -ErrorAction SilentlyContinue } else { $env:FRACTADYNE_CONFIG_DIR = $prev }
        Remove-Item -Recurse -Force $scratch -ErrorAction SilentlyContinue
    }
}
$tag = ""; $sha = ""
if (Test-Path $exe) {
    $v = Get-ExeVersion $exe
    if ($v -match '^fractadyne (\S+) \(build [0-9]+, g([0-9a-f]{7,40})(-dirty)?\)$') {
        $tag = "v" + $Matches[1]; $sha = $Matches[2]
        if ($Matches[3]) { Bad "target\release\fractadyne.exe is a -dirty build ($v); the farm refuses one" "commit, then $rebuild" }
        elseif (-not $head.StartsWith($sha)) { Ok "target\release\fractadyne.exe: $v (not HEAD - fine while the commits since touch no code)" }
        else { Ok "target\release\fractadyne.exe: $v (HEAD)" }
    }
    else { Bad "target\release\fractadyne.exe reports '$v', which names no commit" $rebuild }
}
else { Bad "target\release\fractadyne.exe is missing" $rebuild }
if ($tag) {
    $idPath = Join-Path (Join-Path (Join-Path $Share "builds") $tag) "BUILD-ID.txt"
    if (Test-Path -LiteralPath $idPath) {
        $commit = ((Get-Content -LiteralPath $idPath | Where-Object { $_ -match '^commit: ' } | Select-Object -First 1) -replace '^commit: ', '').Trim()
        if ($commit.StartsWith($sha)) { Ok "$tag on the share is the same build (g$sha)" }
        else { Bad "$tag on the share is commit $($commit.Substring(0, [math]::Min(9, $commit.Length))), target\release is g$sha" "pwsh -File scripts\publish-share.ps1 -SkipSource   (it publishes only a clean build of HEAD: rebuild first if HEAD moved)" }
    }
    else { Bad "no published build $tag on the share" $rebuild }
}

# --- the agent -------------------------------------------------------------------------------------
$hb = $null
try { $hb = Get-Content -LiteralPath (Join-Path $Share "field\agent\$Agent.json") -Raw | ConvertFrom-Json } catch { }
if (-not $hb) { Bad "no heartbeat from $Agent's field agent" "is the agent installed there? (scripts\field-agent-setup.ps1)" }
else {
    $last = if ($hb.last_poll_utc -is [datetime]) { $hb.last_poll_utc.ToUniversalTime() } else { [datetime]::Parse([string]$hb.last_poll_utc, [Globalization.CultureInfo]::InvariantCulture, [Globalization.DateTimeStyles]::AssumeUniversal -bor [Globalization.DateTimeStyles]::AdjustToUniversal) }
    $age = [int]((Get-Date).ToUniversalTime() - $last).TotalSeconds
    if ($age -gt 3 * [int]$hb.poll_seconds) { Bad "$Agent's agent last polled $age s ago" "wake $Agent's display / check its session" }
    elseif ($hb.state -eq "paused") { Bad "$Agent's agent is paused" "remove the PAUSE file ($Share\field\PAUSE or on $Agent)" }
    elseif ([int]$hb.agent_version -lt $(if ($ShareMode) { 15 } else { 14 })) { Bad "$Agent runs agent v$($hb.agent_version); this needs v$(if ($ShareMode) { 15 } else { 14 })" "copy scripts\field-agent.ps1 to $Share\field\setup\ - the agent updates itself within 5 minutes" }
    else { Ok "$Agent's agent v$($hb.agent_version) is $($hb.state) (last poll $age s ago)" }
}

# field-request.ps1 in a child pwsh: its host output is our stdout, so the request id ("Filed <id>")
# can be read back - the agent may claim the request before we could list it.
function Send-FieldRequest([string[]]$argv) {
    $lines = @(& pwsh -NoProfile -File (Join-Path $root "scripts\field-request.ps1") @argv -Share $Share 2>&1 | ForEach-Object { "$_" })
    $lines | Out-Host
    $m = $lines | Select-String -Pattern '^Filed ([0-9]{8}-[0-9]{6}-[a-z0-9]{4}) ' | Select-Object -First 1
    if (-not $m) { throw "field-request.ps1 did not file a request" }
    return $m.Matches[0].Groups[1].Value
}

if ($Farmtest) {
    if ($problems.Count -gt 0) { Write-Host ""; Write-Host "Not ready: $($problems.Count) thing(s) to fix." -ForegroundColor Yellow; exit 1 }
    if ($Check) { Write-Host ""; Write-Host "Ready to run --farmtest on $Agent."; return }
    Write-Host "Queueing --farmtest on $Agent ($tag)..." -ForegroundColor Cyan
    $id = Send-FieldRequest @("-Action", "harness", "-Run", "--farmtest", "-Build", $tag, "-TimeoutMin", "20", "-Wait", "-WaitMinutes", "30")
    $o = Get-ChildItem -LiteralPath (Join-Path $Share "field\results\$id") -Recurse -Filter "output.txt" -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($o) { Write-Host ""; Get-Content -LiteralPath $o.FullName | Where-Object { $_ -match '^\s*\[(PASS|FAIL)\]|^farmtest' } }
    return
}

# --- the network: an address the test machine can dial, and a firewall that lets it in --------------
if (-not $Address) {
    $route = Get-NetRoute -DestinationPrefix "0.0.0.0/0" -ErrorAction SilentlyContinue | Sort-Object RouteMetric | Select-Object -First 1
    if ($route) { $Address = (Get-NetIPAddress -InterfaceIndex $route.InterfaceIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue | Select-Object -First 1).IPAddress }
}
if ($Address) { Ok "$Agent will dial $Address`:$Port (pass -Address to choose another)" }
else { Bad "cannot tell which address $Agent should dial" "pass -Address <this machine's LAN IPv4>" }
# Read through the firewall's COM policy: unelevated, and 0.2 s where Get-NetFirewall* takes minutes
# (and Get-NetFirewallPortFilter -Protocol is refused unelevated). An inbound rule for TCP <Port>,
# or for fractadyne.exe itself; a BLOCK rule for the exe (a dismissed "allow access" prompt makes
# one) beats any allow rule, so it is named.
$allow = @(); $block = @()
if ($Address -match '^127\.') { Ok "loopback address: no firewall rule needed (a rehearsal on this machine)" }
else { try {
    foreach ($r in (New-Object -ComObject HNetCfg.FwPolicy2).Rules) {
        if ($r.Direction -ne 1 -or -not $r.Enabled) { continue }
        $tcp = $r.Protocol -eq 6 -or $r.Protocol -eq 256
        $byPort = $tcp -and ($r.LocalPorts -split ',' | Where-Object { $_ -eq "$Port" -or $_ -eq "*" -or ($_ -match '^([0-9]+)-([0-9]+)$' -and [int]$Matches[1] -le $Port -and $Port -le [int]$Matches[2]) })
        $byApp = $r.ApplicationName -and $r.ApplicationName -match 'fractadyne\.exe$'
        if ($byApp -and $r.Action -eq 0) { $block += $r }
        elseif ($r.Action -eq 1 -and (($byPort -and -not $r.ApplicationName) -or ($byApp -and $tcp -and $byPort))) { $allow += $r }
    }
    if ($block.Count -gt 0) { Bad "a firewall rule BLOCKS fractadyne.exe inbound ('$($block[0].Name)')" "wf.msc -> Inbound Rules -> delete or allow '$($block[0].Name)'" }
    elseif ($allow.Count -gt 0) { Ok "inbound TCP $Port is allowed ('$($allow[0].Name)')" }
    else { Bad "no firewall rule lets $Agent reach TCP $Port here" ".\scripts\farm-pluto.ps1 -AddFirewallRule   (asks for elevation; Private profile, local subnet only)" }
}
catch { Write-Host "  ?     could not read the firewall rules ($($_.Exception.Message)); if $Agent cannot connect, run -AddFirewallRule" } }

if ($problems.Count -gt 0) { Write-Host ""; Write-Host "Not ready: $($problems.Count) thing(s) to fix." -ForegroundColor Yellow; exit 1 }
if ($Check) { Write-Host ""; Write-Host "Ready."; return }

# --- the run -------------------------------------------------------------------------------------------
$run = Join-Path $root ("local\farm-runs\" + (Get-Date -Format "yyyyMMdd-HHmmss"))
New-Item -ItemType Directory -Force -Path $run | Out-Null
if (-not $Tour) {
    # The gate tour: 19 frames, normalized, a palette blend and a caption, from home to 1e30.
    $Tour = Join-Path $run "farm-gate.toml"
    [IO.File]::WriteAllText($Tour, @'
format_version = 2
name = "Farm gate"

[render]
size = "640x360"
fps = 3
ss = 1
normalize = true
max_iter = 2000
auto_iter = false

[[location]]
id = "target"
re = "-5.62202621523037212744969596262961926232336058642000859332104071064648040651980117009368022864076665266819518615342205563126413961786451e-1"
im = "6.42817149072775248899624656627830941472997397665282056405495715932366418738755172614822993656471501541311398325174287701850021449311247e-1"

[[keyframe]]
t = 0
re = "-0.5"
im = "0"
zoom = 1.0
max_iter = 2000
palette = "Ember"

[[keyframe]]
t = 2
location = "target"
zoom = 8.0
max_iter = 3000

[[keyframe]]
t = 6
location = "target"
zoom = "1e30"
max_iter = 20000
palette = "Nebula"

[[annotation]]
kind = "caption"
text = "farm gate"
t = 0
secs = 0
pos = "bottom"
'@, [Text.Encoding]::ASCII)
}
$Tour = (Resolve-Path $Tour).Path
$extra = @()
if ($Size) { $extra += @("--size", $Size) }
if ($Ss -gt 0) { $extra += @("--ss", "$Ss") }
$keyFile = Join-Path $run "farm-key.txt"
$out = Join-Path $run "farm-out"
$ctlArgs = @("--farm-render", "`"$Tour`"", "--out", "`"$out`"", "--listen", "0.0.0.0:$Port", "--farm-key-file", "`"$keyFile`"", "--min-clients", $(if ($NoLocal) { "1" } else { "2" })) + $extra
if (-not $NoLocal) { $ctlArgs += "--local" }
# Share mode: this machine's path to the share; the test machine's agent passes its own.
if ($ShareMode) { $ctlArgs += @("--share-root", "`"$Share`"") }

function Start-Fd([string]$name, [string[]]$argv, [string]$cfg) {
    if ($argv.Count -eq 0) { throw "refusing to start fractadyne with no arguments" }
    New-Item -ItemType Directory -Force -Path $cfg | Out-Null
    $env:FRACTADYNE_CONFIG_DIR = $cfg
    $env:FRACTADYNE_NO_SOUND = "1"
    try {
        return Start-Process -FilePath $exe -ArgumentList $argv -PassThru -NoNewWindow -WorkingDirectory $run `
            -RedirectStandardOutput (Join-Path $run "$name.txt") -RedirectStandardError (Join-Path $run "$name.err.txt")
    }
    finally { Remove-Item Env:FRACTADYNE_CONFIG_DIR -ErrorAction SilentlyContinue }
}

# Stop a process this script started and everything under it (the controller's local client and its
# render children), by the process tree - never by name, so the user's own windows are untouched.
# A child born before its parent is a recycled pid, not a child.
function Stop-Mine($p) {
    if ($null -eq $p) { return }
    $all = @(Get-CimInstance Win32_Process -ErrorAction SilentlyContinue)
    $tree = @(); $frontier = @(@{ Id = $p.Id; Born = $p.StartTime })
    while ($frontier.Count -gt 0) {
        $next = @()
        foreach ($f in $frontier) {
            foreach ($c in $all | Where-Object { $_.ParentProcessId -eq $f.Id -and $_.CreationDate -ge $f.Born }) {
                $tree += $c.ProcessId; $next += @{ Id = $c.ProcessId; Born = $c.CreationDate }
            }
        }
        $frontier = $next
    }
    foreach ($id in @($p.Id) + $tree) { Stop-Process -Id $id -Force -ErrorAction SilentlyContinue }
}

Write-Host ""
Write-Host "Run folder: $run" -ForegroundColor Cyan
$ctl = $null
try {
    $ctl = Start-Fd "controller" $ctlArgs (Join-Path $run "controller-cfg")
    $deadline = (Get-Date).AddSeconds(30)
    while (-not (Select-String -LiteralPath (Join-Path $run "controller.txt") -Pattern '^listening on ' -Quiet -ErrorAction SilentlyContinue)) {
        if ($ctl.HasExited -or (Get-Date) -gt $deadline) { throw "the controller did not start listening: see $run\controller.txt / controller.err.txt" }
        Start-Sleep -Milliseconds 300
    }
    Write-Host "Controller listening (pid $($ctl.Id)); asking $Agent to join..."
    $fr = @("-Action", "farm-client", "-Controller", "$Address`:$Port", "-FarmKeyFile", $keyFile, "-Build", $tag, "-TimeoutMin", "$TimeoutMin")
    if ($ShareMode) { $fr += "-FarmShare" }
    $reqId = Send-FieldRequest $fr

    # Follow the controller until the job ends.
    $seen = 0
    $deadline = (Get-Date).AddMinutes($TimeoutMin + 5)
    while (-not $ctl.HasExited) {
        $lines = @(Get-Content -LiteralPath (Join-Path $run "controller.txt") -ErrorAction SilentlyContinue)
        for ($i = $seen; $i -lt $lines.Count; $i++) { if ($lines[$i] -notmatch '^\[fd-') { Write-Host "  [controller] $($lines[$i])" } }
        $seen = $lines.Count
        if ((Get-Date) -gt $deadline) { Write-Host "Timed out; stopping the controller." -ForegroundColor Yellow; Stop-Mine $ctl; break }
        Start-Sleep -Seconds 1
    }
    $ctl.WaitForExit()
    $lines = @(Get-Content -LiteralPath (Join-Path $run "controller.txt") -ErrorAction SilentlyContinue)
    for ($i = $seen; $i -lt $lines.Count; $i++) { if ($lines[$i] -notmatch '^\[fd-') { Write-Host "  [controller] $($lines[$i])" } }
    $code = $ctl.ExitCode
    Write-Host "Controller exit $code"
}
finally { Stop-Mine $ctl }

# --- the reference: the same tour, same settings, rendered here alone -------------------------------------
$ctlText = Get-Content -LiteralPath (Join-Path $run "controller.txt") -Raw -ErrorAction SilentlyContinue
$refArgs = @("--render-tour", "`"$Tour`"", "--out", "`"$(Join-Path $run 'reference')`"", "--farm-child", "-y") + $extra
$anchors = Join-Path $out "farm\anchors.toml"
if (Test-Path -LiteralPath $anchors) { $refArgs += @("--norm-anchors", "`"$anchors`"") }
if ($ctlText -match 'reference-orbit cap for this job: ([0-9]+) samples') { $refArgs += @("--set", "ORBIT_LEN_CAP=$($Matches[1])") }
# The clients' settings ARE the session the controller wrote for its anchors child: render with a copy.
$refCfg = Join-Path $run "reference-cfg"
$ctlCfg = Join-Path $out "farm\controller-cfg"
if (Test-Path -LiteralPath $ctlCfg) { Copy-Item -LiteralPath $ctlCfg -Destination $refCfg -Recurse }
Write-Host "Rendering the reference here..."
$ref = Start-Fd "reference" $refArgs $refCfg
$ref.WaitForExit()
if ($ref.ExitCode -ne 0) { Write-Host "The reference render failed (exit $($ref.ExitCode)); see $run\reference.err.txt" -ForegroundColor Yellow }

# --- the comparison ------------------------------------------------------------------------------------------
$prefix = [IO.Path]::GetFileNameWithoutExtension($Tour)
& python (Join-Path $root "scripts\farm_compare.py") $out (Join-Path $run "reference") $prefix 2>&1 | Tee-Object -FilePath (Join-Path $run "compare.txt") | Out-Host

if ($reqId) {
    Write-Host ""
    Write-Host "$Agent's side (field request $reqId):"
    # A request never claimed still holds the (now useless) key on the share: withdraw it.
    $pending = Join-Path $Share "field\requests\$reqId.json"
    if (Test-Path -LiteralPath $pending) {
        Remove-Item -LiteralPath $pending -Force
        Write-Host "  $Agent never claimed the request; withdrawn (is its agent running, and not paused?)" -ForegroundColor Yellow
    }
    $deadline = if (Test-Path -LiteralPath (Join-Path $Share "field\results\$reqId")) { (Get-Date).AddMinutes(3) } else { Get-Date }
    do {
        $s = $null
        try { $s = Get-Content -LiteralPath (Join-Path $Share "field\results\$reqId\status.json") -Raw | ConvertFrom-Json } catch { }
        if ($s -and $s.state -in @("done", "failed", "rejected")) { break }
        Start-Sleep -Seconds 5
    } while ((Get-Date) -lt $deadline)
    if ($s) { Write-Host "  $($s.state): $($s.detail)" } else { Write-Host "  no result yet" }
    Write-Host "  -> $Share\field\results\$reqId"
}
Write-Host ""
Write-Host "Results: $run"
exit $code
