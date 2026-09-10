#!/usr/bin/env bash

set -euo pipefail

# Layout proof at real device widths. Starts the application, drives Chrome
# through every route, and fails on the first screen that breaks a DESIGN.md
# rule the Rust suites cannot see.
#
# Requires Node and the local Playwright install: npm install

cd "$(dirname "$0")/.."

if [[ ! -d node_modules/playwright ]]; then
  echo "playwright is not installed; run: npm install" >&2
  exit 1
fi

: "${DATABASE_URL:=postgres://postgres@127.0.0.1:54329/dac2}"
: "${SESSION_KEY:=$(head -c 96 /dev/urandom | od -An -tx1 | tr -d ' \n')}"
: "${APP_BASE_URL:=http://127.0.0.1:3000}"
export DATABASE_URL SESSION_KEY APP_BASE_URL

docker compose up -d --wait
cargo sqlx migrate run
cargo leptos build

cargo leptos serve &
server=$!
trap 'kill "$server" 2>/dev/null || true' EXIT

for _ in $(seq 1 120); do
  if curl -sf -o /dev/null "$APP_BASE_URL/" 2>/dev/null; then break; fi
  sleep 1
done
curl -sf -o /dev/null "$APP_BASE_URL/" || { echo "the application did not start" >&2; exit 1; }

node scripts/responsive.mjs
