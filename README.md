<div align="center">
    <img width="600rem" src=".images/odra_logo.png"></img>
    <h3>Odra - Smart contracts for Casper Network.</h3>
    <p>
        <a href="https://odra.dev/docs">Docs</a> |
        <a href="https://odra.dev/docs/getting-started/installation">Installation</a> |
        <a href="https://odra.dev/docs/category/tutorials">Tutorials</a> |
        <a href="https://github.com/odradev/cargo-odra">Cargo Odra</a> |
        <a href="https://github.com/odradev/odradev-plugins">Claude Code Plugin</a> |
        <a href="https://discord.com/invite/Mm5ABc9P8k">Discord</a> |
        <a href="https://odra.dev/blog">Blog</a>
    </p>
    <p>
        <a href="https://github.com/odradev/odra/actions">
            <img src="https://img.shields.io/github/actions/workflow/status/odradev/odra/test.yml?branch=release%2F1.0.0" alt="GitHub Workflow Status" />
        </a>
        <a href="https://codecov.io/gh/odradev/odra">
            <img src="https://codecov.io/gh/odradev/odra/graph/badge.svg?token=8AT1UNOJMS" alt="Code coverage">
        </a>
        <a href="https://crates.io/crates/odra">
            <img src="https://img.shields.io/crates/v/odra" alt="Version" />
        </a>
        <a href="https://crates.io/crates/odra">
            <img src="https://img.shields.io/crates/l/odra" alt="License" />
        </a>
        <img src="https://img.shields.io/github/languages/top/odradev/odra" alt="Language" />
    </p>
</div>

## Table of Contents
- [Project Setup](#project-setup)
- [Usage](#usage)
- [Example](#example)
- [Tests](#tests)
- [Links](#links)
- [Contact](#contact)

## Project Setup

This repository is the framework crate itself. You do **not** clone it to write contracts —
you scaffold a project with [Cargo Odra](https://github.com/odradev/cargo-odra) and depend on
`odra` from crates.io.

```bash
# 1. Rust toolchain with the wasm target (see https://rustup.rs)
rustup target add wasm32-unknown-unknown

# 2. The project generator / build tool
cargo install cargo-odra --locked

# 3. A new project
cargo odra new --name my_project && cd my_project
cargo odra test
```

Full instructions, including the `wasm-strip` and `wasm-opt` prerequisites, are in the
[Installation guide](https://odra.dev/docs/getting-started/installation). On Ubuntu or WSL, use
[Ubuntu / WSL setup](https://odra.dev/docs/getting-started/ubuntu-wsl-setup) — the same steps with
exact commands, including the `binaryen` version apt is too old to give you.

### Working with an AI agent

If you use Claude Code, install the [Odra plugin](https://github.com/odradev/odradev-plugins).
It teaches the agent Odra's APIs, project layout and tooling, so it scaffolds, writes, tests and
deploys contracts the way this framework expects instead of guessing:

```
/plugin marketplace add odradev/odradev-plugins
/plugin install odra-plugin@odradev-plugins
```

Agents without the plugin should read [https://odra.dev/llms.txt](https://odra.dev/llms.txt)
first — it is an index of the whole documentation set.

## Usage

Use [Cargo Odra](https://github.com/odradev/cargo-odra) to generate, build and test you code.

<div align="center">
    <img src=".images/cargo_odra.gif"></img>
</div>

## Example

```rust
use odra::prelude::*;

#[odra::module]
pub struct Flipper {
    value: Var<bool>,
}

#[odra::module]
impl Flipper {
    pub fn init(&mut self) {
        self.value.set(false);
    }

    pub fn set(&mut self, value: bool) {
        self.value.set(value);
    }

    pub fn flip(&mut self) {
        self.value.set(!self.get());
    }

    pub fn get(&self) -> bool {
        self.value.get_or_default()
    }
}

#[cfg(test)]
mod tests {
    use crate::flipper::Flipper;
    use odra::host::{Deployer, NoArgs};

    #[test]
    fn flipping() {
        let env = odra_test::env();
        let mut contract = Flipper::deploy(&env, NoArgs);
        assert!(!contract.get());
        contract.flip();
        assert!(contract.get());
    }
}
```

Checkout our [examples](https://github.com/odradev/odra/tree/HEAD/examples).
It shows most of Odra features.

## Tests

Before running tests make sure you have following packages installed:

- Rust toolchain (see [rustup.rs](https://rustup.rs/)) with `wasm32-unknown-unknown` target.
- `cargo-odra` with its dependencies (see [Cargo Odra](https://github.com/odradev/cargo-odra))
- `just` (see [just](https://github.com/casey/just#packages))

Run tests:

```bash
$ just test
```

## Links

* [Odra Book - Docs and Tutorials](https://odra.dev/docs) — source: [odradev/odradev.github.io](https://github.com/odradev/odradev.github.io)
* [API Documentation](https://docs.rs/odra/latest/odra/)
* [Cargo Odra](https://github.com/odradev/cargo-odra) — the `cargo odra` project generator and build tool
* [Odra Claude Code Plugin](https://github.com/odradev/odradev-plugins) — skills for agentic Odra development
* [llms.txt](https://odra.dev/llms.txt) — documentation index for LLM agents
* [Example Contracts](https://github.com/odradev/odra/tree/HEAD/examples)

## Contact
Need some help? Write to **contract@odra.dev**.

<div align="center">
    by <a href="https://odra.dev">odra.dev<a>
</dev>
