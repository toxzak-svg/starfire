#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UI_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$UI_DIR/.." && pwd)"
EVIDENCE_DIR="$UI_DIR/public/arise-a1-evidence"
FORMATTED_DIR="$EVIDENCE_DIR/formatted"
A0_TRACE_FILE="$EVIDENCE_DIR/a0-execution-trace.json"
A0_REPLAY_FILE="$EVIDENCE_DIR/a0-execution-trace-replay.json"
A1_TRACE_FILE="$EVIDENCE_DIR/a1-typed-program-trace.json"
A1_REPLAY_FILE="$EVIDENCE_DIR/a1-typed-program-trace-replay.json"
STATUS_FILE="$EVIDENCE_DIR/status.json"
export ARISE_GATE_VERSION="a1-bootstrap-v1"

mkdir -p "$EVIDENCE_DIR" "$FORMATTED_DIR"

write_status() {
  local state="$1"
  local detail="$2"
  node - "$STATUS_FILE" "$state" "$detail" <<'NODE'
const fs = require('fs');
const [path, state, detail] = process.argv.slice(2);
const report = {
  verifier: 'ARISE-A0/A1 Vercel Rust gate',
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
  write_status "failed" "ARISE A0/A1 verification exited with code ${exit_code}. Inspect the Vercel build log."
  exit "$exit_code"
}
trap on_error ERR

write_status "running" "ARISE A0 regression and A1 typed-program verification are executing."

echo "== ARISE A0/A1 Vercel verifier (${ARISE_GATE_VERSION}) =="
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
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/starfire-arise-a1-target}"
cd "$REPO_ROOT"

A0_SOURCES=(
  lib/arise_edge/mod.rs
  lib/arise_edge/types.rs
  lib/arise_edge/engine.rs
  lib/arise_edge/runtime_shadow.rs
  lib/examples/arise_a0_edge_bridge.rs
)
A1_SOURCES=(
  lib/arise_typed_plan_shadow.rs
  lib/arise_response_shadow_ext.rs
  lib/examples/arise_a1_typed_program_shadow.rs
)

echo "== A0 strict source formatting =="
rustfmt --edition 2021 --check "${A0_SOURCES[@]}"

echo "== A1 bootstrap formatting =="
rustfmt --edition 2021 "${A1_SOURCES[@]}"
cp lib/arise_typed_plan_shadow.rs "$FORMATTED_DIR/arise_typed_plan_shadow.rs.txt"
cp lib/arise_response_shadow_ext.rs "$FORMATTED_DIR/arise_response_shadow_ext.rs.txt"
cp lib/examples/arise_a1_typed_program_shadow.rs "$FORMATTED_DIR/arise_a1_typed_program_shadow.rs.txt"

echo "== Default library compile =="
cargo check -p star --lib --locked

echo "== A0 regression compile =="
cargo check -p star --lib --features arise-edge --locked
cargo check -p star --example arise_a0_edge_bridge --features arise-edge --locked

echo "== A1 typed-program compile =="
cargo check -p star --lib --features arise-typed-plan --locked
cargo check -p star --example arise_a1_typed_program_shadow --features arise-typed-plan --locked

echo "== A0 unit contracts =="
cargo test -p star --lib --features arise-typed-plan --locked \
  arise_edge:: -- --test-threads=1

echo "== A1 unit contracts =="
cargo test -p star --lib --features arise-typed-plan --locked \
  arise_typed_plan_shadow:: -- --test-threads=1

echo "== Scoped A0/A1 Clippy =="
CLIPPY_LOG="$EVIDENCE_DIR/clippy.log"
set +e
{
  cargo clippy -p star --lib --features arise-typed-plan --locked --message-format=short
  cargo clippy -p star --example arise_a1_typed_program_shadow \
    --features arise-typed-plan --locked --message-format=short
} 2>&1 | tee "$CLIPPY_LOG"
clippy_status=${PIPESTATUS[0]}
set -e
if [[ $clippy_status -ne 0 ]]; then
  exit "$clippy_status"
fi
if grep -E 'lib/arise_edge/|lib/arise_typed_plan_shadow.rs|lib/arise_response_shadow_ext.rs|lib/examples/arise_a[01]_' "$CLIPPY_LOG" \
  | grep -E 'warning:|error:'; then
  echo "Scoped ARISE Clippy finding detected."
  exit 92
fi

echo "== Deterministic A0 probe =="
cargo build -p star --example arise_a0_edge_bridge --features arise-edge --locked
A0_PROBE_BIN="$CARGO_TARGET_DIR/debug/examples/arise_a0_edge_bridge"
"$A0_PROBE_BIN" | tee "$A0_TRACE_FILE"
"$A0_PROBE_BIN" > "$A0_REPLAY_FILE"
cmp -s "$A0_TRACE_FILE" "$A0_REPLAY_FILE"
grep -F '"terminal_classification": "Pass"' "$A0_TRACE_FILE"
grep -F '"final_residual": 0' "$A0_TRACE_FILE"

echo "== Deterministic A1 typed-program probe =="
cargo build -p star --example arise_a1_typed_program_shadow --features arise-typed-plan --locked
A1_PROBE_BIN="$CARGO_TARGET_DIR/debug/examples/arise_a1_typed_program_shadow"
"$A1_PROBE_BIN" | tee "$A1_TRACE_FILE"
"$A1_PROBE_BIN" > "$A1_REPLAY_FILE"
cmp -s "$A1_TRACE_FILE" "$A1_REPLAY_FILE"
grep -F '"terminal_classification": "Pass"' "$A1_TRACE_FILE"
grep -F '"final_residual": 0' "$A1_TRACE_FILE"

write_status "passed" "A0 regression and A1 formatting, compilation, tests, scoped Clippy, deterministic replay, and executable probes passed."
trap - ERR

echo "ARISE-A0/A1 Vercel verification PASS"
