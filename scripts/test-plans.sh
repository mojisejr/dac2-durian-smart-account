#!/usr/bin/env bash

set -euo pipefail

: "${DATABASE_URL:=postgres://postgres@127.0.0.1:54329/dac2}"
export DATABASE_URL

cargo test -p web --lib --features ssr plan_form::tests
cargo test -p web --test plan_ssr --features ssr
cargo test -p store --test plans
cargo test -p web --test plan_server --features ssr
