#!/usr/bin/env bash

# Temporary clean-branch bootstrap for S6-D. The private-repository GitHub Actions
# budget is currently unavailable, so this script uses the connected Cloudflare
# Pages build container as an independent Rust runner. It fetches the frozen S6-C
# mechanics source, applies the accepted review hardening, administratively
# reclassifies the stage as S6-D after PR #71 occupied S6-C, reconciles current
# main, and pushes only after every frozen gate passes. It never force-pushes.

set -uo pipefail

ROOT="$(pwd)"
BRANCH="experiment/s6d-limited-runtime-canary"
SOURCE_SHA="cb2175d8cc67ffcbb22f8866998f535f3eaa6bad"
REPORT="$ROOT/ui/public/s6d-validation.txt"
WORK="$(mktemp -d)"
STATUS=0

mkdir -p "$ROOT/ui/public"
: >"$REPORT"
exec > >(tee -a "$REPORT") 2>&1

finish() {
  printf '\nS6D_EXTERNAL_VALIDATION_STATUS=%s\n' "$STATUS"
  printf 'S6D_EXTERNAL_VALIDATION_FINISHED=1\n'
}
trap finish EXIT

printf 'S6-D clean bootstrap started\n'
printf 'bootstrap_head=%s\n' "$(git rev-parse HEAD 2>/dev/null || printf unknown)"
printf 'frozen_source_head=%s\n' "$SOURCE_SHA"
printf 'workdir=%s\n' "$WORK"

if ! git ls-files -z | tar --null -cf - -T - | tar -xf - -C "$WORK"; then
  echo 'failed to copy tracked current-main source into disposable checkout'
  STATUS=90
  exit 0
fi

if ! git fetch --no-tags origin "$SOURCE_SHA"; then
  echo 'failed to fetch frozen S6-C mechanics source'
  STATUS=91
  exit 0
fi

extract_old() {
  local source_path="$1"
  local target_path="$2"
  mkdir -p "$(dirname "$target_path")"
  git show "$SOURCE_SHA:$source_path" >"$target_path"
}

cd "$WORK" || {
  STATUS=92
  exit 0
}

extract_old lib/companion_runtime_canary.rs lib/companion_runtime_canary.rs
extract_old lib/examples/s6c_limited_runtime_canary.rs /tmp/s6c_probe.rs
extract_old docs/experiments/S6C_LIMITED_RUNTIME_CANARY.md /tmp/s6c_prereg.md
extract_old docs/experiments/S6C_LIMITED_RUNTIME_CANARY_RESULT.md /tmp/s6c_result.md
extract_old .github/workflows/companion-runtime-canary-s6c-ci.yml /tmp/s6c_workflow.yml
extract_old .github/workflows/apply-s6c-review-fixes.yml /tmp/apply_s6c_review.yml

python3 - <<'PY'
from pathlib import Path

# Apply the exact accepted review-hardening transformation from PR #68.
workflow = Path('/tmp/apply_s6c_review.yml').read_text()
opener = "          python3 - <<'PY'\n"
start = workflow.index(opener) + len(opener)
end = workflow.index("          PY\n", start)
script = "\n".join(
    line[10:] if line.startswith("          ") else line
    for line in workflow[start:end].splitlines()
) + "\n"

Path('lib/examples/s6c_limited_runtime_canary.rs').write_text(
    Path('/tmp/s6c_probe.rs').read_text()
)
Path('docs/experiments/S6C_LIMITED_RUNTIME_CANARY_RESULT.md').write_text(
    Path('/tmp/s6c_result.md').read_text()
)
exec(compile(script, 'apply-s6c-review-fixes.py', 'exec'))

# Fix the two ordinary borrow-order defects found by the independent compiler
# diagnostics. Capture the ledger version before borrowing the ledger mutably.
probe_path = Path('lib/examples/s6c_limited_runtime_canary.rs')
lines = probe_path.read_text().splitlines()
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
probe_path.write_text('\n'.join(lines) + '\n')

