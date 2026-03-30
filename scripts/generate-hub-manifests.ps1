param(
    [string] $SkillsAggregatedDir = ".\skill-manage\skills-aggregated",
    [string] $OutputCsv = ".\skill-manage\hub-manifests.csv",
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
        Write-Warning "Cancelled by user before write operations started."
        exit 0
    }
}

function Resolve-Phase {
    param([int] $Score)

    if ($Score -ge 80) { return 1 }
    if ($Score -ge 20) { return 2 }
    if ($Score -ge 10) { return 3 }
    return 4
}

function Resolve-Required {
    param([int] $Score)

    return ($Score -ge 80)
}

function To-DisplayName {
    param([string] $SkillId)

    if ([string]::IsNullOrWhiteSpace($SkillId)) {
        return ""
    }

    $parts = @($SkillId -split "-" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | ForEach-Object {
            if ($_.Length -eq 1) { $_.ToUpperInvariant() }
            else { $_.Substring(0, 1).ToUpperInvariant() + $_.Substring(1) }
        })

    return ($parts -join " ")
}

$repoRoot = (Get-Location).Path
$skillsRoot = Resolve-Path -Path (Join-Path $repoRoot $SkillsAggregatedDir)
$skillsRootPath = $skillsRoot.Path

$allRows = New-Object System.Collections.Generic.List[psobject]
$subHubDirs = Get-ChildItem -Path $skillsRootPath -Directory | ForEach-Object {
    $mainHub = $_
    Get-ChildItem -Path $mainHub.FullName -Directory | ForEach-Object {
        [PSCustomObject]@{
            MainHub = $mainHub.Name
            SubHub = $_.Name
            Path = $_.FullName
        }
    }
}

foreach ($subHub in $subHubDirs) {
    $indexPath = Join-Path $subHub.Path "skills-index.json"
    $catalogPath = Join-Path $subHub.Path "routing.csv"

    if (-not (Test-Path $indexPath) -or -not (Test-Path $catalogPath)) {
        Write-Warning "Skipping $($subHub.MainHub)/$($subHub.SubHub): missing index or catalog"
        continue
    }

    $indexItems = Get-Content -Path $indexPath -Raw | ConvertFrom-Json
    $indexById = @{}
    foreach ($item in $indexItems) {
        if ($null -ne $item.id) {
            $indexById[[string]$item.id] = $item
        }
    }

    $catalogItems = Import-Csv -Path $catalogPath
    foreach ($entry in $catalogItems) {
        $skillId = [string]$entry.id
        if ([string]::IsNullOrWhiteSpace($skillId)) {
            continue
        }

        $indexItem = $null
        if ($indexById.ContainsKey($skillId)) {
            $indexItem = $indexById[$skillId]
        }

        $score = 0
        if ($null -ne $indexItem -and $null -ne $indexItem.match_score) {
            $score = [int]$indexItem.match_score
        }
        elseif ($null -ne $entry.match_score) {
            $score = [int]$entry.match_score
        }

        $triggers = @()
        if ($null -ne $indexItem -and $null -ne $indexItem.triggers) {
            $triggers = @($indexItem.triggers)
        }
        elseif ($null -ne $entry.triggers) {
            $triggers = @(([string]$entry.triggers) -split ';' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
        }

        $phase = Resolve-Phase -Score $score
        $required = Resolve-Required -Score $score

        $allRows.Add([PSCustomObject]@{
            module = "BMad"
            hub = $subHub.MainHub
            sub_hub = $subHub.SubHub
            skill_id = $skillId
            display_name = (To-DisplayName -SkillId $skillId)
            description = [string]$(if ($null -ne $entry.description) { $entry.description } else { "" })
            triggers = (($triggers | ForEach-Object { [string]$_ }) -join ";")
            match_score = $score
            phase = $phase
            after = ""
            before = ""
            required = $required.ToString().ToLowerInvariant()
            action = "invoke"
            output_location = "outputs/$($subHub.MainHub)/$($subHub.SubHub)"
            outputs = "$skillId-*"
        })
    }
}

$sorted = $allRows | Sort-Object hub, sub_hub, @{ Expression = "match_score"; Descending = $true }, skill_id

$outputPath = Join-Path $repoRoot $OutputCsv
$outputDir = Split-Path -Parent $outputPath
Confirm-OrExit -Message "Proceed with writing hub manifest CSV to '$outputPath'?"
if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir | Out-Null
}

$sorted | Export-Csv -Path $outputPath -NoTypeInformation -Encoding UTF8

Write-Host "Generated: $outputPath" -ForegroundColor Green
Write-Host "Rows: $($sorted.Count)" -ForegroundColor Cyan
Write-Host "Sub-hubs scanned: $($subHubDirs.Count)" -ForegroundColor Cyan
