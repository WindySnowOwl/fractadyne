# field-add-machine.ps1 - make this computer a Fractadyne test machine the dev box can run tests on.
# Run it ON the new machine, as the user normally logged on there, from the share:
#
#   powershell -ExecutionPolicy Bypass -File \\<host>\share\Fractadyne\field\setup\field-add-machine.ps1 -Name VENUS
#
# (pwsh works too.) It does four things, and says what it did after each:
#
#   1. Checks this is the machine named -Name (its COMPUTERNAME): the dev box addresses requests by
#      that name (field-request.ps1 -Machine VENUS), and an agent takes only requests for its own.
#   2. Writes what this machine is to the share, field\agent\<NAME>-setup.txt: model, CPU, memory,
#      Windows build, each graphics adapter and its driver, whether Vulkan is installed, Windows'
#      GPU timeout. The dev box reads it to know which graphics APIs to test.
#   3. Keeps it awake while plugged in: sleep AND hibernate "never" on AC power. Eris (2026-11-09)
#      had sleep off and still went quiet after exactly 3 hours idle: Windows' separate "hibernate
#      after" timer. Skip with -KeepPowerSettings. Battery settings are not touched.
#   4. Installs the field agent with field-agent-setup.ps1 from beside this script (a scheduled
#      task, hidden, in your session; it updates itself from the share afterwards).
#
# -ReportOnly does steps 1 and 2 alone. Re-running it is safe: it rewrites the report, sets the same
# power values and reinstalls the agent.
#
# ASCII-only on purpose: Windows PowerShell 5.1 reads a BOM-less .ps1 as ANSI.

[CmdletBinding()]
param(
    # The name the dev box will use for this machine: must be its COMPUTERNAME (case does not matter).
    [Parameter(Mandatory = $true)][string]$Name,
    # The Fractadyne share root. Default: $env:FRACTADYNE_SHARE, else the share this script is run
    # from (...\Fractadyne\field\setup\ -> ...\Fractadyne).
    [string]$Share = "",
    # Minutes without keyboard or mouse input before a job may start (0 = do not wait).
    [int]$IdleMinutes = 5,
    # Leave the power settings as they are.
    [switch]$KeepPowerSettings,
    # Only steps 1 and 2: check the name and write the report; change nothing on this machine.
    [switch]$ReportOnly
)

$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path

function Say([string]$text, [string]$color = "") {
    if ($color) { Write-Host $text -ForegroundColor $color } else { Write-Host $text }
}

# --- 1. the name ----------------------------------------------------------------------------------
if ($env:COMPUTERNAME -ne $Name) {
    Say ("This computer is named '{0}', not '{1}'." -f $env:COMPUTERNAME, $Name) Red
    Say "Either run this on the right machine, rename this one first (Settings > System > About >"
    Say "Rename this PC, then restart), or pass -Name $($env:COMPUTERNAME)."
    exit 1
}
$Name = $env:COMPUTERNAME
Say ("1. Name: {0}" -f $Name) Green

if (-not $Share) { $Share = $env:FRACTADYNE_SHARE }
if (-not $Share) {
    if ((Split-Path -Leaf $here) -eq "setup" -and (Split-Path -Leaf (Split-Path -Parent $here)) -eq "field") {
        $Share = Split-Path -Parent (Split-Path -Parent $here)
    }
}
if (-not $Share) { throw "no share: run this from <share>\Fractadyne\field\setup\, or pass -Share" }
if (-not (Test-Path -LiteralPath $Share)) { throw "the share $Share is not reachable from this machine" }
$setup = Join-Path $here "field-agent-setup.ps1"
if (-not (Test-Path -LiteralPath $setup)) { throw "field-agent-setup.ps1 not found beside this script ($setup)" }

# --- 2. what this machine is ----------------------------------------------------------------------
function Read-Safely([scriptblock]$get) {
    try { & $get } catch { "(could not read: $($_.Exception.Message))" }
}

