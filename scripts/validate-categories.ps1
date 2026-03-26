# Pre-Aggregation Category Validation
# Runs before aggregation to ensure no major skill groups are left uncategorized

param(
    [string] $SourceRepoDir,
    [hashtable] $SubHubDefinitions,
    [int] $MinCriticalThreshold = 60,
    [int] $MinImportantThreshold = 30
)

Write-Host "[INFO] Running pre-aggregation category validation..." -ForegroundColor Cyan

# Get all unique keywords from existing SUB_HUB_DEFINITIONS
$definedKeywords = @()
foreach ($mainHub in $SubHubDefinitions.Keys) {
    foreach ($subHub in $SubHubDefinitions[$mainHub].Keys) {
        $def = $SubHubDefinitions[$mainHub][$subHub]
        if ($def.keywords) {
            $definedKeywords += @($def.keywords)
        }
    }
}

Write-Host "[✓] Found $($definedKeywords.Count) defined keywords across hubs" -ForegroundColor Green

# Known major categories that must have dedicated hubs
$requiredCategories = @{
    "productivity" = 60
    "security" = 60
    "testing" = 60
    "ai-llm" = 60
    "data-science" = 20
}

# Check which required categories are covered
$missingCritical = @()
foreach ($category in $requiredCategories.Keys) {
    $threshold = $requiredCategories[$category]
    $isCovered = $false
    
    foreach ($mainHub in $SubHubDefinitions.Keys) {
        foreach ($subHub in $SubHubDefinitions[$mainHub].Keys) {
            $def = $SubHubDefinitions[$mainHub][$subHub]
            $hubName = "$mainHub/$subHub".ToLower()
            
            if ($hubName -match $category) {
                $isCovered = $true
                break
            }
        }
        if ($isCovered) { break }
    }
    
    if (-not $isCovered) {
        $missingCritical += @{
            category = $category
            threshold = $threshold
        }
    }
}

# Report findings
if ($missingCritical.Count -gt 0) {
    Write-Host ""
    Write-Host "[WARN] Missing required hub categories:" -ForegroundColor Yellow
    foreach ($missing in $missingCritical) {
        Write-Host "  ⚠️  $($missing.category) (threshold: >= $($missing.threshold) skills)" -ForegroundColor Yellow
    }
    Write-Host ""
    Write-Host "[SUGGESTION] Consider adding these categories to SUB_HUB_DEFINITIONS before aggregating" -ForegroundColor Cyan
}
else {
    Write-Host "[✓] All required categories are covered" -ForegroundColor Green
}

return $missingCritical.Count -eq 0
