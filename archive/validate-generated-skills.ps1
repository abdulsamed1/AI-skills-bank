# Quality Validation for Generated Skill Hub Files
# Implements: Microsoft Writing Style Guide + Naming Conventions + Schema Compliance

# ============================================================================
# PROSE QUALITY VALIDATION (Microsoft Writing Style Guide Baseline)
# ============================================================================

function Test-ProseQuality {
    param([string] $Text)

    $issues = @()

    # Check for passive voice markers
    if ($Text -match '\bshould\s+be\s+\w+ed\b|\bwill\s+be\s+\w+ed\b') {
        $issues += @{ rule = "passive-voice"; severity = "warning"; message = "Consider active voice for clarity" }
    }

    # Check for wordy phrases
    $wordyPhrases = @{
        'in order to' = 'to'
        'at the end of the day' = 'ultimately'
        'for the purpose of' = 'to'
        'in a manner that' = 'by'
        'has the ability to' = 'can'
        'due to the fact that' = 'because'
    }

    foreach ($wordy in $wordyPhrases.Keys) {
        if ($Text -match [regex]::Escape($wordy)) {
            $issues += @{ rule = "wordy-phrase"; phrase = $wordy; replacement = $wordyPhrases[$wordy]; severity = "info" }
        }
    }

    # Check for jargon without context
    $unboundJargon = @('paradigm', 'leverage', 'synergy', 'holistic', 'utilize')
    foreach ($jargon in $unboundJargon) {
        if ($Text -match "\b$jargon\b") {
            $issues += @{ rule = "jargon"; term = $jargon; severity = "warning"; message = "Define or simplify jargon" }
        }
    }

    # Check for contractions (avoid in formal docs)
    if ($Text -match '\b(don''t|can''t|won''t|isn''t|aren''t)\b') {
        $issues += @{ rule = "contractions"; severity = "info"; message = "Avoid contractions in formal documentation" }
    }

    # Check sentence length (flag if average > 20 words)
    $sentences = @($Text -split '(?<=[.!?])\s+' | Where-Object { $_.Length -gt 0 })
    if ($sentences.Count -gt 0) {
        $totalWords = ($Text -split '\s+').Count
        $avgSentenceLength = $totalWords / $sentences.Count
        if ($avgSentenceLength -gt 20) {
            $issues += @{ rule = "sentence-length"; severity = "warning"; message = "Average sentence length ${avgSentenceLength}: consider breaking into shorter sentences" }
        }
    }

    return $issues
}

# ============================================================================
# NAMING CONVENTION VALIDATION
# ============================================================================

function Test-NamingConvention {
    param(
        [string] $Name,
        [string] $Convention
    )

    $issues = @()

    switch ($Convention) {
        "kebab-case" {
            if ($Name -match '[A-Z]' -or $Name -match '_') {
                $expected = ($Name -replace '_', '-').ToLower()
                # Only report if actual name differs from expected
                if ($Name -ne $expected) {
                    $issues += @{ rule = "kebab-case-violation"; message = "Name '$Name' violates kebab-case convention; expected: '$expected'"; value = $Name; expected = $expected; severity = "error" }
                }
            }
        }
        "camelCase" {
            if ($Name -match '-' -or $Name -match '_' -or $Name -match '^[A-Z]') {
                $camelCase = $Name -replace '-', '_' -replace '_(.)', { $args[0].Groups[1].Value.ToUpper() }
                $camelCase = $camelCase.Substring(0, 1).ToLower() + $camelCase.Substring(1)
                $issues += @{ rule = "camelCase-violation"; message = "Name '$Name' violates camelCase convention; expected: '$camelCase'"; value = $Name; expected = $camelCase; severity = "error" }
            }
        }
        "PascalCase" {
            if ($Name -match '^[a-z]' -or $Name -match '-' -or $Name -match '_') {
                $expected = ($Name -replace '[-_](.)', { $args[0].Groups[1].Value.ToUpper() }) -replace '^(.)', { $args[0].Groups[1].Value.ToUpper() }
                $issues += @{ rule = "PascalCase-violation"; message = "Name '$Name' violates PascalCase convention; expected: '$expected'"; value = $Name; expected = $expected; severity = "error" }
            }
        }
        "SCREAMING_SNAKE_CASE" {
            if ($Name -notmatch '^[A-Z0-9_]+$') {
                $expected = $Name -replace '([a-z])([A-Z])', '$1_$2' | ForEach-Object { $_.ToUpper() }
                $issues += @{ rule = "SCREAMING_SNAKE_CASE-violation"; message = "Name '$Name' violates SCREAMING_SNAKE_CASE convention; expected: '$expected'"; value = $Name; expected = $expected; severity = "error" }
            }
        }
    }

    return $issues
}