# Administrative stage reclassification: PR #71 merged the evidence-intake
# boundary as S6-C while this mechanics layer was still under review.
source = Path('lib/companion_runtime_canary.rs')
source.write_text(source.read_text().replace('S6-C', 'S6-D'))

probe_text = probe_path.read_text()
probe_text = probe_text.replace('S6CReport', 'S6DReport')
probe_text = probe_text.replace('S6-C', 'S6-D')
probe_text = probe_text.replace('s6c', 's6d')
Path('lib/examples/s6d_limited_runtime_canary.rs').write_text(probe_text)
probe_path.unlink()

prereg = Path('/tmp/s6c_prereg.md').read_text().replace('S6-C', 'S6-D')
status = 'Status: **PREREGISTERED — no result yet**\n'
erratum = '''Status: **PREREGISTERED BY INHERITANCE — no terminal result yet**

## Stage reclassification erratum

This contract was originally frozen as S6-C at commit
`ad71b18d7bcaca6aef48db2042a625e1a3586aaf`. While its implementation was under
review, the independently preregistered evidence-intake program merged first as
S6-C in PR #71 (`de27f0f23f1254702c3afdeb3cbc43a33cd5e3c4`). This runtime mechanics
stage is therefore reclassified as **S6-D**. The hypothesis, controls, thresholds,
fallback rules, and authority boundaries are unchanged; only the stage identifier
and parent-gate ordering changed.

S6-D depends on the merged `companion-real-interaction-canary` evidence layer.
Production activation still requires held-out-conversation evidence and rejects
frozen simulation evidence by default.
'''
if status not in prereg:
    raise SystemExit('missing frozen preregistration status marker')
prereg = prereg.replace(status, erratum, 1)
prereg = prereg.replace(
    '4. production activation already rejects `FrozenSimulation` evidence by default.',
    '4. S6-C provides consented, independently witnessed, privacy-minimized evidence intake;\n'
    '5. production activation already rejects `FrozenSimulation` evidence by default.',
    1,
)
Path('docs/experiments/S6D_LIMITED_RUNTIME_CANARY.md').write_text(prereg)

initial_result = Path('docs/experiments/S6C_LIMITED_RUNTIME_CANARY_RESULT.md').read_text()
initial_result = initial_result.replace('S6-C', 'S6-D')
initial_result = initial_result.replace(
    'Status: **PROVISIONAL — superseded by review hardening**',
    'Status: **PENDING CLEAN-BRANCH VERIFICATION**',
    1,
)
header = '# S6-D Limited Runtime Canary — Result\n\n'
if not initial_result.startswith(header):
    raise SystemExit('unexpected inherited result header')
reclassification = '''## Stage reclassification

The original mechanics contract and initial run were labeled S6-C. After PR #71
merged the evidence-intake boundary as S6-C, this mechanics stage was reclassified
to S6-D without changing its frozen acceptance criteria. The initial artifact below
remains superseded because review found reusable trial IDs and a non-independent
event-replay assertion. A clean S6-D terminal verdict requires the corrected source
to pass the full gate from current `main`.

'''
initial_result = initial_result.replace(
    header,
    header + 'Status: **PENDING CLEAN-BRANCH VERIFICATION**\n\n' + reclassification,
    1,
)
initial_result = initial_result.replace(
    'Status: **PENDING CLEAN-BRANCH VERIFICATION**\n\n'
    'Status: **PENDING CLEAN-BRANCH VERIFICATION**\n\n',
    'Status: **PENDING CLEAN-BRANCH VERIFICATION**\n\n',
    1,
)
# Remove any inherited duplicate status immediately before Scientific provenance.
initial_result = initial_result.replace(
    '\nStatus: **PENDING CLEAN-BRANCH VERIFICATION**\n\n## Scientific provenance\n',
    '\n## Scientific provenance\n',
    1,
)
Path('docs/experiments/S6D_LIMITED_RUNTIME_CANARY_RESULT.md').write_text(initial_result)
Path('docs/experiments/S6C_LIMITED_RUNTIME_CANARY_RESULT.md').unlink()

