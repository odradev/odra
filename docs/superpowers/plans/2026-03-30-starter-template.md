# Starter Template Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a `starter` template with 9 composable Claude Code skills that support both guided onboarding and direct development workflows.

**Architecture:** A workspace-based Cargo project template (contracts crate + CLI crate) with a `.claude/` directory containing a minimal CLAUDE.md and 9 skill files. Template uses cargo-generate placeholders (`{{project-name}}`, `{{date}}`). Skills are self-contained SKILL.md files following the existing Odra skill format.

**Tech Stack:** Rust, cargo-generate templates, Claude Code skills (SKILL.md format)

**Spec:** `docs/superpowers/specs/2026-03-30-onboarding-template-design.md`

---

## File Structure

```
templates/starter/
  _Cargo.toml                     # workspace manifest (members = ["contracts", "cli"])
  Odra.toml                       # empty contracts list
  rust-toolchain                  # nightly toolchain
  .gitignore                      # root gitignore
  README.md                       # project README with usage instructions
  CHANGELOG.md                    # changelog template
  .env.sample                     # livenet env vars example
  .claude/
    CLAUDE.md                     # minimal — two-mode entry point
    skills/
      check-env/SKILL.md          # validate dev environment
      onboard/SKILL.md            # guided learning orchestrator
      new-contract/SKILL.md       # scaffold regular contract
      new-factory-contract/SKILL.md # scaffold factory contract
      new-entrypoint/SKILL.md     # add entry point to existing contract
      new-version/SKILL.md        # create upgraded contract version
      new-scenario/SKILL.md       # implement CLI scenario
      start-nctl/SKILL.md         # start local Casper node
      deploy-to-livenet/SKILL.md  # deploy contracts to network
  contracts/
    Cargo.toml                    # lib crate depending on odra
    build.rs                      # odra build script
    .gitignore                    # contracts gitignore
    src/
      lib.rs                      # empty lib root
    bin/
      build_contract.rs           # wasm builder binary
      build_schema.rs             # schema generator binary
  cli/
    Cargo.toml                    # CLI crate depending on contracts + odra-cli
    cli.rs                        # minimal CLI skeleton
```

All files are new (created under `templates/starter/`). One existing file is modified: `templates/templates.json` (register the new template).

---

### Task 1: Template Scaffold — Workspace Root Files

**Files:**
- Create: `templates/starter/_Cargo.toml`
- Create: `templates/starter/Odra.toml`
- Create: `templates/starter/rust-toolchain`
- Create: `templates/starter/.gitignore`
- Create: `templates/starter/README.md`
- Create: `templates/starter/CHANGELOG.md`
- Create: `templates/starter/.env.sample`

- [ ] **Step 1: Create the workspace root _Cargo.toml**

```toml
[workspace]
members = ["contracts", "cli"]
resolver = "2"

[workspace.dependencies]
#odra_dependency
#odra_test_dependency
#odra_build_dependency
#odra_cli_dependency

[profile.release]
codegen-units = 1
lto = true

[profile.dev.package."*"]
opt-level = 3
```

Write to `templates/starter/_Cargo.toml`.

- [ ] **Step 2: Create Odra.toml**

```toml
# Register contracts here. Each [[contracts]] entry needs a fully qualified name (fqn).
# Example:
# [[contracts]]
# fqn = "contracts::my_module::MyModule"
```

Write to `templates/starter/Odra.toml`.

- [ ] **Step 3: Create rust-toolchain**

```
nightly-2026-01-01
```

Write to `templates/starter/rust-toolchain`.

- [ ] **Step 4: Create .gitignore**

```
/target
.env
```

Write to `templates/starter/.gitignore`.

- [ ] **Step 5: Create README.md**

```markdown
# {{project-name}}

An Odra smart contract project.

## Prerequisites

Install [cargo-odra](https://github.com/odradev/cargo-odra) and its dependencies.

If you have Claude Code, run `/check-env` to verify your setup.

## Getting Started

### With Claude Code (recommended)

- **New to Odra?** Run `/onboard` for a guided walkthrough.
- **Know Odra?** Use skills directly: `/new-contract`, `/new-entrypoint`, `/new-version`, etc.

### Manual

#### Build

```
cargo odra build -b casper
```

#### Test

```
cargo odra test
```

#### Test against Casper VM

```
cargo odra test -b casper
```
```

Write to `templates/starter/README.md`.

- [ ] **Step 6: Create CHANGELOG.md**

```markdown
# Changelog

Changelog for `{{project-name}}`.

## [0.1.0] - {{date}}
### Added
- Initial project scaffold.
```

Write to `templates/starter/CHANGELOG.md`.

- [ ] **Step 7: Create .env.sample**

