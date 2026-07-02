<#
.SYNOPSIS
  Compare two Fractadyne profile JSON logs (before/after an optimization).

.DESCRIPTION
  Matches regions by name and reports the change in each timed stage (reference, series-skip,
  GPU iterate, GPU render) and the total, flagging speed-ups and regressions. Use this to
  validate that a change actually helped — and didn't regress another region.

.EXAMPLE
  ./scripts/profile-compare.ps1 logs/profile-before.json logs/profile-after.json
#>
param(
    [Parameter(Mandatory)][string]$Before,
    [Parameter(Mandatory)][string]$After
)
$ErrorActionPreference = "Stop"
$a = Get-Content $Before -Raw | ConvertFrom-Json
$b = Get-Content $After -Raw | ConvertFrom-Json

Write-Host "Before: $($a.utc)  $($a.gpu) / $($a.cpu)  SA=$($a.series_approx) BLA=$($a.use_bla)"
Write-Host "After:  $($b.utc)  $($b.gpu) / $($b.cpu)  SA=$($b.series_approx) BLA=$($b.use_bla)"
if ($a.gpu -ne $b.gpu -or $a.cpu -ne $b.cpu) {
    Write-Host "WARNING: hardware differs between runs — timings are not directly comparable." -ForegroundColor Yellow
}
Write-Host ""

$bmap = @{}
foreach ($r in $b.regions) { $bmap[$r.name] = $r }

# Coalesce a possibly-missing field (older logs predate bla_build) to 0.
function V($x) { if ($null -eq $x) { 0.0 } else { [double]$x } }
function Total($r) { (V $r.timings_ms.reference) + (V $r.timings_ms.series_skip) + (V $r.timings_ms.bla_build) + (V $r.timings_ms.gpu_render.median) }

"{0,-20} {1,10} {2,10} {3,9}" -f "region / metric", "before", "after", "change" | Write-Host
"-" * 54 | Write-Host
foreach ($r in $a.regions) {
    $o = $bmap[$r.name]
    if (-not $o) { Write-Host ("{0,-20}  (missing in 'after')" -f $r.name); continue }
    $rows = @(
        @{ k = "reference";  bv = (V $r.timings_ms.reference);          av = (V $o.timings_ms.reference) }
        @{ k = "series";     bv = (V $r.timings_ms.series_skip);        av = (V $o.timings_ms.series_skip) }
        @{ k = "bla_build";  bv = (V $r.timings_ms.bla_build);          av = (V $o.timings_ms.bla_build) }
        @{ k = "gpu_iterate";bv = (V $r.timings_ms.gpu_iterate.median); av = (V $o.timings_ms.gpu_iterate.median) }
        @{ k = "gpu_render"; bv = (V $r.timings_ms.gpu_render.median);  av = (V $o.timings_ms.gpu_render.median) }
        @{ k = "TOTAL";      bv = (Total $r);                           av = (Total $o) }
    )
    Write-Host ("{0}  (mode {1}, skip {2}->{3})" -f $r.name, $r.mode, $r.sa_skip, $o.sa_skip)
    foreach ($row in $rows) {
        $bv = [double]$row.bv; $av = [double]$row.av
        $pct = if ($bv -gt 1e-6) { ($av - $bv) / $bv * 100 } else { 0 }
        $tag = if ($pct -lt -3) { "faster" } elseif ($pct -gt 3) { "SLOWER" } else { "~same" }
        $color = if ($pct -lt -3) { "Green" } elseif ($pct -gt 3) { "Red" } else { "Gray" }
        Write-Host ("  {0,-16} {1,10:N2} {2,10:N2} {3,8:N1}%  {4}" -f $row.k, $bv, $av, $pct, $tag) -ForegroundColor $color
    }
}
