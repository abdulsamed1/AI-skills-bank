# Skill Aggregation System - BMAD Style Builder
# Transforms flat hub-manifest structure to sub-hub architecture
# Generates lightweight SKILL.md router + workflow.md + external catalog data

param(
    [string] $srcHubsDir = ".\AI-skills-bank\hub-skills",
    [string] $OutputDir = ".\AI-skills-bank\skills-aggregated",
    [array] $FallbackSkillRoots = @(".\_bmad", ".\AI-skills-bank\src"),
    [ValidateSet("latest", "all", "selected", "changed-only")]
    [string] $srcRepoMode = "latest",
    [string[]] $srcRepoNames = @(),
    [Switch] $DryRun = $false,
    [Switch] $AllowMultiHub = $false,
    [string[]] $ExcludeCategories = @(),
    [Switch] $NoCategoryExclusions = $false,
    [ValidateRange(1, 500)]
    [int] $MinSkillsPerHub = 10,
    [ValidateRange(1, 500)]
    [int] $CategoryGapThreshold = 30,
    [Switch] $FailOnCategoryGaps = $false,
    [ValidateRange(1, 5)]
    [int] $MaxHubsPerSkill = 3,
    [ValidateRange(1, 20)]
    [int] $PrimaryMinScore = 4,
    [ValidateRange(1, 20)]
    [int] $SecondaryMinScore = 6,
    [Switch] $EnableReviewBand = $false,
    [ValidateRange(1, 30)]
    [int] $ReviewMinScore = 4,
    [ValidateRange(1, 30)]
    [int] $AutoAcceptMinScore = 8,
    [Switch] $EnableSemanticScoring = $false,
    [string] $SemanticClassificationsFile = ".\AI-skills-bank\skills-aggregated\semantic-classifications.json",
    [ValidateRange(0.0, 1.0)]
    [double] $SemanticWeightFactor = 0.6
)

if ($ReviewMinScore -gt $AutoAcceptMinScore) {
    throw "ReviewMinScore ($ReviewMinScore) cannot be greater than AutoAcceptMinScore ($AutoAcceptMinScore)."
}

if ($EnableReviewBand -and $SecondaryMinScore -lt $AutoAcceptMinScore) {
    $SecondaryMinScore = $AutoAcceptMinScore
}

# src validation module (load from same directory)
$validationScriptPath = Join-Path $PSScriptRoot "validate-generated-skills.ps1"
if (-not (Test-Path $validationScriptPath)) {
    Write-Warning "Validation module not found at $validationScriptPath; attempting fallback..."
    $validationScriptPath = Join-Path (Split-Path $PSScriptRoot -Parent) "validate-generated-skills.ps1"
}
if (Test-Path $validationScriptPath) {
    . $validationScriptPath
    Write-Host "[✓] Loaded validation module from $validationScriptPath" -ForegroundColor Green
} else {
    Write-Error "Cannot load validation module; script will fail at validation checks."
}

# Use $PSScriptRoot to resolve paths relative to the script location
if ($PSScriptRoot) {
    # Normalize script root and derive repository root even when script lives under AI-skills-bank/scripts.
    $normalizedScriptRoot = (Get-Item $PSScriptRoot).FullName
    $candidateRootObj = Get-Item (Join-Path $normalizedScriptRoot "..")
    if ($candidateRootObj.Name -ieq "AI-skills-bank") {
        $RepoRootObj = Get-Item (Join-Path $candidateRootObj.FullName "..")
    }
    else {
        $RepoRootObj = $candidateRootObj
    }
    $RepoRoot = $RepoRootObj.FullName
    
    $legacyHubsDir = Join-Path $RepoRoot "AI-skills-bank/hub-skills"
    $srcReposDir = Join-Path $RepoRoot "AI-skills-bank/src"
    if (Test-Path $legacyHubsDir) {
        $srcHubsDir = $legacyHubsDir
    }
    else {
        # New layout keeps skills under src repos; hub-manifest is optional.
        $srcHubsDir = $srcReposDir
    }
    $OutputDir = Join-Path $RepoRoot "AI-skills-bank/skills-aggregated"
    $FallbackSkillRoots = @(
        (Join-Path $RepoRoot "_bmad"),
        $srcReposDir
    )
}

$srcRootPath = Join-Path $RepoRoot "AI-skills-bank/src"
$SkillLockPath = Join-Path $OutputDir ".skill-lock.json"
$RestrictsrcRepos = ($srcRepoMode -ne "all")
$ChangedOnlyFallbackToLatest = $false

function Get-srcRepoState {
    param([string] $RepoPath)

    $repoName = Split-Path -Leaf $RepoPath
    $state = [ordered]@{
        name = $repoName
        vcs = "filesystem"
        revision = $null
        dirty = $false
        fingerprint = $null
    }

    $hasGitRepo = Test-Path (Join-Path $RepoPath ".git")
    $gitCommand = Get-Command git -ErrorAction SilentlyContinue
    if ($hasGitRepo -and $gitCommand) {
        try {
            $revision = (& git -C $RepoPath rev-parse HEAD 2>$null)
            if ($LASTEXITCODE -eq 0 -and -not [string]::IsNullOrWhiteSpace($revision)) {
                $state.vcs = "git"
                $state.revision = ($revision | Select-Object -First 1).Trim()

                $statusOutput = (& git -C $RepoPath status --porcelain 2>$null)
                $statusText = ""
                if ($LASTEXITCODE -eq 0) {
                    $statusText = (($statusOutput | ForEach-Object { $_.TrimEnd() }) -join "`n")
                }

                if (-not [string]::IsNullOrWhiteSpace($statusText)) {
                    $state.dirty = $true
                }

                $statusHashInput = "$($state.revision)`n$statusText"
                $statusHash = [System.BitConverter]::ToString((New-Object Security.Cryptography.SHA256Managed).ComputeHash([System.Text.Encoding]::UTF8.GetBytes($statusHashInput))).Replace("-", "").ToLower()
                $state.fingerprint = "git:$statusHash"
            }
        }
        catch {
            # Fallback to filesystem fingerprint below.
        }
    }

    if ($state.vcs -ne "git") {
        $files = @(Get-ChildItem -Path $RepoPath -Recurse -File -ErrorAction SilentlyContinue)
        $latestTicks = 0
        if ($files.Count -gt 0) {
            $latestTicks = ($files | Sort-Object LastWriteTimeUtc -Descending | Select-Object -First 1).LastWriteTimeUtc.Ticks
        }
        $state.fingerprint = "files:$($files.Count)|ticks:$latestTicks"
    }

    return [PSCustomObject] $state
}

function Get-ChangedsrcRepos {
    param(
        [string] $srcRoot,
        [string] $LockPath
    )

    if (-not (Test-Path $srcRoot)) {
        return [PSCustomObject]@{
            HasLock = $false
            ChangedRepos = @()
            RepoStates = @()
        }
    }

    $repos = @(Get-ChildItem -Path $srcRoot -Directory)
    $repoStates = @($repos | ForEach-Object { Get-srcRepoState -RepoPath $_.FullName })

    $hasLock = Test-Path $LockPath
    $previousByName = @{}
    if ($hasLock) {
        try {
            $lock = Get-Content $LockPath -Raw | ConvertFrom-Json
            foreach ($repo in @($lock.src_repositories)) {
                if ($repo.name) {
                    $previousByName[$repo.name] = $repo
                }
            }
        }
        catch {
            $hasLock = $false
        }
    }

    $changed = @()
    foreach ($state in $repoStates) {
        if (-not $previousByName.ContainsKey($state.name)) {
            $changed += $state.name
            continue
        }

        $previous = $previousByName[$state.name]
        if ($state.vcs -eq "git") {
            $prevRevision = [string] $previous.revision
            $prevFingerprint = [string] $previous.fingerprint
            if ([string]::IsNullOrWhiteSpace($prevRevision) -or $prevRevision -ne $state.revision -or [string]::IsNullOrWhiteSpace($prevFingerprint) -or $prevFingerprint -ne $state.fingerprint) {
                $changed += $state.name
            }
            continue
        }

        $prevFingerprint = [string] $previous.fingerprint
        if ([string]::IsNullOrWhiteSpace($prevFingerprint) -or $prevFingerprint -ne $state.fingerprint) {
            $changed += $state.name
        }
    }

    return [PSCustomObject]@{
        HasLock = $hasLock
        ChangedRepos = @($changed)
        RepoStates = @($repoStates)
    }
}

