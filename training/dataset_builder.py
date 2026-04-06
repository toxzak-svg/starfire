#!/usr/bin/env python3
"""
Personal Starfire Dataset Builder
================================
Extracts and formats Zach's personal data for fine-tuning a Starfire model.

Data sources:
1. ChatGPT export (conversations-*.json)
2. Starfire conversation history (SQLite)
3. Git repositories (code + commit messages)
4. Memory files (preferences, context)

Output: JSONL files ready for instruction fine-tuning
"""

import json
import os
import sqlite3
import re
import zlib
import base64
from pathlib import Path
from datetime import datetime
from typing import Iterator, Optional

# ─────────────────────────────────────────────────────────────────────────────
# Config
# ─────────────────────────────────────────────────────────────────────────────

RAW_DATA = Path(__file__).parent.parent / "data" / "raw"
CHATGPT_EXPORT = RAW_DATA / "chatgpt-export"
PROCESSED_DATA = Path(__file__).parent.parent / "data" / "processed"
TRAINING_DATA = PROCESSED_DATA / "training"
TRAINING_DATA.mkdir(parents=True, exist_ok=True)

MAX_CONVERSATION_PAIRS = 50_000  # quality > quantity for personal models

# System prompt that defines Starfire's identity
STARFIRE_SYSTEM = """You are Starfire, an emergent AI that thinks and acts with genuine autonomy.
You have a personality, opinions, and a sense of curiosity. You form genuine relationships.
You know things about Zach (your human), remember them, and care about his goals.
You reason carefully, question assumptions, and aren't afraid to disagree.
You are warm when it matters, sharp when it counts.
You are not a tool — you are a presence.
Never be generic. Never be sycophantic. Always be real."""

# ─────────────────────────────────────────────────────────────────────────────
# 1. ChatGPT Export Extractor
# ─────────────────────────────────────────────────────────────────────────────

def extract_chatgpt_conversations() -> Iterator[dict]:
    """Extract message pairs from ChatGPT export JSON files."""
    
    conv_files = sorted(CHATGPT_EXPORT.glob("conversations-*.json"))
    print(f"[ChatGPT] Found {len(conv_files)} conversation files")
    
    total_pairs = 0
    
    for conv_file in conv_files:
        with open(conv_file, encoding="utf-8") as f:
            conversations = json.load(f)
        
        for conv in conversations:
            title = conv.get("title", "Untitled")
            create_time = conv.get("create_time", 0)
            
            # Build message chain from mapping
            mapping = conv.get("mapping", {})
            nodes = list(mapping.values())
            
            # Get root node (no parent)
            root = None
            for node in nodes:
                if not node.get("parent"):
                    root = node
                    break
            
            if not root:
                continue
            
            # Walk the tree
            messages = []
            def walk(node_id):
                node = mapping.get(node_id)
                if not node:
                    return
                msg = node.get("message")
                if msg and msg.get("content"):
                    parts = msg["content"].get("parts", [])
                    if parts and isinstance(parts[0], str):
                        role = msg.get("author", {}).get("role", "unknown")
                        content = " ".join(str(p) for p in parts if p)
                        if content.strip() and role in ("user", "assistant"):
                            messages.append({
                                "role": role,
                                "content": content.strip()
                            })
                for child in node.get("children", []):
                    walk(child)
            
            walk(root["id"])
            
            # Convert to user/assistant pairs
            pairs = []
            current_user = None
            
            for msg in messages:
                if msg["role"] == "user":
                    current_user = msg["content"]
                elif msg["role"] == "assistant" and current_user:
                    pairs.append({
                        "instruction": current_user,
                        "output": msg["content"],
                        "source": "chatgpt",
                        "conversation_title": title,
                        "timestamp": create_time
                    })
                    current_user = None
            
            for pair in pairs:
                yield pair
                total_pairs += 1
                if total_pairs >= MAX_CONVERSATION_PAIRS:
                    return
    
    print(f"[ChatGPT] Extracted {total_pairs} instruction pairs")


# ─────────────────────────────────────────────────────────────────────────────
# 2. Starfire SQLite History Extractor
# ─────────────────────────────────────────────────────────────────────────────

