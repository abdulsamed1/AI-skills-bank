#!/usr/bin/env pwsh
<#
.SYNOPSIS
Sync Hub System to all Tools

.DESCRIPTION
Syncs skill-manage/skills-aggregated/ to:
- ~/.gemini/antigravity/skills/
- ~/.claude/skills/
- ~/.agents/skills/
- ~/.cursor/skills/
- ~/.copilot/skills/
- ~/.config/opencode/skills/
- ~/.codeium/windsurf/skills/

Workspace-local targets are optional and disabled by default to avoid duplicate indexing conflicts.

Supports two strategies:
- Copy files (legacy behavior)
- Link directories (junction/symlink) to avoid redundant duplicates

(Skips individual skills if they're not hubs)
#>

param(
    [string]$Hubsrc = ".\skill-manage\skills-aggregated",
    [array]$TargetTools,
    [switch]$IncludeWorkspaceTargets,
    [switch]$PruneWorkspaceTargets,
    [switch]$IncludeGlobal,
    [ValidateSet("Auto", "Copy", "Junction", "SymbolicLink")]
    [string]$SyncMode = "Auto",
    [switch]$DisableGlobalAbsoluteRouting,
    [switch]$NoPrompt,
    [switch]$Force
)

# Resolve repo root and standard target sets
$PSScriptRootRef = $PSScriptRoot
if (-not $PSScriptRootRef) { $PSScriptRootRef = "." }
$candidateRoot = Get-Item (Join-Path $PSScriptRootRef "..")
if ($candidateRoot.Name -ieq "skill-manage") {
    $RepoRoot = (Get-Item (Join-Path $candidateRoot.FullName "..")).FullName
}
else {
    $RepoRoot = $candidateRoot.FullName
}

# Also resolve Hubsrc dynamically if it's the default
if ($Hubsrc -eq ".\skill-manage\skills-aggregated") {
    $Hubsrc = Join-Path $RepoRoot "skill-manage\skills-aggregated"
}

$WorkspaceTargets = @(
    (Join-Path $RepoRoot ".agent\skills"),
    (Join-Path $RepoRoot ".claude\skills"),
    (Join-Path $RepoRoot ".agents\skills"),
    (Join-Path $RepoRoot ".cursor\skills"),
    (Join-Path $RepoRoot ".gemini\skills"),
    (Join-Path $RepoRoot ".github\skills"),
    (Join-Path $RepoRoot ".opencode\skills"),
    (Join-Path $RepoRoot ".windsurf\skills")
)

$userHome = [Environment]::GetFolderPath("UserProfile")
$GlobalTargets = @(
    (Join-Path $userHome ".gemini\antigravity\skills"),
    (Join-Path $userHome ".claude\skills"),
    (Join-Path $userHome ".agents\skills"),
    (Join-Path $userHome ".cursor\skills"),
    (Join-Path $userHome ".gemini\skills"),
    (Join-Path $userHome ".copilot\skills"),
    (Join-Path $userHome ".config\opencode\skills"),
    (Join-Path $userHome ".codeium\windsurf\skills")
)

$TargetNameMap = @{
    (Join-Path $userHome ".gemini\antigravity\skills") = "Antigravity"
    (Join-Path $userHome ".claude\skills") = "Claude"
    (Join-Path $userHome ".agents\skills") = "Codex"
    (Join-Path $userHome ".cursor\skills") = "Cursor"
    (Join-Path $userHome ".gemini\skills") = "Gemini"
    (Join-Path $userHome ".copilot\skills") = "GitHub Copilot"
    (Join-Path $userHome ".config\opencode\skills") = "OpenCode"
    (Join-Path $userHome ".codeium\windsurf\skills") = "Windsurf"
    (Join-Path $RepoRoot ".agent\skills") = "Workspace Antigravity"
    (Join-Path $RepoRoot ".claude\skills") = "Workspace Claude"
    (Join-Path $RepoRoot ".agents\skills") = "Workspace Codex"
    (Join-Path $RepoRoot ".cursor\skills") = "Workspace Cursor"
    (Join-Path $RepoRoot ".gemini\skills") = "Workspace Gemini"
    (Join-Path $RepoRoot ".github\skills") = "Workspace GitHub"
    (Join-Path $RepoRoot ".opencode\skills") = "Workspace OpenCode"
    (Join-Path $RepoRoot ".windsurf\skills") = "Workspace Windsurf"
}

$TargetDetectionMap = @{
    (Join-Path $userHome ".gemini\antigravity\skills") = @((Join-Path $userHome ".gemini\antigravity"), (Join-Path $userHome ".gemini"))
    (Join-Path $userHome ".claude\skills") = @((Join-Path $userHome ".claude"))
    (Join-Path $userHome ".agents\skills") = @((Join-Path $userHome ".agents"))
    (Join-Path $userHome ".cursor\skills") = @((Join-Path $userHome ".cursor"))
    (Join-Path $userHome ".gemini\skills") = @((Join-Path $userHome ".gemini"))
    (Join-Path $userHome ".copilot\skills") = @((Join-Path $userHome ".copilot"))
    (Join-Path $userHome ".config\opencode\skills") = @((Join-Path $userHome ".config\opencode"))
    (Join-Path $userHome ".codeium\windsurf\skills") = @((Join-Path $userHome ".codeium\windsurf"), (Join-Path $userHome ".codeium"))
    (Join-Path $RepoRoot ".agent\skills") = @((Join-Path $RepoRoot ".agent\skills"), (Join-Path $RepoRoot ".agent\settings.json"), (Join-Path $RepoRoot ".agent\config.json"), (Join-Path $RepoRoot ".agent\README.md"))
    (Join-Path $RepoRoot ".claude\skills") = @((Join-Path $RepoRoot ".claude\skills"), (Join-Path $RepoRoot ".claude\settings.json"), (Join-Path $RepoRoot "CLAUDE.md"))
    (Join-Path $RepoRoot ".agents\skills") = @((Join-Path $RepoRoot ".agents\skills"), (Join-Path $RepoRoot ".agents\settings.json"), (Join-Path $RepoRoot ".agents\config.json"), (Join-Path $RepoRoot ".agents\README.md"))
    (Join-Path $RepoRoot ".cursor\skills") = @((Join-Path $RepoRoot ".cursor\skills"), (Join-Path $RepoRoot ".cursor\mcp.json"), (Join-Path $RepoRoot ".cursor\settings.json"))
    (Join-Path $RepoRoot ".gemini\skills") = @((Join-Path $RepoRoot ".gemini\skills"), (Join-Path $RepoRoot ".gemini\settings.json"), (Join-Path $RepoRoot ".gemini\antigravity"))
    (Join-Path $RepoRoot ".github\skills") = @((Join-Path $RepoRoot ".github\skills"), (Join-Path $RepoRoot ".github\copilot-instructions.md"), (Join-Path $RepoRoot ".github\workflows"))
    (Join-Path $RepoRoot ".opencode\skills") = @((Join-Path $RepoRoot ".opencode\skills"), (Join-Path $RepoRoot ".opencode\config.json"))
    (Join-Path $RepoRoot ".windsurf\skills") = @((Join-Path $RepoRoot ".windsurf\skills"), (Join-Path $RepoRoot ".windsurf\settings.json"), (Join-Path $RepoRoot ".windsurf\mcp.json"))
}

$ErrorActionPreference = "Stop"

function Write-Log {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "HH:mm:ss"
    Write-Host "[$timestamp] [$Level] $Message"
}

function New-DirectoryLink {
    param(
        [string]$LinkPath,
        [string]$TargetPath,
        [ValidateSet("Junction", "SymbolicLink")]
        [string]$LinkType
    )

    $linkParent = Split-Path -Path $LinkPath -Parent
    if (-not (Test-Path $linkParent)) {
        New-Item -ItemType Directory -Path $linkParent -Force | Out-Null
    }

    if ($LinkType -eq "Junction") {
        try {
            New-Item -ItemType Junction -Path $LinkPath -Target $TargetPath -ErrorAction Stop | Out-Null
            return
        } catch {
            $null = cmd /c "mklink /J \"$LinkPath\" \"$TargetPath\"" 2>&1
            if ($LASTEXITCODE -ne 0) {
                throw "Unable to create junction from '$LinkPath' to '$TargetPath'."
            }
        }
    } else {
        New-Item -ItemType SymbolicLink -Path $LinkPath -Target $TargetPath -ErrorAction Stop | Out-Null
    }
}

function Write-FileUtf8NoBom {
    param(
        [string]$Path,
        [string]$Content
    )

    $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($Path, $Content, $utf8NoBom)
}

function Convert-RoutingCsvToAbsolute {
    param(
        [string]$TargetRoot,
        [string]$SkillManageRoot
    )

    $routingFiles = Get-ChildItem -Path $TargetRoot -Recurse -Filter "routing.csv" -File -ErrorAction SilentlyContinue
    $updatedFiles = 0

    foreach ($routingFile in $routingFiles) {
        $rows = Import-Csv -Path $routingFile.FullName
        if (-not $rows -or $rows.Count -eq 0) { continue }

        $changed = $false
        foreach ($row in $rows) {
            $srcPath = [string]$row.src_path
            if ([string]::IsNullOrWhiteSpace($srcPath)) { continue }
            if ($srcPath -match '^[A-Za-z]:/') { continue }
            if ($srcPath -notmatch '^lib/') { continue }

            $absolute = Join-Path $SkillManageRoot ($srcPath -replace '/', '\\')
            $normalized = ($absolute -replace '\\', '/') -replace '/+', '/'
            if ($normalized -match '^[A-Za-z]:/') {
                $normalized = $normalized.Substring(0, 3) + ($normalized.Substring(3) -replace '/+', '/')
            }
            $row.src_path = $normalized
            $changed = $true
        }

        if ($changed) {
            $content = ($rows | ConvertTo-Csv -NoTypeInformation) -join "`n"
            Write-FileUtf8NoBom -Path $routingFile.FullName -Content ($content + "`n")
            $updatedFiles++
        }
    }

    return $updatedFiles
}

function Read-ConsoleSingleSelect {
    param(
        [string]$Title,
        [array]$Options,
        [int]$DefaultIndex = 0
    )

    if ($Options.Count -eq 0) {
        return $null
    }

    $index = [Math]::Min([Math]::Max($DefaultIndex, 0), $Options.Count - 1)

    while ($true) {
        Clear-Host
        Write-Host $Title -ForegroundColor Cyan
        Write-Host "Use Up/Down arrows and Enter to select." -ForegroundColor DarkGray
        Write-Host ""

        for ($i = 0; $i -lt $Options.Count; $i++) {
            $prefix = if ($i -eq $index) { ">" } else { " " }
            Write-Host ("{0} {1}" -f $prefix, $Options[$i].Label)
        }

        $key = [Console]::ReadKey($true)
        switch ($key.Key) {
            "UpArrow" { $index = if ($index -le 0) { $Options.Count - 1 } else { $index - 1 } }
            "DownArrow" { $index = if ($index -ge ($Options.Count - 1)) { 0 } else { $index + 1 } }
            "Enter" { return $Options[$index] }
            "Escape" { return $null }
        }
    }
}

function Read-ConsoleMultiSelect {
    param(
        [string]$Title,
        [array]$Options,
        [bool]$DefaultSelected = $true
    )

    if ($Options.Count -eq 0) {
        return @()
    }

    $index = 0
    $selected = @{}
    for ($i = 0; $i -lt $Options.Count; $i++) {
        $selected[$i] = $DefaultSelected
    }

    while ($true) {
        Clear-Host
        Write-Host $Title -ForegroundColor Cyan
        Write-Host "Use Up/Down arrows, Space to toggle, Enter to confirm." -ForegroundColor DarkGray
        Write-Host "Press A to select all, N to clear all." -ForegroundColor DarkGray
        Write-Host ""

        for ($i = 0; $i -lt $Options.Count; $i++) {
            $cursor = if ($i -eq $index) { ">" } else { " " }
            $check = if ($selected[$i]) { "[x]" } else { "[ ]" }
            Write-Host ("{0} {1} {2}" -f $cursor, $check, $Options[$i].Label)
        }

        $key = [Console]::ReadKey($true)
        switch ($key.Key) {
            "UpArrow" { $index = if ($index -le 0) { $Options.Count - 1 } else { $index - 1 } }
            "DownArrow" { $index = if ($index -ge ($Options.Count - 1)) { 0 } else { $index + 1 } }
            "Spacebar" { $selected[$index] = -not $selected[$index] }
            "A" {
                for ($j = 0; $j -lt $Options.Count; $j++) { $selected[$j] = $true }
            }
            "N" {
                for ($j = 0; $j -lt $Options.Count; $j++) { $selected[$j] = $false }
            }
            "Enter" {
                $picked = @()
                for ($j = 0; $j -lt $Options.Count; $j++) {
                    if ($selected[$j]) {
                        $picked += $Options[$j]
                    }
                }

                if ($picked.Count -gt 0) {
                    return $picked
                }
            }
            "Escape" { return @() }
        }
    }
}

function Get-InteractiveSyncScope {
    param([switch]$AllowScopeSelection)

    if (-not $AllowScopeSelection) {
        return [PSCustomObject]@{ Scope = "keep"; Cancelled = $false }
    }

    $scopeOptions = @(
        [PSCustomObject]@{ Label = "Global only"; Value = "global" },
        [PSCustomObject]@{ Label = "Workspace local only"; Value = "workspace" },
        [PSCustomObject]@{ Label = "Both global + workspace"; Value = "both" },
        [PSCustomObject]@{ Label = "Cancel"; Value = "cancel" }
    )

    $selected = Read-ConsoleSingleSelect -Title "Select sync destination scope" -Options $scopeOptions -DefaultIndex 0
    if (-not $selected -or $selected.Value -eq "cancel") {
        return [PSCustomObject]@{ Scope = "cancel"; Cancelled = $true }
    }

    return [PSCustomObject]@{ Scope = [string]$selected.Value; Cancelled = $false }
}

function Get-ToolName {
    param([string]$TargetPath)

    if ($TargetNameMap.ContainsKey($TargetPath)) {
        return [string]$TargetNameMap[$TargetPath]
    }

    return [string](Split-Path (Split-Path $TargetPath -Parent) -Leaf)
}

function Test-TargetInstalled {
    param([string]$TargetPath)

    $isWorkspaceTarget = $TargetPath.StartsWith($RepoRoot, [System.StringComparison]::OrdinalIgnoreCase)

    if ($TargetDetectionMap.ContainsKey($TargetPath)) {
        foreach ($checkPath in @($TargetDetectionMap[$TargetPath])) {
            if (-not [string]::IsNullOrWhiteSpace($checkPath) -and (Test-Path $checkPath)) {
                return $true
            }
        }
    }

    if (Test-Path $TargetPath) {
        return $true
    }

    if ($isWorkspaceTarget) {
        return $false
    }

    $parent = Split-Path $TargetPath -Parent
    return (-not [string]::IsNullOrWhiteSpace($parent) -and (Test-Path $parent))
}

function Filter-InstalledTargets {
    param([array]$Targets)

    $installed = @()
    foreach ($target in @($Targets | Select-Object -Unique)) {
        if (Test-TargetInstalled -TargetPath $target) {
            $installed += $target
        }
    }

    return @($installed | Select-Object -Unique)
}

function Select-InstalledTargetsInteractive {
    param([array]$InstalledTargets)

    if ($InstalledTargets.Count -le 1) {
        return $InstalledTargets
    }

    $options = @()
    foreach ($target in $InstalledTargets) {
        $options += [PSCustomObject]@{
            Label = ("{0} -> {1}" -f (Get-ToolName -TargetPath $target), $target)
            Value = $target
        }
    }

    $picked = Read-ConsoleMultiSelect -Title "Detected installed tools" -Options $options -DefaultSelected $true
    if ($picked.Count -eq 0) {
        return @()
    }

    return @($picked | ForEach-Object { $_.Value } | Select-Object -Unique)
}

function Confirm-OrExit {
    param(
        [string]$Message,
        [switch]$Enabled
    )

    if (-not $Enabled) {
        return
    }

    $options = @(
        [PSCustomObject]@{ Label = "Yes"; Value = "yes" },
        [PSCustomObject]@{ Label = "No"; Value = "no" }
    )

    $confirmation = Read-ConsoleSingleSelect -Title $Message -Options $options -DefaultIndex 1
    if (-not $confirmation -or $confirmation.Value -ne "yes") {
        Write-Log "Cancelled by user." "WARN"
        exit 0
    }
}

function Resolve-TargetTools {
    param(
        [array]$InputTargetTools,
        [switch]$IncludeWorkspace,
        [switch]$IncludeGlobalTargets,
        [switch]$InteractivePrompt,
        [array]$GlobalTargetsSet,
        [array]$WorkspaceTargetsSet
    )

    $explicitTargetsProvided = $PSBoundParameters.ContainsKey("InputTargetTools") -and $InputTargetTools -and $InputTargetTools.Count -gt 0
    $targets = @()

    if ($explicitTargetsProvided) {
        $targets = @($InputTargetTools)
        if ($IncludeGlobalTargets) {
            $targets = @($targets + $GlobalTargetsSet | Select-Object -Unique)
        }
    }
    else {
        $scopeSelection = Get-InteractiveSyncScope -AllowScopeSelection:$InteractivePrompt
        if ($scopeSelection.Cancelled) {
            Write-Log "Cancelled by user before sync started." "WARN"
            exit 0
        }

        switch ($scopeSelection.Scope) {
            "workspace" { $targets = @($WorkspaceTargetsSet) }
            "both" { $targets = @($GlobalTargetsSet + $WorkspaceTargetsSet | Select-Object -Unique) }
            default {
                $targets = @($GlobalTargetsSet)
                if ($IncludeWorkspace) {
                    $targets = @($targets + $WorkspaceTargetsSet | Select-Object -Unique)
                }
            }
        }
    }

    $targets = @($targets | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Select-Object -Unique)
    $installedTargets = Filter-InstalledTargets -Targets $targets

    if ($installedTargets.Count -eq 0) {
        Write-Log "No installed tool targets were detected for the selected scope. Nothing to sync." "WARN"
        return @()
    }

    if ($InteractivePrompt -and -not $explicitTargetsProvided) {
        return Select-InstalledTargetsInteractive -InstalledTargets $installedTargets
    }

    return $installedTargets
}

$interactivePrompt = -not $NoPrompt -and [Environment]::UserInteractive
$TargetTools = Resolve-TargetTools `
    -InputTargetTools $TargetTools `
    -IncludeWorkspace:$IncludeWorkspaceTargets `
    -IncludeGlobalTargets:$IncludeGlobal `
    -InteractivePrompt:$interactivePrompt `
    -GlobalTargetsSet $GlobalTargets `
    -WorkspaceTargetsSet $WorkspaceTargets

if ($TargetTools.Count -eq 0) {
    Write-Log "No valid targets resolved. Nothing to do." "WARN"
    exit 0
}

function Ensure-MainHubRouters {
    param(
        [string]$TargetPath,
        [array]$HubRelativePaths
    )

    $mainHubs = @{}
    foreach ($relativePath in $HubRelativePaths) {
        if ([string]::IsNullOrWhiteSpace($relativePath)) { continue }
        $normalized = ($relativePath -replace "\\", "/")
        $parts = $normalized.Split("/")
        if ($parts.Count -gt 0 -and -not [string]::IsNullOrWhiteSpace($parts[0])) {
            $mainHubs[$parts[0]] = $true
        }
    }

    foreach ($mainHub in ($mainHubs.Keys | Sort-Object)) {
        $mainHubPath = Join-Path $TargetPath $mainHub
        if (-not (Test-Path $mainHubPath)) {
            New-Item -ItemType Directory -Path $mainHubPath -Force | Out-Null
        }

        $skillPath = Join-Path $mainHubPath "SKILL.md"
        $skillContent = @"
---
name: $mainHub
description: Main router for the $mainHub hub. For story/epic requests, route to multiple sub-hubs when needed.
---

1. List available sub-folders in this hub.
2. For simple requests: choose the single best sub-hub.
3. For story/epic or multi-part requests: choose all relevant sub-hubs and process them sequentially.
4. In each chosen sub-hub: open SKILL.md, read routing.csv, select 1-2 skills per sub-problem.
5. Merge selected skills into one implementation plan and avoid duplicate/overlapping skills.
"@
        Write-FileUtf8NoBom -Path $skillPath -Content $skillContent
    }
}

function Sync-Hub-To-Tool {
    param(
        [string]$HubRelativePath,
        [string]$HubPath,
        [string]$TargetPath,
        [string]$ModeForTarget
    )
    
    $targetHubPath = Join-Path $TargetPath $HubRelativePath
    
    if (Test-Path $targetHubPath) {
        $isBroken = $false
        try {
            $null = Get-ChildItem -Path $targetHubPath -ErrorAction Stop
        } catch {
            $isBroken = $true
        }

        if ($Force) {
            Remove-Item -Path $targetHubPath -Recurse -Force | Out-Null
        } else {
            if ($isBroken) {
                Write-Log "BROKEN JUNCTION DETECTED: $HubRelativePath. Please re-run with -Force to repair it!" "ERROR"
            } else {
                Write-Log "Hub already exists: $HubRelativePath (use -Force to overwrite)" "WARN"
            }
            return [PSCustomObject]@{ Success = $false; Mode = "Skipped" }
        }
    }

    $linkOrCopyMode = $ModeForTarget
    if ($ModeForTarget -eq "Auto") {
        $linkOrCopyMode = "Junction"
    }

    if ($linkOrCopyMode -eq "Copy") {
        New-Item -ItemType Directory -Path $targetHubPath -Force | Out-Null
        Copy-Item -Path (Join-Path $HubPath "*") -Destination $targetHubPath -Recurse -Force
        return [PSCustomObject]@{ Success = $true; Mode = "Copy" }
    }

    try {
        New-DirectoryLink -LinkPath $targetHubPath -TargetPath $HubPath -LinkType $linkOrCopyMode
        return [PSCustomObject]@{ Success = $true; Mode = $linkOrCopyMode }
    } catch {
        if ($SyncMode -ne "Auto") {
            throw
        }

        Write-Log "Link mode failed for $HubRelativePath. Falling back to Copy. Reason: $($_.Exception.Message)" "WARN"
        New-Item -ItemType Directory -Path $targetHubPath -Force | Out-Null
        Copy-Item -Path (Join-Path $HubPath "*") -Destination $targetHubPath -Recurse -Force
        return [PSCustomObject]@{ Success = $true; Mode = "Copy" }
    }
}

# Main execution
Write-Log "Starting Hub Sync to Tools..."
Write-Log "Sync mode: $SyncMode"
if (-not $IncludeWorkspaceTargets) {
    Write-Log "Workspace targets: disabled (global-only policy)"
}

if ($interactivePrompt) {
    Write-Host ""
    Write-Host "Targets selected for sync:" -ForegroundColor Cyan
    foreach ($target in $TargetTools) {
        Write-Host "  - $target"
    }
    Confirm-OrExit -Message "Proceed with sync to these targets?" -Enabled:$true
}

# Remove workspace-local targets only when explicitly requested.
if ($PruneWorkspaceTargets) {
    Confirm-OrExit -Message "Prune workspace-local targets? This removes directories under the repository" -Enabled:$interactivePrompt
    foreach ($workspaceTarget in $WorkspaceTargets) {
        if (Test-Path $workspaceTarget) {
            Remove-Item -Path $workspaceTarget -Recurse -Force
            Write-Log "Pruned workspace target: $workspaceTarget"
        }
    }
}

# Verify hub src exists
if (-not (Test-Path $Hubsrc)) {
    Write-Log "ERROR: Hub src not found at $Hubsrc" "ERROR"
    exit 1
}

# Find all hub directories that actually contain a SKILL.md file (supports nested sub-hubs)
$skillFiles = Get-ChildItem -Path $Hubsrc -Filter "SKILL.md" -Recurse -File
$hubs = @()
$resolvedHubsrc = (Resolve-Path -LiteralPath $Hubsrc).Path
foreach ($skillFile in $skillFiles) {
    $hubDir = Split-Path -Path $skillFile.FullName -Parent
    $resolvedHubDir = (Resolve-Path -LiteralPath $hubDir).Path
    if (-not $resolvedHubDir.StartsWith($resolvedHubsrc, [System.StringComparison]::OrdinalIgnoreCase)) {
        Write-Log "Skipping unsafe hub path outside src root: $resolvedHubDir" "WARN"
        continue
    }

    $relativePath = $resolvedHubDir.Substring($resolvedHubsrc.Length).TrimStart('\', '/')
    if (-not [string]::IsNullOrWhiteSpace($relativePath)) {
        $hubs += [PSCustomObject]@{
            RelativePath = $relativePath
            FullPath = $resolvedHubDir
        }
    }
}

$hubs = $hubs | Sort-Object RelativePath -Unique

Write-Log "Found $($hubs.Count) hubs to sync"

# Sync to each tool
foreach ($toolPath in $TargetTools) {
    if (-not (Test-Path $toolPath)) {
        Write-Log "Creating tool directory: $toolPath"
        New-Item -ItemType Directory -Path $toolPath -Force | Out-Null
    }
    
    $toolName = Split-Path $toolPath -Parent | Split-Path -Leaf
    Write-Log ""
    Write-Log "Syncing to: $toolName ($toolPath)"

    $isGlobalTarget = $GlobalTargets -contains $toolPath
    $rewriteAbsoluteForThisTarget = $isGlobalTarget -and (-not $DisableGlobalAbsoluteRouting)
    $modeForTarget = if ($rewriteAbsoluteForThisTarget) { "Copy" } else { $SyncMode }
    if ($rewriteAbsoluteForThisTarget) {
        Write-Log "  Using Copy mode for global absolute routing rewrite"
    }
    
    $syncedCount = 0
    $modeCounts = @{}
    foreach ($hub in $hubs) {
        $syncResult = Sync-Hub-To-Tool -HubRelativePath $hub.RelativePath -HubPath $hub.FullPath -TargetPath $toolPath -ModeForTarget $modeForTarget
        if ($syncResult.Success) {
            if (-not $modeCounts.ContainsKey($syncResult.Mode)) {
                $modeCounts[$syncResult.Mode] = 0
            }
            $modeCounts[$syncResult.Mode]++
            Write-Log "  ✓ $($hub.RelativePath) [$($syncResult.Mode)]"
            $syncedCount++
        }
    }
    
    Write-Log "  Synced: $syncedCount/$($hubs.Count) hubs"
    if ($modeCounts.Count -gt 0) {
        $modeSummary = ($modeCounts.Keys | Sort-Object | ForEach-Object { "$_=$($modeCounts[$_])" }) -join ", "
        Write-Log "  Modes: $modeSummary"
    }

    Ensure-MainHubRouters -TargetPath $toolPath -HubRelativePaths ($hubs | ForEach-Object { $_.RelativePath })

    if ($rewriteAbsoluteForThisTarget) {
        $updated = Convert-RoutingCsvToAbsolute -TargetRoot $toolPath -SkillManageRoot (Join-Path $RepoRoot "skill-manage")
        Write-Log "  routing.csv absolute rewrite: $updated file(s) updated"
    }

    Write-Log "  Main-hub routers ensured"
}

# Copy master catalog and lock file
Write-Log ""
Write-Log "Copying catalog and lock files..."

$catalogSrc = Join-Path $Hubsrc "master-catalog.json"
$lockSrc = Join-Path $Hubsrc ".skill-lock.json"

foreach ($toolPath in $TargetTools) {
    if (Test-Path $catalogSrc) {
        Copy-Item -Path $catalogSrc -Destination (Join-Path $toolPath "..") -Force | Out-Null
    }
    if (Test-Path $lockSrc) {
        Copy-Item -Path $lockSrc -Destination (Join-Path $toolPath "..") -Force | Out-Null
    }
}

Write-Log "✓ Sync COMPLETE"
Write-Log ""
Write-Log "Summary:"
Write-Log "  Hubs synced: $($hubs.Count)"
Write-Log "  Targets: $($TargetTools.Count)"
Write-Log ""
Write-Log "Synced to:"
foreach ($toolPath in $TargetTools) {
    Write-Log "  ✓ $toolPath"
}
