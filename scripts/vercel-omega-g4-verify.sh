#!/usr/bin/env bash
set -euo pipefail

printf 'OMEGA_G4_EXACT_SOURCE_VALIDATION_STARTED=1\n'
printf 'OMEGA_G4_VALIDATION_ATTEMPT=4\n'
printf 'OMEGA_G4_VERIFIED_TRIGGER=1\n'
printf 'OMEGA_G4_PUBLIC_INTEGRATION_GATE=1\n'
printf 'OMEGA_G4_COMMIT_SHA=%s\n' "${VERCEL_GIT_COMMIT_SHA:-unknown}"
printf 'OMEGA_G4_PREREGISTRATION_COMMIT=d6778cf29db725775c0d6815a6d23d6398c74010\n'

printf 'OMEGA_G4_SOURCE_HASH_BEFORE_KERNEL='
sha256sum lib/intervention_guided_abstraction_selection.rs | awk '{print $1}'
printf 'OMEGA_G4_SOURCE_HASH_BEFORE_PROBE='
sha256sum lib/examples/omega_g4_intervention_guided_abstraction_selection.rs | awk '{print $1}'

cargo check -p star --all-targets --locked
cargo test -p star --test omega_g4_selection --locked
cargo run -p star --example omega_g4_intervention_guided_abstraction_selection --locked 2>&1 | tee omega-g4-probe.log

grep -F '"terminal_classification": "PASS"' omega-g4-probe.log
test -f target/omega-g4-intervention-guided-abstraction-selection-report.json
grep -F '"terminal_classification": "PASS"' target/omega-g4-intervention-guided-abstraction-selection-report.json

printf 'OMEGA_G4_REPORT_DIGEST='
sha256sum target/omega-g4-intervention-guided-abstraction-selection-report.json | awk '{print $1}'
printf 'OMEGA_G4_SOURCE_HASH_AFTER_KERNEL='
sha256sum lib/intervention_guided_abstraction_selection.rs | awk '{print $1}'
printf 'OMEGA_G4_SOURCE_HASH_AFTER_PROBE='
sha256sum lib/examples/omega_g4_intervention_guided_abstraction_selection.rs | awk '{print $1}'

npm --prefix ui run build:app

printf 'OMEGA_G4_EXACT_SOURCE_VALIDATION_STATUS=PASS\n'
printf 'OMEGA_G4_EXACT_SOURCE_VALIDATION_FINISHED=1\n'
