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

## AI Context Docs

Reference docs for Odra concepts live in `.claude/context/`. Skills load them
on demand. You can also read them directly when answering questions about Odra.

- `overview/` — architecture, contract model, testing model
- `reference/` — storage, entry points, events, errors, cross-contract, testing, deployment
