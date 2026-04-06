import json
from pathlib import Path

DISCORD_BASE = Path(r'C:\Users\Zwmar\.openclaw\workspace\projects\starfire\data\raw\downloads-package\Messages')

with open(DISCORD_BASE / "index.json", 'r', encoding='utf-8') as f:
    channel_map = json.load(f)

# Get top 3 channels
channels = []
for cid, name in channel_map.items():
    if isinstance(name, str):
        msg_path = DISCORD_BASE / cid / "messages.json"
        if msg_path.exists():
            size = msg_path.stat().st_size
            channels.append((cid, name, size, msg_path))

channels.sort(key=lambda x: x[2], reverse=True)

for cid, name, size, msg_path in channels[:3]:
    print(f"\n=== {name} ({size/1024:.1f} KB) ===")
    with open(msg_path, 'r', encoding='utf-8') as f:
        msgs = json.load(f)
    
    print(f"Total messages: {len(msgs)}")
    
    # Show first 5 messages
    for m in msgs[:5]:
        author = m.get('author', {}).get('name', 'unknown')
        content = str(m.get('content', ''))[:100]
        msg_type = m.get('type', 'message')
        print(f"  [{author}] ({msg_type}): {content}")
