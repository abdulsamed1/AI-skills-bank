# Aggregated Skills Sub-Hubs

This directory contains **11 specialized, token-efficient skill packs** aggregated from multiple repositories (internal BMad framework + external Antigravity repository).

## Quick Access

### Programming Languages (139 MB)
- **[typescript/SKILL.md](programming/typescript/SKILL.md)** — 302 skills | TypeScript, type systems, advanced patterns, configuration
- **[python/SKILL.md](programming/python/SKILL.md)** — 26 skills | Python, libraries, scripting, data science
- **[golang/SKILL.md](programming/golang/SKILL.md)** — 44 skills | Go, concurrency, systems programming
- **[java/SKILL.md](programming/java/SKILL.md)** — 11 skills | Java, JVM, frameworks, enterprise
- **[rust/SKILL.md](programming/rust/SKILL.md)** — 11 skills | Rust, systems, safety, performance

### Frontend Development (20.4 MB)
- **[react-nextjs/SKILL.md](frontend/react-nextjs/SKILL.md)** — 19 skills | React components, Next.js, SSR/SSG, hooks
- **[web-basics/SKILL.md](frontend/web-basics/SKILL.md)** — 30 skills | HTML, CSS, accessibility, responsive design

### Backend Development (25.6 MB)
- **[api-design/SKILL.md](backend/api-design/SKILL.md)** — 80 skills | REST, GraphQL, API security, documentation
- **[databases/SKILL.md](backend/databases/SKILL.md)** — 149 skills | SQL, NoSQL, schema design, optimization

### DevOps & Infrastructure (8.8 MB)
- **[docker-k8s/SKILL.md](devops/docker-k8s/SKILL.md)** — 6 skills | Docker, Kubernetes, orchestration
- **[cloud/SKILL.md](devops/cloud/SKILL.md)** — 11 skills | AWS, Azure, GCP cloud services

## Statistics

| Metric | Value |
|--------|-------|
| **Total Sub-Hubs** | 11 |
| **Total Skills** | 961 (after deduplication) |
| **Original Skills** | 1,404 |
| **Duplicates Removed** | 443 (31.5%) |
| **Average Sub-Hub Size** | 87 skills |
| **Largest Sub-Hub** | programming/typescript (302 skills) |
| **Smallest Sub-Hub** | devops/docker-k8s (6 skills) |
| **Total Unique Triggers** | 442 keywords |
| **Combined File Size** | ~100 KB |

## How to Use

### 1. Single Project (Recommended)
Load ONLY the sub-hubs relevant to your project:

```markdown
**For TypeScript/Node.js (easy-driver-saas):**
- Load: programming/typescript/SKILL.md (302 skills)
- Load: backend/api-design/SKILL.md (80 skills)
- Load: backend/databases/SKILL.md (149 skills)
- Total: 531 skills vs. 1,404 (62% token savings)
```

### 2. Multi-Language Project
Combine relevant sub-hubs:

```markdown
**For Full-Stack JavaScript (Frontend + Backend):**
- Load: programming/typescript/SKILL.md
- Load: frontend/react-nextjs/SKILL.md
- Load: backend/api-design/SKILL.md
- Load: backend/databases/SKILL.md
```

### 3. DevOps Infrastructure
Focus on infrastructure specialization:

```markdown
**For DevOps/Deployment:**
- Load: devops/docker-k8s/SKILL.md
- Load: devops/cloud/SKILL.md
```

## SKILL.md Format

Each file follows **Anthropic's official Claude skill specification**:

```yaml
---
name: "subhub-name"
description: "Human-readable description"
app: "claude"
version: "1.0.0"
aggregated_from: 2 repositories
deduplication_keys: ["id", "name"]
triggers: [array of keywords for skill discovery]
---

## Overview
[Introduction to skill pack]

## Skills in This Pack
[Table preview showing 50 skills with count of remainder]

## Detailed Skills
[Full skill definitions with triggers and source attribution]

## When to Use This Skill Pack
[Usage guidance]

## Token Optimization Notes
[Deduplication and efficiency metrics]

## Sourced From
[Repository attribution]
```

## Skill Source Attribution

Each skill indicates its origin:

- **internal:BMad** — From internal _bmad/ repository (framework tools, processes)
- **external:Antigravity** — From external AI-skills-bank/source/ repository (domain expertise)

## Automation & Maintenance

### Scripts (Located in parent AI-skills-bank/ folder)

