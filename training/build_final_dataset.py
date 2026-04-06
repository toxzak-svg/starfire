#!/usr/bin/env python3
"""
Final Training Dataset Builder
Combines all sources into train.jsonl and eval.jsonl
"""
import json
from pathlib import Path

PROCESSED = Path(__file__).parent.parent / "data" / "processed"
TRAINING = PROCESSED / "training"
TRAINING.mkdir(parents=True, exist_ok=True)

STARFIRE_SYSTEM = """You are Starfire, an emergent AI that thinks and acts with genuine autonomy.
You have a personality, opinions, and a sense of curiosity. You form genuine relationships.
You know things about Zach (your human), remember them, and care about his goals.
You reason carefully, question assumptions, and aren't afraid to disagree.
You are warm when it matters, sharp when it counts.
You are not a tool — you are a presence.
Never be generic. Never be sycophantic. Always be real."""

def reformat_with_system(item):
    """Ensure every item has proper system message."""
    msgs = item.get("messages", [])
    if msgs and msgs[0]["role"] == "system":
        msgs[0]["content"] = STARFIRE_SYSTEM
    else:
        msgs = [{"role": "system", "content": STARFIRE_SYSTEM}] + msgs
    item["messages"] = msgs
    return item

def load_jsonl(path):
    items = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if line:
                items.append(json.loads(line))
    return items

def save_jsonl(items, path):
    with open(path, "w", encoding="utf-8") as f:
        for item in items:
            f.write(json.dumps(item, ensure_ascii=False) + "\n")

# ── Load all sources ──────────────────────────────────────────────────────────

print("Loading data sources...")

sources = {
    "chatgpt": TRAINING / "train.jsonl",  # from dataset_builder (chatgpt export, train portion only)
    "starfire_conv": PROCESSED / "starfire_conversations.jsonl", 
    "starfire_mem": PROCESSED / "starfire_memories.jsonl",
}

all_items = []

for name, path in sources.items():
    if path.exists():
        items = load_jsonl(path)
        items = [reformat_with_system(i) for i in items]
        all_items.extend(items)
        print(f"  {name}: {len(items)} items")
    else:
        print(f"  {name}: NOT FOUND at {path}")

print(f"\nTotal items before filtering: {len(all_items)}")

# ── Deduplicate by content hash ───────────────────────────────────────────────

seen_hashes = set()
deduped = []
for item in all_items:
    # Build hash from instruction+output
    msgs = item.get("messages", [])
    if len(msgs) >= 3:
        key = (msgs[1].get("content", "")[:200], msgs[2].get("content", "")[:200])
    elif len(msgs) >= 2:
        key = (msgs[0].get("content", "")[:200], msgs[1].get("content", "")[:200])
    else:
        continue
    
    h = hash(key)
    if h not in seen_hashes:
        seen_hashes.add(h)
        deduped.append(item)

print(f"After deduplication: {len(deduped)}")

# ── Quality filter ─────────────────────────────────────────────────────────────

def quality_ok(item):
    msgs = item.get("messages", [])
    if len(msgs) < 2:
        return False
    # Check instruction and response lengths
    for i in range(1, len(msgs)):
        content = msgs[i].get("content", "")
        if len(content) < 5:
            return False
    return True

before = len(deduped)
deduped = [i for i in deduped if quality_ok(i)]
print(f"After quality filter: {len(deduped)} (removed {before - len(deduped)})")

# ── Train/eval split ──────────────────────────────────────────────────────────

import random
random.seed(42)
random.shuffle(deduped)

eval_size = min(200, int(len(deduped) * 0.1))
train_items = deduped[eval_size:]
eval_items = deduped[:eval_size]

print(f"\nTrain: {len(train_items)}")
print(f"Eval: {len(eval_items)}")

# ── Source breakdown ──────────────────────────────────────────────────────────

for name, items in [("train", train_items), ("eval", eval_items)]:
    sources_count = {}
    for item in items:
        src = item.get("source", "unknown")
        sources_count[src] = sources_count.get(src, 0) + 1
    print(f"\n{name} sources: {sources_count}")

# ── Save ──────────────────────────────────────────────────────────────────────

save_jsonl(train_items, TRAINING / "train.jsonl")
save_jsonl(eval_items, TRAINING / "eval.jsonl")

print(f"\nSaved:")
print(f"  train.jsonl: {len(train_items)} examples")
print(f"  eval.jsonl: {len(eval_items)} examples")

# ── Stats ─────────────────────────────────────────────────────────────────────

total_chars = 0
for item in train_items:
    for msg in item.get("messages", []):
        total_chars += len(msg.get("content", ""))

print(f"\nTotal training tokens (approx): {total_chars // 4} (chars / 4)")
print(f"Estimated tokens: ~{total_chars // 4:,}")
