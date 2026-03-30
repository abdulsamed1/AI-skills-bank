<#
.SYNOPSIS
    Compatibility entrypoint for CSV routing generation.

.DESCRIPTION
    Invokes generate-routing-tsv.ps1, which now emits routing.csv files.
#>

[CmdletBinding()]
param(
    [switch]$DryRun,
    [ValidateSet('Auto', 'HubLocal', 'SourceDirect', 'SourceDirectStatic')]
    [string]$ToolProfile = 'Auto'
)

$scriptPath = Join-Path $PSScriptRoot "generate-routing-tsv.ps1"
$pathMode = 'SourceDirectRelative'
if ($ToolProfile -eq 'HubLocal') {
    $pathMode = 'HubLocal'
}
elseif ($ToolProfile -eq 'SourceDirect') {
    $pathMode = 'SourceDirectRelative'
}
elseif ($ToolProfile -eq 'SourceDirectStatic') {
    $pathMode = 'SourceDirectAbsolute'
}

& $scriptPath -DryRun:$DryRun -PathMode $pathMode
