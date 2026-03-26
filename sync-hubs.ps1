#!/usr/bin/env pwsh
<#
.SYNOPSIS
Sync Hub System to all Tools — Copies hub-skills to .agent, .gemini, .github

.DESCRIPTION
Syncs AI-skills-bank/hub-skills/ to:
- .agent/skills/
- .gemini/skills/
- .github/skills/

(Skips individual skills if they're not hubs)
#>

param(
    [string]$HubSource = ".\AI-skills-bank\skills-aggregated",
    [array]$TargetTools,
    [switch]$Force
)

# If TargetTools not specified, use defaults
if (-not $TargetTools -or $TargetTools.Count -eq 0) {
    # Get current script folder as reference point
    $PSScriptRootRef = $PSScriptRoot
    if (-not $PSScriptRootRef) { $PSScriptRootRef = "." }
    $RepoRoot = Join-Path $PSScriptRootRef ".."
    
    # Also resolve HubSource dynamically if it's the default
    if ($HubSource -eq ".\AI-skills-bank\skills-aggregated") {
        $HubSource = Join-Path $RepoRoot "AI-skills-bank\skills-aggregated"
    }

    $TargetTools = @(
        (Join-Path $RepoRoot ".agent\skills"),
        (Join-Path $RepoRoot ".gemini\skills"),
        (Join-Path $RepoRoot ".github\skills")
    )
}

$ErrorActionPreference = "Stop"

function Write-Log {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "HH:mm:ss"
    Write-Host "[$timestamp] [$Level] $Message"
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
            Write-Log "Hub already exists: $HubName (use -Force to overwrite)" "WARN"
            return $false
        }
    }
    
    New-Item -ItemType Directory -Path $targetHubPath -Force | Out-Null

    # Copy all files for this hub/sub-hub (router + workflow + catalogs + metadata)
    Copy-Item -Path (Join-Path $HubPath "*") -Destination $targetHubPath -Recurse -Force
    
    return $true
}

# Main execution
Write-Log "Starting Hub Sync to Tools..."

# Verify hub source exists
if (-not (Test-Path $HubSource)) {
    Write-Log "ERROR: Hub source not found at $HubSource" "ERROR"
    exit 1
}

# Find all hub directories that actually contain a SKILL.md file (supports nested sub-hubs)
$skillFiles = Get-ChildItem -Path $HubSource -Filter "SKILL.md" -Recurse -File
$hubs = @()
foreach ($skillFile in $skillFiles) {
    $hubDir = Split-Path -Path $skillFile.FullName -Parent
    $relativePath = $hubDir.Substring((Resolve-Path $HubSource).Path.Length).TrimStart('\', '/')
    if (-not [string]::IsNullOrWhiteSpace($relativePath)) {
        $hubs += [PSCustomObject]@{
            RelativePath = $relativePath
            FullPath = $hubDir
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
    foreach ($hub in $hubs) {
        if (Sync-Hub-To-Tool -HubRelativePath $hub.RelativePath -HubPath $hub.FullPath -TargetPath $toolPath) {
            Write-Log "  ✓ $($hub.RelativePath)"
            $syncedCount++
        }
    }
    
    Write-Log "  Synced: $syncedCount/$($hubs.Count) hubs"
}

# Copy master catalog and lock file
Write-Log ""
Write-Log "Copying catalog and lock files..."

$catalogSrc = Join-Path $HubSource "master-catalog.json"
$lockSrc = Join-Path $HubSource ".skill-lock.json"

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
