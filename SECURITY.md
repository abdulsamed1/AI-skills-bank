# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Security Architecture & Invariants

`skills-bank` serves as an ultra-lightweight routing and aggregation layer for AI agent workflows. It is built with the following security considerations:

1. **Sandboxed Skill Ingestion**:
   - Skills are ingested as declarative Markdown contracts (`SKILL.md`). The pipeline does not execute arbitrary shell commands or untrusted binaries from third-party repositories during aggregation or synchronization.
   - Raw clone caches in `lib/` are kept isolated and are strictly excluded from version control.

2. **Atomic Directory & Symlink Operations**:
   - All file and directory synchronizations use atomic temp-file creation and atomic renames (`atomicity.rs`). This prevents file-descriptor race conditions, symlink-following vulnerabilities, and corrupted partial states across Windows, macOS, and Linux.

3. **Zero Credential Exposure**:
   - All LLM provider keys (Groq, Anthropic, OpenAI, Cerebras, etc.) are ingested exclusively via environment variables (`.env`) or local proxies. No tokens, keys, or credentials are ever written into generated hub manifests, routing tables (`routing.csv`), or aggregated router files.

## Reporting a Vulnerability

If you identify a security vulnerability or potential sensitive data leak in `skills-bank`, please do **NOT** open a public issue.

Instead, please submit a private report:
- **GitHub Security Advisory**: [Report a Vulnerability](https://github.com/abdulsamed1/AI-skills-bank/security/advisories/new)
- Please include:
  - A summary of the vulnerability
  - Clear steps or a script to reproduce
  - The potential security impact

All valid disclosures will be investigated and addressed promptly.
