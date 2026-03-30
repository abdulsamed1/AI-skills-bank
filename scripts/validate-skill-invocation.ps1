<#
.SYNOPSIS
    Validates consistency between quick-index.json, routing.csv files,
    and sub-hub SKILL.md files.

.DESCRIPTION
    Checks:
    1-10: Original consistency checks
    11: Every sub-hub directory has a routing.csv file
    12: Every src_path in routing.csv resolves to a real file

.EXAMPLE
    .\scripts\validate-skill-invocation.ps1
#>

[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$repoRoot  = (Resolve-Path (Join-Path $scriptDir '..')).Path

$quickIndexPath = Join-Path (Join-Path $repoRoot 'skills-aggregated') 'quick-index.json'
$manifestPath   = Join-Path $repoRoot 'hub-manifests.csv'
$subhubDir      = Join-Path $repoRoot 'skills-aggregated'
$protocolPath   = Join-Path (Join-Path $repoRoot './AGENTS.md') 'AGENTS.md'

$passed = 0
$failed = 0
$results = @()

function Add-Result {
    param([string]$ID, [string]$Name, [bool]$Pass, [string]$Details)
    $script:results += [PSCustomObject]@{
        ID      = $ID
        Name    = $Name
        Result  = if ($Pass) { 'PASS' } else { 'FAIL' }
        Details = $Details
    }
    if ($Pass) { $script:passed++ } else { $script:failed++ }
}

Write-Host "── validate-skill-invocation.ps1 ──" -ForegroundColor Cyan
Write-Host ""

# ── V01: quick-index.json exists (optional in CSV-only mode) ─────────────────
$v01 = Test-Path $quickIndexPath
if ($v01) {
    Add-Result 'V01' 'quick-index.json exists (optional)' $true $quickIndexPath
} else {
    Add-Result 'V01' 'quick-index.json exists (optional)' $true "not present (CSV-only mode)"
}

# ── V02: quick-index.json is valid JSON ──────────────────────────────────────
$qiData = $null
if ($v01) {
    try {
        $qiData = Get-Content $quickIndexPath -Raw | ConvertFrom-Json
        Add-Result 'V02' 'quick-index.json is valid JSON' $true "parsed successfully"
    } catch {
        Add-Result 'V02' 'quick-index.json is valid JSON' $false $_.Exception.Message
    }
} else {
    Add-Result 'V02' 'quick-index.json is valid JSON' $true "skipped (file not configured)"
}

# ── V03: quick-index.json size < 15KB ────────────────────────────────────────
if ($v01) {
    $fileSize = (Get-Item $quickIndexPath).Length
    $sizeKB = [Math]::Round($fileSize / 1024, 1)
    Add-Result 'V03' 'quick-index.json size < 15KB' ($fileSize -lt 15360) "${sizeKB}KB"
} else {
    Add-Result 'V03' 'quick-index.json size < 15KB' $true "skipped (file not configured)"
}

# ── V04: hub-manifests.csv exists ────────────────────────────────────────────
$v04 = Test-Path $manifestPath
Add-Result 'V04' 'hub-manifests.csv exists' $v04 $manifestPath

# ── V05: AGENTS.md exists (dynamic entrypoint) ────────────────────────
$v05 = Test-Path $protocolPath
Add-Result 'V05' 'AGENTS.md exists' $v05 $protocolPath

# ── V06: Load CSV and check hub/sub_hub coverage ────────────────────────────
if ($v04 -and $qiData) {
    $manifest = Import-Csv $manifestPath
    $csvPairs = $manifest | ForEach-Object { "$($_.hub)/$($_.sub_hub)" } | Sort-Object -Unique

    # Extract all hub/sub_hub pairs from quick-index
    $qiPairs = @()
    $keywords = $qiData.keywords
    foreach ($prop in $keywords.PSObject.Properties) {
        $val = $prop.Value
        if ($val -is [array]) {
            foreach ($entry in $val) {
                $qiPairs += "$($entry.hub)/$($entry.sub_hub)"
            }
        } else {
            $qiPairs += "$($val.hub)/$($val.sub_hub)"
        }
    }
    $qiPairs = $qiPairs | Sort-Object -Unique

    # Check: every quick-index pair exists in CSV
    $orphaned = @()
    foreach ($pair in $qiPairs) {
        if ($pair -notin $csvPairs) { $orphaned += $pair }
    }
    Add-Result 'V06' 'All quick-index hubs exist in CSV' ($orphaned.Count -eq 0) $(
        if ($orphaned.Count -eq 0) { "all $($qiPairs.Count) pairs valid" }
        else { "orphaned: $($orphaned -join ', ')" }
    )

    # ── V07: Coverage — how many CSV sub-hubs are in quick-index ──────────────
    $csvSubHubs = $csvPairs.Count
    $covered    = ($qiPairs | Where-Object { $_ -in $csvPairs }).Count
    Add-Result 'V07' 'Quick-index covers all CSV sub-hubs' ($covered -eq $csvSubHubs) "$covered / $csvSubHubs sub-hubs covered"
} else {
    Add-Result 'V06' 'All quick-index hubs exist in CSV' $true "skipped (quick-index not configured)"
    Add-Result 'V07' 'Quick-index covers all CSV sub-hubs' $true "skipped (quick-index not configured)"
}

# ── V08: Per-subhub SKILL.md not required ───────────────────────────────
# SKILL.md files in the aggregated output are no longer generated. Agents
# should use `./AGENTS.md` + routing.csv to discover skills.
Add-Result 'V08' 'Per-subhub SKILL.md not required (use AGENTS.md + routing.csv)' $true "SKIP: SKILL.md not required"

# ── V09: quick-index has _meta section ────────────────────────────────────────
if ($qiData) {
    $hasMeta = $null -ne $qiData._meta
    Add-Result 'V09' 'quick-index has _meta section' $hasMeta $(
        if ($hasMeta) { "keywords=$($qiData._meta.total_keywords)" }
        else { "missing _meta" }
    )
} else {
    Add-Result 'V09' 'quick-index has _meta section' $true "skipped (quick-index not configured)"
}

# ── V10: AGENTS.md contains guidance (routing) ───────────────────────
if ($v05) {
    $protocolContent = Get-Content $protocolPath -Raw
    $hasRoutingHint = $protocolContent -match 'routing.csv' -or $protocolContent -match 'Route'
    Add-Result 'V10' 'AGENTS.md contains routing guidance' $hasRoutingHint $(
        if ($hasRoutingHint) { "contains routing guidance" } else { "missing routing guidance" }
    )
}

# ── V11: Sub-hub directories have routing.csv (or are BMAD-only) ──────────────
$missingCSV = @()
$bmadOnlySkipped = 0
foreach ($hub in $hubDirs) {
    $subDirs = Get-ChildItem $hub.FullName -Directory -ErrorAction SilentlyContinue
    foreach ($sub in $subDirs) {
        $csvPath = Join-Path $sub.FullName 'routing.csv'
        if (-not (Test-Path $csvPath)) {
            # Check if this sub-hub only has BMAD skills (no routing.csv expected)
            $skillMd = Join-Path $sub.FullName 'SKILL.md'
            if (Test-Path $skillMd) {
                $bmadOnlySkipped++
            } else {
                $missingCSV += "$($hub.Name)/$($sub.Name)"
            }
        }
    }
}
Add-Result 'V11' 'Sub-hub directories have routing.csv or are BMAD-only' ($missingCSV.Count -eq 0) $(
    if ($missingCSV.Count -eq 0) { "all present ($bmadOnlySkipped BMAD-only sub-hubs OK)" }
    else { "missing: $($missingCSV -join ', ')" }
)

# ── V12: All src_path in routing.csv resolve to real files ────────────────────
$brokenPaths = @()
$totalPaths = 0
$csvFiles = Get-ChildItem $subhubDir -Recurse -Filter 'routing.csv'
foreach ($csv in $csvFiles) {
    $rows = Import-Csv $csv.FullName
    foreach ($row in $rows) {
        if ([string]::IsNullOrWhiteSpace([string]$row.src_path)) { continue }
        $totalPaths++
        $srcPath = [string]$row.src_path
        $fullPath = Join-Path $repoRoot $srcPath
        if (-not (Test-Path $fullPath)) {
            $brokenPaths += "$($row.skill_id) -> $srcPath"
        }
    }
}
Add-Result 'V12' 'All routing.csv src_path resolve to files' ($brokenPaths.Count -eq 0) $(
    if ($brokenPaths.Count -eq 0) { "all $totalPaths paths valid" }
    else { "$($brokenPaths.Count) broken: $($brokenPaths[0..4] -join ', ')" }
)

# ── Output Results ───────────────────────────────────────────────────────────
Write-Host ""
Write-Host "# Skill Invocation Validation Report" -ForegroundColor Cyan
Write-Host ""
Write-Host "Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')"
Write-Host "Summary: $passed/$($passed + $failed) passed"
Write-Host ""

$results | Format-Table -AutoSize

if ($failed -gt 0) {
    Write-Host "❌ $failed check(s) FAILED" -ForegroundColor Red
    exit 1
} else {
    Write-Host "✅ All $passed checks passed" -ForegroundColor Green
    exit 0
}
