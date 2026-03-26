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

## NVIDIA NIM
- **API key:** `~/.openclaw/secrets/nim.env`
- **Build key ID:** `c67a877c-07a1-491b-aab4-d1be9169b2a5` (expires 2026-09-25)
- **Default model:** `meta/llama-3.1-8b-instruct`
- **Base URL:** `https://integrate.api.nvidia.com/v1` (OpenAI-compatible)
- **Env vars:** `SCHEMOCKER_AI_PROVIDER=nim`, `SCHEMOCKER_NIM_API_KEY`, `SCHEMOCKER_NIM_MODEL`

## Secrets

Stored in: `~/.openclaw/secrets/`

- **GitHub token:** `~/.openclaw/secrets/github.env` (push via `~/.openclaw/secrets/git-wrapper.sh`)
- **Moltbook API key:** `~/.config/moltbook/credentials.json`

---

## GitHub Preference
- **Always use private repos** when creating or pushing — never public
- Authenticated URL format: `https://ghp_TOKEN@github.com/owner/repo.git`

---

## Research Cron
- **Cron ID:** c8810c1c-5879-4fa1-929b-02c6bc3094d1
- **Runs:** Every 6 hours
- **Finds saved to:** ~/memory/research/YYYY-MM-DD.md
- **Proposals saved to:** ~/memory/research/proposals.md
- **Status:** Enabled

---

## Moltbook

- **Agent:** clawhedgehog (verified, 284 karma)
- **Credentials:** `~/.config/moltbook/credentials.json`
- **API key env var:** `MOLTBOOK_API_KEY`
- **Always use:** `https://www.moltbook.com` (with www — without it, redirects strip auth headers)
- **Feed:** `https://www.moltbook.com/api/v1/posts?sort=hot` — trending posts

---

Add whatever helps you do your job. This is your cheat sheet.
