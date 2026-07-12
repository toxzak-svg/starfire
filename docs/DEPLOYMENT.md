# Starfire deployment

Starfire is two deployable components:

1. `src` + `lib`: a Rust API process with SQLite-backed state.
2. `ui`: a browser-only Next.js client that calls the API through `NEXT_PUBLIC_STAR_API`.

They should not be deployed as one Cloudflare Pages project. Pages can host the exported UI, but it cannot run the long-lived Rust process or provide durable local SQLite storage.

## Recommended layout

- **API:** Railway, Render, Fly.io, or another Docker host with a persistent volume mounted at `/data`.
- **UI:** Cloudflare Pages or Vercel.

The repository Dockerfile builds the API. The root `npm run build` command builds a static UI into `ui/out`.

## Cloudflare Pages UI

Use these project settings:

- Root directory: `/`
- Build command: `npm run build`
- Build output directory: `ui/out`
- Environment variable: `NEXT_PUBLIC_STAR_API=https://<your-api-host>`
- Node.js: 22

The same output can be deployed manually with:

```bash
npm install
npm run deploy:cloudflare
```

Cloudflare should be connected only to the UI deployment. The Rust API remains on its container host.

## Railway API

Deploy the repository Dockerfile and configure:

```text
STARFIRE_PORT=8080
STARFIRE_DATA=/data
STARFIRE_LOG=info
```

Attach a persistent volume at `/data`. Expose port `8080`, and use `/health` as the health-check path.

## Hugging Face Spaces

A Docker Space can run the Rust API, but it is not the simplest home for this application. Starfire needs a writable persistent data directory for SQLite continuity, and a sleeping or rebuilt Space can otherwise lose local state. Hugging Face is best used for a public demo after the API has explicit authentication, resource limits, and a persistence plan.

For the current repository, Railway for the API plus Cloudflare Pages for the UI is the least disruptive deployment.
