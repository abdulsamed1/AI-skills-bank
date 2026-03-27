param(
    [string] $Hub,
    [string] $SubHub,
    [string] $ManifestCsv = ".\AI-skills-bank\hub-manifests.csv",
    [string] $SkillsRoot = ".\AI-skills-bank\skills-aggregated",
    [ValidateRange(3, 10)]
    [int] $TopN = 5,
    [switch] $All,
    [switch] $DryRun,
    [switch] $NoPrompt
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$interactivePrompt = -not $NoPrompt -and [Environment]::UserInteractive

function Confirm-OrExit {
    param([string] $Message)

    if (-not $interactivePrompt -or $DryRun) {
        return
    }

    $confirmation = (Read-Host "$Message [y/N]").Trim().ToLowerInvariant()
    if ($confirmation -ne "y" -and $confirmation -ne "yes") {
        Write-Warning "Cancelled by user before write operations started."
        exit 0
    }
}

function Get-IntentLabel {
    param([string] $Triggers, [string] $SkillId)

    if (-not [string]::IsNullOrWhiteSpace($Triggers)) {
        $parts = @($Triggers -split ";" | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne "" })
        if ($parts.Count -ge 2) { return "$($parts[0]) + $($parts[1])" }
        if ($parts.Count -eq 1) { return $parts[0] }
    }

    return $SkillId
}

function Get-TopTriggerSummary {
    param([System.Collections.IEnumerable] $Rows, [int] $Take = 10)

    $counts = @{}
    foreach ($row in $Rows) {
        if ([string]::IsNullOrWhiteSpace($row.triggers)) { continue }
        $tokens = @($row.triggers -split ";" | ForEach-Object { $_.Trim().ToLowerInvariant() } | Where-Object { $_ -ne "" })
        foreach ($token in $tokens) {
            if ($counts.ContainsKey($token)) { $counts[$token] += 1 }
            else { $counts[$token] = 1 }
        }
    }

    $sorted = $counts.GetEnumerator() |
        Sort-Object -Property @{ Expression = "Value"; Descending = $true }, @{ Expression = "Name"; Descending = $false } |
        Select-Object -First $Take |
        ForEach-Object { $_.Name }

    return @($sorted)
}

