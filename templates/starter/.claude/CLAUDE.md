# {{project-name}}

An Odra smart contract workspace. Contracts live in `contracts/`, the CLI for deployment in `cli/`.

## Getting Started

**New to Odra?** Run `/odra:onboard` for a guided walkthrough — from writing your first contract to deploying it.

**Know Odra?** Use skills directly:

| Skill | What it does |
|---|---|
| `/odra:check-env` | Verify your development environment |
| `/odra:setup-nctl` | Setup a local Casper node via Docker |
| `/odra:deploy-to-livenet` | Deploy contracts to nctl, testnet, or mainnet |

## Project Structure

- `contracts/src/` — contract modules (one `.rs` file per contract)
- `cli/cli.rs` — CLI for deploying and interacting with contracts on livenet
- `Odra.toml` — contract registry for `cargo odra`
- `.env.sample` — livenet environment variable template

