# Skill Aggregation System - BMAD Style Builder
# Transforms flat hub-manifest structure to sub-hub architecture
# Generates lightweight SKILL.md router + external catalog data

param(
    [string] $srcHubsDir = ".\skill-manage\hub-skills",
    [string] $OutputDir = ".\skill-manage\skills-aggregated",
    [array] $FallbackSkillRoots = @(".\skill-manage\src"),
    [ValidateSet("latest", "all", "selected", "changed-only")]
    [string] $srcRepoMode = "latest",
    [string[]] $srcRepoNames = @(),
    [Switch] $DryRun = $false,
    [Switch] $AllowMultiHub = $false,
    [string[]] $ExcludeCategories = @(),
    [Switch] $NoCategoryExclusions = $false,
    [Switch] $SyncToTools = $false,
    [ValidateRange(1, 500)]
    [int] $MinSkillsPerHub = 5,
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
    [string] $SemanticClassificationsFile = ".\skill-manage\skills-aggregated\semantic-classifications.json",
    [ValidateRange(0.0, 1.0)]
    [double] $SemanticWeightFactor = 0.6,
    [bool] $MarketingFirst = $true,
    [Switch] $NoPrompt = $false
)

if ($ReviewMinScore -gt $AutoAcceptMinScore) {
    throw "ReviewMinScore ($ReviewMinScore) cannot be greater than AutoAcceptMinScore ($AutoAcceptMinScore)."
}

if ($EnableReviewBand -and $SecondaryMinScore -lt $AutoAcceptMinScore) {
    $SecondaryMinScore = $AutoAcceptMinScore
}

$InteractivePrompt = -not $NoPrompt -and [Environment]::UserInteractive

function Confirm-OrExit {
    param([string] $Message)

    if (-not $InteractivePrompt) {
        return
    }

    $confirmation = (Read-Host "$Message [y/N]").Trim().ToLowerInvariant()
    if ($confirmation -ne "y" -and $confirmation -ne "yes") {
        Write-Host "[WARN] Cancelled by user before write operations started." -ForegroundColor Yellow
        exit 0
    }
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
    # Normalize script root and derive repository root even when script lives under skill-manage/scripts.
    $normalizedScriptRoot = (Get-Item $PSScriptRoot).FullName
    $candidateRootObj = Get-Item (Join-Path $normalizedScriptRoot "..")
    if ($candidateRootObj.Name -ieq "skill-manage") {
        $RepoRootObj = Get-Item (Join-Path $candidateRootObj.FullName "..")
    }
    else {
        $RepoRootObj = $candidateRootObj
    }
    $RepoRoot = $RepoRootObj.FullName
    
    $legacyHubsDir = Join-Path $RepoRoot "skill-manage/hub-skills"
    $srcReposDir = Join-Path $RepoRoot "skill-manage/src"
    if (Test-Path $legacyHubsDir) {
        $srcHubsDir = $legacyHubsDir
    }
    else {
        # New layout keeps skills under src repos; hub-manifest is optional.
        $srcHubsDir = $srcReposDir
    }
    $OutputDir = Join-Path $RepoRoot "skill-manage/skills-aggregated"
}

$srcRootPath = Join-Path $RepoRoot "skill-manage/src"
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

Read routing.csv to find the exact skill file path needed for the user request.
'@



# ============================================================================
# SKILL DEFINITIONS (TAXONOMY)
# ============================================================================

