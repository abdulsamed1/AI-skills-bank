use crate::components::aggregator::rules::{CSV_COLUMNS, VALID_HUBS};
use crate::components::manifest::RepoManifest;
use crate::components::CommandResult;
use crate::error::SkillManageError;
use crossterm::style::{Color, Stylize};
use minijinja::{context, Environment};
use rayon::prelude::*;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticIssue {
    pub description: String,
    pub location: Option<String>,
    pub current: Option<String>,
    pub should_be: Option<String>,
    pub why: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiagnosticStatus {
    Pass,
    Warn {
        issues: Vec<DiagnosticIssue>,
        fix: String,
    },
    Fail {
        issues: Vec<DiagnosticIssue>,
        fix: String,
    },
}

pub trait Check: Sync + Send {
    fn name(&self) -> &str;
    fn run(&self) -> DiagnosticStatus;
}

// 1. Manifest Exists Check
struct ManifestExistsCheck;
impl Check for ManifestExistsCheck {
    fn name(&self) -> &str {
        "Manifest File (repos.json)"
    }
    fn run(&self) -> DiagnosticStatus {
        let path = Path::new("repos.json");
        if path.exists() {
            match RepoManifest::load(path) {
                Ok(_) => DiagnosticStatus::Pass,
                Err(e) => DiagnosticStatus::Fail {
                    issues: vec![DiagnosticIssue {
                        description: format!("Found but unparseable: {}", e),
                        location: Some("repos.json".to_string()),
                        current: None,
                        should_be: None,
                        why: "Manifest must be valid JSON following the RepoManifest schema"
                            .to_string(),
                    }],
                    fix: "Check for JSON syntax errors or unknown fields in.skill-manage-cli-config.json".to_string(),
                },
            }
        } else {
            DiagnosticStatus::Fail {
                issues: vec![DiagnosticIssue {
                    description: "repos.json not found".to_string(),
                    location: None,
                    current: None,
                    should_be: Some("repos.json in root".to_string()),
                    why: "The manifest file defines which skill repositories to manage".to_string(),
                }],
                fix: "Create a.skill-manage-cli-config.json manifest in the root".to_string(),
            }
        }
    }
}

// 2. Source Directory Check
struct SourceDirCheck;
impl Check for SourceDirCheck {
    fn name(&self) -> &str {
        "Repository Cache (lib/)"
    }
    fn run(&self) -> DiagnosticStatus {
        if Path::new("lib").is_dir() {
            DiagnosticStatus::Pass
        } else {
            DiagnosticStatus::Fail {
                issues: vec![DiagnosticIssue {
                    description: "lib/ directory not found".to_string(),
                    location: None,
                    current: None,
                    should_be: Some("lib/ directory containing git clones".to_string()),
                    why: "The source directory acts as a local cache for all skills".to_string(),
                }],
                fix: "Run 'fetch' to download skills".to_string(),
            }
        }
    }
}

// 3. CSV Schema Check
struct CsvSchemaCheck;
impl Check for CsvSchemaCheck {
    fn name(&self) -> &str {
        "Aggregated Manifest (hub-manifests.csv) Schema"
    }
    fn run(&self) -> DiagnosticStatus {
        let path = Path::new("hub-manifests.csv");
        if !path.exists() {
            return DiagnosticStatus::Warn {
                issues: vec![DiagnosticIssue {
                    description: "hub-manifests.csv not found".to_string(),
                    location: None,
                    current: None,
                    should_be: Some("hub-manifests.csv in root".to_string()),
                    why: "This file is the primary routing table for agents".to_string(),
                }],
                fix: "Run 'aggregate' to generate the routing table".to_string(),
            };
        }

        let mut rdr = match csv::Reader::from_path(path) {
            Ok(r) => r,
            Err(e) => {
                return DiagnosticStatus::Fail {
                    issues: vec![DiagnosticIssue {
                        description: format!("Failed to open CSV: {}", e),
                        location: Some("hub-manifests.csv".to_string()),
                        current: None,
                        should_be: None,
                        why: "CSV must be readable to validate its schema".to_string(),
                    }],
                    fix: "Check file permissions or if the file is corrupted".to_string(),
                }
            }
        };

        let mut issues = Vec::new();

        // Check header
        let headers = rdr.headers().unwrap();
        if headers.len() != CSV_COLUMNS.len() {
            issues.push(DiagnosticIssue {
                description: "Header column count mismatch".to_string(),
                location: Some("Line 1".to_string()),
                current: Some(format!("{} columns", headers.len())),
                should_be: Some(format!("{} columns", CSV_COLUMNS.len())),
                why: "Downstream tools expect an exact CSV schema".to_string(),
            });
        } else {
            for (i, col) in CSV_COLUMNS.iter().enumerate() {
                let current_col = headers.get(i).unwrap_or_default();
                if current_col != *col {
                    issues.push(DiagnosticIssue {
                        description: format!("Header column {} mismatch", i),
                        location: Some(format!("Column {}", i + 1)),
                        current: Some(current_col.to_string()),
                        should_be: Some(col.to_string()),
                        why: "Column names must match the system specification exactly".to_string(),
                    });
                }
            }
        }

        if !issues.is_empty() {
            return DiagnosticStatus::Fail {
                issues,
                fix: "Regenerate the manifest using the 'aggregate' command".to_string(),
            };
        }

        // Check rows (limit to 10 errors)
        let mut row_count = 0;
        for result in rdr.records() {
            row_count += 1;
            let record = result.unwrap();

            let hub = record.get(0).unwrap_or_default();
            if !VALID_HUBS.contains(&hub) {
                issues.push(DiagnosticIssue {
                    description: format!("Invalid hub found: '{}'", hub),
                    location: Some(format!("Line {}", row_count + 1)),
                    current: Some(hub.to_string()),
                    should_be: Some(format!("One of: {:?}", VALID_HUBS)),
                    why: "Hub names are strictly limited to the 11 verified hubs".to_string(),
                });
            }

            if issues.len() >= 10 {
                break;
            }
        }

        if issues.is_empty() {
            DiagnosticStatus::Pass
        } else {
            DiagnosticStatus::Fail {
                issues,
                fix: "Check the SKILL.md frontmatter for valid hub names and run aggregate"
                    .to_string(),
            }
        }
    }
}

// 4. Repo Integrity Check (Parallel)
struct RepoIntegrityCheck;
impl Check for RepoIntegrityCheck {
    fn name(&self) -> &str {
        "Repository Integrity"
    }
    fn run(&self) -> DiagnosticStatus {
        let path = Path::new("repos.json");
        let manifest = match RepoManifest::load(path) {
            Ok(m) => m,
            Err(_) => {
                return DiagnosticStatus::Fail {
                    issues: vec![DiagnosticIssue {
                        description: "Skipped due to manifest error".to_string(),
                        location: None,
                        current: None,
                        should_be: None,
                        why: "Integrity check requires a valid.skill-manage-cli-config.json".to_string(),
                    }],
                    fix: "Fix.skill-manage-cli-config.json first".to_string(),
                }
            }
        };

        let missing: Vec<String> = manifest
            .repositories
            .par_iter()
            .filter(|repo| !Path::new("lib").join(&repo.name).exists())
            .map(|repo| repo.name.clone())
            .collect();

        if missing.is_empty() {
            DiagnosticStatus::Pass
        } else {
            DiagnosticStatus::Fail {
                issues: vec![DiagnosticIssue {
                    description: format!("{} missing repositories", missing.len()),
                    location: Some("lib/ directory".to_string()),
                    current: Some(format!("Missing: {}", missing.join(", "))),
                    should_be: Some("All repos from manifest present in lib/".to_string()),
                    why: "Skills must be locally cached before they can be synchronized or routed"
                        .to_string(),
                }],
                fix: "Run 'fetch' to download missing repos".to_string(),
            }
        }
    }
}

// 5. Master Router Check
struct MasterRouterCheck;
impl Check for MasterRouterCheck {
    fn name(&self) -> &str {
        "Skills Bank Master Router (SKILL.md)"
    }
    fn run(&self) -> DiagnosticStatus {
        // Prefer the dynamic AGENTS.md as the canonical entrypoint.
        let agents_candidates = [
            Path::new("skills-aggregated/AGENTS.md"),
            Path::new("skill-manage/skills-aggregated/AGENTS.md"),
            Path::new("AGENTS.md"),
            Path::new("../AGENTS.md"),
            Path::new("skill-manage/AGENTS.md"),
        ];

        if agents_candidates.iter().any(|p| p.exists()) {
            return DiagnosticStatus::Pass;
        }

        // Fallback: still accept legacy SKILL.md if present (backward compatibility)
        let candidates = [
            Path::new("skills-aggregated/SKILL.md"),
            Path::new("skill-manage/skills-aggregated/SKILL.md"),
        ];

        let path = candidates.iter().find(|p| p.exists());
        if path.is_none() {
            return DiagnosticStatus::Warn {
                issues: vec![DiagnosticIssue {
                    description: "Master router not found (AGENTS.md or SKILL.md)".to_string(),
                    location: Some("skills-aggregated/".to_string()),
                    current: None,
                    should_be: Some("AGENTS.md or SKILL.md file present".to_string()),
                    why: "The master router is the entry point for all skill discovery".to_string(),
                }],
                fix: "Ensure skills-aggregated directory is synchronized".to_string(),
            };
        }

        let path = path.unwrap();

        match std::fs::read_to_string(path) {
            Ok(content) => {
                if content.contains("11 HUBS ONLY") {
                    DiagnosticStatus::Pass
                } else {
                    DiagnosticStatus::Warn {
                        issues: vec![DiagnosticIssue {
                            description: "Guard rules missing".to_string(),
                            location: Some(path.to_string_lossy().to_string()),
                            current: None,
                            should_be: Some("Contains '11 HUBS ONLY' section".to_string()),
                            why: "Guard rules prevent hallucination by explicitly limiting hubs"
                                .to_string(),
                        }],
                        fix: "Check skills-bank-router template for '11 HUBS ONLY' section"
                            .to_string(),
                    }
                }
            }
            Err(e) => DiagnosticStatus::Fail {
                issues: vec![DiagnosticIssue {
                    description: format!("Failed to read master router: {}", e),
                    location: None,
                    current: None,
                    should_be: None,
                    why: "Master router must be readable to verify contents".to_string(),
                }],
                fix: "Check file permissions".to_string(),
            },
        }
    }
}

// 6. V11 Sub-hub routing.csv presence check
struct V11SubHubRoutingCheck;
impl Check for V11SubHubRoutingCheck {
    fn name(&self) -> &str {
        "V11 Sub-hub Routing Coverage"
    }

    fn run(&self) -> DiagnosticStatus {
        let root_candidates = [
            Path::new("skills-aggregated"),
            Path::new("skill-manage/skills-aggregated"),
        ];

        let root = match root_candidates.iter().find(|p| p.exists()) {
            Some(p) => *p,
            None => {
                return DiagnosticStatus::Warn {
                    issues: vec![DiagnosticIssue {
                        description: "skills-aggregated not found".to_string(),
                        location: None,
                        current: None,
                        should_be: Some("skills-aggregated/ with hub/sub_hub folders".to_string()),
                        why: "V11 requires scanning generated sub-hub routing tables".to_string(),
                    }],
                    fix: "Run 'aggregate' before doctor V11 validation".to_string(),
                }
            }
        };

        let mut missing = Vec::new();
        let mut _only = 0usize;

        let hub_dirs = match fs::read_dir(root) {
            Ok(d) => d,
            Err(e) => {
                return DiagnosticStatus::Fail {
                    issues: vec![DiagnosticIssue {
                        description: format!("Failed to read {}: {}", root.display(), e),
                        location: Some(root.display().to_string()),
                        current: None,
                        should_be: None,
                        why: "Cannot validate routing coverage without scanning sub-hub folders"
                            .to_string(),
                    }],
                    fix: "Check directory permissions and rerun aggregate".to_string(),
                }
            }
        };

        for hub_entry in hub_dirs.flatten() {
            let hub_path = hub_entry.path();
            if !hub_path.is_dir() {
                continue;
            }

            let sub_dirs = match fs::read_dir(&hub_path) {
                Ok(s) => s,
                Err(_) => continue,
            };

            for sub_entry in sub_dirs.flatten() {
                let sub_path = sub_entry.path();
                if !sub_path.is_dir() {
                    continue;
                }

                let has_routing = sub_path.join("routing.csv").exists();
                if has_routing {
                    continue;
                }

                if sub_path.join("SKILL.md").exists() {
                    _only += 1;
                    continue;
                }

                let rel = sub_path
                    .strip_prefix(root)
                    .unwrap_or(&sub_path)
                    .to_string_lossy()
                    .replace('\\', "/");
                missing.push(rel);
            }
        }

        if missing.is_empty() {
            DiagnosticStatus::Pass
        } else {
            let preview = missing.iter().take(8).cloned().collect::<Vec<_>>().join(", ");
            DiagnosticStatus::Fail {
                issues: vec![DiagnosticIssue {
                    description: format!(
                        "{} sub-hub folders missing routing.csv ({} -only ignored)",
                        missing.len(),
                        _only
                    ),
                    location: Some(root.display().to_string()),
                    current: Some(preview),
                    should_be: Some("Each sub-hub should include routing.csv".to_string()),
                    why: "Agents route skills through routing.csv per sub-hub".to_string(),
                }],
                fix: "Regenerate aggregated outputs and ensure routing.csv is emitted".to_string(),
            }
        }
    }
}

// 7. V12 src_path resolution check
struct V12RoutingSrcPathCheck;
impl Check for V12RoutingSrcPathCheck {
    fn name(&self) -> &str {
        "V12 routing.csv src_path Resolution"
    }

    fn run(&self) -> DiagnosticStatus {
        let root_candidates = [
            Path::new("skills-aggregated"),
            Path::new("skill-manage/skills-aggregated"),
        ];

        let root = match root_candidates.iter().find(|p| p.exists()) {
            Some(p) => *p,
            None => {
                return DiagnosticStatus::Warn {
                    issues: vec![DiagnosticIssue {
                        description: "skills-aggregated not found".to_string(),
                        location: None,
                        current: None,
                        should_be: Some("skills-aggregated/ with routing.csv files".to_string()),
                        why: "V12 validates src_path links produced by aggregation".to_string(),
                    }],
                    fix: "Run 'aggregate' before doctor V12 validation".to_string(),
                }
            }
        };

        #[derive(serde::Deserialize)]
        struct RoutingPathRow {
            skill_id: String,
            #[serde(default)]
            src_path: String,
        }

        let mut broken = Vec::new();
        let mut total_paths = 0usize;

        let routing_files = walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() && e.file_name() == "routing.csv")
            .map(|e| e.path().to_path_buf())
            .collect::<Vec<_>>();

        for file in routing_files {
            let mut rdr = match csv::Reader::from_path(&file) {
                Ok(r) => r,
                Err(e) => {
                    broken.push(format!("{}: unreadable csv ({})", file.display(), e));
                    continue;
                }
            };

            for row in rdr.deserialize::<RoutingPathRow>() {
                let row = match row {
                    Ok(v) => v,
                    Err(e) => {
                        broken.push(format!("{}: parse error ({})", file.display(), e));
                        continue;
                    }
                };

                if row.src_path.trim().is_empty() {
                    continue;
                }

                total_paths += 1;
                let path = Path::new(row.src_path.trim());
                let resolved = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    Path::new(".").join(path)
                };

                if !resolved.exists() {
                    broken.push(format!("{} -> {}", row.skill_id, row.src_path));
                }
            }
        }

        if broken.is_empty() {
            DiagnosticStatus::Pass
        } else {
            let preview = broken.iter().take(8).cloned().collect::<Vec<_>>().join(", ");
            DiagnosticStatus::Fail {
                issues: vec![DiagnosticIssue {
                    description: format!("{} broken src_path entries", broken.len()),
                    location: Some(root.display().to_string()),
                    current: Some(preview),
                    should_be: Some(format!("All {} src_path entries resolve to files", total_paths)),
                    why: "Broken src_path entries cause silent agent routing failures".to_string(),
                }],
                fix: "Rebuild routing.csv and verify src_path rewrite rules".to_string(),
            }
        }
    }
}

