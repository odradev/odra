---
name: new-contract
description: >
  Scaffold a new Odra contract module in the contracts crate.
  Use when the user says "new contract", "create a contract", "scaffold a module",
  "add a contract", "new module", or "new-contract".
---

# Scaffold New Contract

Creates a new Odra contract module in the `contracts/` crate.

---

## Step 1 — Gather Requirements

Ask the user for (skip any they've already provided):

1. **Module name** — PascalCase (e.g., `Counter`, `Vault`, `TokenSale`)
2. **Brief description** — one sentence describing what the contract does
3. **Storage fields** — what state it needs. Suggest appropriate types:
   - `Var<T>` for single values (e.g., `Var<u32>`, `Var<Address>`, `Var<U256>`)
   - `Mapping<K, V>` for key-value storage (e.g., `Mapping<Address, U256>`)
4. **SubModules** — composing existing modules (e.g., `SubModule<Ownable>`, `SubModule<Erc20>`)
   - If used, ask which methods to delegate via `delegate!`
5. **External contracts** — cross-contract calls via `External<ContractRef>`
   - If used, ask which external contract and which methods to call
6. **Events** — what events the contract should emit (name + fields)
7. **Errors** — what error conditions exist (name + variants)

Do not guess or infer any of these. If the user hasn't specified something, ask.

---

## Step 2 — Generate the Contract File

Create `contracts/src/<snake_name>.rs` following this structure:

```rust
//! Brief module description.

use odra::prelude::*;
// Add imports based on what's actually used:
// use odra::{casper_types::U256, Address, Mapping, SubModule, Var};
// use odra::External;

use self::errors::Error;
use self::events::*;

/// Module doc comment.
#[odra::module(events = [Event1, Event2], errors = Error)]
pub struct ModuleName {
    field_name: Var<Type>,
    // mapping_name: Mapping<Key, Value>,
    // sub_module: SubModule<OtherModule>,
}

#[odra::module]
impl ModuleName {
    /// Initialize the contract. Called once at deploy time.
    pub fn init(&mut self, /* constructor args */) {
        // Set initial state
        // Initialize SubModules if any
    }

    // Public entry points (become callable on-chain)

    // Private helpers (not pub — not exposed as entry points)
}

// If using delegate!, add the delegation block:
// delegate! {
//     to self.sub_module {
//         fn delegated_method(&self, arg: Type) -> ReturnType;
//     }
// }

pub mod events {
    use odra::prelude::*;

    #[odra::event]
    pub struct Event1 {
        pub field: Type,
    }
}

pub mod errors {
    use odra::prelude::*;

    #[odra::odra_error]
    pub enum Error {
        SomeError = 1,
        AnotherError = 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::{Deployer, HostEnv, HostRef, NoArgs};

    fn setup() -> (HostEnv, ModuleNameHostRef) {
        let env = odra_test::env();
        let args = ModuleNameInitArgs {
            // constructor args
        };
        (env.clone(), ModuleName::deploy(&env, args))
    }

    #[test]
    fn initialization_works() {
        let (_env, contract) = setup();
        // Assert initial state
    }
}
```

Adapt imports, storage fields, events, errors, and methods based on gathered requirements. Only include sections that are needed (e.g., skip `events` module if no events, skip `errors` module if no errors, skip `delegate!` if no delegation).

If the user didn't request events or errors, use the simpler `#[odra::module]` attribute without `events` or `errors` parameters.

### Key rules

- **`no_std` compatibility**: Use `odra::prelude::*` (provides `String`, `Vec`, `ToString`, etc.). Never import from `std`.
- **Imports**: Only import what's actually used. The `odra` crate re-exports `casper_types`.
- **`init` method**: Convention for constructor. Takes `&mut self` + args.
- **Generated types**: The `#[odra::module]` macro generates:
  - `ModuleNameHostRef` — host-side proxy for testing
  - `ModuleNameInitArgs` — struct for constructor arguments (if `init` has args)
  - `ModuleNameContractRef` — for external contract references
- **External contracts**: Use `External<ModuleNameContractRef>` for cross-contract calls. Store the target address in a `Var<Address>` and construct the external ref in methods.
- **SubModule initialization**: Call child module init in the parent's `init()`.
- **Tests are inline**: Always use `#[cfg(test)] mod tests` in the same file.

---

## Step 3 — Register the Contract

### 3a — Add to contracts/src/lib.rs

Add to `contracts/src/lib.rs`:

```rust
pub mod <snake_name>;
```

### 3b — Register in Odra.toml

Add to the root `Odra.toml`:

```toml
[[contracts]]
fqn = "contracts::<snake_name>::ModuleName"
```

### 3c — Wire into CLI

In `cli/cli.rs`:

1. Add import: `use contracts::<snake_name>::ModuleName;`
2. Add deployment in `ContractsDeployScript::deploy()`:
   ```rust
   let _ = ModuleName::load_or_deploy(&env, ModuleNameInitArgs { /* args */ }, container, 250_000_000_000)?;
   ```
   Use `NoArgs` instead of `ModuleNameInitArgs` if `init()` takes no arguments.
3. Register in the builder chain: `.contract::<ModuleName>()`

Add necessary imports (`DeployerExt`, `ContractProvider`, `NoArgs` if needed) to the `use` block.

---

## Step 4 — Verify

Run the tests:

```bash
cargo test -p contracts
```

Fix any compilation errors before reporting success.

---

## Step 5 — Report

Show the user:
- Files created/modified
- How to run tests: `cargo test -p contracts <snake_name>`
- How to build WASM: `cargo odra build -b casper`
