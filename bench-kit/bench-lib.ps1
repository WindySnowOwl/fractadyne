# bench-lib.ps1 - shared helpers for the Fractadyne benchmark kit. ASCII only (see
# CONTRIBUTING: PowerShell 5.1 reads BOM-less files as ANSI; keep every .ps1 plain ASCII).

Set-StrictMode -Version 2

# Parse a .kfr (Kalles Fraktaler text) location: Re, Im, Zoom, Iterations.
# Zoom exponents exceed double range (4.6E1105), so the magnitude is split textually:
# returns mag_log10 = log10(mantissa) + exponent without ever forming the number.
function Read-Kfr($path) {
    $kv = @{}
    foreach ($line in Get-Content $path) {
        if ($line -match '^\s*([A-Za-z]+)\s*:\s*(.+?)\s*$') { $kv[$Matches[1]] = $Matches[2] }
    }
    $zoom = $kv['Zoom']
    if ($zoom -match '^([0-9.]+)[eE]\+?(-?[0-9]+)$') {
        $kv['MagLog10'] = [math]::Log10([double]$Matches[1]) + [double]$Matches[2]
    } else {
        $kv['MagLog10'] = [math]::Log10([double]$zoom)
    }
    $kv
}

function Read-Scenes($kitRoot) {
    Import-Csv (Join-Path $kitRoot 'scenes.csv')
}

# Run one external render attempt and time it. Returns a result object; never throws.
# The timeout is a DNF, not an error - a renderer that cannot finish is a data point.
function Invoke-TimedRender($exe, $args, $timeoutS, $cwd) {
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $exe
    $psi.Arguments = $args
    $psi.WorkingDirectory = $cwd
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $p = [System.Diagnostics.Process]::Start($psi)
    $out = $p.StandardOutput.ReadToEndAsync()
    $err = $p.StandardError.ReadToEndAsync()
    if (-not $p.WaitForExit($timeoutS * 1000)) {
        try { $p.Kill() } catch {}
        $sw.Stop()
        return @{ status = 'DNF-timeout'; wall_s = [math]::Round($sw.Elapsed.TotalSeconds, 1); stdout = ''; stderr = '' }
    }
    $sw.Stop()
    @{
        status = $(if ($p.ExitCode -eq 0) { 'ok' } else { "DNF-exit$($p.ExitCode)" })
        wall_s = [math]::Round($sw.Elapsed.TotalSeconds, 1)
        stdout = $out.Result
        stderr = $err.Result
    }
}

# Append one row to results.csv (schema: renderer,scene,rep,status,wall_s,reported_s,note).
function Write-Result($csvPath, $renderer, $scene, $rep, $status, $wallS, $reportedS, $note) {
    if (-not (Test-Path $csvPath)) {
        'renderer,scene,rep,status,wall_s,reported_s,note' | Out-File -FilePath $csvPath -Encoding ascii
    }
    $line = '{0},{1},{2},{3},{4},{5},"{6}"' -f $renderer, $scene, $rep, $status, $wallS, $reportedS, ($note -replace '"', "'")
    Add-Content -Path $csvPath -Value $line -Encoding ascii
    Write-Host ('  {0,-13} {1,-18} rep{2}  {3,-12} wall={4,-8} reported={5}' -f $renderer, $scene, $rep, $status, $wallS, $reportedS)
}

# System facts for the results header: honest hardware disclosure or the numbers mean nothing.
function Write-SysInfo($path) {
    $lines = @()
    $lines += 'Fractadyne benchmark kit - system info'
    $lines += 'Timestamp: ' + (Get-Date -Format 'yyyy-MM-dd HH:mm:ss') + ' (local)'
    $lines += 'Host: ' + $env:COMPUTERNAME
    $os = Get-CimInstance Win32_OperatingSystem
    $lines += 'OS: ' + $os.Caption + ' ' + $os.Version
    $cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
    $lines += 'CPU: ' + $cpu.Name.Trim() + ' (' + $cpu.NumberOfCores + 'C/' + $cpu.NumberOfLogicalProcessors + 'T)'
    $ram = [math]::Round((Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory / 1GB, 1)
    $lines += 'RAM: ' + $ram + ' GB'
    foreach ($gpu in Get-CimInstance Win32_VideoController) {
        $lines += 'GPU: ' + $gpu.Name + ' (driver ' + $gpu.DriverVersion + ')'
    }
    $plan = (powercfg /getactivescheme) -join ''
    if ($plan -match '\((.+)\)') { $lines += 'Power plan: ' + $Matches[1] }
    $lines | Out-File -FilePath $path -Encoding ascii
    Write-Host ($lines -join "`n")
}

# Prompt-based lane for renderers with no headless mode (Imagina, FractalShark): launch the
# app on the scene file, the operator renders and TRANSCRIBES the app's own reported time.
function Invoke-AssistedLane($renderer, $exe, $sceneFile, $sceneName, $rep, $csvPath, $hints) {
    Write-Host ''
    Write-Host ('--- {0} / {1} (rep {2}) ---' -f $renderer, $sceneName, $rep)
    Write-Host ($hints -join "`n")
    Write-Host ('Launching: {0} "{1}"' -f $exe, $sceneFile)
    Start-Process -FilePath $exe -ArgumentList ('"' + $sceneFile + '"') | Out-Null
    $ans = Read-Host 'Enter the render time the app reports IN SECONDS (or DNF <reason>)'
    if ($ans -match '^(?i)dnf') {
        Write-Result $csvPath $renderer $sceneName $rep 'DNF-operator' '' '' ($ans -replace '^(?i)dnf\s*', '')
    } elseif ($ans -match '^[0-9.]+$') {
        Write-Result $csvPath $renderer $sceneName $rep 'ok' '' $ans 'operator-transcribed'
    } else {
        Write-Result $csvPath $renderer $sceneName $rep 'DNF-operator' '' '' ('unparseable entry: ' + $ans)
    }
}