pub struct Diagnostics;

impl Diagnostics {
    pub fn new() -> Self {
        Self
    }

    pub fn run_all(&self) -> Result<CommandResult, SkillManageError> {
        let checks: Vec<Box<dyn Check>> = vec![
            Box::new(ManifestExistsCheck),
            Box::new(SourceDirCheck),
            Box::new(CsvSchemaCheck),
            Box::new(MasterRouterCheck),
            Box::new(V11SubHubRoutingCheck),
            Box::new(V12RoutingSrcPathCheck),
            Box::new(RepoIntegrityCheck),
        ];

        let results: Vec<(String, DiagnosticStatus)> = checks
            .par_iter()
            .map(|check| (check.name().to_string(), check.run()))
            .collect();

        let mut critical_count = 0;
        let mut warning_count = 0;
        for (_, status) in &results {
            match status {
                DiagnosticStatus::Fail { .. } => critical_count += 1,
                DiagnosticStatus::Warn { .. } => warning_count += 1,
                _ => {}
            }
        }
        let total_checks = results.len();
        let passed_checks = total_checks - critical_count - warning_count;
        let health_score = (passed_checks as f32 / total_checks as f32 * 100.0) as u32;

        self.print_report(
            &results,
            critical_count as u32,
            warning_count as u32,
            health_score,
        )?;

        Ok(CommandResult::Doctor {
            checks: results,
            health_score,
        })
    }

