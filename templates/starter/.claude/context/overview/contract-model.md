# Odra Contract Model

## The `#[odra::module]` Macro

Apply `#[odra::module]` to both the struct and its impl block. The struct's fields are
on-chain storage. Every `pub fn` in the impl block becomes an entry point (callable on-chain).

```rust
use odra::prelude::*;

#[odra::module]
pub struct Counter {
    value: Var<u32>,
}

#[odra::module]
impl Counter {
    pub fn init(&mut self, start: u32) {
        self.value.set(start);
    }

    pub fn increment(&mut self) {
        let v = self.value.get_or_default();
        self.value.set(v + 1);
    }

    pub fn get(&self) -> u32 {
        self.value.get_or_default()
    }
}
```

## What the Macro Generates

For a contract named `Counter`, the macro generates:

| Generated type | Purpose |
|---|---|
| `CounterHostRef` | Host-side proxy — all entry points as Rust methods |
| `CounterInitArgs` | Struct built from `init`'s parameters (if `init` exists) |
| `CounterContractRef` | Used for cross-contract calls (see cross-contract.md) |
| `EntryPointsCaller` | Internal bridge from host call to contract function |

## Constructor: `init`

The function named `init` is the constructor. It runs once at deploy time.
Its parameters become fields of the generated `<Name>InitArgs` struct.

```rust
// init with parameters → generates CounterInitArgs { start: u32 }
pub fn init(&mut self, start: u32) { ... }
```

Contracts with no constructor omit `init`. When deploying, use `NoArgs`:

```rust
use odra::host::{Deployer, NoArgs};
let contract = MyContract::deploy(&env, NoArgs);
```

## Module Attributes

The struct attribute accepts optional lists:

```rust
#[odra::module(events = [TransferEvent, ApprovalEvent], errors = Error)]
pub struct Token { ... }
```

- `events` — list of event types emitted by this module (for schema generation)
- `errors` — the error enum for this module (for schema generation)

## The `self.env()` Method

Inside any entry point, `self.env()` returns a `ContractEnv` reference for accessing
on-chain context:

```rust
self.env().caller()           // Address of the caller
self.env().get_block_time()   // Current block time (milliseconds)
self.env().emit_event(e)      // Emit an event
self.env().revert(err)        // Revert with an error
self.env().self_balance()     // Contract's native token balance (U512)
self.env().transfer_tokens(&to, &amount) // Transfer native tokens
```

## SubModules

Contracts can nest other modules as fields using `SubModule<T>`:

```rust
#[odra::module]
pub struct ManagedToken {
    ownable: SubModule<Ownable>,
    token: SubModule<Erc20>,
}
```

SubModule methods are called directly: `self.ownable.get_owner()`.
To forward entry points to a SubModule, use the `delegate!` macro.
