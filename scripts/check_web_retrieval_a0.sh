#!/usr/bin/env sh
set -eu

# Mirrors the permanent Web Retrieval CI gate for local and alternate runners.
# Keep this command path deterministic and network-independent beyond Cargo fetches.
FEATURES="web-search-searxng"

rustfmt --edition 2021 --config skip_children=true --check lib/lib.rs
rustfmt --edition 2021 --check \
  lib/web_retrieval.rs \
  lib/web_search_searxng.rs \
  lib/web_content_extract.rs \
  lib/web_research.rs

cargo check -p star --lib --features "$FEATURES" --locked
cargo clippy -p star --lib --features "$FEATURES" --locked -- -D warnings

for suite in \
  web_retrieval:: \
  web_search_searxng:: \
  web_content_extract:: \
  web_research::
do
  cargo test -p star --lib --features "$FEATURES" --locked \
    "$suite" -- --test-threads=1
done
