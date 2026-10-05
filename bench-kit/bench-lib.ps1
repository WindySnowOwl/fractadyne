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
        return @{ status = 'DNF-timeout'; wall_s = [math]::Round($sw.Elapsed.TotalSeconds, 1)
                 wall_ms = [math]::Round($sw.Elapsed.TotalMilliseconds, 1); stdout = ''; stderr = '' }
    }
    $sw.Stop()
    @{
        status = $(if ($p.ExitCode -eq 0) { 'ok' } else { "DNF-exit$($p.ExitCode)" })
        wall_s = [math]::Round($sw.Elapsed.TotalSeconds, 1)
        # Unrounded, for the per-phase table: its phases are milliseconds, and a wall rounded to
        # 0.1 s cannot hold the remainder they leave.
        wall_ms = [math]::Round($sw.Elapsed.TotalMilliseconds, 1)
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
#     image reaches disk later. A lane that checks the file straight after the render finds
#     nothing and scores a DNF for a frame that rendered perfectly. And the NEXT render waits for
#     that encode, so back-to-back client times each carry the previous image's encode: the lane
#     waits for each PNG to be complete (Wait-PngComplete) and times the image to that point.
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

# Wait until a PNG is COMPLETE on disk - its last chunk is IEND - and say how long that took.
# FractalShark's server encodes each image in the background after the client returns, and the
# NEXT render waits for that encode before it starts. Timing client calls back to back therefore
# charged every image with the PREVIOUS image's encode (measured 2026-10-04 at its author's
# request: the spar scene's 69 ms render took 3.5 s, waiting for scene 17's 12.7 MB PNG). The lane
# waits here after each image, so each time is that image's render plus its OWN encode, and the
# next image starts with nothing queued. Read with sharing, so a file still being written is read
# as incomplete rather than locked.
function Wait-PngComplete($png, $timeoutS) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.Elapsed.TotalSeconds -lt $timeoutS) {
        if (Test-Path -LiteralPath $png) {
            try {
                $fs = [System.IO.File]::Open($png, 'Open', 'Read', 'ReadWrite, Delete')
                try {
                    if ($fs.Length -ge 20) {
                        $null = $fs.Seek(-12, 'End')
                        $b = New-Object byte[] 12
                        $null = $fs.Read($b, 0, 12)
                        if ([System.Text.Encoding]::ASCII.GetString($b, 4, 4) -eq 'IEND') {
                            return @{ ok = $true; ms = [math]::Round($sw.Elapsed.TotalMilliseconds, 1) }
                        }
                    }
                } finally { $fs.Dispose() }
            } catch {}
        }
        Start-Sleep -Milliseconds 10
    }
    @{ ok = $false; ms = [math]::Round($sw.Elapsed.TotalMilliseconds, 1) }
}

# FractalShark's own figures for one image, from the report a client call prints without --quiet:
# Overall = reference orbit + LA tables + per-pixel, all in ms, and "Frame time". Empty strings
# when absent. Never used to judge a row (a REFUSED render prints a Frame time too).
function Read-SharkPhases($text) {
    $p = @{ frame_time_ms = ''; overall_ms = ''; per_pixel_ms = ''; ref_orbit_ms = ''; la_ms = '' }
    $map = @{ overall_ms = 'Overall \(ms\) = ([0-9.]+)'; per_pixel_ms = 'Per pixel \(ms\) = ([0-9.]+)'
              ref_orbit_ms = 'RefOrbit \(ms\) = ([0-9.]+)'; la_ms = 'LA generation time \(ms\) = ([0-9.]+)'
              frame_time_ms = 'Frame time:\s*([0-9.]+)\s*ms' }
    foreach ($k in $map.Keys) { if ($text -match $map[$k]) { $p[$k] = $Matches[1] } }
    $p
}

$script:FsPhaseCols = @('scene', 'rep', 'status', 'wall_ms', 'client_ms', 'png_ms', 'frame_time_ms', 'overall_ms', 'ref_orbit_ms', 'la_ms', 'per_pixel_ms')

