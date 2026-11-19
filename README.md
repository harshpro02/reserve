# Reserve — learning Rust through inventory reservations

We are building this in small lessons over approximately one week, followed by
Restock Radar in week two. This folder is the Rust implementation. The separate
Rails prototype is retained as a reference.

## Day 1: reserve a few items

Start with five shirts. Alice holds three, leaving two available. Bob holds
the remaining two, leaving zero available. Nothing has been sold yet, so there
are still five physical items, all held. The tests also cover rejecting a
request for more than the available stock.

```text
available = on_hand - held
```

The fields are private so callers use the reservation method to change the hold
count. Quantity zero and quantities larger than available stock return errors
without changing the stock.

## Run from PowerShell

Rust is installed in Ubuntu (WSL). From this folder:

```powershell
.\dev.ps1 run
.\dev.ps1 test
```

From an Ubuntu terminal in the same folder, use `cargo run` and `cargo test`.

Other checks: `./dev.ps1 fmt` and `./dev.ps1 clippy`.

Verified on November 18, 2026 with Rust 1.100.0 in Ubuntu WSL: the example
runs, all four tests pass, formatting passes, and Clippy reports no warnings.

## Read these files in order

1. `src/main.rs`: the example with Alice and Bob.
2. `src/lib.rs`: the stock representation, reservation rule, and tests.
3. `lessons/day-01.md`: explanations and a small exercise.

## What this step covers

- A `struct` to group stock data.
- An `enum` to describe possible errors.
- `&self` to read stock and `&mut self` to change it.
- `Result` to return success or an expected failure.
- Tests for successful, invalid, and sequential requests.

## Later steps

Add real reservation records and expiry, an Axum API, PostgreSQL transactions,
durable idempotency keys, concurrency checks, and a small dashboard. These are
not implemented in this Rust starter. The current data exists only in memory
and resets on each run. Its tests do not establish concurrent or production
correctness.

References: [Rust Book](https://doc.rust-lang.org/book/) and
[official installation guide](https://rust-lang.org/tools/install/).
