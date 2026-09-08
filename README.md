# DAC2 — Durian Smart Account

A local-first Thai durian-orchard business planning calculator. The application
is a Rust workspace: a pure calculation crate, a PostgreSQL store, and one
Leptos SSR plus hydration web crate.

This repository is being built in owner-reviewed slices. Slices 1–3 proved the
stack, calculation engine, and PostgreSQL persistence. Slice 4 adds local
account registration and recovery. Slice 5 adds the usable plan workspace: six
Thai input sections, the workbook sample, a one-action clear flow, season
duplication and close, and live in-browser totals. The screen keeps user
information to the email address only; there is no profile image, avatar, or
social login.

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

## Account and trust proof

Registration compares trimmed email addresses without ASCII case distinctions,
while retaining the original spelling for display and mail. PostgreSQL is the
final duplicate guard. Passwords use Argon2id and accounts remain unable to log
in until the single-use verification link has been opened.

Mailpit is the local SMTP destination and inbox. Mail is available only at
<http://127.0.0.1:8025>; this slice uses no SaaS and sends nothing to a real mail
provider. Run the complete account suite with:

```bash
./scripts/test-auth.sh
```

That command starts PostgreSQL and Mailpit, then runs the account database,
session HTTP, and mail-delivery integration suites. It covers canonical and
concurrent duplicates, token expiry and single use, verified activation,
session rotation, protected routes, server-side logout, and session invalidation
after a password reset.

## Run locally

Prerequisites are Rust 1.88, `wasm32-unknown-unknown`, Docker, and
`cargo-leptos` 0.3.7.

Use the official prebuilt binary from the
[`cargo-leptos` v0.3.7 release](https://github.com/leptos-rs/cargo-leptos/releases/tag/v0.3.7).
On 2026-09-07, compiling that CLI from source with Rust 1.88 and `--locked`
resolved transitive packages requiring Rust 1.89 and 1.90, despite the crate's
published Rust 1.82 minimum. That affects installation of the build tool, not
this workspace, which is pinned and checked with Rust 1.88.

```bash
cp .env.example .env
# Fill DATABASE_URL, SESSION_KEY (at least 64 bytes), and the local mail values.
# Keep .env on this machine; it is ignored by Git.
set -a
source .env
set +a
docker compose up -d --wait
cargo sqlx migrate run
cargo leptos build
cargo leptos serve
```

Open the application at <http://127.0.0.1:3000>. After registration, open the
message in Mailpit and follow its verification link before logging in. The
cookie is intentionally non-Secure only for this localhost workflow. A network
deployment requires the deferred rate limiting, real SMTP, HTTPS, and Secure
cookie gate first.

## Plan workspace proof

The browser retains raw field text while the owner types, then maps it into the
same `calc::Plan` contract used by the calculation engine. Invalid numbers stay
visible for correction, percentages are converted only at the boundary, and a
grade mix is accepted only when its entered shares total 100%. Closed seasons
render as text rather than disabled form controls; the PostgreSQL store remains
the final read-only enforcement.

Run the plan-specific unit, SSR smoke, PostgreSQL store, and authenticated plan
operation suites with:

```bash
docker compose up -d --wait
./scripts/test-plans.sh
```

The six section routes are server rendered and hydrated. Sarabun font files are
served by the application itself under the SIL Open Font License; no font CDN
or other runtime SaaS is used.

The database is bound to localhost and uses PostgreSQL trust authentication for
this local compile proof only. No deployment configuration exists.

After `.sqlx` has been prepared, the complete compile and dependency-boundary
proof runs without a database connection:

```bash
./scripts/check.sh
```
