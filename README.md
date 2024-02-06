<div align="center">
    <img src=".images/odra_logo.png"></img>
    <h3>Odra - High-level smart contract framework for Rust.</h3>
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
- [Backends](#backends)
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
use odra::Var;

#[odra::module]
pub struct Flipper {
    value: Var<bool>,
}

#[odra::module]
impl Flipper {
    pub fn flip(&mut self) {
        self.value.set(!self.get());
    }

    pub fn get(&self) -> bool {
        self.value.get_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::FlipperDeployer;

    #[test]
    fn flipping() {
        let mut contract = FlipperDeployer::default();
        assert_eq!(contract.get(), false);
        contract.flip();
        assert_eq!(contract.get(), true);
    }
}
```

Checkout our [examples](https://github.com/odradev/odra/tree/HEAD/examples).
It shows most of Odra features.

## Backends

Odra is designed to integrate with most WebAssembly-based smart contract systems.

Integrated blockchains:
* Casper Network.

## Tests

Before running tests make sure you have following packages installed:

- Rust toolchain (see [rustup.rs](https://rustup.rs/)) with `wasm32-unknown-unknown` target.
- `wasm-strip` (see [wabt](https://github.com/WebAssembly/wabt))
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
* [Original Proposal for Odra Framework](https://github.com/odradev/odra-proposal)

## Contact
Need some help? Write to **contract@odra.dev**.

<div align="center">
    by <a href="https://odra.dev">odra.dev<a>
</dev>
