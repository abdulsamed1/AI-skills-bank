# Extending Classification Domains & Rules

This guide explains how to add new classification keywords, modify sub-hub logic, and configure new routing directories in **skills-bank**.

---

## 🛠️ How to Add Keywords to an Existing Sub-Hub

All keyword classifications are configured statically in `src/components/aggregator/rules.rs`.

### Step 1: Locate the Hub and Sub-Hub Definition
Open `src/components/aggregator/rules.rs` and find `SUB_HUB_DEFINITIONS`.
Identify the insertion point under the respective Hub (e.g. `code-quality`, `frontend`, `server-side`, or `business`).

### Step 2: Update the `SubHubRule` Struct
Each sub-hub configuration contains three vectors:
- **`keywords`**: Terms that add points to the candidate score.
- **`anchor_keywords`**: High-weight terms that indicate strong alignment with the sub-hub.
- **`negative_keywords`**: Terms that block/invalidate matches (to avoid false positives).

Example: Adding a new database tool keyword (e.g. `edgedb`):
```rust
be_sub.insert(
    "databases",
    SubHubRule {
        keywords: vec![
            "sql", "postgres", "mysql", "sqlite", "edgedb", // <-- Added here
            // ...
        ],
        anchor_keywords: vec![
            "sql", "postgres", "database",
        ],
        negative_keywords: vec![
            "frontend", "ui", "css",
        ],
    },
);
```

---

## ⚙️ Adding a New Sub-Hub

To add a completely new sub-hub (for example, `data-science` under the `server-side` hub):

### Step 1: Declare the Sub-Hub Keywords
In `src/components/aggregator/rules.rs` under `SUB_HUB_DEFINITIONS`:
```rust
be_sub.insert(
    "data-science",
    SubHubRule {
        keywords: vec!["pandas", "numpy", "pytorch", "tensorflow", "jupyter", "scikit-learn"],
        anchor_keywords: vec!["data-science", "machine-learning", "pytorch"],
        negative_keywords: vec!["frontend", "ui", "seo"],
    },
);
```

### Step 2: Add Aliases (Optional)
If downstream tools might use variant tags for this sub-hub, map them in `CANONICAL_SUBHUB_ALIASES` in `rules.rs`:
```rust
static CANONICAL_SUBHUB_ALIASES: &[(&str, &str, &str)] = &[
    // ...
    ("ml", "server-side", "data-science"),
    ("deep-learning", "server-side", "data-science"),
];
```

### Step 3: Run Compilation and Tests
Verify the code compiles and format is correct:
```bash
cargo fmt
cargo test
cargo build --release
```

---

## 🏗️ Adding a New Hub

Adding a new primary hub requires updating the master registry list:

1. Add the hub name to the static slice `VALID_HUBS` in `src/components/aggregator/rules.rs`:
   ```rust
   pub static VALID_HUBS: &[&str] = &[
       "business",
       "code-quality",
       "frontend",
       "server-side",
       "new-hub-name", // <-- Added here
   ];
   ```
2. Insert the hub definition mapping into `SUB_HUB_DEFINITIONS` in `rules.rs`.
3. Update default sub-hub fallbacks in `default_subhub_for_hub()` to ensure a safe default exists.
4. Rebuild using `cargo build --release`.
