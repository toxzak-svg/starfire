import json
from pathlib import Path

BASE = Path(r'C:\Users\Zwmar\.openclaw\workspace\projects\starfire\data\raw\downloads-package\Messages')

with open(BASE / "index.json", 'r', encoding='utf-8') as f:
    data = json.load(f)

# channel_id -> channel_name mapping
channel_info = {}
for cid, name in data.items():
    if isinstance(name, str):
        channel_info[cid] = name

# Get sizes
channel_sizes = []
for cid, name in channel_info.items():
    msg_path = BASE / cid / "messages.json"
    if msg_path.exists():
        size = msg_path.stat().st_size
        channel_sizes.append((cid, name, size))

channel_sizes.sort(key=lambda x: x[2], reverse=True)
print(f"Total channels with messages: {len(channel_sizes)}")
print(f"\nTop channels by size:")
for cid, name, size in channel_sizes[:20]:
    print(f"  {name}: {size/1024:.1f} KB")

# Sample the largest real conversation
print("\n--- Sample from largest channel ---")
for cid, name, size in channel_sizes[:5]:
    if size > 1000:
        msg_path = BASE / cid / "messages.json"
        with open(msg_path, 'r', encoding='utf-8') as f:
            msgs = json.load(f)
        if msgs:
            for m in msgs[:3]:
                author = m.get('author', {}).get('name', 'unknown')
                content = str(m.get('content', ''))[:120]
                print(f"  [{author}]: {content}")
        print()
        break