function Resolve-SelectedsrcRepos {
    param(
        [string] $srcRoot,
        [string] $Mode,
        [string[]] $RequestedNames
    )

    if (-not (Test-Path $srcRoot)) {
        return @()
    }

    $repos = @(Get-ChildItem -Path $srcRoot -Directory)
    if ($repos.Count -eq 0) {
        return @()
    }

    if ($Mode -eq "all") {
        return @($repos | ForEach-Object { $_.Name })
    }

    if ($Mode -eq "selected") {
        if (-not $RequestedNames -or $RequestedNames.Count -eq 0) {
            throw "srcRepoMode=selected requires at least one value in srcRepoNames."
        }

        $available = @($repos | ForEach-Object { $_.Name })
        $missing = @($RequestedNames | Where-Object { $_ -notin $available })
        if ($missing.Count -gt 0) {
            throw "Selected src repositories not found: $($missing -join ', ')"
        }

        return @($RequestedNames)
    }

    # latest mode
    $latest = $repos | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if ($null -eq $latest) {
        return @()
    }

    return @($latest.Name)
}
$CurrentsrcRepoStates = @()
if ($srcRepoMode -eq "changed-only") {
    $changedResult = Get-ChangedsrcRepos -srcRoot $srcRootPath -LockPath $SkillLockPath
    $CurrentsrcRepoStates = @($changedResult.RepoStates)

    if (-not $changedResult.HasLock) {
        $ChangedOnlyFallbackToLatest = $true
        $SelectedsrcRepos = @(Resolve-SelectedsrcRepos -srcRoot $srcRootPath -Mode "latest" -RequestedNames @())
    }
    else {
        $SelectedsrcRepos = @($changedResult.ChangedRepos)
    }
}
else {
    $SelectedsrcRepos = @(Resolve-SelectedsrcRepos -srcRoot $srcRootPath -Mode $srcRepoMode -RequestedNames $srcRepoNames)
    if (Test-Path $srcRootPath) {
        $CurrentsrcRepoStates = @(Get-ChildItem -Path $srcRootPath -Directory | ForEach-Object { Get-srcRepoState -RepoPath $_.FullName })
    }
}

if ($AllowMultiHub -and $SecondaryMinScore -lt $PrimaryMinScore) {
    throw "SecondaryMinScore must be greater than or equal to PrimaryMinScore when AllowMultiHub is enabled."
}

# ============================================================================
# BMAD STYLE TEMPLATES (TOOL-NEUTRAL)
# ============================================================================

$SKILL_ROUTER_TEMPLATE = @'
---
name: {SKILL_NAME}
description: '{SKILL_DESCRIPTION}'
---

Follow the instructions in ./workflow.md.
'@

$WORKFLOW_TEMPLATE = @'
# {TITLE}

## Purpose

{DESCRIPTION}

This sub-hub is optimized for multi-tool usage (Gemini CLI, Antigravity, GitHub Copilot) with minimal context overhead.

## Loading Strategy

1. Start with `skills-manifest.json` to understand scope and top triggers.
2. Narrow by user intent and trigger keywords first.
3. Load only relevant lines from `skills-catalog.ndjson`.
4. Avoid loading the entire catalog unless explicitly needed.

## Execution Rule (Mandatory)

1. Do not stop at `SKILL.md`, `workflow.md`, or `skills-manifest.json`.
2. After filtering candidate entries from `skills-catalog.ndjson`, open at least one concrete skill file from the `path` field.
3. If multiple candidates exist, open the best match first, then continue with implementation using that skill.
4. If a `path` under `AI-skills-bank/src/` is missing, report it explicitly and request re-aggregation with src repos included.

## Files

- `skills-manifest.json`: Summary, counts, and top triggers.
- `skills-index.json`: Lightweight index for quick filtering before deep reads.
- `skills-catalog.ndjson`: One JSON object per skill (stream-friendly).

## Recommended Use Cases

- {USE_CASE_1}
- {USE_CASE_2}
- {USE_CASE_3}

## Quick Trigger Hints

{TRIGGER_HINTS}

## Data Contract

Each index item contains:

```json
{"id":"...","triggers":["..."],"src":"...","primary_hub":"...","is_primary":true,"match_score":8}
```

Each NDJSON item contains:

```json
{"id":"...","description":"...","path":"...","triggers":["..."],"src":"...","primary_hub":"...","assigned_hubs":["..."],"match_score":8,"is_primary":true}
```

## Notes

- Keep this workflow lightweight.
- Prefer selective reads from the catalog.
- This mirrors BMAD's router pattern (`SKILL.md` delegates to `workflow.md`).
'@

# ============================================================================
# SKILL DEFINITIONS (TAXONOMY)
# ============================================================================

