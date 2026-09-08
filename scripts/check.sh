#!/usr/bin/env bash

set -euo pipefail

cargo fmt --all -- --check
cargo test -p calc
cargo test -p store --lib
cargo test -p web --lib --features ssr
cargo test -p web --test plan_ssr --features ssr
SQLX_OFFLINE=true cargo check --workspace --all-targets --all-features
SQLX_OFFLINE=true cargo clippy --workspace --all-targets --all-features -- -D warnings
SQLX_OFFLINE=true cargo leptos build
cargo build -p calc --target wasm32-unknown-unknown

calc_tree=$(cargo tree -p calc)
if grep -Eiq 'sqlx|axum|leptos|tokio' <<<"$calc_tree"; then
  echo "calc dependency boundary contains an I/O or web crate" >&2
  exit 1
fi

hydrate_tree=$(cargo tree -p web --target wasm32-unknown-unknown --no-default-features --features hydrate --edges normal)
if grep -Eiq 'sqlx' <<<"$hydrate_tree"; then
  echo "hydrate dependency graph contains sqlx" >&2
  exit 1
fi

if grep -Eaiq 'sqlx' target/site/pkg/dac2.wasm target/site/pkg/dac2.js; then
  echo "generated browser bundle contains an sqlx symbol" >&2
  exit 1
fi

echo "DAC2 stack and dependency-boundary checks passed"
