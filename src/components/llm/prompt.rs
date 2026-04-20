use crate::components::llm::types::LlmClassificationContext;

pub fn build_classification_prompt(context: &LlmClassificationContext, is_batch: bool) -> String {
    let mut prompt = format!(
        r#"You are a precise skill classification engine for a developer tools platform.

VALID HUBS AND THEIR PURPOSE:
- code-quality: Programming languages (rust, python, typescript, golang, java, javascript), testing, security, CI/CD, code review
- server-side: Backend frameworks (express, fastapi, django), API design, databases, caching (redis), messaging (kafka), containers (docker/k8s), serverless (cloudflare workers), observability
- frontend: Web frameworks (react, vue, nextjs), UI/UX, state management
- ai: Prompt engineering, LLM tools, AI agent workflows
- business: Product management, strategy, marketing, sales, analytics, operations
- mobile: iOS (swift), Android (kotlin), cross-platform (flutter, react-native)

DISAMBIGUATION RULES (apply strictly):
1. If the skill mentions "cloudflare workers" or "edge functions" -> server-side/serverless-edge (NOT security)
2. If the skill is a programming language guide -> code-quality/{{language}} (NOT server-side)
3. If the skill mentions "go-to-market" or "gtm" -> business/strategy (NOT code-quality/golang)
4. If the skill is about "logging" or "monitoring" -> server-side/observability (NOT code-quality)
5. If the skill content is primarily instructional workflow, not technical -> business/operations

CRITICAL - "design" DISAMBIGUATION (the word "design" does NOT always mean frontend/ui-ux):
6. If the skill is about AI agents, multi-agent systems, agent tools, LLM tools, function calling, MCP, or CrewAI -> ai/prompting-factory (NOT frontend)
7. If the skill is about software architecture, system design, DDD, bounded contexts, or architectural decisions -> server-side/architect (NOT frontend)
8. If the skill is about database design, NoSQL, DynamoDB, schema modeling, or data architecture -> server-side/databases (NOT frontend)
9. If the skill is about identity governance, incident response, access provisioning, privileged access, cybersecurity, or IAM -> code-quality/security (NOT frontend)
10. If the skill is about team composition, hiring, equity allocation, or organizational design -> business/operations (NOT frontend)
11. If the skill is about design space exploration (DSE), EDA, hardware parameter tuning -> server-side/architect (NOT frontend)
12. If the skill is about ML pipelines, machine learning ops, or ML model training -> ai/prompting-factory (NOT frontend)
13. If the skill is about workflow orchestration (Temporal, Saga, etc.) -> server-side/frameworks (NOT frontend)
14. Only classify as frontend/ui-ux if the skill is genuinely about visual UI, CSS, HTML, Tailwind, Figma, component libraries, or user interface rendering

Valid Hubs: {}
Valid Sub-Hubs: {}
Excluded Categories: {}

Return ONLY valid JSON. No explanation, no markdown.
"#,
        context.valid_hubs.join(", "),
        context.valid_sub_hubs.join(", "),
        context.excluded_categories.join(", ")
    );

    if is_batch {
        prompt.push_str(r#"
CRITICAL: You are in BATCH MODE. You must return a JSON ARRAY of objects.
Each object in the array MUST have a "ranked_suggestions" key containing a list with exactly one classification.
The length of the output array MUST match the number of input skills.

EXAMPLE OUTPUT FORMAT:
[
  {
    "ranked_suggestions": [
      { "hub": "code-quality", "sub_hub": "rust", "confidence": 100, "reasoning": "Explicit Rust project" }
    ]
  },
  {
    "ranked_suggestions": [
      { "hub": "ai", "sub_hub": "prompt-engineering", "confidence": 90, "reasoning": "Focuses on LLM prompting" }
    ]
  }
]

Return ONLY the JSON array.
"#);
    } else {
        prompt.push_str(r#"Return: {"ranked_suggestions":[{"hub":"...","sub_hub":"...","confidence":0-100,"reasoning":"one sentence"}]}"#);
    }
    prompt
}
