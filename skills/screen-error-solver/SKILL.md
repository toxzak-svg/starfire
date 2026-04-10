---
name: screen-error-solver
description: >-
  Real-time screen watcher agent that monitors a user's display as they work,
  detects errors (terminal errors, IDE warnings, browser exceptions, build failures,
  stack traces, crash dialogs), and automatically offers solutions. Use when the user
  asks to watch their screen, monitor for errors, build a screen-aware coding assistant,
  create an error detection agent, or set up real-time developer help. Covers screen
  capture, vision-based error detection, OCR, LLM-powered diagnostics, and solution
  delivery via desktop notifications or overlay UI.
metadata:
  author: zwmaronek
  version: '1.0'
  category: developer-tools
---

# Screen Error Solver

A real-time screen-watching agent that monitors a developer's display, detects errors as they appear, and offers actionable solutions — without the user needing to copy-paste anything.

## When to Use This Skill

Use this skill when the user asks to:

- Watch their screen in real-time and help with errors
- Build a screen-aware coding assistant or error detector
- Create an agent that monitors terminal/IDE output for failures
- Set up automatic error detection and solution suggestions
- Build a "copilot that watches over your shoulder"

## Architecture Overview

The system has three layers that run as a continuous loop:

```
┌─────────────────────────────────────────────────────┐
│  LAYER 1: Screen Capture (mss / Wayland / X11)      │
│  Captures screen at configurable interval (1-5s)     │
│  Supports region-of-interest targeting               │
├─────────────────────────────────────────────────────┤
│  LAYER 2: Error Detection Engine                     │
│  Option A: Vision LLM (send screenshot to model)     │
│  Option B: OCR + regex pattern matching              │
│  Option C: Hybrid (OCR first, LLM for ambiguous)     │
├─────────────────────────────────────────────────────┤
│  LAYER 3: Solution Engine + Delivery                 │
│  LLM generates fix based on error context            │
│  Delivers via: desktop notification / overlay /      │
│  terminal / file log / webhook                       │
└─────────────────────────────────────────────────────┘
```

## Instructions

### Step 1: Determine the User's Setup

Ask the user (or infer from context):

1. **Operating system**: Windows, Linux (X11 or Wayland), or macOS
2. **Primary workflow**: Terminal, IDE (VS Code, Cursor, etc.), browser, or mixed
3. **LLM backend preference**: Local (Ollama), OpenAI API, Gemini API, Anthropic API, or any OpenAI-compatible endpoint
4. **Notification preference**: Desktop notifications (default), terminal overlay, log file, or webhook
5. **Capture region**: Full screen, specific monitor, or specific window/region

### Step 2: Install Dependencies

Generate the appropriate setup based on OS:

```bash
# Core dependencies (all platforms)
pip install mss Pillow requests websockets

# For OCR-based detection (optional but recommended as fallback)
pip install pytesseract
# Also install Tesseract engine:
#   Ubuntu/Debian: sudo apt install tesseract-ocr
#   macOS: brew install tesseract
#   Windows: download from https://github.com/UB-Mannheim/tesseract/wiki

# For desktop notifications
pip install plyer  # cross-platform notifications

# For local LLM (Ollama)
# Install from https://ollama.com, then:
# ollama pull llava       # vision model for screenshot analysis
# ollama pull llama3.2    # text model for solution generation

# For vision API (choose one)
pip install openai        # OpenAI GPT-4o / GPT-4-turbo with vision
pip install google-genai  # Gemini 2.0 Flash (free tier available)
pip install anthropic     # Claude with vision
```

### Step 3: Deploy the Screen Watcher

Use the bundled script at `scripts/screen_watcher.py` as the foundation. The script implements the full capture → detect → solve → notify loop.

Key configuration is done via environment variables or a `config.yaml`:

```yaml
# config.yaml
capture:
  interval_seconds: 2        # How often to capture (1-10)
  monitor: 0                 # Which monitor (0 = primary)
  region: null               # null = full screen, or {x, y, width, height}
  downscale_factor: 0.5      # Reduce resolution before sending to LLM (saves tokens)

detection:
  mode: hybrid               # "vision", "ocr", or "hybrid"
  vision_model: gemini-2.0-flash  # or gpt-4o, llava:13b, claude-sonnet
  ocr_confidence_threshold: 60
  error_cooldown_seconds: 30  # Don't re-alert same error within this window

solution:
  model: gemini-2.0-flash    # Model for generating solutions
  max_tokens: 1024
  include_context_lines: 20  # How many lines of surrounding text to include

delivery:
  method: notification        # "notification", "terminal", "file", "webhook"
  webhook_url: null
  log_file: ./error_solutions.log
  sound: true                 # Play sound on error detection

api:
  provider: gemini            # "openai", "gemini", "anthropic", "ollama"
  api_key_env: GEMINI_API_KEY # Environment variable name holding the API key
  ollama_base_url: http://localhost:11434
```

