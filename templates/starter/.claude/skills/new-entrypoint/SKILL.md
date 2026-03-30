---
name: new-entrypoint
description: >
  Add a new entry point (public method) to an existing contract.
  Use when the user says "new entrypoint", "add method", "add entry point",
  "add function to contract", or "new-entrypoint".
---

# Add Entry Point to Existing Contract

Adds a new public method to an existing contract's `#[odra::module] impl` block.

---

## Step 1 — Identify the Contract

List available contracts:

```bash
ls contracts/src/*.rs | grep -v lib.rs
```

If the user hasn't specified which contract, show the list and ask. Do not guess.

---

## Step 2 — Gather Entry Point Details

Ask for (do not guess or infer any of these):

1. **Entry point name** — snake_case (e.g., `transfer`, `set_owner`, `withdraw`)
2. **Arguments** — name and type for each (e.g., `recipient: Address, amount: U256`)
3. **Return type** — if any (e.g., `-> U256`, `-> bool`). Default: no return (void).
4. **Mutability** — does it modify state? (`&mut self` vs `&self`)

If the user hasn't specified arguments or return type, ask explicitly. Never infer from the method name.

---

## Step 3 — Add the Entry Point

Read the contract file and add the new method inside the `#[odra::module] impl` block, after existing methods:

```rust
/// Brief description of what this entry point does.
pub fn method_name(&mut self, arg1: Type1, arg2: Type2) -> ReturnType {
    todo!("Implement method_name")
}
```

Use `&self` instead of `&mut self` if the method is read-only.

---

## Step 4 — Add a Test

Add a test in the `#[cfg(test)] mod tests` block:

```rust
#[test]
fn test_method_name() {
    let (_env, contract) = setup();
    // TODO: test the new entry point
    // contract.method_name(arg1, arg2);
}
```

---

## Step 5 — Verify

```bash
cargo test -p contracts <module_name>
```

Fix compilation errors. The test will pass since it uses `todo!()` or is a stub — the user implements the logic.

---

## Step 6 — Report

Show the user:
- What was added and where (file:line)
- Remind them to implement the `todo!()` body and fill in the test
