# Starfire Web UI

The `ui/` directory contains the browser client for Starfire. It is a Next.js 16 / React 19 application that calls the Rust API through `NEXT_PUBLIC_STAR_API`.

The UI does not run the cognitive architecture itself. Identity, memory, reasoning, metacognition, response planning, and persistence remain in the Rust service.

## Features

- chat interface;
- health and connection state;
- optional live turn, intent, pipeline, and trace labels;
- explicit legacy/failure-open labeling when live metadata is unavailable;
- identity drawer;
- cognitive-state drawer;
- metacognition and curiosity views;
- memory statistics;
- normalized API host configuration.

## Local setup

```bash
cd ui
cp .env.local.example .env.local
npm install
npm run dev
```

Open `http://localhost:3000`.

The example environment file points to the hosted research API:

```text
NEXT_PUBLIC_STAR_API=https://starfire-cuee.onrender.com
```

For a local backend, replace it with:

```text
NEXT_PUBLIC_STAR_API=http://localhost:8080
```

Start the backend from the repository root:

```bash
cargo run --release -p star_bin --bin star -- \
  --data-dir ./data/dev \
  api --host 0.0.0.0 --port 8080
```

## Build

```bash
npm run build
npm run start
```

## Deploy to Vercel

Use these project settings:

| Setting | Value |
|---|---|
| Root directory | `ui` |
| Framework | Next.js |
| Install command | `npm install` |
| Build command | `npm run build` |
| Environment variable | `NEXT_PUBLIC_STAR_API=https://starfire-cuee.onrender.com` |

The API client accepts a scheme-less host, adds `https://`, and removes trailing slashes.

## Chat contract

The client sends:

```json
{
  "message": "hello",
  "history": []
}
```

The current protected Rust handler uses `message` and ignores the unknown `history` field. Runtime persistence and session state remain authoritative.

The stable response field is:

```json
{
  "response": "..."
}
```

A production response may also contain:

```json
{
  "live": {
    "enabled": true,
    "pipeline": "live-integration-1",
    "trace_id": "...",
    "turn": 14,
    "intent": "Reflection"
  }
}
```

UI components must treat `live` as optional and avoid claiming that a fallback response passed through the live plan path.

## Technology

- Next.js 16 App Router;
- React 19;
- Tailwind CSS 4;
- Lucide React icons;
- custom terminal-inspired component styling;
- no required external component library.

## Security

The hosted API has no built-in authentication or per-user isolation. Do not deploy a private-memory UI against a public API without adding access control to the backend and inspection routes.

## Related docs

- [Project README](../README.md)
- [API reference](../docs/api.md)
- [Deployment guide](../docs/deployment.md)
- [Current status](../docs/CURRENT_STATUS.md)