workflow_text = Path('/tmp/s6c_workflow.yml').read_text()
workflow_text = workflow_text.replace('S6-C', 'S6-D')
workflow_text = workflow_text.replace('S6C', 'S6D')
workflow_text = workflow_text.replace('s6c', 's6d')
# Re-run both merged S6-C evidence probes before accepting S6-D.
compile_marker = '''          cargo check -p star --example s6d_limited_runtime_canary \\
            --features companion-runtime-canary --locked
'''
compile_extra = compile_marker + '''          cargo check -p star --example s6c_real_interaction_canary_evidence \\
            --features companion-runtime-canary --locked
          cargo check -p star --example s6c_external_evaluator_channel_probe \\
            --features companion-runtime-canary --locked
'''
if compile_marker not in workflow_text:
    raise SystemExit('missing S6-D compile insertion marker')
workflow_text = workflow_text.replace(compile_marker, compile_extra, 1)
run_marker = '''      - name: Run frozen S6-D canary probe
'''
s6c_regression = '''      - name: Re-run merged S6-C evidence-intake probes
        run: |
          cargo run -p star --example s6c_real_interaction_canary_evidence \\
            --features companion-runtime-canary --locked
          cargo run -p star --example s6c_external_evaluator_channel_probe \\
            --features companion-runtime-canary --locked

'''
if run_marker not in workflow_text:
    raise SystemExit('missing S6-D run insertion marker')
workflow_text = workflow_text.replace(run_marker, s6c_regression + run_marker, 1)
Path('.github/workflows/companion-runtime-canary-s6d-ci.yml').write_text(workflow_text)

# Reconcile current-main Cargo registrations without replacing S6-C or STLM.
cargo = Path('lib/Cargo.toml')
cargo_text = cargo.read_text()
example_marker = '''[[example]]
name = "stlm_l0_semantic_program_probe"
'''
example = '''[[example]]
name = "s6d_limited_runtime_canary"
path = "examples/s6d_limited_runtime_canary.rs"
required-features = ["companion-runtime-canary"]

'''
if 'name = "s6d_limited_runtime_canary"' not in cargo_text:
    if example_marker not in cargo_text:
        raise SystemExit('missing Cargo example insertion marker')
    cargo_text = cargo_text.replace(example_marker, example + example_marker, 1)
feature_marker = '''companion-real-interaction-canary = ["companion-policy-evaluation"]  # S6-C consented typed canary intake; no runtime authority
'''
feature = feature_marker + '''companion-runtime-canary = ["companion-live-policy-stress", "companion-real-interaction-canary"]  # S6-D two-phase session canary; disabled and unwired by default
'''
if 'companion-runtime-canary =' not in cargo_text:
    if feature_marker not in cargo_text:
        raise SystemExit('missing Cargo feature insertion marker')
    cargo_text = cargo_text.replace(feature_marker, feature, 1)
cargo.write_text(cargo_text)

lib = Path('lib/lib.rs')
lib_text = lib.read_text()
module_marker = '''#[cfg(feature = "companion-real-interaction-canary")]
pub mod companion_real_interaction_canary;
'''
module = module_marker + '''
// S6-D: session-scoped two-phase runtime canary. Preparation occurs against a
// cloned S6 controller and the response remains opaque until an exact registered
// S5-B delivered arm is validated. Disabled and not attached to Runtime::chat().
#[cfg(feature = "companion-runtime-canary")]
pub mod companion_runtime_canary;
'''
if 'pub mod companion_runtime_canary;' not in lib_text:
    if module_marker not in lib_text:
        raise SystemExit('missing lib.rs module insertion marker')
    lib_text = lib_text.replace(module_marker, module, 1)
