#!/usr/bin/env bash
set -euo pipefail

# Temporary transport route. Normalize the two ΩG1 source files in the verified
# Vercel checkout and push the byte-exact result to an isolated private branch.
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

printf 'OMEGA_G1_RUSTFMT_TRANSPORT_START=1\n'
printf 'committed_head=%s\n' "$(git rev-parse HEAD)"

if ! command -v rustfmt >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --component rustfmt
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

rustfmt --edition 2021 \
  lib/grammar_extension.rs \
  lib/examples/omega_g1_bounded_grammar_extension.rs

git diff --check -- \
  lib/grammar_extension.rs \
  lib/examples/omega_g1_bounded_grammar_extension.rs
git diff --stat -- \
  lib/grammar_extension.rs \
  lib/examples/omega_g1_bounded_grammar_extension.rs

git config user.name "starfire-verifier[bot]"
git config user.email "starfire-verifier@users.noreply.github.com"
git add \
  lib/grammar_extension.rs \
  lib/examples/omega_g1_bounded_grammar_extension.rs
git commit -m "style(cognition): rustfmt ΩG1 source"

git push origin HEAD:refs/heads/ci/omega-g1-rustfmt-output
printf 'OMEGA_G1_RUSTFMT_TRANSPORT_STATUS=PUSHED\n'
printf 'transport_commit=%s\n' "$(git rev-parse HEAD)"
printf 'OMEGA_G1_RUSTFMT_TRANSPORT_FINISHED=1\n'

cd "$ROOT/ui"
npx next build
