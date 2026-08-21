# package.ps1 - assemble the standalone benchmark kit zip from the repo. Run from anywhere;
# writes fractadyne-bench-kit-<version>.zip next to this script. ASCII only.
#
# The kit must be self-contained for someone with none of our repo: scenes are COPIED in
# (6 locations x 3 formats from validation/corpus), the vendored Fraktaler-3 binary is copied
# WITH its source directory (AGPL-3.0 requires the corresponding source to travel with the
# binary), and fractadyne.exe is included when present so the kit is runnable out of the box.

[CmdletBinding()]
param([string]$FractadyneExe = '')

$ErrorActionPreference = 'Stop'
$kit = Split-Path -Parent $MyInvocation.MyCommand.Path
$repo = Split-Path -Parent $kit
$scenes = (Import-Csv (Join-Path $kit 'scenes.csv')).slug

$stage = Join-Path $env:TEMP ('fd-bench-kit-' + [guid]::NewGuid().ToString('N').Substring(0, 8))
New-Item -ItemType Directory -Force $stage | Out-Null
try {
    foreach ($f in 'README.md', 'scenes.csv', 'bench-lib.ps1', 'run-all.ps1',
                   'bench-latest.ps1', 'bench-latest.sh') {
        Copy-Item (Join-Path $kit $f) $stage
    }
    New-Item -ItemType Directory -Force (Join-Path $stage 'scenes') | Out-Null
    foreach ($s in $scenes) {
        foreach ($ext in '.kfr', '.f3.toml', '.fdn') {
            Copy-Item (Join-Path $repo ('validation\corpus\locations\' + $s + $ext)) (Join-Path $stage 'scenes')
        }
        # ⚠The corpus .f3.toml files are CORRECTNESS fixtures and render 1280x720; the benchmark
        # protocol is 1920x1080 for every lane. Copying them verbatim silently gave Fraktaler-3
        # 2.25x FEWER PIXELS than every other lane while the README promised equal work - the
        # first published run compared 0.92 MP against 2.07 MP. Rewrite the size in the kit's
        # copy only; the corpus originals must stay at their blessed resolution.
        $toml = Join-Path $stage ('scenes\' + $s + '.f3.toml')
        (Get-Content $toml) `
            -replace '^\s*width\s*=\s*\d+', 'width = 1920' `
            -replace '^\s*height\s*=\s*\d+', 'height = 1080' |
            Set-Content $toml -Encoding ascii
    }
    # Fraktaler-3: binary, manual, and source (AGPL) from the vendored copy.
    New-Item -ItemType Directory -Force (Join-Path $stage 'fraktaler3') | Out-Null
    $f3src = Join-Path $repo 'diag\fraktaler'
    Copy-Item (Join-Path $f3src 'fraktaler-3-3.1.x86_64.exe') (Join-Path $stage 'fraktaler3\fraktaler-3.exe')
    Copy-Item (Join-Path $f3src 'fraktaler-3-3.1.pdf') (Join-Path $stage 'fraktaler3')
    Copy-Item -Recurse (Join-Path $f3src 'source') (Join-Path $stage 'fraktaler3\source')
    # Fractadyne binary, when available (release zips already ship one; a repo build works too).
    New-Item -ItemType Directory -Force (Join-Path $stage 'bin') | Out-Null
    if (-not $FractadyneExe) { $FractadyneExe = Join-Path $repo 'target\release\fractadyne.exe' }
    if (Test-Path $FractadyneExe) {
        Copy-Item $FractadyneExe (Join-Path $stage 'bin\fractadyne.exe')
    } else {
        'Place fractadyne.exe here (or pass -FractadyneExe to run-all.ps1).' |
            Out-File (Join-Path $stage 'bin\PUT-FRACTADYNE-HERE.txt') -Encoding ascii
    }

    $ver = 'dev'
    $cargo = Get-Content (Join-Path $repo 'Cargo.toml') | Where-Object { $_ -match '^version = "(.+)"' } | Select-Object -First 1
    if ($cargo -match '"(.+)"') { $ver = $Matches[1] }
    $zip = Join-Path $kit ('fractadyne-bench-kit-' + $ver + '.zip')
    if (Test-Path $zip) { Remove-Item $zip }
    Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip
    Write-Host ('Kit written: ' + $zip)
    Write-Host ('Size: ' + [math]::Round((Get-Item $zip).Length / 1MB, 1) + ' MB')
} finally {
    Remove-Item -Recurse -Force $stage -ErrorAction SilentlyContinue
}
