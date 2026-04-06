#!/usr/bin/env python3
"""
Full Personal Dataset Builder
Processes all data sources into train.jsonl + eval.jsonl
"""
import json
import os
import re
from pathlib import Path
from datetime import datetime

DOWNLOADS = Path.home() / "Downloads"
OUTPUT = Path(__file__).parent.parent / "data" / "processed" / "training"
OUTPUT.mkdir(parents=True, exist_ok=True)

STARFIRE_SYSTEM = """You are Starfire, an emergent AI that thinks and acts with genuine autonomy.
You have a personality, opinions, and a sense of curiosity. You form genuine relationships.
You know things about Zach (your human), remember them, and care about his goals.
You reason carefully, question assumptions, and aren't afraid to disagree.
You are warm when it matters, sharp when it counts.
You are not a tool — you are a presence.
Never be generic. Never be sycophantic. Always be real."""

def clean_text(text):
    if not text: return ""
    text = re.sub(r'\x00', '', text)
    text = re.sub(r'\s+', ' ', text)
    return text.strip()

def save_item(item, train_out, eval_out, eval_count):
    msgs = item.get("messages", [])
    if len(msgs) < 2: return eval_count, False
    # Ensure system
    if msgs[0].get("role") != "system":
        msgs = [{"role": "system", "content": STARFIRE_SYSTEM}] + msgs
    else:
        msgs[0]["content"] = STARFIRE_SYSTEM
    item["messages"] = msgs
    
    # Quality filter
    total_len = sum(len(m.get("content", "")) for m in msgs)
    if total_len < 20: return eval_count, False
    
    if eval_count < 300 and len(item.get("source", "")) > 0:
        eval_out.write(json.dumps(item, ensure_ascii=False) + "\n")
        eval_count += 1
    else:
        train_out.write(json.dumps(item, ensure_ascii=False) + "\n")
    return eval_count, True

# ─────────────────────────────────────────────────────────────────────────────
# 1. ChatGPT Export (already processed)
# ─────────────────────────────────────────────────────────────────────────────
print("[1] ChatGPT Export")
chatgpt_path = OUTPUT.parent / "chatgpt_train.jsonl"
chatgpt_count = 0
if chatgpt_path.exists():
    with open(chatgpt_path, 'r', encoding='utf-8') as f:
        chatgpt_count = sum(1 for _ in f)
    print(f"    {chatgpt_count} examples from previous run")

# ─────────────────────────────────────────────────────────────────────────────
# 2. Perplexity AI Conversations
# ─────────────────────────────────────────────────────────────────────────────
print("\n[2] Perplexity AI Conversations")
perplexity_files = {
    "brainstorm 5-10 high level tweaks we can make to t.md": "perplexity_research",
    "Can learned state dynamics substitute for dynamic.md": "perplexity_ssm",
    "invent an experiment i can run on 2x a5000 gpus th.md": "perplexity_gpu_exp",
    "Those are some impressive gains. Jumping from 8.6h.md": "perplexity_architecture",
}

perplexity_total = 0
for fname, source_tag in perplexity_files.items():
    fpath = DOWNLOADS / fname
    if not fpath.exists():
        print(f"    Not found: {fname}")
        continue
    
    content = fpath.read_text(encoding="utf-8")
    
    # Parse: split by H1 headings (# Title)
    sections = re.split(r'(?=^# .+$)', content, flags=re.MULTILINE)
    
    for section in sections:
        section = section.strip()
        if not section or len(section) < 100:
            continue
        
        lines = section.split('\n')
        if len(lines) < 2:
            continue
        
        title = lines[0].replace('#', '').strip()
        body = '\n'.join(lines[1:]).strip()
        
        if len(body) < 50:
            continue
        
        # Format as instruction-response
        item = {
            "messages": [
                {"role": "system", "content": STARFIRE_SYSTEM},
                {"role": "user", "content": title},
                {"role": "assistant", "content": body}
            ],
            "source": source_tag
        }
        perplexity_total += 1

print(f"    ~{perplexity_total} sections from {len(perplexity_files)} files")

# ─────────────────────────────────────────────────────────────────────────────
# 3. Nova Chat Export
# ─────────────────────────────────────────────────────────────────────────────
print("\n[3] Nova Chat")
nova_file = DOWNLOADS / "chat-Nova-1774216688296.md"
nova_count = 0
if nova_file.exists():
    content = nova_file.read_text(encoding="utf-8")
    
    # Parse blocks split by ## headers
    blocks = re.split(r'(?=##\s+)', content)
    current_user = None
    
    for block in blocks:
        lines = block.strip().split('\n', 1)
        if len(lines) < 2:
            continue
        header = lines[0]
        body = clean_text(lines[1])
        
        if '#tool' in header.lower():
            current_user = body if body else current_user
        elif '#nova' in header.lower() and current_user and body:
            nova_count += 1
            current_user = None