### Step 4: Error Detection Strategies

The detection engine uses multiple strategies in order of efficiency:

#### Strategy A: OCR + Pattern Matching (fastest, cheapest)

Read the `references/error-patterns.md` file for the full pattern library. Core patterns:

- **Stack traces**: `Traceback (most recent call last)`, `at .+\(.+:\d+\)`, `Error:`, `Exception:`
- **Build failures**: `FAILED`, `BUILD FAILED`, `error\[E\d+\]`, `error TS\d+`
- **Terminal errors**: `command not found`, `No such file`, `Permission denied`, `ENOENT`, `EACCES`
- **HTTP errors**: `4\d{2}`, `5\d{2}`, `ERR_`, `CORS`, `fetch failed`
- **Git errors**: `CONFLICT`, `fatal:`, `error: failed to push`
- **Docker errors**: `ERROR`, `exited with code`, `OOMKilled`
- **GPU/CUDA errors**: `CUDA out of memory`, `RuntimeError: CUDA`, `cuDNN error`

When OCR detects a pattern match, extract the full error text and surrounding context before sending to the solution engine.

#### Strategy B: Vision LLM (most accurate, higher cost)

Send the screenshot directly to a vision-capable model with a structured prompt:

```
Analyze this screenshot of a developer's screen. Your task:

1. Is there an error, warning, or failure visible? (yes/no)
2. If yes, identify:
   - Error type (syntax, runtime, build, network, permission, dependency, etc.)
   - The exact error message
   - The file/line if visible
   - The programming language/framework
3. Provide a severity rating (critical/warning/info)

Respond in JSON format:
{
  "has_error": true/false,
  "error_type": "...",
  "error_message": "...",
  "file_location": "...",
  "language": "...",
  "framework": "...",
  "severity": "critical|warning|info",
  "context_summary": "brief description of what the user appears to be working on"
}

If no error is visible, respond with {"has_error": false}.
```

#### Strategy C: Hybrid (recommended)

1. Capture screenshot
2. Run OCR to extract all visible text
3. Run regex patterns against OCR text
4. If pattern match found → send extracted error text to solution LLM (cheap, text-only)
5. If no pattern match but screen changed significantly → send screenshot to vision LLM (catches visual errors like red squiggles, dialog boxes, crash screens)
6. Track frame-to-frame pixel difference to skip analysis when screen is static

### Step 5: Solution Generation

When an error is detected, generate a solution using this prompt template:

```
You are an expert developer assistant. The user is actively coding and hit this error.
Provide a concise, actionable fix.

## Error Context
- Error type: {error_type}
- Error message: {error_message}
- File: {file_location}
- Language/Framework: {language}/{framework}
- What the user appears to be doing: {context_summary}

## Instructions
1. Explain what caused this error in ONE sentence
2. Provide the exact fix (code snippet, command, or config change)
3. If there are multiple possible causes, list the top 2-3 most likely
4. Keep it under 200 words — the user is in flow state, don't interrupt with essays

## Response Format
**Cause:** <one sentence>
**Fix:**
```
<code or command>
```
**If that doesn't work:** <alternative fix>
```

### Step 6: Solution Delivery

Configure delivery based on user preference:

#### Desktop Notification (default, least intrusive)
```python
from plyer import notification
notification.notify(
    title=f"🔴 {error_type}: {short_message}",
    message=solution_text[:256],  # OS limits notification length
    timeout=10
)
```

#### Terminal Overlay
Print a formatted box to a dedicated terminal pane:
```
╔══════════════════════════════════════════╗
║  ❌ ImportError: No module named 'torch'  ║
║──────────────────────────────────────────║
║  Fix: pip install torch                  ║
║  Alt: Check your venv is activated       ║
╚══════════════════════════════════════════╝
```

#### Log File
Append to a structured log for later review:
```json
{
  "timestamp": "2026-04-09T12:30:00",
  "error_type": "ImportError",
  "error_message": "No module named 'torch'",
  "solution": "pip install torch",
  "screenshot_path": "./captures/2026-04-09_12-30-00.png",
  "resolved": false
}
```

