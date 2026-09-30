use skills_bank::components::llm::relevance;
use skills_bank::components::llm::cost;
use skills_bank::components::llm::provider::LlmProvider;
use skills_bank::components::llm::types::LlmClassificationContext;

fn load_dotenv() {
    let env_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
    if let Ok(content) = std::fs::read_to_string(&env_path) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim();
                let val = val.trim().trim_matches('"').trim_matches('\'');
                if !key.is_empty() && std::env::var(key).is_err() {
                    std::env::set_var(key, val);
                }
            }
        }
    }
}

#[tokio::test]
async fn smoke_e2e_llm_pipeline() {
    load_dotenv();

    println!("=== SMOKE TEST: LLM Pipeline via FreeLLMAPI ===\n");

    cost::record("test", 1000, 500);
    let summary = cost::summary();
    println!("[COST] {}", summary);
    assert!(summary.contains("1 calls"), "cost summary should show 1 call");

    let cands = vec!["code-quality".to_string(), "frontend".to_string()];
    let val = serde_json::from_str(r#"{"scores":[{"hub":"code-quality","probability":0.92},{"hub":"frontend","probability":0.08}]}"#).unwrap();
    let parsed = relevance::strict_parse_panel(&val, &cands).expect("panel should parse");
    assert_eq!(parsed.len(), 2);
    let winner = relevance::decide_winner(&parsed, 0.5).expect("should have winner");
    assert_eq!(winner.0, "code-quality");
    println!("[PANEL] Parsed {} candidates, winner: {} (p={:.2})", parsed.len(), winner.0, winner.1);

    let dval = serde_json::from_str(r#"{"verdicts":[{"duplicate":true,"probability":0.95}]}"#).unwrap();
    let verdicts = relevance::strict_parse_dedup(&dval, 1).expect("dedup should parse");
    assert_eq!(verdicts[0], (true, 0.95));
    println!("[DEDUP] Parsed {} verdict: duplicate={} p={:.2}", verdicts.len(), verdicts[0].0, verdicts[0].1);

    let key_v1 = skills_bank::components::llm::key_for_skill_versioned(
        std::path::Path::new("."), std::path::Path::new("test/SKILL.md"),
        "test", "desc", None, Some("relevance-panel-1")
    );
    let key_v2 = skills_bank::components::llm::key_for_skill_versioned(
        std::path::Path::new("."), std::path::Path::new("test/SKILL.md"),
        "test", "desc", None, Some("relevance-panel-2")
    );
    assert_ne!(key_v1, key_v2, "versioned keys should differ");
    println!("[CACHE] Versioned keys differ: OK");

    println!("\n[LLM] Testing live classification via FreeLLMAPI...");
    let config = skills_bank::components::llm::LlmClientConfig::from_env()
        .expect("LLM config from env");
    println!("[LLM] Provider: {}, URL: {:?}", config.provider, config.api_url);
    let provider = skills_bank::components::llm::CustomProvider::new(config)
        .expect("provider init");

    let ctx = LlmClassificationContext {
        valid_hubs: vec!["code-quality".to_string(), "frontend".to_string(), "server-side".to_string(), "business".to_string()],
        valid_sub_hubs: vec!["security".to_string(), "testing-qa".to_string(), "ui-ux".to_string(), "architect".to_string()],
        excluded_categories: vec![],
    };

    let result = provider.classify("test-security-skill", "A penetration testing and vulnerability assessment tool", Some("Security testing workflows"), &ctx).await;
    match result {
        Ok(resp) => {
            let top = &resp.ranked_suggestions[0];
            println!("[LLM] classify: hub={} sub_hub={} confidence={}", top.hub, top.sub_hub, top.confidence);
        }
        Err(e) => println!("[LLM] classify error: {:?}", e),
    }

    let items = vec![
        ("skill-a".to_string(), "A React component library for building UIs".to_string(), None),
        ("skill-b".to_string(), "A SQL query optimizer for PostgreSQL".to_string(), None),
    ];
    let hub_cands = vec!["code-quality".to_string(), "frontend".to_string(), "server-side".to_string(), "business".to_string()];
    let panel_result = provider.classify_panel(&items, &hub_cands, &ctx).await;
    match panel_result {
        Ok(panels) => {
            for (i, p) in panels.iter().enumerate() {
                match p {
                    Some(scores) => {
                        let winner = relevance::decide_winner(scores, 0.5);
                        println!("[LLM] panel[{}]: {} scores, winner={:?}", i, scores.len(), winner);
                    }
                    None => println!("[LLM] panel[{}]: abstained", i),
                }
            }
        }
        Err(e) => println!("[LLM] panel error: {:?}", e),
    }

    let pairs = vec![
        ("pen-test-tool".to_string(), "A penetration testing tool for vulnerability assessment".to_string(),
         "vuln-scanner".to_string(), "A vulnerability scanning and penetration testing tool".to_string()),
    ];
    let dedup_result = provider.classify_duplicates(&pairs).await;
    match dedup_result {
        Ok(verdicts) => {
            for (i, v) in verdicts.iter().enumerate() {
                match v {
                    Some((dup, p)) => println!("[LLM] dedup[{}]: duplicate={} p={:.2}", i, dup, p),
                    None => println!("[LLM] dedup[{}]: abstained", i),
                }
            }
        }
        Err(e) => println!("[LLM] dedup error: {:?}", e),
    }

    println!("\n[COST] Final: {}", cost::summary());
    println!("\n=== SMOKE TEST COMPLETE ===");
}