```env
# Casper Livenet Configuration
# Copy to .env and fill in values for your target network.
#
# For local nctl node:
#   ODRA_CASPER_LIVENET_SECRET_KEY_PATH=.node-keys/secret_key.pem
#   ODRA_CASPER_LIVENET_NODE_ADDRESS=http://localhost:11101
#   ODRA_CASPER_LIVENET_EVENTS_URL=http://localhost:18101/events
#   ODRA_CASPER_LIVENET_CHAIN_NAME=casper-net-1
#
# For testnet:
#   ODRA_CASPER_LIVENET_NODE_ADDRESS=http://<node-ip>:7777
#   ODRA_CASPER_LIVENET_EVENTS_URL=http://<node-ip>:9999/events
#   ODRA_CASPER_LIVENET_CHAIN_NAME=casper-test
#
# For mainnet:
#   ODRA_CASPER_LIVENET_NODE_ADDRESS=http://<node-ip>:7777
#   ODRA_CASPER_LIVENET_EVENTS_URL=http://<node-ip>:9999/events
#   ODRA_CASPER_LIVENET_CHAIN_NAME=casper

ODRA_CASPER_LIVENET_SECRET_KEY_PATH=
ODRA_CASPER_LIVENET_NODE_ADDRESS=
ODRA_CASPER_LIVENET_EVENTS_URL=
ODRA_CASPER_LIVENET_CHAIN_NAME=
```

Write to `templates/starter/.env.sample`.

- [ ] **Step 8: Commit**

```bash
git add templates/starter/_Cargo.toml templates/starter/Odra.toml templates/starter/rust-toolchain templates/starter/.gitignore templates/starter/README.md templates/starter/CHANGELOG.md templates/starter/.env.sample
git commit -m "feat(starter): add workspace root files for starter template"
```

---

### Task 2: Template Scaffold — Contracts Crate

**Files:**
- Create: `templates/starter/contracts/Cargo.toml`
- Create: `templates/starter/contracts/build.rs`
- Create: `templates/starter/contracts/.gitignore`
- Create: `templates/starter/contracts/src/lib.rs`
- Create: `templates/starter/contracts/bin/build_contract.rs`
- Create: `templates/starter/contracts/bin/build_schema.rs`

- [ ] **Step 1: Create contracts/Cargo.toml**

```toml
[package]
edition = "2021"
name = "contracts"
version = "0.1.0"

[dependencies]
odra = { workspace = true }

[dev-dependencies]
odra-test = { workspace = true }

[build-dependencies]
odra-build = { workspace = true }

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
odra-build = { workspace = true }

[[bin]]
name = "{{project-name}}_build_contract"
path = "bin/build_contract.rs"
test = false

[[bin]]
name = "{{project-name}}_build_schema"
path = "bin/build_schema.rs"
test = false
```

Write to `templates/starter/contracts/Cargo.toml`.

- [ ] **Step 2: Create contracts/build.rs**

```rust
//! Odra's contracts build script.

/// Uses the ENV variable `ODRA_MODULE` to set the `odra_module` cfg flag.
pub fn main() {
    odra_build::build();
}
```

Write to `templates/starter/contracts/build.rs`.

- [ ] **Step 3: Create contracts/.gitignore**

```
.idea
.vscode
/target
Cargo.lock
.backend*
.builder*
/wasm
.env
```

Write to `templates/starter/contracts/.gitignore`.

- [ ] **Step 4: Create contracts/src/lib.rs**

```rust
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
extern crate alloc;
```

Write to `templates/starter/contracts/src/lib.rs`.

- [ ] **Step 5: Create contracts/bin/build_contract.rs**

```rust
#![doc = "Binary for building wasm files from odra contracts."]
#![no_std]
#![cfg_attr(target_arch = "wasm32", no_main)]
#![allow(unused_imports, clippy::single_component_path_imports)]
use contracts;

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
```

Write to `templates/starter/contracts/bin/build_contract.rs`.

- [ ] **Step 6: Create contracts/bin/build_schema.rs**

```rust
#![doc = "Binary for building schema definitions from odra contracts."]
#![allow(unused_imports, redundant_imports)]
#![allow(clippy::single_component_path_imports)]
use contracts;

#[cfg(all(not(odra_module = ""), not(target_arch = "wasm32")))]
extern "Rust" {
    fn module_schema() -> odra::contract_def::ContractBlueprint;
    fn casper_contract_schema() -> odra::schema::casper_contract_schema::ContractSchema;
}

#[cfg(all(not(odra_module = ""), not(target_arch = "wasm32")))]
fn main() {
    odra_build::schema(unsafe { crate::module_schema() }, unsafe {
        crate::casper_contract_schema()
    });
}

#[cfg(any(odra_module = "", target_arch = "wasm32"))]
fn main() {}
```

Write to `templates/starter/contracts/bin/build_schema.rs`.

- [ ] **Step 7: Commit**

```bash
git add templates/starter/contracts/
git commit -m "feat(starter): add contracts crate scaffold"
```

---

### Task 3: Template Scaffold — CLI Crate

