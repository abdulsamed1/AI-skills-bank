# BMAD Hybrid Validation Report (20 Scenarios)

Generated: 2026-03-27 19:48:26+02:00
Summary: 20/20 passed

| ID | Scenario | Result | Details |
|---|---|---|---|
| S01 | Manifest file exists | PASS | C:\Users\ASUS\production\.\skill manage\hub-manifests.csv |
| S02 | Manifest has at least 1 row | PASS | rows=1632 |
| S03 | Manifest has at least 1000 rows | PASS | rows=1632 |
| S04 | Unique (hub,sub_hub,skill_id) triplets | PASS | total=1632, unique=1632 |
| S05 | No empty skill_id values | PASS | missing=0 |
| S06 | match_score is integer between 1 and 100 | PASS | invalid=0 |
| S07 | phase is integer between 1 and 4 | PASS | invalid=0 |
| S08 | required values are true/false only | PASS | invalid=0 |
| S09 | Main hubs count is at least 11 | PASS | hubs=12 |
| S10 | Sub-hub groups count is 27 | PASS | subhubs=27 |
| S11 | Every manifest sub-hub has SKILL.md | PASS | missing=0 |
| S12 | All generated SKILL files contain frontmatter | PASS | missing=0 |
| S13 | Frontmatter name matches folder name | PASS | mismatch=0 |
| S14 | Contains Critical Instructions section | PASS | missing=0 |
| S15 | Contains Quick Intent Matcher section | PASS | missing=0 |
| S16 | Quick Intent table row count is valid for each sub-hub | PASS | invalid_files=0 |
| S17 | Contains Selection Rules section | PASS | missing=0 |
| S18 | Contains Dependency Rules section | PASS | missing=0 |
| S19 | Contains anti-hallucination rule text | PASS | missing=0 |
| S20 | Contains score threshold rule (>=10) | PASS | missing=0 |
