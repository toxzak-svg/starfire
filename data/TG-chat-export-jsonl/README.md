---
license: other
license_name: personal-use-only
tags:
- telegram
- chat
- private
- personal
size_categories:
- 10K<n<100K
---

# personal-tg-clean

Private Telegram chat exports reformatted to JSONL. Each line is one message with chat-level context inlined.

## Source

Exported from 4 Telegram chats on 2026-06-21, then converted from `result.json` (Telegram Desktop export) to one-line-per-message JSONL.

## Files

| file | chat | messages | bytes |
|---|---|---:|---:|
| `data/inky.jsonl`            | Inky            | 10,887 | 18.5 MB |
| `data/el.jsonl`              | 🌀 El 🌀        |  8,718 | 16.0 MB |
| `data/bot_shoppe.jsonl`      | Bot Shoppe      |  5,827 |  7.4 MB |
| `data/marble.jsonl`          | 🌎 Marble 🧠    | 13,775 | 23.3 MB |
| **total**                    |                 | **39,207** | **65.2 MB** |

## Schema

Each line is a JSON object. Original Telegram message fields are preserved verbatim. Three chat-level keys are inlined so every line is self-describing:

| key | type | source |
|---|---|---|
| `_chat_id`   | int    | from `result.json` chat metadata |
| `_chat_name` | string | from `result.json` chat metadata |
| `_chat_type` | string | `private_chat`, `group`, `supergroup`, etc. |

Original Telegram message fields (selection — full set preserved):

| key | type | note |
|---|---|---|
| `id`              | int    | message id within chat |
| `type`            | string | `message` |
| `date`            | string | ISO-8601 UTC |
| `date_unixtime`   | string | unix timestamp (string in export) |
| `from`            | string | sender display name |
| `from_id`         | string | sender id (e.g. `user123...`) |
| `text`            | string \| array | plain string, OR array of strings/entity-dicts (Telegram dual format) |
| `text_entities`   | array  | parallel to `text` when `text` is an array |
| `reply_to_message_id` | int \| null | set if this is a reply |
| `media_type`      | string | set if message has media |
| `file`            | string | set if message has a file |
| ... | | all other fields preserved verbatim |

## Notes

- `text` is **not flattened**. It still carries Telegram's dual format (plain string OR array of mixed strings/entity-dicts). If you want a flattened plain-string `text` field, run a second pass over the JSONL.
- Conversations were exported on 2026-06-21. Anything sent after that is not included.
- This is a **private** dataset. It contains personal Telegram messages. Do not redistribute.

## Build

The converter that produced the JSONL from `result.json` lives at `scripts/tg_export_to_jsonl.py` in the originating repo (`starfire/`).
