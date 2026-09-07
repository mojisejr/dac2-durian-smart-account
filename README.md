# DAC2 — Durian Smart Account

A local-first Thai durian-orchard business planning calculator. The application
is a Rust workspace: a pure calculation crate, a PostgreSQL store, and one
Leptos SSR plus hydration web crate.

This repository is being built in owner-reviewed slices. Slice 1 proves only
that the chosen stack compiles and connects; it contains no product behaviour.

## Local stack proof

Prerequisites are Rust 1.88, `wasm32-unknown-unknown`, Docker, and
`cargo-leptos` 0.3.7.

```bash
docker compose up -d --wait
DATABASE_URL=postgres://postgres@127.0.0.1:54329/dac2 cargo sqlx migrate run
DATABASE_URL=postgres://postgres@127.0.0.1:54329/dac2 cargo leptos build
DATABASE_URL=postgres://postgres@127.0.0.1:54329/dac2 cargo leptos serve
```

The database is bound to localhost and uses PostgreSQL trust authentication for
this local compile proof only. No deployment configuration exists.

After `.sqlx` has been prepared, the complete compile and dependency-boundary
proof runs without a database connection:

```bash
./scripts/check.sh
```
Local-first Thai durian orchard business planning calculator built with Rust and Leptos
