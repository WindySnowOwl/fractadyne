# run-all.ps1 - the Fractadyne relative-performance benchmark. See README.md for the protocol.
# ASCII only (PowerShell 5.1 reads BOM-less files as ANSI; no smart punctuation).
#
#   powershell -ExecutionPolicy Bypass -File run-all.ps1 [-Reps 2] [-Skip imagina,fractalshark]
#       [-FractadyneExe path] [-Fraktaler3Exe path] [-ImaginaExe path] [-FractalSharkExe path]
#       [-TimeoutS 7200] [-Size 3840x2160] [-ZoomSeqFrames 8] [-PythonExe python]
#       [-F3Wisdom path] [-FractalSharkCliExe path] [-FractalSharkAlgo NAME]

[CmdletBinding()]
param(
    [int]$Reps = 1,
    [string[]]$Skip = @(),
    [string]$FractadyneExe = '',
    [string]$Fraktaler3Exe = '',
    [string]$ImaginaExe = '',
    [string]$FractalSharkExe = '',
    [int]$TimeoutS = 7200,
    [string[]]$Scenes = @(),
    # Render size for EVERY lane. 4K because it is what someone actually renders at, and
    # because a benchmark should be a real load: 3840x2160 is 4x the old Fractadyne size
    # and 9x the size Fraktaler-3 was silently given.
    [ValidatePattern('^[0-9]+x[0-9]+$')]
    [string]$Size = '3840x2160',
    # ZOOM SEQUENCE lane. Every other scene here is ONE FRAME, and a single frame is blind to
    # the optimisation that matters most for zoom video: a dive toward a fixed centre keeps the
    # same reference orbit valid across many frames, so the expensive setup can be amortised
    # instead of paid per frame. The metric is each app measured against ITSELF -
    # amortisation = (frames x single_frame_wall) / sequence_wall - so it needs no cross-app
    # calibration. Set 0 to skip the lane (or -Skip zoomseq).
    [int]$ZoomSeqFrames = 8,
    # The ladder places its rungs with 400-digit decimal arithmetic, which is why that lane is
    # Python. Without an interpreter the lane records itself as skipped; the rest of the run is
    # unaffected.
    [string]$PythonExe = '',
    # Fraktaler-3's hardware tuning. The comment on the generator below has always said "once per
    # machine" but the file lived in the per-run results folder, so every run re-derived it -- and
    # the benchmark half of that step is bounded at 1800s, which is a long time to spend measuring
    # the same hardware again. It now persists beside the kit and is COPIED into each run folder
    # for provenance. Point -F3Wisdom at a file to reuse or share one.
    [string]$F3Wisdom = '',
    # FractalShark ships a headless renderer, FractalSharkCli.exe, beside the GUI. Point at it to
    # automate the lane; left empty it is looked for next to -FractalSharkExe.
    [string]$FractalSharkCliExe = '',
    # WARNING: CPU BY DEFAULT, AND THAT IS NOT A PREFERENCE. In FractalShark 0.532 every GPU
    # render algorithm returns an EMPTY image headlessly: each pixel comes back with iteration
    # count 1, the PNG is a single flat colour, and the exit status is 0. The CLI says so itself
    # on the --console path ("all exterior pixels have the same iteration count 1"), and its
    # stderr carries "OpenGL context creation FAILED, no rendering will occur" - the GPU results
    # never reach the image. Upstream CI smoke-tests Cpu64 only, so it does not see this.
    # AutoSelect picks a GPU algorithm, so the obvious invocation is silently broken.
    #
    # The CPU algorithms render correctly at shallow and mid depth - verified 1e6 through 1e27 -
    # but ALSO come back blank on the deeper corpus locations (5.07e27, 6.6e43, 1.2e148, 1.47e77,
    # 4.6e1105), and that is independent of how many digits of centre they are given (40, 60, 100
    # and 196 all blank). So this lane produces real numbers for the shallow scenes and honest
    # DNF-blank rows for the rest. That is the point of the guard: never a TIME without an IMAGE.
    [string]$FractalSharkAlgo = 'Cpu64PerturbedBLAV2HDR'
)

$ErrorActionPreference = 'Stop'
$kit = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $kit 'bench-lib.ps1')