$SUB_HUB_DEFINITIONS = @{

    "programming" = @{
        "typescript" = @{
            keywords = @("typescript", "tsconfig", "tsx", "type-system", "generics", "type-safe", "javascript", "angular", "electron", "fp-ts", "zustand", "node", "eslint", "webpack", "vite", "sanity")
            anchor_keywords = @("typescript", "tsconfig", "tsx", "angular", "electron")
            negative_keywords = @("python", "golang", "rust", "java", "postgres", "mongodb", "redis", "kubernetes")
            description = "TypeScript language expertise: types, patterns, advanced features, configuration, and best practices"
            best_for = @(
                "Building type-safe applications",
                "Creating reusable component libraries",
                "Implementing complex generic patterns"
            )
        }
        "python" = @{
            keywords = @("python", "py", "django", "fastapi", "async", "asyncio", "plotly", "polars", "pandas", "numpy", "scipy", "data-scientist", "data-science", "jupyter", "cirq", "qiskit")
            anchor_keywords = @("python", "py", "django", "fastapi", "plotly", "polars", "cirq", "qiskit")
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
            keywords = @("rust", "cargo", "ownership", "lifetimes", "unsafe", "robius", "bevy")
            anchor_keywords = @("rust", "cargo", "ownership", "robius")
            negative_keywords = @("typescript", "python", "golang", "java")
            description = "Rust: memory safety, performance, systems programming, and async patterns"
            best_for = @(
                "Building fast, memory-safe systems",
                "Systems programming",
                "WebAssembly applications"
            )
        }
        "java" = @{
            keywords = @("java", "spring", "maven", "jvm", "virtual-threads", "dotnet", "csharp", "php", "scala", "elixir", "haskell", "cpp", "avalonia", "salesforce", "arm-cortex", "minecraft-bukkit")
            anchor_keywords = @("java", "spring", "jvm", "maven", "dotnet", "csharp", "php", "scala", "elixir", "haskell", "cpp", "salesforce")
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
            keywords = @("react", "nextjs", "jsx", "hooks", "server-components", "app-router", "astro", "svelte", "vue", "remix")
            negative_keywords = @("postgres", "mongodb", "redis", "sql")
            description = "React and Next.js: components, hooks, server-side rendering, and performance optimization"
            best_for = @(
                "Building modern web applications",
                "Full-stack development with Next.js",
                "Server and client component patterns"
            )
        }
        "web-basics" = @{
            keywords = @("html", "css", "javascript", "dom", "responsive", "web-standards", "pwa", "progressive-web-app", "browser-extension", "chrome-extension", "web-performance", "favicon", "i18n", "localization")
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
            keywords = @("api", "rest", "graphql", "openapi", "swagger", "pagination", "architecture", "cqrs", "ddd", "event-sourcing", "event-store", "microservices", "projection", "error-handling", "webhook", "stripe", "payment", "odoo", "rails", "monorepo", "clean-code", "refactoring", "code-review", "software-architecture", "domain-driven")
            negative_keywords = @("react", "nextjs", "html", "css")
            description = "API design: REST, GraphQL, and best practices for scalable web services"
            best_for = @(
                "Designing robust APIs",
                "GraphQL schema design",
                "API versioning and deprecation"
            )
        }
        "databases" = @{
            keywords = @("database", "sql", "postgres", "mongodb", "redis", "nosql", "orm", "data-engineer", "data-pipeline", "dbt", "etl", "spark", "duckdb", "data-quality", "data-warehouse", "migration")
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
            keywords = @("aws", "gcp", "azure", "cloudflare", "lambda", "serverless", "terraform", "devops", "deployment", "devcontainer", "prometheus", "grafana", "observability", "monitoring", "slo", "sli", "incident", "on-call", "postmortem", "ci", "cd", "bazel", "infrastructure", "bash", "linux", "powershell", "windows", "posix", "shell")
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
            keywords = @("saas", "pricing", "revenue", "arr", "mrr", "churn", "ltv", "cac", "unit-economics", "go-to-market", "gtm", "market-sizing", "tam", "sam", "som", "roadmap", "startup", "fintech", "alpha-vantage", "risk-metrics", "cohort", "finance", "financial", "metrics", "business-analyst", "business-model", "risk-manager")
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
            keywords = @("product", "strategy", "roadmap", "prd", "stakeholder", "discovery", "market", "positioning", "vision", "prioritization", "alignment", "wds", "user-stories", "user-story", "job-stories", "proto-persona", "persona", "epic", "brainstorming", "brainstorm", "workshop", "pestle", "customer-journey", "user-segmentation", "interview", "problem-framing", "hr", "employment", "team-composition")
            anchor_keywords = @("product", "strategy", "roadmap", "prd", "wds", "user-stories", "persona", "customer-journey", "workshop")
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
            keywords = @("marketing", "strategy", "brand", "positioning", "customer", "audience", "market-analysis", "competitive-analysis", "go-to-market", "competitor", "outreach", "ideal-customer", "growth", "ua-campaign", "lead-generation", "lead-magnets", "cold-outreach", "acquisition", "sales")
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
            keywords = @("content", "copywriting", "seo", "blog", "article", "writing", "editorial", "keyword", "search-engine", "writing-style", "muscular-prose", "copy-editing", "prose", "de-ai-ify", "voice-extractor", "proofreader", "homepage-audit", "page-cro", "form-cro", "signup-flow", "case-study", "storytelling", "humanizer", "citation")
            anchor_keywords = @("seo", "content-marketing", "copywriting", "writing-style")
            negative_keywords = @("email", "social", "video", "design", "html", "css")
            description = "Content marketing, SEO & Writing Styles: copywriting, blog strategy, search optimization, and high-quality prose standards"
            best_for = @(
                "Creating SEO-optimized content",
                "Establishing high-quality writing styles",
                "Improving search and editorial quality"
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
            keywords = @("social", "twitter", "linkedin", "instagram", "tiktok", "youtube", "content-calendar", "engagement", "viral", "publisher", "posting", "article-publisher", "tweet", "x-article", "reddit")
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
            keywords = @("security", "authentication", "authorization", "oauth", "jwt", "encryption", "tls", "ssl", "vulnerability", "secure", "burpsuite", "malware", "red-team", "forensics", "binary-analysis", "reverse-engineer", "semgrep", "ffuf", "gdpr", "privacy", "compliance", "pentest", "constant-time", "variant-analysis")
            anchor_keywords = @("security", "authentication", "oauth", "jwt", "burpsuite", "malware", "red-team", "forensics", "reverse-engineer", "semgrep", "gdpr", "privacy", "pentest")
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
            keywords = @("testing", "test", "unit-test", "integration-test", "e2e", "qa", "cypress", "playwright", "vitest", "jest", "automation", "debug", "debugging", "bug-hunter", "fix-review")
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
    "design" = @{
        "ui-ux" = @{
            keywords = @("ui", "ux", "design", "designer", "wireframe", "prototype", "accessibility", "usability", "design-system", "figma", "interaction", "hig", "swiftui", "wcag", "a11y")
            anchor_keywords = @("ui", "ux", "design-system", "wireframe", "accessibility", "hig", "swiftui", "wcag")
            negative_keywords = @("kubernetes", "docker", "postgres", "mongodb", "redis")
            description = "UI/UX design: interface design, wireframes, design systems, accessibility, and interaction patterns"
            best_for = @(
                "Designing intuitive user interfaces",
                "Building and maintaining design systems",
                "Improving usability and accessibility"
            )
        }
        "design-thinking" = @{
            keywords = @("design-thinking", "design-process", "design-strategy", "design-research", "design-sprint", "design-workshop", "double-diamond", "ideation", "empathy-mapping", "user-journey", "service-design")
            anchor_keywords = @("design-thinking", "design-sprint", "design-workshop", "double-diamond", "service-design")
            negative_keywords = @("marketing", "seo", "newsletter")
            description = "Design Thinking: human-centered design methodology, strategic discovery, workshops, and innovation frameworks."
            best_for = @(
                "Mapping user psychology to business goals",
                "Facilitating multi-stakeholder workshops",
                "Solving complex, ambiguous product problems"
            )
        }
        "brand-guidelines" = @{
            keywords = @("brand-guidelines", "brand-strategy", "brand-identity", "visual-identity", "brand-voice", "brand-archetype", "logo-usage", "color-palette", "typography", "style-guide")
            anchor_keywords = @("brand-guidelines", "brand-identity", "visual-identity", "style-guide")
            negative_keywords = @("marketing", "seo", "newsletter")
            description = "Brand Identity & Guidelines: visual systems, brand strategy, voice and tone, and identity governance."
            best_for = @(
                "Establishing consistent visual identities",
                "Defining brand voice and communication standards",
                "Creating and maintaining comprehensive style guides"
            )
        }
    }

    "ai" = @{
        "llm-agents" = @{
            keywords = @("llm", "gpt", "prompt", "rag", "embedding", "vector", "agent", "transformer", "chatbot", "fine-tuning", "claude", "context", "fal", "hugging", "ml", "nlp", "voice-ai", "notebooklm", "stability", "image-studio", "computer-vision", "ml-engineer", "mlops", "machine-learning", "deep-research", "ai-studio", "mcp", "model")
            anchor_keywords = @("llm", "gpt", "rag", "agent", "claude", "hugging", "ml-engineer", "mlops", "mcp", "fal", "computer-vision")
            negative_keywords = @("newsletter", "seo", "css", "html")
            description = "AI engineering: LLM prompting, RAG pipelines, embeddings, and autonomous agent patterns"
            best_for = @(
                "Building LLM-powered assistants",
                "Designing RAG and retrieval workflows",
                "Improving prompt and agent reliability"
            )
        }
        "prompting-builder" = @{
            keywords = @("prompt", "prompt-engineering", "context", "system-prompt", "llm-prompt", "tuning", "compression")
            anchor_keywords = @("prompt-engineering", "system-prompt", "context-compression")
            negative_keywords = @("ui", "css", "html", "saas")
            description = "Prompt engineering: advanced prompt techniques, context management, compression, and fine-tuning prompts"
            best_for = @(
                "Optimizing LLM instructions",
                "Managing large context windows",
                "Structuring model outputs",
                "Improving prompt and agent reliability",
                "enhancing prompt reliability"

            )
        }
        "skills-factory" = @{
            keywords = @("skill", "skill-creation", "skill-factory", "authoring", "generation", "lint", "validate")
            anchor_keywords = @("skill-creation", "skill-factory", "authoring")
            negative_keywords = @("ui", "css", "html", "saas")
            description = "AI skill authoring: creating, refining, auditing, and validating new AI skills"
            best_for = @(
                "Authoring and developing skills",
                "Auditing and validating skills",
                "Providing an interactive skills builder experience"
            )
        }
    
    }

    "productivity" = @{
        "workflow-automation" = @{
            keywords = @("productivity", "workflow", "automation", "automate", "automated", "automates", "task-management", "project-management", "agile", "scrum", "kanban", "notion", "planning", "orchestration", "orchestrate", "orchestrator", "agentic", "autonomous", "n8n", "zapier", "make", "langgraph", "crewai", "autogen", "tool-calling", "pipeline", "bmad", "commit", "git", "pr", "wiki", "ship", "issue", "diary", "daily", "conductor", "plan", "sprint", "standup", "documentation", "docs", "readme", "changelog", "onboarding", "tutorial", "obsidian", "json-canvas", "pdf", "pptx", "xlsx", "draw", "file-organizer", "kaizen", "closed-loop", "slack-bot", "telegram-bot", "chat-widget", "twilio")
            anchor_keywords = @("workflow", "automation", "productivity", "orchestration", "orchestrate", "orchestrator", "agentic", "n8n", "zapier", "langgraph", "crewai", "bmad", "git", "conductor", "obsidian", "wiki")
            negative_keywords = @("encryption", "oauth", "jwt", "unit-test", "integration-test", "e2e", "qa")
            description = "Productivity and automation: workflow automation, agentic orchestration, project delivery, and tool-connected process automation"
            best_for = @(
                "Automating repetitive delivery tasks",
                "Structuring team workflows and agentic pipelines",
                "Improving execution velocity with orchestration tools"
            )
        }
    }

    "mobile" = @{
        "cross-platform" = @{
            keywords = @("mobile", "android", "ios", "react-native", "flutter", "swift", "kotlin", "mobile-app", "app-clip", "app-store", "crash-analytics", "expo", "macos", "tuist", "xcode", "swiftpm")
            anchor_keywords = @("mobile", "android", "ios", "react-native", "flutter", "app-store", "expo", "macos", "swift")
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
    "ui-ux" = @("ui", "ux", "wireframe", "prototype", "design-system", "accessibility", "usability", "figma", "design-thinking", "brand-guidelines")
    "marketing" = @("marketing", "seo", "email", "newsletter", "campaign", "audience", "publisher", "social-media", "content-marketing")
    "security" = @("security", "auth", "authentication", "authorization", "oauth", "jwt", "encryption", "tls", "ssl", "vulnerability")
    "testing" = @("test", "testing", "unit-test", "integration-test", "e2e", "qa", "cypress", "vitest", "jest", "playwright", "playwright-test", "black-box", "white-box", "grey-box")
    "ai-llm" = @("llm", "gpt", "prompt", "embedding", "rag", "agent", "transformer", "chatbot")
    "data-science" = @("machine-learning", "ml", "data-science", "pandas", "numpy", "tensorflow", "pytorch", "analytics")
    "mobile" = @("mobile", "android", "ios", "flutter", "react-native", "swift", "kotlin")
}

$CATEGORY_PATTERN_TO_MAIN_HUB = @{
    "business" = "business"
    "product-strategy" = "business"
    "ui-ux" = "design"
    "marketing" = "marketing"
    "security" = "security"
    "testing" = "testing"
    "ai-llm" = "ai"
    "data-science" = "data-science"
    "mobile" = "mobile"
}

$MANUAL_HUB_OVERRIDES = @{
    "NanoBanana-PPT-Skills"        = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "acquisition-channel-advisor"  = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "active-directory-attacks"     = @{ main = "security"; sub = "core"; score = 100 }
    "ad-creative"                  = @{ main = "marketing"; sub = "content"; score = 100 }
    "agents-md"                    = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "ai-agent-development"         = @{ main = "ai"; sub = "llm-agents"; score = 100 }
    "ai-discoverability-audit"     = @{ main = "marketing"; sub = "content"; score = 100 }
    "airflow-dag-patterns"         = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "amazon-alexa"                 = @{ main = "ai"; sub = "llm-agents"; score = 100 }
    "animejs-animation"            = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "antigravity-skill-orchestrator"         = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "api-documentation"            = @{ main = "backend"; sub = "api-design"; score = 100 }
    "apify-ultimate-scraper"       = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "app-analytics"                = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "app-clips"                    = @{ main = "mobile"; sub = "cross-platform"; score = 100 }
    "app-icon-optimization"        = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "app-store-changelog"          = @{ main = "mobile"; sub = "cross-platform"; score = 100 }
    "architecture-patterns"        = @{ main = "backend"; sub = "api-design"; score = 100 }
    "astro"                        = @{ main = "frontend"; sub = "web-basics"; score = 100 }
    "attach-db"                    = @{ main = "backend"; sub = "databases"; score = 100 }
    "attack-tree-construction"     = @{ main = "security"; sub = "core"; score = 100 }
    "audit-skills"                           = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "autonomous-agent-patterns"    = @{ main = "ai"; sub = "llm-agents"; score = 100 }
    "aws-penetration-testing"      = @{ main = "security"; sub = "core"; score = 100 }
    "azure-keyvault-keys-ts"       = @{ main = "devops"; sub = "cloud"; score = 100 }
    "azure-keyvault-secrets-ts"    = @{ main = "devops"; sub = "cloud"; score = 100 }
    "beautiful-prose"              = @{ main = "marketing"; sub = "content"; score = 100 }
    "billing-automation"           = @{ main = "business"; sub = "saas"; score = 100 }
    "blockchain-developer"         = @{ main = "programming"; sub = "typescript"; score = 100 }
    "bmad-cis-storytelling"        = @{ main = "marketing"; sub = "content"; score = 100 }
    "brainstorm-experiments-existing" = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "brainstorming"                = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "brand-guidelines"             = @{ main = "design"; sub = "brand-guidelines"; score = 100 }
    "brand-guidelines-anthropic"   = @{ main = "design"; sub = "brand-guidelines"; score = 100 }
    "brand-guidelines-community"   = @{ main = "design"; sub = "brand-guidelines"; score = 100 }
    "bullmq-specialist"            = @{ main = "programming"; sub = "typescript"; score = 100 }
    "c4-architecture-c4-architecture"= @{ main = "design"; sub = "ui-ux"; score = 100 }
    "c4-component"                 = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "canvas-design"                = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "carrier-relationship-management" = @{ main = "business"; sub = "saas"; score = 100 }
    "changelog-automation"         = @{ main = "devops"; sub = "cloud"; score = 100 }
    "churn-prevention"             = @{ main = "marketing"; sub = "email"; score = 100 }
    "cicd-automation-workflow-automate"= @{ main = "devops"; sub = "cloud"; score = 100 }
    "claude-api"                   = @{ main = "ai"; sub = "llm-agents"; score = 100 }
    "claude-code-expert"           = @{ main = "ai"; sub = "llm-agents"; score = 100 }
    "claude-d3js-skill"            = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "claude-monitor"               = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "clerk-auth"                   = @{ main = "frontend"; sub = "react-nextjs"; score = 100 }
    "cloudflare" = @{ main = "devops"; sub = "cloud"; score = 100 }
    "cloudformation-best-practices"= @{ main = "devops"; sub = "cloud"; score = 100 }
    "code-documentation-code-explain" = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "code-review-checklist"        = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "codebase-audit-pre-push"      = @{ main = "security"; sub = "core"; score = 100 }
    "comfyui-gateway"              = @{ main = "backend"; sub = "api-design"; score = 100 }
    "competitor-alternatives"      = @{ main = "marketing"; sub = "content"; score = 100 }
    "competitor-tracking"          = @{ main = "marketing"; sub = "strategy"; score = 100 }
    "content-experimentation-best-practices" = @{ main = "testing"; sub = "automation"; score = 100 }
    "content-strategy"             = @{ main = "marketing"; sub = "content"; score = 100 }
    "context-compression"                 = @{ main = "ai"; sub = "prompting-builder"; score = 100 }
    "context-engineering-advisor"         = @{ main = "ai"; sub = "prompting-builder"; score = 100 }
    "context7-auto-research"       = @{ main = "ai"; sub = "llm-agents"; score = 100 }
    "copywriting"                  = @{ main = "marketing"; sub = "content"; score = 100 }
    "crash-analytics"              = @{ main = "mobile"; sub = "cross-platform"; score = 100 }
    "creem"                        = @{ main = "backend"; sub = "api-design"; score = 100 }
    "creem-heartbeat"              = @{ main = "backend"; sub = "api-design"; score = 100 }
    "cred-omega"                   = @{ main = "security"; sub = "core"; score = 100 }
    "crewai"                       = @{ main = "ai"; sub = "llm-agents"; score = 100 }
    "cro-optimization"             = @{ main = "marketing"; sub = "content"; score = 100 }
    "csharp-pro"                   = @{ main = "programming"; sub = "java"; score = 100 }
    "data-engineering-data-driven-feature" = @{ main = "backend"; sub = "databases"; score = 100 }
    "data-quality-frameworks"      = @{ main = "backend"; sub = "databases"; score = 100 }
    "data-storytelling"            = @{ main = "marketing"; sub = "content"; score = 100 }
    "database-admin"               = @{ main = "backend"; sub = "databases"; score = 100 }
    "dbos-python"                  = @{ main = "programming"; sub = "python"; score = 100 }
    "dbos-typescript"              = @{ main = "programming"; sub = "typescript"; score = 100 }
    "dbt-transformation-patterns"  = @{ main = "backend"; sub = "databases"; score = 100 }
    "debugging-strategies"         = @{ main = "testing"; sub = "automation"; score = 100 }
    "defi-protocol-templates"      = @{ main = "security"; sub = "core"; score = 100 }
    "dependency-upgrade"           = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "deploy" = @{ main = "devops"; sub = "cloud"; score = 100 }
    "deployment-engineer"          = @{ main = "devops"; sub = "cloud"; score = 100 }
    "derivatives-trading-coin-futures" = @{ main = "backend"; sub = "api-design"; score = 100 }
    "derivatives-trading-options"  = @{ main = "backend"; sub = "api-design"; score = 100 }
    "derivatives-trading-portfolio-margin" = @{ main = "backend"; sub = "api-design"; score = 100 }
    "derivatives-trading-usds-futures" = @{ main = "backend"; sub = "api-design"; score = 100 }
    "design-md"                    = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "design-orchestration"         = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "differential-review"          = @{ main = "testing"; sub = "automation"; score = 100 }
    "doc-coauthoring"              = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "documentation-generation-doc-generate"  = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "dummy-dataset"                = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "email-marketing"              = @{ main = "marketing"; sub = "email"; score = 100 }
    "email-sequence"               = @{ main = "marketing"; sub = "email"; score = 100 }
    "enhance-prompt"                      = @{ main = "ai"; sub = "prompting-builder"; score = 100 }
    "ethical-hacking-methodology"  = @{ main = "security"; sub = "core"; score = 100 }
    "ffuf-web-fuzzing"             = @{ main = "security"; sub = "core"; score = 100 }
    "figma-create-design-system-rules" = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "figma-generate-design"        = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "figma-use"                    = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "file-uploads"                 = @{ main = "backend"; sub = "api-design"; score = 100 }
    "find-bugs"                    = @{ main = "testing"; sub = "automation"; score = 100 }
    "finishing-a-development-branch" = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "firebase"                     = @{ main = "devops"; sub = "cloud"; score = 100 }
    "fp-data-transforms"           = @{ main = "programming"; sub = "typescript"; score = 100 }
    "framework-migration-deps-upgrade" = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "free-tool-strategy"           = @{ main = "marketing"; sub = "content"; score = 100 }
    "gemini-api-integration"       = @{ main = "ai"; sub = "llm-agents"; score = 100 }
    "geo-fundamentals"             = @{ main = "marketing"; sub = "content"; score = 100 }
    "gha-security-review"          = @{ main = "security"; sub = "core"; score = 100 }
    "github-actions-templates"     = @{ main = "devops"; sub = "cloud"; score = 100 }
    "gitlab-ci-patterns"           = @{ main = "devops"; sub = "cloud"; score = 100 }
    "gitops-workflow"              = @{ main = "devops"; sub = "cloud"; score = 100 }
    "go-mode"                      = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "google-stitch"                = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "graphql"                      = @{ main = "backend"; sub = "api-design"; score = 100 }
    "gsap-frameworks"              = @{ main = "frontend"; sub = "web-basics"; score = 100 }
    "gsap-plugins"                 = @{ main = "frontend"; sub = "web-basics"; score = 100 }
    "gsap-react"                   = @{ main = "frontend"; sub = "web-basics"; score = 100 }
    "gsap-utils"                   = @{ main = "frontend"; sub = "web-basics"; score = 100 }
    "gtm-motions"                  = @{ main = "marketing"; sub = "strategy"; score = 100 }
    "gtm-strategy"                 = @{ main = "marketing"; sub = "strategy"; score = 100 }
    "hierarchical-agent-memory"    = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "html-injection-testing"       = @{ main = "security"; sub = "core"; score = 100 }
    "hubspot-integration"          = @{ main = "business"; sub = "saas"; score = 100 }
    "identify-assumptions-existing" = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "identify-assumptions-new"     = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "idor-testing"                 = @{ main = "security"; sub = "core"; score = 100 }
    "inngest"                      = @{ main = "programming"; sub = "typescript"; score = 100 }
    "instagram"                    = @{ main = "marketing"; sub = "social"; score = 100 }
    "interview-script"             = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "istio-traffic-management"     = @{ main = "devops"; sub = "docker-k8s"; score = 100 }
    "iterate-pr"                   = @{ main = "testing"; sub = "automation"; score = 100 }
    "javascript-mastery"           = @{ main = "programming"; sub = "typescript"; score = 100 }
    "javascript-pro"               = @{ main = "programming"; sub = "typescript"; score = 100 }
    "julia-pro"                    = @{ main = "programming"; sub = "python"; score = 100 }
    "kotlin-coroutines-expert"     = @{ main = "mobile"; sub = "cross-platform"; score = 100 }
    "langfuse"                     = @{ main = "ai"; sub = "llm-agents"; score = 100 }
    "langgraph"                    = @{ main = "ai"; sub = "llm-agents"; score = 100 }
    "latex-paper-conversion"       = @{ main = "marketing"; sub = "content"; score = 100 }
    "lean-canvas"                  = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "leiloeiro-risco"              = @{ main = "business"; sub = "saas"; score = 100 }
    "lightning-architecture-review"= @{ main = "security"; sub = "core"; score = 100 }
    "linear-claude-skill"          = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "linkerd-patterns"             = @{ main = "devops"; sub = "docker-k8s"; score = 100 }
    "lint-and-validate"                      = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "llm-application-dev-prompt-optimize" = @{ main = "ai"; sub = "prompting-builder"; score = 100 }
    "llm-prompt-optimizer"                = @{ main = "ai"; sub = "prompting-builder"; score = 100 }
    "llm-structured-output"               = @{ main = "ai"; sub = "prompting-builder"; score = 100 }
    "logistics-exception-management" = @{ main = "business"; sub = "saas"; score = 100 }
    "loki-mode"                    = @{ main = "ai"; sub = "llm-agents"; score = 100 }
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
    "market-movers"                = @{ main = "marketing"; sub = "strategy"; score = 100 }
    "marketing-ideas"              = @{ main = "marketing"; sub = "strategy"; score = 100 }
    "mcp-builder"                  = @{ main = "ai"; sub = "llm-agents"; score = 100 }
    "mermaid-expert"               = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "micro-saas-launcher"          = @{ main = "business"; sub = "saas"; score = 100 }
    "microservices-patterns"       = @{ main = "backend"; sub = "api-design"; score = 100 }
    "ml-engineer"                  = @{ main = "ai"; sub = "llm-agents"; score = 100 }
    "modern-javascript-patterns"   = @{ main = "programming"; sub = "typescript"; score = 100 }
    "n8n-code-javascript"          = @{ main = "programming"; sub = "typescript"; score = 100 }
    "n8n-code-python"              = @{ main = "programming"; sub = "python"; score = 100 }
    "native-data-fetching"         = @{ main = "mobile"; sub = "cross-platform"; score = 100 }
    "nerdzao-elite"                = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "nestjs-expert"                = @{ main = "backend"; sub = "api-design"; score = 100 }
    "netlify-cli-and-deploy"       = @{ main = "devops"; sub = "cloud"; score = 100 }
    "netlify-deploy"               = @{ main = "devops"; sub = "cloud"; score = 100 }
    "netlify-edge-functions"       = @{ main = "frontend"; sub = "web-basics"; score = 100 }
    "network-engineer"             = @{ main = "security"; sub = "core"; score = 100 }
    "nextjs-supabase-auth"         = @{ main = "frontend"; sub = "react-nextjs"; score = 100 }
    "nodejs-backend-patterns"      = @{ main = "backend"; sub = "api-design"; score = 100 }
    "nodejsbestpractices"          = @{ main = "backend"; sub = "api-design"; score = 100 }
    "notebooklm"                   = @{ main = "ai"; sub = "llm-agents"; score = 100 }
    "odoo-accounting-setup"        = @{ main = "business"; sub = "saas"; score = 100 }
    "odoo-backup-strategy"         = @{ main = "devops"; sub = "cloud"; score = 100 }
    "odoo-hr-payroll-setup"        = @{ main = "business"; sub = "saas"; score = 100 }
    "odoo-rpc-api"                 = @{ main = "backend"; sub = "api-design"; score = 100 }
    "odoo-sales-crm-expert"        = @{ main = "business"; sub = "saas"; score = 100 }
    "onboarding-cro"               = @{ main = "marketing"; sub = "content"; score = 100 }
    "openapi-spec-generation"      = @{ main = "backend"; sub = "api-design"; score = 100 }
    "opportunity-solution-tree"    = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "page-cro"                     = @{ main = "marketing"; sub = "content"; score = 100 }
    "paid-ads"                     = @{ main = "marketing"; sub = "social"; score = 100 }
    "pakistan-payments-stack"      = @{ main = "business"; sub = "saas"; score = 100 }
    "payment-integration"          = @{ main = "business"; sub = "saas"; score = 100 }
    "paywall-upgrade-cro"          = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "pentest-commands"             = @{ main = "security"; sub = "core"; score = 100 }
    "podcast-generation"           = @{ main = "marketing"; sub = "content"; score = 100 }
    "polars"                       = @{ main = "programming"; sub = "python"; score = 100 }
    "popup-cro"                    = @{ main = "marketing"; sub = "content"; score = 100 }
    "positioning-basics"           = @{ main = "marketing"; sub = "strategy"; score = 100 }
    "positioning-ideas"            = @{ main = "marketing"; sub = "strategy"; score = 100 }
    "pricing-strategy"             = @{ main = "marketing"; sub = "strategy"; score = 100 }
    "prioritize-assumptions"       = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "privilege-escalation-methods" = @{ main = "security"; sub = "core"; score = 100 }
    "production-scheduling"        = @{ main = "business"; sub = "saas"; score = 100 }
    "programmatic-seo"             = @{ main = "marketing"; sub = "content"; score = 100 }
    "prompt-caching"                      = @{ main = "ai"; sub = "prompting-builder"; score = 100 }
    "prompt-engineer"                     = @{ main = "ai"; sub = "prompting-builder"; score = 100 }
    "prompt-engineering"                  = @{ main = "ai"; sub = "prompting-builder"; score = 100 }
    "prompt-engineering-patterns"         = @{ main = "ai"; sub = "prompting-builder"; score = 100 }
    "prompt-library"                      = @{ main = "ai"; sub = "prompting-builder"; score = 100 }
    "query"                        = @{ main = "backend"; sub = "databases"; score = 100 }
    "rating-prompt-strategy"              = @{ main = "ai"; sub = "prompting-builder"; score = 100 }
    "react:components"             = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "read-file"                    = @{ main = "backend"; sub = "databases"; score = 100 }
    "read-memories"                = @{ main = "backend"; sub = "databases"; score = 100 }
    "referral-program"             = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "render-automation"            = @{ main = "devops"; sub = "cloud"; score = 100 }
    "returns-reverse-logistics"    = @{ main = "business"; sub = "saas"; score = 100 }
    "revops"                       = @{ main = "business"; sub = "saas"; score = 100 }
    "risk-metrics-calculation"     = @{ main = "business"; sub = "saas"; score = 100 }
    "saas-mvp-launcher"            = @{ main = "business"; sub = "saas"; score = 100 }
    "sankhya-dashboard-html-jsp-custom-best-pratices" = @{ main = "backend"; sub = "api-design"; score = 100 }
    "sast-configuration"           = @{ main = "security"; sub = "core"; score = 100 }
    "schema-markup"                = @{ main = "marketing"; sub = "content"; score = 100 }
    "scientific-writing"           = @{ main = "marketing"; sub = "content"; score = 100 }
    "screenshot-optimization"      = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "scroll-experience"            = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "secrets-management"           = @{ main = "security"; sub = "core"; score = 100 }
    "seo-image-gen"                = @{ main = "marketing"; sub = "content"; score = 100 }
    "seo-programmatic"             = @{ main = "marketing"; sub = "content"; score = 100 }
    "service-mesh-expert"          = @{ main = "devops"; sub = "docker-k8s"; score = 100 }
    "shodan-reconnaissance"        = @{ main = "security"; sub = "core"; score = 100 }
    "shopify-development"          = @{ main = "backend"; sub = "api-design"; score = 100 }
    "site-architecture"            = @{ main = "marketing"; sub = "content"; score = 100 }
    "skill-authoring-workflow"               = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "skill-check"                            = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "skill-creator"                          = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "skill-creator-ms"                       = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "skill-developer"                        = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "skill-improver"                         = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "skill-installer"                        = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "skill-router"                           = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "skill-scanner"                          = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "skill-seekers"                          = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "skill-sentinel"                         = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "skill-writer"                           = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "slack-messaging"              = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "spot"                         = @{ main = "backend"; sub = "api-design"; score = 100 }
    "sred-work-summary"            = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "stripe-integration"           = @{ main = "business"; sub = "saas"; score = 100 }
    "supabase-automation"          = @{ main = "backend"; sub = "databases"; score = 100 }
    "sveltekit"                    = @{ main = "frontend"; sub = "react-nextjs"; score = 100 }
    "tdd-orchestrator"             = @{ main = "testing"; sub = "automation"; score = 100 }
    "tdd-workflow"                 = @{ main = "testing"; sub = "automation"; score = 100 }
    "tdd-workflows-tdd-cycle"      = @{ main = "testing"; sub = "automation"; score = 100 }
    "tdd-workflows-tdd-green"      = @{ main = "testing"; sub = "automation"; score = 100 }
    "tdd-workflows-tdd-red"        = @{ main = "testing"; sub = "automation"; score = 100 }
    "tdd-workflows-tdd-refactor"   = @{ main = "testing"; sub = "automation"; score = 100 }
    "test-scenarios"               = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "testimonial-collector"        = @{ main = "marketing"; sub = "content"; score = 100 }
    "theme-factory"                = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "threejs-animation"            = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "threejs-fundamentals"         = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "threejs-geometry"             = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "threejs-interaction"          = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "threejs-lighting"             = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "threejs-loaders"              = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "threejs-materials"            = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "threejs-postprocessing"       = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "threejs-skills"               = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "threejs-textures"             = @{ main = "design"; sub = "ui-ux"; score = 100 }
    "tinybird-cli-guidelines"      = @{ main = "backend"; sub = "databases"; score = 100 }
    "trigger-dev"                  = @{ main = "programming"; sub = "typescript"; score = 100 }
    "user-story"                   = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "user-story-mapping"           = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "user-story-mapping-workshop"  = @{ main = "business"; sub = "product-strategy"; score = 100 }
    "uv-package-manager"           = @{ main = "programming"; sub = "python"; score = 100 }
    "varlock"                      = @{ main = "security"; sub = "core"; score = 100 }
    "varlock-claude-skill"         = @{ main = "security"; sub = "core"; score = 100 }
    "vercel-automation"            = @{ main = "devops"; sub = "cloud"; score = 100 }
    "vercel-deployment"            = @{ main = "devops"; sub = "cloud"; score = 100 }
    "verification-before-completion"         = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "vexor"                        = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "vexor-cli"                    = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "wds-5-agentic-development"    = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
    "web-artifacts-builder"        = @{ main = "frontend"; sub = "react-nextjs"; score = 100 }
    "windows-privilege-escalation" = @{ main = "security"; sub = "core"; score = 100 }
    "windows-vm"                   = @{ main = "devops"; sub = "docker-k8s"; score = 100 }
    "wordpress-penetration-testing"= @{ main = "security"; sub = "core"; score = 100 }
    "wordpress-plugin-development" = @{ main = "backend"; sub = "api-design"; score = 100 }
    "writing-skills"                         = @{ main = "ai"; sub = "skills-factory"; score = 100 }
    "yes-md"                       = @{ main = "productivity"; sub = "workflow-automation"; score = 100 }
}

$EXCLUDE_CATEGORY_PATTERNS = [ordered]@{
    "games" = @("game", "games", "gaming", "gameplay", "unity", "unreal", "godot", "2d-games", "3d-games", "pc-games", "web-games", "multiplayer", "game-audio", "game-design")
    "law-legal" = @("law", "legal", "lawyer", "attorney", "litigation", "court", "jurisdiction", "advogado", "leiloeiro", "juridico", "nda", "contract-templates")
    "medicine-medical" = @("medicine", "medical", "clinical", "healthcare", "diagnosis", "patient", "hospital", "health-analyzer", "tcm-constitution", "oral-health", "skin-health", "mental-health", "sexual-health", "fitness-analyzer", "sleep-analyzer", "nutrition-analyzer", "rehabilitation", "weightloss", "wellally", "occupational-health", "family-health", "travel-health", "goal-analyzer", "fda-food-safety", "fda-medtech")
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
    "chemistry"
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
    
    if ($Path -match 'skill-manage[\\/]src[\\/]([^\\/]+)') {
        return "external:$($matches[1])"
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
            index = "skills-index.json"
            catalog = "skills-catalog.csv"
        }
    }

    $indexItems = foreach ($skill in $Skills | Sort-Object id) {
        [PSCustomObject][ordered]@{
            id = $skill.id
            triggers = @($skill.triggers | Select-Object -First 5)
            src = $skill.src
            primary_hub = $skill.primary_hub
            is_primary = [bool] $skill.is_primary
            match_score = [int] $skill.match_score
        }
    }

    $catalogRows = foreach ($skill in $Skills | Sort-Object id) {
        [PSCustomObject][ordered]@{
            id = $skill.id
            description = $skill.description
            path = $skill.path
            triggers = (@($skill.triggers) -join ';')
            src = $skill.src
            primary_hub = $skill.primary_hub
            assigned_hubs = (@($skill.assigned_hubs) -join ';')
            match_score = [int] $skill.match_score
            is_primary = [bool] $skill.is_primary
        }
    }

    # Run quality validation if enabled
    if ($ValidateQuality) {
        # Simply convert manifest hashtable to PSCustomObject (PowerShell handles nested objects)
        $manifestObj = [PSCustomObject]$manifest
        $report = New-ValidationReport -SubHubKey "$MainHub/$SubHub" -Manifest $manifestObj -CatalogItems $catalogRows -WorkflowText "" -RepoRoot $RepoRoot
        Write-ValidationReport -Report $report
        if (-not $report.passed) {
            Write-Host "[ERROR] Quality validation failed for $MainHub/$SubHub. Fix issues above before proceeding." -ForegroundColor Red
            return $false
        }
    }

    if (-not $DryRun) {
        mkdir -Path $OutPath -Force | Out-Null
        Write-FileUtf8NoBom -Path (Join-Path $OutPath "SKILL.md") -Content $skillMd
        Write-FileUtf8NoBom -Path (Join-Path $OutPath "skills-manifest.json") -Content (($manifest | ConvertTo-Json -Depth 8) + [Environment]::NewLine)
        Write-FileUtf8NoBom -Path (Join-Path $OutPath "skills-index.json") -Content (($indexItems | ConvertTo-Json -Depth 6) + [Environment]::NewLine)
        Write-FileUtf8NoBom -Path (Join-Path $OutPath "skills-catalog.csv") -Content ((($catalogRows | ConvertTo-Csv -NoTypeInformation) -join [Environment]::NewLine) + [Environment]::NewLine)
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
    Confirm-OrExit -Message "Proceed with generating aggregated outputs in '$OutputDir'?"
}

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

$MarketingSkillIdSet = @{}
if ($MarketingFirst) {
    $marketingSkillRoot = Join-Path $srcRootPath "marketingskills/skills"
    if (Test-Path $marketingSkillRoot) {
        $marketingSkillDirs = Get-ChildItem -Path $marketingSkillRoot -Directory -ErrorAction SilentlyContinue
        foreach ($dir in $marketingSkillDirs) {
            $MarketingSkillIdSet[$dir.Name.ToLowerInvariant()] = $true
        }
        Write-Host "[INFO] Marketing-first id set loaded: $($MarketingSkillIdSet.Count) skills" -ForegroundColor DarkCyan
    }
}
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
    $skillIdLower = $skill.id.ToLower().Trim()
    if ($MANUAL_HUB_OVERRIDES.ContainsKey($skillIdLower)) {
        $override = $MANUAL_HUB_OVERRIDES[$skillIdLower]
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

    # Marketing-first: prioritize skills coming from external:marketingskills under marketing/*.
    if ($MarketingFirst) {
        $srcLower = ([string]$skill.src).ToLowerInvariant()
        $pathLower = ([string]$skill.path).ToLowerInvariant() -replace '\\', '/'
        $skillIdLower = ([string]$skill.id).ToLowerInvariant()
        $isMarketingSkill = ($srcLower -eq "external:marketingskills") -or ($pathLower -like "*/src/marketingskills/*") -or $MarketingSkillIdSet.ContainsKey($skillIdLower)

        if ($isMarketingSkill) {
            $marketingDefs = @{}
            if ($SUB_HUB_DEFINITIONS.ContainsKey("marketing")) {
                $marketingDefs["marketing"] = $SUB_HUB_DEFINITIONS["marketing"]
            }

            if ($marketingDefs.Count -gt 0) {
                $marketingAssignments = @(Get-SkillAssignments -Skill $skill -SubHubDefs $marketingDefs -EnableMultiHub:$false -PrimaryThreshold 1 -SecondaryThreshold 1 -MaxAssignments 1)

                if ($marketingAssignments.Count -gt 0) {
                    # Ensure marketing assignment is primary for marketingskills source.
                    $assignments = @($marketingAssignments)
                }
                elseif ($assignments.Count -gt 0) {
                    # Fallback if no match in marketing defs but we still want marketing-first behavior.
                    $assignments = @([PSCustomObject]@{
                        main = "marketing"
                        sub = "content"
                        key = "marketing-content"
                        score = [int]$assignments[0].score
                    })
                }
            }
        }
    }

    if ($assignments.Count -eq 0) {
        # No general/misc fallback — route unclassified to productivity as last resort
        Write-Host "[WARN] Skill '$($skill.id)' unclassified — routing to productivity/workflow-automation" -ForegroundColor Yellow
        $assignments = @([PSCustomObject]@{
            main = "productivity"
            sub = "workflow-automation"
            key = "productivity-workflow-automation"
            score = 4
        })
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
    
    # Skip hubs with fewer than minimum required skills, EXCEPT if protected
    $PROTECTED_HUBS = @("design-brand-guidelines", "design-design-thinking")
    if ($uniqueSkills.Count -lt $MIN_SKILLS_PER_HUB -and $PROTECTED_HUBS -notcontains $subHubKey) {
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

if ($SyncToTools -and -not $DryRun) {
    # Find sync-hubs.ps1 in the same directory as this script
    $syncScript = Join-Path $PSScriptRoot "sync-hubs.ps1"
    if (Test-Path $syncScript) {
        Write-Host ""
        Write-Host "[INFO] Automatically syncing to tools..." -ForegroundColor Cyan
        powershell -ExecutionPolicy Bypass -File $syncScript -Force
    }
}
