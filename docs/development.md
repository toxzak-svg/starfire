# Starfire Development Guide

> **Last reviewed:** 2026-07-21

This guide covers ordinary local work on the Rust workspace and Next.js UI. The production Dockerfile is a release gate and is intentionally much heavier than the normal development loop.

## Prerequisites

- current stable Rust toolchain;
- Git;
- Node.js 20+ for `ui/`;
- Docker only for production-style verification;
- `sqlite3` CLI only when manually inspecting the database.

```bash
rustc --version
cargo --version
node --version
```

## Workspace

```text
starfire/
├── Cargo.toml            # workspace: src + lib
├── src/                  # star_bin executable crate
│   ├── main.rs           # CLI and API startup
│   └── bin/              # ad hoc training, benchmark, and smoke binaries
├── lib/                  # star library crate
│   ├── runtime/
│   ├── persistence/
│   ├── reasoning/
│   ├── prediction/
│   ├── metacog/
│   ├── quanot/
│   ├── neural/
│   ├── language_model/
│   ├── user_model/
│   └── examples/         # executable research probes
├── ui/                   # Next.js web client
├── docs/                 # living docs and evidence records
├── plans/                # research and implementation plans
├── scripts/              # evaluators and integration utilities
├── data/                 # local assets and checkpoints
├── Dockerfile            # gated release image
└── render.yaml           # Render blueprint
```

See [`architecture.md`](architecture.md) for the current subsystem map.

## Run locally

### CLI chat

```bash
cargo run --release -p star_bin --bin star -- chat
```

### Isolated data directory

Place global arguments before the subcommand:

```bash
cargo run --release -p star_bin --bin star -- \
  --data-dir ./data/dev \
  chat
```

### Status

```bash
cargo run --release -p star_bin --bin star -- \
  --data-dir ./data/dev \
  status
```

### HTTP API

```bash
cargo run --release -p star_bin --bin star -- \
  --data-dir ./data/dev \
  api --host 0.0.0.0 --port 8080
```

```bash
curl http://localhost:8080/health
```

The binary starts `star::api::start` directly. `Runtime::chat` is the sole text
producer; an optional F2 observer can run only after final response JSON has been
produced and cannot alter it.

## Web UI

```bash
cd ui
cp .env.local.example .env.local
npm install
npm run dev
```

The example environment file points to the hosted Render API. For a local backend, set:

```text
NEXT_PUBLIC_STAR_API=http://localhost:8080
```

Open `http://localhost:3000`.

## Core test loop

### Fast library check

```bash
cargo check -p star --locked
cargo test -p star --locked
```

### Executable crate

```bash
cargo check -p star_bin --bin star --locked
cargo test -p star_bin --locked
```

### Workspace

```bash
cargo test --workspace --locked
```

### Formatting and linting

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
```

The repository contains old examples and broad feature combinations that may expose pre-existing warnings or defects. When changing a bounded experiment, run the exact frozen command from its document in addition to broad linting.

## Feature-gated experiments

The library defaults to no research features. Enable only the feature needed by the target probe.

Examples:

```bash
cargo run -p star \
  --example omega_v1a_voice_baseline \
  --features omega-v1-baseline \
  --locked
```

```bash
cargo run -p star \
  --example omega_v1f2_shadow_probe \
  --features omega-v1-f2-shadow \
  --locked
```

```bash
cargo run -p star \
  --example r1_relational_residual_bridge \
  --features relational-evidence \
  --locked
```

Do not invent an umbrella feature for convenience if it changes the frozen authority or dependency boundary.

## Production-feature local run

Build the executable with the same top-level feature as the Docker image:

```bash
cargo run --release -p star_bin --bin star \
  --features starfire-observer -- \
  --data-dir ./data/live-dev \
  api --host 0.0.0.0 --port 8080
