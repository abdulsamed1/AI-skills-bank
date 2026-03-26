# Clone all skills repositories from source.md

param(
    [switch] $DryRun,
    [switch] $Force
)

$srcDir = Join-Path $PSScriptRoot "..\src"
$sourceListFile = Join-Path $srcDir "source.md"

# Repository list to clone
$repos = @(
    "anthropics/skills",
    "supabase/agent-skills",
    "google-gemini/gemini-skills",
    "callstackincubator/agent-skills",
    "better-auth/skills",
    "tinybirdco/tinybird-agent-skills",
    "hashicorp/agent-skills",
    "sanity-io/agent-toolkit",
    "firecrawl/cli",
    "neondatabase/agent-skills",
    "ClickHouse/agent-skills",
    "remotion-dev/skills",
    "replicate/skills",
    "typefully/agent-skills",
    "vercel-labs/agent-skills",
    "cloudflare/skills",
    "netlify/context-and-tools",
    "google-labs-code/stitch-skills",
    "google-workspace/google-workspace-cli",
    "expo/skills",
    "trailofbits/skills",
    "getsentry/skills",
    "microsoft/skills",
    "fal-ai-community/skills",
    "WordPress/agent-skills",
    "transloadit/skills",
    "openai/skills",
    "figma/mcp-server-guide",
    "coreyhaines31/marketingskills",
    "binance/binance-skills-hub",
    "deanpeters/Product-Manager-Skills",
    "phuryn/pm-skills",
    "MiniMax-AI/skills",
    "duckdb/duckdb-skills",
    "greensock/gsap-skills",
    "garrytan/gstack",
    "BrianRWagner/ai-marketing-skills",
    "AgriciDaniel/claude-seo",
    "wshuyi/x-article-publisher-skill",
    "CosmoBlk/email-marketing-bible",
    "smixs/creative-director-skill",
    "Xquik-dev/tweetclaw",
    "SHADOWPR0/beautiful_prose",
    "blader/humanizer",
    "Eronred/aso-skills",
    "PSPDFKit-labs/nutrient-agent-skill",
    "op7418/NanoBanana-PPT-Skills",
    "zarazhangrui/frontend-slides",
    "PleasePrompto/notebooklm-skill",
    "obra/superpowers-lab",
    "obra/superpowers",
    "op7418/Youtube-clipper-skill",
    "ognjengt/founder-skills",
    "EveryInc/charlie-cfo-skill",
    "wrsmith108/linear-claude-skill",
    "Shpigford/skills"
)

# Add special cases (non-github repos or special handling)
$specialRepos = @{
    "microsoft/skills" = @{
        url = "https://github.com/microsoft/skills.git"
        branch = "main"
    }
    "google-workspace/google-workspace-cli" = @{
        url = "https://github.com/google-workspace/google-workspace-cli.git"
        branch = "main"
    }
    "obra/superpowers" = @{
        url = "https://github.com/obra/superpowers.git"
        branch = "main"
    }
}

Write-Host "[INFO] Repository Cloning System" -ForegroundColor Cyan
Write-Host "[INFO] Source directory: $srcDir"
Write-Host "[INFO] Total repositories to clone: $($repos.Count)"
Write-Host ""

if ($DryRun) {
    Write-Host "[DRY-RUN] No changes will be made. Listing repositories to clone:" -ForegroundColor Yellow
    foreach ($repo in $repos) {
        Write-Host "  - $repo"
    }
    exit 0
}

# Ensure src directory exists
if (-not (Test-Path $srcDir)) {
    mkdir $srcDir -Force | Out-Null
}

$successCount = 0
$failureCount = 0
$skippedCount = 0
$errors = @()

foreach ($repo in $repos) {
    $repoName = ($repo -split '/')[-1]
    $targetPath = Join-Path $srcDir $repoName
    
    # Skip if already cloned (unless -Force)
    if ((Test-Path $targetPath) -and -not $Force) {
        Write-Host "[⊘] $repo (already cloned)" -ForegroundColor Gray
        $skippedCount++
        continue
    }
    
    # Remove if -Force
    if ((Test-Path $targetPath) -and $Force) {
        Write-Host "[↯] Removing $targetPath for re-clone..." -ForegroundColor Yellow
        Remove-Item $targetPath -Recurse -Force | Out-Null
    }
    
    $url = "https://github.com/$repo.git"
    
    try {
        Write-Host "[↓] Cloning $repo..." -ForegroundColor Cyan
        & git clone --depth 1 $url $targetPath 2>&1 | Out-Null
        $successCount++
        Write-Host "[✓] $repo cloned successfully" -ForegroundColor Green
    }
    catch {
        Write-Host "[✗] Failed to clone $repo" -ForegroundColor Red
        $failureCount++
        $errors += "[ERROR] $repo : $_"
    }
}

Write-Host ""
Write-Host "[INFO] ============================================" -ForegroundColor Cyan
Write-Host "[INFO] Clone Operation Complete" -ForegroundColor Cyan
Write-Host "[INFO]   Successful: $successCount" -ForegroundColor Green
Write-Host "[INFO]   Skipped: $skippedCount" -ForegroundColor Gray
Write-Host "[INFO]   Failed: $failureCount" -ForegroundColor $(if ($failureCount -gt 0) {"Red"} else {"Green"})
Write-Host "[INFO] ============================================" -ForegroundColor Cyan
Write-Host ""

if ($errors.Count -gt 0) {
    Write-Host "[ERRORS]" -ForegroundColor Red
    foreach ($error in $errors) {
        Write-Host "  $error" -ForegroundColor Red
    }
    Write-Host ""
}

exit $(if ($failureCount -gt 0) { 1 } else { 0 })
