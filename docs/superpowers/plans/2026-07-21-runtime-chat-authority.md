# Runtime Chat Authority Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the HTTP response wrapper so `Runtime::chat` is the only production text authority while retaining the F2 observer as an optional library-owned shadow path.

**Architecture:** The binary always starts `star::api::start` directly. The renamed binary feature enables only the library F2 observer already dispatched after a finalized `/chat` response; it starts no second HTTP server, maintains no duplicate voice state, and transforms no response. The obsolete wrapper implementation and public `/live/status` API disappear.

**Tech Stack:** Rust 2021, Cargo features, tiny_http library API, Markdown documentation.

## Global Constraints

- `Runtime::chat` is the sole response-text producer.
- The F2 observer stays optional, post-response, shadow-only, and cannot alter response bytes.
- No legacy HTTP proxy, duplicate `VoiceState`, loopback forwarding, `/live/status`, or `live_*` state files remain in the production binary.
- Preserve the existing `omega-v1-f2-shadow` library feature and its frozen test/probe gates.
- Update living status and API documentation in the same change.

---

### Task 1: Remove the legacy wrapper from the binary

**Files:**
- Modify: `src/main.rs`
- Modify: `src/Cargo.toml`
- Delete: `src/live_api.rs`
- Modify: `Dockerfile`

**Interfaces:**
- Consumes: `star::api::start(Arc<Mutex<Runtime>>, &str, u16) -> anyhow::Result<()>`.
- Produces: `star` binary whose `Api` command directly calls `api::start` with or without the optional observer feature.

- [x] **Step 1: Verify the wrapper is still the feature-gated API route**

Run: `cargo check -p star_bin --features starfire-live --locked`

Expected: PASS before the change, establishing the legacy build baseline.

Completed after the implementation snapshot: the legacy `starfire-live` feature is intentionally absent, so its historical pre-change baseline cannot be rerun.

- [x] **Step 2: Remove the failing authority boundary**

Change `src/main.rs` so `use star::api;` is unconditional and the `Api` command calls `api::start(rt, &host, port)?` directly. Delete the `live_api` module and its cfg branches. Replace the `starfire-live` binary feature with `starfire-observer = ["star/omega-v1-f2-shadow"]`, remove wrapper-only dependencies, and build the Docker image binary with `starfire-observer`.

- [x] **Step 3: Verify direct API builds with and without the observer**

Run: `cargo check -p star_bin --locked` and `cargo check -p star_bin --features starfire-observer --locked`

Expected: both PASS; neither build compiles or starts a response-transforming proxy.

Verified 2026-07-22: both commands passed.

### Task 2: Reconcile living documentation

**Files:**
- Modify: `docs/CURRENT_STATUS.md`
- Modify: `docs/api.md`
- Modify: `docs/architecture.md`
- Modify: `docs/development.md`

**Interfaces:**
- Consumes: the direct binary API route from Task 1 and the library-owned F2 post-response observer.
- Produces: current documentation that names a single response authority and omits the removed legacy endpoint.

- [x] **Step 1: Replace the dual-path status description**

State that `Runtime::chat` is the sole text authority, the binary starts the library API directly, and F2 observes finalized responses only when explicitly enabled.

- [x] **Step 2: Remove wrapper-only API claims**

Remove `/live/status`, the `live` envelope example, and any advice that treats the wrapper endpoint as a public API surface. Update the living architecture and development guides so they no longer name `starfire-live`, `src/live_api.rs`, wrapper-owned `VoiceState`, or loopback forwarding as current behavior.

- [x] **Step 3: Run focused regression checks**

Run: `cargo test -p star --lib --features omega-v1-f2-shadow --locked omega_v1f2_shadow:: -- --test-threads=1`, `cargo check -p star_bin --features starfire-observer --locked`, and `git diff --check`.

Expected: all commands PASS with no whitespace errors.

Verified 2026-07-22: F2 shadow tests and `git diff --check` passed. A direct local smoke also returned `{"status":"ok"}` from `GET /health` and a normal JSON response from `POST /chat` on the direct API server.
