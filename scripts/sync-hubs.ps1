#!/usr/bin/env pwsh
<#
.SYNOPSIS
Sync Hub System to all Tools

.DESCRIPTION
Syncs AI-skills-bank/skills-aggregated/ to:
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
    [string]$Hubsrc = ".\AI-skills-bank\skills-aggregated",
    [array]$TargetTools,
    [switch]$IncludeWorkspaceTargets,
    [switch]$PruneWorkspaceTargets,
    [switch]$IncludeGlobal,
    [ValidateSet("Auto", "Copy", "Junction", "SymbolicLink")]
    [string]$SyncMode = "Auto",
    [switch]$Force
)

# Resolve repo root and standard target sets
$PSScriptRootRef = $PSScriptRoot
if (-not $PSScriptRootRef) { $PSScriptRootRef = "." }
$candidateRoot = Get-Item (Join-Path $PSScriptRootRef "..")
if ($candidateRoot.Name -ieq "AI-skills-bank") {
    $RepoRoot = (Get-Item (Join-Path $candidateRoot.FullName "..")).FullName
}
else {
    $RepoRoot = $candidateRoot.FullName
}

# Also resolve Hubsrc dynamically if it's the default
if ($Hubsrc -eq ".\AI-skills-bank\skills-aggregated") {
    $Hubsrc = Join-Path $RepoRoot "AI-skills-bank\skills-aggregated"
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

# If TargetTools not specified, default to global-only targets.
if (-not $TargetTools -or $TargetTools.Count -eq 0) {
    $TargetTools = @($GlobalTargets)
    if ($IncludeWorkspaceTargets) {
        $TargetTools = @($TargetTools + $WorkspaceTargets | Select-Object -Unique)
    }
} elseif ($IncludeGlobal) {
    # Backward compatibility: add global targets to explicit custom targets.
    $TargetTools = @($TargetTools + $GlobalTargets | Select-Object -Unique)
}

$TargetTools = @($TargetTools | Where-Object { -not [string]::IsNullOrWhiteSpace($_) } | Select-Object -Unique)

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
        $workflowPath = Join-Path $mainHubPath "workflow.md"

        $skillContent = @"
---
name: $mainHub
description: Main router for the $mainHub hub. Choose a concrete sub-hub under this folder.
---

Read workflow.md, then route to the most relevant sub-hub SKILL.md in this folder.
"@

        $workflowContent = @"
# $mainHub Router Workflow

1. List available sub-folders in this hub.
2. Choose the most relevant sub-hub for the user request.
3. Open that sub-hub SKILL.md and follow its workflow.
"@

        Write-FileUtf8NoBom -Path $skillPath -Content $skillContent
        Write-FileUtf8NoBom -Path $workflowPath -Content $workflowContent
    }
}

function Sync-Hub-To-Tool {
    param(
        [string]$HubRelativePath,
        [string]$HubPath,
        [string]$TargetPath
    )
    
    $targetHubPath = Join-Path $TargetPath $HubRelativePath
    
    if (Test-Path $targetHubPath) {
        if ($Force) {
            Remove-Item -Path $targetHubPath -Recurse -Force | Out-Null
        } else {
            Write-Log "Hub already exists: $HubRelativePath (use -Force to overwrite)" "WARN"
            return [PSCustomObject]@{ Success = $false; Mode = "Skipped" }
        }
    }

    $linkOrCopyMode = $SyncMode
    if ($SyncMode -eq "Auto") {
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

# Remove workspace-local targets only when explicitly requested.
if ($PruneWorkspaceTargets) {
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
    
    $syncedCount = 0
    $modeCounts = @{}
    foreach ($hub in $hubs) {
        $syncResult = Sync-Hub-To-Tool -HubRelativePath $hub.RelativePath -HubPath $hub.FullPath -TargetPath $toolPath
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
