#!/usr/bin/env python3
"""
Conversation Harvester for Daily Dataset Building

Collects conversation pairs from OpenClaw session transcripts and adds them
to the training dataset, then archives processed sessions.

Usage:
    python conversation_harvester.py [--dry-run] [--archive]
"""

import json
import os
import sys
import argparse
from datetime import datetime, timedelta
from pathlib import Path
from typing import Optional

# Dataset paths (workspace-relative)
WORKSPACE = Path("/home/zach/.openclaw/workspace")
DATASET_DIR = WORKSPACE / "dev/Claw/convo2data"
RAW_FILE = DATASET_DIR / "raw_conversations.jsonl"
CLEAN_FILE = DATASET_DIR / "cleaned_conversations.jsonl"
ARCHIVE_DIR = WORKSPACE / "memory/sessions_archive"

# Sessions path
SESSIONS_DIR = Path("/home/zach/.openclaw/agents/main/sessions")

# Days to keep before archiving
ARCHIVE_AFTER_DAYS = 7

# Simple spell/grammar corrections
CORRECTIONS = {
    "thats": "that's", "im": "I'm", "dont": "don't", "cant": "can't",
    "wont": "won't", "isnt": "isn't", "didnt": "didn't", "doesnt": "doesn't",
    "youre": "you're", "theyre": "they're", "were": "we're", "couldnt": "couldn't",
    "shouldnt": "shouldn't", "wouldnt": "wouldn't", "hes": "he's", "shes": "she's",
    "lets": "let's", "rn": "right now", "idk": "I don't know", "btw": "by the way",
    "imo": "in my opinion", "tbh": "to be honest", "lol": "laugh out loud",
    "aka": "also known as", "eg": "for example", "etc": "and so on",
    "asap": "as soon as possible", "fyi": "for your information", "nvm": "never mind",
    "smth": "something", "sb": "somebody", "tho": "though", "kinda": "kind of",
    "sorta": "sort of", "gotta": "got to", "wanna": "want to", "gonna": "going to",
    "lemme": "let me", "ya": "you", "yall": "you all", "ain't": "isn't",
    "u": "you", "ur": "your", "r": "are", "b": "be", "c": "see", "n": "and",
    "w": "with", "m": "am", "plz": "please", "thx": "thanks", "pls": "please",
    "ok": "okay", "yeah": "yes", "yep": "yes", "nope": "no", "lit": "lit",
    "v": "very",
}


def clean_text(text: str) -> str:
    """Clean and normalize text."""
    import re
    # Spelling corrections
    for wrong, correct in CORRECTIONS.items():
        pattern = r'\b' + re.escape(wrong) + r'\b'
        text = re.sub(pattern, correct, text, flags=re.IGNORECASE)
    
    # Grammar fixes
    text = re.sub(r'\bdon\'t have no\b', "don't have any", text, flags=re.IGNORECASE)
    text = re.sub(r'\bcan\'t hardly\b', "can hardly", text, flags=re.IGNORECASE)
    text = re.sub(r'\bcould of\b', "could have", text, flags=re.IGNORECASE)
    text = re.sub(r'\bwould of\b', "would have", text, flags=re.IGNORECASE)
    text = re.sub(r'\bshould of\b', "should have", text, flags=re.IGNORECASE)
    text = re.sub(r'\bhe don\'t\b', "he doesn't", text, flags=re.IGNORECASE)
    text = re.sub(r'\bshe don\'t\b', "she doesn't", text, flags=re.IGNORECASE)
    text = re.sub(r'\bthey was\b', "they were", text, flags=re.IGNORECASE)
    text = re.sub(r'\balot\b', "a lot", text, flags=re.IGNORECASE)
    
    # Normalize whitespace
    text = re.sub(r' +', ' ', text)
    text = re.sub(r'\n\n+', '\n\n', text)
    
    return text.strip()


def parse_sessions():
    """Parse all session transcripts and extract message pairs."""
    pairs = []
    seen_hashes = set()
    
    # Load existing entries to avoid duplicates
    if CLEAN_FILE.exists():
        with open(CLEAN_FILE, 'r', encoding='utf-8') as f:
            for line in f:
                try:
                    entry = json.loads(line)
                    # Create hash of user+assistant to dedupe
                    h = hash((entry.get('user', ''), entry.get('assistant', '')))
                    seen_hashes.add(h)
                except:
                    pass
    
    for session_file in SESSIONS_DIR.glob("*.jsonl"):
        if '.reset.' in session_file.name or session_file.name == 'sessions.json':
            continue
        
        try:
            with open(session_file, 'r', encoding='utf-8') as f:
                messages = []
                for line in f:
                    try:
                        obj = json.loads(line)
                        if obj.get('type') == 'message':
                            msg = obj.get('message', {})
                            if msg.get('role') in ('user', 'assistant'):
                                content = msg.get('content', [])
                                if isinstance(content, list):
                                    text = ' '.join(c.get('text', '') for c in content if c.get('type') == 'text')
                                else:
                                    text = str(content)
                                if text.strip():
                                    messages.append({
                                        'role': msg['role'],
                                        'text': text,
                                        'timestamp': obj.get('timestamp')
                                    })
                    except json.JSONDecodeError:
                        continue
                
                # Pair up user/assistant messages
                for i in range(len(messages) - 1):
                    if messages[i]['role'] == 'user' and messages[i+1]['role'] == 'assistant':
                        user_text = messages[i]['text']
                        assistant_text = messages[i+1]['text']
                        
                        # Skip very short messages or system-like content
                        if len(user_text) < 5 or len(assistant_text) < 5:
                            continue
                        if user_text.startswith('[[') or user_text.startswith('HEARTBEAT'):
                            continue
                        
                        # Dedupe check
                        h = hash((clean_text(user_text), clean_text(assistant_text)))
                        if h not in seen_hashes:
                            pairs.append({
                                'user': user_text,
                                'assistant': assistant_text,
                                'timestamp': messages[i].get('timestamp') or datetime.now().isoformat()
                            })
                            seen_hashes.add(h)
                            
        except Exception as e:
            print(f"Error reading {session_file}: {e}", file=sys.stderr)
    
    return pairs


