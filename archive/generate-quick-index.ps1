<#
.SYNOPSIS
    Generates quick-index.json — a compact keyword → hub/sub_hub lookup map.

.DESCRIPTION
    Reads ONLY subhub-index.json (curated top_triggers per sub-hub).
    Produces a flat dictionary where each keyword maps to one or more
    {hub, sub_hub} entries, sorted by skill_count descending.

    Does NOT read hub-manifests.csv — that would create 1000+ noisy entries.
    The subhub-index.json already contains the best routing keywords.

    Target: < 5KB output for ~50 token reads by agents.

.EXAMPLE
    .\archive\generate-quick-index.ps1
#>

[CmdletBinding()]
param(
    [string]$OutputPath,
    [string]$SubhubIndexPath
)

$ErrorActionPreference = 'Stop'

# ── Resolve paths ────────────────────────────────────────────────────────────
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$repoRoot  = (Resolve-Path (Join-Path $scriptDir '..')).Path

if (-not $OutputPath)       { $OutputPath       = Join-Path (Join-Path $repoRoot 'skills-aggregated') 'quick-index.json' }
if (-not $SubhubIndexPath)  { $SubhubIndexPath  = Join-Path (Join-Path $repoRoot 'skills-aggregated') 'subhub-index.json' }

Write-Host "── generate-quick-index.ps1 ──" -ForegroundColor Cyan
Write-Host "  subhub-index : $SubhubIndexPath"
Write-Host "  output       : $OutputPath"

# ── Load source ──────────────────────────────────────────────────────────────
if (-not (Test-Path $SubhubIndexPath)) { throw "Missing: $SubhubIndexPath" }
$subhubs = Get-Content $SubhubIndexPath -Raw | ConvertFrom-Json

Write-Host "`n  Sub-hubs loaded : $($subhubs.Count)"

# ── Stopwords: too ambiguous to route accurately ─────────────────────────────
$stopwords = @(
    'pro', 'expert', 'best', 'practices', 'patterns', 'development',
    'builder', 'guide', 'core', 'ms', 'check', 'review',
    'automation', 'context', 'agent', 'code', 'system',
    'ai', 'design', 'testing', 'service', 'deployment',
    'optimization', 'security', 'development', 'workflow',
    '', 'wds', 'azure', 'ts', 'error', 'fixing'
)

# ── Build keyword map ────────────────────────────────────────────────────────
$map = @{}

foreach ($sh in $subhubs) {
    $hub       = $sh.main_hub
    $sub_hub   = $sh.sub_hub
    $count     = [int]$sh.skill_count

    foreach ($kw in $sh.top_triggers) {
        $kw = $kw.Trim().ToLower()
        if ($kw.Length -lt 2) { continue }
        if ($kw -in $stopwords) { continue }

        if (-not $map.ContainsKey($kw)) {
            $map[$kw] = [System.Collections.ArrayList]::new()
        }

        # Avoid duplicate hub/sub_hub pairs
        $exists = $map[$kw] | Where-Object { $_.hub -eq $hub -and $_.sub_hub -eq $sub_hub }
        if (-not $exists) {
            [void]$map[$kw].Add(@{
                hub         = $hub
                sub_hub     = $sub_hub
                skill_count = $count
            })
        }
    }
}

Write-Host "  Unique keywords : $($map.Count)"

# ── Sort and build output ────────────────────────────────────────────────────
$sortedMap = [ordered]@{}
foreach ($kw in ($map.Keys | Sort-Object)) {
    $entries = @($map[$kw] | Sort-Object { -[int]$_.skill_count })

    if ($entries.Count -eq 1) {
        # Single mapping — flat object (saves tokens for agents)
        $sortedMap[$kw] = [ordered]@{
            hub     = $entries[0].hub
            sub_hub = $entries[0].sub_hub
        }
    } else {
        # Multiple candidates — array sorted by skill_count DESC
        $arr = @()
        foreach ($e in $entries) {
            $arr += [ordered]@{
                hub     = $e.hub
                sub_hub = $e.sub_hub
            }
        }
        $sortedMap[$kw] = $arr
    }
}

# ── Build compact JSON manually (PowerShell ConvertTo-Json is too verbose) ────
$lines = [System.Collections.ArrayList]::new()
[void]$lines.Add('{')

# _meta block
$metaJson = '  "_meta": {"generated":"' + (Get-Date -Format 'yyyy-MM-dd') + '","version":"1.0","total_keywords":' + $sortedMap.Count + ',"total_sub_hubs":' + $subhubs.Count + ',"usage":"keyword->hub/sub_hub. Object=unique. Array=candidates sorted by relevance."}'
[void]$lines.Add($metaJson + ',')
[void]$lines.Add('  "keywords": {')

# Keywords
$keys = @($sortedMap.Keys)
for ($i = 0; $i -lt $keys.Count; $i++) {
    $kw  = $keys[$i]
    $val = $sortedMap[$kw]
    $comma = if ($i -lt $keys.Count - 1) { ',' } else { '' }

    if ($val -is [System.Collections.Specialized.OrderedDictionary]) {
        # Single mapping
        $entry = '    "' + $kw + '": {"hub":"' + $val.hub + '","sub_hub":"' + $val.sub_hub + '"}' + $comma
    } else {
        # Array of candidates
        $items = @()
        foreach ($e in $val) { $items += '{"hub":"' + $e.hub + '","sub_hub":"' + $e.sub_hub + '"}' }
        $entry = '    "' + $kw + '": [' + ($items -join ',') + ']' + $comma
    }
    [void]$lines.Add($entry)
}

[void]$lines.Add('  }')
[void]$lines.Add('}')

$json = $lines -join "`n"
[System.IO.File]::WriteAllText($OutputPath, $json, [System.Text.UTF8Encoding]::new($false))

$fileSize = (Get-Item $OutputPath).Length
$fileSizeKB = [Math]::Round($fileSize / 1024, 1)

# ── Stats ────────────────────────────────────────────────────────────────────
$singleCount = ($sortedMap.Values | Where-Object { $_ -is [System.Collections.Specialized.OrderedDictionary] }).Count
$multiCount  = $sortedMap.Count - $singleCount
$coveredHubs = ($sortedMap.Values | ForEach-Object {
    if ($_ -is [array]) { $_ | ForEach-Object { "$($_.hub)/$($_.sub_hub)" } }
    else { "$($_.hub)/$($_.sub_hub)" }
} | Sort-Object -Unique).Count

Write-Host "`n  ✅ Generated: $OutputPath" -ForegroundColor Green
Write-Host "  File size        : $fileSizeKB KB"
Write-Host "  Single-mapping   : $singleCount keywords"
Write-Host "  Multi-mapping    : $multiCount keywords (array candidates)"
Write-Host "  Sub-hubs covered : $coveredHubs / $($subhubs.Count)"

if ($fileSize -gt 5120) {
    Write-Warning "  ⚠ File exceeds 5KB target ($fileSizeKB KB). Consider adding more stopwords."
} else {
    Write-Host "  ✅ Within 5KB target" -ForegroundColor Green
}
Write-Host ""
