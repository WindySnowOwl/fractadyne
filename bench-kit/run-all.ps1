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
    # GPU HDR by default, valid from FractalShark 0.541. That release added sm_75 code (PTX+SASS)
    # which JITs onto sm_86, so a GPU algorithm now renders real pictures on an RTX 3080; verified
    # 2026-09-12 (10/10 fat binaries load, GpuHDRx32PerturbedLAv2 renders 1e6 through 4.6e1105).
    # A leftover "OpenGL context creation FAILED" warning still prints but no longer blanks output.
    # GpuHDRx32PerturbedLAv2 spans the whole corpus; it DNFs by LOCATION (the two nuclei, the spar,
    # and -- through 0.542 -- 4.2e275), not by depth. Do NOT use AutoSelect: it picks a non-HDR GPU
    # algorithm that goes flat at deep zoom, and there is no "Auto" either (0.543 rejects
    # `--render-algorithm Auto` as an unknown name, despite its own error text suggesting one).
    # NOTE: 0.543 RENDERS ALL FOUR OF THEM (measured 2026-09-21 on an RTX 3080), so that DNF list
    # is stale for 0.543+: 4.2e275 gives 1683 colours at modal 0.053, and the spar and both nuclei
    # give real dendrite structure. WARNING: the latter three come out in a very low-contrast
    # palette -
    # luminance stddev ~1.1 of 255 - so they sit close to the structure guard's margin while being
    # entirely correct pictures. Check the image before believing a DNF-blank on those three; the
    # guard's job is "is this a picture", and it has never claimed to judge "is this the RIGHT
    # picture", which only a cross-renderer comparison can say.
    # WARNING: pick an algorithm that can represent the depth: Gpu1x32PerturbedLAv2 exits 1 with
    # "cannot represent this viewport's pixel spacing" past f32 spacing, and the lane turns that
    # into DNF-algo-too-narrow rather than letting it look like a fast frame.
    #
    # THROUGH 0.54, every GPU algorithm returned an EMPTY image on this box for two independent
    # reasons: the release binaries carried CUDA code for sm_89 + sm_120 only (RTX 40/50; see
    # tools/cuda-arch-inventory.py), so no kernel could run on an RTX 3080; and the CLI's GL
    # consumer needs a window it never creates. If you must benchmark such an old build, switch to a
    # CPU algorithm (e.g. Cpu64PerturbedBLAV2HDR) - correct at shallow/mid depth, blank past ~1e27.
    # Either way the structure guard stands: a flat frame is DNF-blank, never a TIME without an IMAGE.
    [string]$FractalSharkAlgo = 'GpuHDRx32PerturbedLAv2',
    # FractalShark 0.543 added a CLI server (--server / --connect / --shutdown) so CUDA and
    # process startup are paid once for a whole run instead of once per frame. The kit uses it by
    # default because that is the honest per-frame cost. Set this switch to go back to a process
    # per frame, which is what every number this kit published before 0.543 carries.
    [switch]$NoFractalSharkServer,
    # Imagina ships a Win32/OpenGL GUI only, which is why its lane was operator-assisted and in
    # practice never ran. imagina-cli is a small headless fork of it (AGPL, its own repo, built
    # from source - see README "Imagina"); point at it and the lane automates like the others.
    [string]$ImaginaCliExe = '',
    # imagina-cli is built with msys2 mingw and needs that toolchain's DLLs on PATH; without them
    # it exits 0xC0000135 (DLL not found) with no message at all, which reads exactly like a
    # crash. Prepended for this lane only, never for the process. Autodetected when left empty.
    [string]$ImaginaRuntimeDir = '',
    # ONE palette for the lanes that can take one. Comparing two renderers by image only works
    # while their palettes are comparable, and by default they are not at all: on the period-148
    # nucleus, Fractadyne against Imagina scored 0.067 on their own palettes and 0.496 on a
    # shared .map, which is the difference between a view check that can confirm nothing and one
    # that can. A Fractint / Kalles Fraktaler .map is the portable format Fractadyne already
    # imports and our imagina-cli fork now does too.
    # Only those two can join: Fraktaler-3 3.1 exposes no colour options in its batch toml or its
    # CLI, and FractalSharkCli has none at all (its --color is ANSI console art). Saying that is
    # better than pretending the column is like-for-like.
    # OFF by default, because switching Fractadyne off its preset palette changes every reference
    # image and would silently break comparison with everything this kit published before.
    [string]$SharedPalette = '',
    # MATCH IMAGINA TO FRACTADYNE'S OWN PALETTE instead of moving both onto a neutral one. This is
    # the better default: Fractadyne keeps rendering exactly as it ships, so every reference image
    # and every historical number stays comparable, and only the lane we control is adjusted to
    # meet it.
    #
    # palettes/ember.map is Fractadyne's built-in "Ember" (PRESETS[0], its default) baked to 1024
    # entries the way Gradient::bake does - texel centres, linear between stops.
    #
    # The POSITION mapping matters as much as the colours, and is why a neutral shared map only
    # reached 0.496. Fractadyne maps position = smooth_iteration * cycle + offset, wrapped, with
    # cycle = 0.004 + 0.27*0.06 = 0.0202 and offset = 0.1 at its defaults. Imagina's smooth value
    # advances at twice that rate, so the matching cycle is EXACTLY HALF, 0.0101, at offset 0.
    # That factor of two was fitted over a 2-D sweep on two scenes at opposite ends of the corpus
    # and is a clean convention difference rather than a per-scene fudge; both scenes peak there.
    # Measured on the period-148 nucleus: 0.067 with each renderer's own palette, 0.496 on a
    # neutral shared map, 0.949 here.
    [string]$ImaginaPaletteMap = '',
    [double]$ImaginaPaletteCycle = 0.0101,
    [double]$ImaginaPaletteOffset = 0.0
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
$have += @{ imaginacli = ($ImaginaCliExe -and (Test-Path $ImaginaCliExe)) }
# Default Imagina to Fractadyne's own Ember palette, calibrated. Off only if the file is gone.
if ($have.imaginacli -and -not $ImaginaPaletteMap) {
    $cand = Join-Path $kit 'palettes\ember.map'
    if (Test-Path $cand) { $ImaginaPaletteMap = $cand }
}
if ($have.imaginacli -and -not $ImaginaRuntimeDir) {
    foreach ($cand in 'C:\msys64\mingw64\bin', 'C:\msys64\clang64\bin') {
        if (Test-Path $cand) { $ImaginaRuntimeDir = $cand; break }
    }
}
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
# Same rule for Imagina: transcription is for renderers with no headless mode, and this one now
# has one. The assisted lane only runs when the CLI is absent.
if ($have.imaginacli) { $have.imagina = $false }

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
            if ($SharedPalette) { $argLine += (' --palette-map "{0}"' -f $SharedPalette) }
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
            Add-RunRecord @{
                renderer = 'fractadyne'; scene = $s.slug; rep = $rep
                exe = $FractadyneExe; args = $argLine; cwd = $kit
                # The lane derives zoom-log2 from the scene's mag_log10 rather than passing the
                # .kfr's Zoom: it is the same magnification in the units this CLI takes.
                source = ('scenes\' + $s.slug + '.kfr')
                inputs = @{ center_re = $kfr['Re']; center_im = $kfr['Im']
                            kfr_zoom = $kfr['Zoom']; zoom_log2 = $zl2
                            iterations = $s.iterations; size = $Size
                            samples_per_pixel = 1; palette = 0
                            normalize = ($s.normalize -eq '1') }
                output = ('fd-' + $s.slug + '.png')
                status = $r.status; wall_s = $r.wall_s; reported_s = $reported; note = ''
            }
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
        $wArgs = '-w "' + $wisdom + '" -W'
        $w = Invoke-TimedRender $Fraktaler3Exe $wArgs 300 $outDir
        Write-Host ('  wisdom init: ' + $w.status + ' in ' + $w.wall_s + 's')
        Add-RunRecord @{ renderer = 'fraktaler3'; scene = '(tuning) wisdom init'; rep = 0
                         exe = $Fraktaler3Exe; args = $wArgs; cwd = $outDir; source = ''
                         inputs = @{}; output = 'f3-wisdom.toml'
                         status = $w.status; wall_s = $w.wall_s; reported_s = ''
                         note = 'one-off hardware tuning, not a benchmark row' }
        $bArgs = '-w "' + $wisdom + '" -B'
        $w = Invoke-TimedRender $Fraktaler3Exe $bArgs 1800 $outDir
        Write-Host ('  wisdom benchmark: ' + $w.status + ' in ' + $w.wall_s + 's')
        Add-RunRecord @{ renderer = 'fraktaler3'; scene = '(tuning) wisdom benchmark'; rep = 0
                         exe = $Fraktaler3Exe; args = $bArgs; cwd = $outDir; source = ''
                         inputs = @{}; output = 'f3-wisdom.toml'
                         status = $w.status; wall_s = $w.wall_s; reported_s = ''
                         note = 'one-off number-type benchmark, not a benchmark row' }
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
            # Fraktaler-3 takes its whole view from the .toml, not from flags, so the argument
            # line alone does not say what was rendered. The rewritten toml IS the input and is
            # kept beside the results; the report inlines it.
            Add-RunRecord @{
                renderer = 'fraktaler3'; scene = $s.slug; rep = $rep
                exe = $Fraktaler3Exe; args = $argLine; cwd = $outDir
                source = ($s.slug + '.bench.f3.toml')
                inputs = @{ size = $Size; subframes = 1; wisdom = 'f3-wisdom.toml'
                            iterations = $s.iterations
                            note = 'view comes from the .toml; width/height/subframes rewritten per run' }
                output = ($s.slug + '-f3.png')
                status = $r.status; wall_s = $r.wall_s; reported_s = ''; note = ''
            }
        }
    }
}

