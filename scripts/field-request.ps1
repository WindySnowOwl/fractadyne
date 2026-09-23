# field-request.ps1 - ask the field agent on a test machine for a run, and (optionally) wait for it.
# The dev-box half of scripts/field-agent.ps1: it writes a request into <share>\field\requests\ and
# reads the answer from <share>\field\results\<id>\.
#
#   .\scripts\field-request.ps1                                   # status: agents, queue, recent results
#   .\scripts\field-request.ps1 -Action battery -Wait             # the full battery, latest build
#   .\scripts\field-request.ps1 -Action battery -Quick -Package accelerated
#   .\scripts\field-request.ps1 -Action harness -Run "--recordtest" -Wait
#   .\scripts\field-request.ps1 -Action harness -Run "--zoomtest 40 --zoomtest-rate 4.0" `
#       -Builds v0.2.41-beta.113,v0.2.41-beta.114 -Repeat 3 -Wait    # A,B,A,B,A,B
#   .\scripts\field-request.ps1 -Action events -Days 30 -Wait     # display-driver resets, crash reports
#   .\scripts\field-request.ps1 -Cancel <id>                      # withdraw a request not yet started
#
# The agent is the authority on what it will run (field-agent.ps1 $Allowed); this only checks the
# shape of a request so a typo fails here rather than after a wait.
#
# ASCII-only on purpose: Windows PowerShell 5.1 reads a BOM-less .ps1 as ANSI.

[CmdletBinding()]
param(
    [ValidateSet("status", "battery", "harness", "events")][string]$Action = "status",
    [string]$Build = "latest",
    [string[]]$Builds = @(),
    [ValidateSet("standard", "accelerated")][string]$Package = "standard",
    [switch]$Quick,
    [string]$Label = "",
    # harness: the fractadyne flags, as one string ("--zoomtest 40 --zoomtest-rate 4.0").
    [string]$Run = "",
    [int]$Repeat = 1,
    [int]$Days = 30,
    [int]$TimeoutMin = 0,
    [string]$Note = "",
    [switch]$Wait,
    [int]$WaitMinutes = 120,
    [string]$Cancel = "",
    [string]$Share = ""
)

$ErrorActionPreference = "Stop"
if (-not $Share) { $Share = if (Test-Path "D:\share\Fractadyne") { "D:\share\Fractadyne" } else { "\\vger\share\Fractadyne" } }
$Field = Join-Path $Share "field"
$ReqDir = Join-Path $Field "requests"
$ResDir = Join-Path $Field "results"
$AgentDir = Join-Path $Field "agent"
$Utf8 = New-Object System.Text.UTF8Encoding($false)

function Read-Json([string]$p) { try { return Get-Content -LiteralPath $p -Raw -ErrorAction Stop | ConvertFrom-Json } catch { return $null } }

# A "...Z" stamp as UTC. PowerShell 7's ConvertFrom-Json has already turned it into a DateTime
# (5.1 leaves a string); re-parsing that DateTime's TEXT reads it as local time - 5 h off here.
function ConvertTo-UtcTime($v) {
    if ($v -is [datetime]) { return $v.ToUniversalTime() }
    return [datetime]::Parse([string]$v, [Globalization.CultureInfo]::InvariantCulture,
        [Globalization.DateTimeStyles]::AssumeUniversal -bor [Globalization.DateTimeStyles]::AdjustToUniversal)
}