    fn print_report(
        &self,
        results: &[(String, DiagnosticStatus)],
        critical_count: u32,
        warning_count: u32,
        health_score: u32,
    ) -> Result<(), SkillManageError> {
        let mut env = Environment::new();

        let template = r#"
🔍 {{ name }} Audit

{% for check_name, status in results %}
---
### {{ check_name }}
**Status:** {% if status.type == 'PASS' %}✅ PASS{% elif status.type == 'WARN' %}⚠️ WARNING{% else %}❌ CRITICAL{% endif %}

{% if status.type != 'PASS' %}
**Issues Found:**
{% for issue in status.issues %}
{{ loop.index }}. {{ issue.description }}
   - Location: {{ issue.location | default('N/A', true) }}
   {% if issue.current -%}- Current: {{ issue.current }}{% endif %}
   {% if issue.should_be -%}- Should be: {{ issue.should_be }}{% endif %}
   - Why: {{ issue.why }}
{% endfor %}

**Recommendation:**
{{ status.fix }}
{% endif %}
{% endfor %}

---
## Executive Summary
**Overall Status:** {{ summary.status }}
**Critical Issues:** {{ summary.critical_count }}
**Warnings:** {{ summary.warning_count }}
**Health Score:** {{ summary.health_score }}%

{{ summary.message }}
"#;

        env.add_template("report", template)
            .map_err(|e| SkillManageError::ConfigError(e.to_string()))?;

        let overall_status = if critical_count > 0 {
            "❌ CRITICAL ISSUES"
        } else if warning_count > 0 {
            "⚠️ NEEDS WORK"
        } else {
            "✅ PASS"
        };

        let message = if critical_count > 0 {
            "System is currently non-compliant. Please fix critical issues before proceeding."
        } else if warning_count > 0 {
            "System is functional but has warnings. Recommended to address them for full compliance."
        } else {
            "All systems normal. Skills bank is in optimal health."
        };

        let ctx = context!(
            name => "skill-manage",
            results => results,
            summary => context!(
                status => overall_status,
                critical_count => critical_count,
                warning_count => warning_count,
                health_score => health_score,
                message => message
            )
        );

        let rendered = env
            .get_template("report")
            .unwrap()
            .render(ctx)
            .map_err(|e| SkillManageError::ConfigError(e.to_string()))?;

        // Print with colors to stderr
        use std::io::Write;
        for line in rendered.lines() {
            if line.contains("✅ PASS") {
                let _ = writeln!(std::io::stderr(), "{}", line.with(Color::Green));
            } else if line.contains("⚠️ WARNING") {
                let _ = writeln!(std::io::stderr(), "{}", line.with(Color::Yellow));
            } else if line.contains("❌ CRITICAL") {
                let _ = writeln!(std::io::stderr(), "{}", line.with(Color::Red));
            } else if line.starts_with("### ") || line.starts_with("## ") {
                let _ = writeln!(std::io::stderr(), "{}", line.bold().underlined());
            } else if line.starts_with("🔍 ") {
                let _ = writeln!(std::io::stderr(), "{}", line.bold().cyan());
            } else {
                let _ = writeln!(std::io::stderr(), "{}", line);
            }
        }

        Ok(())
    }
}
