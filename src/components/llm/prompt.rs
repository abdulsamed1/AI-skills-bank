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
2. AI agents, prompt engineering, MCP, LLMs -> ai/prompting-factory
3. API design, system architecture, DDD -> server-side/architect
4. Databases, SQL, NoSQL -> server-side/databases
5. Security, IAM, cybersecurity -> code-quality/security
6. Programming language guides -> code-quality/{{language}}
7. Mobile/iOS/Android -> mobile/{{platform}}
8. If the skill uses excluded categories, classify as hub: "excluded", sub_hub: "excluded" (MUST be string "excluded", NOT null).
"#,
        context.valid_hubs.join(","),
        context.valid_sub_hubs.join(","),
        context.excluded_categories.join(",")
    );

    if is_batch {
        prompt.push_str(r#"
CRITICAL: You MUST return ONLY a raw JSON OBJECT. Do NOT use markdown code blocks (```json). Do NOT add text before or after.
The array length MUST match the number of input skills EXACTLY.

{
  "results": [
    {"ranked_suggestions":[{"hub":"code-quality","sub_hub":"rust","confidence":100,"reasoning":"Memory-safe systems programming."}]},
    {"ranked_suggestions":[{"hub":"frontend","sub_hub":"ui-ux","confidence":90,"reasoning":"Tailwind CSS styling."}]}
  ]
}
"#);
    } else {
        prompt.push_str(r#"Return ONLY raw JSON. No markdown blocks. Format: {"ranked_suggestions":[{"hub":"...","sub_hub":"...","confidence":100,"reasoning":"Technical reason"}]}"#);
    }
    prompt
}
