#!/usr/bin/env bash
set -euo pipefail

printf 'OMEGA_G3_EXACT_SOURCE_VALIDATION_STARTED=1\n'
printf 'OMEGA_G3_COMMIT_SHA=%s\n' "${VERCEL_GIT_COMMIT_SHA:-unknown}"
printf 'OMEGA_G3_PREREGISTRATION_COMMIT=723f1233db6573d117212e064c2d6a113640c855\n'

printf 'OMEGA_G3_SOURCE_HASH_BEFORE_KERNEL='
sha256sum lib/multistep_abstraction_reuse.rs | awk '{print $1}'
printf 'OMEGA_G3_SOURCE_HASH_BEFORE_PROBE='
sha256sum lib/examples/omega_g3_multistep_abstraction_reuse.rs | awk '{print $1}'

cargo check -p star --all-targets --locked
cargo test -p star multistep_abstraction_reuse --locked
cargo run -p star --example omega_g3_multistep_abstraction_reuse --locked 2>&1 | tee omega-g3-probe.log

grep -F '"terminal_classification": "PASS"' omega-g3-probe.log
test -f target/omega-g3-multistep-abstraction-reuse-report.json
grep -F '"terminal_classification": "PASS"' target/omega-g3-multistep-abstraction-reuse-report.json

printf 'OMEGA_G3_REPORT_DIGEST='
sha256sum target/omega-g3-multistep-abstraction-reuse-report.json | awk '{print $1}'
printf 'OMEGA_G3_SOURCE_HASH_AFTER_KERNEL='
sha256sum lib/multistep_abstraction_reuse.rs | awk '{print $1}'
printf 'OMEGA_G3_SOURCE_HASH_AFTER_PROBE='
sha256sum lib/examples/omega_g3_multistep_abstraction_reuse.rs | awk '{print $1}'

npm --prefix ui run build:app

printf 'OMEGA_G3_EXACT_SOURCE_VALIDATION_STATUS=PASS\n'
printf 'OMEGA_G3_EXACT_SOURCE_VALIDATION_FINISHED=1\n'
