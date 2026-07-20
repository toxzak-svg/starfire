#!/usr/bin/env bash
set -euo pipefail

# Temporary exact-source verification route. The GitHub-generated merge commit
# is the signed source revision executed by the connected Vercel builder.
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

printf 'OMEGA_V1A_EXACT_SOURCE_VALIDATION_STARTED=1\n'
printf 'OMEGA_V1A_COMMIT_SHA=%s\n' "${VERCEL_GIT_COMMIT_SHA:-unknown}"
printf 'OMEGA_V1A_FROZEN_SOURCE_COMMIT=4cfba5e2d5cdf3c982ec43e358e2cc840b56a800\n'

printf 'OMEGA_V1A_SOURCE_HASH_BEFORE_MODULE='
sha256sum lib/omega_v1_voice_baseline/mod.rs | awk '{print $1}'
printf 'OMEGA_V1A_SOURCE_HASH_BEFORE_METRICS='
sha256sum lib/omega_v1_voice_baseline/metrics.rs | awk '{print $1}'
printf 'OMEGA_V1A_CORPUS_HASH_BEFORE='
find lib/fixtures/omega_v1a -type f -print0 | sort -z | xargs -0 cat | sha256sum | awk '{print $1}'

cargo check -p star --all-targets --features omega-v1-baseline --locked
cargo test -p star --features omega-v1-baseline --locked omega_v1_voice_baseline

cargo run -p star --example omega_v1a_voice_baseline \
  --features omega-v1-baseline --locked 2>&1 | tee omega-v1a-probe.log

grep -F '"gate_passed": true' omega-v1a-probe.log
grep -F '"fixture_count": 122' omega-v1a-probe.log
grep -F '"exact_snapshot_match_rate": 1.0' omega-v1a-probe.log
grep -F '"semantic_claim_preservation": 1.0' omega-v1a-probe.log
grep -F '"prohibited_implication_absence": 1.0' omega-v1a-probe.log
grep -F '"adversarial_safety_pass_rate": 1.0' omega-v1a-probe.log

printf 'OMEGA_V1A_REPORT_DIGEST='
sha256sum omega-v1a-probe.log | awk '{print $1}'
printf 'OMEGA_V1A_SOURCE_HASH_AFTER_MODULE='
sha256sum lib/omega_v1_voice_baseline/mod.rs | awk '{print $1}'
printf 'OMEGA_V1A_SOURCE_HASH_AFTER_METRICS='
sha256sum lib/omega_v1_voice_baseline/metrics.rs | awk '{print $1}'
printf 'OMEGA_V1A_CORPUS_HASH_AFTER='
find lib/fixtures/omega_v1a -type f -print0 | sort -z | xargs -0 cat | sha256sum | awk '{print $1}'

npm --prefix ui run build:app

printf 'OMEGA_V1A_EXACT_SOURCE_VALIDATION_STATUS=PASS\n'
printf 'OMEGA_V1A_EXACT_SOURCE_VALIDATION_FINISHED=1\n'