function Build-SkillFileContent {
    param(
        [string] $HubName,
        [string] $SubHubName,
        [System.Collections.IEnumerable] $Rows,
        [int] $TopCount
    )

    $rowsArray = @($Rows)
    $skillCount = $rowsArray.Count
    $topRows = @($rowsArray | Sort-Object @{ Expression = { [int]$_.match_score }; Descending = $true }, skill_id | Select-Object -First $TopCount)
    $topTriggers = @(Get-TopTriggerSummary -Rows $rowsArray -Take 10)

    $phase1Count = @($rowsArray | Where-Object { [int]$_.phase -eq 1 }).Count
    $phase2Count = @($rowsArray | Where-Object { [int]$_.phase -eq 2 }).Count
    $phase3Count = @($rowsArray | Where-Object { [int]$_.phase -eq 3 }).Count
    $phase4Count = @($rowsArray | Where-Object { [int]$_.phase -eq 4 }).Count

    $requiredCount = @($rowsArray | Where-Object { ([string]$_.required).ToLowerInvariant() -eq "true" }).Count
    $generatedAt = (Get-Date).ToString("yyyy-MM-dd HH:mm:ssK")

    $tableLines = New-Object System.Collections.Generic.List[string]
    $tableLines.Add("| User Intent | Skill ID | Score | Phase | Required | Depends On | Code |") | Out-Null
    $tableLines.Add("|---|---|---:|---:|---|---|---|") | Out-Null

    $index = 1
    foreach ($row in $topRows) {
        $intent = Get-IntentLabel -Triggers ([string]$row.triggers) -SkillId ([string]$row.skill_id)
        $dependsOn = if ([string]::IsNullOrWhiteSpace([string]$row.after)) { "none" } else { [string]$row.after }
        $required = ([string]$row.required).ToLowerInvariant()
        $code = "A$index"
        $tableLines.Add("| $intent | $($row.skill_id) | $($row.match_score) | $($row.phase) | $required | $dependsOn | [$code] |") | Out-Null
        $index += 1
    }

    $topTriggersText = if ($topTriggers.Count -gt 0) { $topTriggers -join ", " } else { "n/a" }

        $content = @"
---
name: $SubHubName
description: |
  Auto-generated router for $HubName/$SubHubName.

  Skills: $skillCount
  Required gates: $requiredCount
  Generated: $generatedAt
metadata:
    agent_party: '{project-root}/AI-skills-bank/hub-manifests.csv'
    bmad_compatible: true
    version: '1.0'
---

# $HubName / $SubHubName Skill Router

## Critical Instructions

1. Load hub-manifests.csv first.
2. Filter only rows where hub=$HubName and sub_hub=$SubHubName.
3. Match user intent against triggers exactly.
4. Never invent a skill_id.
5. Verify score, phase, and dependencies before selection.

## Hub Snapshot

- Total skills: $skillCount
- Required skills: $requiredCount
- Phase distribution: P1=$phase1Count, P2=$phase2Count, P3=$phase3Count, P4=$phase4Count
- Top triggers: $topTriggersText

## Quick Intent Matcher

$($tableLines -join "`r`n")

## Selection Rules

1. Select candidates with score >= 10.
2. If multiple candidates remain, sort by score descending.
3. If the best candidate has required=true and unmet dependency (after), block and explain the prerequisite.
4. If user intent is ambiguous, present top 3 candidates and ask user to choose.

## Dependency Rules

- after: prerequisite skill IDs that must be completed first.
- before: reverse dependency links for planning only.
- required=true: blocking gate for progression.

## Output Format

When proposing a skill, respond with:
- skill_id
- reason (trigger + score)
- phase
- required
- dependencies (after)
- next step

## Verification Checklist

- Skill exists in hub-manifests.csv
- Trigger overlap is explicit
- Score is >= 10
- Dependency gates are respected
- No hallucinated IDs
"@

    return $content
}

$root = (Get-Location).Path
$manifestPath = Join-Path $root $ManifestCsv
$skillsRootPath = Join-Path $root $SkillsRoot

if (-not (Test-Path $manifestPath)) {
    throw "Manifest file not found: $manifestPath"
}

if (-not (Test-Path $skillsRootPath)) {
    throw "Skills root not found: $skillsRootPath"
}

$allRows = Import-Csv $manifestPath
if ($allRows.Count -eq 0) {
    throw "Manifest CSV is empty: $manifestPath"
}

$targets = @()
if ($All) {
    $targets = @($allRows | Group-Object hub, sub_hub | ForEach-Object {
            $parts = $_.Name -split ",\s*"
            [PSCustomObject]@{ Hub = $parts[0]; SubHub = $parts[1] }
        })
}
else {
    if ([string]::IsNullOrWhiteSpace($Hub) -or [string]::IsNullOrWhiteSpace($SubHub)) {
        throw "Provide -Hub and -SubHub, or use -All"
    }
    $targets = @([PSCustomObject]@{ Hub = $Hub; SubHub = $SubHub })
}

Confirm-OrExit -Message "Proceed with generating/updating $($targets.Count) SKILL.md file(s) under '$skillsRootPath'?"

$written = 0
foreach ($target in $targets) {
    $rows = @($allRows | Where-Object { $_.hub -eq $target.Hub -and $_.sub_hub -eq $target.SubHub })
    if ($rows.Count -eq 0) {
        Write-Warning "No manifest rows found for $($target.Hub)/$($target.SubHub)"
        continue
    }

    $content = Build-SkillFileContent -HubName $target.Hub -SubHubName $target.SubHub -Rows $rows -TopCount $TopN
    $targetFile = Join-Path $skillsRootPath "$($target.Hub)\$($target.SubHub)\SKILL.md"

    if ($DryRun) {
        Write-Host "[dry-run] Would write: $targetFile" -ForegroundColor Yellow
        continue
    }

    Set-Content -Path $targetFile -Value $content -Encoding UTF8
    Write-Host "Updated: $targetFile" -ForegroundColor Green
    $written += 1
}

Write-Host "Done. Files written: $written" -ForegroundColor Cyan
