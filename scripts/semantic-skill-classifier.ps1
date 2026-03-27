# Semantic Skill Classifier
# Uses Claude AI to understand skill purposes and classify them semantically
# Enhances keyword-based routing with semantic understanding

param(
    [string] $SkillsDir = ".\AI-skills-bank\src",
    [string] $OutputFile = ".\AI-skills-bank\skills-aggregated\semantic-classifications.json",
    [int] $MaxSkillsToClassify = 100,
    [Switch] $DryRun = $false,
    [string] $ApiKey = $null,
    [ValidateRange(5, 300)]
    [int] $ApiTimeoutSec = 30,
    [ValidateRange(1, 5)]
    [int] $MaxRetries = 2
)

# Detect API key from environment if not passed
if ([string]::IsNullOrWhiteSpace($ApiKey)) {
    $ApiKey = $env:ANTHROPIC_API_KEY
}

if ([string]::IsNullOrWhiteSpace($ApiKey) -and -not $DryRun) {
    Write-Error "ANTHROPIC_API_KEY environment variable not set. Cannot classify skills semantically."
    exit 1
}

$HubDefinitions = @{
    "marketing" = @("content", "copywriting", "writing", "humanizing", "tone", "voice", "persuasion", "storytelling")
    "business" = @("product", "strategy", "roadmap", "prd", "planning", "saas", "pricing", "unit-economics")
    "content" = @("writing", "editing", "humanize", "prose", "narrative", "copywriting", "article", "blog")
    "creative" = @("design", "ux", "ui", "visual", "wireframe", "prototyping", "user-experience")
    "technical" = @("code", "backend", "frontend", "api", "database", "devops", "infrastructure")
    "data" = @("analytics", "machine-learning", "data-science", "sql", "statistical", "reporting")
    "testing" = @("test", "qa", "automation", "verification", "validation", "coverage")
    "general" = @("miscellaneous", "utility", "helper")
}

function Get-SkillMetadata {
    param([string] $SkillPath)
    
    if (-not (Test-Path $SkillPath)) {
        return $null
    }

    try {
        $content = Get-Content -Path $SkillPath -Raw -ErrorAction Stop
        $match = $content -match "^---`r?`n([\s\S]*?)`r?`n---"
        
        if ($match) {
            $frontmatter = $matches[1]
            $skillMetadata = @{}
            
            $frontmatter -split "`n" | ForEach-Object {
                if ($_ -match "^(\w+):\s*(.*)$") {
                    $key = $matches[1].Trim()
                    $value = $matches[2].Trim().Trim('"')
                    $skillMetadata[$key] = $value
                }
            }
            
            # Extract description
            $descStart = $content.IndexOf("`n---`n") + 5
            $descContent = $content.Substring($descStart).Trim()
            $descContent = $descContent -replace "^# .*`r?`n", ""
            $skillMetadata["full_description"] = $descContent.Substring(0, [Math]::Min(500, $descContent.Length))
            
            return $skillMetadata
        }
    }
    catch {
        Write-Warning "Failed to read metadata from $SkillPath`: $_"
    }
    
    return $null
}

function Invoke-SemanticClassification {
    param(
        [string] $SkillId,
        [string] $SkillDescription,
        [hashtable] $Hubs
    )

    $hubList = ($Hubs.Keys | ForEach-Object { 
        $hubName = $_
        $keywords = $Hubs[$hubName] -join ", "
        "$($hubName): $keywords"
    }) -join "`n"
    
    $userPrompt = @"
Classify this skill into the most appropriate domain/hub.

SKILL ID: $SkillId
DESCRIPTION: $SkillDescription

AVAILABLE HUBS WITH KEYWORDS:
$hubList

TASK:
1. Identify the PRIMARY PURPOSE of this skill
2. Determine which hub it belongs in based on semantic meaning (not keyword matching)
3. Rate your confidence: 1-10 (10=certain, 1=very uncertain)
4. List top 3 hubs sorted by match strength

RESPOND ONLY WITH VALID JSON (no markdown, no code fence):
{
  "skill_id": "$SkillId",
  "primary_purpose": "one sentence describing what this skill does",
  "primary_hub": "best hub name",
  "confidence": 8,
  "top_matches": [
    {"hub": "marketing", "reason": "brief reason", "score": 9},
    {"hub": "content", "reason": "brief reason", "score": 8},
    {"hub": "creative", "reason": "brief reason", "score": 6}
  ]
}
"@

    $headers = @{
        "x-api-key" = $ApiKey
        "anthropic-version" = "2023-06-01"
        "content-type" = "application/json"
    }

    $body = @{
        model = "claude-3-5-sonnet-20241022"
        max_tokens = 500
        messages = @(
            @{
                role = "user"
                content = $userPrompt
            }
        )
    } | ConvertTo-Json -Depth 10

    for ($attempt = 1; $attempt -le $MaxRetries; $attempt++) {
        try {
            $response = Invoke-RestMethod `
                -Uri "https://api.anthropic.com/v1/messages" `
                -Method POST `
                -Headers $headers `
                -Body $body `
                -TimeoutSec $ApiTimeoutSec `
                -ErrorAction Stop

            if ($response.content -and $response.content[0].text) {
                $responseText = $response.content[0].text.Trim()

                # Clean markdown code fences if present
                $responseText = $responseText -replace '```json\s*', ''
                $responseText = $responseText -replace '```\s*$', ''

                $classification = $responseText | ConvertFrom-Json -ErrorAction Stop
                return $classification
            }
        }
        catch {
            Write-Warning "Attempt $attempt/$MaxRetries failed for skill '$SkillId': $_"
            if ($attempt -lt $MaxRetries) {
                Start-Sleep -Seconds 1
            }
        }
    }

    return $null
}

