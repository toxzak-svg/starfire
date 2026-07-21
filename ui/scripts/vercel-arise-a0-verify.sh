#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UI_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$UI_DIR/.." && pwd)"
EVIDENCE_DIR="$UI_DIR/public/arise-a0-evidence"
TRACE_FILE="$EVIDENCE_DIR/execution-trace.json"
STATUS_FILE="$EVIDENCE_DIR/status.json"

mkdir -p "$EVIDENCE_DIR"

write_status() {
  local state="$1"
  local detail="$2"
  node - "$STATUS_FILE" "$state" "$detail" <<'NODE'
const fs = require('fs');
const [path, state, detail] = process.argv.slice(2);
const report = {
  verifier: 'ARISE-A0 Vercel Rust gate',
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
  write_status "failed" "Rust verification exited with code ${exit_code}. Inspect the Vercel build log."
  exit "$exit_code"
}
trap on_error ERR

write_status "running" "Rust verification is executing."

echo "== ARISE-A0 Vercel verifier =="
echo "Repository root: $REPO_ROOT"
echo "Commit: ${VERCEL_GIT_COMMIT_SHA:-unknown}"
echo "Branch: ${VERCEL_GIT_COMMIT_REF:-unknown}"

if [[ ! -f "$REPO_ROOT/Cargo.toml" || ! -f "$REPO_ROOT/Cargo.lock" ]]; then
  echo "Repository root is unavailable from the Vercel UI root directory."
  exit 91
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "Installing the stable Rust toolchain with rustfmt and Clippy..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain stable --component rustfmt,clippy
fi

# shellcheck disable=SC1091
source "$HOME/.cargo/env"
rustc --version
cargo --version

export CARGO_TERM_COLOR=always
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/starfire-arise-a0-target}"
cd "$REPO_ROOT"

echo "== Formatting =="
rustfmt --edition 2021 --check \
  lib/arise_edge/mod.rs \
  lib/arise_edge/types.rs \
  lib/arise_edge/engine.rs \
  lib/arise_edge/runtime_shadow.rs \
  lib/examples/arise_a0_edge_bridge.rs

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

echo "== Executable edge probe =="
cargo run -p star --example arise_a0_edge_bridge --features arise-edge --locked \
  | tee "$TRACE_FILE"
grep -F '"terminal_classification": "Pass"' "$TRACE_FILE"
grep -F '"final_residual": 0' "$TRACE_FILE"

write_status "passed" "Formatting, compilation, tests, scoped Clippy, and executable probe passed."
trap - ERR

echo "ARISE-A0 Vercel verification PASS"
