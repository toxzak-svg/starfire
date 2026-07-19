#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export CARGO_INCREMENTAL=0
export RUST_BACKTRACE=1

HEAD_SHA="$(git rev-parse HEAD 2>/dev/null || printf unknown)"
printf 'OMEGA_G1_VERCEL_VALIDATION_START=1\n'
printf 'committed_head=%s\n' "$HEAD_SHA"
printf 'preregistration=d890a55fcaa9f30148835b42325da7456829f807\n'

if ! command -v cargo >/dev/null 2>&1; then
  printf '\n===== install stable Rust =====\n'
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --component rustfmt,clippy
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

printf '\n===== verify committed formatting =====\n'
rustfmt --edition 2021 --check \
  lib/grammar_extension.rs \
  lib/examples/omega_g1_bounded_grammar_extension.rs

printf '\n===== compile all default-feature targets =====\n'
cargo check -p star --all-targets --locked

printf '\n===== run ΩG1 kernel tests =====\n'
cargo test -p star grammar_extension --locked

printf '\n===== preserve Ω1 representation-genesis regression =====\n'
cargo test -p star representation_genesis --locked
cargo run -p star --example omega1_endogenous_state_space_genesis --locked \
  2>&1 | tee /tmp/omega1-regression.log
grep -F '"terminal_classification": "PASS"' /tmp/omega1-regression.log

printf '\n===== run frozen ΩG1 probe =====\n'
cargo run -p star --example omega_g1_bounded_grammar_extension --locked \
  2>&1 | tee /tmp/omega-g1-probe.log
grep -F '"terminal_classification": "PASS"' /tmp/omega-g1-probe.log
test -f target/omega-g1-bounded-grammar-extension-report.json
grep -F '"terminal_classification": "PASS"' \
  target/omega-g1-bounded-grammar-extension-report.json

printf '\nOMEGA_G1_VERCEL_VALIDATION_STATUS=PASS\n'
printf 'OMEGA_G1_VERCEL_VALIDATION_FINISHED=1\n'

# Produce the normal project output without recursively invoking npm run build.
cd "$ROOT/ui"
npx next build