#### Webhook
POST to any URL (Slack, Discord, custom dashboard):
```json
POST https://hooks.slack.com/services/...
{
  "text": "🔴 *Error detected*: ImportError: No module named 'torch'\n*Fix:* `pip install torch`"
}
```

### Step 7: Performance Optimization

Key optimizations to keep CPU/memory usage low:

1. **Frame differencing**: Compare current frame hash to previous. Skip analysis if <5% pixels changed. Use `imagehash` or simple pixel-sum comparison.
2. **Region targeting**: Only capture the terminal/IDE pane, not the full screen. Reduces capture size by 60-80%.
3. **Adaptive polling**: Start at 2s interval. When user is idle (no screen changes for 30s), slow to 5s. When errors are detected, temporarily increase to 1s.
4. **Error deduplication**: Hash each error message. Don't re-alert for the same error within the cooldown window.
5. **OCR caching**: If using OCR, only re-OCR regions that changed between frames.
6. **Batch vision calls**: If using vision LLM, queue frames and send only when change is detected.

### Step 8: Running the Agent

```bash
# Start with default config
python screen_watcher.py

# Start with custom config
python screen_watcher.py --config ./config.yaml

# Start watching only a specific region (terminal area)
python screen_watcher.py --region 0,400,1920,680

# Start with local Ollama
python screen_watcher.py --provider ollama --model llava:13b

# Start with Gemini (free tier)
GEMINI_API_KEY=your-key python screen_watcher.py --provider gemini
```

### Step 9: Advanced Features (Optional)

When the user wants to go further:

1. **Auto-fix mode**: Instead of just suggesting, automatically run the fix command (with confirmation). Requires terminal integration via subprocess or tmux.
2. **Context accumulation**: Keep a sliding window of the last N errors + solutions to give the LLM better context about the project.
3. **Project awareness**: Read the project's `package.json`, `requirements.txt`, `Cargo.toml`, etc. to give more accurate solutions.
4. **VS Code extension wrapper**: Package as a VS Code extension that uses the Extension API to read diagnostics directly (no OCR needed for IDE errors).
5. **Multi-monitor support**: Watch all monitors simultaneously with separate detection threads.
6. **Error frequency dashboard**: Local web UI showing error trends, most common errors, time-to-resolution.

## Customization Points

The agent is designed to be modular. Users can swap out:

| Component | Options |
|-----------|---------|
| Screen capture | `mss` (default), `fast-ctypes-screenshots` (Windows), Wayland portal API |
| OCR engine | `pytesseract` (default), `PaddleOCR` (better accuracy), `EasyOCR` (GPU-accelerated) |
| Vision model | GPT-4o, Gemini 2.0 Flash, Claude Sonnet, LLaVA (local), Qwen-VL (local) |
| Solution model | Any text LLM — local or API |
| Notification | `plyer`, `notify-send` (Linux), `osascript` (macOS), `toast` (Windows) |

## Examples

### Example 1: Python developer in terminal

User runs `python train.py`, gets a CUDA out of memory error.

Agent captures screen → OCR extracts `RuntimeError: CUDA out of memory. Tried to allocate 2.00 GiB` → Solution engine generates:

> **Cause:** Your model + batch size exceeds GPU VRAM.
> **Fix:** Reduce batch size: `--batch_size 4` or add `torch.cuda.empty_cache()` before training.
> **If that doesn't work:** Enable gradient checkpointing: `model.gradient_checkpointing_enable()`

### Example 2: JavaScript developer in VS Code

User has a red squiggly under a TypeScript type error.

Agent captures screen → Vision LLM detects red underline and error popup → Identifies `Type 'string' is not assignable to type 'number'` → Solution engine generates:

> **Cause:** Type mismatch — you're passing a string where a number is expected.
> **Fix:** `const value: number = parseInt(inputStr, 10);`
> **If that doesn't work:** Check if the function signature expects `string | number` and update the type.

### Example 3: Docker build failure

User runs `docker build .` and gets a multi-line error.

Agent captures screen → OCR extracts build log → Pattern match on `ERROR` and `exited with code 1` → Solution engine generates:

> **Cause:** The `RUN pip install` step failed because the base image doesn't have pip.
> **Fix:** Change your base image to `python:3.11-slim` or add `RUN apt-get update && apt-get install -y python3-pip` before the install step.