**Files:**
- Create: `templates/starter/cli/Cargo.toml`
- Create: `templates/starter/cli/cli.rs`

- [ ] **Step 1: Create cli/Cargo.toml**

```toml
[package]
edition = "2021"
name = "{{project-name}}-cli"
version = "0.1.0"

[dependencies]
contracts = { path = "../contracts" }
odra = { workspace = true }
odra-cli = { workspace = true }

[[bin]]
name = "{{project-name}}_cli"
path = "cli.rs"
```

Write to `templates/starter/cli/Cargo.toml`.

- [ ] **Step 2: Create cli/cli.rs**

```rust
//! CLI tool for deploying and interacting with smart contracts.

use odra::host::HostEnv;
use odra_cli::{
    deploy::DeployScript,
    DeployedContractsContainer, OdraCli,
};

/// Deploys all project contracts.
pub struct ContractsDeployScript;

impl DeployScript for ContractsDeployScript {
    fn deploy(
        &self,
        _env: &HostEnv,
        _container: &mut DeployedContractsContainer,
    ) -> Result<(), odra_cli::deploy::Error> {
        // Add contract deployments here using:
        //   let _ = MyContract::load_or_deploy(&env, args, container, 250_000_000_000)?;
        Ok(())
    }
}

/// Main function to run the CLI tool.
pub fn main() {
    OdraCli::new()
        .about("CLI tool for {{project-name}} smart contracts")
        .deploy(ContractsDeployScript)
        // Register contracts with .contract::<MyContract>()
        // Register scenarios with .scenario(MyScenario)
        .build()
        .run();
}
```

Write to `templates/starter/cli/cli.rs`.

- [ ] **Step 3: Commit**

```bash
git add templates/starter/cli/
git commit -m "feat(starter): add CLI crate scaffold"
```

---

### Task 4: Register Starter Template

**Files:**
- Modify: `templates/templates.json`

- [ ] **Step 1: Add starter entry to templates.json**

Add this entry to the JSON array in `templates/templates.json`, after the existing `workspace` entry:

```json
{
  "name": "starter",
  "description": "Project template with workspace layout and Claude Code skills for guided development",
  "path": "templates/starter",
  "template_type": "Project"
}
```

- [ ] **Step 2: Commit**

```bash
git add templates/templates.json
git commit -m "feat(starter): register starter template in templates.json"
```

---

### Task 5: CLAUDE.md

**Files:**
- Create: `templates/starter/.claude/CLAUDE.md`

- [ ] **Step 1: Create .claude/CLAUDE.md**

```markdown
# {{project-name}}

An Odra smart contract workspace. Contracts live in `contracts/`, the CLI for deployment in `cli/`.

## Getting Started

**New to Odra?** Run `/onboard` for a guided walkthrough — from writing your first contract to deploying it.

**Know Odra?** Use skills directly:

| Skill | What it does |
|---|---|
| `/check-env` | Verify your development environment |
| `/new-contract` | Scaffold a new contract module |
| `/new-factory-contract` | Scaffold a factory contract |
| `/new-entrypoint` | Add an entry point to an existing contract |
| `/new-version` | Create an upgraded contract version |
| `/new-scenario` | Add a CLI deployment/interaction scenario |
| `/start-nctl` | Start a local Casper node via Docker |
| `/deploy-to-livenet` | Deploy contracts to nctl, testnet, or mainnet |

## Project Structure

- `contracts/src/` — contract modules (one `.rs` file per contract)
- `cli/cli.rs` — CLI for deploying and interacting with contracts on livenet
- `Odra.toml` — contract registry for `cargo odra`
- `.env.sample` — livenet environment variable template

## Commands

```bash
cargo odra test              # test on OdraVM (fast, in-memory)
cargo odra test -b casper    # test on Casper VM (full execution engine)
cargo odra build -b casper   # build WASM binaries
```
```

Write to `templates/starter/.claude/CLAUDE.md`.

- [ ] **Step 2: Commit**

```bash
git add templates/starter/.claude/CLAUDE.md
git commit -m "feat(starter): add CLAUDE.md with two-mode entry point"
```

---

### Task 6: Skill — check-env

**Files:**
- Create: `templates/starter/.claude/skills/check-env/SKILL.md`

- [ ] **Step 1: Create the skill file**

```markdown
---
name: check-env
description: >
  Validate the development environment for Odra smart contract development.
  Use when the user says "check env", "check setup", "check prerequisites",
  "verify environment", or "check-env".
allowed-tools: Bash(rustc *),Bash(rustup *),Bash(cargo odra *),Bash(wasm-opt *),Bash(wasm-strip *),Bash(docker *)
---

# Check Development Environment

Validates that all prerequisites for Odra development are installed.

---

## Step 1 — Check Each Prerequisite

Run these checks and collect results:

### Rust nightly toolchain

```bash
rustc --version
```

Compare against the version in `rust-toolchain` file. If the required nightly is not installed:
- Install: `rustup toolchain install <version>`

### wasm32-unknown-unknown target

```bash
rustup target list --installed | grep wasm32-unknown-unknown
```

If missing:
- Install: `rustup target add wasm32-unknown-unknown`

### cargo-odra

```bash
cargo odra --version
```

If missing:
- Install: `cargo install cargo-odra --git https://github.com/odradev/cargo-odra --locked`