# ---- lane: FractalShark (automated, via FractalSharkCli) ----
# Declared before the lane, not inside it: the summary states which shape produced the numbers,
# and under Set-StrictMode reading a variable the lane never defined is an ERROR, not an empty
# string. A skipped lane must not be able to take the whole report down with it.
$fsShape = ''
$fsStartup = ''
# Both automated third-party lanes ask the view question, so the tool path is script-level. Under
# StrictMode a variable defined inside a lane that did not run is an ERROR for the lane that did.
$vv = Join-Path $kit 'tools\verify-views.py'
# From 0.541 the GPU lane WORKS on this box (see README "FractalShark, honestly"): the release adds
# sm_75 code that JITs onto sm_86, so a GPU algorithm now renders real pictures. The default here is
# a GPU HDR algorithm for that reason. Through 0.54 every GPU algorithm returned a BLANK image
# headlessly (no sm_86 kernel + an OpenGL-consumer bug), which is why this lane STILL checks that a
# render is actually a PICTURE before it records a TIME - a flat frame is DNF-blank, never a number.
if ($have.fractalsharkcli) {
    $wh = $Size -split 'x'
    # 0.543 added --server/--connect so CUDA + process startup is paid ONCE rather than per frame.
    # Measured here on an RTX 3080, scene 03 at 1.33e6, three reps: 1776 ms median with a process
    # per frame, 399 ms median through a server. About 1.4 s of every old number was startup --
    # most of a shallow frame, and a bias that grew as the frame got cheaper.
    # -NoFractalSharkServer restores the old shape, which is the only way to compare against the
    # numbers this kit published before 0.543.
    $useServer = -not $NoFractalSharkServer
    $endpoint = 'FractalSharkCli-benchkit-' + $PID
    $srv = $null
    if ($useServer) {
        Write-Host ('FractalShark: starting the CLI server (endpoint ' + $endpoint + ')')
        $srv = Start-SharkServer $FractalSharkCliExe $endpoint $wh[0] $wh[1] $outDir
        if (-not $srv.ok) {
            Write-Host '  server did not come up - falling back to one process per frame.'
            $useServer = $false
        } else {
            Write-Host ('  ready in ' + $srv.startup_s + ' s')
            $fsStartup = [string]$srv.startup_s
        }
    }
    $fsShape = $(if ($useServer) { 'server-amortized (--server/--connect, startup paid once)' }
                 else { 'one process per frame (startup folded into every frame)' })
    Write-Host ('FractalShark: automated via ' + (Split-Path $FractalSharkCliExe -Leaf) +
                ', algorithm ' + $FractalSharkAlgo +
                $(if ($useServer) { ' (server-amortized)' } else { ' (process per frame)' }))
    # PNG encoding is ASYNCHRONOUS and only flushed by --shutdown, so a render cannot be validated
    # where it is timed: checking the file straight after the client returns finds nothing and
    # scores a DNF for a frame that rendered perfectly. Collect, flush, then judge.
    $pending = @()
    foreach ($rep in 1..$Reps) {
        foreach ($s in $sceneRows) {
            $kfr = Read-Kfr (Join-Path $kit ('scenes\' + $s.slug + '.kfr'))
            # The magnification is a STRING, never a number: 10^1105.79 is +inf in double and the
            # corpus goes there. Its zoom convention is Kalles Fraktaler's - the same one we and
            # Fraktaler-3 use - so the .kfr's own Zoom field is handed over VERBATIM.
            #
            # It used to send '1e' + mag_log10, and the comment here claimed a fractional exponent
            # was "verified". It is ACCEPTED, and then SILENTLY TRUNCATED. Measured on 0.543,
            # 2026-09-21: `--zoom 1e6.1249387366083` and `--zoom 1e6` produce BYTE-IDENTICAL PNGs,
            # while the correct `--zoom 1.333333E6` produces a different and correct one. Exit 0,
            # a real picture, a plausible time -- and the wrong view, under-zoomed by up to 10x.
            # Interior-area calibration against the Fractadyne render of scene 03: 1.333333E6 gives
            # 0.411 against our 0.427, and 1E6 gives 0.251.
            #
            # That is why it was fast. An under-zoomed frame is a CHEAPER frame, so the error
            # flattered FractalShark at every depth, and it flattered it MORE the deeper the scene
            # (where the truncated fraction is worth the most magnification). Every FractalShark
            # number this kit published before 2026-09-21 is from the wrong view and must not be
            # compared with anything.
            #
            # LESSON, because the structure guard did not save us: a mantissa is not decoration.
            # The guard asks "is this a picture" and all ten scenes passed it while being the wrong
            # picture -- one of them a near-flat field. Only a comparison against another
            # renderer's image of the SAME scene catches this class, which is why this lane now
            # ships tools/verify-views.py.
            $zoom = $kfr['Zoom']
            # WARNING: the output name may not be the one you pass. FractalSharkCli strips the
            # final extension and re-appends .png only when what remains has no dot, so
            # "fs-21-m43-spar-1e27.7.png" lands as "fs-21-m43-spar-1e27.7" with no extension - and
            # three of our slugs contain dots. Hand it a dot-free stem; let it add the extension.
            # The rep is in the name because a server run keeps every frame for the flush.
            $stem = Join-Path $outDir ('fs-' + ($s.slug -replace '\.', 'p') + '-r' + $rep)
            $png = $stem + '.png'
            # SAMPLING PARITY, restated rather than inherited: one sample per pixel, like every
            # other lane. This is the field that was silently 4x for Fraktaler-3 in every run this
            # kit ever published.
            $renderArgs = ('--render-algorithm {0} --center-x {1} --center-y {2} --zoom {3} --iterations {4} --width {5} --height {6} --antialiasing 1 --out "{7}" --quiet' -f $FractalSharkAlgo, $kfr['Re'], $kfr['Im'], $zoom, $s.iterations, $wh[0], $wh[1], $stem)
            $argLine = if ($useServer) { ('--connect --endpoint {0} ' -f $endpoint) + $renderArgs } else { $renderArgs }
            $r = Invoke-TimedRender $FractalSharkCliExe $argLine $TimeoutS $outDir
            $note = $(if ($useServer) { 'server-amortized' } else { 'process per frame' }) +
                    $(if ($FractalSharkAlgo -like 'Gpu*') { '; GPU path (CUDA)' } else { '; CPU path' })
            # The reported "Frame time" is worth HAVING - it is FractalShark's own render cost,
            # and the gap between it and our wall is the pipe plus the PNG encode, which is the
            # one number that says whether the CLI transport is in the way. But it is NEVER
            # allowed to decide anything: the client prints one even for a render it REFUSED.
            # Measured on 0.543 -- "error: ... cannot represent this viewport's pixel spacing"
            # followed by "Frame time: 17.1 ms", and no image written. So parse it here, carry it
            # alongside, and stamp it into the CSV only for a row that survives BOTH the exit code
            # and the structure guard. Status is judged on the exit code and the image, never text.
            $fsReported = ''
            if ($r.stdout -match 'Frame time:\s*([0-9.]+)\s*ms') {
                $fsReported = [string][math]::Round([double]$Matches[1] / 1000.0, 3)
            }
            if ($r.status -eq 'ok' -and ($r.stdout + $r.stderr) -match 'cannot represent') {
                $r.status = 'DNF-algo-too-narrow'
                $note += '; ' + $FractalSharkAlgo + ' cannot represent this depth - needs an HDR algorithm'
            }
            $pending += @{ scene = $s.slug; rep = $rep; png = $png; r = $r; note = $note
                           reported = $fsReported; args = $argLine
                           inputs = @{ center_re = $kfr['Re']; center_im = $kfr['Im']
                                       zoom = $zoom; iterations = $s.iterations; size = $Size
                                       antialiasing = 1; algorithm = $FractalSharkAlgo
                                       mode = $(if ($useServer) { 'client against a live server' }
                                                else { 'one process per frame' }) } }
        }
    }
    if ($useServer) {
        Write-Host 'FractalShark: shutting the server down - this is what flushes the PNGs'
        Stop-SharkServer $FractalSharkCliExe $endpoint $srv
    }
    # IS IT THE RIGHT PICTURE? The structure guard below only ever asked "is it A picture", and on
    # 2026-09-21 all ten scenes passed it while being renders of the wrong place, because the CLI
    # truncated the fractional exponent in the zoom it was handed. An under-zoomed frame is a
    # cheaper frame, so the error arrived disguised as a 20-50x speed win. Nothing in the lane
    # could have caught it: only comparing against another renderer's image of the SAME scene can.
    # Needs the fractadyne lane for its reference renders; without it the check says so and the
    # numbers stay unverified rather than quietly passing.
    $viewOk = @{}
    $viewConfirmed = @{}
    $viewChecked = $false
    if ($PythonExe -and (Test-Path $vv) -and $have.fractadyne) {
        $vjson = Join-Path $outDir 'fs-view-check.json'
        Write-Host 'FractalShark: checking the renders are the scenes that were ASKED for'
        & $PythonExe $vv $outDir --prefix fs --json $vjson --quiet 2>&1 | ForEach-Object { '  ' + $_ }
        if (Test-Path $vjson) {
            $viewChecked = $true
            $vd = Get-Content $vjson -Raw | ConvertFrom-Json
            foreach ($p in $vd.PSObject.Properties) {
                # magnification_ok is the verdict that can VOID a time. same_view is the stronger
                # claim and is only used to decide whether to annotate the row.
                $viewOk[$p.Name] = [bool]$p.Value.magnification_ok
                $viewConfirmed[$p.Name] = [bool]$p.Value.same_view
            }
        }
    } else {
        Write-Host 'FractalShark: view check SKIPPED (needs Python and the fractadyne lane) - times unverified.'
    }
    foreach ($q in $pending) {
        $status = $q.r.status
        $note = $q.note
        $reported = $q.reported
        if ($status -eq 'ok') {
            if (-not (Test-Path $q.png)) {
                $status = 'DNF-no-output'
                $note = 'exit 0 but no PNG at the expected path, even after the shutdown flush'
            } elseif (-not (Test-RenderHasStructure $q.png)) {
                $status = 'DNF-blank'
                $note = 'exit 0 but the image is uniform - a time here would be meaningless'
            }
        }
        # A picture of the WRONG PLACE is not a slow or a fast result, it is a VOID one. But only
        # the INTERIOR FRACTION can say that: it tracks magnification and no palette choice moves
        # it. A collapsed edge correlation on its own means "could not confirm", because two
        # renderers can draw the same field unrecognisably differently - FractalShark cycles its
        # palette once per iteration with no CLI option to slow it, and at 1.6e148 that aliases
        # into apparent static against our --normalize render of the same scene. Treating that as
        # a DNF made this lane publish a claim that another project's renderer was broken. It was
        # not. Do not let a colouring difference delete a measurement or accuse anyone.
        if ($status -eq 'ok' -and $viewChecked -and $viewOk.ContainsKey($q.scene) -and -not $viewOk[$q.scene]) {
            $status = 'DNF-not-the-scene'
            $note = 'exit 0 and a structured image, but its interior fraction says this is a different magnification; see fs-view-check.json and LOOK at the PNG'
        } elseif ($status -eq 'ok' -and $viewChecked -and $viewConfirmed.ContainsKey($q.scene) -and -not $viewConfirmed[$q.scene]) {
            $note += '; view UNCONFIRMED (magnification agrees; the images differ, usually a palette difference - compare at 1:1 before concluding)'
        } elseif ($status -eq 'ok' -and -not $viewChecked) {
            $note += '; view NOT verified'
        }
        # A self-reported time only survives on a row that EARNED it. On any DNF it is dropped,
        # because the whole trap is that a refusal still prints one.
        if ($status -ne 'ok') { $reported = '' }
        elseif ($reported -and $q.r.wall_s) {
            $note += ('; render ' + $reported + 's of ' + $q.r.wall_s + 's wall (rest is pipe + PNG encode)')
        }
        Write-Result $csv 'fractalshark' $q.scene $q.rep $status $q.r.wall_s $reported $note
        Add-RunRecord @{
            renderer = 'fractalshark'; scene = $q.scene; rep = $q.rep
            exe = $FractalSharkCliExe; args = $q.args; cwd = $outDir
            source = ('scenes\' + $q.scene + '.kfr')
            inputs = $q.inputs
            output = (Split-Path $q.png -Leaf)
            status = $status; wall_s = $q.r.wall_s; reported_s = $reported; note = $note
        }
    }
    if ($useServer) {
        Add-RunRecord @{ renderer = 'fractalshark'; scene = '(server) startup'; rep = 0
                         exe = $FractalSharkCliExe
                         args = ('--server --endpoint {0} --width {1} --height {2}' -f $endpoint, $wh[0], $wh[1])
                         cwd = $outDir; source = ''; inputs = @{ endpoint = $endpoint }
                         output = ('fs-server-' + $endpoint + '.log')
                         status = 'ok'; wall_s = $fsStartup; reported_s = ''
                         note = 'paid ONCE for the whole lane; the per-scene rows are client calls' }
    }
}

# ---- lane: Imagina (automated, via the headless imagina-cli fork) ----
if ($have.imaginacli) {
    $wh = $Size -split 'x'
    $savedPath = $env:PATH
    if ($ImaginaRuntimeDir -and (Test-Path $ImaginaRuntimeDir)) {
        $env:PATH = $ImaginaRuntimeDir + ';' + $env:PATH
        Write-Host ('Imagina: automated via imagina-cli (runtime ' + $ImaginaRuntimeDir + ')')
    } else {
        Write-Host 'Imagina: automated via imagina-cli (no runtime dir set - may fail to launch)'
    }
    $p2p = Join-Path $kit 'tools\ppm-to-png.py'
    foreach ($rep in 1..$Reps) {
        foreach ($s in $sceneRows) {
            $kfr = Read-Kfr (Join-Path $kit ('scenes\' + $s.slug + '.kfr'))
            # ZOOM IS THE .kfr STRING, lowercased. imagina-cli parses it with mpf_set_str, which
            # takes a mantissa and an INTEGER exponent ("5.071075e27") and does NOT accept the
            # fractional-exponent spelling; it also wants a lowercase 'e'. This is the same class
            # of trap that had the FractalShark lane rendering the wrong view for months, so the
            # scene file's own field is handed over rather than anything recomputed.
            $zoom = ($kfr['Zoom'] -replace 'E', 'e')
            # Working precision from the centre's own digit count: the corpus carries centres to
            # the precision the location needs, so that count IS the requirement. +96 bits of
            # headroom, floor of 128.
            $digits = ($kfr['Re'] -replace '[^0-9]', '').Length
            $prec = [Math]::Max(128, [int]($digits * 3.3219) + 96)
            $ppm = Join-Path $outDir ('im-' + $s.slug + '.ppm')
            $png = Join-Path $outDir ('im-' + $s.slug + '.png')
            $argLine = ('--center-x {0} --center-y {1} --precision {2} --zoom {3} --iter {4} --width {5} --height {6} --out "{7}"' -f
                        $kfr['Re'], $kfr['Im'], $prec, $zoom, $s.iterations, $wh[0], $wh[1], $ppm)
            if ($SharedPalette) {
                $argLine += (' --palette-map "{0}"' -f $SharedPalette)
            } elseif ($ImaginaPaletteMap) {
                $argLine += (' --palette-map "{0}" --palette-cycle {1} --palette-offset {2} --palette-smooth' -f
                             $ImaginaPaletteMap, $ImaginaPaletteCycle, $ImaginaPaletteOffset)
            }
            $r = Invoke-TimedRender $ImaginaCliExe $argLine $TimeoutS $outDir
            $note = 'headless imagina-cli (AGPL fork, built from source); CPU path'
            $status = $r.status
            if ($status -eq 'ok') {
                # PPM is a fine image and unreadable to the structure guard and to a browser, so
                # it is converted once here rather than taught to every later stage.
                if ($PythonExe -and (Test-Path $p2p)) {
                    & $PythonExe $p2p $ppm $png 2>&1 | Out-Null
                }
                if (-not (Test-Path $png)) {
                    $status = 'DNF-no-output'
                    $note = 'exit 0 but no PNG - check that Pillow is installed for the PPM conversion'
                } elseif (-not (Test-RenderHasStructure $png)) {
                    $status = 'DNF-blank'
                    $note = 'exit 0 but the image is uniform - a time here would be meaningless'
                }
            }
            Write-Result $csv 'imagina' $s.slug $rep $status $r.wall_s '' $note
            Add-RunRecord @{
                renderer = 'imagina'; scene = $s.slug; rep = $rep
                exe = $ImaginaCliExe; args = $argLine; cwd = $outDir
                source = ('scenes\' + $s.slug + '.kfr')
                inputs = @{ center_re = $kfr['Re']; center_im = $kfr['Im']; zoom = $zoom
                            precision_bits = $prec; iterations = $s.iterations; size = $Size
                            runtime_dir = $ImaginaRuntimeDir }
                output = ('im-' + $s.slug + '.png')
                status = $status; wall_s = $r.wall_s; reported_s = ''; note = $note
            }
        }
    }
    $env:PATH = $savedPath
    # Same question asked of this lane as of FractalShark's, and answered with the same two-tier
    # verdict: only a magnification disagreement voids a time.
    if ($PythonExe -and (Test-Path $vv) -and $have.fractadyne) {
        Write-Host 'Imagina: checking the renders are the scenes that were ASKED for'
        & $PythonExe $vv $outDir --prefix im --json (Join-Path $outDir 'im-view-check.json') --quiet 2>&1 |
            ForEach-Object { '  ' + $_ }
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
    Add-RunRecord @{ renderer = '(lane) zoomseq'; scene = ('zoomseq-' + $ZoomSeqFrames + 'f'); rep = 0
                     exe = $PythonExe; args = ($zsArgs -join ' '); cwd = $kit; source = ''
                     inputs = @{ frames = $ZoomSeqFrames; size = $Size
                                 ladder = '21-m43-spar-1e27.7 Misiurewicz lambda-ladder'
                                 note = 'drives fractadyne --render-tour and N F3 invocations; see zoomseq\results.csv' }
                     output = 'zoomseq'; status = 'ok'; wall_s = ''; reported_s = ''
                     note = 'the sequence lane runs its own sub-harness' }
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
# PARENTHESES REQUIRED. In PowerShell the comma operator binds TIGHTER than `+`, so
#   @('a' + $x, '', 'b')
# parses as @('a' + @($x, '', 'b')) - the rest of the literal is absorbed into an array, coerced
# to a string and concatenated onto the first element with $OFS spaces. Every summary.md this kit
# ever wrote had its title, its whole explanation, the table header AND the separator row welded
# into line 1, which is why the markdown table never rendered as a table: a header glued to a
# paragraph is just a paragraph. Only the `$md +=` rows below were ever real lines.
$md = @(('# Benchmark summary - ' + $env:COMPUTERNAME + ' - ' + $stamp), '',
        'Fastest run per renderer and scene. `wall_s` compares the AUTOMATED lanes end-to-end;',
        '`reported_s` is each renderer''s own figure (see README for why they are never mixed).', '',
        'FractalShark is a wall column, not a reported one. It had one only while the lane was',
        'OPERATOR-ASSISTED and a human transcribed the figure off the GUI; the CLI lane writes',
        'wall_s and leaves reported_s empty, so printing `reported` here rendered every automated',
        'FractalShark result as a BLANK CELL while the numbers sat in results.csv. A summary that',
        'silently drops a lane it ran is worse than one that admits it skipped it.',
        'In server mode that wall is the CLIENT call - the amortized per-frame cost, with CUDA and',
        'process startup paid once for the whole run rather than folded into every frame.', '',
        'Rows run SHALLOWEST FIRST. scenes.csv is ordered by scene id, which is not depth order -',
        'it puts 1e1105 above 1e27.7 - and a benchmark whose whole axis is magnification should',
        'not be read in an order that hides it. Execution order is unchanged and stays in the',
        'manifest; only the presentation is sorted.', '',
        'Imagina is a wall column too now that imagina-cli automates it; it was a transcribed',
        'figure only while a human had to read it off the GUI.', '',
        '| Scene | Magnification | fractadyne wall | fraktaler3 wall | fractalshark wall | imagina wall | fd reported |',
        '|---|---|---|---|---|---|---|')
# Slug is the tiebreak, not decoration: the corpus has two scenes at exactly 5.1e27, PowerShell's
# sort is not stable for equal keys, and two runs over the same data swapping rows makes a diff
# between reports look like a change when nothing changed.
foreach ($s in ($sceneRows | Sort-Object @{ e = { [double]$_.mag_log10 } }, @{ e = { $_.slug } })) {
    # 10^1105.79 overflows a double, so the mantissa and the exponent are split textually and
    # never multiplied back together.
    $e = [math]::Floor([double]$s.mag_log10)
    $mant = [math]::Pow(10, [double]$s.mag_log10 - $e)
    if ($mant -ge 9.995) { $mant = $mant / 10; $e = $e + 1 }
    $magStr = ('{0:N1}e{1}' -f $mant, [int]$e)
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
    $md += ('| {0} | {1} | {2} | {3} | {4} | {5} | {6} |' -f $s.slug, $magStr, $cell.fractadyne.wall, $cell.fraktaler3.wall, $cell.fractalshark.wall, $cell.imagina.wall, $cell.fractadyne.rep)
}
$md += ''
# WHICH SHAPE produced the FractalShark column is not a footnote: a server number and a
# process-per-frame number differ by more than a second on this box and are not comparable.
# A table that does not say which one it holds invites exactly that comparison.
if ($fsShape) {
    $md += ('FractalShark lane: ' + $FractalSharkAlgo + ', ' + $fsShape +
            $(if ($fsStartup) { '; server ready in ' + $fsStartup + ' s' } else { '' }) + '.')
    $md += ''
}
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

# ---- run manifest + HTML report -----------------------------------------------------------
# The manifest is written ALWAYS, because it is the primary record: exactly what was executed,
# with the arguments and the parsed inputs, per render. The HTML is a rendering of it and of the
# images beside it, and is allowed to be skipped - a missing Python must cost you the pretty
# report, never the evidence.
$meta = @{
    host = $env:COMPUTERNAME; stamp = $stamp
    started = (Get-Date).ToString('s')
    size = $Size; reps = $Reps; timeout_s = $TimeoutS
    scenes = @($sceneRows.slug)
    # The sort key for every presentation of these results. Scenes are EXECUTED in scenes.csv
    # order, which is neither alphabetical nor by depth, and their slugs order 1e1105 before
    # 1e27.7 because the leading number is an id, not a magnitude. Shipping the magnification
    # here means a report never has to guess it out of a filename.
    scene_mag = $(
        $m = @{}
        foreach ($s in $sceneRows) { $m[$s.slug] = [double]$s.mag_log10 }
        $m
    )
    lanes = @(($have.GetEnumerator() | Where-Object Value | ForEach-Object Key))
    fractalshark_algorithm = $FractalSharkAlgo
    fractalshark_shape = $fsShape
    shared_palette = $SharedPalette
    imagina_palette = @{ map = $ImaginaPaletteMap; cycle = $ImaginaPaletteCycle; offset = $ImaginaPaletteOffset }
    zoomseq_frames = $ZoomSeqFrames
    zoomseq_amortisation = $zsAmort
    exes = @{
        fractadyne   = (Get-ExeStamp $FractadyneExe)
        fraktaler3   = (Get-ExeStamp $Fraktaler3Exe)
        fractalshark = (Get-ExeStamp $FractalSharkCliExe)
        imagina      = (Get-ExeStamp $ImaginaExe)
    }
}
$manifest = Join-Path $outDir 'run-manifest.json'
Save-RunManifest $manifest $meta
Write-Host ''
Write-Host ('Run manifest: ' + $manifest)

$mk = Join-Path $kit 'tools\make-report.py'
if ($PythonExe -and (Test-Path $mk)) {
    Write-Host 'Building the HTML report...'
    & $PythonExe $mk $outDir 2>&1 | ForEach-Object { '  ' + $_ }
} else {
    Write-Host 'HTML report SKIPPED (needs Python 3 with Pillow). run-manifest.json still written.'
}

Write-Host ''
Write-Host ('Done. Results: ' + $outDir)
$rep = Join-Path $outDir 'report.html'
if (Test-Path $rep) { Write-Host ('Report:  ' + $rep) }