$SUB_HUB_DEFINITIONS = @{
    "general" = @{
        "misc" = @{
            keywords = @("skill")
            negative_keywords = @()
            description = "General fallback skills that do not confidently match a specialized sub-hub"
            best_for = @(
                "Capturing uncategorized capabilities",
                "Manual review and future taxonomy refinement",
                "Ensuring zero skill loss during aggregation"
            )
        }
    }

    "programming" = @{
        "typescript" = @{
            keywords = @("typescript", "tsconfig", "tsx", "type-system", "generics", "type-safe")
            anchor_keywords = @("typescript", "tsconfig", "tsx")
            negative_keywords = @("python", "golang", "rust", "java", "postgres", "mongodb", "redis", "kubernetes")
            description = "TypeScript language expertise: types, patterns, advanced features, configuration, and best practices"
            best_for = @(
                "Building type-safe applications",
                "Creating reusable component libraries",
                "Implementing complex generic patterns"
            )
        }
        "python" = @{
            keywords = @("python", "py", "django", "fastapi", "async", "asyncio")
            anchor_keywords = @("python", "py", "django", "fastapi")
            negative_keywords = @("typescript", "golang", "rust", "java")
            description = "Python development: patterns, async, frameworks, and modern Python 3.10+ features"
            best_for = @(
                "Building REST APIs and backends",
                "Data processing and scripting",
                "Async application design"
            )
        }
        "golang" = @{
            keywords = @("golang", "go", "grpc", "concurrency", "channels")
            anchor_keywords = @("golang", "go", "grpc")
            negative_keywords = @("typescript", "python", "rust", "java")
            description = "Go programming: concurrency patterns, microservices, and system programming"
            best_for = @(
                "Building high-performance services",
                "Concurrent system design",
                "Microservices architecture"
            )
        }
        "rust" = @{
            keywords = @("rust", "cargo", "ownership", "lifetimes", "unsafe")
            anchor_keywords = @("rust", "cargo", "ownership")
            negative_keywords = @("typescript", "python", "golang", "java")
            description = "Rust: memory safety, performance, systems programming, and async patterns"
            best_for = @(
                "Building fast, memory-safe systems",
                "Systems programming",
                "WebAssembly applications"
            )
        }
        "java" = @{
            keywords = @("java", "spring", "maven", "jvm", "virtual-threads")
            anchor_keywords = @("java", "spring", "jvm", "maven")
            negative_keywords = @("typescript", "python", "golang", "rust")
            description = "Java development: Spring ecosystem, modern Java features, and JVM optimization"
            best_for = @(
                "Enterprise application development",
                "Building scalable backends",
                "Integration with existing systems"
            )
        }
    }
    
    "frontend" = @{
        "react-nextjs" = @{
            keywords = @("react", "nextjs", "jsx", "hooks", "server-components", "app-router")
            negative_keywords = @("postgres", "mongodb", "redis", "sql")
            description = "React and Next.js: components, hooks, server-side rendering, and performance optimization"
            best_for = @(
                "Building modern web applications",
                "Full-stack development with Next.js",
                "Server and client component patterns"
            )
        }
        "ui-ux" = @{
            keywords = @("ui", "ux", "design", "designer", "wireframe", "prototype", "accessibility", "usability", "design-system", "figma", "interaction")
            anchor_keywords = @("ui", "ux", "design-system", "wireframe", "accessibility")
            negative_keywords = @("kubernetes", "docker", "postgres", "mongodb", "redis")
            description = "UI/UX design: interface design, wireframes, design systems, accessibility, and interaction patterns"
            best_for = @(
                "Designing intuitive user interfaces",
                "Building and maintaining design systems",
                "Improving usability and accessibility"
            )
        }
        "web-basics" = @{
            keywords = @("html", "css", "javascript", "dom", "responsive", "web-standards")
            negative_keywords = @("postgres", "mongodb", "redis", "kubernetes")
            description = "Web fundamentals: HTML, CSS, JavaScript, accessibility, and web standards"
            best_for = @(
                "Understanding web standards",
                "Building accessible UIs",
                "CSS architecture and performance"
            )
        }
    }
    
    "backend" = @{
        "api-design" = @{
            keywords = @("api", "rest", "graphql", "openapi", "swagger", "pagination")
            negative_keywords = @("react", "nextjs", "html", "css")
            description = "API design: REST, GraphQL, and best practices for scalable web services"
            best_for = @(
                "Designing robust APIs",
                "GraphQL schema design",
                "API versioning and deprecation"
            )
        }
        "databases" = @{
            keywords = @("database", "sql", "postgres", "mongodb", "redis", "nosql", "orm")
            negative_keywords = @("react", "nextjs", "html", "css", "typescript")
            description = "Database expertise: SQL, NoSQL, schema design, and query optimization"
            best_for = @(
                "Database schema design",
                "Query optimization",
                "Choosing the right database"
            )
        }
    }
    
    "devops" = @{
        "docker-k8s" = @{
            keywords = @("docker", "kubernetes", "k8s", "container", "orchestration", "helm")
            negative_keywords = @("react", "nextjs", "html", "css")
            description = "Container orchestration: Docker, Kubernetes, and deployment strategies"
            best_for = @(
                "Containerizing applications",
                "Scaling with Kubernetes",
                "Managing microservices"
            )
        }
        "cloud" = @{
            keywords = @("aws", "gcp", "azure", "cloudflare", "lambda", "serverless")
            negative_keywords = @("react", "nextjs", "html", "css")
            description = "Cloud platforms: AWS, GCP, Azure, and serverless architecture"
            best_for = @(
                "Cloud infrastructure design",
                "Serverless applications",
                "Cost optimization"
            )
        }
    }

    "business" = @{
        "saas" = @{
            keywords = @("saas", "pricing", "revenue", "arr", "mrr", "churn", "ltv", "cac", "unit-economics", "go-to-market", "gtm", "market-sizing", "tam", "sam", "som", "roadmap", "startup")
            anchor_keywords = @("saas", "arr", "mrr", "unit-economics", "go-to-market", "market-sizing")
            negative_keywords = @("react", "nextjs", "html", "css", "kubernetes", "docker")
            description = "Business and SaaS strategy: pricing, growth metrics, unit economics, market sizing, and go-to-market planning"
            best_for = @(
                "Evaluating SaaS business health",
                "Designing pricing and growth strategies",
                "Planning market entry and product strategy"
            )
        }
        "product-strategy" = @{
            keywords = @("product", "strategy", "roadmap", "prd", "stakeholder", "discovery", "market", "positioning", "vision", "prioritization", "alignment", "wds")
            anchor_keywords = @("product", "strategy", "roadmap", "prd", "wds")
            negative_keywords = @("react", "nextjs", "html", "css", "kubernetes", "docker")
            description = "Product and business strategy: discovery, roadmaps, prioritization, stakeholder alignment, and strategic planning"
            best_for = @(
                "Defining product strategy and direction",
                "Building roadmaps and prioritization frameworks",
                "Aligning teams around business goals"
            )
        }
    }

    "marketing" = @{
        "strategy" = @{
            keywords = @("marketing", "strategy", "brand", "positioning", "customer", "audience", "market-analysis", "competitive-analysis", "go-to-market")
            anchor_keywords = @("marketing-strategy", "brand-strategy", "positioning")
            negative_keywords = @("email", "seo", "content", "social", "copywrite", "html", "css")
            description = "Marketing strategy: brand positioning, customer acquisition, market analysis, and GTM planning"
            best_for = @(
                "Developing marketing strategies",
                "Brand positioning",
                "Customer acquisition planning"
            )
        }
        "content" = @{
            keywords = @("content", "copywriting", "seo", "blog", "article", "writing", "editorial", "keyword", "search-engine")
            anchor_keywords = @("seo", "content-marketing", "copywriting")
            negative_keywords = @("email", "social", "video", "design", "html", "css")
            description = "Content marketing & SEO: copywriting, blog strategy, search optimization, and editorial best practices"
            best_for = @(
                "Creating SEO-optimized content",
                "Building blog strategies",
                "Improving search rankings"
            )
        }
        "email" = @{
            keywords = @("email", "newsletter", "email-marketing", "campaigns", "subscribers", "automation", "segmentation")
            anchor_keywords = @("email-marketing", "email-campaigns", "newsletter")
            negative_keywords = @("social", "seo", "video", "design")
            description = "Email marketing: campaigns, automation, segmentation, and subscriber engagement strategies"
            best_for = @(
                "Building email campaigns",
                "Marketing automation",
                "List segmentation"
            )
        }
        "social" = @{
            keywords = @("social", "twitter", "linkedin", "instagram", "tiktok", "youtube", "content-calendar", "engagement", "viral", "publisher", "posting", "article-publisher")
            anchor_keywords = @("social-media", "social-marketing", "twitter-strategy")
            negative_keywords = @("email", "seo", "copywrite")
            description = "Social media marketing: strategy, content distribution, engagement, and multi-platform publishing"
            best_for = @(
                "Social media strategy",
                "Content distribution",
                "Engagement optimization"
            )
        }
    }

    "security" = @{
        "core" = @{
            keywords = @("security", "authentication", "authorization", "oauth", "jwt", "encryption", "tls", "ssl", "vulnerability", "secure")
            anchor_keywords = @("security", "authentication", "oauth", "jwt")
            negative_keywords = @("marketing", "seo", "newsletter", "ui", "css")
            description = "Application security: authentication, authorization, encryption, and vulnerability hardening"
            best_for = @(
                "Designing secure authentication flows",
                "Implementing encryption and key handling",
                "Reducing common web security risks"
            )
        }
    }

    "testing" = @{
        "automation" = @{
            keywords = @("testing", "test", "unit-test", "integration-test", "e2e", "qa", "cypress", "playwright", "vitest", "jest", "automation")
            anchor_keywords = @("testing", "test", "tdd", "test-driven-development", "unit-test", "integration-test", "e2e", "qa")
            negative_keywords = @("marketing", "seo", "newsletter")
            description = "Software testing: unit, integration, E2E, and automated quality workflows"
            best_for = @(
                "Building reliable automated test suites",
                "Designing integration and end-to-end tests",
                "Improving test coverage and quality gates"
            )
        }
    }

    "ai" = @{
        "llm-agents" = @{
            keywords = @("llm", "gpt", "prompt", "rag", "embedding", "vector", "agent", "transformer", "chatbot", "fine-tuning")
            anchor_keywords = @("llm", "gpt", "rag", "agent")
            negative_keywords = @("newsletter", "seo", "css", "html")
            description = "AI engineering: LLM prompting, RAG pipelines, embeddings, and autonomous agent patterns"
            best_for = @(
                "Building LLM-powered assistants",
                "Designing RAG and retrieval workflows",
                "Improving prompt and agent reliability"
            )
        }
        "automation" = @{
            keywords = @("automation", "automate", "automated", "automates", "workflow", "orchestration", "orchestrate", "orchestrator", "agentic", "autonomous", "n8n", "zapier", "make", "langgraph", "crewai", "autogen", "tool-calling", "pipeline")
            anchor_keywords = @("automation", "automated", "workflow", "orchestration", "orchestrate", "orchestrator", "agentic", "n8n", "zapier", "langgraph", "crewai")
            negative_keywords = @("unit-test", "integration-test", "e2e", "qa", "marketing", "newsletter")
            description = "AI automation: agentic workflows, orchestration pipelines, and tool-connected process automation"
            best_for = @(
                "Designing agentic automation workflows",
                "Integrating tools and orchestration pipelines",
                "Automating multi-step AI operations"
            )
        }
    }

    "productivity" = @{
        "workflow-automation" = @{
            keywords = @("productivity", "workflow", "automation", "task-management", "project-management", "agile", "scrum", "kanban", "notion", "planning")
            anchor_keywords = @("workflow", "automation", "productivity")
            negative_keywords = @("encryption", "oauth", "jwt")
            description = "Productivity systems: workflow automation, project orchestration, and delivery optimization"
            best_for = @(
                "Automating repetitive delivery tasks",
                "Structuring team workflows",
                "Improving execution velocity"
            )
        }
    }

    "mobile" = @{
        "cross-platform" = @{
            keywords = @("mobile", "android", "ios", "react-native", "flutter", "swift", "kotlin", "mobile-app")
            anchor_keywords = @("mobile", "android", "ios", "react-native", "flutter")
            negative_keywords = @("seo", "newsletter", "email-marketing")
            description = "Mobile development: iOS, Android, and cross-platform application engineering"
            best_for = @(
                "Building native and cross-platform apps",
                "Designing mobile architecture",
                "Improving mobile UX and performance"
            )
        }
    }
}

$CATEGORY_GAP_PATTERNS = @{
    "business" = @("saas", "pricing", "revenue", "arr", "mrr", "churn", "ltv", "cac", "unit-economics", "market-sizing", "tam", "sam", "som", "go-to-market", "startup")
    "product-strategy" = @("product", "roadmap", "prd", "stakeholder", "prioritization", "discovery", "strategy", "wds")
    "ui-ux" = @("ui", "ux", "wireframe", "prototype", "design-system", "accessibility", "usability", "figma")
    "marketing" = @("marketing", "seo", "email", "newsletter", "campaign", "audience", "publisher", "social-media", "content-marketing")
    "security" = @("security", "auth", "authentication", "authorization", "oauth", "jwt", "encryption", "tls", "ssl", "vulnerability")
    "testing" = @("test", "testing", "unit-test", "integration-test", "e2e", "qa", "cypress", "vitest", "jest")
    "ai-llm" = @("llm", "gpt", "prompt", "embedding", "rag", "agent", "transformer", "chatbot")
    "data-science" = @("machine-learning", "ml", "data-science", "pandas", "numpy", "tensorflow", "pytorch", "analytics")
    "mobile" = @("mobile", "android", "ios", "flutter", "react-native", "swift", "kotlin")
}

$CATEGORY_PATTERN_TO_MAIN_HUB = @{
    "business" = "business"
    "product-strategy" = "business"
    "ui-ux" = "frontend"
    "marketing" = "marketing"
    "security" = "security"
    "testing" = "testing"
    "ai-llm" = "ai"
    "data-science" = "data-science"
    "mobile" = "mobile"
}

