#!/usr/bin/env bash

# Temporary external verifier for ΩG1 while private-repository GitHub Actions
# jobs terminate before executing steps. The verifier runs inside the connected
# Cloudflare Pages build container against the exact committed branch source,
# writes a public validation transcript, and then builds the ordinary UI. It
# does not alter source, push commits, or weaken the frozen experiment gate.

set -uo pipefail

ROOT="$(pwd)"
REPORT="$ROOT/ui/public/omega-g1-validation.txt"
PROBE_LOG="$ROOT/ui/public/omega-g1-probe.log"
OMEGA1_LOG="$ROOT/ui/public/omega1-regression.log"
STATUS=0

mkdir -p "$ROOT/ui/public"
: >"$REPORT"
exec > >(tee -a "$REPORT") 2>&1

finish() {
  printf '\nOMEGA_G1_EXTERNAL_VALIDATION_STATUS=%s\n' "$STATUS"
  printf 'OMEGA_G1_EXTERNAL_VALIDATION_FINISHED=1\n'
}
trap finish EXIT

run_gate() {
  local code="$1"
  shift
  printf '\n===== %s =====\n' "$*"
  "$@"
  local gate_status=$?
  if [ "$gate_status" -ne 0 ]; then
    printf 'gate_failed=%s status=%s\n' "$code" "$gate_status"
    STATUS="$code"
    return 1
  fi
  return 0
}

printf 'ΩG1 external committed-source verification\n'
printf 'head=%s\n' "$(git rev-parse HEAD 2>/dev/null || printf unknown)"
printf 'preregistration=d890a55fcaa9f30148835b42325da7456829f807\n'
printf 'branch=%s\n' "$(git branch --show-current 2>/dev/null || printf detached)"

if ! command -v cargo >/dev/null 2>&1; then
  printf '\n===== install stable Rust =====\n'
  if ! curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --profile minimal --component rustfmt,clippy; then
    echo 'Rust installation failed'
    STATUS=90
  else
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
  fi
fi

if [ "$STATUS" -eq 0 ]; then
  run_gate 10 cargo fmt --all -- --check || true
fi
if [ "$STATUS" -eq 0 ]; then
  run_gate 20 cargo check -p star --all-targets --locked || true
fi
if [ "$STATUS" -eq 0 ]; then
  run_gate 30 cargo test -p star grammar_extension --locked || true
fi
if [ "$STATUS" -eq 0 ]; then
  run_gate 40 cargo test -p star representation_genesis --locked || true
fi
if [ "$STATUS" -eq 0 ]; then
  printf '\n===== Ω1 committed regression =====\n'
  set -o pipefail
  cargo run -p star --example omega1_endogenous_state_space_genesis --locked \
    2>&1 | tee "$OMEGA1_LOG"
  omega1_status=${PIPESTATUS[0]}
  if [ "$omega1_status" -ne 0 ] \
      || ! grep -F '"terminal_classification": "PASS"' "$OMEGA1_LOG" >/dev/null; then
    echo 'Ω1 regression failed or did not emit PASS'
    STATUS=50
  fi
fi
if [ "$STATUS" -eq 0 ]; then
  printf '\n===== ΩG1 frozen probe =====\n'
  set -o pipefail
  cargo run -p star --example omega_g1_bounded_grammar_extension --locked \
    2>&1 | tee "$PROBE_LOG"
  probe_status=${PIPESTATUS[0]}
  if [ "$probe_status" -ne 0 ] \
      || ! grep -F '"terminal_classification": "PASS"' "$PROBE_LOG" >/dev/null \
      || ! test -f target/omega-g1-bounded-grammar-extension-report.json \
      || ! grep -F '"terminal_classification": "PASS"' \
          target/omega-g1-bounded-grammar-extension-report.json >/dev/null; then
    echo 'ΩG1 frozen probe failed or did not emit PASS'
    STATUS=60
  fi
fi

if [ "$STATUS" -eq 0 ]; then
  cp target/omega-g1-bounded-grammar-extension-report.json \
    "$ROOT/ui/public/omega-g1-bounded-grammar-extension-report.json"
  sha256sum \
    lib/grammar_extension.rs \
    lib/examples/omega_g1_bounded_grammar_extension.rs \
    docs/experiments/OMEGAG1_BOUNDED_GRAMMAR_EXTENSION.md \
    target/omega-g1-bounded-grammar-extension-report.json
  echo 'ΩG1 external verification passed every frozen gate'
fi

printf '\n===== ordinary UI build =====\n'
if ! npm --prefix ui ci; then
  echo 'UI dependency installation failed'
  [ "$STATUS" -ne 0 ] || STATUS=80
fi
if ! npm --prefix ui run build; then
  echo 'UI build failed'
  [ "$STATUS" -ne 0 ] || STATUS=81
fi

# Return success so Cloudflare can publish the validation transcript even when
# Rust verification fails. The authoritative verdict is the explicit status in
# the report, never the deployment badge.
exit 0
