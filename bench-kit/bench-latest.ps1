# bench-latest.ps1 - download the LATEST releases of every benchmarked renderer, run the
# benchmark sequentially, and produce the summary report. ASCII only (PowerShell 5.1 reads
# BOM-less files as ANSI; no smart punctuation).
#
#   powershell -ExecutionPolicy Bypass -File bench-latest.ps1
#       [-AppsDir <folder>]      where the apps land (default <kit>\apps); free space verified
#       [-RequiredGB 2.0]        free-space floor for downloads + extraction + results
#       [-Reps 1] [-Skip imagina,fractalshark] [-TimeoutS 7200] [-Scenes 03-seahorse-1e6,...]
#       [-SkipDownload]          reuse whatever AppsDir already holds (offline mode)
#       [-FractadyneExe <path>]  benchmark a LOCAL fractadyne build instead of the release
#
# What gets fetched, and from where (versions recorded in apps-manifest.txt and the summary):
#   Fractadyne    github.com/WindySnowOwl/fractadyne     newest release with a windows-x64 zip
#                 (prereleases included - the beta track is where Linux/Windows pairs live);
#                 sha256 side-asset verified when published.
#   Fraktaler-3   fraktaler.mathr.co.uk/download/latest  fraktaler-3-<v>-windows.7z. Extracted
#                 with 7z.exe when installed, else Windows bsdtar (whose 7z reader fails on the
#                 ARM exes in the archive - tolerated; only the x86_64 exe is validated). If the
#                 download or extraction fails the kit's vendored copy is used and noted.
#   Imagina       github.com/5E-324/Imagina              newest release (all are prereleases,
#                 so /releases/latest 404s - the script scans /releases).
#   FractalShark  github.com/mattsaccount364/FractalShark  newest FractalShark-<v>.zip. CUDA:
#                 downloaded only when an NVIDIA GPU is present and the lane is not skipped
#                 (the asset is ~160 MB - no point fetching it for an N/A lane).
#
# The run itself is exactly run-all.ps1 (sequential lanes, results\<host>-<stamp>\ with
# sysinfo.txt, results.csv, summary.md); this script resolves the freshly downloaded exes,
# invokes it, then stamps the app versions into the summary so a "latest" result always says
# which latest it measured.

[CmdletBinding()]
param(
    [string]$AppsDir = '',
    [double]$RequiredGB = 2.0,
    [int]$Reps = 1,
    [string[]]$Skip = @(),
    [int]$TimeoutS = 7200,
    [ValidatePattern('^[0-9]+x[0-9]+$')]
    [string]$Size = '3840x2160',
    [string[]]$Scenes = @(),
    [switch]$SkipDownload,
    [string]$FractadyneExe = ''
)

$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
# `powershell -File` passes `-Skip a,b` as ONE comma-bearing string; normalize (see run-all).
$Skip = @($Skip | ForEach-Object { $_ -split ',' } | Where-Object { $_ })
$Scenes = @($Scenes | ForEach-Object { $_ -split ',' } | Where-Object { $_ })
$kit = Split-Path -Parent $MyInvocation.MyCommand.Path
if (-not $AppsDir) { $AppsDir = Join-Path $kit 'apps' }
New-Item -ItemType Directory -Force $AppsDir | Out-Null
$AppsDir = (Resolve-Path $AppsDir).Path
$manifest = Join-Path $AppsDir 'apps-manifest.txt'
$ua = @{ 'User-Agent' = 'fractadyne-bench-kit' }

function Write-Step($msg) { Write-Host ('== ' + $msg) }