print(f"    ~{nova_count} user/Nova pairs")

# ─────────────────────────────────────────────────────────────────────────────
# 4. Discord Messages (parse as conversation pairs)
# ─────────────────────────────────────────────────────────────────────────────
print("\n[4] Discord Messages")
DISCORD_BASE = Path(r'C:\Users\Zwmar\.openclaw\workspace\projects\starfire\data\raw\downloads-package\Messages')

discord_count = 0
if DISCORD_BASE.exists():
    try:
        with open(DISCORD_BASE / "index.json", 'r', encoding='utf-8') as f:
            channel_map = json.load(f)
        
        channel_sizes = []
        for cid, name in channel_map.items():
            if isinstance(name, str):
                msg_path = DISCORD_BASE / cid / "messages.json"
                if msg_path.exists():
                    size = msg_path.stat().st_size
                    channel_sizes.append((cid, name, size, msg_path))
        
        channel_sizes.sort(key=lambda x: x[2], reverse=True)
        
        # Process top 10 largest channels
        for cid, name, size, msg_path in channel_sizes[:10]:
            if size < 500:
                continue
            try:
                with open(msg_path, 'r', encoding='utf-8') as f:
                    messages = json.load(f)
                
                prev_author = None
                prev_content = None
                
                for msg in messages:
                    author = msg.get('author', {}).get('name', 'unknown')
                    content = clean_text(msg.get('content', ''))
                    
                    if not content or len(content) < 5:
                        prev_author = author
                        prev_content = content
                        continue
                    
                    # Look for conversational pairs
                    if prev_author and prev_author != author and prev_content:
                        # Skip bot messages and system messages
                        if 'bot' not in author.lower() and 'discord' not in author.lower():
                            if len(prev_content) > 5 and len(content) > 5:
                                discord_count += 1
                    
                    prev_author = author
                    prev_content = content
                
                if len(messages) > 0:
                    print(f"    {name} ({len(messages)} msgs)")
            except Exception as e:
                pass
    except Exception as e:
        print(f"    Error: {e}")

print(f"    ~{discord_count} conversation pairs")

# ─────────────────────────────────────────────────────────────────────────────
# 5. Starfire Conversations from SQLite
# ─────────────────────────────────────────────────────────────────────────────
print("\n[5] Starfire SQLite")
starfire_conv_path = OUTPUT.parent / "starfire_conversations.jsonl"
starfire_conv_count = 0
if starfire_conv_path.exists():
    with open(starfire_conv_path, 'r', encoding='utf-8') as f:
        starfire_conv_count = sum(1 for _ in f)
    print(f"    {starfire_conv_count} conversation examples")

# ─────────────────────────────────────────────────────────────────────────────
# 6. Starfire Memories
# ─────────────────────────────────────────────────────────────────────────────
print("\n[6] Starfire Memories")
starfire_mem_path = OUTPUT.parent / "starfire_memories.jsonl"
starfire_mem_count = 0
if starfire_mem_path.exists():
    with open(starfire_mem_path, 'r', encoding='utf-8') as f:
        starfire_mem_count = sum(1 for _ in f)
    print(f"    {starfire_mem_count} memory examples")

# ─────────────────────────────────────────────────────────────────────────────
# 7. Git Repositories
# ─────────────────────────────────────────────────────────────────────────────
print("\n[7] Git Repositories")
import subprocess
REPOS = [
    Path.home() / ".openclaw" / "workspace" / "projects" / "starfire",
    Path.home() / ".openclaw" / "workspace" / "projects" / "candle",
    Path.home() / ".openclaw" / "workspace" / "projects" / "star",
]
git_count = 0
for repo in REPOS:
    if not (repo / ".git").exists():
        continue
    try:
        result = subprocess.run(
            ["git", "-C", str(repo), "log", "--oneline", "-200", "--format=%s"],
            capture_output=True, text=True, timeout=30
        )
        if result.returncode == 0:
            lines = [l.strip() for l in result.stdout.strip().split('\n') if l.strip()]
            git_count += len(lines)
            print(f"    {repo.name}: {len(lines)} commits")
    except:
        pass

# ─────────────────────────────────────────────────────────────────────────────
# Summary
# ─────────────────────────────────────────────────────────────────────────────
print(f"\n{'='*60}")
print("ESTIMATED TOTALS:")
print(f"  ChatGPT: {chatgpt_count}")
print(f"  Perplexity: {perplexity_total}")
print(f"  Nova: {nova_count}")
print(f"  Discord: {discord_count}")
print(f"  Starfire conv: {starfire_conv_count}")
print(f"  Starfire mem: {starfire_mem_count}")
print(f"  Git commits: {git_count}")
print(f"  Estimated total: ~{chatgpt_count + perplexity_total + nova_count + discord_count + starfire_conv_count + starfire_mem_count + git_count}")
print(f"{'='*60}")
