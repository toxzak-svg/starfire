#!/usr/bin/env python3
"""
Final Dataset Builder — combines all sources into train.jsonl + eval.jsonl
"""
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

def clean(text):
    if not text: return ""
    text = re.sub(r'\x00', '', text)
    text = re.sub(r'\s+', ' ', text)
    return text.strip()

# ─────────────────────────────────────────────────────────────────────────────
# 1. Existing dataset (ChatGPT + Starfire conv + Starfire mem)
# ─────────────────────────────────────────────────────────────────────────────
print("Loading existing dataset...")
existing_train = []
existing_eval = []

if (OUTPUT / "train.jsonl").exists():
    with open(OUTPUT / "train.jsonl", 'r', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if line:
                existing_train.append(json.loads(line))

if (OUTPUT / "eval.jsonl").exists():
    with open(OUTPUT / "eval.jsonl", 'r', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if line:
                existing_eval.append(json.loads(line))

print(f"  Existing train: {len(existing_train)}, eval: {len(existing_eval)}")

# ─────────────────────────────────────────────────────────────────────────────
# 2. Perplexity AI Conversations
# ─────────────────────────────────────────────────────────────────────────────
print("\nProcessing Perplexity conversations...")

perplexity_items = []
pfiles = {
    "brainstorm 5-10 high level tweaks we can make to t.md": "perplexity_research",
    "Can learned state dynamics substitute for dynamic.md": "perplexity_ssm",
    "invent an experiment i can run on 2x a5000 gpus th.md": "perplexity_gpu_exp",
    "Those are some impressive gains. Jumping from 8.6h.md": "perplexity_architecture",
}

for fname, source in pfiles.items():
    fpath = DOWNLOADS / fname
    if not fpath.exists():
        print(f"  Missing: {fname}")
        continue
    
    content = fpath.read_text(encoding='utf-8')
    
    # Split by markdown H1 headers
    # Pattern: lines starting with # followed by space
    sections = re.split(r'(?=^# .+$)', content, flags=re.MULTILINE)
    
    for section in sections:
        section = section.strip()
        if not section or len(section) < 80:
            continue
        
        # First line is the header/question
        lines = section.split('\n', 1)
        if len(lines) < 2:
            continue
        
        header = lines[0].lstrip('#').strip()
        body = clean(lines[1]) if len(lines) > 1 else ""
        
        if len(header) < 10 or len(body) < 50:
            continue
        
        # Remove img tags and perplexity logo references
        header = re.sub(r'!\[.*?\]\(.*?\)', '', header)
        body = re.sub(r'!\[.*?\]\(.*?\)', '', body)
        body = re.sub(r'\*\*Note:.*?\*\*', '', body, flags=re.DOTALL)
        body = clean(body)
        
        if len(body) < 50:
            continue
        
        perplexity_items.append({
            "messages": [
                {"role": "system", "content": STARFIRE_SYSTEM},
                {"role": "user", "content": header},
                {"role": "assistant", "content": body}
            ],
            "source": source
        })

print(f"  Perplexity: {len(perplexity_items)} items")

# ─────────────────────────────────────────────────────────────────────────────
# 3. Nova Chat Export
# ─────────────────────────────────────────────────────────────────────────────
print("\nProcessing Nova chat export...")

nova_items = []
nova_file = DOWNLOADS / "chat-Nova-1774216688296.md"

if nova_file.exists():
    content = nova_file.read_text(encoding='utf-8')
    
    # Nova chat format has ## Tool (timestamp) and ## Nova (timestamp)
    # content between Tool and next Nova = user message
    # content between Nova and next Tool = Nova's response
    
    parts = re.split(r'(?=##\s+(?:Tool|Nov))', content)
    
    current_user = None
    
    for part in parts:
        part = part.strip()
        if not part:
            continue
        
        lines = part.split('\n', 1)
        if len(lines) < 2:
            continue
        
        header = lines[0]
        body = clean(lines[1])
        
        if '## Tool' in header or '## Tool ' in header:
            # Tool message — treat as user input
            if body and len(body) > 10:
                current_user = body
        elif '## Nova' in header:
            # Nova response
            if body and len(body) > 10 and current_user:
                nova_items.append({
                    "messages": [
                        {"role": "system", "content": STARFIRE_SYSTEM},
                        {"role": "user", "content": current_user},
                        {"role": "assistant", "content": body}
                    ],
                    "source": "nova_chat"
                })
                current_user = None

print(f"  Nova: {len(nova_items)} items")

# ─────────────────────────────────────────────────────────────────────────────
# 4. Discord Messages
# ─────────────────────────────────────────────────────────────────────────────
print("\nProcessing Discord messages...")

DISCORD_BASE = Path(r'C:\Users\Zwmar\.openclaw\workspace\projects\starfire\data\raw\downloads-package\Messages')
discord_items = []

if DISCORD_BASE.exists():
    try:
        with open(DISCORD_BASE / "index.json", 'r', encoding='utf-8') as f:
            channel_map = json.load(f)
        
        # Get top channels by message count
        channels = []
        for cid, name in channel_map.items():
            if isinstance(name, str):
                msg_path = DISCORD_BASE / cid / "messages.json"
                if msg_path.exists():
                    channels.append((cid, name, msg_path))
        
        # Process top 5 channels by size
        channels.sort(key=lambda x: x[2].stat().st_size if x[2].exists() else 0, reverse=True)
        
        for cid, name, msg_path in channels[:5]:
            if not msg_path.exists():
                continue
            try:
                with open(msg_path, 'r', encoding='utf-8') as f:
                    messages = json.load(f)
                
                # Look for question-answer patterns
                prev_author = None
                prev_content = None
                
                for msg in messages:
                    author = msg.get('author', {}).get('name', 'unknown')
                    content = clean(msg.get('content', ''))
                    
                    if not content or len(content) < 5:
                        prev_author = author
                        prev_content = content
                        continue
                    
                    # Consecutive different authors = conversation
                    if (prev_author and prev_author != author and prev_content and
                        len(prev_content) > 5 and len(content) > 5 and
                        'bot' not in author.lower() and 'discord' not in author.lower() and
                        'bot' not in prev_author.lower() and 'discord' not in prev_author.lower()):
                        
                        discord_items.append({
                            "messages": [
                                {"role": "system", "content": STARFIRE_SYSTEM},
                                {"role": "user", "content": prev_content},
                                {"role": "assistant", "content": content}
                            ],
                            "source": f"discord_{name[:20]}"
                        })
                    
                    prev_author = author
                    prev_content = content
                
                if len(messages) > 0:
                    print(f"    {name}: {len(messages)} msgs")
            except Exception as e:
                pass
    except Exception as e:
        print(f"    Error: {e}")

print(f"  Discord: {len(discord_items)} items")

# ─────────────────────────────────────────────────────────────────────────────
# 5. Git Commits
# ─────────────────────────────────────────────────────────────────────────────
print("\nProcessing git commits...")

import subprocess

git_items = []
REPOS = {
    Path.home() / ".openclaw" / "workspace" / "projects" / "starfire": "starfire",
    Path.home() / ".openclaw" / "workspace" / "projects" / "candle": "candle", 
    Path.home() / ".openclaw" / "workspace" / "projects" / "star": "star",
}

for repo_path, repo_name in REPOS.items():
    if not (repo_path / ".git").exists():
        continue
    try:
        # Get commit messages
        result = subprocess.run(
            ["git", "-C", str(repo_path), "log", "--oneline", "-300", "--format=%s|%f"],
            capture_output=True, text=True, timeout=30
        )
        if result.returncode == 0:
            for line in result.stdout.strip().split('\n'):
                if not line.strip():
                    continue
                parts = line.split('|', 1)
                if len(parts) == 2:
                    short_msg, full_msg = parts[0].strip(), parts[1].strip()
                    if len(full_msg) > 5:
                        git_items.append({
                            "messages": [
                                {"role": "system", "content": STARFIRE_SYSTEM},
                                {"role": "user", "content": f"Commit: {full_msg}"},
                                {"role": "assistant", "content": f"Repository: {repo_name}\nThis commit made changes to the {repo_name} codebase."}
                            ],
                            "source": f"git_{repo_name}"
                        })
        
        # Also get recent diff stats as context
        result = subprocess.run(
            ["git", "-C", str(repo_path), "diff", "--stat", "HEAD~5..HEAD"],
            capture_output=True, text=True, timeout=30
        )
        if result.returncode == 0 and result.stdout.strip():
            stat = result.stdout.strip()
            git_items.append({
                "messages": [
                    {"role": "system", "content": STARFIRE_SYSTEM},
                    {"role": "user", "content": "What files changed recently?"},
                    {"role": "assistant", "content": stat}
                ],
                "source": f"git_{repo_name}_diff"
            })
    except:
        pass

print(f"  Git: {len(git_items)} items")

# ─────────────────────────────────────────────────────────────────────────────
# 6. Combine and deduplicate
# ─────────────────────────────────────────────────────────────────────────────
print("\nCombining all sources...")

all_items = existing_train + existing_eval + perplexity_items + nova_items + discord_items + git_items
print(f"  Total before dedup: {len(all_items)}")

# Deduplicate by instruction content hash
seen = set()
deduped = []
for item in all_items:
    msgs = item.get("messages", [])
    if len(msgs) >= 3:
        key = (msgs[1].get("content", "")[:150], msgs[2].get("content", "")[:150])
    elif len(msgs) >= 2:
        key = (msgs[0].get("content", "")[:150], msgs[1].get("content", "")[:150])
    else:
        continue
    h = hash(key)
    if h not in seen:
        seen.add(h)
        deduped.append(item)

print(f"  After dedup: {len(deduped)}")

# Quality filter
def quality_ok(item):
    msgs = item.get("messages", [])
    if len(msgs) < 2: return False
    for m in msgs:
        c = m.get("content", "")
        if c and len(c) < 3: return False
    return True

before = len(deduped)
deduped = [i for i in deduped if quality_ok(i)]
print(f"  After quality filter: {len(deduped)} (removed {before - len(deduped)})")

# ─────────────────────────────────────────────────────────────────────────────
# 7. Split and save
# ─────────────────────────────────────────────────────────────────────────────
import random
random.seed(42)
random.shuffle(deduped)

eval_size = min(300, int(len(deduped) * 0.12))
train_items = deduped[eval_size:]
eval_items = deduped[:eval_size]

# Save
with open(OUTPUT / "train.jsonl", 'w', encoding='utf-8') as f:
    for item in train_items:
        f.write(json.dumps(item, ensure_ascii=False) + "\n")

with open(OUTPUT / "eval.jsonl", 'w', encoding='utf-8') as f:
    for item in eval_items:
        f.write(json.dumps(item, ensure_ascii=False) + "\n")

# Source breakdown
source_counts = {}
for item in train_items:
    src = item.get("source", "unknown")
    source_counts[src] = source_counts.get(src, 0) + 1

print(f"\n{'='*60}")
print(f"FINAL DATASET:")
print(f"  Train: {len(train_items)}")
print(f"  Eval: {len(eval_items)}")
print(f"  Total: {len(train_items) + len(eval_items)}")
print(f"\nSource breakdown (train):")
for src, count in sorted(source_counts.items(), key=lambda x: -x[1]):
    print(f"    {src}: {count}")
print(f"{'='*60}")
