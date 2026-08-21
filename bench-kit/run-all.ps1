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
    [int]$TimeoutS = 7200
)

$ErrorActionPreference = 'Stop'
$kit = Split-Path -Parent $MyInvocation.MyCommand.Path
. (Join-Path $kit 'bench-lib.ps1')

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
$scenes = Read-Scenes $kit
Write-Host ''
Write-Host ('Lanes: ' + (($have.GetEnumerator() | Where-Object Value | ForEach-Object Key) -join ', '))
Write-Host ('Scenes: ' + ($scenes.slug -join ', '))
Write-Host ('Reps: ' + $Reps + '   Timeout per render: ' + $TimeoutS + 's')

# ---- lane: Fractadyne (automated; hermetic config so nothing local leaks in) ----
if ($have.fractadyne) {
    $cfg = Join-Path $outDir 'fd-config'
    foreach ($rep in 1..$Reps) {
        foreach ($s in $scenes) {
            if (Test-Path $cfg) { Remove-Item -Recurse -Force $cfg }
            New-Item -ItemType Directory -Force $cfg | Out-Null
            $env:FRACTADYNE_CONFIG_DIR = $cfg
            $kfr = Read-Kfr (Join-Path $kit ('scenes\' + $s.slug + '.kfr'))
            $png = Join-Path $outDir ('fd-' + $s.slug + '.png')
            $zl2 = [double]$s.mag_log10 * [math]::Log(10.0, 2.0)
            $args = ('--render --out "{0}" --size 1920x1080 --center {1} {2} --zoom-log2 {3} --iter {4} --ss 1 --palette 0' -f $png, $kfr['Re'], $kfr['Im'], $zl2, $s.iterations)
            if ($s.normalize -eq '1') { $args += ' --normalize' }
            $r = Invoke-TimedRender $FractadyneExe $args $TimeoutS $kit
            $reported = ''
            if ($r.stdout -match '\(in ([0-9hms. ]+)\)') { $reported = $Matches[1].Trim() }
            Write-Result $csv 'fractadyne' $s.slug $rep $r.status $r.wall_s $reported ''
        }
    }
    Remove-Item Env:FRACTADYNE_CONFIG_DIR -ErrorAction SilentlyContinue
}

# ---- lane: Fraktaler-3 (automated; wisdom generated once per machine) ----
if ($have.fraktaler3) {
    $wisdom = Join-Path $outDir 'f3-wisdom.toml'
    if (-not (Test-Path $wisdom)) {
        Write-Host 'Fraktaler-3: generating tuning wisdom (once)...'
        $w = Invoke-TimedRender $Fraktaler3Exe ('-W "' + $wisdom + '"') 1800 $outDir
        Write-Host ('  wisdom: ' + $w.status + ' in ' + $w.wall_s + 's')
    }
    foreach ($rep in 1..$Reps) {
        foreach ($s in $scenes) {
            $toml = Join-Path $kit ('scenes\' + $s.slug + '.f3.toml')
            $args = ('-w "{0}" -b "{1}"' -f $wisdom, $toml)
            $r = Invoke-TimedRender $Fraktaler3Exe $args $TimeoutS $outDir
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
        foreach ($s in $scenes) {
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
        foreach ($s in $scenes) {
            Invoke-AssistedLane 'fractalshark' $FractalSharkExe (Join-Path $kit ('scenes\' + $s.slug + '.kfr')) $s.slug $rep $csv $hints
        }
    }
} elseif ($fsNa) {
    foreach ($s in $scenes) { Write-Result $csv 'fractalshark' $s.slug 1 'NA-no-nvidia' '' '' 'CUDA renderer, no NVIDIA GPU present' }
}

# ---- summary: fastest run per renderer x scene, ratio vs fractadyne where possible ----
$rows = Import-Csv $csv
$md = @('# Benchmark summary - ' + $env:COMPUTERNAME + ' - ' + $stamp, '',
        'Fastest run per renderer and scene. `wall_s` compares the two automated lanes end-to-end;',
        '`reported_s` is each renderer''s own figure (see README for why they are never mixed).', '',
        '| Scene | fractadyne wall | fraktaler3 wall | fd reported | imagina reported | fractalshark reported |',
        '|---|---|---|---|---|---|')
foreach ($s in $scenes) {
    $cell = @{}
    foreach ($ren in 'fractadyne', 'fraktaler3', 'imagina', 'fractalshark') {
        $best = $rows | Where-Object { $_.renderer -eq $ren -and $_.scene -eq $s.slug -and $_.status -eq 'ok' } |
            Sort-Object { [double]($_.wall_s + 0 + $_.reported_s) } | Select-Object -First 1
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
