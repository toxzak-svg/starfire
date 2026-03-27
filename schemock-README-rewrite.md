# Schemock

**Describe your API. Your AI agent gets a live endpoint.**

Tell Schemock what you need in plain English. It generates the schema, boots the server, and exposes everything over MCP. Your AI coding assistant (Claude, Cursor, any MCP client) can explore, create, and mutate the mock directly — no schema writing, no backend, no waiting.

[![GitHub release](https://img.shields.io/github/v/release/toxzak-svg/schemock-app)](https://github.com/toxzak-svg/schemock-app/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-176%20passing-success)](.)
[![Coverage](https://img.shields.io/badge/coverage-76.88%25-green)](.)

---

## TL;DR

```bash
# Install (one command, any OS)
curl -fsSL https://raw.githubusercontent.com/toxzak-svg/schemock-app/main/install.sh | bash

# Describe what you need — server starts live
schemock init "a blog with users, posts, and comments"

# AI coding assistant connects via MCP and starts building
```

No schema writing. No backend. No waiting.

---

## One-liner pitch

**The problem:** You need a backend that doesn't exist yet. You could write a JSON Schema by hand (takes time, easy to get wrong), use MSW (lives in your app code, hard to share), or spin up a full mock server (minutes of setup).

**Schemock:** Tell it what you want. It generates the schema, runs the server, and exposes an MCP interface your AI tools can drive directly. Start in 60 seconds.

---

## Quick Start

### 1. Install

**macOS / Linux:**
```bash
curl -fsSL https://raw.githubusercontent.com/toxzak-svg/schemock-app/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
iwr https://raw.githubusercontent.com/toxzak-svg/schemock-app/main/install.ps1 | iex
```

**Or download a binary directly:**
📥 [Latest release](https://github.com/toxzak-svg/schemock-app/releases)

### 2. Describe your API

```bash
schemock init "an e-commerce API with products, users, and orders"
```

Schemock uses AI to generate a complete JSON Schema, boots the server, and makes it available. Done.

### 3. Connect your AI coding assistant

```json
{
  "mcpServers": {
    "schemock": {
      "command": "schemocker-mcp",
      "env": {
        "SCHEMOCKER_BASE_URL": "http://localhost:3000"
      }
    }
  }
}
```

Now ask your AI: *"Build a product listing page that fetches from my /api/products endpoint"* — it discovers the schema, makes live requests, and generates matching code.

---

## AI-Native by Design

Every layer is built for AI agents:

### `schemock init` — Natural language to live API

```bash
schemock init "a social media API with users, posts, likes, and comments"
```

The primary interface. No schema required. AI generates it, AI can mutate it later.

### MCP tools — AI agents drive the mock

| Tool | What it does |
|------|-------------|
| `generate_schema` | NL description → live schema → server hot-reloads |
| `mutate_schema` | NL instruction → current schema + instruction → merged schema |
| `seed_world` | Populate with N realistic records |
| `list_routes` | AI discovers available endpoints |
| `call_endpoint` | AI makes live requests with optional scenarios |
| `world_snapshot` | AI understands what data exists |

Scenario flags (`slow`, `error-heavy`, `sad-path`) are MCP params — no CLI flags needed.

### Semantic field generation

When a field name contains `bio`, `description`, `title`, `review`, `comment`, or `body` — Schemock uses AI to generate realistic content instead of faker noise. Demo data that actually looks real.

---

## Drop a schema if you want

Not using AI? Drop a JSON Schema in and go.

```bash
schemock start my-schema.json
```

Single binary, hot reload, CORS on, scenario flags — everything works with or without AI.

---

## Scenario testing

Test your UI in different conditions — no config files, just flags:

```bash
schemock init "a payment API" --scenario slow        # 1-3s delays
schemock init "a payment API" --scenario error-heavy  # random 4xx/5xx
schemock init "a payment API" --scenario sad-path     # slow + errors
```

Or via MCP — scenarios are parameters, not flags:

```javascript
// AI agent calls with scenario baked in
call_endpoint({ method: "GET", path: "/api/users", scenario: "slow" })
```

---

## AI Provider Configuration

Defaults to OpenAI `gpt-4o-mini`. Swap providers with env vars:

```bash
# OpenAI (default — no setup needed)
export SCHEMOCKER_API_KEY=sk-...

# Ollama (local, private)
export SCHEMOCKER_AI_PROVIDER=ollama
export SCHEMOCKER_AI_BASE_URL=http://localhost:11434

# vLLM (self-hosted GPU)
export SCHEMOCKER_AI_PROVIDER=vllm
export SCHEMOCKER_AI_BASE_URL=https://your-gpu-endpoint.com
export SCHEMOCKER_API_KEY=your-key
```

---

## vs Alternatives

| | Schemock | Mockoon | MSW | JSON Server |
|--|:--:|:--:|:--:|:--:|
| AI-native (NL → API) | ✅ | ❌ | ❌ | ❌ |
| MCP server built-in | ✅ | ❌ | ❌ | ❌ |
| Binary, zero deps | ✅ | ❌ desktop app | ❌ npm | ❌ Node |
| Semantic AI field gen | ✅ | ❌ | ❌ | ❌ |
| Setup time | **<60s** | 2–5 min | 15+ min | 10+ min |

---

## All commands

```bash
schemock init "description"       # NL → live API (primary)
schemock start [schema]           # Schema file → live API
schemock validate [schema]        # Check schema validity
schemock crud [resource]           # Generate CRUD schema for a resource
schemock --help                   # Full help
```

---

## Install

**macOS / Linux:**
```bash
curl -fsSL https://raw.githubusercontent.com/toxzak-svg/schemock-app/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
iwr https://raw.githubusercontent.com/toxzak-svg/schemock-app/main/install.ps1 | iex
```

**Scoop:**
```powershell
scoop install https://raw.githubusercontent.com/toxzak-svg/schemock-app/main/schemock.json
```

**Download a binary:**
📥 [github.com/toxzak-svg/schemock-app/releases](https://github.com/toxzak-svg/schemock-app/releases)

---

## Docs

| Guide | What it covers |
|-------|---------------|
| [Quick Start](QUICK-START.md) | 5-minute setup walkthrough |
| [User Guide](docs/user-guide.md) | Full feature reference |
| [API Docs](docs/api-documentation.md) | HTTP endpoints and response format |
| [MCP Server](src/mcp-server/README.md) | AI integration setup |
| [Deployment](DEPLOYMENT-GUIDE.md) | CI/CD, production considerations |
| [Troubleshooting](docs/troubleshooting.md) | Common issues and fixes |

---

## Build from source

```bash
git clone https://github.com/toxzak-svg/schemock-app.git
cd schemock-app
npm install
npm run build
npm test
npm run build:exe
```

Prerequisites: Node.js 18+ for development only.

---

## Project stats

- **176 tests** — 100% pass
- **76.88% coverage**
- **~1.5s startup time**
- **~15ms GET response latency**
- **MIT License** — free to use, commercial use allowed

---

<div align="center">

**Stop waiting on backend teams. Start building.**

[⭐ Star on GitHub](https://github.com/toxzak-svg/schemock-app) · [📥 Download](https://github.com/toxzak-svg/schemock-app/releases) · [📖 Docs](docs/)

</div>
