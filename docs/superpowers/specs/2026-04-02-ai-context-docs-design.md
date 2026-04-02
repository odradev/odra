# AI Context Docs for Odra Starter Template

**Date**: 2026-04-02
**Status**: Approved

## Goal

Add a modular, AI-only reference documentation layer to the Odra starter template. The docs complement the existing Claude Code skills: skills handle step-by-step workflows; context docs provide the conceptual and API reference knowledge skills load on demand.

Target: AI coding agents (Claude Code) helping developers write Odra smart contracts in scaffolded projects. Applicable across task granularities — full system design, single contract, single entry point, bug fixing.

---

## Design Principles

- **AI-only**: docs live in `.claude/context/`, not surfaced to developers browsing the project
- **On-demand**: skills explicitly load only the files they need; CLAUDE.md loads nothing automatically
- **Non-overlapping**: each file has a single clear responsibility; cross-references replace duplication
- **Two-tier**: overview files for broad tasks, reference files for targeted tasks

---

## Directory Structure

```
templates/starter/.claude/
  CLAUDE.md                           # minimal entry point (~20 lines)
  skills/
    new-contract/SKILL.md
    new-factory-contract/SKILL.md
    new-entrypoint/SKILL.md
    new-version/SKILL.md
    new-scenario/SKILL.md
    check-env/SKILL.md
    start-nctl/SKILL.md
    deploy-to-livenet/SKILL.md
    onboard/SKILL.md
  context/
    overview/
      architecture.md                 # crate map, execution flow, two context layers
      contract-model.md               # #[odra::module], HostRef, EntryPointsCaller, Deployer
      testing-model.md                # OdraVM vs CasperVM vs livenet
    reference/
      storage.md                      # Var<T>, Mapping<K,V>, Sequence<T>, List<T>
      entry-points.md                 # fn rules, #[odra(init)], payable, return types
      events.md                       # #[odra::event], emit, test assertions
      errors.md                       # OdraError, revert!, unwrap_or_revert
      cross-contract.md               # #[odra::external_contract], ContractRef
      testing.md                      # env(), deploy(), caller switching, assertions
      deployment.md                   # livenet env vars, load_or_deploy, OdraCli, scenarios
```

---

## Loading Model

Skills declare a `## Context` section listing which files to read before proceeding. The AI reads them explicitly via the Read tool.

### Context assignments per skill

| Skill | Overview files | Reference files |
|---|---|---|
| `/new-contract` | `contract-model.md` | `storage.md`, `entry-points.md` |
| `/new-factory-contract` | `contract-model.md` | `storage.md`, `entry-points.md` |
| `/new-entrypoint` | — | `entry-points.md` + conditionally `events.md`, `errors.md` |
| `/new-version` | `contract-model.md` | `storage.md`, `entry-points.md` |
| `/new-scenario` | — | `deployment.md` |
| `/deploy-to-livenet` | — | `deployment.md` |
| `/start-nctl` | — | `deployment.md` |
| `/onboard` | `architecture.md`, `contract-model.md`, `testing-model.md` | — |
| `/check-env` | — | — |

Reference files are loaded conditionally when the skill can infer they're needed (e.g., `/new-entrypoint` loads `events.md` only if the entry point emits events).

---

## File Content Scope

### Overview files (~100-150 lines each)

**`architecture.md`**
- What Odra is and what problem it solves
- Crate map: `core`, `odra`, `odra-macros`, `odra-vm`, `odra-casper/*`, `odra-cli`, `odra-schema`
- Execution flow: HostRef → HostEnv → EntryPointsCaller → ContractEnv
- Two context layers: `HostContext` (host side) vs `ContractContext` (on-chain side)
- Smart pointer conventions: `Rc<T>`, `RefCell<T>` (single-threaded)

**`contract-model.md`**
- What `#[odra::module]` generates: `HostRef`, `Deployer`, `EntryPointsCaller`, `InitArgs`
- Struct fields = on-chain storage; pub fn = entry points
- How `HostRef` proxies method calls through `HostEnv`
- The `Deployer` trait: `deploy()` / `try_deploy()`
- `ContractRef` for cross-contract use

**`testing-model.md`**
- OdraVM: fast, in-memory, default — `cargo odra test`
- CasperVM: full Casper execution engine — `cargo odra test -b casper`
- Livenet: real node, requires Docker/NCTL or remote — `cargo odra run`
- When to use each; how to switch backends

### Reference files (~60-100 lines each)

**`storage.md`**
- `Var<T>`: construction, `get()`, `get_or_default()`, `set()`, `is_none()`
- `Mapping<K, V>`: `get()`, `get_or_default()`, `set()`, nested mappings
- `Sequence<T>` and `List<T>`: append, iterate
- Common patterns: initializing in `init`, updating in entry points

**`entry-points.md`**
- Visibility rules: only `pub fn` become entry points
- Constructor: `#[odra(init)]`, `InitArgs` struct generation
- Payable entry points: `#[odra(payable)]`
- Return types: `()`, `T`, `OdraResult<T>`
- Accessing caller, balance, block time via `self.env()`

**`events.md`**
- Defining events: `#[odra::event]` on a struct
- Emitting: `self.env().emit_event(MyEvent { ... })`
- Test assertions: `assert_events!(contract, MyEvent { field: value, .. })`

**`errors.md`**
- Defining errors: `#[odra::odra_error]` on an enum
- Reverting: `revert!(MyError::Variant)` or `self.env().revert(MyError::Variant)`
- `unwrap_or_revert`: on `Option<T>` and `Result<T, E>`
- Asserting errors in tests: `contract.try_call().unwrap_err()`

**`cross-contract.md`**
- Defining an external interface: `#[odra::external_contract]` on a trait
- Getting a `ContractRef`: `MyContractContractRef::at(&env, address)`
- Calling across contracts: same as local calls, but via ref
- Using cross-contract in tests: deploy both, wire addresses

**`testing.md`**
- Creating env: `odra_test::env()`
- Deploying: `MyContract::deploy(&env, MyContractInitArgs { ... })`
- Switching caller: `env.set_caller(env.get_account(1))`
- Advancing time/block: `env.advance_block_time(ms)`
- Balance assertions: `env.balance_of(&address)`
- Event assertions: see `events.md`
- Error assertions: see `errors.md`

**`deployment.md`**
- Required env vars: `ODRA_CASPER_LIVENET_SECRET_KEY_PATH`, `NODE_ADDRESS`, `EVENTS_URL`, `CHAIN_NAME`
- `load_or_deploy`: deploys on first run, loads on subsequent runs
- `OdraCli`: `.deploy()`, `.contract::<T>()`, `.scenario()`
- Writing scenarios: structs implementing `OdraScenario`
- NCTL vs testnet vs mainnet: chain name differences, key sources

---

## Integration with CLAUDE.md

CLAUDE.md gains a single section pointing to the context layer:

```markdown
## AI Context Docs

Reference docs for Odra concepts live in `.claude/context/`. Skills load them
on demand. You can also read them directly when answering questions about Odra.

- `overview/` — architecture, contract model, testing model
- `reference/` — storage, entry points, events, errors, cross-contract, testing, deployment
```

No files are auto-loaded. The index is enough for the AI to know what's available.
