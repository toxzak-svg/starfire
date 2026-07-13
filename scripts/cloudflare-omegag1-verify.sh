#!/usr/bin/env bash

# Temporary self-recording external verifier for ΩG1 while private-repository
# GitHub Actions jobs terminate before executing steps. The connected Cloudflare
# Pages build container compiles the exact committed branch source, runs every
# frozen gate, commits the transcript and machine report, restores the ordinary
# UI build, removes this script, and pushes the evidence back to the PR branch.
# It never edits the mechanism or preregistration after observing a result.

set -uo pipefail

ROOT="$(pwd)"
BRANCH="research/omega-g1-bounded-grammar-extension"
SOURCE_HEAD="$(git rev-parse HEAD 2>/dev/null || printf unknown)"
REPORT="$ROOT/ui/public/omega-g1-validation.txt"
PROBE_LOG="$ROOT/ui/public/omega-g1-probe.log"
OMEGA1_LOG="$ROOT/ui/public/omega1-regression.log"
TRACKED_REPORT="$ROOT/docs/experiments/OMEGAG1_EXTERNAL_VALIDATION.txt"
TRACKED_JSON="$ROOT/docs/experiments/OMEGAG1_BOUNDED_GRAMMAR_EXTENSION_REPORT.json"
RESULT_DOC="$ROOT/docs/experiments/OMEGAG1_BOUNDED_GRAMMAR_EXTENSION_RESULT.md"
STATUS=0

mkdir -p "$ROOT/ui/public"
: >"$REPORT"
exec 3>&1 4>&2
exec > >(tee -a "$REPORT") 2>&1

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
printf 'head=%s\n' "$SOURCE_HEAD"
printf 'preregistration=d890a55fcaa9f30148835b42325da7456829f807\n'
printf 'branch=%s\n' "$BRANCH"

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

printf '\nOMEGA_G1_EXTERNAL_VALIDATION_STATUS=%s\n' "$STATUS"
printf 'OMEGA_G1_EXTERNAL_VALIDATION_FINISHED=1\n'

# Restore the original descriptors so the process-substitution tee is closed and
# fully flushed before the transcript is copied into the tracked evidence path.
exec 1>&3 2>&4
exec 3>&- 4>&-
wait || true
cp "$REPORT" "$TRACKED_REPORT"

if [ "$STATUS" -eq 0 ]; then
  cp target/omega-g1-bounded-grammar-extension-report.json "$TRACKED_JSON"
else
  rm -f "$TRACKED_JSON"
fi

SOURCE_HEAD_ENV="$SOURCE_HEAD" STATUS_ENV="$STATUS" python3 - <<'PY'
from hashlib import sha256
from os import environ
from pathlib import Path

result_path = Path('docs/experiments/OMEGAG1_BOUNDED_GRAMMAR_EXTENSION_RESULT.md')
text = result_path.read_text()
source_head = environ['SOURCE_HEAD_ENV']
status = int(environ['STATUS_ENV'])
transcript = Path('docs/experiments/OMEGAG1_EXTERNAL_VALIDATION.txt')
transcript_digest = sha256(transcript.read_bytes()).hexdigest()

if status == 0:
    report = Path('docs/experiments/OMEGAG1_BOUNDED_GRAMMAR_EXTENSION_REPORT.json')
    report_digest = sha256(report.read_bytes()).hexdigest()
    replacement = f'''Status: **PASS — EXTERNALLY VERIFIED ON COMMITTED SOURCE**.

## Terminal external verification

```text
verified source head: {source_head}
preregistration:      d890a55fcaa9f30148835b42325da7456829f807
external status:      0
transcript sha256:    {transcript_digest}
report sha256:        {report_digest}
terminal class:       PASS
```
'''
else:
    report_digest = 'not produced'
    replacement = f'''Status: **FAILED EXTERNAL VERIFICATION — NO SCIENTIFIC PASS**.

## Terminal external verification

```text
verified source head: {source_head}
preregistration:      d890a55fcaa9f30148835b42325da7456829f807
external status:      {status}
transcript sha256:    {transcript_digest}
report sha256:        {report_digest}
terminal class:       NOT PASS
```
'''

old = 'Status: **PENDING — no scientific verdict has been produced**.\n'
if old not in text:
    raise SystemExit('pending result marker is missing; refusing to rewrite result')
text = text.replace(old, replacement, 1)
text = text.replace(
    '''workflow run: pending
committed source head: pending
terminal classification: pending
artifact id: pending
artifact digest: pending''',
    f'''workflow run: Cloudflare connected build container
committed source head: {source_head}
terminal classification: {'PASS' if status == 0 else 'NOT PASS'}
artifact id: tracked transcript and machine report
artifact digest: {report_digest}''',
    1,
)
result_path.write_text(text)
PY
rewrite_status=$?
if [ "$rewrite_status" -ne 0 ] && [ "$STATUS" -eq 0 ]; then
  STATUS=91
fi

# Restore the normal deployment route and remove this temporary verifier before
# publishing the result commit, preventing recursive validation builds.
python3 - <<'PY'
import json
from pathlib import Path
path = Path('package.json')
data = json.loads(path.read_text())
data['scripts']['build'] = 'npm --prefix ui ci && npm --prefix ui run build'
path.write_text(json.dumps(data, indent=2) + '\n')
PY
rm -f scripts/cloudflare-omegag1-verify.sh

# Build the ordinary UI after the transcript has been placed under ui/public.
# Deployment success remains distinct from the scientific validation status.
npm --prefix ui ci >/tmp/omega-g1-ui-install.log 2>&1 || true
npm --prefix ui run build >/tmp/omega-g1-ui-build.log 2>&1 || true

git config user.name 'cloudflare-pages[bot]'
git config user.email '73139402+cloudflare-workers-and-pages[bot]@users.noreply.github.com'
git add \
  docs/experiments/OMEGAG1_BOUNDED_GRAMMAR_EXTENSION_RESULT.md \
  docs/experiments/OMEGAG1_EXTERNAL_VALIDATION.txt \
  package.json
git add -u scripts/cloudflare-omegag1-verify.sh
if [ -f "$TRACKED_JSON" ]; then
  git add docs/experiments/OMEGAG1_BOUNDED_GRAMMAR_EXTENSION_REPORT.json
fi

git commit -m 'docs(cognition): record external ΩG1 verification' || true
RESULT_HEAD="$(git rev-parse HEAD)"
printf 'omega_g1_result_commit=%s\n' "$RESULT_HEAD"
if git push origin "HEAD:refs/heads/$BRANCH"; then
  echo 'omega_g1_evidence_push=success'
else
  echo 'omega_g1_evidence_push=failed'
fi

# Always return success so Cloudflare may publish diagnostics. The tracked
# result status and explicit validation code are the scientific authority.
exit 0
