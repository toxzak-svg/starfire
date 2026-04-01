# Star + Aion on Railway — Deployment Guide

## Prerequisites

- Railway account at [railway.app](https://railway.app)
- GitHub repo with this codebase pushed
- Telegram bot token from @BotFather

---

## One-Time Local Build

Build the binaries on your local machine (fast, no Docker needed):

```bash
# Build Star
cd ~/.openclaw/workspace/life/life
cargo build --release

# Build Aion
cd ~/.openclaw/workspace/aion
cargo build --release
```

---

## Step 1: Push to GitHub

```bash
cd ~/.openclaw/workspace/aion
git init
git add .
git commit -m "Aion + Star Telegram integration"
git remote add origin https://github.com/YOUR_USERNAME/aion.git
git push -u origin main
```

---

## Step 2: Create Railway Project

1. Go to [railway.app](https://railway.app) → **New Project** → **Deploy from GitHub repo**
2. Select your `aion` repo
3. Click **Add a Dockerfile** — Railway will auto-detect the `Dockerfile`

---

## Step 3: Configure Environment Variables

In the Railway dashboard → your service → **Variables**, add:

| Variable | Value |
|---|---|
| `STAR_API_URL` | `http://localhost:8080` |
| `TELEGRAM_BOT_TOKEN` | `8587631426:AAHvqmo--g25LqsClCWCXj4m617cn2v7V7Q` |
| `RUST_LOG` | `aion_core=info,aion_cli=info` |

---

## Step 4: Configure Startup Command

In Railway dashboard → **Settings** → **Start Command**:

```bash
sh -c "star api --port $PORT & echo 'Waiting for Star...' && for i in $(seq 1 30); do curl -sf http://localhost:$PORT/health && break || true; sleep 1; done && aion start-star --api-url http://localhost:$PORT"
```

Or if using the Dockerfile's default `CMD`:

```
sh -c "star api --port $PORT & for i in $(seq 1 30); do curl -sf http://localhost:$PORT/health && break || true; sleep 1; done && aion start-star --api-url http://localhost:$PORT"
```

---

## Step 5: Add Persistent Volume

Star's `star.db` and training data need to persist across deploys:

1. Railway dashboard → **Volumes** → **Add Volume**
2. Name it `star-data`
3. Mount at: `/app/star_data`

Or use the `VOLUME /app/star_data` in the Dockerfile and Railway will provision it automatically.

---

## Step 6: Configure Health Check

- **HTTP Check**: `http://localhost:8080/health`
- **Interval**: 30s
- **Timeout**: 10s

---

## Step 7: Deploy

Click **Deploy** 🚀

Watch the logs — you should see:
```
Starting Star API server at http://0.0.0.0:8080
Star API ready. Starting Aion...
Started StarMind ...
Telegram polling enabled — messages to your bot will reach Star
```

---

## Verify It Works

1. Open Telegram → find your bot
2. Send any message
3. StarMind should respond

---

## Troubleshooting

**Star not starting:**
```bash
# Check if port is already in use
railway logs | grep "already in use"

# Adjust PORT if Railway uses a different one
```

**Telegram not responding:**
```bash
# Verify bot token is correct
railway variables | grep TELEGRAM

# Check Telegram polling logs
railway logs | grep "Telegram"
```

**StarMind not starting:**
```bash
# Verify Star API URL is reachable from Aion
railway logs | grep "Star not responding"

# The Star API must be up before Aion starts
```
