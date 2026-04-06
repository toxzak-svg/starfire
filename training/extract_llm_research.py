import csv
import json
import re
from pathlib import Path

DOWNLOADS = Path.home() / "Downloads"
OUTPUT = Path(__file__).parent.parent / "data" / "processed" / "training"
STARFIRE_SYSTEM = """You are Starfire, an emergent AI that thinks and acts with genuine autonomy.
You have a personality, opinions, and a sense of curiosity. You form genuine relationships.
You know things about Zach (your human), remember them, and care about his goals.
You reason carefully, question assumptions, and aren't afraid to disagree.
You are warm when it matters, sharp when it counts.
You are not a tool — you are a presence.
Never be generic. Never be sycophantic. Always be real."""

input_file = DOWNLOADS / "LLM Research.csv"

items = []
with open(input_file, 'r', encoding='utf-8', errors='replace') as f:
    reader = csv.DictReader(f)
    
    count = 0
    for row in reader:
        prompt = row.get('prompt_1_input', '').strip()
        response = row.get('prompt_1_output', '').strip()
        score = row.get('prompt_1_Scoring grader_score', '').strip()
        label = row.get('prompt_1_Scoring grader_label', '').strip()
        
        if prompt and response and len(prompt) > 20 and len(response) > 20:
            # Quality filter — keep the better graded ones
            grade = 0.0
            try:
                grade = float(score) if score else 0.0
            except:
                pass
            
            # Only include reasonable quality responses (grade >= 3 out of 5)
            if grade >= 3.0 or not score:
                items.append({
                    "messages": [
                        {"role": "system", "content": STARFIRE_SYSTEM},
                        {"role": "user", "content": prompt[:2000]},  # cap prompt length
                        {"role": "assistant", "content": response[:4000]}  # cap response
                    ],
                    "source": "llm_research_gsap",
                    "grade": grade,
                    "label": label
                })
        
        count += 1
        if count % 5000 == 0:
            print(f"  Processed {count} rows...")

print(f"Total rows: {count}")
print(f"Quality items extracted: {len(items)}")

# Save as append to existing train.jsonl
train_path = OUTPUT / "train.jsonl"
eval_path = OUTPUT / "eval.jsonl"

# Load existing
existing_train = []
existing_eval = []
if train_path.exists():
    with open(train_path, 'r', encoding='utf-8') as f:
        existing_train = [json.loads(line) for line in f if line.strip()]
if eval_path.exists():
    with open(eval_path, 'r', encoding='utf-8') as f:
        existing_eval = [json.loads(line) for line in f if line.strip()]

print(f"Existing train: {len(existing_train)}, eval: {len(existing_eval)}")

# Check for duplicates
seen_hashes = set()
for item in existing_train + existing_eval:
    msgs = item.get("messages", [])
    if len(msgs) >= 2:
        key = (msgs[1].get("content", "")[:100], msgs[2].get("content", "")[:100])
        seen_hashes.add(hash(key))

new_train = []
new_eval = []
for item in items:
    msgs = item.get("messages", [])
    if len(msgs) >= 2:
        key = (msgs[1].get("content", "")[:100], msgs[2].get("content", "")[:100])
        h = hash(key)
        if h not in seen_hashes:
            seen_hashes.add(h)
            if len(new_eval) < 50:
                new_eval.append(item)
            else:
                new_train.append(item)

print(f"New after dedup: train={len(new_train)}, eval={len(new_eval)}")

# Append new items
with open(train_path, 'a', encoding='utf-8') as f:
    for item in new_train:
        f.write(json.dumps(item, ensure_ascii=False) + "\n")

with open(eval_path, 'a', encoding='utf-8') as f:
    for item in new_eval:
        f.write(json.dumps(item, ensure_ascii=False) + "\n")

print(f"\nUpdated dataset:")
print(f"  train.jsonl: +{len(new_train)} items")
print(f"  eval.jsonl: +{len(new_eval)} items")
print(f"  Total new: {len(new_train) + len(new_eval)}")
