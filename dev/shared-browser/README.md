# Shared Browser

A browser we can both see and control!

## Setup

```bash
# Install dependencies
pip install -r requirements.txt

# Install Playwright browsers
playwright install chromium
```

## Run

```bash
python server.py
```

Then open http://localhost:5000

## What It Does

- 🌐 Browser embedded in webpage
- 📍 Navigate to any URL
- ⬆️⬇️ Scroll
- ⬅️➡️ Back/Forward
- 📸 Screenshot

## How We Use It

1. You open http://localhost:5000
2. I send commands via API to control the browser
3. We both see the same browser!

## Next Steps

- Add screenshot sharing so I can see
- Add click coordination
- Add form filling
- Add WebSocket for real-time sync
