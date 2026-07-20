#!/usr/bin/env bash
set -euo pipefail

printf 'OMEGA_G2_EXACT_SOURCE_VALIDATION_STARTED=1\n'
printf 'OMEGA_G2_VALIDATION_ATTEMPT=3\n'
printf 'OMEGA_G2_COMMIT_SHA=%s\n' "${VERCEL_GIT_COMMIT_SHA:-unknown}"

printf 'OMEGA_G2_SOURCE_HASH_BEFORE_KERNEL='
sha256sum lib/recursive_grammar_composition.rs | awk '{print $1}'
printf 'OMEGA_G2_SOURCE_HASH_BEFORE_PROBE='
sha256sum lib/examples/omega_g2_recursive_grammar_composition.rs | awk '{print $1}'

cargo check -p star --all-targets --locked
cargo test -p star recursive_grammar_composition --locked
cargo run -p star --example omega1_endogenous_state_space_genesis --locked
cargo run -p star --example omega_g1_bounded_grammar_extension --locked
cargo run -p star --example omega_g2_recursive_grammar_composition --locked

printf 'OMEGA_G2_REPORT_DIGEST='
sha256sum target/omega-g2-recursive-grammar-composition-report.json | awk '{print $1}'
printf 'OMEGA_G2_SOURCE_HASH_AFTER_KERNEL='
sha256sum lib/recursive_grammar_composition.rs | awk '{print $1}'
printf 'OMEGA_G2_SOURCE_HASH_AFTER_PROBE='
sha256sum lib/examples/omega_g2_recursive_grammar_composition.rs | awk '{print $1}'

npm --prefix ui ci
npm --prefix ui run build:app

printf 'OMEGA_G2_EXACT_SOURCE_VALIDATION_STATUS=PASS\n'
printf 'OMEGA_G2_EXACT_SOURCE_VALIDATION_FINISHED=1\n'
