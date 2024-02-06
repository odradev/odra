<div align="center">
    <img src=".images/odra_logo.png"></img>
    <h3>Odra - Smart contracts for Casper Network.</h3>
    <p>
        <a href="https://odra.dev/docs">Docs</a> |
        <a href="https://odra.dev/docs/getting-started/installation">Installation</a> |
        <a href="https://odra.dev/docs/category/tutorials">Tutorials</a> |
        <a href="https://github.com/odradev/cargo-odra">Cargo Odra</a> |
        <a href="https://discord.com/invite/Mm5ABc9P8k">Discord</a> |
        <a href="https://odra.dev/blog">Blog</a
    </p>
    <p>
        <a href="https://github.com/odradev/odra/actions">
            <img src="https://img.shields.io/github/actions/workflow/status/odradev/odra/odra-ci.yml?branch=release%2F0.3.0" alt="GitHub Workflow Status" />
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
- [Usage](#usage)
- [Example](#example)
- [Tests](#tests)
- [Links](#links)
- [Contact](#contact)

## Usage

Use [Cargo Odra](https://github.com/odradev/cargo-odra) to generate, build and test you code.

<div align="center">
    <img src=".images/cargo_odra.gif"></img>
</div>

## Example

```rust
use odra::prelude::*;
use odra::Var;

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
    use crate::flipper::FlipperHostRef;
    use odra::host::{Deployer, NoArgs};

    #[test]
    fn flipping() {
        let env = odra_test::env();
        let mut contract = FlipperHostRef::deploy(&env, NoArgs);
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
- `cargo-odra` (see [Cargo Odra](https://github.com/odradev/cargo-odra))
- `just` (see [just](https://github.com/casey/just#packages))

Run tests:

```bash
$ just test
```

## Links

* [Odra Book - Docs and Tutorials](https://odra.dev/docs)
* [API Documentation](https://docs.rs/odra/latest/odra/)
* [Cargo Odra](https://github.com/odradev/cargo-odra)
* [Example Contracts](https://github.com/odradev/odra/tree/HEAD/examples)

## Contact
Need some help? Write to **contract@odra.dev**.

<div align="center">
    by <a href="https://odra.dev">odra.dev<a>
</dev>