lib.write_text(lib_text)
PY
PATCH_STATUS=$?
if [ "$PATCH_STATUS" -ne 0 ]; then
  echo "S6-D bootstrap transformation failed with status $PATCH_STATUS"
  STATUS=93
  exit 0
fi

printf '\n===== generated source inventory =====\n'
find lib docs/experiments .github/workflows -maxdepth 2 -type f \
  \( -name '*s6d*' -o -name '*S6D*' -o -name 'companion_runtime_canary.rs' \) \
  -print -exec sha256sum {} \;

if ! command -v cargo >/dev/null 2>&1; then
  echo 'installing stable Rust toolchain'
  if ! curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --profile minimal --component rustfmt,clippy; then
    echo 'Rust installation failed'
    STATUS=94
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

run_gate rustfmt-write \
  rustfmt --edition 2021 \
    lib/companion_runtime_canary.rs \
    lib/examples/s6d_limited_runtime_canary.rs
run_gate rustfmt-check \
  rustfmt --edition 2021 --check \
    lib/companion_runtime_canary.rs \
    lib/examples/s6d_limited_runtime_canary.rs
run_gate library-check \
  cargo check -p star --lib --features companion-runtime-canary --locked
run_gate inherited-example-checks \
  cargo check -p star --examples --features companion-runtime-canary --locked

printf '\n===== scoped Clippy =====\n'
set -o pipefail
{
  cargo clippy -p star --lib --features companion-runtime-canary --locked \
    --message-format=short
  cargo clippy -p star --example s6d_limited_runtime_canary \
    --features companion-runtime-canary --locked --message-format=short
} 2>&1 | tee /tmp/s6d-clippy.log
CLIPPY_STATUS=${PIPESTATUS[0]}
set +o pipefail
printf 'gate_status[scoped-clippy-command]=%s\n' "$CLIPPY_STATUS"
if [ "$CLIPPY_STATUS" -ne 0 ] && [ "$STATUS" -eq 0 ]; then
  STATUS="$CLIPPY_STATUS"
fi
if grep -E \
  'lib/companion_runtime_canary\.rs:|lib/examples/s6d_limited_runtime_canary\.rs:' \
  /tmp/s6d-clippy.log; then
  echo 'S6-D scoped Clippy finding detected'
  if [ "$STATUS" -eq 0 ]; then
    STATUS=95
  fi
else
  echo 'S6-D scoped Clippy findings: 0'
fi

run_gate s6d-unit-tests \
  cargo test -p star --lib --features companion-runtime-canary --locked \
    companion_runtime_canary:: -- --test-threads=1
run_gate s6a-regression \
  cargo run -p star --example s6a_bounded_live_policy_probe \
    --features companion-runtime-canary --locked
run_gate s6b-regression \
  cargo run -p star --example s6b_adversarial_live_policy_stress \
    --features companion-runtime-canary --locked
run_gate s6c-evidence-regression \
  cargo run -p star --example s6c_real_interaction_canary_evidence \
    --features companion-runtime-canary --locked
run_gate s6c-external-evaluator-regression \
  cargo run -p star --example s6c_external_evaluator_channel_probe \
    --features companion-runtime-canary --locked
run_gate s6d-corrected-probe \
  cargo run -p star --example s6d_limited_runtime_canary \
    --features companion-runtime-canary --locked

if [ "$STATUS" -ne 0 ]; then
  echo 'S6-D gate failed closed; no source commit will be published'
  exit 0
fi

# Record the exact validated source as a code commit, then add a result-only
# commit that names that immutable source commit.
cd "$ROOT" || {
  STATUS=96
  exit 0
}
cp "$WORK/lib/companion_runtime_canary.rs" lib/companion_runtime_canary.rs
cp "$WORK/lib/examples/s6d_limited_runtime_canary.rs" \
  lib/examples/s6d_limited_runtime_canary.rs