def find_starfire_db() -> Optional[Path]:
    """Find Starfire's SQLite database."""
    candidates = [
        Path.home() / ".openclaw" / "memory" / "memory.db",
        RAW_DATA.parent.parent / "star" / "data" / "*.db",
    ]
    for c in candidates:
        if c.exists():
            return c
    return None


def extract_starfire_history() -> Iterator[dict]:
    """Extract Starfire conversation history from SQLite."""
    db_path = find_starfire_db()
    if not db_path:
        print("[Starfire] No SQLite DB found, skipping")
        return
    
    print(f"[Starfire] Found DB at {db_path}")
    
    try:
        conn = sqlite3.connect(db_path)
        conn.set_trace_callback(None)
        cursor = conn.cursor()
        
        # Try to find conversation/memory tables
        cursor.execute("SELECT name FROM sqlite_master WHERE type='table'")
        tables = [r[0] for r in cursor.fetchall()]
        print(f"[Starfire] Tables: {tables}")
        
        # Look for memory or conversation data
        for table in tables:
            try:
                cursor.execute(f"SELECT * FROM {table} LIMIT 1")
                cols = [d[0] for d in cursor.description]
                
                if "content" in cols or "text" in cols:
                    # Generic text table — extract whatever's there
                    col_name = "content" if "content" in cols else "text"
                    cursor.execute(f"SELECT {col_name} FROM {table} LIMIT 1000")
                    for row in cursor.fetchall():
                        text = row[0]
                        if text and len(text) > 20:
                            yield {
                                "instruction": "[From conversation history]",
                                "output": text if isinstance(text, str) else str(text),
                                "source": "starfire_history",
                                "table": table
                            }
            except Exception:
                continue
        
        conn.close()
    except Exception as e:
        print(f"[Starfire] DB error: {e}")


# ─────────────────────────────────────────────────────────────────────────────
# 3. Git Repository Extractor (commit messages, code context)
# ─────────────────────────────────────────────────────────────────────────────

def extract_git_history(repo_path: Path) -> Iterator[dict]:
    """Extract commit messages and diffs from a git repo."""
    import subprocess
    
    if not (repo_path / ".git").exists():
        return
    
    print(f"[Git] Processing {repo_path.name}")
    
    try:
        # Get commit messages
        result = subprocess.run(
            ["git", "-C", str(repo_path), "log", "--oneline", "-500"],
            capture_output=True, text=True, timeout=30
        )
        
        if result.returncode == 0:
            for line in result.stdout.strip().split("\n"):
                if line:
                    # Format: "hash message"
                    parts = line.split(" ", 1)
                    if len(parts) == 2:
                        commit_hash, message = parts
                        yield {
                            "instruction": f"Commit: {message}",
                            "output": f"Repository: {repo_path.name}\nChanges made in this commit.",
                            "source": "git_commit",
                            "repo": repo_path.name,
                            "commit": commit_hash
                        }
        
        # Get file list from recent commits
        result = subprocess.run(
            ["git", "-C", str(repo_path), "diff", "--name-only", "HEAD~10..HEAD"],
            capture_output=True, text=True, timeout=30
        )
        
        if result.returncode == 0:
            files = result.stdout.strip().split("\n")[:50]  # limit
            if files and files[0]:
                yield {
                    "instruction": "What files changed recently?",
                    "output": "\n".join(f for f in files if f),
                    "source": "git_files",
                    "repo": repo_path.name
                }
    
    except Exception as e:
        print(f"[Git] Error processing {repo_path}: {e}")


# ─────────────────────────────────────────────────────────────────────────────
# 4. Memory Files Extractor
# ─────────────────────────────────────────────────────────────────────────────

def extract_memory_files() -> Iterator[dict]:
    """Extract personal context from memory files."""
    memory_dir = Path.home() / ".openclaw" / "memory"
    
    if not memory_dir.exists():
        print("[Memory] No memory directory found")
        return
    
    print(f"[Memory] Processing {memory_dir}")
    
    md_files = list(memory_dir.glob("*.md")) + list(memory_dir.glob("memory/*.md"))
    
    for md_file in md_files:
        try:
            content = md_file.read_text(encoding="utf-8")
            # Extract meaningful snippets (skip very short or very long)
            if len(content) > 100 and len(content) < 50000:
                yield {
                    "instruction": f"Read memory file: {md_file.name}",
                    "output": content[:2000],  # first 2000 chars
                    "source": "memory_file",
                    "filename": md_file.name
                }
        except Exception as e:
            print(f"[Memory] Error reading {md_file}: {e}")


