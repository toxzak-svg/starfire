#!/usr/bin/env bash
set -euo pipefail

printf 'OMEGA_G2_S0_EXACT_SOURCE_VALIDATION_STARTED=1\n'
printf 'OMEGA_G2_S0_COMMIT_SHA=%s\n' "${VERCEL_GIT_COMMIT_SHA:-unknown}"

printf 'OMEGA_G2_S0_SOURCE_HASH_BEFORE_OBSERVER='
sha256sum lib/omega_g2_shadow.rs | awk '{print $1}'
printf 'OMEGA_G2_S0_SOURCE_HASH_BEFORE_PREDICTION='
sha256sum lib/prediction/mod.rs | awk '{print $1}'
printf 'OMEGA_G2_S0_SOURCE_HASH_BEFORE_PROBE='
sha256sum lib/examples/omega_g2_s0_real_trace_shadow.rs | awk '{print $1}'

cargo check -p star --all-targets --locked
cargo check -p star --all-targets --features omega-g2-shadow --locked
cargo test -p star omega_g2_shadow --features omega-g2-shadow --locked
cargo run -p star --example omega_g2_s0_real_trace_shadow --features omega-g2-shadow --locked

printf 'OMEGA_G2_S0_SOURCE_HASH_AFTER_OBSERVER='
sha256sum lib/omega_g2_shadow.rs | awk '{print $1}'
printf 'OMEGA_G2_S0_SOURCE_HASH_AFTER_PREDICTION='
sha256sum lib/prediction/mod.rs | awk '{print $1}'
printf 'OMEGA_G2_S0_SOURCE_HASH_AFTER_PROBE='
sha256sum lib/examples/omega_g2_s0_real_trace_shadow.rs | awk '{print $1}'

npm --prefix ui run build:app

printf 'OMEGA_G2_S0_EXACT_SOURCE_VALIDATION_STATUS=PASS\n'
printf 'OMEGA_G2_S0_EXACT_SOURCE_VALIDATION_FINISHED=1\n'
