# Reserve

An inventory reservation service in Rust, Axum and PostgreSQL. Create
time-limited holds, confirm checkout, cancel holds and retry accepted requests
safely across API processes. Local dashboard: http://localhost:4101.

## Run

Use the parent lab scripts, or configure PostgreSQL with `DATABASE_URL` and run:

```sh
cargo run --bin server
cargo test
cargo clippy --all-targets -- -D warnings
```

From PowerShell on this laptop: `./dev.ps1 serve`. The original teaching example
is `./dev.ps1 run`; the pure in-memory lifecycle example is `./dev.ps1 checkout`.
The HTTP service uses PostgreSQL, not the teaching example's in-memory state.

## API

- `GET /api/state`: products, current available/held units and recent holds.
- `POST /api/reservations`: `sku`, positive `quantity`, `ttl_seconds` (1–3600),
  and `idempotency_key` (1–128 bytes).
- `POST /api/reservations/{id}/confirm`: complete a held checkout, retry safely.
- `POST /api/reservations/{id}/cancel`: release a hold, retry safely.

Each writer takes a PostgreSQL row lock on the SKU before checking availability
or transitioning a hold. Expiry uses database time after lock acquisition.
Accepted creation retries persist by `(sku, request_key)`; changing quantity
or TTL with that key returns a conflict. Rejected creates are not cached.
Confirm reduces physical stock once. Cancel and expiry only release held units.
Snapshots and writes reclaim expired holds; no separate scheduler is required
for availability correctness, and no background scheduler is implemented.

## Evidence

Pure-rule unit tests remain in `src/lib.rs` and `src/reservations.rs`. The parent
`verify-all.py` tests the actual HTTP/PostgreSQL implementation: two-process
competition, concurrent creation/confirmation retries, restart persistence,
expiry boundaries and invalid inputs. The machine-generated local benchmark
report is `../reports/integration.json`; it is not production capacity evidence.

## Boundaries

Local demo without payment processing, authentication, checkout ownership,
multi-tenant data isolation or HA deployment. A bounded pool reuses up to 16 PostgreSQL connections per API process. One hot SKU is serialised, and the dashboard locks
all SKU rows while taking its snapshot. These are deliberate simplicity
tradeoffs, not claims of production-scale throughput. Local demo database
defaults are documented in the parent README and must not be used publicly.

Read `INTERVIEW.md` for the correctness argument and follow-up questions.
The original beginner walkthrough remains in `lessons/day-01.md`.
