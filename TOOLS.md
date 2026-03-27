# TOOLS.md - Local Notes

Skills define _how_ tools work. This file is for _your_ specifics — the stuff that's unique to your setup.

## What Goes Here

Things like:

- Camera names and locations
- SSH hosts and aliases
- Preferred voices for TTS
- Speaker/room names
- Device nicknames
- Anything environment-specific

## Examples

```markdown
### Cameras

- living-room → Main area, 180° wide angle
- front-door → Entrance, motion-triggered

### SSH

- home-server → 192.168.1.100, user: admin

### TTS

- Preferred voice: "Nova" (warm, slightly British)
- Default speaker: Kitchen HomePod
```

## Why Separate?

Skills are shared. Your setup is yours. Keeping them apart means you can update skills without losing your notes, and share skills without leaking your infrastructure.

---

## Projects

- **attention** → C:\dev\research\attention — research project on attention mechanisms

OpenClaw is configured with an **ollama** provider so you can run a small model locally when you want (e.g. offline, privacy, or uncensored tone).

**Setup:**

1. Install [Ollama](https://ollama.com) and start it.
2. Pull the model: `ollama pull smollm2:1.7b` (~1.8GB).
3. In OpenClaw, the model is available as **ollama/smollm2:1.7b** (alias: `smollm2`). It’s a fallback after MiniMax; switch to it in the dashboard or config if you want it as primary.

**Other small Ollama models in the same ballpark:**

- `tinyllama` — 1.1B, ~638MB
- `smollm2:360m` — 360M, even lighter
- `qwen2:0.5b` — 0.5B if you need the smallest option

SmolLM2 1.7B is a good balance: 8K context, decent for chat and short tasks. For heavy reasoning or long context, keep MiniMax as primary and use the local model for quick or sensitive replies.

---

Add whatever helps you do your job. This is your cheat sheet.
