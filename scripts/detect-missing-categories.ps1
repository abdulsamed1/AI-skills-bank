# Missing Category Detector
# Analyzes skill descriptions to identify uncategorized skill groups
# Alerts when a category has substantial skills but no dedicated hub

param(
    [string] $CatalogPath = "",
    [int] $MinSkillsPerCategory = 10,
    [int] $TopCategoriesCount = 15
)

# Resolve path
if (-not $CatalogPath -or -not (Test-Path $CatalogPath)) {
    # Try script location
    $scriptDir = Split-Path -Parent $PSCommandPath
    $candidates = @(
        (Join-Path $scriptDir "..\AI-skills-bank\skills-aggregated\general\misc\skills-catalog.ndjson"),
        "C:\Users\ASUS\production\AI-skills-bank\skills-aggregated\general\misc\skills-catalog.ndjson"
    )
    
    foreach ($candidate in $candidates) {
        if (Test-Path $candidate) {
            $CatalogPath = $candidate
            break
        }
    }
}

if (-not (Test-Path $CatalogPath)) {
    Write-Host "[ERROR] Catalog not found at $CatalogPath" -ForegroundColor Red
    exit 1
}

Write-Host "[INFO] Analyzing skills for uncategorized patterns..." -ForegroundColor Cyan

# Common category keywords (maps to potential hub names)
$CategoryPatterns = @{
    "marketing" = @("marketing", "brand", "seo", "copywrite", "campaign", "audience", "positioning", "email-marketing", "content-marketing", "social-media", "newsletter", "engagement", "viral", "publisher")
    "data-science" = @("machine-learning", "ml", "data-science", "nlp", "deep-learning", "neural", "tensorflow", "pytorch", "scikit", "pandas", "numpy", "analytics", "prediction", "classification")
    "mobile" = @("ios", "android", "react-native", "flutter", "swift", "kotlin", "mobile-app", "cross-platform", "cordova")
    "design" = @("ui-design", "ux-design", "figma", "design-system", "wireframe", "accessibility", "responsive", "design-pattern", "font", "color-theory")
    "testing" = @("testing", "unit-test", "integration-test", "e2e-test", "cypress", "selenium", "jest", "mocha", "vitest", "test-automation", "qa", "quality-assurance")
    "security" = @("security", "encryption", "authentication", "oauth", "jwt", "penetration", "vulnerability", "secure", "password", "hashing", "ssl", "tls", "cors", "csrf")
    "documentation" = @("documentation", "api-docs", "swagger", "openapi", "readme", "technical-writing", "knowledge-base", "wiki")
    "devtools" = @("git", "github", "gitlab", "bitbucket", "ci-cd", "jenkins", "gitlab-ci", "github-actions", "docker-compose", "devops", "monitoring", "logging", "observability")
    "ai-llm" = @("gpt", "llm", "language-model", "ai-agent", "prompt-engineering", "fine-tuning", "embedding", "vector", "rag", "transformer", "attention", "chatbot")
    "productivity" = @("productivity", "automation", "workflow", "task-management", "time-management", "project-management", "agile", "scrum", "kanban", "notion", "asana")
}

# Load and analyze skills
$catalog = @()
Get-Content $CatalogPath | ForEach-Object {
    $catalog += ($_ | ConvertFrom-Json)
}

Write-Host "[✓] Loaded $($catalog.Count) skills from generic/misc" -ForegroundColor Green

# Analyze by category
$categoryScores = @{}

foreach ($item in $catalog) {
    $skillText = "$($item.id) $($item.description) $($item.path)".ToLower()
    
    foreach ($categoryName in $CategoryPatterns.Keys) {
        $keywords = $CategoryPatterns[$categoryName]
        
        $matchCount = 0
        foreach ($keyword in $keywords) {
            if ($skillText -match [regex]::Escape($keyword)) {
                $matchCount++
            }
        }
        
        if ($matchCount -gt 0) {
            if (-not $categoryScores[$categoryName]) {
                $categoryScores[$categoryName] = @{
                    count = 0
                    skills = @()
                    matchScore = 0
                }
            }
            
            $categoryScores[$categoryName].count++
            $categoryScores[$categoryName].matchScore += $matchCount
            $categoryScores[$categoryName].skills += $item.id
        }
    }
}

# Sort by count and report
Write-Host ""
Write-Host "[ANALYSIS RESULTS]" -ForegroundColor Yellow
Write-Host "==================" -ForegroundColor Yellow

$sorted = $categoryScores.GetEnumerator() | 
    Where-Object { $_.Value.count -ge $MinSkillsPerCategory } |
    Sort-Object { $_.Value.count } -Descending |
    Select-Object -First $TopCategoriesCount

if ($sorted.Count -eq 0) {
    Write-Host "[INFO] No uncategorized skill groups found with >= $MinSkillsPerCategory skills" -ForegroundColor Green
}
else {
    Write-Host "[WARN] Found $($sorted.Count) potential new hub categories:" -ForegroundColor Yellow
    Write-Host ""
    
    $index = 1
    foreach ($category in $sorted) {
        $name = $category.Key
        $count = $category.Value.count
        $score = $category.Value.matchScore
        
        $status = if ($count -ge 50) { "CRITICAL" } elseif ($count -ge 30) { "IMPORTANT" } else { "CONSIDER" }
        $color = if ($status -eq "CRITICAL") { "Red" } elseif ($status -eq "IMPORTANT") { "Yellow" } else { "Cyan" }
        
        Write-Host "  [$index] $($name.ToUpper()) - $count skills (score: $score) [$status]" -ForegroundColor $color
        $index++
    }
}

# Generate configuration update suggestions
Write-Host ""
Write-Host "[CONFIG UPDATE SUGGESTIONS]" -ForegroundColor Cyan
Write-Host "============================" -ForegroundColor Cyan
Write-Host ""
Write-Host "To add new hubs, update SUB_HUB_DEFINITIONS in aggregate-skills-to-subhubs.ps1:" -ForegroundColor Cyan
Write-Host ""

foreach ($category in $sorted | Select-Object -First 3) {
    $name = $category.Key
    $keywords = $CategoryPatterns[$name] -join '", "'
    
    Write-Host "    `"$name`" = @{" -ForegroundColor Green
    Write-Host "        `"default`" = @{" -ForegroundColor Green
    Write-Host "            keywords = @(`"$keywords`")" -ForegroundColor Green
    Write-Host "            description = `"Add description here`"" -ForegroundColor Green
    Write-Host "            best_for = @(" -ForegroundColor Green
    Write-Host "                `"Use case 1`"," -ForegroundColor Green
    Write-Host "                `"Use case 2`"" -ForegroundColor Green
    Write-Host "            )" -ForegroundColor Green
    Write-Host "        }" -ForegroundColor Green
    Write-Host "    }" -ForegroundColor Green
    Write-Host ""
}

# Export detailed report
$reportPath = Join-Path (Split-Path $CatalogPath) ".category-analysis.json"
$report = @{
    generated_at = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssK")
    total_skills_analyzed = $catalog.Count
    categories_found = $categoryScores.Count
    categories_meeting_threshold = ($sorted | Measure-Object).Count
    min_threshold = $MinSkillsPerCategory
    uncategorized_suggestions = @($sorted | ForEach-Object {
        @{
            name = $_.Key
            skill_count = $_.Value.count
            match_score = $_.Value.matchScore
            sample_skills = @($_.Value.skills | Select-Object -First 5)
        }
    })
}

$report | ConvertTo-Json -Depth 8 | Set-Content $reportPath
Write-Host "[✓] Full analysis saved to .category-analysis.json" -ForegroundColor Green
