"""One-shot screen capture + Blackbox vision analysis."""
import base64, mss, sys, os, json

BLACKBOX_API_KEY = os.environ.get("BLACKBOX_API_KEY", "sk-z3eX9Vn4X5i-BLstbx5I2A")

with mss.mss() as sct:
    monitor = sct.monitors[1]
    filename = sct.shot(mon=1, output="marble_scan.png")
    print(f"Captured: {filename}")

with open(filename, "rb") as f:
    img_b64 = base64.b64encode(f.read()).decode()

payload = {
    "model": "blackboxai/anthropic/claude-sonnet-4.6",
    "max_tokens": 512,
    "messages": [{
        "role": "user",
        "content": [
            {"type": "text", "text": "You are a screen error detector. Analyze this screen截图. Look for: error dialogs, exception traces, crash reports, red text, warning icons, stack traces, browser error pages, or anything that needs attention. Respond with ONLY a JSON object: {\"has_error\": true/false, \"error_type\": \"type\", \"error_message\": \"brief description\", \"severity\": \"critical/warning/info\", \"fix\": \"what to do about it\"}"},
            {"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": img_b64}}
        ]
    }]
}

import urllib.request
req = urllib.request.Request(
    "https://api.blackbox.ai/v1/messages",
    data=json.dumps(payload).encode(),
    headers={"Authorization": f"Bearer {BLACKBOX_API_KEY}", "Content-Type": "application/json"},
    method="POST"
)
with urllib.request.urlopen(req, timeout=30) as resp:
    result = json.loads(resp.read())

print(f"Status: {resp.status}")
print(f"Response: {result}")

# Cleanup
os.remove(filename)