# ---- free-space check on the AppsDir's drive ----
if (-not $SkipDownload) {
    $root = [System.IO.Path]::GetPathRoot($AppsDir)
    try {
        $drive = New-Object System.IO.DriveInfo($root)
        $freeGB = [math]::Round($drive.AvailableFreeSpace / 1GB, 2)
        if ($freeGB -lt $RequiredGB) {
            Write-Host ('ERROR: ' + $root + ' has ' + $freeGB + ' GB free; ' + $RequiredGB +
                ' GB required (downloads + extraction + results). Free space, choose another ' +
                '-AppsDir, or lower -RequiredGB if you know better.')
            exit 1
        }
        Write-Step ('Space: ' + $freeGB + ' GB free on ' + $root + ' (need ' + $RequiredGB + ' GB) - ok')
    } catch {
        Write-Host ('WARNING: cannot read free space for ' + $root + ' (' + $_.Exception.Message +
            ') - continuing without the check.')
    }
}

# ---- download helpers ----

# Newest GitHub release (prereleases INCLUDED) whose assets match $assetPattern. /releases is
# newest-first; /releases/latest is useless here twice over: it skips prereleases (Imagina has
# nothing else; fractadyne's Linux/Windows pairs live on the beta track) and it can name a
# release that lacks the platform asset entirely.
function Get-GithubAsset($repo, $assetPattern) {
    $rel = Invoke-RestMethod -Headers $ua -Uri ('https://api.github.com/repos/' + $repo + '/releases?per_page=15')
    foreach ($r in $rel) {
        $asset = $r.assets | Where-Object { $_.name -match $assetPattern } | Select-Object -First 1
        if ($asset) {
            $sha = $r.assets | Where-Object { $_.name -eq ($asset.name + '.sha256') } | Select-Object -First 1
            return @{ tag = $r.tag_name; name = $asset.name; url = $asset.browser_download_url
                      sha_url = $(if ($sha) { $sha.browser_download_url } else { '' }) }
        }
    }
    throw ('no release asset in ' + $repo + ' matches ' + $assetPattern)
}

function Get-File($url, $dest) {
    Write-Host ('   fetching ' + $url)
    Invoke-WebRequest -Headers $ua -Uri $url -OutFile $dest -UseBasicParsing
    Write-Host ('   ' + [math]::Round((Get-Item $dest).Length / 1MB, 1) + ' MB')
}

# Verify "<hash>  <name>" side-file when one was published; a mismatch is a hard stop.
function Test-Sha256($file, $shaUrl) {
    if (-not $shaUrl) { return }
    # GitHub serves release assets as octet-stream, so .Content arrives as byte[], not string.
    $c = (Invoke-WebRequest -Headers $ua -Uri $shaUrl -UseBasicParsing).Content
    if ($c -is [byte[]]) { $c = [System.Text.Encoding]::ASCII.GetString($c) }
    $expected = ($c -split '\s+')[0].ToLower()
    $actual = (Get-FileHash $file -Algorithm SHA256).Hash.ToLower()
    if ($expected -ne $actual) { throw ('sha256 mismatch for ' + $file) }
    Write-Host '   sha256 verified'
}

