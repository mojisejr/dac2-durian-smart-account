# DAC2 — Durian Smart Account

A local-first Thai durian-orchard business planning calculator. The application
is a Rust workspace: a pure calculation crate, a PostgreSQL store, and one
Leptos SSR plus hydration web crate.

This repository is being built in owner-reviewed slices. Slices 1–3 proved the
stack, calculation engine, and PostgreSQL persistence. Slice 4 adds local
account registration and recovery. Slice 5 added the initial plan workspace;
the season UX follow-up now supplies six Thai input sections, a non-persistent
workbook demonstration, season duplication and close, and quiet live browser
calculation. Slice 6 adds the analysis
experience: a dashboard, nine efficiency KPIs against owner-set targets, the six
completeness rules, both preliminary tax methods, and the price-by-yield scenario
matrix. The screen keeps user information to the email address only; there is
no profile image, avatar, or social login. The season UX follow-up gives every
stored season an independent Buddhist harvest year, name, and note; makes one
season the combined forecast for all plots in that year; and turns the workbook
sample into an editable, resettable browser demonstration that never creates
history.

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
PostgreSQL also enforces one season per owner and harvest year. Metadata is
editable only while the season is open, and list order follows the stored year
rather than insertion order.

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

## Analysis proof

The analysis screens add no arithmetic. They read the `Analysis` the calculation
crate already produces and decide only how a figure is named, formatted, and
withheld. A figure the engine cannot compute is shown as `ยังไม่มีข้อมูล`, never
as zero, and a KPI whose target the owner has not set shows `ยังไม่ได้ตั้งเป้า`
with a route to the targets screen rather than a verdict nobody chose.

Every panel is server rendered with the inactive ones carrying `hidden`, so the
whole analysis reaches the reader in the first response and the tests below read
what the server actually sends.

```bash
cargo test -p web --test analysis_ssr --features ssr
```

The suite proves the dashboard against the workbook's own cached business
figures, the empty-plan state that names what is missing instead of showing a
number, all nine KPI rows with their explanations, the graded and ungraded target
cases, all six completeness rules with the routes that would fix them, both tax
methods with the cheaper one marked and the disclaimer present, and all
twenty-five scenario cells including the centre the sliders start from.

## Layout proof

The Rust suites render components to an HTML string. That can prove what a
screen says and never what it does on a phone, which is how the analysis screens
shipped a table that widened every page to 705 pixels while a passing test
asserted all twenty-five of its cells were present. The markup was correct; the
scroll container never scrolled, and layout is not in a string.

`scripts/check-responsive.sh` drives real Chrome at 320, 360, 393, and 412
pixels, exercises the non-persistent demonstration and real season creation,
opens every explanation and every tab in turn, and asserts these properties:

- No page is wider than the device, and content may not push the layout viewport
  out to absorb an overflow.
- No interactive label is clipped by its own box. Scrolling and an ellipsis are
  deliberate and pass; silent clipping does not.
- Every activation target meets the 48-pixel minimum of `DESIGN.md` rule 1,
  measured on the label that activates a wrapped control, and nothing that must
  be tapped stays covered by the sticky bars once scrolled to.
- Every text colour clears 6:1 against the surface behind it, which is
  `DESIGN.md` rule 2. It is as measurable as the 48-pixel rule and went
  unmeasured until a sheet rendered its text at 1.04:1 and still passed.
- An open explanation lies wholly within the screen, closes by a control of its
  own that meets the same 48-pixel rule, and can be scrolled when its text is
  longer than the room it has. Measured at 320 pixels, every explanation is
  taller than half the screen and six are taller than all of it, so this is the
  assertion that decides the pattern rather than a preference about it.

```bash
npm install
./scripts/check-responsive.sh
```

It requires Node and Docker and starts the application itself. It is deliberately
not part of `scripts/check.sh`, which stays the compile and dependency-boundary
proof that runs without a database.

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
