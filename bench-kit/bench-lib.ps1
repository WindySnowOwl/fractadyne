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

# ---- run manifest: what was actually EXECUTED, per render -------------------------------------
# results.csv answers "how long did it take". It has never answered "what exactly did you run",
# and that is the question a reader needs in order to reproduce or challenge a number. The gap is
# not theoretical: on 2026-09-21 the FractalShark lane spent its whole life sending a zoom the
# renderer silently truncated, and nothing written to disk recorded the argument that did it.
# Every automated render now files the exe, the full argument line, the working directory, the
# parsed inputs and the output it produced.
$script:RunLog = New-Object System.Collections.ArrayList

function Add-RunRecord($rec) { [void]$script:RunLog.Add($rec) }

function Get-RunLog { , $script:RunLog }

# ConvertTo-Json defaults to -Depth 2 in 5.1, which silently renders nested hashtables as the
# literal string "System.Collections.Hashtable". The inputs live one level down, so this must be
# explicit or the manifest is worthless in exactly the field that matters.
function Save-RunManifest($path, $meta) {
    $doc = @{ meta = $meta; runs = @($script:RunLog) }
    $doc | ConvertTo-Json -Depth 8 | Out-File -FilePath $path -Encoding ascii
}

# sha256 + size, for the appendix that says which binary produced these numbers. Never throws:
# a missing or locked exe must not take a whole run's report down at the last step.
function Get-ExeStamp($path) {
    if (-not $path -or -not (Test-Path $path)) { return $null }
    try {
        $fi = Get-Item $path
        @{ path = $fi.FullName; bytes = $fi.Length
           sha256 = (Get-FileHash $path -Algorithm SHA256).Hash
           modified = $fi.LastWriteTime.ToString('s') }
    } catch { @{ path = $path; bytes = $null; sha256 = $null; modified = $null } }
}

