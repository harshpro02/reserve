# Day 1 — one shelf and one reservation function

The goal is to understand this example before adding a web server or database.

## 1. Data: `struct Stock`

`Stock` groups three values:

- `sku`: the product's code, owned by the stock object as a `String`.
- `on_hand`: how many physical items remain.
- `held`: how many items are set aside.

`u32` is a non-negative, 32-bit whole number. A hold changes `held`; a completed
purchase will eventually change `on_hand`. Completing purchases is a later step.

`Stock::new("TEE", 5)` creates stock with five items and zero held.
`to_string()` gives this stock object its own product-code string.

## 2. Reading versus changing

`available(&self)` borrows the stock to read it. It calculates
`on_hand - held` without changing either number.

`reserve(&mut self, quantity)` borrows the stock with permission to change it.
That is why the example declares `let mut stock`: this variable can be changed.
The borrow is temporary; the caller can use the stock again after the method
finishes. We do not need explicit lifetime annotations for this lesson.

## 3. Success and failure: `Result`

`Result<u32, ReserveError>` means the function returns one of two outcomes:

- `Ok(2)`: success, with two items still available.
- `Err(ReserveError::InsufficientStock)`: an expected failure.

The enum also has `InvalidQuantity` for a request for zero items.
These errors are ordinary business outcomes, so the function does not panic.

## 4. Walk through `reserve`

1. Reject zero.
2. Reject a quantity larger than available stock.
3. Increase the held count.
4. Return the remaining available quantity.

Validation happens before mutation. A failed request leaves the counts alone.
Because the quantity is checked against the remaining capacity before addition,
the held total stays within the on-hand total in this sequential example.

## 5. Your first exercise

Read `main.rs`, predict the output, and then run it:

```powershell
.\dev.ps1 run
```

Bob currently requests `2`. Predict the final held and available counts, then
run it. Change Bob's quantity to `3` and run it again to see the rejected
request. Explain why `on_hand` stays at five in both cases.

Run the tests:

```powershell
.\dev.ps1 test
```

`assert_eq!` checks that an actual result equals the expected one. Read each test
as a small story about a buyer and the shelf.

## Stop here for this lesson

This is one process changing one stock object. Coordinating two API servers or
two database transactions is a different problem, which we will test in a later
lesson. We will add features once the current rule and Rust syntax are clear.
