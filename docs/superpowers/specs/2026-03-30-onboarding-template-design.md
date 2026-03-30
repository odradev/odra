# Starter Template Design

**Date**: 2026-03-30
**Status**: Approved

## Goal

Create a `starter` template for Odra — the go-to project scaffold for both newcomers and experienced developers. Scaffolded via `cargo odra new --template starter`.

The template ships with composable Claude Code skills that support two usage modes:

- **Guided mode** (`/onboard`) — for newcomers. A step-by-step learning path with verbose explanations, baby steps, and progressive concept introduction. Culminates in deploying the first contract.
- **Development mode** (individual skills) — for developers who know Odra. Use `/new-contract`, `/new-entrypoint`, `/new-version`, etc. directly to pave the development workflow.

Both modes use the same skill set. The `onboard` skill orchestrates the others with extra teaching between steps.

## Target Audience

Any developer working with Odra — from first-timers to experienced users. The skills adapt: newcomers get guided through `/onboard`, experienced developers invoke skills directly.

## Design Principles

- **Skills-first**: CLAUDE.md is minimal (~20 lines). All guidance lives in small, composable skills.
- **Two modes, one skill set**: `onboard` adds verbosity and teaching around the same skills used in development mode.
- **Progressive**: OdraVM testing first, NCTL deployment optional.
- **Workspace layout**: Multi-crate structure (contracts + CLI), extensible for future backend/frontend crates.
- **No guessing**: Skills ask for information they need; they never infer contract names, arguments, or types.

---

## Template Layout

```
templates/starter/
  _Cargo.toml              # workspace: members = ["contracts", "cli"]
  Odra.toml                # empty [[contracts]] list
  rust-toolchain
  .gitignore
  README.md
  CHANGELOG.md
  .env.sample              # example livenet env vars (documented)
  .claude/
    CLAUDE.md
    skills/
      check-env/SKILL.md
      onboard/SKILL.md
      new-contract/SKILL.md
      new-factory-contract/SKILL.md
      new-entrypoint/SKILL.md
      new-version/SKILL.md
      new-scenario/SKILL.md
      start-nctl/SKILL.md
      deploy-to-livenet/SKILL.md
  contracts/
    Cargo.toml             # lib crate, depends on odra
    build.rs
    .gitignore
    src/
      lib.rs               # empty
    bin/
      build_contract.rs
      build_schema.rs
  cli/
    Cargo.toml             # depends on contracts crate + odra-cli
    cli.rs                 # minimal skeleton
```

### `.env.sample`

```env
# Casper Livenet Configuration
# Copy to .env and fill in values for your target network.

# Path to your secret key PEM file
ODRA_CASPER_LIVENET_SECRET_KEY_PATH=

# Node RPC address (e.g., http://localhost:11101 for nctl)
ODRA_CASPER_LIVENET_NODE_ADDRESS=

# SSE events URL (e.g., http://localhost:18101/events for nctl)
ODRA_CASPER_LIVENET_EVENTS_URL=

# Chain name (e.g., casper-net-1 for nctl, casper-test for testnet)
ODRA_CASPER_LIVENET_CHAIN_NAME=
```

### `.claude/CLAUDE.md`

Minimal file (~20 lines):
- Describes the project as an Odra smart contract workspace
- Two-path entry point:
  - **New to Odra?** Run `/onboard` for a guided walkthrough
  - **Know Odra?** Use skills directly: `/new-contract`, `/new-entrypoint`, `/new-version`, etc.
- Lists the 9 available skills and one-line descriptions
- No tutorial content — that lives in the skills

---

## Skills

### Category: Environment & Orchestration

#### 1. `check-env` — Validate development environment

Checks the full cargo-odra prerequisite chain:

| Prerequisite | Required | How to check |
|---|---|---|
| Rust (nightly) | yes | `rustc --version`, matches `rust-toolchain` file |
| `wasm32-unknown-unknown` target | yes | `rustup target list --installed` |
| `cargo-odra` | yes | `cargo odra --version` |
| `wasm-opt` (binaryen) | yes | `wasm-opt --version` |
| `wasm-strip` (wabt) | yes | `wasm-strip --version` |
| Docker | optional | `docker --version` |

Reports missing items with platform-appropriate install commands (macOS: brew, Linux: apt/wget). Non-blocking: tells user what's missing and what it's needed for.

#### 2. `onboard` — Guided learning path (newcomer mode)

Orchestrates the same skills used in development mode, but wraps each step with:
- **Concept explanations** — what Odra storage types are, how entry points work, what the VM does
- **Baby steps** — breaks each skill invocation into smaller, explained chunks
- **Checkpoints** — confirms understanding before moving on

Progressive walkthrough:

1. **Environment check** — invokes `check-env`
2. **Write your first contract** — invokes `new-contract`, explains each concept as it appears
3. **Test on OdraVM** — teaches `cargo odra test`, explains the in-memory VM and what happens under the hood
4. **(Optional) Deploy to local node** — invokes `start-nctl` then `deploy-to-livenet`, explains the Casper network model

