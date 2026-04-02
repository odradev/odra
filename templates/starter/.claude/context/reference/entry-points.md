# Odra Entry Points

## What Makes a Function an Entry Point

Only `pub fn` methods in the `#[odra::module]` impl block become entry points (callable
on-chain). Non-public functions are internal helpers only.

```rust
#[odra::module]
impl Counter {
    pub fn increment(&mut self) { ... }  // ✓ entry point
    pub fn get(&self) -> u32 { ... }     // ✓ entry point
    fn internal_helper(&self) { ... }    // ✗ not an entry point
}
```

## `&self` vs `&mut self`

- Use `&self` for read-only entry points (don't write to storage)
- Use `&mut self` for entry points that write to storage or emit events

## Constructor: `init`

The function named `init` is the constructor — it runs once at deploy time.
Its parameters are collected into a generated `<Name>InitArgs` struct.

```rust
// Generates: CounterInitArgs { initial_value: u32 }
pub fn init(&mut self, initial_value: u32) {
    self.value.set(initial_value);
}
```

Deploying with constructor args:

```rust
Counter::deploy(&env, CounterInitArgs { initial_value: 0 })
```

Contracts with no `init` function are deployed with `NoArgs`:

```rust
use odra::host::{Deployer, NoArgs};
Counter::deploy(&env, NoArgs)
```

## Return Types

```rust
pub fn get_count(&self) -> u32 { ... }         // returns value directly
pub fn maybe_value(&self) -> Option<String> { ... } // Option is valid
pub fn transfer(&mut self) { ... }              // () — no return needed
```

On-chain, return values are serialized via `casper_types`. Use types that implement
`CLTyped + ToBytes + FromBytes`: primitive integers, `bool`, `String`, `Address`,
`U256`, `U512`, `Vec<T>`, `Option<T>`, tuples, custom types with `#[odra::odra_type]`.

## Payable Entry Points

Mark an entry point with `#[odra(payable)]` to allow it to receive native CSPR tokens:

```rust
#[odra(payable)]
pub fn deposit(&mut self) {
    // self.env().self_balance() now includes the attached tokens
}
```

Calling a payable entry point from a test:

```rust
contract.with_tokens(U512::from(100)).deposit();
```

Calling a non-payable entry point with tokens fails and refunds the tokens.

## Accessing On-Chain Context

Inside any entry point, use `self.env()`:

```rust
pub fn do_something(&mut self) {
    let caller = self.env().caller();             // Address
    let now = self.env().get_block_time();         // u64 (milliseconds)
    let balance = self.env().self_balance();       // U512 (contract balance)
    self.env().transfer_tokens(&caller, &amount);  // send native tokens
    self.env().emit_event(MyEvent { ... });        // emit event
    self.env().revert(MyError::SomeVariant);       // revert (never returns)
}
```

## Optional Arguments

Use `Maybe<T>` (Odra's optional argument type) for truly optional init/entry args:

```rust
pub fn init(&mut self, name: String, metadata: Maybe<String>) {
    if let Some(m) = metadata.into() {
        self.metadata.set(m);
    }
}
```
