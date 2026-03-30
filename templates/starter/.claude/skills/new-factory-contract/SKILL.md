---
name: new-factory-contract
description: >
  Scaffold a new Odra factory contract module in the contracts crate.
  Use when the user says "new factory", "create a factory contract",
  "factory module", or "new-factory-contract".
---

# Scaffold New Factory Contract

Creates a new Odra factory contract module in the `contracts/` crate. Factory contracts can deploy and manage child contracts on-chain.

---

## Step 1 — Gather Requirements

Ask the user for (skip any they've already provided):

1. **Module name** — PascalCase (e.g., `TokenFactory`, `VaultFactory`)
2. **Brief description** — one sentence
3. **Storage fields** — same types as regular contracts: `Var<T>`, `Mapping<K, V>`, `SubModule<T>`
4. **SubModules** — composing existing modules, delegation via `delegate!`
5. **External contracts** — cross-contract calls via `External<ContractRef>`
6. **Events** — custom events (factory auto-generates a `ContractDeployed` event)
7. **Errors** — error conditions

Do not guess or infer. Ask if not provided.

---

## Step 2 — Generate the Contract File

Create `contracts/src/<snake_name>.rs`. The key difference from regular contracts is `factory=on`:

```rust
//! Brief module description.

use odra::prelude::*;

/// Module doc comment.
#[odra::module(factory=on)]
pub struct ModuleName {
    value: Var<Type>,
}

#[odra::module(factory=on)]
impl ModuleName {
    pub fn init(&mut self, /* constructor args */) {
        // Set initial state
    }

    // Public entry points
}

// events, errors modules same as regular contracts

#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::{Deployer, HostRef, NoArgs};

    #[test]
    fn test_standalone() {
        let env = odra_test::env();
        let mut contract = ModuleName::deploy(&env, ModuleNameInitArgs { /* args */ });
        // Assert initial state
    }

    // Factory tests require Casper VM (not OdraVM), so mark with #[ignore]:
    // #[test]
    // #[ignore = "Factory tests require Casper VM"]
    // fn test_factory() {
    //     let env = odra_test::env();
    //     let mut factory = ModuleNameFactory::deploy(&env, NoArgs);
    //     let (address, _access_uref) = factory.new_contract(String::from("Child1"), /* init args */);
    //     let child = ModuleNameHostRef::new(address, env);
    //     // Assert child state
    // }
}
```

**Important**: Both the struct and impl block must have `#[odra::module(factory=on)]`.

### Factory-generated types

The `factory=on` attribute generates additional types:
- `ModuleNameFactory` — factory proxy with `new_contract()` method
- `ModuleNameFactoryContractDeployed` — event emitted when a child is created
- `ModuleNameHostRef` — direct reference to a deployed instance

---

## Step 3 — Register the Contract

Same as regular contracts:

### 3a — Add to contracts/src/lib.rs

```rust
pub mod <snake_name>;
```

### 3b — Register in Odra.toml

```toml
[[contracts]]
fqn = "contracts::<snake_name>::ModuleName"
```

### 3c — Wire into CLI

In `cli/cli.rs`, use the factory deploy pattern:

1. Add import: `use contracts::<snake_name>::ModuleNameFactory;`
2. In deploy script:
   ```rust
   let _ = ModuleNameFactory::load_or_deploy(&env, NoArgs, container, 250_000_000_000)?;
   ```
3. Register: `.contract::<ModuleName>()`

---

## Step 4 — Verify

```bash
cargo test -p contracts <snake_name>
```

Note: factory-specific tests (using `new_contract()`) only work on Casper VM, not OdraVM. Standalone module tests work on both.

---

## Step 5 — Report

Show the user:
- Files created/modified
- Explain factory-specific patterns: `ModuleNameFactory::deploy()`, `factory.new_contract(name, args)`
- How to run standalone tests: `cargo test -p contracts <snake_name>`
- How to run factory tests: `cargo odra test -b casper` (requires Casper VM)
