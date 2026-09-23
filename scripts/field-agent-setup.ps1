# field-agent-setup.ps1 - install, pause, resume or remove the Fractadyne field agent on a test
# machine. Run it ON the test machine, as the user who is normally logged on there:
#
#   pwsh -ExecutionPolicy Bypass -File \\vger\share\Fractadyne\field\setup\field-agent-setup.ps1
#   ...\field-agent-setup.ps1 -Disable      # pause: no job starts until -Enable
#   ...\field-agent-setup.ps1 -Enable
#   ...\field-agent-setup.ps1 -Status
#   ...\field-agent-setup.ps1 -Uninstall    # stop it and remove everything it installed
#
# WHAT IT INSTALLS: field-agent.ps1 (from beside this script) into %LOCALAPPDATA%\Fractadyne-field\,
# and a scheduled task, "Fractadyne field agent", that starts it hidden when you log on and keeps
# it running. The agent runs in YOUR session, as you, with your normal (not administrator) rights;
# nothing listens on the network. See field-agent.ps1 for exactly what it will and will not run.
#
# Re-running the install replaces the agent with the copy beside this script (how it is updated).
#
# If the task registration says "Access is denied", run this once from an elevated PowerShell
# (Run as administrator). The task is still created to run as you, not as an administrator.
#
# ASCII-only on purpose: Windows PowerShell 5.1 reads a BOM-less .ps1 as ANSI.

[CmdletBinding()]
param(
    # The Fractadyne share root the agent polls.
    [string]$Share = "\\vger\share\Fractadyne",
    # Minutes without keyboard or mouse input before a job may start. 0 = do not wait.
    [int]$IdleMinutes = 5,
    [switch]$Disable,
    [switch]$Enable,
    [switch]$Status,
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"
$TaskName = "Fractadyne field agent"
$Dest = Join-Path $env:LOCALAPPDATA "Fractadyne-field"
$User = "$env:USERDOMAIN\$env:USERNAME"

function Show-Status {
    $t = Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
    if (-not $t) { Write-Host "Not installed."; return }
    $i = $t | Get-ScheduledTaskInfo
    Write-Host ("Task      : {0} ({1}), last run {2}, last result {3}" -f $TaskName, $t.State, $i.LastRunTime, $i.LastTaskResult)
    $cfgPath = Join-Path $Dest "config.json"
    if (Test-Path $cfgPath) { Write-Host ("Config    : " + ((Get-Content $cfgPath -Raw) -replace '\s+', ' ')) }
    $hb = Join-Path (Join-Path (Join-Path $Share "field") "agent") "$env:COMPUTERNAME.json"
    if (Test-Path $hb) { Write-Host ("Heartbeat : " + ((Get-Content $hb -Raw) -replace '\s+', ' ')) }
    $log = Join-Path $Dest "agent.log"
    if (Test-Path $log) { Write-Host "Agent log (last 5 lines):"; Get-Content $log -Tail 5 | ForEach-Object { Write-Host "  $_" } }
}

if ($Status) { Show-Status; return }

if ($Uninstall) {
    if (Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue) {
        Stop-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
        Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false
    }
    if (Test-Path $Dest) { Remove-Item -LiteralPath $Dest -Recurse -Force }
    Write-Host "Removed the task and $Dest. (Results already on the share are left there.)"
    return
}

if ($Disable) {
    Stop-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
    Disable-ScheduledTask -TaskName $TaskName | Out-Null
    Write-Host "Paused. No job will start until you run this with -Enable."
    return
}

if ($Enable) {
    Enable-ScheduledTask -TaskName $TaskName | Out-Null
    Start-ScheduledTask -TaskName $TaskName
    Write-Host "Resumed."
    return
}

# --- install / update -----------------------------------------------------------------------------
$src = Join-Path (Split-Path -Parent $MyInvocation.MyCommand.Path) "field-agent.ps1"
if (-not (Test-Path $src)) { throw "field-agent.ps1 not found beside this script ($src)" }
if (-not (Test-Path -LiteralPath $Share)) { throw "the share $Share is not reachable from this machine" }

New-Item -ItemType Directory -Force -Path $Dest | Out-Null
if (Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue) {
    Stop-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
}
Copy-Item -LiteralPath $src -Destination (Join-Path $Dest "field-agent.ps1") -Force
$cfg = [ordered]@{ share = $Share; idle_minutes = $IdleMinutes }
[IO.File]::WriteAllText((Join-Path $Dest "config.json"), ($cfg | ConvertTo-Json), (New-Object System.Text.UTF8Encoding($false)))

# PowerShell 7 if present (what the battery has been run with), else Windows PowerShell 5.1.
$psHost = (Get-Command pwsh -ErrorAction SilentlyContinue).Source
if (-not $psHost) { $psHost = (Get-Command powershell).Source }

$action = New-ScheduledTaskAction -Execute $psHost `
    -Argument "-NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$(Join-Path $Dest 'field-agent.ps1')`""
# At logon; and every 5 minutes as a safety net - a start while it is already running is ignored.
$triggers = @(
    (New-ScheduledTaskTrigger -AtLogOn -User $User),
    (New-ScheduledTaskTrigger -Once -At (Get-Date).AddMinutes(1) -RepetitionInterval (New-TimeSpan -Minutes 5)))
$principal = New-ScheduledTaskPrincipal -UserId $User -LogonType Interactive -RunLevel Limited
$settings = New-ScheduledTaskSettingsSet -ExecutionTimeLimit ([TimeSpan]::Zero) -MultipleInstances IgnoreNew `
    -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1) -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable
Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $triggers -Principal $principal -Settings $settings -Force | Out-Null
Start-ScheduledTask -TaskName $TaskName

Write-Host ""
Write-Host "Installed: the Fractadyne field agent is running." -ForegroundColor Green
Write-Host "  agent  : $(Join-Path $Dest 'field-agent.ps1') (hidden, in your session, as $User)"
Write-Host "  share  : $Share\field\"
Write-Host "  runs a job only when this machine is unlocked, idle for $IdleMinutes min, and Fractadyne is closed"
Write-Host ""
Write-Host "Pause with -Disable, resume with -Enable, remove with -Uninstall. -Status shows what it is doing."
Start-Sleep -Seconds 3
Show-Status