# Extract zip/7z/tar.* into a fresh folder and return the folder.
# .zip: Expand-Archive first; on failure (Imagina ships Deflate64, which .NET cannot read and
#   bsdtar SILENTLY CORRUPTS - measured: plausible sizes, wrong bytes) fall back to the
#   Explorer COM handler, which decodes Deflate64 correctly. CopyHere is asynchronous, so the
#   fallback polls until the top-level item count matches the archive's.
# .7z: 7z.exe when present, else Windows bsdtar. BOTH are allowed to exit nonzero: each fails
#   on the BCJ-filtered ARM exes inside the Fraktaler-3 archive while extracting the x86_64
#   exe byte-perfectly (verified against the vendored copy) - so the exit code is a warning
#   and the CALLER validates the one file it actually needs.
function Expand-Any($archive, $destName) {
    $dest = Join-Path $AppsDir $destName
    if (Test-Path $dest) { Remove-Item -Recurse -Force $dest }
    New-Item -ItemType Directory -Force $dest | Out-Null
    if ($archive -match '\.zip$') {
        try {
            Expand-Archive -Path $archive -DestinationPath $dest -Force -ErrorAction Stop
        } catch {
            Write-Host '   (Expand-Archive cannot read this zip - using the Explorer handler)'
            $shell = New-Object -ComObject Shell.Application
            $items = $shell.NameSpace($archive).Items()
            $shell.NameSpace($dest).CopyHere($items, 4 + 16) # no UI, yes-to-all
            $deadline = (Get-Date).AddSeconds(300)
            while ((Get-ChildItem $dest).Count -lt $items.Count) {
                if ((Get-Date) -gt $deadline) { throw ('Explorer extraction timed out on ' + $archive) }
                Start-Sleep -Milliseconds 500
            }
        }
    } elseif ($archive -match '\.7z$') {
        $sevenZip = @('7z.exe', (Join-Path $env:ProgramFiles '7-Zip\7z.exe')) |
            Where-Object { Get-Command $_ -ErrorAction SilentlyContinue } | Select-Object -First 1
        # EAP is lowered around the native call: under Stop, PS 5.1 turns any redirected native
        # stderr into a terminating NativeCommandError - and these extractors are EXPECTED to
        # complain about the ARM entries while delivering the x86_64 one intact.
        $eap = $ErrorActionPreference; $ErrorActionPreference = 'Continue'
        if ($sevenZip) {
            & $sevenZip x ('-o' + $dest) -y $archive 2>$null | Out-Null
        } else {
            & (Join-Path $env:WINDIR 'System32\tar.exe') -xf $archive -C $dest 2>$null
        }
        $ErrorActionPreference = $eap
        if ($LASTEXITCODE -ne 0) {
            Write-Host '   (extractor reported per-file errors - validating the file we need)'
        }
    } else {
        & (Join-Path $env:WINDIR 'System32\tar.exe') -xf $archive -C $dest
        if ($LASTEXITCODE -ne 0) { throw ('tar failed on ' + $archive) }
    }
    $dest
}

# Find one exe under $dir matching $namePattern; empty string when absent.
function Find-Exe($dir, $namePattern) {
    $hit = Get-ChildItem -Recurse -Path $dir -Filter '*.exe' -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -match $namePattern } | Select-Object -First 1
    if ($hit) { $hit.FullName } else { '' }
}

$manifestLines = @('Fractadyne bench-kit app manifest - ' + (Get-Date -Format 'yyyy-MM-dd HH:mm:ss'))
function Note-App($name, $version, $source, $exe) {
    $hash = $(if ($exe -and (Test-Path $exe)) { (Get-FileHash $exe -Algorithm SHA256).Hash.ToLower() } else { '-' })
    $script:manifestLines += ('{0}: {1}  source={2}  exe={3}  sha256={4}' -f $name, $version, $source, $exe, $hash)
}

# ---- lane preconditions (mirrors run-all: no NVIDIA GPU means the CUDA lane is N/A) ----
$skipL = @(); foreach ($k in $Skip) { $skipL += $k.ToLower() }
$hasNvidia = [bool](Get-CimInstance Win32_VideoController | Where-Object { $_.Name -match 'NVIDIA' })
$wantImagina = -not ($skipL -contains 'imagina')
$wantShark = (-not ($skipL -contains 'fractalshark')) -and $hasNvidia
if ((-not ($skipL -contains 'fractalshark')) -and -not $hasNvidia) {
    Write-Step 'FractalShark: no NVIDIA GPU - lane will be N/A, release not downloaded.'
}

# ---- fetch (or reuse) each app ----
$fraktaler3Exe = ''
$imaginaExe = ''
$fractalSharkExe = ''
$fractalSharkCliExe = ''

