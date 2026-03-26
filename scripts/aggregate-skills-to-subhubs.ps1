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
    [ValidateRange(1, 5)]
    [int] $MaxHubsPerSkill = 3,
    [ValidateRange(1, 20)]
    [int] $PrimaryMinScore = 4,
    [ValidateRange(1, 20)]
    [int] $SecondaryMinScore = 6
)

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
    
    $srcHubsDir = Join-Path $RepoRoot "AI-skills-bank/hub-skills"
    $OutputDir = Join-Path $RepoRoot "AI-skills-bank/skills-aggregated"
    $FallbackSkillRoots = @(
        (Join-Path $RepoRoot "_bmad"),
        (Join-Path $RepoRoot "AI-skills-bank/src")
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
}

# ============================================================================
# MAIN AGGREGATION LOGIC
# ============================================================================

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
    Write-Host "[WARN] No hub-manifest.json found in $srcHubsDir; using fallback roots: $($FallbackSkillRoots -join ', ')" -ForegroundColor Yellow
    $allSkills = Load-SkillsFromFiles -Roots $FallbackSkillRoots
}

Write-Host "[✓] Loaded $($allSkills.Count) skills from $(($allSkills.src | Select-Object -Unique).Count) srcs"
Write-Host ""

# Categorize into sub-hubs
Write-Host "[INFO] Step 2: Categorizing skills into sub-hubs..."
$subHubMap = @{}
$unmatchedSkills = @()
$multiAssignedSkillCount = 0
$totalAssignments = 0

foreach ($skill in $allSkills) {
    $assignments = @(Get-SkillAssignments -Skill $skill -SubHubDefs $SUB_HUB_DEFINITIONS -EnableMultiHub:$AllowMultiHub -PrimaryThreshold $PrimaryMinScore -SecondaryThreshold $SecondaryMinScore -MaxAssignments $MaxHubsPerSkill)

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
}

Write-Host "[✓] Categorized into $($subHubMap.Count) sub-hubs (unmatched routed: $($unmatchedSkills.Count), multi-assigned skills: $multiAssignedSkillCount, total assignments: $totalAssignments)"
Write-Host ""

# Generate BMAD-style files for each sub-hub
Write-Host "[INFO] Step 3: Generating BMAD-style sub-hubs (SKILL router + workflow + catalog)..."

$routingIndex = @()

foreach ($subHubKey in $subHubMap.Keys) {
    $subHubData = $subHubMap[$subHubKey]
    $subHubDef = $SUB_HUB_DEFINITIONS[$subHubData.main][$subHubData.sub]
    
    # Deduplicate
    $uniqueSkills = Deduplicate-Skills -Skills $subHubData.skills
    
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

    $lockPayload = [ordered]@{
        generated_at = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssK")
        src_repo_mode = $srcRepoMode
        selected_src_repos = @($SelectedsrcRepos)
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
Write-Host "[INFO]   Sub-hubs created: $($subHubMap.Count)"
Write-Host "[INFO]   Total skills: $($allSkills.Count)"
Write-Host "[INFO]   Output dir: $OutputDir"
Write-Host "[INFO] ============================================"
