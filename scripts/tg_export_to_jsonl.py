"""Flatten Telegram Desktop chat-export JSON to true JSONL.

Input shape (per file):
  {"name": "...", "type": "bot_chat", "id": 123, "messages": [ {msg}, ... ]}

Output: one JSON object per line. Each line is the original message object
with the chat-level metadata inlined as `_chat_*` keys so lines are
self-describing when concatenated across exports.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

SRC_DIR = Path(r"C:\Users\Zwmar\projects\starfire\data\TG-chat-export-jsonl")


def convert(json_path: Path) -> tuple[Path, int]:
    out_path = json_path.with_suffix(".jsonl")
    with json_path.open("r", encoding="utf-8") as f:
        chat = json.load(f)

    chat_id = chat.get("id")
    chat_name = chat.get("name")
    chat_type = chat.get("type")
    messages = chat.get("messages", [])

    n = 0
    with out_path.open("w", encoding="utf-8", newline="\n") as out:
        for msg in messages:
            line = {
                "_chat_id": chat_id,
                "_chat_name": chat_name,
                "_chat_type": chat_type,
                **msg,
            }
            out.write(json.dumps(line, ensure_ascii=False))
            out.write("\n")
            n += 1

    return out_path, n


def main() -> int:
    json_files = sorted(SRC_DIR.rglob("result.json")) + sorted(
        SRC_DIR.glob("*.json")
    )
    # de-dup while preserving order
    seen = set()
    unique = []
    for p in json_files:
        if p not in seen:
            seen.add(p)
            unique.append(p)

    if not unique:
        print("no result.json / *.json files found", file=sys.stderr)
        return 1

    grand_total = 0
    for p in unique:
        out, n = convert(p)
        print(f"{p.name}  ->  {out.name}   ({n:,} messages)")
        grand_total += n

    print(f"\nTotal messages written: {grand_total:,}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