### wasm-opt (binaryen)

```bash
wasm-opt --version
```

If missing:
- macOS: `brew install binaryen`
- Linux: download from https://github.com/WebAssembly/binaryen/releases

### wasm-strip (wabt)

```bash
wasm-strip --version
```

If missing:
- macOS: `brew install wabt`
- Linux: `sudo apt install wabt`

### Docker (optional)

```bash
docker --version
```

If missing, note it is optional — only needed for local NCTL node testing.
- Install from https://docs.docker.com/get-docker/

---

## Step 2 — Report Results

Present a summary table:

```
| Prerequisite             | Status | Action needed         |
|--------------------------|--------|-----------------------|
| Rust nightly (YYYY-MM-DD)| OK/MISSING | install command    |
| wasm32-unknown-unknown   | OK/MISSING | install command    |
| cargo-odra               | OK/MISSING | install command    |
| wasm-opt (binaryen)      | OK/MISSING | install command    |
| wasm-strip (wabt)        | OK/MISSING | install command    |
| Docker (optional)        | OK/MISSING | install link       |
```

If everything is OK, report: "Environment is ready for Odra development."

If items are missing, list the install commands and explain what each tool is for:
- **Rust nightly**: required compiler for Odra contracts
- **wasm32-unknown-unknown**: WebAssembly compilation target for smart contracts
- **cargo-odra**: Odra's build/test tool — wraps cargo with WASM compilation steps
- **wasm-opt**: optimizes WASM binaries for smaller contract size
- **wasm-strip**: strips debug info from WASM binaries
- **Docker**: runs a local Casper blockchain node for testing deployments
```

Write to `templates/starter/.claude/skills/check-env/SKILL.md`.

- [ ] **Step 2: Commit**

```bash
git add templates/starter/.claude/skills/check-env/
git commit -m "feat(starter): add check-env skill"
```

---

### Task 7: Skill — new-contract

**Files:**
- Create: `templates/starter/.claude/skills/new-contract/SKILL.md`

- [ ] **Step 1: Create the skill file**

```markdown
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
```

Write to `templates/starter/.claude/skills/new-contract/SKILL.md`.

- [ ] **Step 2: Commit**

```bash
git add templates/starter/.claude/skills/new-contract/
git commit -m "feat(starter): add new-contract skill"
```

---

### Task 8: Skill — new-factory-contract

**Files:**
- Create: `templates/starter/.claude/skills/new-factory-contract/SKILL.md`

- [ ] **Step 1: Create the skill file**

```markdown
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
```

Write to `templates/starter/.claude/skills/new-factory-contract/SKILL.md`.

- [ ] **Step 2: Commit**

```bash
git add templates/starter/.claude/skills/new-factory-contract/
git commit -m "feat(starter): add new-factory-contract skill"
```

---

### Task 9: Skill — new-entrypoint

**Files:**
- Create: `templates/starter/.claude/skills/new-entrypoint/SKILL.md`

- [ ] **Step 1: Create the skill file**

```markdown
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
```

Write to `templates/starter/.claude/skills/new-entrypoint/SKILL.md`.

- [ ] **Step 2: Commit**

```bash
git add templates/starter/.claude/skills/new-entrypoint/
git commit -m "feat(starter): add new-entrypoint skill"
```

---

### Task 10: Skill — new-version

**Files:**
- Create: `templates/starter/.claude/skills/new-version/SKILL.md`

- [ ] **Step 1: Create the skill file**

```markdown
---
name: new-version
description: >
  Create an upgraded version of an existing contract with an upgrade entry point.
  Use when the user says "new version", "upgrade contract", "create v2",
  "contract upgrade", or "new-version".
---

# Create Upgraded Contract Version

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
- Explain the upgrade flow: `deploy_with_cfg` (V1, upgradable) → `try_upgrade` (V2)
- Key concept: `InstallConfig::new::<V1>(true, true)` enables upgradability
- Key concept: `V2UpgradeArgs` are passed to the `upgrade` method, not `init`
```

Write to `templates/starter/.claude/skills/new-version/SKILL.md`.

- [ ] **Step 2: Commit**

```bash
git add templates/starter/.claude/skills/new-version/
git commit -m "feat(starter): add new-version skill"
```

---

### Task 11: Skill — new-scenario

**Files:**
- Create: `templates/starter/.claude/skills/new-scenario/SKILL.md`

- [ ] **Step 1: Create the skill file**

```markdown
---
name: new-scenario
description: >
  Implement a new CLI scenario for interacting with deployed contracts.
  Use when the user says "new scenario", "add scenario", "cli scenario",
  "deployment scenario", or "new-scenario".
