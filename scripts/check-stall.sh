#!/usr/bin/env bash

set -euo pipefail

# Server-rendering stall proof. Starts the application and drives it over
# persistent HTTP/1.1 sockets under wasm-download load, without a browser,
# failing on any page whose response never terminates. See
# scripts/stall-probe.mjs for what it found and how sure it can be.
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
: "${ROUNDS:=40}"
: "${LOAD:=3}"
export DATABASE_URL SESSION_KEY APP_BASE_URL ROUNDS LOAD

docker compose up -d --wait
cargo sqlx migrate run
cargo leptos build

cargo leptos serve &
server=$!
cleanup() {
  kill "$server" 2>/dev/null || true
  docker compose exec -T database psql -U postgres -d dac2 -v ON_ERROR_STOP=1 \
    -c "DELETE FROM users WHERE email_canonical = 'stall-probe@dac2.local'" \
    >/dev/null 2>&1 || true
}
trap cleanup EXIT

docker compose exec -T database psql -U postgres -d dac2 -v ON_ERROR_STOP=1 \
  -c "DELETE FROM users WHERE email_canonical = 'stall-probe@dac2.local'" \
  >/dev/null

for _ in $(seq 1 120); do
  if curl -sf -o /dev/null "$APP_BASE_URL/" 2>/dev/null; then break; fi
  sleep 1
done
curl -sf -o /dev/null "$APP_BASE_URL/" || { echo "the application did not start" >&2; exit 1; }

node scripts/stall-probe.mjs
