use skills_bank::components::aggregator::rules::*;
use skills_bank::components::aggregator::SkillMetadata;
use std::path::PathBuf;

#[test]
fn test_tokenization() {
    let (_, tokens) = normalize_text("React-Next.js best practices!");
    assert!(tokens.contains("react"));
    assert!(tokens.contains("next"));
    assert!(tokens.contains("js"));
}

#[test]
fn test_exclusion() {
    let (norm, tokens) = normalize_text("My medical records");
    assert!(is_excluded(&norm, &tokens));
}

#[test]
fn test_trigger_gen() {
    let triggers = generate_triggers("my-awesome-skill");
    assert_eq!(triggers, "my;awesome;skill");
}

#[test]
fn test_keyword_matching_avoids_substring_false_positive() {
    let (norm, tokens) = normalize_text("zero trust architecture");
    assert!(!keyword_matches(&norm, &tokens, "rust"));
}

#[test]
fn test_keyword_matching_is_case_insensitive() {
    let (norm, tokens) = normalize_text("Create a gtm strategy for launch");
    assert!(keyword_matches(&norm, &tokens, "gTM strategy"));
}

#[test]
fn test_html_injection_routes_to_testing_security() {
    let mut meta = SkillMetadata {
        name: "html-injection-testing".to_string(),
        description: "Identify and exploit HTML injection vulnerabilities in web applications and test input sanitization.".to_string(),
        path: PathBuf::from("lib/antigravity-awesome-skills/skills/html-injection-testing/SKILL.md"),
        hub: String::new(),
        sub_hub: String::new(),
        triggers: None,
        match_score: None,
        phase: None,
        required: None,
        action: None,
        content_body: None,
    };

    let kept = apply_rules(&mut meta);
    assert!(kept);
    assert_eq!(meta.hub, "code-quality");
    assert_eq!(meta.sub_hub, "security");
}

#[test]
fn test_omero_not_forced_to_marketing_content() {
    let mut meta = SkillMetadata {
        name: "omero-integration".to_string(),
        description: "Microscopy platform. Access images via Python and analyze high-content screening datasets.".to_string(),
        path: PathBuf::from("lib/K-Dense-AI-claude-scientific-skills/scientific-skills/omero-integration/SKILL.md"),
        hub: String::new(),
        sub_hub: String::new(),
        triggers: None,
        match_score: None,
        phase: None,
        required: None,
        action: None,
        content_body: None,
    };

    let kept = apply_rules(&mut meta);
    assert!(kept);
    assert!(!(meta.hub == "marketing" && meta.sub_hub == "content"));
}

#[test]
fn test_product_strategy_skill_stays_in_business() {
    let mut meta = SkillMetadata {
        name: "product-strategy-session".to_string(),
        description: "Run an end-to-end product strategy session across positioning, discovery, and roadmap planning.".to_string(),
        path: PathBuf::from("lib/Product-Manager-Skills/skills/product-strategy-session/SKILL.md"),
        hub: String::new(),
        sub_hub: String::new(),
        triggers: None,
        match_score: None,
        phase: None,
        required: None,
        action: None,
        content_body: None,
    };

    let kept = apply_rules(&mut meta);
    assert!(kept);
    assert_eq!(meta.hub, "business",
        "product-strategy should route to business hub, got hub={}", meta.hub);
}

#[test]
fn test_agent_tool_builder_does_not_route_to_frontend() {
    let mut meta = SkillMetadata {
        name: "agent-tool-builder".to_string(),
        description: "You are an expert in the interface between LLMs and the outside world. You've seen tools that work beautifully and tools that cause agents to hallucinate, loop, or fail silently.".to_string(),
        path: PathBuf::from("lib/antigravity-awesome-skills/skills/agent-tool-builder/SKILL.md"),
        hub: String::new(),
        sub_hub: String::new(),
        triggers: None,
        match_score: None,
        phase: None,
        required: None,
        action: None,
        content_body: None,
    };

    let kept = apply_rules(&mut meta);
    assert!(kept);
    assert_ne!(meta.hub, "frontend",
        "Agent tool builder should NOT route to frontend, got hub={} sub_hub={}", meta.hub, meta.sub_hub);
}