# `powershell -File` does NOT parse `-Skip imagina,fractalshark` into an array the way an
# interactive call does - the whole thing arrives as one comma-bearing string. Normalize both
# list parameters so the documented invocation actually works.
$Skip = @($Skip | ForEach-Object { $_ -split ',' } | Where-Object { $_ })
$Scenes = @($Scenes | ForEach-Object { $_ -split ',' } | Where-Object { $_ })

# ---- resolve tools ----
if (-not $FractadyneExe) { $FractadyneExe = Join-Path $kit 'bin\fractadyne.exe' }
if (-not $Fraktaler3Exe) { $Fraktaler3Exe = Join-Path $kit 'fraktaler3\fraktaler-3.exe' }
$have = @{
    fractadyne   = (Test-Path $FractadyneExe)
    fraktaler3   = (Test-Path $Fraktaler3Exe)
    imagina      = ($ImaginaExe -and (Test-Path $ImaginaExe))
    fractalshark = ($FractalSharkExe -and (Test-Path $FractalSharkExe))
}
# The headless binary sits beside the GUI in the release zip.
if (-not $FractalSharkCliExe -and $FractalSharkExe) {
    $cand = Join-Path (Split-Path -Parent $FractalSharkExe) 'FractalSharkCli.exe'
    if (Test-Path $cand) { $FractalSharkCliExe = $cand }
}
$have += @{ fractalsharkcli = ($FractalSharkCliExe -and (Test-Path $FractalSharkCliExe)) }
# The sequence lane needs Python and at least one automated renderer to compare against.
if (-not $PythonExe) {
    $py = Get-Command python -ErrorAction SilentlyContinue
    if ($py) { $PythonExe = $py.Source }
}
$zsScript = Join-Path $kit 'zoom-seq.py'
$have += @{
    zoomseq = ($ZoomSeqFrames -gt 0 -and $PythonExe -and (Test-Path $zsScript) -and ($have.fractadyne -or $have.fraktaler3))
}
foreach ($k in $Skip) { $have[$k.ToLower()] = $false }
# FractalShark's GUI lane is CUDA: without an NVIDIA GPU it is N/A, not a failure. The CLI lane
# runs a CPU algorithm (see -FractalSharkAlgo) and is NOT gated on the GPU.
$fsNa = $false
if ($have.fractalshark) {
    $nv = Get-CimInstance Win32_VideoController | Where-Object { $_.Name -match 'NVIDIA' }
    if (-not $nv) {
        Write-Host 'FractalShark GUI lane: no NVIDIA GPU detected - recording N/A.'
        $have.fractalshark = $false
        $fsNa = $true
    }
}
# The automated CLI lane supersedes the assisted one when it is available: transcription is for
# renderers with no headless mode, and this one has had a headless mode all along.
if ($have.fractalsharkcli) { $have.fractalshark = $false; $fsNa = $false }