---

# Implement New CLI Scenario

Creates a new scenario in `cli/cli.rs` for interacting with deployed contracts on livenet.

---

## Step 1 — Gather Requirements

Ask the user for (skip any already provided):

1. **Scenario name** — PascalCase (e.g., `TransferTokens`, `MintNft`)
2. **Description** — one sentence explaining what the scenario does
3. **Which contracts** it interacts with (must already exist in the project)
4. **Arguments** — CLI arguments the scenario accepts:
   - Name, type, description, and whether required or optional
   - Available types: `NamedCLType::U64`, `NamedCLType::U256`, `NamedCLType::Key`, `NamedCLType::String`, etc.

Do not guess which contracts or arguments are needed.

---

## Step 2 — Generate the Scenario

Add to `cli/cli.rs`:

### Import the contracts used

```rust
use contracts::<module>::ModuleName;
```

### Implement the scenario struct

```rust
/// Description of what this scenario does.
pub struct ScenarioName;

impl Scenario for ScenarioName {
    fn args(&self) -> Vec<CommandArg> {
        vec![
            CommandArg::new("arg_name", "Argument description", NamedCLType::U256).required(),
            CommandArg::new("optional_arg", "Optional arg description", NamedCLType::Key),
        ]
    }

    fn run(
        &self,
        env: &HostEnv,
        container: &DeployedContractsContainer,
        args: Args,
    ) -> Result<(), Error> {
        let mut contract = container.contract_ref::<ModuleName>(env)?;

        // Read arguments
        let value = args.get_single::<U256>("arg_name")?;

        // Set gas for the operation
        env.set_gas(50_000_000);

        // Call contract methods
        contract.try_some_method(&value)?;

        Ok(())
    }
}

impl ScenarioMetadata for ScenarioName {
    const NAME: &'static str = "scenario_name_snake_case";
    const DESCRIPTION: &'static str = "Description of what this scenario does.";
}
```

### Register in the builder chain

Add `.scenario(ScenarioName)` to the `OdraCli::new()` builder in `main()`.

### Add necessary imports

Ensure these are in the `use` block:

```rust
use odra_cli::{
    scenario::{Args, Error, Scenario, ScenarioMetadata},
    CommandArg,
};
```

---

## Step 3 — Verify

```bash
cargo build -p {{project-name}}-cli
```

---

## Step 4 — Report

Show the user:
- What was added to `cli/cli.rs`
- How to run: deploy contracts first, then `cargo run --bin {{project-name}}_cli -- scenario_name --arg_name value`
```

Write to `templates/starter/.claude/skills/new-scenario/SKILL.md`.

- [ ] **Step 2: Commit**

```bash
git add templates/starter/.claude/skills/new-scenario/
git commit -m "feat(starter): add new-scenario skill"
```

---

### Task 12: Skill — start-nctl

**Files:**
- Create: `templates/starter/.claude/skills/start-nctl/SKILL.md`
- Create: `templates/starter/.claude/skills/start-nctl/scripts/wait-for-nctl.sh`
- Create: `templates/starter/.claude/skills/start-nctl/scripts/extract-keys.sh`

- [ ] **Step 1: Create the wait-for-nctl.sh script**

```bash
#!/bin/bash
# Wait for the NCTL node to be ready by polling the RPC endpoint.
# Usage: ./wait-for-nctl.sh [timeout_seconds]
# Default timeout: 120 seconds

TIMEOUT=${1:-120}
ELAPSED=0
INTERVAL=3
RPC_URL="http://localhost:11101/rpc"

echo "Waiting for NCTL node at $RPC_URL (timeout: ${TIMEOUT}s)..."

while [ $ELAPSED -lt $TIMEOUT ]; do
  if curl -s -m 2 "$RPC_URL" \
    -H 'content-type: application/json' \
    --data-raw '{"jsonrpc":"2.0","id":"1","method":"info_get_status"}' \
    2>/dev/null | grep -q '"jsonrpc"'; then
    echo "NCTL node is ready (after ${ELAPSED}s)."
    exit 0
  fi
  sleep $INTERVAL
  ELAPSED=$((ELAPSED + INTERVAL))
done

echo "ERROR: NCTL node did not become ready within ${TIMEOUT}s."
exit 1
```

Write to `templates/starter/.claude/skills/start-nctl/scripts/wait-for-nctl.sh`.

- [ ] **Step 2: Create the extract-keys.sh script**

```bash
#!/bin/bash
# Extract Casper user keys from the Docker NCTL container.

project_root=$(git rev-parse --show-toplevel)
cd "$project_root"

mkdir -p .node-keys

docker exec mynctl /bin/bash -c \
  "cat /home/casper/casper-nctl/assets/net-1/users/user-1/secret_key.pem" \
  > .node-keys/secret_key.pem