#[test]
fn test_identity_governance_does_not_route_to_frontend() {
    let mut meta = SkillMetadata {
        name: "building-identity-governance-lifecycle-process".to_string(),
        description: "Builds comprehensive identity governance and lifecycle management processes including joiner-mover-leaver automation, role mining, access request workflows.".to_string(),
        path: PathBuf::from("lib/mukul975-anthropic-cybersecurity-skills/skills/building-identity-governance-lifecycle-process/SKILL.md"),
        hub: String::new(),
        sub_hub: String::new(),
        triggers: None,
        match_score: None,
        phase: None,
        required: None,
        action: None,
        content_body: None,
    };

    let kept = apply_rules(&mut meta);
    assert!(kept);
    assert_ne!(meta.hub, "frontend",
        "Identity governance should NOT route to frontend, got hub={} sub_hub={}", meta.hub, meta.sub_hub);
}

#[test]
fn test_software_architecture_does_not_route_to_frontend() {
    let mut meta = SkillMetadata {
        name: "software-architecture".to_string(),
        description: "Guide for quality focused software architecture. This skill should be used when users want to write code, design architecture, analyze code.".to_string(),
        path: PathBuf::from("lib/antigravity-awesome-skills/skills/software-architecture/SKILL.md"),
        hub: String::new(),
        sub_hub: String::new(),
        triggers: None,
        match_score: None,
        phase: None,
        required: None,
        action: None,
        content_body: None,
    };

    let kept = apply_rules(&mut meta);
    assert!(kept);
    assert_ne!(meta.hub, "frontend",
        "Software architecture should NOT route to frontend, got hub={} sub_hub={}", meta.hub, meta.sub_hub);
}

#[test]
fn test_cybersecurity_repo_routes_to_security_not_ai() {
    // Root cause: "anthropic" in repo name matched before "cybersecurity"
    let mut meta = SkillMetadata {
        name: "exploiting-sql-injection-vulnerabilities".to_string(),
        description: "Identifies and exploits SQL injection vulnerabilities in web applications during authorized penetration tests using sqlmap.".to_string(),
        path: PathBuf::from("lib/mukul975-anthropic-cybersecurity-skills/skills/exploiting-sql-injection-vulnerabilities/SKILL.md"),
        hub: String::new(), sub_hub: String::new(),
        triggers: None, match_score: None, phase: None,
        required: None, action: None, content_body: None,
    };
    apply_rules(&mut meta);
    assert_eq!(meta.hub, "code-quality");
    assert_eq!(meta.sub_hub, "security");
}

#[test]
fn test_detecting_skill_routes_to_security() {
    let mut meta = SkillMetadata {
        name: "detecting-lateral-movement-in-network".to_string(),
        description: "Identifies lateral movement techniques in enterprise networks by analyzing authentication logs.".to_string(),
        path: PathBuf::from("lib/mukul975-anthropic-cybersecurity-skills/skills/detecting-lateral-movement-in-network/SKILL.md"),
        hub: String::new(), sub_hub: String::new(),
        triggers: None, match_score: None, phase: None,
        required: None, action: None, content_body: None,
    };
    apply_rules(&mut meta);
    assert_eq!(meta.hub, "code-quality");
    assert_eq!(meta.sub_hub, "security");
}

#[test]
fn test_azure_keyvault_routes_to_security_not_version_control() {
    let mut meta = SkillMetadata {
        name: "azure-security-keyvault-keys-dotnet".to_string(),
        description: "Azure Key Vault Keys SDK for cryptographic key management.".to_string(),
        path: PathBuf::from("lib/microsoft-skills/.github/plugins/azure-sdk-dotnet/skills/azure-security-keyvault-keys-dotnet/SKILL.md"),
        hub: String::new(), sub_hub: String::new(),
        triggers: None, match_score: None, phase: None,
        required: None, action: None, content_body: None,
    };
    apply_rules(&mut meta);
    assert_eq!(meta.hub, "code-quality");
    assert_eq!(meta.sub_hub, "security");
}

