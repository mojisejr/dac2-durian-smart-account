#!/usr/bin/env bash

set -euo pipefail

: "${DATABASE_URL:=postgres://postgres@127.0.0.1:54329/dac2}"
export DATABASE_URL

cargo test -p store --test plans