function Write-FsPhases($csvPath, $row) {
    if (-not (Test-Path $csvPath)) {
        ($script:FsPhaseCols -join ',') | Out-File -FilePath $csvPath -Encoding ascii
    }
    $vals = foreach ($c in $script:FsPhaseCols) { [string]$row[$c] }
    Add-Content -Path $csvPath -Value ($vals -join ',') -Encoding ascii
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

# ---- Fractadyne per-phase figures, from the render's own log --------------------------------
# A wall time says HOW LONG; it cannot say WHERE. Every fd render logs its phases anyway (the
# `[+ s]` stamp on each line, the `[fd-perf]` GPU and step line, and under FRACTADYNE_TRACE=ref
# the reference pick/orbit/SA/BLA times), so the lane keeps each render's log and this turns it
# into one row of numbers. Phases, all in ms:
#   startup     process start to the CLI render starting (window, GPU adapter, fonts)
#   ref_wait    how long the render waited for the reference, which is built from the first
#               millisecond BESIDE startup; pick/orbit/sa/bla are that build's own costs, mostly
#               hidden behind startup, so they do not add up to the wall
#   render      CLI render start to the [fd-perf] line: ref_wait + GPU + host work
#   gpu_iterate / gpu_color   pure GPU pass time (timestamp queries; chunk walls when chunked)
#   cpu_other   render - ref_wait - gpu_iterate - gpu_color: readback, normalization, tiling
#   write       PNG encode + write (the file-write line; b149+)
#   exit        the rest up to the [fd-exit] line
#   outside     wall - the last log stamp: OS process launch, DLL load, teardown
# Never throws; a missing log or an older build that lacks a line leaves that column empty.
# Get-Content -Encoding UTF8: the log carries arrows and dashes, and 5.1 reads BOM-less as ANSI.
$script:FdPhaseCols = @('scene', 'rep', 'status', 'wall_ms', 'startup_ms', 'ref_wait_ms',
    'early_ref', 'early_lead_ms', 'ref_builds', 'pick_ms', 'orbit_ms', 'sa_ms', 'bla_ms',
    'overlap', 'render_ms', 'gpu_iterate_ms', 'gpu_color_ms', 'max_dispatch_ms', 'cpu_other_ms',
    'write_ms', 'file_bytes', 'exit_ms', 'outside_ms', 'mode', 'iter', 'tiles', 'passes', 'rebase', 'ext', 'glitch',
    'bla_skip', 'maxiter', 'step_px', 'step_executed', 'step_iterations', 'iters_per_step',
    'step_full', 'df32_pct', 'logcheck')

function Read-FdPhases($logPath, $wallMs) {
    $p = [ordered]@{}
    foreach ($c in $script:FdPhaseCols) { $p[$c] = '' }
    $p.wall_ms = $wallMs
    if (-not $logPath -or -not (Test-Path $logPath)) { return $p }
    $tRender = $null; $tPerf = $null; $tExit = $null; $tLast = $null
    $orbit = 0.0; $sa = 0.0; $bla = 0.0; $pick = 0.0; $builds = 0; $picks = 0
    $num = [System.Globalization.CultureInfo]::InvariantCulture
    foreach ($line in (Get-Content -Encoding UTF8 $logPath)) {
        if ($line -notmatch '^\[\+\s*([0-9.]+)s\]') { continue }
        $t = [double]::Parse($Matches[1], $num) * 1000.0
        $tLast = $t
        if ($null -eq $tRender -and $line -match '\[crumb\] \(main\) CLI render') { $tRender = $t }
        elseif ($line -match 'early reference ([A-Z]+)') {
            $p.early_ref = $Matches[1]
            if ($line -match 'started (\d+) ms before') { $p.early_lead_ms = [int]$Matches[1] }
            if ($line -match 'waited (\d+) ms') { $p.ref_wait_ms = [int]$Matches[1] }
        }
        elseif ($line -match 'pick_reference \(candidate scoring\) took (\d+)ms') { $pick += [double]$Matches[1]; $picks++ }
        elseif ($line -match 'orbit_ms=(\d+) sa_ms=(\d+) bla_ms=(\d+)') {
            $orbit += [double]$Matches[1]; $sa += [double]$Matches[2]; $bla += [double]$Matches[3]; $builds++
        }
        elseif ($line -match 'overlap: centre build ([A-Z]+)') { $p.overlap = $Matches[1] }
        elseif ($null -eq $tPerf -and $line -match '\[fd-perf\] cli-render: ') {
            $tPerf = $t
            $keys = @{ mode = 'mode=(\d+)'; iter = ' iter=(\d+)'
                       gpu_iterate_ms = 'gpu_iterate=([0-9.]+)ms'; gpu_color_ms = 'gpu_color=([0-9.]+)ms'
                       max_dispatch_ms = 'max_dispatch=([0-9.]+)ms'; rebase = 'rebase=(\d+)'
                       tiles = ' tiles=(\d+)'; passes = ' passes=(\d+)'
                       ext = ' ext=(\d+)'; glitch = 'glitch=(\d+)'; bla_skip = 'bla_skip=(\d+)'
                       maxiter = 'maxiter=(\d+)'; step_px = ' px=(\d+)'; step_executed = 'executed=(\d+)'
                       step_iterations = 'iterations=(\d+)'; iters_per_step = '= ([0-9.]+) per step'
                       step_full = 'full=(\d+)'; df32_pct = 'in df32 ([0-9.]+)%' }
            foreach ($k in $keys.Keys) { if ($line -match $keys[$k]) { $p[$k] = $Matches[1] } }
        }
        elseif ($line -match 'file-write: \w+ \d+x\d+ (\d+) bytes in ([0-9.]+)ms') {
            $p.file_bytes = $Matches[1]; $p.write_ms = $Matches[2]
        }
        elseif ($line -match '\[fd-exit\]') { $tExit = $t }
        elseif ($line -match '\[fd-logcheck\] ([A-Z]+)') { $p.logcheck = $Matches[1] }
    }
    $r1 = { param($v) [math]::Round([double]$v, 1) }
    if ($null -ne $tRender) { $p.startup_ms = & $r1 $tRender }
    if ($builds) { $p.ref_builds = $builds; $p.orbit_ms = $orbit; $p.sa_ms = $sa; $p.bla_ms = $bla }
    if ($picks) { $p.pick_ms = $pick }
    if ($null -ne $tRender -and $null -ne $tPerf) {
        $p.render_ms = & $r1 ($tPerf - $tRender)
        if ([string]$p.gpu_iterate_ms -ne '') {
            $wait = 0.0; if ([string]$p.ref_wait_ms -ne '') { $wait = [double]$p.ref_wait_ms }
            $p.cpu_other_ms = & $r1 ($p.render_ms - $wait - [double]::Parse($p.gpu_iterate_ms, $num) - [double]::Parse($p.gpu_color_ms, $num))
        }
    }
    if ($null -ne $tPerf -and $null -ne $tExit) {
        $w = 0.0; if ([string]$p.write_ms -ne '') { $w = [double]::Parse($p.write_ms, $num) }
        $p.exit_ms = & $r1 ($tExit - $tPerf - $w)
    }
    if ($null -ne $tLast -and [string]$wallMs -ne '') { $p.outside_ms = & $r1 ([double]$wallMs - $tLast) }
    $p
}

function Write-FdPhases($csvPath, $row) {
    if (-not (Test-Path $csvPath)) {
        ($script:FdPhaseCols -join ',') | Out-File -FilePath $csvPath -Encoding ascii
    }
    $vals = foreach ($c in $script:FdPhaseCols) { [string]$row[$c] }
    Add-Content -Path $csvPath -Value ($vals -join ',') -Encoding ascii
}

# Median of the numeric values in a list (empty strings dropped); $null when none.
# [string] on the left: `0 -ne ''` is FALSE in PowerShell (the '' is converted to 0), which
# silently dropped every zero reading from the median.
function Get-Median($values) {
    $v = @($values | Where-Object { $null -ne $_ -and [string]$_ -ne '' } | ForEach-Object { [double]$_ } | Sort-Object)
    if (-not $v.Count) { return $null }
    $m = [int][math]::Floor($v.Count / 2)
    if ($v.Count % 2) { return $v[$m] }
    ($v[$m - 1] + $v[$m]) / 2.0
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
