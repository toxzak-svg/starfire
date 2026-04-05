#!/usr/bin/env python3
"""
Populate Starfire's knowledge graph from seed_knowledge.json.

This script:
1. Loads the seed knowledge JSON
2. Injects each fact into the reasoning engine's knowledge graph
3. Stores supporting memories in the SQLite database

Usage:
    python scripts/populate_knowledge.py [--data-dir ~/.star]
"""

import json
import sys
import os
import sqlite3
from pathlib import Path

def load_seed_knowledge(path: str) -> list:
    """Load seed knowledge from JSON file."""
    with open(path, 'r', encoding='utf-8') as f:
        return json.load(f)

def get_db(db_path: Path) -> sqlite3.Connection:
    """Open the star.db connection."""
    db_path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    return conn

def inject_into_kg(conn: sqlite3.Connection, entry: dict) -> int:
    """
    Inject a knowledge entry into the knowledge graph.
    
    In Starfire's architecture, the knowledge graph lives in the ReasoningEngine's
    in-memory KG (lib/reasoning/knowledge.rs). This script populates it by:
    1. Adding entities and relationships to the KG through the Rust binary (preferred)
    2. Alternatively, storing as memories in SQLite for later KG sync
    
    Since we can't run the Rust binary directly here, we:
    - Store seed memories in SQLite (they'll be synced to KG at startup)
    - Return the memory ID
    """
    subject = entry['subject']
    fact = entry['fact']
    domain = entry.get('domain', 'empirical')
    confidence = entry.get('confidence', 0.8)
    
    # Map domain names to MemoryDomain enum values
    domain_map = {
        'physics': 'empirical',
        'biology': 'empirical', 
        'mathematics': 'empirical',
        'logic': 'empirical',
        'philosophy': 'empirical',
        'psychology': 'empirical',
        'neuroscience': 'empirical',
        'computer science': 'empirical',
        'AI': 'empirical',
        'machine learning': 'empirical',
        'systems theory': 'empirical',
        'linguistics': 'empirical',
        'epistemology': 'empirical',
    }
    
    db_domain = domain_map.get(domain, 'empirical')
    
    # Store as memory
    now = 1735689600  # Jan 1 2025, approximate - in practice would use actual timestamp
    
    cursor = conn.execute("""
        INSERT INTO memories (content, domain, confidence, importance, formed_at, 
                              access_count, decay_rate, last_accessed, provenance)
        VALUES (?, ?, ?, ?, ?, 0, 0.001, ?, ?)
    """, (
        f"{subject}: {fact}",
        db_domain,
        confidence,
        0.7,  # Default importance
        now,
        now,
        f"seed:{entry.get('subject', 'unknown')}"
    ))
    
    return cursor.lastrowid

def populate_knowledge(db_path: Path, knowledge_path: Path, verbose: bool = True) -> dict:
    """Populate the knowledge base."""
    
    # Load seed knowledge
    if verbose:
        print(f"Loading seed knowledge from {knowledge_path}...")
    entries = load_seed_knowledge(knowledge_path)
    
    conn = get_db(db_path)
    
    # Check if already populated
    existing = conn.execute(
        "SELECT COUNT(*) FROM memories WHERE provenance LIKE 'seed:%'"
    ).fetchone()[0]
    
    if existing > 0:
        if verbose:
            print(f"Knowledge already populated ({existing} entries). Skipping.")
            print("To repopulate, delete existing seed memories first.")
        return {'status': 'already_populated', 'count': existing}
    
    # Inject each entry
    memory_ids = []
    for i, entry in enumerate(entries):
        try:
            mem_id = inject_into_kg(conn, entry)
            memory_ids.append(mem_id)
            if verbose and (i + 1) % 20 == 0:
                print(f"  Injected {i + 1}/{len(entries)}...")
        except Exception as e:
            print(f"Error injecting entry {i}: {e}", file=sys.stderr)
    
    conn.commit()
    conn.close()
    
    result = {
        'status': 'success',
        'total_entries': len(entries),
        'memory_ids_stored': len(memory_ids),
    }
    
    if verbose:
        print(f"Done. Injected {len(memory_ids)} knowledge entries.")
        print(f"Database: {db_path}")
    
    return result

def main():
    import argparse
    
    parser = argparse.ArgumentParser(description='Populate Starfire knowledge base')
    parser.add_argument('--data-dir', default='~/.star', help='Star data directory')
    parser.add_argument('--knowledge-file', default=None, help='Seed knowledge JSON path')
    parser.add_argument('--force', action='store_true', help='Force repopulation')
    args = parser.parse_args()
    
    data_dir = Path(os.path.expanduser(args.data_dir))
    db_path = data_dir / 'star.db'
    
    if args.knowledge_file:
        knowledge_path = Path(args.knowledge_file)
    else:
        # Default: look for seed_knowledge.json relative to this script
        script_dir = Path(__file__).parent
        knowledge_path = script_dir.parent / 'data' / 'seed_knowledge.json'
    
    if not knowledge_path.exists():
        print(f"Error: Knowledge file not found: {knowledge_path}")
        sys.exit(1)
    
    if not db_path.exists():
        print(f"Error: Database not found: {db_path}")
        print("Star must be run at least once to create the database.")
        sys.exit(1)
    
    result = populate_knowledge(db_path, knowledge_path)
    print(json.dumps(result, indent=2))

if __name__ == '__main__':
    main()
