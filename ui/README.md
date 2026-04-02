# Star UI

Web chat interface for [Star](https://github.com/toxzak-svg/star) — deployed on Vercel, connects to Star's Railway API.

## Setup

```bash
cd ui
cp .env.local.example .env.local   # already configured for Railway
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000).

## Deploy to Vercel

```bash
npm i -g vercel
vercel
```

Or connect the repo to Vercel — it auto-detects Next.js.

Set `NEXT_PUBLIC_STAR_API` in Vercel env vars to your Railway deployment URL.

## Tech

- Next.js 15 (App Router)
- Tailwind CSS 4
- Lucide icons
- No external UI library — custom terminal-inspired design
