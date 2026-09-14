#!/usr/bin/env bash

set -euo pipefail

# Container proof. Builds the image from the working tree, runs it against the
# local compose services with environment variables alone, and points the
# layout proof and the stall probe at it. This is the only place the release
# binary is exercised: `cargo leptos serve` runs a debug build, and the two
# have already differed once (see the runtime comment in crates/web/src/main.rs).
#
# Requires Docker, Node, and the local Playwright install: npm install
#
#   IMAGE=dac2:local   image tag to build and run
#   PORT=3100          host port the container is published on
#   SKIP_BUILD=1       run the existing image instead of building
#   ROUNDS, LOAD       forwarded to the stall probe

cd "$(dirname "$0")/.."

if [[ ! -d node_modules/playwright ]]; then
  echo "playwright is not installed; run: npm install" >&2
  exit 1
fi

: "${IMAGE:=dac2:local}"
: "${PORT:=3100}"
: "${SESSION_KEY:=$(head -c 96 /dev/urandom | od -An -tx1 | tr -d ' \n')}"
: "${ROUNDS:=40}"
: "${LOAD:=3}"
# The proof registers twice per run; a rerun inside the default account
# window would be refused, so the budgets are widened here. The default
# limits are proved by crates/web/tests/gate_http.rs.
: "${RATE_LIMIT_ACCOUNT:=60/3600}"
: "${RATE_LIMIT_LOGIN:=60/300}"
container="dac2-container-proof"
base_url="http://127.0.0.1:${PORT}"

if [[ -z "${SKIP_BUILD:-}" ]]; then
  docker build -t "$IMAGE" .
fi

docker compose up -d --wait

clean_users() {
  docker compose exec -T database psql -U postgres -d dac2 -v ON_ERROR_STOP=1 \
    -c "DELETE FROM users WHERE email_canonical IN ('responsive-proof@dac2.local', 'stall-probe@dac2.local')" \
    >/dev/null 2>&1 || true
}
cleanup() {
  docker rm -f "$container" >/dev/null 2>&1 || true
  clean_users
}
trap cleanup EXIT

docker rm -f "$container" >/dev/null 2>&1 || true
clean_users

docker run -d --name "$container" -p "127.0.0.1:${PORT}:3000" \
  -e DATABASE_URL=postgres://postgres@host.docker.internal:54329/dac2 \
  -e SESSION_KEY="$SESSION_KEY" \
  -e SMTP_HOST=host.docker.internal \
  -e APP_BASE_URL="$base_url" \
  -e PILOT_NOTICE=true \
  -e RATE_LIMIT_ACCOUNT="$RATE_LIMIT_ACCOUNT" \
  -e RATE_LIMIT_LOGIN="$RATE_LIMIT_LOGIN" \
  "$IMAGE" >/dev/null

for _ in $(seq 1 60); do
  if curl -sf -o /dev/null "$base_url/" 2>/dev/null; then break; fi
  sleep 1
done
if ! curl -sf -o /dev/null "$base_url/"; then
  echo "the container did not start" >&2
  docker logs "$container" >&2 || true
  exit 1
fi

docker logs "$container"
docker exec "$container" id

DAC2_BASE_URL="$base_url" node scripts/responsive.mjs
clean_users
DAC2_BASE_URL="$base_url" ROUNDS="$ROUNDS" LOAD="$LOAD" node scripts/stall-probe.mjs

if ! docker ps --filter "name=$container" --filter status=running --format '{{.Names}}' | grep -q "$container"; then
  echo "the container stopped during the proof" >&2
  docker logs "$container" >&2 || true
  exit 1
fi

echo "Container proof passed against $IMAGE on $base_url"
