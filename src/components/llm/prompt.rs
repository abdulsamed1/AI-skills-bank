use crate::components::llm::types::LlmClassificationContext;

pub fn build_classification_prompt(context: &LlmClassificationContext, is_batch: bool) -> String {
    let mut prompt = format!(
        r#"You are a strict, precise routing engine for a developer skills database.
Analyze each skill based on its description and assign it to exactly ONE hub and sub-hub.

Valid Hubs: {}
Valid Sub-Hubs: {}
Excluded Categories: {}

ARCHITECTURE RULES (STRICT):
1. UI/UX design (Tailwind, CSS, Figma) -> frontend/ui-ux
2. skills writing, prompt engineering, MCP, LLMs -> server-side/prompting-factory
3. API design, system architecture, DDD -> server-side/architect
4. Databases, SQL, NoSQL -> server-side/databases
5. Security, IAM, cybersecurity -> code-quality/security
6. If the skill uses excluded categories, classify as hub: "excluded", sub_hub: "excluded" (MUST be string "excluded", NOT null).
"#,
        context.valid_hubs.join(","),
        context.valid_sub_hubs.join(","),
        context.excluded_categories.join(",")
    );

    if is_batch {
        prompt.push_str(r#"
CRITICAL: You MUST return ONLY a raw JSON OBJECT with a "results" array. 
EACH item in "results" MUST be a flat object containing "hub", "sub_hub", "confidence", and "reasoning".
Do NOT nest "ranked_suggestions" inside the items. Do NOT use markdown code blocks.

{
  "results": [
    {"hub":"frontend", "sub_hub":"ui-ux", "confidence":90, "reasoning":"Tailwind CSS styling."}
  ]
}
"#);
    } else {
        prompt.push_str(r#"Return ONLY raw JSON. No markdown blocks. Format: {"hub":"...","sub_hub":"...","confidence":100,"reasoning":"Technical reason"}"#);
    }
    prompt
}