if ($SkipDownload) {
    Write-Step ('Offline mode: using whatever ' + $AppsDir + ' already holds.')
    if (-not $FractadyneExe) { $FractadyneExe = Find-Exe $AppsDir '^fractadyne\.exe$' }
    $fraktaler3Exe = Find-Exe $AppsDir '^fraktaler-3.*x86_64\.exe$'
    $imaginaExe = Find-Exe $AppsDir '^Imagina.*\.exe$'
    # ANCHOR the name. '^FractalShark.*\.exe$' also matches FractalSharkCli.exe and
    # FractalSharkTest.exe, so which binary the lane got depended on enumeration order -- and
    # handing the assisted lane the TEST RUNNER would look like a hung render.
    $fractalSharkExe = Find-Exe $AppsDir '^FractalShark\.exe$'
    $fractalSharkCliExe = Find-Exe $AppsDir '^FractalSharkCli\.exe$'
    Note-App 'fractadyne' 'offline-reuse' $AppsDir $FractadyneExe
    Note-App 'fraktaler3' 'offline-reuse' $AppsDir $fraktaler3Exe
    Note-App 'imagina' 'offline-reuse' $AppsDir $imaginaExe
    Note-App 'fractalshark' 'offline-reuse' $AppsDir $fractalSharkExe
} else {
    # Fractadyne - skipped entirely when the operator points at a local build.
    if ($FractadyneExe) {
        Write-Step ('Fractadyne: using local build ' + $FractadyneExe)
        Note-App 'fractadyne' 'local-build' $FractadyneExe $FractadyneExe
    } else {
        Write-Step 'Fractadyne: newest release with a windows-x64 zip'
        try {
            $a = Get-GithubAsset 'WindySnowOwl/fractadyne' 'fractadyne-.*-windows-x64\.zip$'
            $zip = Join-Path $AppsDir $a.name
            Get-File $a.url $zip
            Test-Sha256 $zip $a.sha_url
            $FractadyneExe = Find-Exe (Expand-Any $zip 'fractadyne') '^fractadyne\.exe$'
            if (-not $FractadyneExe) { throw 'fractadyne.exe not found in the release zip' }
            Note-App 'fractadyne' $a.tag $a.url $FractadyneExe
        } catch {
            Write-Host ('   FAILED: ' + $_.Exception.Message + ' - falling back to the kit bin\ copy if present.')
            Note-App 'fractadyne' 'FETCH-FAILED' $_.Exception.Message ''
        }
    }

    # Fraktaler-3 - scrape the stable download/latest directory for the windows 7z.
    Write-Step 'Fraktaler-3: latest from fraktaler.mathr.co.uk'
    try {
        $base = 'https://fraktaler.mathr.co.uk/download/latest/'
        $page = (Invoke-WebRequest -Headers $ua -Uri $base -UseBasicParsing).Content
        if ($page -notmatch 'href="(fraktaler-3-[0-9.]+-windows\.7z)"') { throw 'no windows 7z listed' }
        $name = $Matches[1]
        $sevenZ = Join-Path $AppsDir $name
        Get-File ($base + $name) $sevenZ
        $f3dir = Expand-Any $sevenZ 'fraktaler3'
        $fraktaler3Exe = Find-Exe $f3dir '^fraktaler-3-[0-9.]+\.x86_64\.exe$'
        if (-not $fraktaler3Exe -or (Get-Item $fraktaler3Exe).Length -lt 1MB) {
            throw 'x86_64 exe missing or truncated after extraction'
        }
        $ver = $(if ($name -match 'fraktaler-3-([0-9.]+)-') { $Matches[1] } else { '?' })
        Note-App 'fraktaler3' $ver ($base + $name) $fraktaler3Exe
    } catch {
        Write-Host ('   FAILED: ' + $_.Exception.Message + ' - using the vendored copy.')
        $fraktaler3Exe = Join-Path $kit 'fraktaler3\fraktaler-3.exe'
        Note-App 'fraktaler3' 'vendored-3.1' 'kit' $fraktaler3Exe
    }

    # Imagina - every release is a prerelease, so scan /releases.
    if ($wantImagina) {
        Write-Step 'Imagina: newest release'
        try {
            $a = Get-GithubAsset '5E-324/Imagina' '^Imagina.*\.zip$'
            $zip = Join-Path $AppsDir $a.name
            Get-File $a.url $zip
            $imaginaExe = Find-Exe (Expand-Any $zip 'imagina') '^Imagina.*\.exe$'
            if (-not $imaginaExe) { throw 'no Imagina exe in the zip' }
            Note-App 'imagina' $a.tag $a.url $imaginaExe
        } catch {
            Write-Host ('   FAILED: ' + $_.Exception.Message + ' - lane will be skipped.')
            Note-App 'imagina' 'FETCH-FAILED' $_.Exception.Message ''
        }
    }

    # FractalShark - CUDA-only, so the ~160 MB asset is fetched only for NVIDIA machines.
    if ($wantShark) {
        Write-Step 'FractalShark: newest release'
        try {
            $a = Get-GithubAsset 'mattsaccount364/FractalShark' '^FractalShark-[0-9.]+\.zip$'
            $zip = Join-Path $AppsDir $a.name
            Get-File $a.url $zip
            $fsDir = Expand-Any $zip 'fractalshark'
            $fractalSharkExe = Find-Exe $fsDir '^FractalShark\.exe$'
            # The headless renderer ships in the same zip; passing it explicitly is what makes the
            # lane automated instead of a transcription prompt.
            $fractalSharkCliExe = Find-Exe $fsDir '^FractalSharkCli\.exe$'
            if (-not $fractalSharkExe) { throw 'no FractalShark exe in the zip' }
            Note-App 'fractalshark' $a.tag $a.url $fractalSharkExe
        } catch {
            Write-Host ('   FAILED: ' + $_.Exception.Message + ' - lane will be skipped.')
            Note-App 'fractalshark' 'FETCH-FAILED' $_.Exception.Message ''
        }
    }
}

