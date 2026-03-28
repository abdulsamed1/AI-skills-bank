<#
.SYNOPSIS
    Compatibility entrypoint for CSV routing generation.

.DESCRIPTION
    Invokes generate-routing-tsv.ps1, which now emits routing.csv files.
#>

[CmdletBinding()]
param(
    [switch]$DryRun
)

$scriptPath = Join-Path $PSScriptRoot "generate-routing-tsv.ps1"
& $scriptPath @PSBoundParameters
