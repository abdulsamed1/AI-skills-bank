# Skill Semantic Analyzer - AI Agent Review Workflow
# Instead of external API calls, this workflow:
# 1. Extracts low-scoring/unmatched skills
# 2. Prepares them for AI Agent analysis
# 3. Collects AI review results into structured JSON
# 4. Integrates recommendations into aggregation

param(
    [string] $SkillsSourceDir = ".\AI-skills-bank\src",
    [string] $OutputFile = ".\AI-skills-bank\skills-aggregated\ai-semantic-analysis.json",
    [int] $MaxSkillsToAnalyze = 50,
    [Switch] $DryRun = $false,
    [string] $PreviousReviewFile = $null,
    [Switch] $IncludeFullContent = $false,
    [ValidateRange(0, 10)]
    [int] $WriteRetryCount = 2
)

Write-Host "[*] AI Semantic Analyzer - Preparing Skills for Agent Review" -ForegroundColor Cyan

# Step 1: Find all low-scoring skills that need semantic analysis
function Find-SkillsNeedingAnalysis {
    param([string] $SourceDir, [string] $PrevReviewFile)

    $skillList = @()
    $itemsToCheck = @(Get-ChildItem -Path $SourceDir -Directory -ErrorAction SilentlyContinue)
    
    $scannedDirs = 0
    foreach ($item in $itemsToCheck) {
        if ($skillList.Count -ge $MaxSkillsToAnalyze) {
            break
        }

        $scannedDirs++
        if (($scannedDirs % 100) -eq 0) {
            Write-Host "  Scanned directories: $scannedDirs | Collected skills: $($skillList.Count)" -ForegroundColor DarkGray
        }

        $skillPath = Join-Path $item.FullName "SKILL.md"
        
        if (Test-Path $skillPath) {
            $skillList += [PSCustomObject]@{
                id = $item.Name
                path = $skillPath
                dir = $item.FullName
            }

            if ($skillList.Count -ge $MaxSkillsToAnalyze) {
                break
            }
        }
        else {
            $subDirs = @(Get-ChildItem -Path $item.FullName -Directory -ErrorAction SilentlyContinue)
            foreach ($subDir in $subDirs) {
                if ($skillList.Count -ge $MaxSkillsToAnalyze) {
                    break
                }

                $skillPath = Join-Path $subDir.FullName "SKILL.md"
                if (Test-Path $skillPath) {
                    $skillList += [PSCustomObject]@{
                        id = $subDir.Name
                        path = $skillPath
                        dir = $subDir.FullName
                    }
                }
            }
        }
    }

    # Filter out already-analyzed skills if previous review exists
    if ($PrevReviewFile -and (Test-Path $PrevReviewFile)) {
        try {
            $prevAnalyzed = Get-Content $PrevReviewFile -Raw | ConvertFrom-Json -ErrorAction SilentlyContinue
            if ($prevAnalyzed) {
                $analyzed_ids = @($prevAnalyzed | ForEach-Object { $_.skill_id })
                $skillList = @($skillList | Where-Object { $_.id -notin $analyzed_ids })
            }
        }
        catch {
            Write-Warning "Could not load previous review file, analyzing all skills"
        }
    }

    return @($skillList | Select-Object -First $MaxSkillsToAnalyze)
}

# Step 2: Extract skill metadata
function Get-SkillContent {
    param([string] $SkillPath)
    
    try {
        $content = Get-Content -Path $SkillPath -Raw -ErrorAction Stop
        
        # Extract YAML frontmatter
        $match = $content -match "^---`r?`n([\s\S]*?)`r?`n---"
        $frontmatter = if ($match) { $matches[1] } else { "" }
        
        # Extract description from YAML
        $desc = ""
        if ($frontmatter -match "description:\s*\|\s*([\s\S]*?)(?=^[a-z]|$)") {
            $desc = $matches[1].Trim()
        }
        elseif ($frontmatter -match "description:\s*(.+)$") {
            $desc = $matches[1].Trim().Trim('"')
        }
        
        # Get first 200 chars of main content
        $bodyStart = $content.IndexOf("`n---`n")
        if ($bodyStart -ge 0) {
            $bodyStart += 5
            $mainContent = $content.Substring($bodyStart).Trim()
        }
        else {
            $mainContent = $content.Trim()
        }
        $mainContent = $mainContent -replace "^# .*`r?`n", ""
        $summary = $mainContent.Substring(0, [Math]::Min(200, $mainContent.Length))
        
        return @{
            description = $desc
            summary = $summary
            full_content = $content
        }
    }
    catch {
        Write-Warning "Failed to read $SkillPath`: $_"
        return $null
    }
}

