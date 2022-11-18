<div align="center">
    <img src=".images/odra_logo.png"></img>
    <h3>Odra - High-level smart contract framework for Rust.</h3>
    <p>
        <a href="https://github.com/odradev/odra/actions">
            <img src="https://img.shields.io/github/workflow/status/odradev/odra/odra-ci/develop?style=plastic" alt="Build status" />
        </a>
        <a href="https://crates.io/crates/odra">
            <img src="https://img.shields.io/crates/v/odra?style=plastic" alt="Version" />
        </a>
        <a href="https://crates.io/crates/odra">
            <img src="https://img.shields.io/crates/l/odra?style=plastic" alt="License" />
        </a>
        <img src="https://img.shields.io/github/languages/top/odradev/odra" alt="Language" />
    </p>
</div>

## Table of Contents
- [Usage](#usage)
- [Example](#example)
- [Backends](#backends)
- [Links](#links)
- [Contact](#contact)

## Usage

Use [Cargo Odra](https://github.com/odradev/cargo-odra) to generate, build and test you code.

<div align="center">
    <img src=".images/cargo_odra.gif"></img>
</div>

## Example

```rust
use odra::Variable;

#[odra::module]
pub struct Flipper {
    value: Variable<bool>,
}

#[odra::module]
impl Flipper {
    pub fn flip(&self) {
        self.value.set(!self.get());
    }

    pub fn get(&self) -> bool {
        self.value.get_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::Flipper;

    #[test]
    fn flipping() {
        let contract = FlipperDeployer::default();
        assert_eq!(contract.get(), false);
        contract.flip();
        assert_eq!(contract.get(), true);
    }
}
```

Checkout our [examples](https://github.com/odradev/odra/tree/0.2.0/examples).
It shows most of Odra features.

## Backends

Odra is designed to integrate with most WebAssembly-based smart contract systems.

Integrated blockchains:
* Casper Network. See [Odra Casper](https://github.com/odradev/odra/tree/0.2.0/odra-casper) for more details.

## Links

* [Odra](https://github.com/odradev/odra)
* [Documentation](https://docs.rs/odra/latest/odra/)
* [Cargo Odra](https://github.com/odradev/cargo-odra)
* [Odra Template](https://github.com/odradev/odra-template)
* [Example Contracts](https://github.com/odradev/odra-examples)
* [Odra Casper](https://github.com/odradev/odra-casper)
* [Original Proposal for Odra Framework](https://github.com/odradev/odra-proposal)

## Contact
Need some help? Write to **contract@odra.dev**.

---
<div align="center">
    by <a href="https://odra.dev">odra.dev<a>
</dev>
