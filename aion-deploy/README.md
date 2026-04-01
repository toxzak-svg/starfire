# Aion + Star — Railway Deployment

## Quick Start

### Option A: Push to GitHub + Railway builds (no local Docker needed)

1. **Copy this entire directory to a new GitHub repo:**
   ```bash
   git init aion-deploy
   cp -r /home/zach/.openclaw/workspace/aion-deploy/* aion-deploy/
   cd aion-deploy
   git add .
   git commit -m "Initial deployment"
   git remote add origin https://github.com/YOUR_USER/aion-deploy.git
   git push -u origin main
   ```

2. **In Railway:** New Project → Deploy from GitHub → select `aion-deploy`

3. **Set environment variables:**
   | Variable | Value |
   |---|---|
   | `STAR_API_URL` | `http://localhost:8080` |
   | `STAR_PORT` | `8080` |
   | `TELEGRAM_BOT_TOKEN` | `8587631426:AAHvqmo--g25LqsClCWCXj4m617cn2v7V7Q` |
   | `RUST_LOG` | `aion_core=info,aion_cli=info` |

4. **Add persistent volume:** `/app/star_data` (mounts the star.db and training.db)

5. **Deploy!**

---

### Option B: Build locally with Docker

```bash
docker build -t yourhub/aion-star:latest .
docker push yourhub/aion-star:latest
```

Then in Railway: New Project → Deploy from Docker Image → `yourhub/aion-star:latest`

---

## What this deploys

- **Star API** — HTTP API at `:8080` — Star's reasoning engine with memory and identity
- **Aion/StarMind** — Polls Telegram every ~3s via `Impulse::Timer`, calls Star's `/chat`, posts responses back to the same chat

## Persistence

Add a persistent volume at `/app/star_data` in Railway so `star.db` survives redeploys.

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `STAR_API_URL` | `http://localhost:8080` | Star API base URL (Aion connects here) |
| `STAR_PORT` | `8080` | Port Star API listens on |
| `TELEGRAM_BOT_TOKEN` | — | **Required.** Telegram bot token from @BotFather |
| `RUST_LOG` | `info` | Log level |
