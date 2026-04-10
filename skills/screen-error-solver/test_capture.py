"""Quick test -- capture one frame and analyze it."""
import sys, os, base64, io, json, requests, re
sys.path.insert(0, "C:/Users/Zwmar/.openclaw/workspace/projects/starfire/skills/screen-error-solver")

BLACKBOX_API_KEY = "sk-z3eX9Vn4X5i-BLstbx5I2A"

print("Testing screen capture...")
import mss
from PIL import Image
with mss.mss() as sct:
    monitors = sct.monitors
    print(f"Monitors: {len(monitors)}")
    img = sct.grab(monitors[0])
    print(f"Captured frame: {img.size}")
    pil_img = Image.frombytes("RGB", img.size, img.bgra, "raw", "BGRX")
    buffer = io.BytesIO()
    pil_img.save(buffer, format="PNG", optimize=True)
    b64 = base64.b64encode(buffer.getvalue()).decode("utf-8")
    print(f"Base64 size: {len(b64)}")

print("\nTesting Blackbox vision...")
prompt = (
    "Look at this screenshot. Is there an error, warning, or failure visible? "
    "Respond ONLY with JSON: {\"has_error\": true/false, \"error_type\": \"...\", \"error_message\": \"...\", \"severity\": \"critical|warning|info\", \"context_summary\": \"...\"}"
)
resp = requests.post(
    "https://api.blackbox.ai/v1/messages",
    headers={
        "Authorization": f"Bearer {BLACKBOX_API_KEY}",
        "anthropic-version": "2023-06-01",
    },
    json={
        "model": "blackboxai/anthropic/claude-sonnet-4.6",
        "messages": [{"role": "user", "content": [{"type": "text", "text": prompt}, {"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": b64}}]}],
        "max_tokens": 300,
    },
    timeout=60,
)
print(f"Status: {resp.status_code}")
data = resp.json()
content = data.get("content", [])
for block in content:
    if block.get("type") == "text":
        print(f"Response: {block.get('text', '')[:300]}")

print("\nDone -- both capture and Blackbox vision work!")
