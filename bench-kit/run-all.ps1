# run-all.ps1 - the Fractadyne relative-performance benchmark. See README.md for the protocol.
# ASCII only (PowerShell 5.1 reads BOM-less files as ANSI; no smart punctuation).
#
#   powershell -ExecutionPolicy Bypass -File run-all.ps1 [-Reps 2] [-Skip imagina,fractalshark]
#       [-FractadyneExe path] [-Fraktaler3Exe path] [-ImaginaExe path] [-FractalSharkExe path]
#       [-TimeoutS 7200]

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
    [string]$Size = '3840x2160'
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
foreach ($k in $Skip) { $have[$k.ToLower()] = $false }
# FractalShark is CUDA: without an NVIDIA GPU the lane is N/A, not a failure.
$fsNa = $false
if ($have.fractalshark) {
    $nv = Get-CimInstance Win32_VideoController | Where-Object { $_.Name -match 'NVIDIA' }
    if (-not $nv) {
        Write-Host 'FractalShark lane: no NVIDIA GPU detected - recording N/A.'
        $have.fractalshark = $false
        $fsNa = $true
    }
}

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
                # pattern below has no room for one — scene 10's reported_s came back EMPTY.
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

# ---- lane: Fraktaler-3 (automated; wisdom generated once per machine) ----
if ($have.fraktaler3) {
    $wisdom = Join-Path $outDir 'f3-wisdom.toml'
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
    }
    foreach ($rep in 1..$Reps) {
        foreach ($s in $sceneRows) {
            # SIZE PARITY. The scene .f3.toml carries the CORPUS's resolution, which is
            # not this benchmark's; rewrite width/height into a per-run copy rather than
            # editing the corpus file, which also drives the correctness references.
            $srcToml = Join-Path $kit ('scenes\' + $s.slug + '.f3.toml')
            $toml    = Join-Path $outDir ($s.slug + '.bench.f3.toml')
            $wh      = $Size -split 'x'
            $tomlTxt = (Get-Content $srcToml) `
                -replace '^\s*width\s*=.*',  ('width = '  + $wh[0]) `
                -replace '^\s*height\s*=.*', ('height = ' + $wh[1])
            Set-Content -Path $toml -Value $tomlTxt -Encoding ASCII
            $argLine = ('-w "{0}" -b "{1}"' -f $wisdom, $toml)
            $r = Invoke-TimedRender $Fraktaler3Exe $argLine $TimeoutS $outDir
            Write-Result $csv 'fraktaler3' $s.slug $rep $r.status $r.wall_s '' ''
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
$md += 'Send this folder (or its zip) to feedback@fractadyne.org or attach it to a GitHub issue.'
$md | Out-File -FilePath (Join-Path $outDir 'summary.md') -Encoding ascii
Write-Host ''
Write-Host ('Done. Results: ' + $outDir)