$manifestLines | Out-File -FilePath $manifest -Encoding ascii
Write-Step ('Manifest: ' + $manifest)

# ---- run the benchmark (sequential; run-all owns the protocol and the report) ----
$runParams = @{ Reps = $Reps; TimeoutS = $TimeoutS; Size = $Size }
if ($FractadyneExe) { $runParams.FractadyneExe = $FractadyneExe }
if ($fraktaler3Exe) { $runParams.Fraktaler3Exe = $fraktaler3Exe }
if ($imaginaExe) { $runParams.ImaginaExe = $imaginaExe }
if ($fractalSharkExe) { $runParams.FractalSharkExe = $fractalSharkExe }
if ($fractalSharkCliExe) { $runParams.FractalSharkCliExe = $fractalSharkCliExe }
$effSkip = @($Skip)
if (-not $imaginaExe) { $effSkip += 'imagina' }
if ($effSkip.Count) { $runParams.Skip = [string[]]$effSkip }
if ($Scenes.Count) { $runParams.Scenes = [string[]]$Scenes }
Write-Step ('Benchmark: run-all.ps1 ' + (($runParams.GetEnumerator() |
    ForEach-Object { '-' + $_.Key + ' ' + ($_.Value -join ',') }) -join ' '))
& (Join-Path $kit 'run-all.ps1') @runParams

# ---- stamp the app versions into the newest results folder ----
$latest = Get-ChildItem (Join-Path $kit 'results') -Directory | Sort-Object Name | Select-Object -Last 1
if ($latest) {
    Copy-Item $manifest (Join-Path $latest.FullName 'apps-manifest.txt') -Force
    $summary = Join-Path $latest.FullName 'summary.md'
    if (Test-Path $summary) {
        $app = @('', '## Apps measured', '')
        foreach ($l in ($manifestLines | Select-Object -Skip 1)) { $app += ('- ' + $l) }
        Add-Content -Path $summary -Value $app -Encoding ascii
        Write-Host ''
        Get-Content $summary | Write-Host
    }
    Write-Step ('Report: ' + $summary)
}