$lines = New-Object System.Collections.Generic.List[string]
$lines.Add(("machine   : {0}" -f $Name))
$lines.Add(("written   : {0:yyyy-MM-dd HH:mm:ss} local, {1:yyyy-MM-ddTHH:mm:ssZ}" -f (Get-Date), (Get-Date).ToUniversalTime()))
$lines.Add(("model     : {0}" -f (Read-Safely { $c = Get-CimInstance Win32_ComputerSystem; "$($c.Manufacturer) $($c.Model)" })))
$lines.Add(("cpu       : {0}" -f (Read-Safely { ((Get-CimInstance Win32_Processor) | ForEach-Object { $_.Name.Trim() }) -join "; " })))
$lines.Add(("memory    : {0}" -f (Read-Safely { "{0:N1} GB" -f ((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory / 1GB) })))
$lines.Add(("windows   : {0}" -f (Read-Safely { $o = Get-CimInstance Win32_OperatingSystem; "$($o.Caption) (build $($o.BuildNumber)), $($o.OSArchitecture)" })))
$lines.Add(("powershell: {0}{1}" -f $PSVersionTable.PSVersion, $(if (Get-Command pwsh -ErrorAction SilentlyContinue) { " (pwsh is installed)" } else { " (no pwsh: the agent runs on Windows PowerShell)" })))
try {
    $gpus = @(Get-CimInstance Win32_VideoController)
    if ($gpus.Count -eq 0) { $lines.Add("graphics  : (none listed)") }
    foreach ($g in $gpus) {
        $date = ""
        if ($g.DriverDate) { $date = " dated {0:yyyy-MM-dd}" -f $g.DriverDate }
        $mode = ""
        if ($g.CurrentHorizontalResolution) { $mode = ", {0}x{1}" -f $g.CurrentHorizontalResolution, $g.CurrentVerticalResolution }
        $lines.Add(("graphics  : {0}, driver {1}{2}{3}, status {4}" -f $g.Name, $g.DriverVersion, $date, $mode, $g.Status))
    }
} catch { $lines.Add("graphics  : (could not read: $($_.Exception.Message))") }
$vk = Join-Path $env:SystemRoot "System32\vulkan-1.dll"
$lines.Add(("vulkan    : {0}" -f $(if (Test-Path -LiteralPath $vk) { "loader installed ($((Get-Item -LiteralPath $vk).VersionInfo.FileVersion))" } else { "NO loader (vulkan-1.dll missing): OpenGL or DirectX 12 only, until a newer graphics driver" })))
$lines.Add(("gpu reset : {0}" -f (Read-Safely {
    $k = Get-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\GraphicsDrivers" -ErrorAction SilentlyContinue
    if ($k -and $k.PSObject.Properties["TdrDelay"]) { "TdrDelay $($k.TdrDelay) s" } else { "TdrDelay 2 s (the default)" }
})))
$lines.Add(("power     : {0}" -f (Read-Safely {
    $b = @(Get-CimInstance Win32_Battery)
    if ($b.Count -eq 0) { "no battery" } elseif ($b[0].BatteryStatus -eq 2) { "on AC power, battery $($b[0].EstimatedChargeRemaining)%" } else { "ON BATTERY ($($b[0].EstimatedChargeRemaining)%): plug it in for test runs" }
})))
$screens = Read-Safely {
    Add-Type -AssemblyName System.Windows.Forms
    ([System.Windows.Forms.Screen]::AllScreens | ForEach-Object { "$($_.DeviceName) $($_.Bounds.Width)x$($_.Bounds.Height)$(if ($_.Primary) { ' primary' })" }) -join "; "
}
$lines.Add(("screens   : {0}" -f $screens))

$agentDir = Join-Path (Join-Path $Share "field") "agent"
New-Item -ItemType Directory -Force -Path $agentDir | Out-Null
$report = Join-Path $agentDir ("{0}-setup.txt" -f $Name)
[IO.File]::WriteAllText($report, (($lines -join "`r`n") + "`r`n"), (New-Object System.Text.UTF8Encoding($false)))
Say ("2. What this machine is, written to {0}:" -f $report) Green
foreach ($l in $lines) { Say ("   " + $l) }
if ($ReportOnly) { Say "(-ReportOnly: nothing else changed.)"; exit 0 }

# --- 3. awake while plugged in --------------------------------------------------------------------
if ($KeepPowerSettings) {
    Say "3. Power settings left as they are (-KeepPowerSettings)." Yellow
} else {
    $failed = @()
    foreach ($setting in @("standby-timeout-ac", "hibernate-timeout-ac")) {
        & powercfg /change $setting 0 2>&1 | Out-Null
        if ($LASTEXITCODE -ne 0) { $failed += $setting }
    }
    if ($failed.Count -eq 0) {
        Say "3. Plugged in, this machine no longer sleeps or hibernates on its own (battery settings unchanged)." Green
    } else {
        Say ("3. Could not change: {0}. From PowerShell opened with 'Run as administrator':" -f ($failed -join ", ")) Yellow
        foreach ($s in $failed) { Say ("     powercfg /change {0} 0" -f $s) }
    }
    Say "   Keep it plugged in with the lid open: test windows need its screen."
}

# --- 4. the field agent ---------------------------------------------------------------------------
Say "4. Installing the field agent..." Green
try {
    & $setup -Share $Share -IdleMinutes $IdleMinutes
} catch {
    Say ("The agent install failed: {0}" -f $_.Exception.Message) Red
    Say "If it says 'Access is denied', run this same line once from PowerShell opened with"
    Say "'Run as administrator'. The agent still runs as you, not as an administrator."
    exit 1
}

Say ""
Say ("Done. The dev box can now ask for runs on this machine as -Machine {0}." -f $Name) Green
Say "Pause it with field-agent-setup.ps1 -Disable, resume with -Enable, remove with -Uninstall."
