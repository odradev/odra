---
name: new-version
description: >
  Create an upgraded version of an existing contract with an upgrade entry point.
  Use when the user says "new version", "upgrade contract", "create v2",
  "contract upgrade", or "new-version".
---

# Create Upgraded Contract Version

## Context

Read these before proceeding:

- `.claude/context/overview/contract-model.md`
- `.claude/context/reference/storage.md`
- `.claude/context/reference/entry-points.md`

Creates an upgraded version of an existing contract. Upgrades are additive: they preserve all existing entry points and signatures, add new ones, and include an `upgrade` entry point for state migration.

Reference: `examples/src/features/upgrade.rs` in the Odra repository.

---

## Step 1 — Identify the Base Contract

List available contracts:

```bash
ls contracts/src/*.rs | grep -v lib.rs
```

Ask the user which contract to upgrade. Read its source to understand:
- All existing entry points (names, signatures, return types)
- Storage fields
- Events and errors

---

## Step 2 — Choose Upgrade Mode

Ask the user:

**Mode A — Modify existing struct**: Adds new entry points and the `upgrade` method directly to the existing contract. Simpler, but the contract file grows.

**Mode B — New struct**: Creates a new module (e.g., `CounterV2`) in a new file. Keeps versions separate. The new struct must include all storage fields from the old version (to read legacy data) plus any new fields.

---

## Step 3 — Gather New Features

Ask for:
1. **New storage fields** — any additional state the upgraded version needs
2. **New entry points** — name, args, return type for each
3. **Upgrade logic** — what should the `upgrade(&mut self, ...)` method do? Typically migrates data from old storage fields to new ones.

Remind the user: upgrades are additive. All existing entry points must remain with identical signatures. The `upgrade` method handles state migration.

---

## Step 4 — Generate the Upgraded Contract

### Mode A — Modify existing struct

Add to the existing contract file:
1. New storage fields to the struct
2. New entry points to the impl block
3. The `upgrade` method:

```rust
pub fn upgrade(&mut self, /* migration args */) {
    // Migrate state from old fields to new fields
    // e.g., self.new_counter.set(self.counter.get_or_default().into());
}
```

### Mode B — New struct

Create `contracts/src/<snake_name>_v2.rs`:

```rust
//! Upgraded version of ModuleName.

use odra::prelude::*;

#[odra::module]
pub struct ModuleNameV2 {
    // All old storage fields (same names — reads legacy data)
    old_field: Var<OldType>,
    // New storage fields
    new_field: Var<NewType>,
}

#[odra::module]
impl ModuleNameV2 {
    pub fn init(&mut self, /* args */) {
        // Initialize new fields
    }

    /// Upgrade entry point — migrates state from V1.
    pub fn upgrade(&mut self, /* migration args */) {
        // Read from old fields, write to new fields
        // e.g., self.new_field.set(self.old_field.get_or_default().into());
    }

    // All existing entry points with IDENTICAL signatures
    // (copy from V1, adjust to use new storage if needed)

    // New entry points
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::<old_module>::ModuleName;
    use odra::host::{Deployer, HostRef, InstallConfig, NoArgs};
    use odra::prelude::Addressable;

    #[test]
    fn upgrade_preserves_state() {
        let env = odra_test::env();

        // Deploy V1 with upgradability enabled
        let mut v1 = ModuleName::deploy_with_cfg(
            &env,
            ModuleNameInitArgs { /* args */ },
            InstallConfig::new::<ModuleName>(true, true)
        );

        // Perform some operations on V1
        // v1.some_method();

        // Upgrade to V2
        let mut v2 = ModuleNameV2::try_upgrade(
            &env,
            v1.address(),
            ModuleNameV2UpgradeArgs { /* migration args */ }
        ).unwrap();

        // Verify old state is accessible/migrated
        // assert_eq!(v2.get_old_data(), expected);

        // Verify new functionality works
        // v2.new_method();
    }
}
```

Register in `contracts/src/lib.rs` and `Odra.toml` (Mode B only).

---

## Step 5 — Wire into CLI

Update `cli/cli.rs` deploy script to use the upgrade pattern:

```rust
// Try to upgrade existing contract, or deploy fresh
use contracts::<module_v2>::ModuleNameV2;

let _ = ModuleNameV2::load_or_deploy(&env, args, container, 250_000_000_000)?;
```

---

## Step 6 — Verify

```bash
cargo test -p contracts <module_name>
```

---

## Step 7 — Report

Show the user:
- Files created/modified
- Explain the upgrade flow: `deploy_with_cfg` (V1, upgradable) -> `try_upgrade` (V2)
- Key concept: `InstallConfig::new::<V1>(true, true)` enables upgradability
- Key concept: `V2UpgradeArgs` are passed to the `upgrade` method, not `init`
