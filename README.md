# DAC2 — Durian Smart Account

A local-first Thai durian-orchard business planning calculator. The application
is a Rust workspace: a pure calculation crate, a PostgreSQL store, and one
Leptos SSR plus hydration web crate.

This repository is being built in owner-reviewed slices. Slice 1 proved that
the chosen stack compiles and connects. Slice 2 added the pure calculation
engine and its deterministic local tests. Slice 3 adds PostgreSQL persistence,
owner-scoped access, closed-plan enforcement, and independent season
duplication. Interface behaviour remains a later slice.

## Calculation proof

The calculation crate has no I/O dependency and exposes one deterministic
entry point, `analyze(&Plan) -> Analysis`. Its module unit tests cover formula
branches and invalid or missing inputs; one golden regression test checks the
workbook sample across revenue, costs, business analysis, all nine KPIs, both
tax methods, the 25-cell scenario matrix, health scores, and completeness.

```bash
cargo test -p calc
cargo build -p calc --target wasm32-unknown-unknown
```

## Persistence proof

The store saves one complete `calc::Plan` aggregate into section-specific
tables. Every plan query carries its owner ID, every replacement is atomic, and
a closed plan rejects update, close, and delete operations. Duplicating a plan
creates a new open aggregate, including when the source season is closed.

The integration tests use SQLx-managed isolated databases on the real local
PostgreSQL container. Start the database, then run:

```bash
docker compose up -d --wait
./scripts/test-store.sh
```

The suite proves lossless complete and empty-plan round trips, every exposed
operation under the wrong owner, every mutation of a closed plan, and
independence after a deep duplicate is edited.

## Local stack proof

Prerequisites are Rust 1.88, `wasm32-unknown-unknown`, Docker, and
`cargo-leptos` 0.3.7.

Use the official prebuilt binary from the
[`cargo-leptos` v0.3.7 release](https://github.com/leptos-rs/cargo-leptos/releases/tag/v0.3.7).
On 2026-09-07, compiling that CLI from source with Rust 1.88 and `--locked`
resolved transitive packages requiring Rust 1.89 and 1.90, despite the crate's
published Rust 1.82 minimum. That affects installation of the build tool, not
this workspace, which is pinned and checked with Rust 1.88.

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