```

This still starts one API server. The feature only enables the library-owned F2
observer, which remains inactive until its runtime switch is enabled.

## Runtime environment

| Variable | Purpose |
|---|---|
| `STARFIRE_DATA` | Persistent runtime root used by container and newer state files |
| `STARFIRE_HOME` | Fallback runtime state root |
| `STARFIRE_PORT` | Container API port |
| `PORT` | Platform-supplied port fallback |
| `STARFIRE_RUNTIME_VOICE` | Set `0` to disable runtime-owned voice modulation |
| `STARFIRE_OMEGA_V1F2_SHADOW` | Set `1` only for the authorized F2 shadow collection path |
| `OMEGA_V1F2_MODEL_PATH` | Override F2 bounded model artifact location |
| `TELEGRAM_BOT_TOKEN` | Telegram Bot API token |
| `RUST_LOG` | Tracing filter used by `tracing-subscriber` |
| `NEXT_PUBLIC_STAR_API` | API base URL for the UI build |

`STARFIRE_LOG` is set by container configuration for compatibility, but Rust tracing should be debugged with `RUST_LOG` unless code explicitly maps the former.

## Persistent state

For reproducible tests, use a fresh data directory per experiment or test family:

```bash
rm -rf ./data/test-run
mkdir -p ./data/test-run
```

Avoid pointing experiments at a personal or production data directory. Some runtime paths intentionally persist voice, memory, sessions, or traces.

Common files:

```text
<data-root>/
├── IDENTITY.md
├── runtime_voice_profile.json
├── models/
└── *.db
```

Not every file appears in every build.

## SQLite inspection

Locate the database under the selected data root, then inspect it without modifying production state:

```bash
sqlite3 ./data/dev/star.db '.tables'
sqlite3 ./data/dev/star.db 'SELECT * FROM memories LIMIT 5;'
```

Database filenames and effective paths may vary across historical layouts. Check the runtime startup log rather than assuming `~/.star/star.db`.

## Adding runtime behavior

Before wiring a new component into `Runtime::chat`, answer:

1. What input can it read?
2. What state can it mutate?
3. Can it alter returned text?
4. What is the neutral fallback?
5. How is behavior replayed?
6. What kill switch exists?
7. Which test proves the boundary remains closed?
8. Does the change require a preregistered experiment first?

For latent concepts, routing, tools, or persistent belief promotion, assume a new gate is required.

## Adding an API route

1. Add the route in `lib/api.rs`.
2. Preserve CORS behavior or deliberately revise it.
3. Return valid JSON on success and error.
4. Update [`api.md`](api.md).
5. Add route and failure tests.
6. Update UI code only after the wire contract is stable.

The current API sometimes returns application errors with HTTP 200. New routes should prefer accurate status codes, but do not silently change existing clients without a migration.

## Adding a Cargo experiment feature

1. Define the smallest feature in `lib/Cargo.toml`.
2. Express dependencies explicitly.
3. Add `required-features` to examples when appropriate.
4. Keep the default feature set empty unless live promotion is deliberate.
5. Freeze the authority matrix in the preregistration.
6. Run default tests to prove the feature remains absent when disabled.

## Documentation obligations

A merged runtime change should update:

- [`CURRENT_STATUS.md`](CURRENT_STATUS.md);
- [`architecture.md`](architecture.md) when data flow or authority changes;
- [`api.md`](api.md) when the wire contract changes;
- [`deployment.md`](deployment.md) when build or environment behavior changes;
- the relevant preregistration/result record without rewriting prior outcomes.

See the [documentation index](README.md).

## Docker build

```bash
docker build -t starfire .
```

The Dockerfile runs the full release gate and may be slow. A green Cargo unit test is not a substitute for the image gate when changing bundled assets, feature wiring, or release-only evaluators.

Run:

```bash
docker run --rm \
  -p 8080:8080 \
  -v starfire-data:/data \
  starfire
```

## Branch and pull-request practice

- branch from `main`;
- keep experiment identifiers stable;
- include exact verification commands in the pull request;
- distinguish local, external-builder, and deployed evidence;
- do not merge draft experiments merely because infrastructure compiled;
- merge only after required checks and frozen gates are green.

The old `layer4`/Railway branch workflow is historical.

## Debugging

### Trace logs

```bash
RUST_LOG=debug cargo run -p star_bin --bin star -- chat
```

### Test one module

```bash
cargo test -p star response_intent --locked -- --nocapture
```

### Test one feature family

```bash
cargo test -p star \
  --features omega-v1-f2-shadow \
  omega_v1f2_shadow:: \
  --locked \
  -- --test-threads=1 --nocapture
```

### API transport

```bash
curl -v http://localhost:8080/health
curl -v http://localhost:8080/chat \
  -X POST \
  -H 'Content-Type: application/json' \
  -d '{"message":"hello"}'
```

### F2 observer

The optional F2 observer dispatches only after final chat-response JSON has been
produced. Debug the ordinary API at its configured host and port; there is no
separate internal listener.

## Related documents

- [Architecture](architecture.md)
- [API reference](api.md)
- [Deployment](deployment.md)
- [Current status](CURRENT_STATUS.md)
- [Experiment index](experiments/README.md)