# Step 3: Format skills for agent analysis
function Format-SkillsForAnalysis {
    param([array] $Skills, [bool] $IncludeFull = $false)

    $formatted = @()
    
    foreach ($skill in $Skills) {
        Write-Host "  Reading: $($skill.id)" -ForegroundColor Cyan
        
        $content = Get-SkillContent -SkillPath $skill.path
        
        if ($content) {
            $skillObj = [PSCustomObject]@{
                skill_id = $skill.id
                path = $skill.path
                description = $content.description
                summary = $content.summary
                full_content = $(if ($IncludeFull) { $content.full_content } else { $null })
            }
            
            # Output debug info
            Write-Host "    Description length: $($content.description.Length)" -ForegroundColor Gray
            
            $formatted += $skillObj
        }
        else {
            Write-Host "    FAILED to read metadata" -ForegroundColor Yellow
        }
    }
    
    Write-Host "[✓] Formatted $($formatted.Count) skills" -ForegroundColor Green
    return $formatted
}

# Main execution
$skillsToAnalyze = Find-SkillsNeedingAnalysis -SourceDir $SkillsSourceDir -PrevReviewFile $PreviousReviewFile

if ($skillsToAnalyze.Count -eq 0) {
    Write-Host "[!] No skills found needing analysis" -ForegroundColor Yellow
    exit 0
}

Write-Host "[✓] Found $($skillsToAnalyze.Count) skills needing AI semantic analysis" -ForegroundColor Green

if (-not $DryRun) {
    $formattedSkills = Format-SkillsForAnalysis -Skills $skillsToAnalyze -IncludeFull:$IncludeFullContent
    
    # Output NDJSON format (one skill per line) for easy streaming to agent
    $outputDir = Split-Path -Parent $OutputFile
    if (-not (Test-Path $outputDir)) {
        New-Item -ItemType Directory -Path $outputDir -Force | Out-Null
    }
    
    $ndJsonFile = $OutputFile -replace "\.json$", ".ndjson"
    
    # Write each skill as its own JSON line.
    # Use direct value writing to avoid blocking on empty pipeline input.
    $lines = @()
    foreach ($skill in $formattedSkills) {
        $lines += ($skill | ConvertTo-Json -Depth 10 -Compress)
    }
    Write-Host "[i] Writing $($lines.Count) NDJSON records..." -ForegroundColor Gray

    $writeSucceeded = $false
    $tempFile = "$ndJsonFile.tmp"
    $lastWriteError = $null
    for ($attempt = 1; $attempt -le $WriteRetryCount; $attempt++) {
        try {
            Set-Content -Path $tempFile -Value ($lines -join "`n") -Force -Encoding UTF8 -ErrorAction Stop
            if (Test-Path $ndJsonFile) {
                Remove-Item -Path $ndJsonFile -Force -ErrorAction Stop
            }
            Move-Item -Path $tempFile -Destination $ndJsonFile -Force -ErrorAction Stop
            $writeSucceeded = $true
            break
        }
        catch {
            $lastWriteError = $_
            if (Test-Path $tempFile) {
                Remove-Item -Path $tempFile -Force -ErrorAction SilentlyContinue
            }
            Write-Warning "Write attempt $attempt/$WriteRetryCount failed for '$ndJsonFile': $_"
            if ($attempt -lt $WriteRetryCount) {
                Start-Sleep -Milliseconds 400
            }
        }
    }

    $effectiveOutputFile = $ndJsonFile
    if (-not $writeSucceeded) {
        $timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
        $fallbackFile = $ndJsonFile -replace "\.ndjson$", ".${timestamp}.ndjson"
        try {
            Set-Content -Path $fallbackFile -Value ($lines -join "`n") -Force -Encoding UTF8 -ErrorAction Stop
            $writeSucceeded = $true
            $effectiveOutputFile = $fallbackFile
            Write-Warning "Primary output file is locked. Wrote to fallback file: $fallbackFile"
        }
        catch {
            Write-Error "Could not write '$ndJsonFile' or fallback '$fallbackFile'. Last error: $_"
            exit 1
        }
    }

    Write-Host "[✓] Skills prepared for agent analysis:" -ForegroundColor Green
    Write-Host "    Output: $effectiveOutputFile" -ForegroundColor Green
    Write-Host "    Format: NDJSON (one skill per line)" -ForegroundColor Green
    Write-Host "" -ForegroundColor Green
    Write-Host "    NEXT STEP: Use AI Agent to analyze and classify these skills" -ForegroundColor Cyan
    Write-Host "    The Agent will read each skill's description and PURPOSE, then recommend:" -ForegroundColor Cyan
    Write-Host "      - primary_hub (best matching domain)" -ForegroundColor Cyan
    Write-Host "      - confidence (1-10 rating)" -ForegroundColor Cyan
    Write-Host "      - reason (why this hub)" -ForegroundColor Cyan
}
else {
    Write-Host "[DRY-RUN] Would prepare $($skillsToAnalyze.Count) skills for agent analysis" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "[✓] Preparation complete. Waiting for Agent analysis..." -ForegroundColor Green

exit 0
