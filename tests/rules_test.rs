use skill_manage::components::aggregator::rules::*;
use skill_manage::components::aggregator::SkillMetadata;
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
    };

    let kept = apply_rules(&mut meta);
    assert!(kept);
    assert_eq!(meta.hub, "business");
    assert_eq!(meta.sub_hub, "strategy");
}

#[test]
fn test_security_strategy_does_not_fall_into_business_product_strategy() {
    let mut meta = SkillMetadata {
        name: "implementing-envelope-encryption-with-aws-kms".to_string(),
        description: "Envelope encryption strategy where data encryption keys are managed via AWS KMS for secure systems.".to_string(),
        path: PathBuf::from("lib/mukul975-Anthropic-Cybersecurity-Skills/skills/implementing-envelope-encryption-with-aws-kms/SKILL.md"),
        hub: String::new(),
        sub_hub: String::new(),
        triggers: None,
        match_score: None,
        phase: None,
        required: None,
        action: None,
    };

    let kept = apply_rules(&mut meta);
    assert!(kept);
    assert!(!(meta.hub == "business" && meta.sub_hub == "strategy") || meta.sub_hub != "strategy",
        "Security skill should NOT route to business/strategy, got hub={} sub_hub={}", meta.hub, meta.sub_hub);
}

#[test]
fn test_antigravity_skill_pack_routes_to_ai_skills_factory() {
    let mut meta = SkillMetadata {
        name: "skill-installer".to_string(),
        description: "Install and validate new skills in the ecosystem.".to_string(),
        path: PathBuf::from("lib/antigravity-awesome-skills/skills/skill-installer/SKILL.md"),
        hub: String::new(),
        sub_hub: String::new(),
        triggers: None,
        match_score: None,
        phase: None,
        required: None,
        action: None,
    };

    let kept = apply_rules(&mut meta);
    assert!(kept);
    assert_eq!(meta.hub, "ai");
    assert_eq!(meta.sub_hub, "skills-factory");
}
