<#
.SYNOPSIS
    Generates per-sub-hub routing.csv files — lean skill lookup for agents.

.DESCRIPTION
    Reads hub-manifests.csv and resolves each skill_id to its SKILL.md
    path inside src/. Outputs one routing.csv per sub-hub directory
    containing ONLY what agents need: skill_id, triggers, score, src_path.

    Excludes internal/BMAD skills (module != external sources) since
    they are accessible via the IDE skill system directly.

    Agents read these files instead of hub-manifests.csv (572KB → ~2-13KB each).

.EXAMPLE
    .\scripts\generate-routing-tsv.ps1
    .\scripts\generate-routing-tsv.ps1 -DryRun
#>

[CmdletBinding()]
param(
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'

# ── Resolve paths ────────────────────────────────────────────────────────────
$scriptDir   = Split-Path -Parent $MyInvocation.MyCommand.Definition
$repoRoot    = (Resolve-Path (Join-Path $scriptDir '..')).Path
$manifestPath = Join-Path $repoRoot 'hub-manifests.csv'
$skillsAgg   = Join-Path $repoRoot 'skills-aggregated'
$srcRoot     = Join-Path $repoRoot 'src'

Write-Host "── generate-routing-tsv.ps1 ──" -ForegroundColor Cyan
Write-Host "  manifest : $manifestPath"
Write-Host "  src root : $srcRoot"
Write-Host "  output   : $skillsAgg/{hub}/{sub_hub}/routing.csv"
if ($DryRun) { Write-Host "  MODE     : DRY RUN" -ForegroundColor Yellow }

# ── Load CSV ─────────────────────────────────────────────────────────────────
if (-not (Test-Path $manifestPath)) { throw "Missing: $manifestPath" }
$allRows = Import-Csv $manifestPath
Write-Host "`n  Total CSV rows: $($allRows.Count)"

# ── Build src_path lookup: skill_id → relative path from repo root ────────
#    Scans src/*/skills/*/SKILL.md once to build a hash table.
Write-Host "  Scanning src/ for SKILL.md files..."

$srcPathMap = @{}
$srcRepos = Get-ChildItem $srcRoot -Directory -ErrorAction SilentlyContinue
foreach ($repo in $srcRepos) {
    $skillsDir = Join-Path $repo.FullName 'skills'
    if (-not (Test-Path $skillsDir)) { continue }

    $skillDirs = Get-ChildItem $skillsDir -Directory -ErrorAction SilentlyContinue
    foreach ($sd in $skillDirs) {
        $skillMd = Join-Path $sd.FullName 'SKILL.md'
        if (Test-Path $skillMd) {
            $absPath = $skillMd
            # First match wins (handles duplicates by repo order)
            if (-not $srcPathMap.ContainsKey($sd.Name)) {
                $srcPathMap[$sd.Name] = $absPath
            }
        }
    }
}
Write-Host "  Resolved src paths: $($srcPathMap.Count) skills"

# ── Group CSV by hub/sub_hub ─────────────────────────────────────────────────
$groups = $allRows | Group-Object { "$($_.hub)|$($_.sub_hub)" }
Write-Host "  Sub-hub groups: $($groups.Count)"

# ── Generate routing.csv per sub-hub ─────────────────────────────────────────
$filesWritten  = 0
$totalSkills   = 0
$unresolvedIds = [System.Collections.ArrayList]::new()

foreach ($group in $groups) {
    $parts   = $group.Name -split '\|'
    $hub     = $parts[0]
    $subHub  = $parts[1]
    $outDir  = Join-Path $skillsAgg "$hub\$subHub"
    $outFile = Join-Path $outDir 'routing.csv'

    if (-not (Test-Path $outDir)) {
        Write-Warning "  Directory missing: $outDir — skipped"
        continue
    }

    # Sort by score descending (agents read top-down, stop early)
    $sorted = $group.Group | Sort-Object { -[int]$_.match_score }, skill_id

    # Build CSV rows
    $rows = [System.Collections.ArrayList]::new()

    foreach ($row in $sorted) {
        $skillId  = $row.skill_id
        $triggers = $row.triggers
        $score    = $row.match_score

        # Resolve src_path from src/ directly (no copy into hub directories)
        $srcPath = ''
        if ($srcPathMap.ContainsKey($skillId)) {
            $sourceAbsolute = $srcPathMap[$skillId]
            $relative = $sourceAbsolute.Substring($repoRoot.Length).TrimStart('\\')
            $srcPath = ($relative -replace '\\', '/')
        } else {
            # Skip internal/BMAD skills — they live in .agent/skills/
            [void]$unresolvedIds.Add("${hub}/${subHub} -- ${skillId}")
            continue
        }

        [void]$rows.Add([PSCustomObject]@{
            skill_id = $skillId
            triggers = $triggers
            score = $score
            src_path = $srcPath
        })
        $totalSkills++
    }

    # Only write if we have at least 1 skill row
    if ($rows.Count -eq 0) {
        Write-Host "  Skipped (no external skills): $hub/$subHub" -ForegroundColor DarkGray
        continue
    }

    if ($DryRun) {
        Write-Host "  [dry-run] Would write: $outFile ($($rows.Count) skills)" -ForegroundColor Yellow
    } else {
        $content = ($rows | ConvertTo-Csv -NoTypeInformation) -join "`n"
        [System.IO.File]::WriteAllText($outFile, $content, [System.Text.UTF8Encoding]::new($false))
        $sizeKB = [Math]::Round((Get-Item $outFile).Length / 1024, 1)
        Write-Host "  ✅ $hub/$subHub — $($rows.Count) skills, ${sizeKB}KB" -ForegroundColor Green
    }
    $filesWritten++
}

# ── Summary ──────────────────────────────────────────────────────────────────
Write-Host "`n── Summary ──" -ForegroundColor Cyan
Write-Host "  Files generated : $filesWritten"
Write-Host "  Skills routed   : $totalSkills"
Write-Host "  Unresolved (internal/BMAD, excluded): $($unresolvedIds.Count)"

if ($unresolvedIds.Count -gt 0 -and $unresolvedIds.Count -le 20) {
    Write-Host "  Unresolved IDs:" -ForegroundColor DarkGray
    foreach ($id in $unresolvedIds) { Write-Host "    - $id" -ForegroundColor DarkGray }
} elseif ($unresolvedIds.Count -gt 20) {
    Write-Host "  (First 20 unresolved IDs shown)" -ForegroundColor DarkGray
    foreach ($id in $unresolvedIds[0..19]) { Write-Host "    - $id" -ForegroundColor DarkGray }
}

Write-Host ""