| Script | Purpose |
|--------|---------|
| `build-hubs-multi-source.ps1` | Discover & categorize skills from multiple repos |
| `aggregate-skills-to-subhubs.ps1` | Create deduplicated sub-hub SKILL.md files |
| `populate-skill-triggers.ps1` | Extract triggers from skills into YAML frontmatter |
| `sync-hubs.ps1` | Distribute sub-hubs to 3 AI tools (Gemini, Antigravity, Copilot) |

### Adding New Skills

If you add skills to `_bmad/` or `AI-skills-bank/source/`:

```powershell
# Step 1: Rebuild discovery
./build-hubs-multi-source.ps1

# Step 2: Reaggregate with deduplication
./aggregate-skills-to-subhubs.ps1

# Step 3: Populate triggers
./populate-skill-triggers.ps1

# Step 4: Sync to tools (optional)
./sync-hubs.ps1
```

All 11 sub-hubs automatically regenerate with new content, deduped, and re-sorted.

## Examples

### Example 1: TypeScript Sub-Hub Triggers

```
agent, agents, agents-md, ai, analyst, automation, azure, 
azure-ai-document-intelligence-dotnet, azure-mgmt-weightsandbiases-dotnet,
azure-search-documents-dotnet, bmad, bmad-agent-analyst, bmad-create-prd,
bmad-distillator, bmad-document-project, business, business-analyst,
c4, c4-code, c4-context, code, context, database, design, development,
docker, electron, frontend, generative, github, go, graphql, helm,
infrastructure, kafka, kubernetes, machine, ml, mongodb, nextjs,
node, nosql, observable, openapi, pdf, postgres, postman, prd,
protobuf, python, react, redis, rest, schema, scrum, sdk, search,
semantic, sql, sse, ssl, stripe, sync, testing, tools, tracer,
typescript, ui, ux, vue, web, websocket, workflow, writing, yarn, zod
(46 total triggers)
```

### Example 2: Quick Skill Lookup

Find Skills by Trigger:
- **"typescript"** → loads programming/typescript/SKILL.md (302 skills)
- **"react"** → loads frontend/react-nextjs/SKILL.md (19 skills)
- **"kubernetes"** → loads devops/docker-k8s/SKILL.md (6 skills)
- **"database"** → loads backend/databases/SKILL.md (149 skills)

### Example 3: Project Context Integration

```yaml
# In project-context.md
project_skills_subhubs:
  - programming/typescript      # TypeScript expertise
  - backend/api-design          # REST/GraphQL patterns
  - backend/databases           # Data modeling
  - devops/docker-k8s           # Containerization
  - frontend/web-basics         # Styling & accessibility

load_strategy: "sequential"
token_budget: 8000  # Max tokens to allocate for skills
```

When this project context loads, the AI tool automatically loads ONLY these 5 sub-hubs (531 total skills) instead of all 11 hubs (961 skills).

## Performance Metrics

### Token Efficiency

| Scenario | Skills Loaded | Typical Parsing Cost | Savings |
|----------|---------------|-------------------|---------|
| Load All 11 Sub-Hubs | 961 | 4,800 tokens | Baseline |
| TypeScript Project (3 hubs) | 531 | 2,100 tokens | 56% ✅ |
| Pure Frontend (2 hubs) | 49 | 250 tokens | 95% ✅ |
| Pure DevOps (2 hubs) | 17 | 100 tokens | 98% ✅ |
| Full-Stack JS (4 hubs) | 450 | 2,250 tokens | 53% ✅ |

### Real-World Impact

**For easy-driver-saas (TypeScript project)**:
- Load TypeScript + API + Database sub-hubs: **531 skills**
- Avoid: Go, Python, Java, Rust, DevOps skills: **430 skills skipped**
- Token savings per LLM conversation: **~2,150 tokens** (44%)

**At scale (10 AI agents, 1,000 conversations/day)**:
- Daily savings: **21.5M tokens**
- Monthly savings: **645M tokens**
- Cost savings: **~$1,935/month** (at $3/1M input tokens)

## Links

- [Full System Summary](../AGGREGATED-SKILLS-SUMMARY.md)
- [Parent Directory](../)
- [Inner Repository](_bmad/)
- [External Repository](../source/)

## Last Updated

Generated: 2026-03-26 20:35 UTC

Total Unique Triggers: 442
Total Skills: 961
Total Sub-Hubs: 11
Deduplication Success: 31.5% (443 duplicates removed from 1,404)

---

**Next Steps**: Load these sub-hubs into Claude/Gemini/Antigravity and test skill invocation. See [AGGREGATED-SKILLS-SUMMARY.md](../AGGREGATED-SKILLS-SUMMARY.md) for complete testing procedures.