Tracks progress, lets user stop and resume. The key difference from development mode: `onboard` teaches *why*, not just *what*.

---

### Category: Contract Creation

#### 3. `new-contract` — Scaffold a regular contract module

Asks for:
- Module name (PascalCase)
- Brief description
- Storage fields (`Var<T>`, `Mapping<K, V>`, `SubModule<T>`)
- Events (if any)
- Errors (if any)

Generates:
- `contracts/src/<snake_name>.rs` — module with `#[odra::module]`, events, errors, test stub
- Adds `pub mod <snake_name>;` to `contracts/src/lib.rs`
- Adds `[[contracts]] fqn = "<snake_name>::ModuleName"` to `Odra.toml`
- Wires into `cli/cli.rs` — adds to deploy script and registers contract

Teaches: what `#[odra::module]` generates (HostRef, InitArgs, ContractRef), how storage works, how events/errors are declared.

#### 4. `new-factory-contract` — Scaffold a factory contract

Same flow as `new-contract` but:
- Uses `#[odra::module(factory=on)]` on both struct and impl
- Generates factory-specific patterns: explains `new_contract()`, factory events (`ContractDeployed`)
- CLI wiring uses factory deploy pattern (`ModuleFactory::deploy`)

---

### Category: Contract Modification

#### 5. `new-entrypoint` — Add an entry point to an existing contract

Asks for (no guessing):
- Which contract to modify (lists available contracts from `contracts/src/`)
- Entry point name
- Arguments (name + type for each)
- Return type (if any)

Generates:
- New `pub fn` in the contract's `#[odra::module] impl` block
- Test stub in the `#[cfg(test)]` module

Does not modify existing entry points or storage — additive only.

#### 6. `new-version` — Create an upgraded contract version

Two modes (user chooses):

**Mode A — Modify existing struct:**
- Adds new entry points to the existing contract
- Adds new storage fields (if needed)
- Adds `pub fn upgrade(&mut self, ...)` method that handles state migration
- Entry points are additive: no removals, no signature changes

**Mode B — New struct:**
- Creates a new module (e.g., `CounterV2`) in a new file
- Copies all existing entry points with identical signatures
- Adds new entry points and new storage fields
- Adds `pub fn upgrade(&mut self, ...)` method
- Registers the new contract in `Odra.toml`

Both modes:
- Follow the upgrade pattern from `examples/src/features/upgrade.rs`
- Generate a test demonstrating `deploy_with_cfg` (V1) → `try_upgrade` (V2) flow
- Wire into CLI deploy script with upgrade support

---

### Category: Deployment & Scenarios

#### 7. `new-scenario` — Implement a CLI scenario

Asks for:
- Scenario name and description
- Which contracts it interacts with
- Arguments (`CommandArg` definitions)

Generates:
- Struct implementing `Scenario` + `ScenarioMetadata`
- `args()` method with typed `CommandArg` entries
- `run()` method skeleton with contract refs from container
- Registers in `cli/cli.rs` via `.scenario(MyScenario)`

#### 8. `start-nctl` — Start local Casper node

Single responsibility — node lifecycle only:

1. Check Docker is running
2. Check if `mynctl` container already exists
3. Start NCTL (via `just run-nctl` or direct `docker run`)
4. Wait for readiness (poll RPC at `http://localhost:11101`)
5. Extract keys to `.node-keys/`
6. Create `.env` from `.env.sample` with nctl defaults

#### 9. `deploy-to-livenet` — Deploy contracts to local node

1. Check NCTL is running (suggests `start-nctl` if not)
2. Load `.env`
3. Run CLI binary with `--features=livenet`
4. Report deployment results (exit code, key events, errors)

Scoped to nctl only — no testnet/mainnet support (out of scope for onboarding).

---

## Two Modes, One Skill Set

| Aspect | Guided mode (`/onboard`) | Development mode (direct skills) |
|---|---|---|
| Entry point | `/onboard` | `/new-contract`, `/new-entrypoint`, etc. |
| Audience | Newcomers | Developers who know Odra |
| Verbosity | High — explains concepts, checks understanding | Low — asks requirements, generates code |
| Flow | Linear, progressive | On-demand, any order |
| Extra steps | Concept intros, checkpoints, "why" explanations | None — just the skill |
| Skills used | Same 9 skills | Same 9 skills |

The `onboard` skill does not contain duplicate logic. It invokes the same skills a developer would use directly, adding teaching scaffolding around each invocation.

## Relationship to Existing Skills

The existing skills (`gen-module`, `run-example-on-livenet`, `nctl-test`) remain untouched in the main Odra repo. They serve framework development. The starter template skills are independent, ship with the template, and are designed for end-user projects.

## What's NOT in Scope

- Adding CLAUDE.md to existing templates (blank, full, workspace, cep18, cep95)
- Testnet/mainnet deployment
- Frontend/backend scaffolding (workspace is extensible for this, but skills don't cover it)
- Advanced topics: cross-contract calls, delegate!, external contracts