def add_to_dataset(pairs, dry_run=False):
    """Add conversation pairs to the dataset."""
    if not pairs:
        print("No new conversations to add.")
        return 0
    
    print(f"Adding {len(pairs)} conversation pairs to dataset...")
    
    if dry_run:
        print("[DRY RUN] Would add:")
        for p in pairs[:3]:
            print(f"  User: {p['user'][:80]}...")
            print(f"  Assistant: {p['assistant'][:80]}...")
        if len(pairs) > 3:
            print(f"  ... and {len(pairs) - 3} more")
        return len(pairs)
    
    for p in pairs:
        user_clean = clean_text(p['user'])
        assistant_clean = clean_text(p['assistant'])
        
        # Raw entry
        raw_entry = {
            "timestamp": p['timestamp'],
            "user": p['user'],
            "assistant": p['assistant']
        }
        
        # Cleaned entry
        clean_entry = {
            "timestamp": p['timestamp'],
            "user": user_clean,
            "assistant": assistant_clean,
            "user_original": p['user'],
            "assistant_original": p['assistant']
        }
        
        with open(RAW_FILE, 'a', encoding='utf-8') as f:
            f.write(json.dumps(raw_entry, ensure_ascii=False) + '\n')
        
        with open(CLEAN_FILE, 'a', encoding='utf-8') as f:
            f.write(json.dumps(clean_entry, ensure_ascii=False) + '\n')
    
    return len(pairs)


def archive_old_sessions(dry_run=False):
    """Archive sessions older than ARCHIVE_AFTER_DAYS."""
    ARCHIVE_DIR.mkdir(exist_ok=True)
    
    cutoff = datetime.now() - timedelta(days=ARCHIVE_AFTER_DAYS)
    archived = 0
    
    for session_file in SESSIONS_DIR.glob("*.jsonl"):
        if '.reset.' in session_file.name or session_file.name == 'sessions.json':
            continue
        
        # Check timestamp in filename or file content
        try:
            mtime = datetime.fromtimestamp(session_file.stat().st_mtime)
            if mtime < cutoff:
                archive_path = ARCHIVE_DIR / session_file.name
                if not dry_run:
                    session_file.rename(archive_path)
                archived += 1
                print(f"{'Would archive' if dry_run else 'Archived'}: {session_file.name}")
        except Exception as e:
            print(f"Error processing {session_file}: {e}", file=sys.stderr)
    
    # Clean up lock files
    for lock_file in SESSIONS_DIR.glob("*.lock"):
        try:
            if not dry_run:
                lock_file.unlink()
            print(f"{'Would remove' if dry_run else 'Removed'} lock: {lock_file.name}")
        except:
            pass
    
    return archived


def get_stats():
    """Get current dataset stats."""
    stats = {"raw": 0, "clean": 0, "sessions": 0}
    
    if RAW_FILE.exists():
        with open(RAW_FILE, 'r', encoding='utf-8') as f:
            stats["raw"] = sum(1 for _ in f)
    
    if CLEAN_FILE.exists():
        with open(CLEAN_FILE, 'r', encoding='utf-8') as f:
            stats["clean"] = sum(1 for _ in f)
    
    stats["sessions"] = len(list(SESSIONS_DIR.glob("*.jsonl")))
    
    return stats


def main():
    parser = argparse.ArgumentParser(description="Harvest conversations for dataset")
    parser.add_argument("--dry-run", action="store_true", help="Show what would be done")
    parser.add_argument("--no-archive", action="store_true", help="Skip session archival")
    parser.add_argument("--stats", action="store_true", help="Show current stats")
    args = parser.parse_args()
    
    print(f"=== Conversation Harvester ===")
    print(f"Started: {datetime.now().isoformat()}")
    
    if args.stats:
        stats = get_stats()
        print(f"\nCurrent Stats:")
        print(f"  Raw conversations: {stats['raw']}")
        print(f"  Cleaned conversations: {stats['clean']}")
        print(f"  Active sessions: {stats['sessions']}")
        return
    
    # Parse sessions and extract conversations
    pairs = parse_sessions()
    
    # Add to dataset
    added = add_to_dataset(pairs, dry_run=args.dry_run)
    
    # Archive old sessions
    if not args.no_archive:
        archived = archive_old_sessions(dry_run=args.dry_run)
        print(f"Sessions archived: {archived}")
    
    # Final stats
    stats = get_stats()
    print(f"\nFinal Stats:")
    print(f"  Raw conversations: {stats['raw']}")
    print(f"  Cleaned conversations: {stats['clean']}")
    print(f"  New added: {added}")
    print(f"  Active sessions: {stats['sessions']}")
    print(f"Completed: {datetime.now().isoformat()}")


if __name__ == "__main__":
    main()