# ---- results folder ----
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$outDir = Join-Path $kit ('results\' + $env:COMPUTERNAME + '-' + $stamp)
New-Item -ItemType Directory -Force $outDir | Out-Null
$csv = Join-Path $outDir 'results.csv'
Write-SysInfo (Join-Path $outDir 'sysinfo.txt')
# -Scenes filters by slug or two-digit id: a smoke run or a re-measure of one regime should
# not have to pay for all six. The full set stays the default (and the published protocol).
# The data rows are $sceneRows, NOT $scenes: variables are case-insensitive and the
# [string[]]$Scenes parameter TYPE-CONSTRAINS the slot, so `$scenes = Import-Csv ...` would
# silently coerce every row to a string (and strict mode then fails on `.slug`).
$sceneFilter = @($Scenes)
$sceneRows = Read-Scenes $kit
if ($sceneFilter.Count) {
    $sceneRows = @($sceneRows | Where-Object { $sceneFilter -contains $_.slug -or $sceneFilter -contains $_.id })
    if (-not $sceneRows.Count) { Write-Host 'ERROR: -Scenes matched nothing in scenes.csv'; exit 1 }
}
Write-Host ''
Write-Host ('Lanes: ' + (($have.GetEnumerator() | Where-Object Value | ForEach-Object Key) -join ', '))
Write-Host ('Scenes: ' + ($sceneRows.slug -join ', '))
Write-Host ('Reps: ' + $Reps + '   Timeout per render: ' + $TimeoutS + 's')

# ---- lane: Fractadyne (automated; hermetic config so nothing local leaks in) ----
if ($have.fractadyne) {
    $cfg = Join-Path $outDir 'fd-config'
    foreach ($rep in 1..$Reps) {
        foreach ($s in $sceneRows) {
            if (Test-Path $cfg) { Remove-Item -Recurse -Force $cfg }
            New-Item -ItemType Directory -Force $cfg | Out-Null
            $env:FRACTADYNE_CONFIG_DIR = $cfg
            $kfr = Read-Kfr (Join-Path $kit ('scenes\' + $s.slug + '.kfr'))
            $png = Join-Path $outDir ('fd-' + $s.slug + '.png')
            $zl2 = [double]$s.mag_log10 * [math]::Log(10.0, 2.0)
            # $argLine, never $args: $args is PowerShell's AUTOMATIC variable and assigning it
            # at script scope silently does not stick in 5.1 - the render launched with NO
            # arguments and sat in the GUI event loop until the timeout.
            $argLine = ('--render --out "{0}" --size {5} --center {1} {2} --zoom-log2 {3} --iter {4} --ss 1 --palette 0' -f $png, $kfr['Re'], $kfr['Im'], $zl2, $s.iterations, $Size)
            if ($s.normalize -eq '1') { $argLine += ' --normalize' }
            $r = Invoke-TimedRender $FractadyneExe $argLine $TimeoutS $kit
            # The app prints "(in 40.8s)" / "(in 2m07s)" / "(in 1h02m)"; the CSV stores plain
            # SECONDS so the summary can compare numbers, not strings.
            $reported = ''
            if ($r.stdout -match '\(in ([0-9hms. ]+)\)') {
                # Spaces stripped: the app prints "2m 32.4s" for multi-minute renders, and the
                # pattern below has no room for one -- scene 10's reported_s came back EMPTY.
                $t = $Matches[1] -replace '\s', ''
                if ($t -match '^(?:(\d+)h)?(?:(\d+)m)?(?:([0-9.]+)s)?$') {
                    $reported = 3600 * [double]('0' + $Matches[1]) + 60 * [double]('0' + $Matches[2]) + [double]('0' + $Matches[3])
                }
            }
            Write-Result $csv 'fractadyne' $s.slug $rep $r.status $r.wall_s $reported ''
        }
    }
    Remove-Item Env:FRACTADYNE_CONFIG_DIR -ErrorAction SilentlyContinue
}

# Declared out here, not inside the lane: the zoom-sequence lane below reuses the SAME tuning
# file, and under Set-StrictMode an undefined variable is an error, not an empty string.
if (-not $F3Wisdom) { $F3Wisdom = Join-Path $kit 'f3-wisdom.toml' }
$wisdom = $F3Wisdom

# ---- lane: Fraktaler-3 (automated; wisdom generated once per machine) ----
if ($have.fraktaler3) {
    if (-not (Test-Path $wisdom)) {
        # The file goes through -w and the MODE flag follows: `-w path -W` writes the initial
        # hardware config there, `-w path -B` then benchmarks number types for optimal
        # efficiency (the real tuning; bounded - a timeout leaves the initial config in place,
        # which is only a slower F3, honestly noted in the console). A bare `-W "path"` treats
        # the path as an INPUT file and silently writes nothing - the first real kit run
        # benchmarked F3 on built-in defaults that way.
        Write-Host 'Fraktaler-3: generating + benchmarking tuning wisdom (once)...'
        $w = Invoke-TimedRender $Fraktaler3Exe ('-w "' + $wisdom + '" -W') 300 $outDir
        Write-Host ('  wisdom init: ' + $w.status + ' in ' + $w.wall_s + 's')
        $w = Invoke-TimedRender $Fraktaler3Exe ('-w "' + $wisdom + '" -B') 1800 $outDir
        Write-Host ('  wisdom benchmark: ' + $w.status + ' in ' + $w.wall_s + 's')
    } else {
        Write-Host ('Fraktaler-3: reusing tuning wisdom ' + $wisdom)
    }
    # Provenance: the results folder must say which tuning produced these numbers.
    Copy-Item $wisdom (Join-Path $outDir 'f3-wisdom.toml') -ErrorAction SilentlyContinue
    foreach ($rep in 1..$Reps) {
        foreach ($s in $sceneRows) {
            # SIZE AND SAMPLING PARITY. The scene .f3.toml is a CORRECTNESS fixture: it
            # carries the corpus's resolution AND `subframes = 4`, which is Fraktaler-3's
            # antialiasing sample count, chosen there to pair with the corpus's --ss 2 on the
            # Fractadyne side. This benchmark renders every lane at ONE sample per pixel (see
            # README, "supersampling off"), so copying the fixture verbatim had F3 doing 4x the
            # sampling work of the Fractadyne lane in every number this kit ever produced -
            # exactly the defect the size rewrite was added to fix, one field over. Rewrite both
            # into a per-run copy; never touch the corpus file, which also drives the
            # correctness references.
            $srcToml = Join-Path $kit ('scenes\' + $s.slug + '.f3.toml')
            $toml    = Join-Path $outDir ($s.slug + '.bench.f3.toml')
            $wh      = $Size -split 'x'
            $tomlTxt = (Get-Content $srcToml) `
                -replace '^\s*width\s*=.*',  ('width = '  + $wh[0]) `
                -replace '^\s*height\s*=.*', ('height = ' + $wh[1]) `
                -replace '^\s*subframes\s*=.*', 'subframes = 1'
            Set-Content -Path $toml -Value $tomlTxt -Encoding ASCII
            $argLine = ('-w "{0}" -b "{1}"' -f $wisdom, $toml)
            $r = Invoke-TimedRender $Fraktaler3Exe $argLine $TimeoutS $outDir
            Write-Result $csv 'fraktaler3' $s.slug $rep $r.status $r.wall_s '' ''
        }
    }
}

# ---- lane: FractalShark (automated, via FractalSharkCli) ----
# WARNING: read the algorithm note on -FractalSharkAlgo before changing it. Every GPU algorithm
# returns a BLANK image headlessly in 0.532, at exit status 0, so this lane checks that a render
# is actually a PICTURE before it records a TIME. What it measures is FractalShark's CPU path,
# which must be said wherever the number appears: it is not what the app is for.
if ($have.fractalsharkcli) {
    Write-Host ('FractalShark: automated via ' + (Split-Path $FractalSharkCliExe -Leaf) + ', algorithm ' + $FractalSharkAlgo)
    $wh = $Size -split 'x'
    foreach ($rep in 1..$Reps) {
        foreach ($s in $sceneRows) {
            $kfr = Read-Kfr (Join-Path $kit ('scenes\' + $s.slug + '.kfr'))
            # The magnification is a STRING, never a number: 10^1105.79 is +inf in double and the
            # corpus goes there. FractalSharkCli accepts a fractional exponent (verified), and its
            # zoom convention is Kalles Fraktaler's - the same one we and Fraktaler-3 use. That was
            # CALIBRATED against a known scene here, not assumed; a 4/3 alternative scored far worse.
            $zoom = '1e' + $s.mag_log10
            # WARNING: the output name may not be the one you pass. FractalSharkCli strips the
            # final extension and re-appends .png only when what remains has no dot, so
            # "fs-21-m43-spar-1e27.7.png" lands as "fs-21-m43-spar-1e27.7" with no extension - and
            # three of our slugs contain dots. Hand it a dot-free stem; let it add the extension.
            $stem = Join-Path $outDir ('fs-' + ($s.slug -replace '\.', 'p'))
            $png = $stem + '.png'
            # SAMPLING PARITY, restated rather than inherited: one sample per pixel, like every
            # other lane. This is the field that was silently 4x for Fraktaler-3 in every run this
            # kit ever published.
            $argLine = ('--render-algorithm {0} --center-x {1} --center-y {2} --zoom {3} --iterations {4} --width {5} --height {6} --antialiasing 1 --out "{7}" --quiet' -f `
                        $FractalSharkAlgo, $kfr['Re'], $kfr['Im'], $zoom, $s.iterations, $wh[0], $wh[1], $stem)
            $r = Invoke-TimedRender $FractalSharkCliExe $argLine $TimeoutS $outDir
            $note = 'CPU path; its GPU algorithms render blank headlessly in 0.532'
            $status = $r.status
            if ($status -eq 'ok') {
                if (-not (Test-Path $png)) {
                    $status = 'DNF-no-output'
                    $note = 'exit 0 but no PNG at the expected path'
                } elseif (-not (Test-RenderHasStructure $png)) {
                    $status = 'DNF-blank'
                    $note = 'exit 0 but the image is uniform - a time here would be meaningless'
                }
            }
            Write-Result $csv 'fractalshark' $s.slug $rep $status $r.wall_s '' $note
        }
    }
}

# ---- lanes: Imagina / FractalShark (operator-assisted; see README "honestly") ----
if ($have.imagina) {
    $hints = @(
        'Imagina opens with the scene .kfr. If it does not auto-render: render at 1920x1080,',
        'iteration limit as shown in the scene table, then read the computation time it reports.'
    )
    foreach ($rep in 1..$Reps) {
        foreach ($s in $sceneRows) {
            Invoke-AssistedLane 'imagina' $ImaginaExe (Join-Path $kit ('scenes\' + $s.slug + '.kfr')) $s.slug $rep $csv $hints
        }
    }
}
if ($have.fractalshark) {
    $hints = @(
        'FractalShark: load the scene .kfr (right-click > load location), render at 1920x1080',
        'with the scene iteration cap, then transcribe the render time it displays.'
    )
    foreach ($rep in 1..$Reps) {
        foreach ($s in $sceneRows) {
            Invoke-AssistedLane 'fractalshark' $FractalSharkExe (Join-Path $kit ('scenes\' + $s.slug + '.kfr')) $s.slug $rep $csv $hints
        }
    }
} elseif ($fsNa) {
    foreach ($s in $sceneRows) { Write-Result $csv 'fractalshark' $s.slug 1 'NA-no-nvidia' '' '' 'CUDA renderer, no NVIDIA GPU present' }
}

# ---- lane: ZOOM SEQUENCE (the amortisation axis; see the -ZoomSeqFrames parameter) ----
# Fractadyne renders the sequence IN ONE PROCESS via --render-tour, the mode that owns the
# reference prefetch. Fraktaler-3 3.1's batch CLI renders one image per invocation, so its
# sequence is N processes and its amortisation is ~1.0 BY CONSTRUCTION - a difference in what
# the two CLIs offer, NOT a measurement of F3's engine, which has zoom-sequence machinery this
# lane cannot reach. zoom-seq.py prints that caveat too; keep it attached to the number.
$zsAmort = ''
$zsRows = @()
if ($have.zoomseq) {
    $zsDir = Join-Path $outDir 'zoomseq'
    $zsArgs = @($zsScript, '--out', $zsDir, '--size', $Size, '--frames', $ZoomSeqFrames,
                '--timeout', $TimeoutS)
    if ($have.fractadyne) { $zsArgs += @('--fractadyne', $FractadyneExe) }
    if ($have.fraktaler3) { $zsArgs += @('--fraktaler3', $Fraktaler3Exe, '--f3-wisdom', $wisdom) }
    # The kit ships the ladder's scene in its scenes folder; in the repo it lives in the corpus,
    # which is zoom-seq.py's own default. Pass the kit copy only when it is actually there.
    $zsTemplate = Join-Path $kit ('scenes' + [io.path]::DirectorySeparatorChar + '21-m43-spar-1e27.7.f3.toml')
    if (Test-Path $zsTemplate) { $zsArgs += @('--f3-template', $zsTemplate) }
    Write-Host ''
    Write-Host ('--- zoom sequence: ' + $ZoomSeqFrames + ' frames at ' + $Size + ' ---')
    # Hermetic, like the single-frame Fractadyne lane: a benchmark must not inherit a session.
    $zsCfg = Join-Path $outDir 'zs-config'
    if (Test-Path $zsCfg) { Remove-Item -Recurse -Force $zsCfg }
    New-Item -ItemType Directory -Force $zsCfg | Out-Null
    $env:FRACTADYNE_CONFIG_DIR = $zsCfg
    $env:FRACTADYNE_NO_SOUND = '1'
    & $PythonExe $zsArgs
    Remove-Item Env:FRACTADYNE_CONFIG_DIR -ErrorAction SilentlyContinue
    Remove-Item Env:FRACTADYNE_NO_SOUND -ErrorAction SilentlyContinue
    # Re-emit its rows so a run still has ONE results.csv, and derive the ratio for the summary.
    $zsCsv = Join-Path $zsDir 'results.csv'
    if (Test-Path $zsCsv) {
        $zsRows = @(Import-Csv $zsCsv)
        foreach ($r in $zsRows) {
            Write-Result $csv $r.renderer $r.scene $r.rep $r.status $r.wall_s $r.reported_s $r.note
        }
        # zoom-seq.py WRITES the ratio; do not recompute it here. Deriving it a second time in
        # PowerShell means two implementations of an arithmetic that has already been wrong twice
        # (an N+1 frame count, and a denominator that assumed every rung costs the same).
        $am = $zsRows | Where-Object { $_.scene -like '*-amortisation' -and $_.status -eq 'ok' } | Select-Object -First 1
        if ($am -and $am.reported_s) {
            $zsAmort = $am.reported_s
            Write-Host ('  fractadyne amortisation: ' + $zsAmort + 'x')
        }
    }
} else {
    Write-Host ''
    Write-Host 'Zoom-sequence lane: skipped (needs Python and an automated renderer).'
}

# ---- summary: fastest run per renderer x scene, ratio vs fractadyne where possible ----
$rows = Import-Csv $csv
$md = @('# Benchmark summary - ' + $env:COMPUTERNAME + ' - ' + $stamp, '',
        'Fastest run per renderer and scene. `wall_s` compares the two automated lanes end-to-end;',
        '`reported_s` is each renderer''s own figure (see README for why they are never mixed).', '',
        '| Scene | fractadyne wall | fraktaler3 wall | fd reported | imagina reported | fractalshark reported |',
        '|---|---|---|---|---|---|')
foreach ($s in $sceneRows) {
    $cell = @{}
    foreach ($ren in 'fractadyne', 'fraktaler3', 'imagina', 'fractalshark') {
        $best = $rows | Where-Object { $_.renderer -eq $ren -and $_.scene -eq $s.slug -and $_.status -eq 'ok' } |
            Sort-Object { if ($_.wall_s) { [double]$_.wall_s } else { [double]('0' + $_.reported_s) } } |
            Select-Object -First 1
        if ($best) {
            $cell[$ren] = @{ wall = $best.wall_s; rep = $best.reported_s }
        } else {
            $st = $rows | Where-Object { $_.renderer -eq $ren -and $_.scene -eq $s.slug } | Select-Object -First 1
            $cell[$ren] = @{ wall = $(if ($st) { $st.status } else { '-' }); rep = $(if ($st) { $st.status } else { '-' }) }
        }
    }
    $md += ('| {0} | {1} | {2} | {3} | {4} | {5} |' -f $s.slug, $cell.fractadyne.wall, $cell.fraktaler3.wall, $cell.fractadyne.rep, $cell.imagina.rep, $cell.fractalshark.rep)
}
$md += ''
if ($zsRows.Count) {
    $md += ''
    $md += '## Zoom sequence - amortisation'
    $md += ''
    $md += ('A ' + $ZoomSeqFrames + '-frame Misiurewicz ladder at ' + $Size + ', one frame per rung.')
    $md += 'Every frame is the SAME picture at a different scale, so per-frame cost SHOULD be flat and'
    $md += 'any ramp is the renderer failing to reuse its setup rather than the scene getting harder.'
    $md += 'The ratio is `frames x single_frame_wall / sequence_wall` - each app against ITSELF, so it'
    $md += 'needs no cross-app calibration. 1.0 means everything is rebuilt every frame.'
    $md += ''
    $md += '| Renderer | Sequence wall | Amortisation | Note |'
    $md += '|---|---|---|---|'
    foreach ($ren in 'fractadyne', 'fraktaler3') {
        $sq = $zsRows | Where-Object { $_.renderer -eq $ren -and $_.scene -notlike '*-single*' -and $_.scene -notlike '*-amortisation' } | Select-Object -First 1
        if (-not $sq) { continue }
        $am = '-'
        if ($ren -eq 'fractadyne' -and $zsAmort -ne '') { $am = [string]$zsAmort + 'x' }
        if ($ren -eq 'fraktaler3') { $am = '1.0x by construction' }
        $md += ('| {0} | {1} | {2} | {3} |' -f $ren, $sq.wall_s, $am, $sq.note)
    }
    $md += ''
    $md += 'Fraktaler-3 3.1 has no sequence mode in its batch CLI - it renders one image per'
    $md += 'invocation, so its figure is N processes and its amortisation is 1.0 BY CONSTRUCTION.'
    $md += 'That is a property of the CLI, not of its engine (which has zoom-sequence and'
    $md += 'exponential-map machinery this lane does not reach). Do not quote it as an engine ceiling.'
}
$md += ''
$md += 'Send this folder (or its zip) to feedback@fractadyne.org or attach it to a GitHub issue.'
$md | Out-File -FilePath (Join-Path $outDir 'summary.md') -Encoding ascii
Write-Host ''
Write-Host ('Done. Results: ' + $outDir)
