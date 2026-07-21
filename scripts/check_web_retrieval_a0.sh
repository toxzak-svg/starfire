#!/usr/bin/env sh
set -eu

rustfmt --edition 2021 --check \
  lib/lib.rs \
  lib/web_retrieval.rs

cargo check -p star --lib --features web-retrieval-a0 --locked
cargo clippy -p star --lib --features web-retrieval-a0 --locked -- -D warnings
cargo test -p star --lib --features web-retrieval-a0 --locked \
  web_retrieval:: -- --test-threads=1