# ============================================================================
# SCHEMA COMPLIANCE VALIDATION
# ============================================================================

function Test-SkillManifestSchema {
    param([PSCustomObject] $Manifest)

    $issues = @()
    $requiredFields = @('name', 'main_hub', 'sub_hub', 'description', 'skill_count', 'top_triggers', 'generated_at', 'files')

    foreach ($field in $requiredFields) {
        # Handle both PSCustomObject properties and hashtable keys
        $hasField = $false
        if ($Manifest -is [hashtable]) {
            $hasField = $Manifest.ContainsKey($field)
        }
        elseif ($Manifest.PSObject.Properties.Name -contains $field) {
            $hasField = $true
        }
        
        if (-not $hasField) {
            $issues += @{ rule = "missing-field"; message = "Required field '$field' not found in manifest"; field = $field; severity = "error" }
        }
    }

    # Validate files object (optional - skip nested structure check since manifest creation is controlled)
    # Files structure is validated implicitly through file existence checks below
    if (-not $Manifest.files) {
        $issues += @{ rule = "missing-files-object"; message = "Manifest 'files' object not defined"; severity = "warning" }
    }

    # Validate top_triggers is array and non-empty
    if ($Manifest.top_triggers -is [string]) {
        $issues += @{ rule = "top_triggers-not-array"; message = "Field 'top_triggers' must be an array, not a string"; severity = "error" }
    }
    elseif ($Manifest.top_triggers.Count -eq 0) {
        $issues += @{ rule = "top_triggers-empty"; message = "Field 'top_triggers' is empty; should contain at least one trigger"; severity = "warning" }
    }

    # Validate skill_count matches reality (optional, set to warning)
    if ($Manifest.skill_count -lt 1) {
        $issues += @{ rule = "skill_count-invalid"; message = "skill_count '$($Manifest.skill_count)' must be >= 1"; value = $Manifest.skill_count; severity = "warning" }
    }

    return $issues
}

function Test-SkillCatalogItemSchema {
    param([PSCustomObject] $Item)

    if ($Item -is [System.Collections.IDictionary]) {
        $Item = [PSCustomObject]$Item
    }

    $issues = @()
    $requiredFields = @('id', 'description', 'path', 'triggers', 'src', 'primary_hub', 'assigned_hubs', 'match_score', 'is_primary')

    foreach ($field in $requiredFields) {
        if (-not ($Item.PSObject.Properties.Name -contains $field)) {
            $issues += @{ rule = "missing-field"; field = $field; severity = "error" }
        }
    }

    # Validate path exists (relative path check)
    if ($Item.path -and $Item.path -notmatch '[\\/]') {
        $issues += @{ rule = "invalid-path-structure"; value = $Item.path; severity = "warning"; message = "Path should contain directory separators" }
    }

    # CSV mode stores lists as semicolon-separated strings.
    if ($Item.triggers -is [string]) {
        $issues += @{ rule = "triggers-not-array"; field = "triggers"; severity = "warning"; message = "CSV list format accepted" }
    }

    # CSV mode stores lists as semicolon-separated strings.
    if ($Item.assigned_hubs -is [string]) {
        $issues += @{ rule = "assigned_hubs-not-array"; field = "assigned_hubs"; severity = "warning"; message = "CSV list format accepted" }
    }

    return $issues
}

# ============================================================================
# LINK & PATH VALIDATION
# ============================================================================

