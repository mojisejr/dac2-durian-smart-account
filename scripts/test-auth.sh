#!/usr/bin/env bash

set -euo pipefail

: "${DATABASE_URL:=postgres://postgres@127.0.0.1:54329/dac2}"
export DATABASE_URL

docker compose up -d --wait
cargo test -p store --test accounts
cargo test -p web --features ssr
