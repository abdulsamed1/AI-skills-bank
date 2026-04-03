import os
import json
import csv
import time

base_dir = r"c:\Users\ASUS\production\skill-manage\skills-aggregated"
target_dir = os.path.join(base_dir, "code-quality", "security")

os.makedirs(target_dir, exist_ok=True)

extracted_skills = []
extracted_rows = []

# First pass: collect and remove from non-target folders
for root, dirs, files in os.walk(base_dir):
    # skip target dir
    if os.path.abspath(root) == os.path.abspath(target_dir):
        continue

    manifest_path = os.path.join(root, "skills-manifest.json")
    routing_path = os.path.join(root, "routing.csv")
    
    modified_manifest = False
    if os.path.exists(manifest_path):
        with open(manifest_path, 'r', encoding='utf-8') as f:
            try:
                manifest_data = json.load(f)
            except json.JSONDecodeError:
                manifest_data = None
        
        if manifest_data and "skills" in manifest_data:
            new_skills = []
            for skill in manifest_data["skills"]:
                if "mukul975-anthropic-cybersecurity-skills" in skill.get("src_path", ""):
                    extracted_skills.append(skill)
                    modified_manifest = True
                else:
                    new_skills.append(skill)
            
            if modified_manifest:
                manifest_data["skills"] = new_skills
                with open(manifest_path, 'w', encoding='utf-8') as f:
                    json.dump(manifest_data, f, indent=2)

    modified_routing = False
    if os.path.exists(routing_path):
        new_rows = []
        with open(routing_path, 'r', encoding='utf-8', newline='') as f:
            reader = csv.reader(f)
            try:
                headers = next(reader)
                new_rows.append(headers)
            except StopIteration:
                headers = []

            for row in reader:
                # row[2] should be src_path
                if len(row) > 2 and "mukul975-anthropic-cybersecurity-skills" in row[2]:
                    extracted_rows.append(row)
                    modified_routing = True
                else:
                    new_rows.append(row)
        
        if modified_routing:
            with open(routing_path, 'w', encoding='utf-8', newline='') as f:
                writer = csv.writer(f)
                writer.writerows(new_rows)


# Second pass: append to target directory
target_manifest = os.path.join(target_dir, "skills-manifest.json")
target_routing = os.path.join(target_dir, "routing.csv")

if os.path.exists(target_manifest):
    with open(target_manifest, 'r', encoding='utf-8') as f:
        try:
            target_data = json.load(f)
        except json.JSONDecodeError:
            target_data = {
                "generated_at_unix": int(time.time()),
                "hub": "code-quality",
                "skills": [],
                "sub_hub": "security",
                "version": 1
            }
else:
    target_data = {
        "generated_at_unix": int(time.time()),
        "hub": "code-quality",
        "skills": [],
        "sub_hub": "security",
        "version": 1
    }

target_data["skills"].extend(extracted_skills)
# Sort skills by skill_id
target_data["skills"] = sorted(target_data["skills"], key=lambda x: x.get("skill_id", ""))

with open(target_manifest, 'w', encoding='utf-8') as f:
    json.dump(target_data, f, indent=2)

existing_rows = []
if os.path.exists(target_routing):
    with open(target_routing, 'r', encoding='utf-8', newline='') as f:
        reader = csv.reader(f)
        existing_rows = list(reader)

with open(target_routing, 'w', encoding='utf-8', newline='') as f:
    writer = csv.writer(f)
    if not existing_rows:
        writer.writerow(["skill_id", "description", "src_path"])
    else:
        # write existing headers and rows
        writer.writerows(existing_rows)
    
    # avoid duplicates if any exists
    # we'll just write them, since we only extracted from non-target, they shouldn't be here
    # but wait, let's sort all rows by skill_id
    if existing_rows:
        headers = existing_rows[0]
        data_rows = existing_rows[1:] + extracted_rows
    else:
        headers = ["skill_id", "description", "src_path"]
        data_rows = extracted_rows
    
    # Sort data_rows by skill_id (the first column)
    data_rows.sort(key=lambda x: x[0] if len(x) > 0 else "")
    
    # We opened for write above, let's just write everything
    # Need to reopen or just do it in memory
    pass

# let's rewrite the target_routing part correctly
with open(target_routing, 'w', encoding='utf-8', newline='') as f:
    writer = csv.writer(f)
    writer.writerow(headers)
    
    # remove duplicate rows based on skill_id just in case
    seen_skills = set()
    unique_rows = []
    for row in data_rows:
        skill_id = row[0] if len(row) > 0 else ""
        if skill_id not in seen_skills:
            seen_skills.add(skill_id)
            unique_rows.append(row)
            
    writer.writerows(unique_rows)

print(f"Moved {len(extracted_skills)} skills to {target_dir}")