$MANUAL_HUB_OVERRIDES = @{
    # 1. MISPLACED SKILLS (Fixing misclassifications)
    "popup-cro"                    = @{ main = "marketing"; sub = "content"; score = 100 }
    "cro-optimization"             = @{ main = "marketing"; sub = "content"; score = 100 }
    "lightning-architecture-review" = @{ main = "backend"; sub = "api-design"; score = 100 }
    "debugging-strategies"         = @{ main = "testing"; sub = "automation"; score = 100 }
    "marketing-ideas"              = @{ main = "marketing"; sub = "strategy"; score = 100 }
    "animejs-animation"            = @{ main = "frontend"; sub = "ui-ux"; score = 100 }
    "identify-assumptions-new"      = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "micro-saas-launcher"          = @{ main = "business"; sub = "saas"; score = 100 }
    "scroll-experience"            = @{ main = "frontend"; sub = "ui-ux"; score = 100 }
    "sred-work-summary"            = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "go-mode"                      = @{ main = "ai"; sub = "automation"; score = 100 }

    # 2. DUPLICATES (Consolidating to primary hub)
    "email-sequence"               = @{ main = "marketing"; sub = "email"; score = 100 }
    "revops"                       = @{ main = "marketing"; sub = "email"; score = 100 }
    "churn-prevention"             = @{ main = "marketing"; sub = "email"; score = 100 }
    "content-strategy"             = @{ main = "marketing"; sub = "content"; score = 100 }
    "positioning-ideas"            = @{ main = "marketing"; sub = "strategy"; score = 100 }
    "positioning-basics"           = @{ main = "marketing"; sub = "strategy"; score = 100 }
    "figma-create-design-system-rules" = @{ main = "frontend"; sub = "ui-ux"; score = 100 }
    "nerdzao-elite"                = @{ main = "frontend"; sub = "ui-ux"; score = 100 }
    "gtm-strategy"                 = @{ main = "marketing"; sub = "strategy"; score = 100 }
    "gtm-motions"                  = @{ main = "marketing"; sub = "strategy"; score = 100 }

    # 3. GENERAL TO SPECIFIC (Moving misc to specialized hubs)
    "mermaid-expert"               = @{ main = "backend"; sub = "api-design"; score = 100 }
    "microservices-patterns"       = @{ main = "backend"; sub = "api-design"; score = 100 }
    "threejs-animation"            = @{ main = "frontend"; sub = "ui-ux"; score = 100 }
    "threejs-fundamentals"         = @{ main = "frontend"; sub = "ui-ux"; score = 100 }
    "threejs-geometry"             = @{ main = "frontend"; sub = "ui-ux"; score = 100 }
    "threejs-interaction"          = @{ main = "frontend"; sub = "ui-ux"; score = 100 }
    "threejs-lighting"             = @{ main = "frontend"; sub = "ui-ux"; score = 100 }
    "threejs-loaders"              = @{ main = "frontend"; sub = "ui-ux"; score = 100 }
    "threejs-materials"            = @{ main = "frontend"; sub = "ui-ux"; score = 100 }
    "threejs-postprocessing"       = @{ main = "frontend"; sub = "ui-ux"; score = 100 }
    "threejs-skills"               = @{ main = "frontend"; sub = "ui-ux"; score = 100 }
    "threejs-textures"             = @{ main = "frontend"; sub = "ui-ux"; score = 100 }
    "makepad-animation"            = @{ main = "programming"; sub = "rust"; score = 100 }
    "makepad-basics"               = @{ main = "programming"; sub = "rust"; score = 100 }
    "makepad-deployment"           = @{ main = "programming"; sub = "rust"; score = 100 }
    "makepad-dsl"                  = @{ main = "programming"; sub = "rust"; score = 100 }
    "makepad-event-action"         = @{ main = "programming"; sub = "rust"; score = 100 }
    "makepad-font"                 = @{ main = "programming"; sub = "rust"; score = 100 }
    "makepad-layout"               = @{ main = "programming"; sub = "rust"; score = 100 }
    "makepad-platform"             = @{ main = "programming"; sub = "rust"; score = 100 }
    "makepad-reference"            = @{ main = "programming"; sub = "rust"; score = 100 }
    "makepad-shaders"              = @{ main = "programming"; sub = "rust"; score = 100 }
    "makepad-splash"               = @{ main = "programming"; sub = "rust"; score = 100 }
    "makepad-widgets"              = @{ main = "programming"; sub = "rust"; score = 100 }
    
    # 4. LEGACY (Existing)
    "iterate-pr"                   = @{ main = "ai"; sub = "automation"; score = 100 }
}

$EXCLUDE_CATEGORY_PATTERNS = [ordered]@{
    "games" = @("game", "games", "gaming", "gameplay", "unity", "unreal", "godot")
    "law-legal" = @("law", "legal", "lawyer", "attorney", "litigation", "court", "jurisdiction", "legal system", "legal systems")
    "medicine-medical" = @("medicine", "medical", "clinical", "healthcare", "diagnosis", "patient", "hospital")
    "pharmacy" = @("pharmacy", "pharmaceutical", "pharmacology", "drug discovery", "medication")
    "biology" = @("biology", "biological", "genomics", "protein", "cell biology", "bioinformatics")
    "chemistry" = @("chemistry", "chemical", "molecule", "molecular", "organic chemistry", "chemical reaction")
}

$DEFAULT_EXCLUDE_CATEGORIES = @(
    "games",
    "law-legal",
    "medicine-medical",
    "pharmacy",
    "biology",
    "chemistry",
    "llm-from-scratch"
)

$LLM_FROM_SCRATCH_PATTERNS = @(
    "from scratch llm",
    "build llm from scratch",
    "train llm from scratch",
    "pretrain llm",
    "pre-train llm",
    "llm pretraining",
    "tokenizer training",
    "train transformer from scratch"
)

$APPLIED_AI_ALLOW_PATTERNS = @(
    "applied ai",
    "rag",
    "retrieval",
    "prompt",
    "agent",
    "tool calling",
    "inference",
    "fine-tuning",
    "embedding",
    "vector",
    "evaluation"
)

$script:ExcludedSkillStats = @{}
$script:ActiveExcludeCategoryPatterns = [ordered]@{}
$script:EnableLlmFromScratchExclusion = $false
$script:EffectiveExcludeCategories = @()

function Initialize-ExcludePolicy {
    $requested = @()

    if (-not $NoCategoryExclusions) {
        if ($ExcludeCategories -and $ExcludeCategories.Count -gt 0) {
            foreach ($item in $ExcludeCategories) {
                if (-not [string]::IsNullOrWhiteSpace($item)) {
                    $parts = @($item -split ',')
                    foreach ($part in $parts) {
                        if (-not [string]::IsNullOrWhiteSpace($part)) {
                            $requested += $part.Trim().ToLower()
                        }
                    }
                }
            }
        }
        else {
            $requested = @($DEFAULT_EXCLUDE_CATEGORIES)
        }
    }

    $requested = @($requested | Select-Object -Unique)
    $script:EffectiveExcludeCategories = @($requested)

    $script:ActiveExcludeCategoryPatterns = [ordered]@{}
    foreach ($key in $EXCLUDE_CATEGORY_PATTERNS.Keys) {
        if ($requested -contains $key) {
            $script:ActiveExcludeCategoryPatterns[$key] = $EXCLUDE_CATEGORY_PATTERNS[$key]
        }
    }

    $script:EnableLlmFromScratchExclusion = ($requested -contains "llm-from-scratch")

    $known = @($EXCLUDE_CATEGORY_PATTERNS.Keys + @("llm-from-scratch"))
    $unknown = @($requested | Where-Object { $_ -notin $known })
    if ($unknown.Count -gt 0) {
        Write-Host "[WARN] Unknown exclude categories ignored: $($unknown -join ', ')" -ForegroundColor Yellow
    }

    if ($script:EffectiveExcludeCategories.Count -gt 0) {
        Write-Host "[INFO] Exclusion policy active: $($script:EffectiveExcludeCategories -join ', ')" -ForegroundColor Yellow
    }
    else {
        Write-Host "[INFO] Exclusion policy active: none" -ForegroundColor Yellow
    }
}

Initialize-ExcludePolicy

# ============================================================================
# MAIN AGGREGATION LOGIC
# ============================================================================

function Get-CategoryGapSignals {
    param(
        [array] $Skills,
        [hashtable] $Patterns,
        [int] $MinCount
    )

    $signals = @()
    if (-not $Skills -or $Skills.Count -eq 0) {
        return $signals
    }

    foreach ($category in $Patterns.Keys) {
        $count = 0
        $samples = @()

        foreach ($skill in $Skills) {
            $skillText = ("{0} {1} {2} {3}" -f $skill.id, $skill.description, $skill.path, ($skill.triggers -join " ")).ToLower()
            $matched = $false

            foreach ($keyword in $Patterns[$category]) {
                if ($skillText -match [regex]::Escape($keyword)) {
                    $matched = $true
                    break
                }
            }

            if ($matched) {
                $count++
                if ($samples.Count -lt 3) {
                    $samples += $skill.id
                }
            }
        }

        if ($count -ge $MinCount) {
            $signals += [PSCustomObject]@{
                category = $category
                count = $count
                threshold = $MinCount
                sample_skills = @($samples)
            }
        }
    }

    return @($signals | Sort-Object count -Descending)
}

