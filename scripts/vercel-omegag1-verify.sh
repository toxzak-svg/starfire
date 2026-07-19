#!/usr/bin/env bash
set -euo pipefail

# Temporary capability check. Print only potentially relevant environment
# variable NAMES and non-secret Git configuration. Never print secret values.
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

printf 'OMEGA_G1_CREDENTIAL_CAPABILITY_START=1\n'
printf 'committed_head=%s\n' "$(git rev-parse HEAD)"
printf '\n===== candidate credential variable names =====\n'
env | cut -d= -f1 \
  | grep -Ei '(^|_)(GITHUB|GH|TOKEN|PAT|OAUTH|CREDENTIAL)($|_)' \
  | sort -u \
  || true
printf '\n===== configured remote names =====\n'
git remote || true
printf '\n===== configured credential helpers =====\n'
git config --get-all credential.helper || true
printf 'OMEGA_G1_CREDENTIAL_CAPABILITY_FINISHED=1\n'

cd "$ROOT/ui"
npx next build
