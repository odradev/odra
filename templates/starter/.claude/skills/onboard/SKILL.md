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