function Add-ExclusionStat {
    param([string] $Category)

    if ([string]::IsNullOrWhiteSpace($Category)) {
        return
    }

    if (-not $script:ExcludedSkillStats.ContainsKey($Category)) {
        $script:ExcludedSkillStats[$Category] = 0
    }
    $script:ExcludedSkillStats[$Category] += 1
}

function Get-ExcludedCategory {
    param(
        [string] $Id,
        [string] $Description,
        [string] $Path,
        [array] $Triggers
    )

    $rawText = "{0} {1} {2} {3}" -f $Id, $Description, $Path, (@($Triggers) -join " ")
    $normalized = (($rawText.ToLower() -replace "[^a-z0-9]+", " ").Trim())
    $text = " $normalized "

    foreach ($category in $script:ActiveExcludeCategoryPatterns.Keys) {
        foreach ($keyword in $script:ActiveExcludeCategoryPatterns[$category]) {
            $normalizedKeyword = (($keyword.ToLower() -replace "[^a-z0-9]+", " ").Trim())
            if (-not [string]::IsNullOrWhiteSpace($normalizedKeyword) -and $text.Contains(" $normalizedKeyword ")) {
                return $category
            }
        }
    }

    if ($script:EnableLlmFromScratchExclusion) {
        $hasScratchSignal = $false
        foreach ($pattern in $LLM_FROM_SCRATCH_PATTERNS) {
            $normalizedPattern = (($pattern.ToLower() -replace "[^a-z0-9]+", " ").Trim())
            if (-not [string]::IsNullOrWhiteSpace($normalizedPattern) -and $text.Contains(" $normalizedPattern ")) {
                $hasScratchSignal = $true
                break
            }
        }

        if ($hasScratchSignal) {
            $hasAppliedSignal = $false
            foreach ($pattern in $APPLIED_AI_ALLOW_PATTERNS) {
                $normalizedPattern = (($pattern.ToLower() -replace "[^a-z0-9]+", " ").Trim())
                if (-not [string]::IsNullOrWhiteSpace($normalizedPattern) -and $text.Contains(" $normalizedPattern ")) {
                    $hasAppliedSignal = $true
                    break
                }
            }

            if (-not $hasAppliedSignal) {
                return "llm-from-scratch"
            }
        }
    }

    return $null
}

function Get-Skillsrc {
    param([string] $Path)
    
    if ($Path -match '_bmad') {
        return "internal:BMad"
    }
    elseif ($Path -match 'AI-skills-bank[\\/]src[\\/]([^\\/]+)') {
        return "external:$($matches[1])"
    }
    elseif ($Path -match 'antigravity-awesome-skills') {
        return "external:antigravity-awesome-skills"
    }
    else {
        return "unknown:$(Split-Path -Leaf $Path)"
    }
}