function Find-LowScoringSkills {
    param([string] $SourceDir)

    $skills = @()
    $itemsToCheck = @(Get-ChildItem -Path $SourceDir -Directory -ErrorAction SilentlyContinue)
    
    foreach ($item in $itemsToCheck) {
        # Check if this directory itself has a SKILL.md (top-level skill)
        $skillPath = Join-Path $item.FullName "SKILL.md"
        
        if (Test-Path $skillPath) {
            $metadata = Get-SkillMetadata -SkillPath $skillPath
            if ($metadata -and -not [string]::IsNullOrWhiteSpace($metadata["description"])) {
                $skills += [PSCustomObject]@{
                    id = $item.Name
                    description = $metadata["description"]
                    path = $skillPath
                    full_description = $metadata["full_description"]
                }
            }
        }
        else {
            # Otherwise check subdirectories for skills
            $subDirs = @(Get-ChildItem -Path $item.FullName -Directory -ErrorAction SilentlyContinue)
            foreach ($subDir in $subDirs) {
                $skillPath = Join-Path $subDir.FullName "SKILL.md"
                
                if (Test-Path $skillPath) {
                    $metadata = Get-SkillMetadata -SkillPath $skillPath
                    if ($metadata -and -not [string]::IsNullOrWhiteSpace($metadata["description"])) {
                        $skills += [PSCustomObject]@{
                            id = $subDir.Name
                            description = $metadata["description"]
                            path = $skillPath
                            full_description = $metadata["full_description"]
                        }
                    }
                }
            }
        }
    }

    return @($skills | Select-Object -First $MaxSkillsToClassify)
}

# Main execution
Write-Host "[*] Semantic Skill Classifier Starting..." -ForegroundColor Cyan

if ($DryRun) {
    Write-Host "[DRY-RUN] Would classify skills but API calls disabled" -ForegroundColor Yellow
}

$skills = Find-LowScoringSkills -SourceDir $SkillsDir
Write-Host "[✓] Found $($skills.Count) skills to classify" -ForegroundColor Green

$classifications = @()

for ($i = 0; $i -lt $skills.Count; $i++) {
    $skill = $skills[$i]
    $percent = [Math]::Round(($i + 1) / $skills.Count * 100, 1)
    
    Write-Host "[$percent%] Classifying: $($skill.id)" -ForegroundColor Cyan
    
    if (-not $DryRun) {
        $classification = Invoke-SemanticClassification `
            -SkillId $skill.id `
            -SkillDescription $skill.description `
            -Hubs $HubDefinitions
        
        if ($classification) {
            $classifications += $classification
            Write-Host "      → Primary hub: $($classification.primary_hub) (confidence: $($classification.confidence)/10)" -ForegroundColor Green
        }
        
        # Rate limiting: slow down API calls to avoid throttling
        Start-Sleep -Milliseconds 500
    }
    else {
        Write-Host "      [DRY-RUN] Would classify as: unknown" -ForegroundColor Yellow
    }
}

# Save results
if ($classifications.Count -gt 0) {
    $outputDir = Split-Path -Parent $OutputFile
    if (-not (Test-Path $outputDir)) {
        New-Item -ItemType Directory -Path $outputDir -Force | Out-Null
    }
    
    $classifications | ConvertTo-Json -Depth 10 | Out-File -FilePath $OutputFile -Encoding UTF8-NoBOM
    Write-Host "[✓] Semantic classifications saved to: $OutputFile" -ForegroundColor Green
    Write-Host "[✓] Total classified: $($classifications.Count) skills" -ForegroundColor Green
}
else {
    Write-Host "[!] No classifications generated (dry-run or API failures)" -ForegroundColor Yellow
}

exit 0
