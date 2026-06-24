#!/usr/bin/env python3
"""Build star_corpus_v1.txt from Telegram chat JSONL exports.

Strategy (per session recommendation):
  - PRIMARY voice source: Marble chat (assistant turns) -- the "sister system"
    with the closest stylistic fit to Star's target voice. Pair with the user
    prompts that triggered them so the corpus preserves conversational rhythm.
  - SECONDARY: a sampled slice of Ton0Fun user turns from the other 3 chats
    (Inky / El / Bot Shoppe), emitted standalone, so Star sees the user's
    tone without confusing the response-style signal.
  - Output format: matches data/personal_training.txt
    * role-prefixed single lines (user: / assistant:)
    * internal newlines escaped as literal \\n
    * blank line separates "conversations" (here = each user/assistant pair)
"""

import json
import random
import re
import sys
from pathlib import Path

DATA_DIR = Path(r"C:\Users\Zwmar\projects\starfire\data\TG-chat-export-jsonl")
OUT_PATH = Path(r"C:\Users\Zwmar\projects\starfire\data\star_corpus_v1.txt")

CHATS = {
    "Inky":       DATA_DIR / "chatexport6-21-2026result.jsonl",
    "El":         DATA_DIR / "ChatExport_2026-06-21" / "result.jsonl",
    "Bot Shoppe": DATA_DIR / "ChatExport_2026-06-21 (1)" / "result.jsonl",
    "Marble":     DATA_DIR / "ChatExport_2026-06-21 (2)" / "result.jsonl",
}

USER_NAME = "Ton0Fun"
PRIMARY_BOT = "Marble"
USER_SAMPLE_FROM_OTHERS = 600
RNG = random.Random(20260623)  # deterministic


def load_jsonl(path: Path):
    n_ok = n_skip = 0
    with path.open("r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                yield json.loads(line)
                n_ok += 1
            except json.JSONDecodeError:
                n_skip += 1
    print(f"  loaded {n_ok} records from {path.name} ({n_skip} skipped)")


def flatten_text(raw) -> str:
    """Telegram JSONL 'text' is either str or list of {type, text, ...}.
    Return a single string."""
    if isinstance(raw, str):
        return raw
    if isinstance(raw, list):
        parts = []
        for seg in raw:
            if isinstance(seg, str):
                parts.append(seg)
            elif isinstance(seg, dict) and "text" in seg:
                parts.append(seg["text"])
        return "".join(parts)
    return ""


def to_role_line(role: str, text: str):
    """Escape internal newlines as literal \\n to match personal_training.txt.
    Returns None for empty / whitespace-only content."""
    if not text or not text.strip():
        return None
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    text = re.sub(r"\n{3,}", "\n\n", text)
    text = text.replace("\n", "\\n")
    return f"{role}: {text}"


def main():
    # ---- Marble: pair user/assistant turns ----
    print(f"Loading {PRIMARY_BOT} chat (primary voice source)...")
    marble_records = list(load_jsonl(CHATS[PRIMARY_BOT]))
    marble_records.sort(key=lambda r: int(r.get("date_unixtime") or 0))

    pairs = []  # (user_text_or_None, assistant_text_or_None)
    pending_user = None
    n_user = n_assistant = 0

    for rec in marble_records:
        frm = rec.get("from", "")
        text = flatten_text(rec.get("text", ""))
        if not text.strip():
            continue
        if frm == USER_NAME:
            n_user += 1
            if pending_user is not None:
                # back-to-back user messages: flush previous as standalone
                pairs.append((pending_user, None))
            pending_user = text
        else:
            n_assistant += 1
            if pending_user is not None:
                pairs.append((pending_user, text))
                pending_user = None
            else:
                pairs.append((None, text))
    if pending_user is not None:
        pairs.append((pending_user, None))

    print(f"  Marble: {n_user} user / {n_assistant} assistant -> {len(pairs)} paired segments")

    # ---- Other chats: sample user turns ----
    print(f"\nLoading user turns from non-primary chats (sampling {USER_SAMPLE_FROM_OTHERS})...")
    other_user_turns = []
    for name, path in CHATS.items():
        if name == PRIMARY_BOT:
            continue
        chat_user = 0
        for rec in load_jsonl(path):
            if rec.get("from") == USER_NAME:
                text = flatten_text(rec.get("text", ""))
                if text.strip():
                    other_user_turns.append((name, text))
                    chat_user += 1
        print(f"    {name}: {chat_user} user turns")
    print(f"  Total available: {len(other_user_turns)}")
    sampled = RNG.sample(other_user_turns, min(USER_SAMPLE_FROM_OTHERS, len(other_user_turns)))

    # ---- Write corpus ----
    print(f"\nWriting {OUT_PATH}...")
    n_user_lines = n_assistant_lines = n_blocks = 0
    with OUT_PATH.open("w", encoding="utf-8") as f:
        f.write("# star_corpus_v1.txt  generated 2026-06-23\n")
        f.write("# source: Telegram bot chat exports (4 chats: Inky, El, Bot Shoppe, Marble)\n")
        f.write("# composition:\n")
        f.write(f"#   - PRIMARY: {PRIMARY_BOT} chat full conversation pairs "
                f"({n_assistant} assistant + {n_user} user -> {len(pairs)} segments)\n")
        f.write(f"#   - SECONDARY: {len(sampled)} sampled user turns from Inky/El/Bot Shoppe\n")
        f.write("# format: matches data/personal_training.txt\n")
        f.write("#   - one turn per line, role-prefixed: user: / assistant:\n")
        f.write("#   - internal newlines in turn content escaped as \\n\n")

        for u, a in pairs:
            ul = to_role_line("user", u) if u else None
            al = to_role_line("assistant", a) if a else None
            if ul:
                f.write(ul + "\n")
                n_user_lines += 1
            if al:
                f.write(al + "\n")
                n_assistant_lines += 1
            f.write("\n")
            n_blocks += 1

        for _chat, text in sampled:
            ul = to_role_line("user", text)
            if ul:
                f.write(ul + "\n\n")
                n_user_lines += 1

    size = OUT_PATH.stat().st_size
    n_lines = sum(1 for _ in OUT_PATH.open("r", encoding="utf-8"))
    print(f"\nOK  {OUT_PATH}")
    print(f"    size:  {size:,} bytes ({size / 1024 / 1024:.2f} MB)")
    print(f"    lines: {n_lines:,}")
    print(f"    blocks (Marble pairs + sampled-user-only): {n_blocks:,}")
    print(f"    user turns emitted: {n_user_lines:,}")
    print(f"    assistant turns emitted: {n_assistant_lines:,}")


if __name__ == "__main__":
    main()