function Convert-ToRepoRelativePath {
    param([string] $Path)

    if ([string]::IsNullOrWhiteSpace($Path)) {
        return $Path
    }

    $resolvedPath = $Path
    try {
        $resolvedPath = (Resolve-Path -LiteralPath $Path -ErrorAction Stop).Path
    }
    catch {
        # Keep the original value if the path cannot be resolved (already relative or external reference).
        $resolvedPath = $Path
    }

    if ($RepoRoot -and $resolvedPath.StartsWith($RepoRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        $resolvedPath = $resolvedPath.Substring($RepoRoot.Length).TrimStart('\', '/')
    }

    return ($resolvedPath -replace '\\', '/')
}

function Extract-FieldFromFrontmatter {
    param(
        [string] $Content,
        [string] $FieldName
    )

    $pattern = "(?m)^" + [regex]::Escape($FieldName) + ":\s*(.+)$"
    $match = [regex]::Match($Content, $pattern)
    if ($match.Success) {
        return $match.Groups[1].Value.Trim().Trim("'").Trim('"')
    }

    return $null
}

function Build-TriggersFromId {
    param([string] $Id)

    $parts = @($Id -split '-') | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
    return @($parts | Select-Object -First 5)
}

function Load-SkillsFromFiles {
    param([array] $Roots)

    $skills = @()
    $srcRootResolved = $null
    if ($srcRootPath -and (Test-Path $srcRootPath)) {
        $srcRootResolved = (Resolve-Path -LiteralPath $srcRootPath).Path
    }

    foreach ($root in $Roots) {
        if (-not (Test-Path $root)) {
            continue
        }

        $skillFiles = Get-ChildItem -Path $root -Filter "SKILL.md" -Recurse -File
        foreach ($skillFile in $skillFiles) {
            if ($srcRootResolved) {
                if ($skillFile.FullName.StartsWith($srcRootResolved, [System.StringComparison]::OrdinalIgnoreCase)) {
                    $relativesrcPath = $skillFile.FullName.Substring($srcRootResolved.Length).TrimStart('\', '/')
                    $repoName = @($relativesrcPath -split '[\\/]')[0]
                    if (-not [string]::IsNullOrWhiteSpace($repoName) -and $RestrictsrcRepos -and ($repoName -notin $SelectedsrcRepos)) {
                        continue
                    }
                }
            }

            $content = Get-Content $skillFile.FullName -Raw
            $id = Extract-FieldFromFrontmatter -Content $content -FieldName "name"
            $description = Extract-FieldFromFrontmatter -Content $content -FieldName "description"

            if ([string]::IsNullOrWhiteSpace($id)) {
                $id = Split-Path -Leaf (Split-Path -Parent $skillFile.FullName)
            }

            if ([string]::IsNullOrWhiteSpace($description)) {
                $description = "Skill extracted from $($skillFile.FullName)"
            }

            $skillPath = Convert-ToRepoRelativePath -Path $skillFile.FullName
            $excludedCategory = Get-ExcludedCategory -Id $id -Description $description -Path $skillPath -Triggers @(Build-TriggersFromId -Id $id)
            if ($excludedCategory) {
                Add-ExclusionStat -Category $excludedCategory
                continue
            }

            $skills += [PSCustomObject]@{
                id = $id
                description = $description
                path = $skillPath
                triggers = @(Build-TriggersFromId -Id $id)
                src = Get-Skillsrc -Path $skillFile.FullName
            }
        }
    }

    return $skills
}

function Deduplicate-Skills {
    param([array] $Skills)
    
    $seen = @{}
    $unique = @()
    
    foreach ($skill in $Skills) {
        $key = $skill.id
        if (-not $seen[$key]) {
            $seen[$key] = $true
            $unique += $skill
        }
    }
    
    return $unique
}

function Get-Tokens {
    param([string] $Text)

    if ([string]::IsNullOrWhiteSpace($Text)) {
        return @()
    }

    return ([regex]::Matches($Text.ToLower(), "[a-z0-9]+") | ForEach-Object { $_.Value })
}

function Load-SemanticClassifications {
    param([string] $FilePath)

    if (-not (Test-Path $FilePath)) {
        Write-Warning "Semantic classifications file not found: $FilePath"
        return @{}
    }

    try {
        $json = Get-Content -Path $FilePath -Raw -ErrorAction Stop | ConvertFrom-Json -ErrorAction Stop
        $lookup = @{}
        
        if ($json -is [array]) {
            foreach ($item in $json) {
                if ($item.skill_id) {
                    $lookup[$item.skill_id] = $item
                }
            }
        }
        elseif ($json -is [object]) {
            $lookup[$json.skill_id] = $json
        }
        
        Write-Host "[✓] Loaded semantic classifications: $($lookup.Count) skills" -ForegroundColor Green
        return $lookup
    }
    catch {
        Write-Warning "Failed to load semantic classifications: $_"
        return @{}
    }
}

function Get-SemanticScoreForSkill {
    param(
        [PSCustomObject] $Skill,
        [hashtable] $SemanticLookup,
        [hashtable] $SubHubDefs
    )

    if (-not $SemanticLookup -or $SemanticLookup.Count -eq 0) {
        return $null
    }

    $skillId = $Skill.id.ToLower()
    $classification = $SemanticLookup[$skillId]
    
    if (-not $classification) {
        return $null
    }

    # Map semantic hub to our internal hub structure
    $primaryHub = $classification.primary_hub
    if ([string]::IsNullOrWhiteSpace($primaryHub)) {
        return $null
    }

    # Find best matching internal hub for semantic classification
    $bestMatch = $null
    $bestScore = 0

    foreach ($mainHub in $SubHubDefs.Keys) {
        foreach ($subHub in $SubHubDefs[$mainHub].Keys) {
            $subHubLower = $subHub.ToLower()
            $primaryHubLower = $primaryHub.ToLower()

            # Direct match is best (10 points)
            if ($subHubLower -eq $primaryHubLower) {
                $bestMatch = @{
                    main = $mainHub
                    sub = $subHub
                    score = 10
                }
                break
            }
            
            # Partial match (5-8 points based on overlap)
            if ($subHubLower.Contains($primaryHubLower) -or $primaryHubLower.Contains($subHubLower)) {
                if (8 -gt $bestScore) {
                    $bestScore = 8
                    $bestMatch = @{
                        main = $mainHub
                        sub = $subHub
                        score = $bestScore
                    }
                }
            }
        }
        if ($bestMatch -and $bestMatch.score -eq 10) { break }
    }

    return $bestMatch
}

function Blend-KeywordAndSemanticScores {
    param(
        [int] $KeywordScore,
        [int] $SemanticScore,
        [double] $WeightFactor
    )

    # WeightFactor = 0.6 means: 60% semantic, 40% keyword
    $keywordWeight = 1.0 - $WeightFactor
    
    # Normalize scores to 0-10 range
    $normalizedKeyword = [Math]::Min(10, $KeywordScore / 1.5)  # typical keyword score 0-15
    $normalizedSemantic = $SemanticScore  # already 0-10
    
    $blended = ($normalizedKeyword * $keywordWeight) + ($normalizedSemantic * $WeightFactor)
    return [Math]::Round($blended, 1)
}

function Get-ScoreForSubHub {
    param(
        [PSCustomObject] $Skill,
        [hashtable] $SubHubRule,
        [string] $SubHubName
    )

    $idLower = $Skill.id.ToLower()
    $descLower = $Skill.description.ToLower()
    $pathLower = $Skill.path.ToLower()
    $triggerLower = (@($Skill.triggers) | ForEach-Object { $_.ToLower() }) -join " "
    $fullText = "$idLower $descLower $triggerLower"
    $tokens = @(Get-Tokens -Text $fullText)
    $tokenSet = @{}
    foreach ($t in $tokens) { $tokenSet[$t] = $true }

    $score = 0

    foreach ($kw in @($SubHubRule.keywords)) {
        $kwLower = $kw.ToLower()
        $isShortKeyword = ($kwLower.Length -lt 3)

        if ($tokenSet.ContainsKey($kwLower)) {
            $score += 4
            continue
        }

        if (-not $isShortKeyword -and ($idLower.Contains($kwLower) -or $descLower.Contains($kwLower) -or $triggerLower.Contains($kwLower))) {
            $score += 2
        }
    }

    if ($idLower.Contains($SubHubName.ToLower()) -or $pathLower.Contains($SubHubName.ToLower())) {
        $score += 5
    }

    foreach ($neg in @($SubHubRule.negative_keywords)) {
        $negLower = $neg.ToLower()
        if ($tokenSet.ContainsKey($negLower) -or $idLower.Contains($negLower) -or $descLower.Contains($negLower)) {
            $score -= 5
        }
    }

    $anchorKeywords = @($SubHubRule.anchor_keywords | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    if ($anchorKeywords.Count -gt 0) {
        $anchorHits = 0
        foreach ($anchor in $anchorKeywords) {
            $anchorLower = $anchor.ToLower()
            $isShortAnchor = ($anchorLower.Length -lt 3)

            if ($tokenSet.ContainsKey($anchorLower)) {
                $anchorHits++
                continue
            }

            if (-not $isShortAnchor -and ($idLower.Contains($anchorLower) -or $descLower.Contains($anchorLower) -or $triggerLower.Contains($anchorLower))) {
                $anchorHits++
            }
        }

        if ($anchorHits -gt 0) {
            $score += 3
        }
        else {
            $score -= 3
        }
    }

    return $score
}

function Match-Skill-ToSubHub {
    param([PSCustomObject] $Skill, [hashtable] $SubHubDefs)

    $matches = @()

    foreach ($mainHub in $SubHubDefs.Keys) {
        foreach ($subHub in $SubHubDefs[$mainHub].Keys) {
            $rule = $SubHubDefs[$mainHub][$subHub]
            $score = Get-ScoreForSubHub -Skill $Skill -SubHubRule $rule -SubHubName $subHub

            $matches += [PSCustomObject]@{
                main = $mainHub
                sub = $subHub
                key = "$mainHub-$subHub"
                score = [int] $score
            }
        }
    }

    return @($matches | Sort-Object -Property @{Expression = 'score'; Descending = $true}, @{Expression = 'key'; Descending = $false})
}

function Get-SkillAssignments {
    param(
        [PSCustomObject] $Skill,
        [hashtable] $SubHubDefs,
        [bool] $EnableMultiHub,
        [int] $PrimaryThreshold,
        [int] $SecondaryThreshold,
        [int] $MaxAssignments
    )

    $sortedMatches = @(Match-Skill-ToSubHub -Skill $Skill -SubHubDefs $SubHubDefs)
    if ($sortedMatches.Count -eq 0) {
        return @()
    }

    $primary = $sortedMatches[0]
    if ($primary.score -lt $PrimaryThreshold) {
        return @()
    }

    $selected = @($primary)
    if ($EnableMultiHub -and $MaxAssignments -gt 1) {
        $secondary = $sortedMatches |
            Where-Object { $_.key -ne $primary.key -and $_.score -ge $SecondaryThreshold } |
            Select-Object -First ($MaxAssignments - 1)

        $selected += @($secondary)
    }

    return @($selected)
}

function New-AssignedSkillRecord {
    param(
        [PSCustomObject] $Skill,
        [string] $PrimaryHub,
        [array] $AssignedHubs,
        [int] $MatchScore,
        [bool] $IsPrimary
    )

    return [PSCustomObject]@{
        id = $Skill.id
        description = $Skill.description
        path = $Skill.path
        triggers = @($Skill.triggers)
        src = $Skill.src
        primary_hub = $PrimaryHub
        assigned_hubs = @($AssignedHubs)
        match_score = [int] $MatchScore
        is_primary = [bool] $IsPrimary
    }
}

function Build-TopTriggers {
    param(
        [array] $Skills,
        [int] $Limit = 20
    )

    $freq = @{}
    foreach ($skill in $Skills) {
        foreach ($trigger in @($skill.triggers)) {
            if (-not [string]::IsNullOrWhiteSpace($trigger)) {
                if (-not $freq.ContainsKey($trigger)) {
                    $freq[$trigger] = 0
                }
                $freq[$trigger] += 1
            }
        }
    }

    return $freq.GetEnumerator() |
        Sort-Object -Property Value -Descending |
        Select-Object -First $Limit |
        ForEach-Object { $_.Key }
}

function Write-FileUtf8NoBom {
    param(
        [string] $Path,
        [string] $Content
    )

    $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($Path, $Content, $utf8NoBom)
}

function Write-SubHubFiles {
    param(
        [string] $OutPath,
        [string] $MainHub,
        [string] $SubHub,
        [array] $Skills,
        [hashtable] $SubHubDef,
        [string] $RepoRoot,
        [bool] $ValidateQuality = $true
    )

    $skillName = "skills-$MainHub-$SubHub"
    $skillDescription = $SubHubDef.description.Replace("'", "''")
    $title = "$MainHub / $SubHub"
    $topTriggers = Build-TopTriggers -Skills $Skills -Limit 20
    $triggerHints = ($topTriggers | ForEach-Object { "- $_" }) -join "`n"

    $workflowMd = $WORKFLOW_TEMPLATE `
        -replace "{TITLE}", $title `
        -replace "{DESCRIPTION}", $SubHubDef.description `
        -replace "{USE_CASE_1}", $SubHubDef.best_for[0] `
        -replace "{USE_CASE_2}", $SubHubDef.best_for[1] `
        -replace "{USE_CASE_3}", $SubHubDef.best_for[2] `
        -replace "{TRIGGER_HINTS}", $triggerHints

    $skillMd = $SKILL_ROUTER_TEMPLATE `
        -replace "{SKILL_NAME}", $skillName `
        -replace "{SKILL_DESCRIPTION}", $skillDescription

    $manifest = [ordered]@{
        name = $skillName
        main_hub = $MainHub
        sub_hub = $SubHub
        description = $SubHubDef.description
        skill_count = $Skills.Count
        src_count = (@($Skills.src | Select-Object -Unique)).Count
        top_triggers = @($topTriggers)
        generated_at = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssK")
        files = [ordered]@{
            skill = "SKILL.md"
            workflow = "workflow.md"
            index = "skills-index.json"
            catalog = "skills-catalog.ndjson"
        }
    }

    $indexItems = foreach ($skill in $Skills | Sort-Object id) {
        [ordered]@{
            id = $skill.id
            triggers = @($skill.triggers | Select-Object -First 5)
            src = $skill.src
            primary_hub = $skill.primary_hub
            is_primary = [bool] $skill.is_primary
            match_score = [int] $skill.match_score
        }
    }

    $catalogLines = foreach ($skill in $Skills | Sort-Object id) {
        [ordered]@{
            id = $skill.id
            description = $skill.description
            path = $skill.path
            triggers = @($skill.triggers)
            src = $skill.src
            primary_hub = $skill.primary_hub
            assigned_hubs = @($skill.assigned_hubs)
            match_score = [int] $skill.match_score
            is_primary = [bool] $skill.is_primary
        } | ConvertTo-Json -Compress
    }

    # Run quality validation if enabled
    if ($ValidateQuality) {
        # Simply convert manifest hashtable to PSCustomObject (PowerShell handles nested objects)
        $manifestObj = [PSCustomObject]$manifest
        $report = New-ValidationReport -SubHubKey "$MainHub/$SubHub" -Manifest $manifestObj -CatalogItems $catalogLines -WorkflowText $workflowMd -RepoRoot $RepoRoot
        Write-ValidationReport -Report $report
        if (-not $report.passed) {
            Write-Host "[ERROR] Quality validation failed for $MainHub/$SubHub. Fix issues above before proceeding." -ForegroundColor Red
            return $false
        }
    }

    if (-not $DryRun) {
        mkdir -Path $OutPath -Force | Out-Null
        Write-FileUtf8NoBom -Path (Join-Path $OutPath "SKILL.md") -Content $skillMd
        Write-FileUtf8NoBom -Path (Join-Path $OutPath "workflow.md") -Content $workflowMd
        Write-FileUtf8NoBom -Path (Join-Path $OutPath "skills-manifest.json") -Content (($manifest | ConvertTo-Json -Depth 8) + [Environment]::NewLine)
        Write-FileUtf8NoBom -Path (Join-Path $OutPath "skills-index.json") -Content (($indexItems | ConvertTo-Json -Depth 6) + [Environment]::NewLine)
        Write-FileUtf8NoBom -Path (Join-Path $OutPath "skills-catalog.ndjson") -Content (($catalogLines -join [Environment]::NewLine) + [Environment]::NewLine)
    }
    
    return $true
}

# ============================================================================
# EXECUTION
# ============================================================================

Write-Host "[INFO] Aggregated Skill System - Initialization" -ForegroundColor Cyan
Write-Host "[INFO] src dir: $srcHubsDir"
Write-Host "[INFO] Output dir: $OutputDir"
if ($srcRepoMode -eq "changed-only" -and $ChangedOnlyFallbackToLatest) {
    Write-Host "[WARN] src repo mode: changed-only (no previous lock found). Falling back to latest: $($SelectedsrcRepos -join ', ')" -ForegroundColor Yellow
}
elseif ($SelectedsrcRepos.Count -gt 0) {
    Write-Host "[INFO] src repo mode: $srcRepoMode (selected: $($SelectedsrcRepos -join ', '))"
}
else {
    Write-Host "[INFO] src repo mode: $srcRepoMode (no external src repos selected)"
}
Write-Host "[INFO] Multi-hub mode: $AllowMultiHub (max hubs per skill: $MaxHubsPerSkill, primary>=${PrimaryMinScore}, secondary>=${SecondaryMinScore})"
Write-Host ""

if (-not $DryRun) {
    if (-not (Test-Path $OutputDir)) {
        mkdir $OutputDir | Out-Null
    }
}

# Load all skills from existing hubs or fallback file discovery
Write-Host "[INFO] Step 1: Loading skills..."
$allSkills = @()
$srcCount = 0

$manifestFiles = @()
if (Test-Path $srcHubsDir) {
    $manifestFiles = Get-ChildItem -Path $srcHubsDir -Filter "hub-manifest.json" -Recurse
}

if ($manifestFiles.Count -gt 0) {
    foreach ($manifestFile in $manifestFiles) {
        $manifest = Get-Content $manifestFile.FullName -Raw | ConvertFrom-Json

        foreach ($skill in $manifest.skills) {
            $skillPath = Convert-ToRepoRelativePath -Path $skill.path
            $excludedCategory = Get-ExcludedCategory -Id $skill.id -Description $skill.description -Path $skillPath -Triggers @($skill.triggers)
            if ($excludedCategory) {
                Add-ExclusionStat -Category $excludedCategory
                continue
            }
            $skillObj = [PSCustomObject]@{
                id = $skill.id
                description = $skill.description
                path = $skillPath
                triggers = @($skill.triggers)
                src = Get-Skillsrc -Path $skill.path
            }

            $allSkills += $skillObj
            $srcCount++
        }
    }
}
else {
    $existingFallbackRoots = @($FallbackSkillRoots | Where-Object { Test-Path $_ })
    if ($existingFallbackRoots.Count -eq 0) {
        Write-Error "No hub-manifest.json found in $srcHubsDir and no valid fallback roots exist. Checked: $($FallbackSkillRoots -join ', ')"
        exit 1
    }

    Write-Host "[INFO] hub-manifest.json not found in $srcHubsDir; using fallback skill discovery from: $($existingFallbackRoots -join ', ')" -ForegroundColor DarkCyan
    $allSkills = Load-SkillsFromFiles -Roots $existingFallbackRoots
}

Write-Host "[✓] Loaded $($allSkills.Count) skills from $(($allSkills.src | Select-Object -Unique).Count) srcs"
if ($script:ExcludedSkillStats.Count -gt 0) {
    $excludedTotal = ($script:ExcludedSkillStats.Values | Measure-Object -Sum).Sum
    Write-Host "[INFO] Excluded skills by policy: $excludedTotal" -ForegroundColor Yellow
    foreach ($key in ($script:ExcludedSkillStats.Keys | Sort-Object)) {
        Write-Host "  - ${key}: $($script:ExcludedSkillStats[$key])" -ForegroundColor Yellow
    }
}
Write-Host ""

# Load semantic classifications if enabled
$semanticLookup = @{}
if ($EnableSemanticScoring) {
    Write-Host "[INFO] Loading semantic classifications..." -ForegroundColor Cyan
    $semanticLookup = Load-SemanticClassifications -FilePath $SemanticClassificationsFile
    if ($semanticLookup.Count -eq 0) {
        Write-Host "[WARN] Semantic classifications not available; proceeding with keyword-based scoring only" -ForegroundColor Yellow
        $EnableSemanticScoring = $false
    }
}

# Categorize into sub-hubs
Write-Host "[INFO] Step 2: Categorizing skills into sub-hubs..."
$subHubMap = @{}
$unmatchedSkills = @()
$multiAssignedSkillCount = 0
$totalAssignments = 0
$categoryGapSignals = @()
$uncoveredGapPatterns = @{}
$reviewCandidates = @()

foreach ($category in $CATEGORY_GAP_PATTERNS.Keys) {
    $mappedMainHub = $category
    if ($CATEGORY_PATTERN_TO_MAIN_HUB.ContainsKey($category)) {
        $mappedMainHub = $CATEGORY_PATTERN_TO_MAIN_HUB[$category]
    }

    if (-not $SUB_HUB_DEFINITIONS.ContainsKey($mappedMainHub)) {
        $uncoveredGapPatterns[$category] = $CATEGORY_GAP_PATTERNS[$category]
    }
}

foreach ($skill in $allSkills) {
    $assignments = @()
    if ($MANUAL_HUB_OVERRIDES.ContainsKey($skill.id)) {
        $override = $MANUAL_HUB_OVERRIDES[$skill.id]
        $assignments = @([PSCustomObject]@{
                main = $override.main
                sub = $override.sub
                key = "$($override.main)-$($override.sub)"
                score = [int] $override.score
            })
    }
    else {
        if ($EnableReviewBand) {
            $sortedMatches = @(Match-Skill-ToSubHub -Skill $skill -SubHubDefs $SUB_HUB_DEFINITIONS)
            
            # Apply semantic scoring refinement if available
            if ($EnableSemanticScoring -and $semanticLookup.Count -gt 0) {
                $semanticMatch = Get-SemanticScoreForSkill -Skill $skill -SemanticLookup $semanticLookup -SubHubDefs $SUB_HUB_DEFINITIONS
                if ($semanticMatch) {
                    # Find matching hub in sortedMatches and boost its score
                    $semanticHubKey = "$($semanticMatch.main)-$($semanticMatch.sub)"
                    for ($i = 0; $i -lt $sortedMatches.Count; $i++) {
                        $matchKey = "$($sortedMatches[$i].main)-$($sortedMatches[$i].sub)"
                        if ($matchKey -eq $semanticHubKey) {
                            $blendedScore = Blend-KeywordAndSemanticScores -KeywordScore $sortedMatches[$i].score -SemanticScore $semanticMatch.score -WeightFactor $SemanticWeightFactor
                            $sortedMatches[$i].score = [int]$blendedScore
                            $sortedMatches[$i] | Add-Member -NotePropertyName "semantic_boost" -NotePropertyValue $true -Force
                            break
                        }
                    }
                    # Re-sort after scoring update
                    $sortedMatches = @($sortedMatches | Sort-Object -Property @{Expression = 'score'; Descending = $true}, @{Expression = 'key'; Descending = $false})
                }
            }
            
            if ($sortedMatches.Count -gt 0) {
                $primary = $sortedMatches[0]
                if ($primary.score -ge $AutoAcceptMinScore) {
                    $assignments = @(Get-SkillAssignments -Skill $skill -SubHubDefs $SUB_HUB_DEFINITIONS -EnableMultiHub:$AllowMultiHub -PrimaryThreshold $AutoAcceptMinScore -SecondaryThreshold $SecondaryMinScore -MaxAssignments $MaxHubsPerSkill)
                }
                elseif ($primary.score -ge $ReviewMinScore) {
                    $topMatches = @($sortedMatches | Select-Object -First 3)
                    $reviewCandidates += [ordered]@{
                        id = $skill.id
                        path = $skill.path
                        src = $skill.src
                        suggested_primary_hub = "$($primary.main)/$($primary.sub)"
                        suggested_score = [int] $primary.score
                        semantic_boosted = if ($primary.semantic_boost) { $true } else { $false }
                        top_matches = @(
                            $topMatches | ForEach-Object {
                                [ordered]@{
                                    hub = "$($_.main)/$($_.sub)"
                                    score = [int] $_.score
                                    semantic_boosted = if ($_.semantic_boost) { $true } else { $false }
                                }
                            }
                        )
                        reason = "review-band"
                    }
                }
            }
        }
        else {
            $assignments = @(Get-SkillAssignments -Skill $skill -SubHubDefs $SUB_HUB_DEFINITIONS -EnableMultiHub:$AllowMultiHub -PrimaryThreshold $PrimaryMinScore -SecondaryThreshold $SecondaryMinScore -MaxAssignments $MaxHubsPerSkill)
        }
    }

    if ($assignments.Count -eq 0) {
        $unmatchedSkills += $skill
        continue
    }

    if ($assignments.Count -gt 1) {
        $multiAssignedSkillCount++
    }

    $assignedHubPaths = @($assignments | ForEach-Object { "$($_.main)/$($_.sub)" })
    $primaryHubPath = "$($assignments[0].main)/$($assignments[0].sub)"

    foreach ($assignment in $assignments) {
        $key = "$($assignment.main)-$($assignment.sub)"
        if (-not $subHubMap[$key]) {
            $subHubMap[$key] = @{
                main = $assignment.main
                sub = $assignment.sub
                skills = @()
            }
        }

        $enrichedSkill = New-AssignedSkillRecord -Skill $skill -PrimaryHub $primaryHubPath -AssignedHubs $assignedHubPaths -MatchScore $assignment.score -IsPrimary ($assignment.key -eq $assignments[0].key)
        $subHubMap[$key].skills += $enrichedSkill
        $totalAssignments++
    }
}

if ($unmatchedSkills.Count -gt 0) {
    $fallbackKey = "general-misc"
    $fallbackSkills = foreach ($skill in $unmatchedSkills) {
        New-AssignedSkillRecord -Skill $skill -PrimaryHub "general/misc" -AssignedHubs @("general/misc") -MatchScore 0 -IsPrimary $true
    }

    $subHubMap[$fallbackKey] = @{
        main = "general"
        sub = "misc"
        skills = @($fallbackSkills)
    }

    $totalAssignments += $unmatchedSkills.Count

    # Guardrail: detect large hidden categories only for categories that still have no dedicated main hub.
    $categoryGapSignals = @(Get-CategoryGapSignals -Skills $unmatchedSkills -Patterns $uncoveredGapPatterns -MinCount $CategoryGapThreshold)
}

Write-Host "[✓] Categorized into $($subHubMap.Count) sub-hubs (unmatched routed: $($unmatchedSkills.Count), multi-assigned skills: $multiAssignedSkillCount, total assignments: $totalAssignments)"
if ($EnableReviewBand) {
    Write-Host "[INFO] Review candidates (score $ReviewMinScore..$($AutoAcceptMinScore - 1)): $($reviewCandidates.Count)"
}
if ($categoryGapSignals.Count -gt 0) {
    Write-Host "[WARN] Potential missing hub categories detected in general/misc:" -ForegroundColor Yellow
    foreach ($signal in $categoryGapSignals) {
        Write-Host ("  - {0}: {1} skills (threshold: {2}) e.g. {3}" -f $signal.category, $signal.count, $signal.threshold, ($signal.sample_skills -join ", ")) -ForegroundColor Yellow
    }

    if ($FailOnCategoryGaps) {
        throw "Category gap guard failed. Add dedicated hubs for the categories above or raise -CategoryGapThreshold."
    }
}
Write-Host ""

# Generate BMAD-style files for each sub-hub
Write-Host "[INFO] Step 3: Generating BMAD-style sub-hubs (SKILL router + workflow + catalog)..."

$MIN_SKILLS_PER_HUB = $MinSkillsPerHub
$routingIndex = @()
$skippedHubsCount = 0
$skippedSkillsCount = 0

foreach ($subHubKey in $subHubMap.Keys) {
    $subHubData = $subHubMap[$subHubKey]
    $subHubDef = $SUB_HUB_DEFINITIONS[$subHubData.main][$subHubData.sub]
    
    # Deduplicate
    $uniqueSkills = Deduplicate-Skills -Skills $subHubData.skills
    
    # Skip hubs with fewer than minimum required skills
    if ($uniqueSkills.Count -lt $MIN_SKILLS_PER_HUB) {
        Write-Host "[!] $subHubKey skipped: $($uniqueSkills.Count) skills < $MIN_SKILLS_PER_HUB min" -ForegroundColor Yellow
        $skippedHubsCount++
        $skippedSkillsCount += $uniqueSkills.Count
        continue
    }
    
    # Create output path
    $subFolder = Join-Path -Path $OutputDir -ChildPath $subHubData.main
    $outPath = Join-Path -Path $subFolder -ChildPath $subHubData.sub
    
    $msg = "[✓] {0}: {1} skills (deduped from {2}) -> router mode" -f $subHubKey, $uniqueSkills.Count, $subHubData.skills.Count
    Write-Host $msg

    $routingIndex += [ordered]@{
        key = $subHubKey
        main_hub = $subHubData.main
        sub_hub = $subHubData.sub
        skill_count = $uniqueSkills.Count
        top_triggers = @(Build-TopTriggers -Skills $uniqueSkills -Limit 10)
        path = ((Join-Path $subHubData.main $subHubData.sub) -replace '\\', '/')
    }
    
    $success = Write-SubHubFiles -OutPath $outPath -MainHub $subHubData.main -SubHub $subHubData.sub -Skills $uniqueSkills -SubHubDef $subHubDef -RepoRoot $RepoRoot -ValidateQuality $true
    if (-not $success) {
        Write-Host "  [!] Skipped $($subHubData.main)/$($subHubData.sub) - validation failed" -ForegroundColor Yellow
    }
}

if (-not $DryRun) {
    Write-FileUtf8NoBom -Path (Join-Path $OutputDir "subhub-index.json") -Content (($routingIndex | ConvertTo-Json -Depth 8) + [Environment]::NewLine)

    $reviewFilePath = Join-Path $OutputDir "review-candidates.ndjson"
    if ($EnableReviewBand -and $reviewCandidates.Count -gt 0) {
        $reviewLines = @($reviewCandidates | ForEach-Object { ($_ | ConvertTo-Json -Depth 6 -Compress) })
        Write-FileUtf8NoBom -Path $reviewFilePath -Content (($reviewLines -join [Environment]::NewLine) + [Environment]::NewLine)
    }
    else {
        Write-FileUtf8NoBom -Path $reviewFilePath -Content ""
    }

    $excludedByCategory = [ordered]@{}
    foreach ($cat in ($script:ExcludedSkillStats.Keys | Sort-Object)) {
        $excludedByCategory[$cat] = $script:ExcludedSkillStats[$cat]
    }

    $lockPayload = [ordered]@{
        generated_at = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssK")
        src_repo_mode = $srcRepoMode
        selected_src_repos = @($SelectedsrcRepos)
        exclude_categories = @($script:EffectiveExcludeCategories)
        exclusion_stats = [ordered]@{
            total = if ($script:ExcludedSkillStats.Count -gt 0) { ($script:ExcludedSkillStats.Values | Measure-Object -Sum).Sum } else { 0 }
            by_category = $excludedByCategory
        }
        min_skills_per_hub = $MinSkillsPerHub
        score_policy = [ordered]@{
            review_band_enabled = [bool] $EnableReviewBand
            review_min_score = $ReviewMinScore
            auto_accept_min_score = $AutoAcceptMinScore
            secondary_min_score = $SecondaryMinScore
            review_candidates = $reviewCandidates.Count
        }
        category_gap_threshold = $CategoryGapThreshold
        category_gaps = @(
            $categoryGapSignals | ForEach-Object {
                [ordered]@{
                    category = $_.category
                    count = $_.count
                    threshold = $_.threshold
                    sample_skills = @($_.sample_skills)
                }
            }
        )
        src_repositories = @(
            $CurrentsrcRepoStates |
                Sort-Object name |
                ForEach-Object {
                    [ordered]@{
                        name = $_.name
                        vcs = $_.vcs
                        revision = $_.revision
                        dirty = [bool] $_.dirty
                        fingerprint = $_.fingerprint
                    }
                }
        )
    }

    Write-FileUtf8NoBom -Path (Join-Path $OutputDir ".skill-lock.json") -Content (($lockPayload | ConvertTo-Json -Depth 8) + [Environment]::NewLine)
}

Write-Host ""
Write-Host "[INFO] ============================================"
Write-Host "[✓] Aggregation Complete"
Write-Host "[INFO]   Sub-hubs created: $(($routingIndex | Measure-Object).Count)"
Write-Host "[INFO]   Sub-hubs skipped (< $MIN_SKILLS_PER_HUB skills): $skippedHubsCount"
Write-Host "[INFO]   Total skills in active hubs: $(($routingIndex | ForEach-Object { $_.skill_count } | Measure-Object -Sum).Sum)"
Write-Host "[INFO]   Skills removed from undersized hubs: $skippedSkillsCount"
Write-Host "[INFO]   Category-gap signals in misc: $($categoryGapSignals.Count)"
if ($EnableReviewBand) {
    Write-Host "[INFO]   Review candidates queued: $($reviewCandidates.Count)"
}
Write-Host "[INFO]   Output dir: $OutputDir"
Write-Host "[INFO] ============================================"
