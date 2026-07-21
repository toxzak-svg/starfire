#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UI_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$UI_DIR/.." && pwd)"
EVIDENCE_DIR="$UI_DIR/public/arise-a0-evidence"
FORMATTED_DIR="$EVIDENCE_DIR/formatted"
TRACE_FILE="$EVIDENCE_DIR/execution-trace.json"
REPLAY_FILE="$EVIDENCE_DIR/execution-trace-replay.json"
STATUS_FILE="$EVIDENCE_DIR/status.json"
export ARISE_GATE_VERSION="strict-v2"

mkdir -p "$EVIDENCE_DIR" "$FORMATTED_DIR"

write_status() {
  local state="$1"
  local detail="$2"
  node - "$STATUS_FILE" "$state" "$detail" <<'NODE'
const fs = require('fs');
const [path, state, detail] = process.argv.slice(2);
const report = {
  verifier: 'ARISE-A0 Vercel Rust gate',
  gateVersion: process.env.ARISE_GATE_VERSION || 'unknown',
  state,
  detail,
  commit: process.env.VERCEL_GIT_COMMIT_SHA || process.env.GITHUB_SHA || 'unknown',
  branch: process.env.VERCEL_GIT_COMMIT_REF || 'unknown',
  environment: process.env.VERCEL_ENV || 'unknown',
  generatedAt: new Date().toISOString(),
};
fs.writeFileSync(path, JSON.stringify(report, null, 2) + '\n');
NODE
}

on_error() {
  local exit_code=$?
  write_status "failed" "Strict Rust verification exited with code ${exit_code}. Inspect the Vercel build log."
  exit "$exit_code"
}
trap on_error ERR

write_status "running" "Strict Rust verification is executing."

echo "== ARISE-A0 strict Vercel verifier (${ARISE_GATE_VERSION}) =="
echo "Repository root: $REPO_ROOT"
echo "Commit: ${VERCEL_GIT_COMMIT_SHA:-unknown}"
echo "Branch: ${VERCEL_GIT_COMMIT_REF:-unknown}"

if [[ ! -f "$REPO_ROOT/Cargo.toml" || ! -f "$REPO_ROOT/Cargo.lock" ]]; then
  echo "Repository root is unavailable from the Vercel UI root directory."
  exit 91
fi

export PATH="$HOME/.cargo/bin:$PATH"

needs_rustup=0
command -v cargo >/dev/null 2>&1 || needs_rustup=1
command -v rustfmt >/dev/null 2>&1 || needs_rustup=1
cargo clippy --version >/dev/null 2>&1 || needs_rustup=1

if [[ $needs_rustup -eq 1 ]]; then
  echo "Installing the stable Rust toolchain with rustfmt and Clippy..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain stable --component rustfmt,clippy --no-modify-path
  export PATH="$HOME/.cargo/bin:$PATH"
fi

if [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

rustc --version
cargo --version
rustfmt --version
cargo clippy --version

export CARGO_TERM_COLOR=always
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/starfire-arise-a0-target}"
cd "$REPO_ROOT"

ARISE_SOURCES=(
  lib/arise_edge/mod.rs
  lib/arise_edge/types.rs
  lib/arise_edge/engine.rs
  lib/arise_edge/runtime_shadow.rs
  lib/examples/arise_a0_edge_bridge.rs
)

echo "== Strict source formatting =="
rustfmt --edition 2021 --check "${ARISE_SOURCES[@]}"

cp lib/arise_edge/mod.rs "$FORMATTED_DIR/mod.rs.txt"
cp lib/arise_edge/types.rs "$FORMATTED_DIR/types.rs.txt"
cp lib/arise_edge/engine.rs "$FORMATTED_DIR/engine.rs.txt"
cp lib/arise_edge/runtime_shadow.rs "$FORMATTED_DIR/runtime_shadow.rs.txt"
cp lib/examples/arise_a0_edge_bridge.rs "$FORMATTED_DIR/arise_a0_edge_bridge.rs.txt"

echo "== Default library compile =="
cargo check -p star --lib --locked

echo "== ARISE feature compile =="
cargo check -p star --lib --features arise-edge --locked
cargo check -p star --example arise_a0_edge_bridge --features arise-edge --locked

echo "== ARISE unit contracts =="
cargo test -p star --lib --features arise-edge --locked \
  arise_edge:: -- --test-threads=1

echo "== Scoped Clippy =="
CLIPPY_LOG="$EVIDENCE_DIR/clippy.log"
set +e
cargo clippy -p star --lib --features arise-edge --locked --message-format=short \
  2>&1 | tee "$CLIPPY_LOG"
clippy_status=${PIPESTATUS[0]}
set -e
if [[ $clippy_status -ne 0 ]]; then
  exit "$clippy_status"
fi
if grep -E 'lib/arise_edge/|lib/examples/arise_a0_edge_bridge' "$CLIPPY_LOG" \
  | grep -E 'warning:|error:'; then
  echo "Scoped ARISE Clippy finding detected."
  exit 92
fi

echo "== Deterministic executable edge probe =="
cargo build -p star --example arise_a0_edge_bridge --features arise-edge --locked
PROBE_BIN="$CARGO_TARGET_DIR/debug/examples/arise_a0_edge_bridge"
"$PROBE_BIN" | tee "$TRACE_FILE"
"$PROBE_BIN" > "$REPLAY_FILE"
cmp -s "$TRACE_FILE" "$REPLAY_FILE"
grep -F '"terminal_classification": "Pass"' "$TRACE_FILE"
grep -F '"final_residual": 0' "$TRACE_FILE"

write_status "passed" "Strict Rustfmt, compilation, tests, scoped Clippy, deterministic replay, and executable probe passed."
trap - ERR

echo "ARISE-A0 strict Vercel verification PASS"