# Run one external render attempt and time it. Returns a result object; never throws.
# The timeout is a DNF, not an error - a renderer that cannot finish is a data point.
# $argLine, never $args: $args is the automatic variable, and binding a parameter over it is
# exactly the silent-empty-arguments trap that had the first kit run launch a GUI (2026-08-21).
function Invoke-TimedRender($exe, $argLine, $timeoutS, $cwd) {
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $exe
    $psi.Arguments = $argLine
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

# ---------------------------------------------------------------------------------------------
# FractalShark 0.543 client/server lane.
#
# WHY IT EXISTS. FractalShark's author added --server/--connect in 0.543 in answer to this kit's
# timing writeup: the server pays CUDA + process initialization ONCE, so what a client call costs
# is close to the frame itself. Measured here on an RTX 3080, scene 03 at 1.33e6, three reps:
# one process per frame 1776 ms median, client against a live server 399 ms median. About 1.4 s
# per frame of the old number was startup, not rendering -- which is most of it at shallow depth
# and would have swamped any comparison this kit published.
#
# THREE PROTOCOL FACTS, each of which silently corrupts a result if ignored. All measured, not
# assumed (tools/fs-server-probe.sh reproduces them):
#
#  1. PNG ENCODING IS ASYNCHRONOUS. The client returns as soon as the frame is computed; the
#     image reaches disk later, and --shutdown is what flushes it. A lane that checks the file
#     straight after the render finds nothing and scores a DNF for a frame that rendered
#     perfectly. Hence: render everything, shut the server down, THEN validate.
#  2. AN ERROR STILL PRINTS "Frame time". Ask for an algorithm that cannot represent the
#     viewport's pixel spacing and the client prints `error: ... cannot represent ...` AND
#     `Frame time: 17.1 ms`, writing no image. Anything scraping the reported time records a
#     fast frame that never happened. The EXIT CODE is honest (1 here, 2 for bad arguments, in
#     both single-shot and connect modes), so gate on that and on the image, never on the text.
#  3. THE ALGORITHM MUST MATCH THE DEPTH. Gpu1x32PerturbedLAv2 fails past f32 pixel spacing;
#     GpuHDRx32PerturbedLAv2 renders 4.2e275 correctly. There is NO "Auto" despite the error
#     message suggesting one -- `--render-algorithm Auto` is rejected as an unknown name.
#
# Starts the server and waits for it to say it is listening, rather than sleeping a guess: a
# fixed sleep either wastes seconds per run or races on a slow CUDA init.
function Start-SharkServer($cli, $endpoint, $w, $h, $logDir) {
    $log = Join-Path $logDir ('fs-server-' + $endpoint + '.log')
    if (Test-Path $log) { Remove-Item -Force $log }
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $cli
    $psi.Arguments = ('--server --endpoint {0} --width {1} --height {2}' -f $endpoint, $w, $h)
    $psi.UseShellExecute = $false
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $p = [System.Diagnostics.Process]::Start($psi)
    # Drain both pipes to a file: a server whose stdout buffer fills stops serving.
    $so = $p.StandardOutput.ReadToEndAsync()
    $se = $p.StandardError.ReadToEndAsync()
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $ready = $false
    while ($sw.Elapsed.TotalSeconds -lt 90) {
        if ($p.HasExited) { break }
        # ReadToEndAsync only completes at exit, so probe the pipe by connecting instead.
        Start-Sleep -Milliseconds 500
        $probe = & $cli --connect --endpoint $endpoint --list-render-algorithms 2>&1
        if ($LASTEXITCODE -eq 0) { $ready = $true; break }
    }
    $sw.Stop()
    @{ ok = $ready; proc = $p; stdout = $so; stderr = $se; log = $log
       startup_s = [math]::Round($sw.Elapsed.TotalSeconds, 1) }
}

# Shut the server down and WAIT for it, because that is what flushes the pending PNG writes.
function Stop-SharkServer($cli, $endpoint, $srv) {
    & $cli --connect --endpoint $endpoint --shutdown 2>&1 | Out-Null
    if ($srv -and $srv.proc) {
        if (-not $srv.proc.WaitForExit(120000)) { try { $srv.proc.Kill() } catch {} }
        try {
            $text = $srv.stdout.Result + "`n" + $srv.stderr.Result
            $text | Out-File -FilePath $srv.log -Encoding ascii
        } catch {}
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

# Is this render actually a picture, or a flat field? A renderer that writes a uniform image and
# exits 0 is the most expensive kind of wrong: this kit once published "144x faster than
# Fraktaler-3" for a frame that was entirely blank. FractalSharkCli 0.532 does exactly that for
# every GPU algorithm in headless mode, so no lane may record a TIME without first checking there
# is an IMAGE. Samples a grid rather than every pixel: a flat image is flat everywhere.
function Test-RenderHasStructure($path) {
    if (-not (Test-Path $path)) { return $false }
    try {
        Add-Type -AssemblyName System.Drawing -ErrorAction Stop
        $bmp = [System.Drawing.Bitmap]::FromFile((Resolve-Path $path).Path)
        try {
            # Demand REAL variation, not just two differing pixels. A near-flat blank with a
            # handful of stray edge pixels used to pass the old "any two differ" test - that is
            # exactly how a blank GPU frame got scored as a render. Sample a dense grid and
            # require many distinct colours AND no single colour dominating, matching
            # tools/image-structure.py (distinct > 16 and modal share < 0.98).
            $counts = @{}
            $total = 0
            $N = 96
            for ($i = 0; $i -lt $N; $i++) {
                for ($j = 0; $j -lt $N; $j++) {
                    $x = [int](($bmp.Width  - 1) * $i / ($N - 1))
                    $y = [int](($bmp.Height - 1) * $j / ($N - 1))
                    $c = $bmp.GetPixel($x, $y).ToArgb()
                    if ($counts.ContainsKey($c)) { $counts[$c]++ } else { $counts[$c] = 1 }
                    $total++
                }
            }
            $modal = 0
            foreach ($v in $counts.Values) { if ($v -gt $modal) { $modal = $v } }
            return ($counts.Count -gt 16 -and ($modal / $total) -lt 0.98)
        } finally { $bmp.Dispose() }
    } catch {
        # Unreadable is not the same as flat. Say so rather than silently passing or failing.
        Write-Host ("  (could not inspect " + (Split-Path $path -Leaf) + ": " + $_.Exception.Message + ")")
        return $true
    }
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