function Show-Agents {
    $agents = @(Get-ChildItem -LiteralPath $AgentDir -Filter "*.json" -ErrorAction SilentlyContinue)
    if ($agents.Count -eq 0) { Write-Host "No agent has ever reported (nothing in $AgentDir)." -ForegroundColor Yellow; return }
    foreach ($a in $agents) {
        $h = Read-Json $a.FullName
        if (-not $h) { continue }
        $age = [int]((Get-Date).ToUniversalTime() - (ConvertTo-UtcTime $h.last_poll_utc)).TotalSeconds
        $alive = if ($h.state -eq "running") { "running $($h.current)" } elseif ($age -le 3 * $h.poll_seconds) { "alive" } else { "SILENT for $age s" }
        Write-Host ("{0,-12} {1,-22} {2}: {3}  (idle {4}/{5} s, locked {6}, pending {7}, last poll {8} s ago)" -f `
                $h.computer, $alive, $h.state, $h.detail, $h.idle_seconds, $h.idle_required, $h.locked, $h.pending, $age)
    }
}

function Show-Result([string]$id) {
    $dir = Join-Path $ResDir $id
    $s = Read-Json (Join-Path $dir "status.json")
    if (-not $s) { Write-Host "$id : no status yet"; return }
    Write-Host ("{0} : {1} - {2} {3}" -f $id, $s.state.ToUpper(), $s.action, $s.detail)
    foreach ($r in @($s.runs)) {
        if ($null -eq $r) { continue }
        $name = if ($r.PSObject.Properties.Name -contains "run") { $r.run } else { "$($r.build) $($r.package)" }
        $flag = if ($r.PSObject.Properties.Name -contains "input_during_run" -and $r.input_during_run) { "  (machine was USED during this run)" } else { "" }
        Write-Host ("    {0,-32} exit {1,-5} {2,7} s{3}{4}" -f $name, $(if ($r.timed_out) { "TIMEOUT" } else { $r.exit }), $r.seconds, $(if ($r.timed_out) { " timed out" } else { "" }), $flag)
    }
    Write-Host "    -> $dir"
}

# --- status / cancel ------------------------------------------------------------------------------
if ($Cancel) {
    $p = Join-Path $ReqDir "$Cancel.json"
    if (Test-Path -LiteralPath $p) { Remove-Item -LiteralPath $p; Write-Host "Withdrawn: $Cancel" }
    else { Write-Host "Not pending (already claimed, finished, or never filed): $Cancel" -ForegroundColor Yellow }
    return
}
if ($Action -eq "status") {
    Show-Agents
    $pending = @(Get-ChildItem -LiteralPath $ReqDir -Filter "*.json" -ErrorAction SilentlyContinue | Sort-Object Name)
    Write-Host ""
    Write-Host "Queued: $($pending.Count)"
    $pending | ForEach-Object { Write-Host "    $($_.BaseName)" }
    Write-Host ""
    Write-Host "Recent results:"
    Get-ChildItem -LiteralPath $ResDir -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending |
    Select-Object -First 5 | ForEach-Object { Show-Result $_.Name }
    if (Test-Path (Join-Path $Field "PAUSE")) { Write-Host ""; Write-Host "PAUSED from the share ($Field\PAUSE)." -ForegroundColor Yellow }
    return
}

# --- file a request -------------------------------------------------------------------------------
# `-Builds a,b` arrives as ONE string under `pwsh -File`.
$Builds = @($Builds | ForEach-Object { $_ -split ',' } | Where-Object { $_ })
$tagRe = '^(latest|v[0-9]+\.[0-9]+\.[0-9]+(-beta\.[0-9]+)?)$'
foreach ($t in @($Build) + $Builds) { if ($t -notmatch $tagRe) { throw "bad build tag: $t" } }
$id = "{0:yyyyMMdd-HHmmss}-{1}" -f (Get-Date), (-join ((48..57) + (97..122) | Get-Random -Count 4 | ForEach-Object { [char]$_ }))
$req = [ordered]@{ schema = 1; id = $id; action = $Action; package = $Package; requested_utc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ"); note = $Note }
switch ($Action) {
    "battery" {
        $req.build = $Build
        $req.quick = [bool]$Quick
        if ($Label) { $req.label = $Label }
    }
    "harness" {
        if (-not $Run) { throw "-Action harness needs -Run `"--flag ...`"" }
        $req.args = @($Run -split '\s+' | Where-Object { $_ })
        if ($Builds.Count -gt 0) { $req.builds = @($Builds) } else { $req.build = $Build }
        $req.repeat = $Repeat
    }
    "events" { $req.days = $Days }
}
if ($TimeoutMin -gt 0) { $req.timeout_min = $TimeoutMin }

New-Item -ItemType Directory -Force -Path $ReqDir | Out-Null
$path = Join-Path $ReqDir "$id.json"
[IO.File]::WriteAllText("$path.tmp", ($req | ConvertTo-Json -Depth 6), $Utf8)
Move-Item -LiteralPath "$path.tmp" -Destination $path   # the agent only picks up *.json
Write-Host "Filed $id ($Action)." -ForegroundColor Cyan
Show-Agents

if (-not $Wait) { return }

# --- wait -----------------------------------------------------------------------------------------
$deadline = (Get-Date).AddMinutes($WaitMinutes)
$last = ""
while ((Get-Date) -lt $deadline) {
    Start-Sleep -Seconds 10
    $s = Read-Json (Join-Path (Join-Path $ResDir $id) "status.json")
    $now = if ($s) { "$($s.state): $($s.detail)" } elseif (Test-Path -LiteralPath $path) { "queued" } else { "claimed" }
    if ($now -ne $last) { Write-Host ("[{0:HH:mm:ss}] {1}" -f (Get-Date), $now); $last = $now }
    if ($s -and $s.state -in @("done", "failed", "rejected")) { break }
}
Write-Host ""
Show-Result $id
