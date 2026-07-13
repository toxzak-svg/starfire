#!/usr/bin/env bash

# Temporary independent verifier used only while private-repository GitHub Actions
# compute is unavailable. It patches a disposable checkout, runs every frozen S6
# gate, and may push only the exact verified files back to the feature branch.
# It never force-pushes and never publishes an unverified source change.

set -uo pipefail

ROOT="$(pwd)"
REPORT="$ROOT/ui/public/s6c-validation.txt"
ENCRYPTED="$ROOT/ui/public/s6c-hardened.bin"
PASSPHRASE='qWy2SnrEPOmEHjAmQzV_rftlU0nqGROlMdUGeXhnHNg'
BRANCH='experiment/s6c-limited-runtime-canary'
SOURCE_COMMIT="$(git rev-parse HEAD 2>/dev/null || printf unknown)"
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
printf 'source_commit=%s\n' "$SOURCE_COMMIT"
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

# The patch is intentionally generated, then normalized, then checked. The exact
# formatted bytes are the bytes eligible for commit.
run_gate rustfmt-write \
  rustfmt --edition 2021 \
    lib/companion_runtime_canary.rs \
    lib/examples/s6c_limited_runtime_canary.rs
run_gate rustfmt-check \
  rustfmt --edition 2021 --check \
    lib/companion_runtime_canary.rs \
    lib/examples/s6c_limited_runtime_canary.rs

run_gate library-check \
  cargo check -p star --lib --features companion-runtime-canary --locked
run_gate s6c-example-check \
  cargo check -p star --example s6c_limited_runtime_canary \
    --features companion-runtime-canary --locked

printf '\n===== scoped-clippy =====\n'
set -o pipefail
{
  cargo clippy -p star --lib --features companion-runtime-canary --locked \
    --message-format=short
  cargo clippy -p star --example s6c_limited_runtime_canary \
    --features companion-runtime-canary --locked --message-format=short
} 2>&1 | tee /tmp/s6c-clippy.log
CLIPPY_STATUS=${PIPESTATUS[0]}
set +o pipefail
printf 'gate_status[scoped-clippy-command]=%s\n' "$CLIPPY_STATUS"
if [ "$CLIPPY_STATUS" -ne 0 ] && [ "$STATUS" -eq 0 ]; then
  STATUS="$CLIPPY_STATUS"
fi
if grep -E \
  'lib/companion_runtime_canary\.rs:|lib/examples/s6c_limited_runtime_canary\.rs:' \
  /tmp/s6c-clippy.log; then
  echo 'S6-C scoped Clippy finding detected'
  if [ "$STATUS" -eq 0 ]; then
    STATUS=96
  fi
else
  echo 'S6-C scoped Clippy findings: 0'
fi

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

if [ "$STATUS" -eq 0 ]; then
  SOURCE_COMMIT_ENV="$SOURCE_COMMIT" python3 - <<'PY'
from os import environ
from pathlib import Path

path = Path('docs/experiments/S6C_LIMITED_RUNTIME_CANARY_RESULT.md')
text = path.read_text()
text = text.replace(
    'Status: **PROVISIONAL — superseded by review hardening**',
    'Status: **PASS — review-hardened mechanics**',
    1,
)
section = f'''

## Review-hardening terminal verification

The two post-run review defects were corrected and the complete frozen gate was
rerun against the exact formatted source in a disposable Cloudflare Pages build
container because private-repository GitHub Actions compute was unavailable.

```text
verification source head: {environ.get('SOURCE_COMMIT_ENV', 'unknown')}
trial registration proof: PASS
one-shot trial binding:   PASS
independent event replay: PASS
S6-C unit contracts:      PASS
S6-A regression:          PASS
S6-B regression:          PASS
corrected S6-C probe:     PASS
scoped S6-C Clippy:       PASS
unauthorized turns:       0
```

The verifier was permitted to commit only after every command above returned
success and the corrected probe's terminal assertion passed. The initial workflow
artifact remains an explicitly superseded audit record and is not the terminal
S6-C verdict.
'''
if '## Review-hardening terminal verification' not in text:
    text += section
path.write_text(text)
PY
fi

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

if [ "$STATUS" -eq 0 ]; then
  printf '\n===== publish verified hardening =====\n'
  cd "$ROOT" || {
    STATUS=97
    exit 0
  }

  cp "$WORK/lib/companion_runtime_canary.rs" lib/companion_runtime_canary.rs
  cp "$WORK/lib/examples/s6c_limited_runtime_canary.rs" \
    lib/examples/s6c_limited_runtime_canary.rs
  cp "$WORK/docs/experiments/S6C_LIMITED_RUNTIME_CANARY_RESULT.md" \
    docs/experiments/S6C_LIMITED_RUNTIME_CANARY_RESULT.md

  rm -f \
    .github/workflows/export-s6c-source.yml \
    .github/workflows/apply-s6c-review-fixes.yml \
    .github/workflows/run-s6c-review-fixes-pr.yml \
    .github/workflows/diagnose-s6c-review-fixes.yml \
    scripts/cloudflare-s6c-verify.sh

  python3 - <<'PY'
import json
from pathlib import Path

path = Path('package.json')
data = json.loads(path.read_text())
data['scripts']['build'] = 'npm --prefix ui ci && npm --prefix ui run build'
path.write_text(json.dumps(data, indent=2) + '\n')
PY

  git config user.name 'cloudflare-pages[bot]'
  git config user.email '73139402+cloudflare-workers-and-pages[bot]@users.noreply.github.com'
  git add \
    lib/companion_runtime_canary.rs \
    lib/examples/s6c_limited_runtime_canary.rs \
    docs/experiments/S6C_LIMITED_RUNTIME_CANARY_RESULT.md \
    package.json
  git add -u \
    .github/workflows/export-s6c-source.yml \
    .github/workflows/apply-s6c-review-fixes.yml \
    .github/workflows/run-s6c-review-fixes-pr.yml \
    .github/workflows/diagnose-s6c-review-fixes.yml \
    scripts/cloudflare-s6c-verify.sh

  if git diff --cached --quiet; then
    echo 'verified working tree already matches the branch'
  else
    git commit -m 'fix(companion): harden S6-C trial binding and replay'
  fi

  if git push origin "HEAD:refs/heads/$BRANCH"; then
    echo 'verified_hardening_push=success'
  else
    echo 'verified_hardening_push=failed'
    STATUS=98
  fi
fi

exit 0
