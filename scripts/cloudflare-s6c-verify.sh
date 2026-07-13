#!/usr/bin/env bash

# Temporary independent verifier used only while private-repository GitHub Actions
# compute is unavailable. It never mutates the checked-out branch and always lets
# the UI preview deploy so the complete validation log can be inspected.

set -uo pipefail

ROOT="$(pwd)"
REPORT="$ROOT/ui/public/s6c-validation.txt"
ENCRYPTED="$ROOT/ui/public/s6c-hardened.bin"
PASSPHRASE='qWy2SnrEPOmEHjAmQzV_rftlU0nqGROlMdUGeXhnHNg'
WORK="$(mktemp -d)"
STATUS=0

mkdir -p "$ROOT/ui/public"
: >"$REPORT"
exec > >(tee -a "$REPORT") 2>&1

finish() {
  printf '\nS6C_EXTERNAL_VALIDATION_STATUS=%s\n' "$STATUS"
  printf 'S6C_EXTERNAL_VALIDATION_FINISHED=1\n'
}
trap finish EXIT

printf 'S6-C external validation started\n'
printf 'source_commit=%s\n' "$(git rev-parse HEAD 2>/dev/null || printf unknown)"
printf 'workdir=%s\n' "$WORK"

if ! git ls-files -z | tar --null -cf - -T - | tar -xf - -C "$WORK"; then
  echo 'failed to copy tracked source into disposable checkout'
  STATUS=90
  exit 0
fi

cd "$WORK" || {
  STATUS=91
  exit 0
}

python3 - <<'PY'
from pathlib import Path

workflow = Path('.github/workflows/apply-s6c-review-fixes.yml').read_text()
opener = "          python3 - <<'PY'\n"
start = workflow.index(opener) + len(opener)
end = workflow.index("          PY\n", start)
script = "\n".join(
    line[10:] if line.startswith("          ") else line
    for line in workflow[start:end].splitlines()
) + "\n"
exec(compile(script, 'apply-s6c-review-fixes.py', 'exec'))

path = Path('lib/examples/s6c_limited_runtime_canary.rs')
lines = path.read_text().splitlines()
version_lines = [
    index for index, line in enumerate(lines)
    if line.strip() == 'predictions.version,'
]
if len(version_lines) != 2:
    raise SystemExit(
        f'expected two prediction-version arguments, found {len(version_lines)}'
    )
for version_index in reversed(version_lines):
    enrollment_index = None
    for index in range(version_index - 1, max(-1, version_index - 8), -1):
        if lines[index].strip() == 'let enrollment = planner':
            enrollment_index = index
            break
    if enrollment_index is None:
        raise SystemExit('prediction version is not inside a planner enrollment')
    indent = lines[enrollment_index][
        : len(lines[enrollment_index]) - len(lines[enrollment_index].lstrip())
    ]
    lines.insert(
        enrollment_index,
        f'{indent}let expected_prediction_version = predictions.version;',
    )
    if enrollment_index <= version_index:
        version_index += 1
    arg_indent = lines[version_index][
        : len(lines[version_index]) - len(lines[version_index].lstrip())
    ]
    lines[version_index] = f'{arg_indent}expected_prediction_version,'
path.write_text('\n'.join(lines) + '\n')
PY
PATCH_STATUS=$?
if [ "$PATCH_STATUS" -ne 0 ]; then
  echo "hardening patch failed with status $PATCH_STATUS"
  STATUS=92
  exit 0
fi

echo 'hardening patch applied'

if ! command -v cargo >/dev/null 2>&1; then
  echo 'installing stable Rust toolchain'
  if ! curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --profile minimal --component rustfmt,clippy; then
    echo 'Rust installation failed'
    STATUS=93
    exit 0
  fi
fi

# shellcheck disable=SC1090
[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"

run_gate() {
  local label="$1"
  shift
  printf '\n===== %s =====\n' "$label"
  "$@"
  local gate_status=$?
  printf 'gate_status[%s]=%s\n' "$label" "$gate_status"
  if [ "$gate_status" -ne 0 ] && [ "$STATUS" -eq 0 ]; then
    STATUS="$gate_status"
  fi
  return 0
}

run_gate rustfmt \
  rustfmt --edition 2021 --check \
    lib/companion_runtime_canary.rs \
    lib/examples/s6c_limited_runtime_canary.rs

# Format a second time without --check so an encrypted exact output archive can
# be recovered if the checks pass.
rustfmt --edition 2021 \
  lib/companion_runtime_canary.rs \
  lib/examples/s6c_limited_runtime_canary.rs || STATUS=94

run_gate library-check \
  cargo check -p star --lib --features companion-runtime-canary --locked
run_gate s6c-example-check \
  cargo check -p star --example s6c_limited_runtime_canary \
    --features companion-runtime-canary --locked
run_gate s6c-unit-tests \
  cargo test -p star --lib --features companion-runtime-canary --locked \
    companion_runtime_canary:: -- --test-threads=1
run_gate s6a-regression \
  cargo run -p star --example s6a_bounded_live_policy_probe \
    --features companion-runtime-canary --locked
run_gate s6b-regression \
  cargo run -p star --example s6b_adversarial_live_policy_stress \
    --features companion-runtime-canary --locked
run_gate s6c-corrected-probe \
  cargo run -p star --example s6c_limited_runtime_canary \
    --features companion-runtime-canary --locked

printf '\n===== hardened source digests =====\n'
sha256sum \
  lib/companion_runtime_canary.rs \
  lib/examples/s6c_limited_runtime_canary.rs \
  docs/experiments/S6C_LIMITED_RUNTIME_CANARY_RESULT.md

if command -v openssl >/dev/null 2>&1; then
  tar -czf /tmp/s6c-hardened.tar.gz \
    lib/companion_runtime_canary.rs \
    lib/examples/s6c_limited_runtime_canary.rs \
    docs/experiments/S6C_LIMITED_RUNTIME_CANARY_RESULT.md
  openssl enc -aes-256-cbc -salt -pbkdf2 \
    -pass "pass:$PASSPHRASE" \
    -in /tmp/s6c-hardened.tar.gz \
    -out "$ENCRYPTED"
  echo 'encrypted_hardened_archive=ui/public/s6c-hardened.bin'
else
  echo 'openssl unavailable; no source archive emitted'
fi

exit 0