cp "$WORK/docs/experiments/S6D_LIMITED_RUNTIME_CANARY.md" \
  docs/experiments/S6D_LIMITED_RUNTIME_CANARY.md
cp "$WORK/docs/experiments/S6D_LIMITED_RUNTIME_CANARY_RESULT.md" \
  docs/experiments/S6D_LIMITED_RUNTIME_CANARY_RESULT.md
cp "$WORK/.github/workflows/companion-runtime-canary-s6d-ci.yml" \
  .github/workflows/companion-runtime-canary-s6d-ci.yml
cp "$WORK/lib/Cargo.toml" lib/Cargo.toml
cp "$WORK/lib/lib.rs" lib/lib.rs

python3 - <<'PY'
import json
from pathlib import Path
path = Path('package.json')
data = json.loads(path.read_text())
data['scripts']['build'] = 'npm --prefix ui ci && npm --prefix ui run build'
path.write_text(json.dumps(data, indent=2) + '\n')
PY
rm -f scripts/cloudflare-s6d-bootstrap.sh

git config user.name 'cloudflare-pages[bot]'
git config user.email '73139402+cloudflare-workers-and-pages[bot]@users.noreply.github.com'
git add \
  lib/companion_runtime_canary.rs \
  lib/examples/s6d_limited_runtime_canary.rs \
  docs/experiments/S6D_LIMITED_RUNTIME_CANARY.md \
  docs/experiments/S6D_LIMITED_RUNTIME_CANARY_RESULT.md \
  .github/workflows/companion-runtime-canary-s6d-ci.yml \
  lib/Cargo.toml \
  lib/lib.rs \
  package.json
git add -u scripts/cloudflare-s6d-bootstrap.sh

git commit -m 'feat(companion): add review-hardened S6-D runtime canary'
CODE_SHA="$(git rev-parse HEAD)"
echo "verified_code_commit=$CODE_SHA"

CODE_SHA_ENV="$CODE_SHA" python3 - <<'PY'
from os import environ
from pathlib import Path
path = Path('docs/experiments/S6D_LIMITED_RUNTIME_CANARY_RESULT.md')
text = path.read_text()
text = text.replace(
    'Status: **PENDING CLEAN-BRANCH VERIFICATION**',
    'Status: **PASS — REVIEW-HARDENED MECHANICS**',
    1,
)
section = f'''

## Clean S6-D terminal verification

```text
verified source commit:       {environ['CODE_SHA_ENV']}
opaque S5-B registration:     PASS
one-shot trial IDs:           PASS
independent canary replay:    PASS
S6-D unit contracts:          PASS
S6-A regression:              PASS
S6-B regression:              PASS
S6-C evidence regression:     PASS
S6-C evaluator regression:    PASS
corrected S6-D probe:         PASS
scoped S6-D Clippy findings:  0
unauthorized applied turns:   0
```

The source commit above was produced only after every frozen command passed in the
independent Cloudflare build container. This PASS establishes session-isolation,
two-phase delivery, exact registered-arm binding, one-shot trial consumption,
neutral fallback, revocation, and deterministic replay mechanics only. It does not
establish real conversational benefit, default live-chat authority, AGI, or
consciousness.
'''
if '## Clean S6-D terminal verification' not in text:
    text += section
path.write_text(text)
PY

git add docs/experiments/S6D_LIMITED_RUNTIME_CANARY_RESULT.md
git commit -m 'docs(companion): freeze clean S6-D mechanics PASS'
RESULT_SHA="$(git rev-parse HEAD)"
echo "result_record_commit=$RESULT_SHA"

if git push origin "HEAD:refs/heads/$BRANCH"; then
  echo 'verified_s6d_push=success'
else
  echo 'verified_s6d_push=failed'
  STATUS=97
fi

exit 0
