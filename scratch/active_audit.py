#!/usr/bin/env python3
import os
import csv
import sys
from pathlib import Path

# ANSI colors for premium look
GREEN = "\033[92m"
RED = "\033[91m"
YELLOW = "\033[93m"
CYAN = "\033[96m"
BOLD = "\033[1m"
RESET = "\033[0m"

VALID_HUBS = {"business", "code-quality", "frontend", "server-side"}
DEPRECATED_HUBS_SUBHUBS = {"mobile", "ios", "android", "cross-platform", "code-review"}

TECHNICAL_KEYWORDS = {
    "swift": ("frontend", "web-frameworks"),
    "kotlin": ("frontend", "web-frameworks"),
    "ios": ("frontend", "web-frameworks"),
    "android": ("frontend", "web-frameworks"),
    "flutter": ("frontend", "web-frameworks"),
    "expo-": ("frontend", "web-frameworks"),
    "react-native": ("frontend", "web-frameworks"),
    "code-review": ("code-quality", "testing-qa"),
    "refactor": ("code-quality", "testing-qa"),
    "clean-code": ("code-quality", "testing-qa"),
}

def main():
    print(f"{BOLD}{CYAN}🚀 Executing Deep Active Audit of Skills Bank Assets...{RESET}\n")

    repo_root = Path(__file__).resolve().parent.parent
    manifest_path = repo_root / "hub-manifests.csv"
    aggregated_dir = repo_root / "skills-aggregated"

    errors = 0
    warnings = 0

    # 1. Verify Manifest CSV File Existence
    if not manifest_path.exists():
        print(f"[{RED}FAIL{RESET}] hub-manifests.csv not found at {manifest_path}")
        sys.exit(1)
    else:
        print(f"[{GREEN}OK{RESET}] CSV Manifest exists.")

    # 2. Read and Analyze CSV Data
    csv_rows = []
    try:
        with open(manifest_path, mode="r", encoding="utf-8") as f:
            reader = csv.reader(f)
            headers = next(reader)
            if headers != ["hub", "sub_hub", "skill_id", "description", "outputs"]:
                print(f"[{RED}FAIL{RESET}] CSV headers are invalid: {headers}")
                errors += 1
            else:
                print(f"[{GREEN}OK{RESET}] CSV Schema headers are correct.")

            for line_no, row in enumerate(reader, start=2):
                if len(row) != 5:
                    print(f"[{RED}FAIL{RESET}] Line {line_no}: Row has incorrect column count ({len(row)})")
                    errors += 1
                    continue
                csv_rows.append((line_no, row))
    except Exception as e:
        print(f"[{RED}FAIL{RESET}] Error reading CSV manifest: {e}")
        sys.exit(1)

    # 3. Perform Decommission and Placement Audits on Rows
    print(f"\n{BOLD}Auditing skill categorizations...{RESET}")
    strategy_technical_count = 0
    displaced_mobile_count = 0
    displaced_review_count = 0

    for line_no, (hub, sub_hub, skill_id, description, outputs) in csv_rows:
        # Check for deprecated hubs or sub-hubs in output
        if hub in DEPRECATED_HUBS_SUBHUBS or sub_hub in DEPRECATED_HUBS_SUBHUBS:
            print(f"[{RED}FAIL{RESET}] Line {line_no}: Skill '{skill_id}' is mapped to decommissioned category ({hub}/{sub_hub})")
            errors += 1

        # Check if any hub is completely invalid
        if hub not in VALID_HUBS:
            print(f"[{RED}FAIL{RESET}] Line {line_no}: Skill '{skill_id}' has unrecognized hub '{hub}'")
            errors += 1

        # Track displacements
        skill_lower = skill_id.lower()
        if any(kw in skill_lower for kw in ["swift", "ios", "android", "flutter", "react-native"]):
            displaced_mobile_count += 1
            if hub != "frontend":
                print(f"[{YELLOW}WARN{RESET}] Mobile skill '{skill_id}' routed to non-frontend hub ({hub}/{sub_hub})")
                warnings += 1

        if any(kw in skill_lower for kw in ["code-review", "clean-code", "refactor"]):
            displaced_review_count += 1
            if hub != "code-quality" or sub_hub != "testing-qa":
                print(f"[{YELLOW}WARN{RESET}] Code-review skill '{skill_id}' routed to ({hub}/{sub_hub}) instead of code-quality/testing-qa")
                warnings += 1

        # Verify no highly technical skill falls back to business-strategy
        if hub == "business" and sub_hub == "business-strategy":
            for kw, target in TECHNICAL_KEYWORDS.items():
                if kw in skill_lower:
                    print(f"[{RED}FAIL{RESET}] Line {line_no}: Technical skill '{skill_id}' fell back to business-strategy (contains '{kw}')")
                    errors += 1
                    strategy_technical_count += 1

    print(f"[{GREEN}INFO{RESET}] Mobile-related skills mapped: {displaced_mobile_count}")
    print(f"[{GREEN}INFO{RESET}] Code-review-related skills mapped: {displaced_review_count}")

    # 4. Physical Aggregation Directories Audit
    print(f"\n{BOLD}Auditing physical directory structures under skills-aggregated/...{RESET}")
    if not aggregated_dir.exists():
        print(f"[{YELLOW}WARN{RESET}] Physical skills-aggregated directory does not exist yet.")
        warnings += 1
    else:
        # Check for obsolete mobile folder
        obsolete_mobile = aggregated_dir / "mobile"
        if obsolete_mobile.exists():
            print(f"[{RED}FAIL{RESET}] Obsolete physical directory '{obsolete_mobile}' still exists on disk!")
            errors += 1
        else:
            print(f"[{GREEN}OK{RESET}] No obsolete 'mobile' directory found on disk.")

        # Check for obsolete code-review folder under code-quality
        obsolete_review = aggregated_dir / "code-quality" / "code-review"
        if obsolete_review.exists():
            print(f"[{RED}FAIL{RESET}] Obsolete physical sub-directory '{obsolete_review}' still exists on disk!")
            errors += 1
        else:
            print(f"[{GREEN}OK{RESET}] No obsolete 'code-quality/code-review' directory found on disk.")

        # Check that actual sub-hub directories on disk match the VALID configurations
        for item in aggregated_dir.iterdir():
            if item.is_dir() and not item.name.startswith("."):
                if item.name not in VALID_HUBS:
                    print(f"[{RED}FAIL{RESET}] Unregistered physical hub directory found on disk: '{item.name}'")
                    errors += 1

    # 5. Audit Results Executive Summary
    print(f"\n{BOLD}{CYAN}=================================================={RESET}")
    print(f"{BOLD}             AUDIT RESULTS SUMMARY                {RESET}")
    print(f"{BOLD}{CYAN}=================================================={RESET}")
    print(f"Total Errors:   {GREEN if errors == 0 else RED}{errors}{RESET}")
    print(f"Total Warnings: {GREEN if warnings == 0 else YELLOW}{warnings}{RESET}")
    
    if errors > 0:
        print(f"\n{RED}❌ ACTIVE AUDIT FAILED.{RESET} There are critical integrity issues that must be addressed.")
        sys.exit(1)
    else:
        print(f"\n{GREEN}🏆 ACTIVE AUDIT PASSED WITH 100% HEALTH SCORE!{RESET} All categories perfectly clean.")
        sys.exit(0)

if __name__ == "__main__":
    main()
