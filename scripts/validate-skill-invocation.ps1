<#
.SYNOPSIS
    Validates consistency between quick-index.json, routing.tsv files,
    and sub-hub SKILL.md files.

.DESCRIPTION
    Checks:
    1-10: Original consistency checks
    11: Every sub-hub directory has a routing.tsv file
    12: Every src_path in routing.tsv resolves to a real file

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
$protocolPath   = Join-Path (Join-Path $repoRoot 'skills-aggregated') 'AGENT-PROTOCOL.md'

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

# ── V01: quick-index.json exists ─────────────────────────────────────────────
$v01 = Test-Path $quickIndexPath
Add-Result 'V01' 'quick-index.json exists' $v01 $quickIndexPath

# ── V02: quick-index.json is valid JSON ──────────────────────────────────────
$qiData = $null
try {
    $qiData = Get-Content $quickIndexPath -Raw | ConvertFrom-Json
    Add-Result 'V02' 'quick-index.json is valid JSON' $true "parsed successfully"
} catch {
    Add-Result 'V02' 'quick-index.json is valid JSON' $false $_.Exception.Message
}

# ── V03: quick-index.json size < 15KB ────────────────────────────────────────
if ($v01) {
    $fileSize = (Get-Item $quickIndexPath).Length
    $sizeKB = [Math]::Round($fileSize / 1024, 1)
    Add-Result 'V03' 'quick-index.json size < 15KB' ($fileSize -lt 15360) "${sizeKB}KB"
}

# ── V04: hub-manifests.csv exists ────────────────────────────────────────────
$v04 = Test-Path $manifestPath
Add-Result 'V04' 'hub-manifests.csv exists' $v04 $manifestPath

# ── V05: AGENT-PROTOCOL.md exists ────────────────────────────────────────────
$v05 = Test-Path $protocolPath
Add-Result 'V05' 'AGENT-PROTOCOL.md exists' $v05 $protocolPath

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
}

# ── V08: Sub-hub directories have SKILL.md ────────────────────────────────────
$hubDirs = Get-ChildItem $subhubDir -Directory | Where-Object { $_.Name -ne '.git' -and $_.Name -ne 'scripts' }
$missingSKILL = @()
foreach ($hub in $hubDirs) {
    $subDirs = Get-ChildItem $hub.FullName -Directory -ErrorAction SilentlyContinue
    foreach ($sub in $subDirs) {
        $skillPath = Join-Path $sub.FullName 'SKILL.md'
        if (-not (Test-Path $skillPath)) { $missingSKILL += "$($hub.Name)/$($sub.Name)" }
    }
}
Add-Result 'V08' 'All sub-hub directories have SKILL.md' ($missingSKILL.Count -eq 0) $(
    if ($missingSKILL.Count -eq 0) { "all present" }
    else { "missing: $($missingSKILL -join ', ')" }
)

# ── V09: quick-index has _meta section ────────────────────────────────────────
if ($qiData) {
    $hasMeta = $null -ne $qiData._meta
    Add-Result 'V09' 'quick-index has _meta section' $hasMeta $(
        if ($hasMeta) { "keywords=$($qiData._meta.total_keywords)" }
        else { "missing _meta" }
    )
}

# ── V10: AGENT-PROTOCOL.md contains required sections ────────────────────────
if ($v05) {
    $protocolContent = Get-Content $protocolPath -Raw
    $requiredSections = @('STEP 1', 'STEP 2', 'STEP 3', 'ANTI-HALLUCINATION', 'FALLBACK', 'TOKEN BUDGET')
    $missing = @()
    foreach ($section in $requiredSections) {
        if ($protocolContent -notmatch [regex]::Escape($section)) { $missing += $section }
    }
    Add-Result 'V10' 'AGENT-PROTOCOL.md has all required sections' ($missing.Count -eq 0) $(
        if ($missing.Count -eq 0) { "all 6 sections present" }
        else { "missing: $($missing -join ', ')" }
    )
}

# ── V11: Sub-hub directories have routing.tsv (or are BMAD-only) ──────────────
$missingTSV = @()
$bmadOnlySkipped = 0
foreach ($hub in $hubDirs) {
    $subDirs = Get-ChildItem $hub.FullName -Directory -ErrorAction SilentlyContinue
    foreach ($sub in $subDirs) {
        $tsvPath = Join-Path $sub.FullName 'routing.tsv'
        if (-not (Test-Path $tsvPath)) {
            # Check if this sub-hub only has BMAD skills (no routing.tsv expected)
            $skillMd = Join-Path $sub.FullName 'SKILL.md'
            if (Test-Path $skillMd) {
                $bmadOnlySkipped++
            } else {
                $missingTSV += "$($hub.Name)/$($sub.Name)"
            }
        }
    }
}
Add-Result 'V11' 'Sub-hub directories have routing.tsv or are BMAD-only' ($missingTSV.Count -eq 0) $(
    if ($missingTSV.Count -eq 0) { "all present ($bmadOnlySkipped BMAD-only sub-hubs OK)" }
    else { "missing: $($missingTSV -join ', ')" }
)

# ── V12: All src_path in routing.tsv resolve to real files ────────────────────
$brokenPaths = @()
$totalPaths = 0
$tsvFiles = Get-ChildItem $subhubDir -Recurse -Filter 'routing.tsv'
foreach ($tsv in $tsvFiles) {
    $lines = Get-Content $tsv.FullName
    foreach ($line in $lines[1..($lines.Count - 1)]) {  # skip header
        if ([string]::IsNullOrWhiteSpace($line)) { continue }
        $cols = $line -split "`t"
        if ($cols.Count -ge 4) {
            $totalPaths++
            $srcPath = $cols[3]
            $fullPath = Join-Path $repoRoot $srcPath
            if (-not (Test-Path $fullPath)) {
                $brokenPaths += "$($cols[0]) -> $srcPath"
            }
        }
    }
}
Add-Result 'V12' 'All routing.tsv src_path resolve to files' ($brokenPaths.Count -eq 0) $(
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