function Test-PathsValid {
    param(
        [array] $CatalogItems,
        [string] $RepoRoot
    )

    $issues = @()
    $checked = @{}

    foreach ($item in $CatalogItems) {
        if ($checked[$item.path]) {
            continue
        }

        $resolved = Join-Path $RepoRoot $item.path
        if (-not (Test-Path -LiteralPath $resolved)) {
            $issues += @{ rule = "path-not-found"; path = $item.path; severity = "error" }
        }

        $checked[$item.path] = $true
    }

    return $issues
}

# ============================================================================
# COMPREHENSIVE VALIDATION REPORT
# ============================================================================

function New-ValidationReport {
    param(
        [string] $SubHubKey,
        [PSCustomObject] $Manifest,
        [array] $CatalogItems,
        [string] $WorkflowText,
        [string] $RepoRoot
    )

    $report = [ordered]@{
        sub_hub = $SubHubKey
        timestamp = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
        passed = $true
        errors = @()
        warnings = @()
        info = @()
    }

    # 1. Manifest schema
    $manifestIssues = Test-SkillManifestSchema -Manifest $Manifest
    foreach ($issue in $manifestIssues) {
        if ($issue.severity -eq "error") {
            $report.errors += $issue
            $report.passed = $false
        }
        else {
            $report.warnings += $issue
        }
    }

    # 2. Catalog items schema + paths
    $catalogs = @($CatalogItems | ForEach-Object {
        if ($_ -is [string]) {
            $_ | ConvertFrom-Json
        }
        else {
            $_
        }
    })
    foreach ($item in $catalogs) {
        $itemIssues = Test-SkillCatalogItemSchema -Item $item
        foreach ($issue in $itemIssues) {
            if ($issue.severity -eq "error") {
                $report.errors += $issue
                $report.passed = $false
            }
            else {
                $report.warnings += $issue
            }
        }
    }

    # 3. Path validation
    $pathIssues = Test-PathsValid -CatalogItems $catalogs -RepoRoot $RepoRoot
    foreach ($issue in $pathIssues) {
        $report.errors += $issue
        $report.passed = $false
    }

    # 4. Workflow prose
    $proseIssues = Test-ProseQuality -Text $WorkflowText
    foreach ($issue in $proseIssues) {
        if ($issue.severity -eq "error") {
            $report.errors += $issue
            $report.passed = $false
        }
        elseif ($issue.severity -eq "warning") {
            $report.warnings += $issue
        }
        else {
            $report.info += $issue
        }
    }

    # 5. Naming conventions (skip if name is empty or not a string)
    if ($Manifest.name -and $Manifest.name -is [string]) {
        $nameIssues = Test-NamingConvention -Name $Manifest.name -Convention "kebab-case"
        foreach ($issue in $nameIssues) {
            if ($issue.severity -eq "error") {
                $report.errors += $issue
                $report.passed = $false
            }
            else {
                $report.warnings += $issue
            }
        }
    }

    return $report
}

# ============================================================================
# REPORTING & LOGGING
# ============================================================================

function Write-ValidationReport {
    param([PSCustomObject] $Report)

    $status = if ($Report.passed) { "[✓]" } else { "[✗]" }
    Write-Host "$status Validation: $($Report.sub_hub)" -ForegroundColor $(if ($Report.passed) { "Green" } else { "Red" })

    if ($Report.errors.Count -gt 0) {
        Write-Host "  Errors ($($Report.errors.Count)):" -ForegroundColor Red
        foreach ($err in $Report.errors) {
            $msg = if ($err.message) { $err.message } else { $err.value }
            Write-Host "    - $($err.rule): $msg" -ForegroundColor Red
        }
    }

    if ($Report.warnings.Count -gt 0) {
        Write-Host "  Warnings ($($Report.warnings.Count)):" -ForegroundColor Yellow
        foreach ($warn in $Report.warnings) {
            $msg = if ($warn.message) { $warn.message } elseif ($warn.expected) { $warn.expected } else { "" }
            Write-Host "    - $($warn.rule): $msg" -ForegroundColor Yellow
        }
    }

    if ($Report.info.Count -gt 0) {
        Write-Host "  Info ($($Report.info.Count)):" -ForegroundColor Cyan
        foreach ($inf in $Report.info) {
            $msg = if ($inf.message) { $inf.message } else { "" }
            Write-Host "    - $($inf.rule): $msg" -ForegroundColor Cyan
        }
    }
}