docker exec mynctl /bin/bash -c \
  "cat /home/casper/casper-nctl/assets/net-1/users/user-2/secret_key.pem" \
  > .node-keys/secret_key_1.pem
```

Write to `templates/starter/.claude/skills/start-nctl/scripts/extract-keys.sh`.

- [ ] **Step 3: Create the skill file**

```markdown
---
name: start-nctl
description: >
  Start a local Casper NCTL node using Docker, wait for readiness, and extract keys.
  Use when the user says "start nctl", "start node", "start local node",
  "run nctl", or "start-nctl".
allowed-tools: Bash(docker *),Bash(chmod *),Bash(curl *),Bash(wc *),Bash(.claude/skills/start-nctl/scripts/*)
---

# Start Local Casper Node (NCTL)

Starts a local Casper blockchain node via Docker for testing contract deployments.

---

## Step 1 — Check Docker

```bash
docker --version
```

If Docker is not installed or not running, tell the user and stop.

---

## Step 2 — Check if NCTL is Already Running

```bash
docker ps --filter name=mynctl --format '{{.Names}}'
```

If `mynctl` is listed, skip to Step 4.

---

## Step 3 — Start NCTL

```bash
docker run --rm -it --cpus=1 --name mynctl -d -p 11101:11101 -p 14101:14101 -p 18101:18101 -p 25101:25101 makesoftware/casper-nctl:v203
```

Wait for readiness:

```bash
chmod +x .claude/skills/start-nctl/scripts/wait-for-nctl.sh
.claude/skills/start-nctl/scripts/wait-for-nctl.sh 120
```

If the wait script exits non-zero, report failure and stop.

---

## Step 4 — Extract Keys

```bash
chmod +x .claude/skills/start-nctl/scripts/extract-keys.sh
.claude/skills/start-nctl/scripts/extract-keys.sh
```

Verify keys are non-empty:

```bash
wc -c .node-keys/secret_key.pem .node-keys/secret_key_1.pem
```

If either file is 0 bytes, report error and stop.

---

## Step 5 — Create .env

Create `.env` from `.env.sample` with nctl defaults:

```env
ODRA_CASPER_LIVENET_SECRET_KEY_PATH=.node-keys/secret_key.pem
ODRA_CASPER_LIVENET_NODE_ADDRESS=http://localhost:11101
ODRA_CASPER_LIVENET_EVENTS_URL=http://localhost:18101/events
ODRA_CASPER_LIVENET_CHAIN_NAME=casper-net-1
```

---

## Step 6 — Report

```
NCTL node is running.
- RPC: http://localhost:11101
- Events: http://localhost:18101/events
- Chain: casper-net-1
- Keys: .node-keys/secret_key.pem, .node-keys/secret_key_1.pem
- .env: configured for nctl

To stop: docker stop mynctl
```
```

Write to `templates/starter/.claude/skills/start-nctl/SKILL.md`.

- [ ] **Step 4: Commit**

```bash
git add templates/starter/.claude/skills/start-nctl/
git commit -m "feat(starter): add start-nctl skill with helper scripts"
```

---

### Task 13: Skill — deploy-to-livenet

**Files:**
- Create: `templates/starter/.claude/skills/deploy-to-livenet/SKILL.md`

- [ ] **Step 1: Create the skill file**

```markdown
---
name: deploy-to-livenet
description: >
  Deploy contracts to a Casper network (nctl, testnet, or mainnet).
  Use when the user says "deploy", "deploy to livenet", "deploy to nctl",
  "deploy to testnet", "deploy to mainnet", "run on livenet", or "deploy-to-livenet".
allowed-tools: Bash(docker ps:*),Bash(curl *),Bash(jq *),Bash(casper-client *),Bash(set -a && source *),Bash(cargo run --bin * --features=livenet),Bash(wc *)
---

# Deploy Contracts to Livenet

Deploys contracts using the CLI binary against a Casper network.

---

## Step 1 — Identify Target Network

If not stated, ask the user:
- **nctl** — local Docker node
- **testnet** — Casper test network
- **mainnet** — Casper main network

---

## Step 2 — Ensure Environment is Ready

### For nctl

Check if NCTL is running:

```bash
docker ps --filter name=mynctl --format '{{.Names}}'
```

If not running, tell the user to run `/start-nctl` first.

Check `.env` exists and has nctl values. If not, suggest running `/start-nctl` which creates it.

### For testnet

Check if `.env.testnet` exists. If not, create it:

1. Discover a responsive node:
   ```bash
   curl -s 'https://node.testnet.cspr.cloud/rpc' \
     -H 'content-type: application/json' \
     --data-raw '{"jsonrpc":"2.0","id":"1","method":"info_get_peers"}' \
     | jq -r '.result.peers[:10][] | .address' | head -5
   ```

2. Test each node until one responds:
   ```bash
   casper-client list-rpcs -n http://<node_ip>:7777/rpc
   ```

3. Ask the user for their secret key path (must be a `.pem` file on disk). Verify it exists.

4. Write `.env.testnet`:
   ```env
   ODRA_CASPER_LIVENET_SECRET_KEY_PATH=<user-provided-path>
   ODRA_CASPER_LIVENET_NODE_ADDRESS=http://<responsive-node-ip>:7777
   ODRA_CASPER_LIVENET_EVENTS_URL=http://<responsive-node-ip>:9999/events
   ODRA_CASPER_LIVENET_CHAIN_NAME=casper-test
   ```

### For mainnet

Same flow as testnet but:
- Use `https://node.cspr.cloud/rpc` for peer discovery
- Chain name: `casper`
- Write to `.env.mainnet`

---

## Step 3 — Identify the CLI Binary

```bash
grep -A2 '\[\[bin\]\]' cli/Cargo.toml | grep 'name'
```

The CLI binary is typically `{{project-name}}_cli`.

---

## Step 4 — Run Deployment

Load the env file and run:

```bash
set -a && source .env && set +a  # or .env.testnet / .env.mainnet
cargo run --bin <CLI_BINARY> --features=livenet 2>&1
```

**Important:**
- The `--features=livenet` flag is mandatory
- Deployments can take 30-90 seconds per contract
- Use a generous timeout (10 minutes)

---

## Step 5 — Report Results

After execution, report:

1. **Exit status**: success or failure with exit code
2. **Key events**: lines containing `Deploying`, `Deploy hash`, `Contract`, `Success`, `Done`
3. **Errors**: lines containing `panicked`, `assertion failed`, `error[`, `Error`
4. **Verdict**: one-line summary

```
### Deployment Summary
**Network**: nctl/testnet/mainnet   **Exit code**: 0

#### Key Events
- Deployed MyContract at hash abc123...
- ...

#### Errors
None

#### Verdict
All contracts deployed successfully to nctl.
```
```

Write to `templates/starter/.claude/skills/deploy-to-livenet/SKILL.md`.

- [ ] **Step 2: Commit**

```bash
git add templates/starter/.claude/skills/deploy-to-livenet/
git commit -m "feat(starter): add deploy-to-livenet skill"
```

---

### Task 14: Skill — onboard

**Files:**
- Create: `templates/starter/.claude/skills/onboard/SKILL.md`

- [ ] **Step 1: Create the skill file**

```markdown
---
name: onboard
description: >
  Guided onboarding for Odra smart contract development. Walks through writing,
  testing, and deploying your first contract step by step with explanations.
  Use when the user says "onboard", "get started", "tutorial", "learn odra",
  "teach me", or "onboard me".
---

# Odra Onboarding

A guided walkthrough for writing, testing, and deploying your first Odra smart contract. Each step explains the concepts behind what you're doing.

---

## Step 1 — Environment Check

Tell the user:

> Let's make sure your development environment is set up. I'll check for all the required tools.

Invoke `/check-env`. If any required tools are missing, help the user install them before continuing. Docker is optional at this stage — it's only needed for the deployment step later.

---

## Step 2 — Your First Contract

### 2a — Explain the Odra model

Before generating code, explain to the user:

> **How Odra contracts work:**
>
> An Odra contract is a Rust struct marked with `#[odra::module]`. The struct's fields are your on-chain storage — they persist between calls. Each `pub fn` in the impl block becomes an **entry point** — a function that can be called on-chain.
>
> Odra uses two special storage types:
> - `Var<T>` — stores a single value (like a variable)
> - `Mapping<K, V>` — stores key-value pairs (like a hash map)
>
> The `init` method is the constructor — it runs once when the contract is deployed.

### 2b — Gather requirements

Ask the user what kind of contract they want to build. Suggest simple examples if they're unsure:
- A counter that increments and stores a value
- A greeting contract that stores and returns a message
- A simple token with balances

### 2c — Generate the contract

Invoke `/new-contract` with the user's requirements.

### 2d — Explain what was generated

After generation, walk through the generated code and explain:

> Here's what each part does:
>
> - **`#[odra::module]` on the struct** — tells Odra this is a contract. It generates a `HostRef` (for testing), `InitArgs` (constructor arguments), and `ContractRef` (for cross-contract calls).
> - **`#[odra::module]` on the impl** — marks public methods as entry points.
> - **`Var<T>` / `Mapping<K, V>`** — these don't store data in memory. They read/write to the blockchain's key-value store.
> - **`self.value.set(x)` / `self.value.get_or_default()`** — how you write and read storage.
> - **The test** — deploys the contract in-memory (OdraVM) and calls its methods. No blockchain needed.

---

## Step 3 — Test Your Contract

### 3a — Explain testing

> **Testing in Odra:**
>
> Odra has a fast in-memory VM called **OdraVM** for unit tests. It simulates the blockchain without actually running one — tests execute in milliseconds.
>
> `odra_test::env()` creates a test environment. `ModuleName::deploy(&env, args)` deploys your contract into this environment. Then you call methods on the returned `HostRef` just like regular Rust method calls.

### 3b — Run the tests

```bash
cargo odra test
```

If tests fail, help the user fix the issues. Explain any errors in context.

### 3c — Celebrate

> Your contract compiles and passes tests. You've written a working Odra smart contract.

---

## Step 4 — Deploy to a Local Node (Optional)

Ask the user:

> Want to deploy your contract to a real (local) Casper blockchain node? This requires Docker and takes a few minutes to set up. You can skip this and come back later.

If they want to continue:

### 4a — Explain what's happening

> **From OdraVM to a real node:**
>
> OdraVM is great for testing logic, but a real deployment compiles your contract to WebAssembly (WASM) and sends it to a Casper node. The local node (NCTL) is an actual Casper blockchain running in Docker — same software as mainnet, just on your machine.
>
> The CLI tool (`cli/cli.rs`) handles deployment. The deploy script you saw earlier tells it which contracts to deploy and with what arguments.

### 4b — Build WASM

```bash
cargo odra build -b casper
```

Explain: this compiles contracts to `.wasm` files in the `wasm/` directory.

### 4c — Start the node

Invoke `/start-nctl`.

### 4d — Deploy

Invoke `/deploy-to-livenet` targeting nctl.

### 4e — Explain what happened

> Your contract is now deployed on a local Casper blockchain. The deploy hash is a unique identifier for this deployment. The CLI stored the contract address so future runs can interact with it without redeploying.

---

## Step 5 — What's Next

> **You've completed the Odra onboarding.** Here's what you can do next:
>
> - `/new-contract` — add more contracts to your project
> - `/new-entrypoint` — add methods to existing contracts
> - `/new-version` — create an upgraded version of a contract
> - `/new-factory-contract` — create a factory that deploys child contracts
> - `/new-scenario` — add CLI scenarios for interacting with deployed contracts
> - `/deploy-to-livenet` — deploy to testnet or mainnet
>
> Run any skill directly — no guided mode needed. You're ready.
```

Write to `templates/starter/.claude/skills/onboard/SKILL.md`.

- [ ] **Step 2: Commit**

```bash
git add templates/starter/.claude/skills/onboard/
git commit -m "feat(starter): add onboard skill — guided learning orchestrator"
```

---

### Task 15: Verify Template Structure and Update Workspace Exclusion

**Files:**
- Modify: `Cargo.toml` (root workspace — exclude the new template directory)

- [ ] **Step 1: Verify template directory structure**

```bash
find templates/starter -type f | sort
```

Expected output should match the file structure defined at the top of this plan. Verify all 24 files are present.

- [ ] **Step 2: Check root Cargo.toml excludes starter template**

Read `Cargo.toml` at the project root. The `[workspace]` section has an `exclude` list with other template directories. Add `"templates/starter"` to the exclude list if present.

If there is no exclude list but template directories are already excluded by not being in the members list, no change is needed.

- [ ] **Step 3: Verify templates.json is valid JSON**

```bash
python3 -c "import json; json.load(open('templates/templates.json'))"
```

- [ ] **Step 4: Commit if any changes were made**

```bash
git add Cargo.toml
git commit -m "chore: exclude starter template from workspace"
```

---

### Task 16: End-to-End Smoke Test

- [ ] **Step 1: Verify all skill files have valid frontmatter**

For each skill file in `templates/starter/.claude/skills/*/SKILL.md`, verify:
- Has `---` frontmatter delimiters
- Has `name:` field
- Has `description:` field

```bash
for f in templates/starter/.claude/skills/*/SKILL.md; do
  echo "=== $f ==="
  head -10 "$f"
  echo
done
```

- [ ] **Step 2: Verify template placeholder usage**

Check that `{{project-name}}` and `{{date}}` placeholders are used correctly:

```bash
grep -r '{{project-name}}\|{{date}}' templates/starter/
```

Expected locations: `_Cargo.toml` (none — workspace has no package name), `cli/Cargo.toml`, `cli/cli.rs`, `README.md`, `CHANGELOG.md`, `contracts/Cargo.toml` (none — uses fixed name "contracts").

- [ ] **Step 3: Verify no broken cross-references**

Check that skill files don't reference paths outside the template:

```bash
grep -r 'examples/' templates/starter/.claude/skills/ || echo "No external path references - OK"
grep -r 'modules/' templates/starter/.claude/skills/ || echo "No external path references - OK"
```

The `new-version` skill references `examples/src/features/upgrade.rs` as a documentation reference (not a path the skill reads) — this is acceptable.

- [ ] **Step 4: Final commit if any fixes were needed**

```bash
git add templates/starter/
git commit -m "fix(starter): address smoke test findings"
```