#[test]
fn test_azure_cosmos_routes_to_databases() {
    let mut meta = SkillMetadata {
        name: "azure-cosmos-db-python".to_string(),
        description: "Azure Cosmos DB client library for Python.".to_string(),
        path: PathBuf::from("lib/microsoft-skills/.github/plugins/azure-sdk-python/skills/azure-cosmos-db-python/SKILL.md"),
        hub: String::new(), sub_hub: String::new(),
        triggers: None, match_score: None, phase: None,
        required: None, action: None, content_body: None,
    };
    apply_rules(&mut meta);
    assert_eq!(meta.hub, "server-side");
    assert_eq!(meta.sub_hub, "databases");
}

#[test]
fn test_azure_ai_routes_to_agents() {
    let mut meta = SkillMetadata {
        name: "azure-ai-translation-ts".to_string(),
        description: "Azure Cognitive Services Translator SDK for JavaScript.".to_string(),
        path: PathBuf::from("lib/microsoft-skills/.github/plugins/azure-sdk-typescript/skills/azure-ai-translation-ts/SKILL.md"),
        hub: String::new(), sub_hub: String::new(),
        triggers: None, match_score: None, phase: None,
        required: None, action: None, content_body: None,
    };
    apply_rules(&mut meta);
    assert_eq!(meta.hub, "server-side");
    assert_eq!(meta.sub_hub, "architect");
}

#[test]
fn test_azure_generic_sdk_routes_to_serverless_edge() {
    let mut meta = SkillMetadata {
        name: "azure-communication-services-java".to_string(),
        description: "Azure Communication Services SDK for Java.".to_string(),
        path: PathBuf::from("lib/microsoft-skills/.github/plugins/azure-sdk-java/skills/azure-communication-services-java/SKILL.md"),
        hub: String::new(), sub_hub: String::new(),
        triggers: None, match_score: None, phase: None,
        required: None, action: None, content_body: None,
    };
    apply_rules(&mut meta);
    assert_eq!(meta.hub, "server-side");
    assert_eq!(meta.sub_hub, "serverless-edge");
}

#[test]
fn test_curated_openai_skill_does_not_route_to_ai_agents() {
    let mut meta = SkillMetadata {
        name: "aspnet-core".to_string(),
        description: "Build, review, refactor, or architect ASP.NET Core web applications using current official guidance for .NET web development.".to_string(),
        path: PathBuf::from("lib/openai/skills/.curated/aspnet-core/SKILL.md"),
        hub: String::new(), sub_hub: String::new(),
        triggers: None, match_score: None, phase: None,
        required: None, action: None, content_body: None,
    };
    apply_rules(&mut meta);
    // Since aspnet-core is in openai curated repo but is C#, it should fall back to business/business-strategy, NOT server-side/agents
    assert_eq!(meta.hub, "business");
    assert_eq!(meta.sub_hub, "business-strategy");
}

#[test]
fn test_ads_amazon_routes_to_marketing() {
    let mut meta = SkillMetadata {
        name: "ads-amazon".to_string(),
        description: "Amazon Ads deep analysis covering Sponsored Products, Sponsored Brands, Sponsored Display, and basic Amazon DSP.".to_string(),
        path: PathBuf::from("lib/claude-ads.git/skills/ads-amazon/SKILL.md"),
        hub: String::new(), sub_hub: String::new(),
        triggers: None, match_score: None, phase: None,
        required: None, action: None, content_body: None,
    };
    apply_rules(&mut meta);
    assert_eq!(meta.hub, "business");
    assert_eq!(meta.sub_hub, "marketing");
}

#[test]
fn test_creating_skills_routes_to_prompting_factory() {
    let mut meta = SkillMetadata {
        name: "creating-skills".to_string(),
        description: "Guide for creating Claude Code skills following Anthropic's official best practices. Use when user wants to create a new skill, build a skill, write SKILL.md, update an existing skill, or needs skill creation guidelines. Provides structure, frontmatter fields, naming conventions, and new features like dynamic context injection and subagent execution.".to_string(),
        path: PathBuf::from("lib/fvadicamo-dev-agent-skills/plugins/skill-authoring/skills/creating-skills/SKILL.md"),
        hub: String::new(), sub_hub: String::new(),
        triggers: None, match_score: None, phase: None,
        required: None, action: None, content_body: None,
    };
    apply_rules(&mut meta);
    assert_eq!(meta.hub, "server-side");
    assert_eq!(meta.sub_hub, "prompting-factory");
}
