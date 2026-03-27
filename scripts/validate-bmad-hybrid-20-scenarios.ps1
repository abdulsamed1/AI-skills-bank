param(
    [string] $ManifestCsv = ".\AI-skills-bank\hub-manifests.csv",
    [string] $SkillsRoot = ".\AI-skills-bank\skills-aggregated",
    [string] $ReportPath = ".\AI-skills-bank\skills-aggregated\VALIDATION-20-SCENARIOS.md",
    [switch] $NoPrompt
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$interactivePrompt = -not $NoPrompt -and [Environment]::UserInteractive

function Confirm-OrExit {
    param([string] $Message)

    if (-not $interactivePrompt) {
        return
    }

    $confirmation = (Read-Host "$Message [y/N]").Trim().ToLowerInvariant()
    if ($confirmation -ne "y" -and $confirmation -ne "yes") {
        Write-Warning "Cancelled by user before report write."
        exit 0
    }
}

$results = New-Object System.Collections.Generic.List[psobject]

function Add-Result {
    param(
        [string] $Id,
        [string] $Scenario,
        [bool] $Passed,
        [string] $Details
    )

    $results.Add([PSCustomObject]@{
            id = $Id
            scenario = $Scenario
            passed = $Passed
            details = $Details
        }) | Out-Null
}

$root = (Get-Location).Path
$manifestPath = Join-Path $root $ManifestCsv
$skillsRootPath = Join-Path $root $SkillsRoot
$reportFilePath = Join-Path $root $ReportPath

# S01-S03: manifest existence and size
$manifestExists = Test-Path $manifestPath
Add-Result -Id "S01" -Scenario "Manifest file exists" -Passed $manifestExists -Details $manifestPath

$rows = @()
if ($manifestExists) {
    $rows = @(Import-Csv $manifestPath)
}

Add-Result -Id "S02" -Scenario "Manifest has at least 1 row" -Passed ($rows.Count -gt 0) -Details "rows=$($rows.Count)"
Add-Result -Id "S03" -Scenario "Manifest has at least 1000 rows" -Passed ($rows.Count -ge 1000) -Details "rows=$($rows.Count)"

# S04: unique triplets
$triplets = @($rows | ForEach-Object { "$($_.hub)|$($_.sub_hub)|$($_.skill_id)" })
$uniqueTriplets = @($triplets | Sort-Object -Unique)
Add-Result -Id "S04" -Scenario "Unique (hub,sub_hub,skill_id) triplets" -Passed ($triplets.Count -eq $uniqueTriplets.Count) -Details "total=$($triplets.Count), unique=$($uniqueTriplets.Count)"

# S05-S08: column validity
$missingSkillId = @($rows | Where-Object { [string]::IsNullOrWhiteSpace($_.skill_id) }).Count
Add-Result -Id "S05" -Scenario "No empty skill_id values" -Passed ($missingSkillId -eq 0) -Details "missing=$missingSkillId"

$invalidScore = @($rows | Where-Object {
        $score = 0
        if (-not [int]::TryParse([string]$_.match_score, [ref]$score)) { return $true }
        return ($score -lt 1 -or $score -gt 100)
    }).Count
Add-Result -Id "S06" -Scenario "match_score is integer between 1 and 100" -Passed ($invalidScore -eq 0) -Details "invalid=$invalidScore"

$invalidPhase = @($rows | Where-Object {
        $phase = 0
        if (-not [int]::TryParse([string]$_.phase, [ref]$phase)) { return $true }
        return ($phase -lt 1 -or $phase -gt 4)
    }).Count
Add-Result -Id "S07" -Scenario "phase is integer between 1 and 4" -Passed ($invalidPhase -eq 0) -Details "invalid=$invalidPhase"

$invalidRequired = @($rows | Where-Object {
        $v = ([string]$_.required).ToLowerInvariant()
        return ($v -ne "true" -and $v -ne "false")
    }).Count
Add-Result -Id "S08" -Scenario "required values are true/false only" -Passed ($invalidRequired -eq 0) -Details "invalid=$invalidRequired"

# S09-S10: coverage
$hubs = @($rows | Select-Object -ExpandProperty hub -Unique)
$subHubGroups = @($rows | Group-Object hub, sub_hub)
Add-Result -Id "S09" -Scenario "Main hubs count is at least 11" -Passed ($hubs.Count -ge 11) -Details "hubs=$($hubs.Count)"
Add-Result -Id "S10" -Scenario "Sub-hub groups count is 27" -Passed ($subHubGroups.Count -eq 27) -Details "subhubs=$($subHubGroups.Count)"

# Build expected skill file map from manifest groups
$expected = New-Object System.Collections.Generic.HashSet[string]
foreach ($g in $subHubGroups) {
    $parts = $g.Name -split ",\s*"
    $key = "$($parts[0])/$($parts[1])"
    [void]$expected.Add($key)
}

$skillFiles = @(Get-ChildItem -Path $skillsRootPath -Recurse -File -Filter "SKILL.md" | Where-Object { $_.FullName -match "skills-aggregated\\[^\\]+\\[^\\]+\\SKILL\.md$" })
$fileMap = @{}
foreach ($f in $skillFiles) {
    $subHub = Split-Path -Leaf (Split-Path -Parent $f.FullName)
    $hub = Split-Path -Leaf (Split-Path -Parent (Split-Path -Parent $f.FullName))
    $key = "$hub/$subHub"
    $fileMap[$key] = $f.FullName
}

$missingSkillFiles = @($expected | Where-Object { -not $fileMap.ContainsKey($_) })
Add-Result -Id "S11" -Scenario "Every manifest sub-hub has SKILL.md" -Passed ($missingSkillFiles.Count -eq 0) -Details "missing=$($missingSkillFiles.Count)"

# S12-S20: content checks
$frontmatterMissing = 0
$nameMismatch = 0
$criticalMissing = 0
$quickMatcherMissing = 0
$tableRowsInvalid = 0
$selectionRulesMissing = 0
$dependencyRulesMissing = 0
$hallucinationRuleMissing = 0
$thresholdRuleMissing = 0

foreach ($k in $expected) {
    if (-not $fileMap.ContainsKey($k)) { continue }

    $path = $fileMap[$k]
    $content = Get-Content -Path $path -Raw

    if ($content -notmatch "(?s)^---\s*\r?\n.*?\r?\n---") {
        $frontmatterMissing += 1
        continue
    }

    $folderName = ($k -split "/")[1]
    if ($content -notmatch ("(?m)^name:\s*" + [regex]::Escape($folderName) + "\s*$")) {
        $nameMismatch += 1
    }

    if ($content -notmatch "(?m)^##\s+Critical Instructions") { $criticalMissing += 1 }
    if ($content -notmatch "(?m)^##\s+Quick Intent Matcher") { $quickMatcherMissing += 1 }

    $expectedRows = @($rows | Where-Object { "{0}/{1}" -f $_.hub, $_.sub_hub -eq $k }).Count
    $requiredTableRows = [Math]::Min(5, $expectedRows)
    $tableRows = [regex]::Matches($content, "(?m)^\|\s*[^\n]*\|\s*\[[A]\d+\]\s*\|\s*$").Count
    if ($tableRows -lt $requiredTableRows) { $tableRowsInvalid += 1 }

    if ($content -notmatch "(?m)^##\s+Selection Rules") { $selectionRulesMissing += 1 }
    if ($content -notmatch "(?m)^##\s+Dependency Rules") { $dependencyRulesMissing += 1 }
    if ($content -notmatch "Never invent a skill_id") { $hallucinationRuleMissing += 1 }
    if ($content -notmatch "score >= 10") { $thresholdRuleMissing += 1 }
}

Add-Result -Id "S12" -Scenario "All generated SKILL files contain frontmatter" -Passed ($frontmatterMissing -eq 0) -Details "missing=$frontmatterMissing"
Add-Result -Id "S13" -Scenario "Frontmatter name matches folder name" -Passed ($nameMismatch -eq 0) -Details "mismatch=$nameMismatch"
Add-Result -Id "S14" -Scenario "Contains Critical Instructions section" -Passed ($criticalMissing -eq 0) -Details "missing=$criticalMissing"
Add-Result -Id "S15" -Scenario "Contains Quick Intent Matcher section" -Passed ($quickMatcherMissing -eq 0) -Details "missing=$quickMatcherMissing"
Add-Result -Id "S16" -Scenario "Quick Intent table row count is valid for each sub-hub" -Passed ($tableRowsInvalid -eq 0) -Details "invalid_files=$tableRowsInvalid"
Add-Result -Id "S17" -Scenario "Contains Selection Rules section" -Passed ($selectionRulesMissing -eq 0) -Details "missing=$selectionRulesMissing"
Add-Result -Id "S18" -Scenario "Contains Dependency Rules section" -Passed ($dependencyRulesMissing -eq 0) -Details "missing=$dependencyRulesMissing"
Add-Result -Id "S19" -Scenario "Contains anti-hallucination rule text" -Passed ($hallucinationRuleMissing -eq 0) -Details "missing=$hallucinationRuleMissing"
Add-Result -Id "S20" -Scenario "Contains score threshold rule (>=10)" -Passed ($thresholdRuleMissing -eq 0) -Details "missing=$thresholdRuleMissing"

$passedCount = @($results | Where-Object { $_.passed }).Count
$totalCount = $results.Count
$failedCount = $totalCount - $passedCount

Write-Host "Validation Summary: $passedCount/$totalCount passed" -ForegroundColor Cyan
foreach ($r in $results) {
    $status = if ($r.passed) { "PASS" } else { "FAIL" }
    $color = if ($r.passed) { "Green" } else { "Red" }
    Write-Host ("{0} {1} - {2} ({3})" -f $status, $r.id, $r.scenario, $r.details) -ForegroundColor $color
}

$reportDir = Split-Path -Parent $reportFilePath
if (-not (Test-Path $reportDir)) {
    New-Item -ItemType Directory -Path $reportDir | Out-Null
}

Confirm-OrExit -Message "Proceed with writing validation report to '$reportFilePath'?"

$reportLines = New-Object System.Collections.Generic.List[string]
$reportLines.Add("# BMAD Hybrid Validation Report (20 Scenarios)") | Out-Null
$reportLines.Add("") | Out-Null
$reportLines.Add(("Generated: {0}" -f (Get-Date).ToString("yyyy-MM-dd HH:mm:ssK"))) | Out-Null
$reportLines.Add(("Summary: {0}/{1} passed" -f $passedCount, $totalCount)) | Out-Null
$reportLines.Add("") | Out-Null
$reportLines.Add("| ID | Scenario | Result | Details |") | Out-Null
$reportLines.Add("|---|---|---|---|") | Out-Null
foreach ($r in $results) {
    $result = if ($r.passed) { "PASS" } else { "FAIL" }
    $reportLines.Add(("| {0} | {1} | {2} | {3} |" -f $r.id, $r.scenario, $result, $r.details.Replace("|", "\\|"))) | Out-Null
}

Set-Content -Path $reportFilePath -Value ($reportLines -join "`r`n") -Encoding UTF8
Write-Host "Report written: $reportFilePath" -ForegroundColor Cyan

if ($failedCount -gt 0) {
    exit 1
}