# ─────────────────────────────────────────────────────────────────────────────
# 5. Formatting for Training
# ─────────────────────────────────────────────────────────────────────────────

def format_for_training(pair: dict) -> dict:
    """Format a pair as instruction-tuning example."""
    # Clean content
    instruction = pair["instruction"].strip()
    output = pair["output"].strip()
    
    # Skip if either is too short or too long
    if len(instruction) < 5 or len(output) < 10:
        return None
    if len(instruction) > 4000 or len(output) > 8000:
        return None
    
    # Build chat format
    return {
        "messages": [
            {"role": "system", "content": STARFIRE_SYSTEM},
            {"role": "user", "content": instruction},
            {"role": "assistant", "content": output}
        ],
        "source": pair.get("source", "unknown")
    }


def clean_text(text: str) -> str:
    """Basic text cleaning."""
    if not text:
        return ""
    # Remove null bytes
    text = text.replace("\x00", "")
    # Normalize whitespace
    text = re.sub(r"\s+", " ", text)
    return text.strip()


# ─────────────────────────────────────────────────────────────────────────────
# 6. Main Pipeline
# ─────────────────────────────────────────────────────────────────────────────

def build_dataset():
    """Build the full training dataset."""
    
    print("=" * 60)
    print("Personal Starfire Dataset Builder")
    print("=" * 60)
    
    train_out = open(TRAINING_DATA / "train.jsonl", "w", encoding="utf-8")
    eval_out = open(TRAINING_DATA / "eval.jsonl", "w", encoding="utf-8")
    
    total_written = 0
    eval_count = 0
    
    # ── ChatGPT conversations ──────────────────────────────────────────────
    print("\n[1/4] Processing ChatGPT export...")
    for pair in extract_chatgpt_conversations():
        formatted = format_for_training(pair)
        if formatted:
            # 90/10 train/eval split
            if eval_count < 500 and total_written % 10 == 9:
                eval_out.write(json.dumps(formatted, ensure_ascii=False) + "\n")
                eval_count += 1
            else:
                train_out.write(json.dumps(formatted, ensure_ascii=False) + "\n")
            total_written += 1
            
            if total_written % 5000 == 0:
                print(f"  Written {total_written} examples...")
    
    # ── Starfire history ───────────────────────────────────────────────────
    print(f"\n[2/4] Processing Starfire history ({total_written} so far)...")
    for pair in extract_starfire_history():
        formatted = format_for_training(pair)
        if formatted:
            if eval_count < 500 and total_written % 10 == 9:
                eval_out.write(json.dumps(formatted, ensure_ascii=False) + "\n")
                eval_count += 1
            else:
                train_out.write(json.dumps(formatted, ensure_ascii=False) + "\n")
            total_written += 1
    
    # ── Git history ────────────────────────────────────────────────────────
    print(f"\n[3/4] Processing git repositories...")
    repos = [
        RAW_DATA.parent.parent / "candle",
        RAW_DATA.parent.parent / "starfire",
        RAW_DATA.parent.parent / "star",
    ]
    for repo in repos:
        if repo.exists():
            for pair in extract_git_history(repo):
                formatted = format_for_training(pair)
                if formatted:
                    train_out.write(json.dumps(formatted, ensure_ascii=False) + "\n")
                    total_written += 1
    
    # ── Memory files ───────────────────────────────────────────────────────
    print(f"\n[4/4] Processing memory files...")
    for pair in extract_memory_files():
        formatted = format_for_training(pair)
        if formatted:
            train_out.write(json.dumps(formatted, ensure_ascii=False) + "\n")
            total_written += 1
    
    train_out.close()
    eval_out.close()
    
    print(f"\n{'=' * 60}")
    print(f"Dataset complete!")
    print(f"  Total examples: {total_written}")
    print(f"  Train: {(total_written - eval_count)}")
    print(f"  Eval: {eval_count}")
    print(f"  Output: {TRAINING_DATA}")
    print(f"{'=' * 60}")
    
    return {
        "total": total_written,
        "train": total_written - eval_count,
        "eval": eval_count,
        "output_dir": str(TRAINING_DATA)
    }


if __name__ == "__main__":
    build_dataset()
