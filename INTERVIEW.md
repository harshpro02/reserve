# Reserve — interview walkthrough

## Problem and demo

Two buyers cannot both buy the last unit. A shopping cart needs a time-limited
hold, while network retries must not allocate stock twice.

Open the local dashboard. Create a hold with a 10-second TTL. Repeat its key,
then change the quantity with the same key. Confirm another hold twice. Let a
third expire and try to pay. Inspect the stock and ledger after every action.

## Architecture and correctness argument

Axum handles HTTP; `tokio-postgres` executes parameterised SQL. PostgreSQL is
the durable authority. Every writer locks the SKU row first. Under that lock it
expires overdue holds using database time, computes active held units, checks
availability, and creates or transitions the hold in one transaction. A second
server follows the same lock protocol, so the invariant crosses process boundaries.

Confirming subtracts from physical stock and changes the hold's status atomically.
Repeated confirmation returns the existing result. Cancelling does not change
physical stock. Expired, cancelled and confirmed holds never become active again.
The unique `(sku, request_key)` constraint persists accepted creation retries;
quantity and TTL must match. Replaying an expired key returns its old record,
not a new hold. Rejected creations are not cached. Keys are scoped per SKU.

UUIDs identify holds; PostgreSQL constraints reject invalid stored quantities,
states and negative physical stock. The application lock protocol, rather than
a cross-row SQL constraint, protects the aggregate `available >= 0` invariant.
Direct external SQL writers must follow that same protocol.

## Tests and evidence

`cargo test` checks the pure lifecycle rules. The parent `verify-all.py` creates
isolated fixtures, races actual HTTP requests through two processes, races
creation and confirmation retries, checks TTL and invalid inputs, and restarts
the second server to verify persistent holds. Exact results are in the parent
`reports/integration.json`, not handwritten into this guide.

## Questions to prepare for

- Why a database row lock instead of an in-process Rust mutex? A mutex cannot
  coordinate separate API processes. Locks serialise one hot SKU, not all SKUs.
- What happens after commit but before the client sees the response? Retry the
  same key. The committed hold is returned without allocating stock again.
- Why check time after acquiring the lock? A request can wait long enough for
  a hold to expire. PostgreSQL's wall clock is read after the lock is acquired.
- What happens if the process crashes mid-transaction? PostgreSQL rolls it back.
- How do expiry updates become visible without a worker? Writes and snapshots
  expire holds before counting. No standalone expiry scheduler is implemented.

## Tradeoffs and limits

A bounded pool reuses at most 16 connections per API process. Its earlier
connection-per-request implementation and actual before/after measurements are
preserved in the parent evidence report, including unfavourable tail results.
The dashboard snapshot
locks every SKU in sorted order, which is consistent but briefly blocks writers.
A hot SKU is deliberately serialised. There is no payment gateway, tenant/auth
layer, checkout ownership, stock-replenishment API, HA deployment, or production
load evidence. The local in-memory lesson code remains separate from the actual
database service so its limitations are easy to compare.
