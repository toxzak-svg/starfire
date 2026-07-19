#!/usr/bin/env bash
set -euo pipefail

# Terminal ΩG1 verification route. This script tests the exact committed source:
# it performs no formatting, generation, mutation, or source rewriting before
# the compiler, regression, and frozen experimental gates execute.
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export CARGO_INCREMENTAL=0
export RUST_BACKTRACE=1

HEAD_SHA="$(git rev-parse HEAD)"
printf 'OMEGA_G1_EXACT_SOURCE_VALIDATION_START=1\n'
printf 'committed_head=%s\n' "$HEAD_SHA"
printf 'preregistration=d890a55fcaa9f30148835b42325da7456829f807\n'
printf 'source_grammar_sha256='
sha256sum lib/grammar_extension.rs | cut -d' ' -f1
printf 'source_probe_sha256='
sha256sum lib/examples/omega_g1_bounded_grammar_extension.rs | cut -d' ' -f1

if ! command -v cargo >/dev/null 2>&1; then
  printf '\n===== install stable Rust =====\n'
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

printf '\n===== assert pristine committed checkout =====\n'
git diff --exit-code -- \
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
printf 'omega_g1_report_sha256='
sha256sum target/omega-g1-bounded-grammar-extension-report.json | cut -d' ' -f1

printf '\n===== reassert source remained immutable =====\n'
git diff --exit-code -- \
  lib/grammar_extension.rs \
  lib/examples/omega_g1_bounded_grammar_extension.rs

printf '\nOMEGA_G1_EXACT_SOURCE_VALIDATION_STATUS=PASS\n'
printf 'OMEGA_G1_EXACT_SOURCE_VALIDATION_FINISHED=1\n'

cd "$ROOT/ui"
npx next build
