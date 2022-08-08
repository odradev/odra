<div align="center">

<img src=".images/odra_logo.png"></img>
<h3>Odra - High-level smart contract framework for Rust.</h3>

</div>

## Table of Contents
* [Usage](#usage)
* [Example](#example)
* [Backends](#backends)
* [Links](#links)
* [Contact](#contact)

## Usage
TODO: Add gif with example.

TODO: Show casper-odra commands.

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
        let contract = Flipper::deploy();
        assert_eq!(contract.get(), false);
        contract.flip();
        assert_eq!(contract.get(), true);
    }
}
```

## Backends

Odra is designed to integrate any most WebAssemby-based smart contract systems.

Integrated blockchains:
* Casper Network. See [Odra Casper](https://github.com/odradev/odra-casper) for more details.

## Links

* [Odra](https://github.com/odradev/odra)
* [Cargo Odra](https://github.com/odradev/cargo-odra)
* [Odra Template](https://github.com/odradev/odra-template)
* [Example Contract: Owned Token](https://github.com/odradev/owned-token)
* [Odra Casper](https://github.com/odradev/odra-casper)
* [Original Proposal for Odra Framework](https://github.com/odradev/odra-proposal)

## Contact
Write **maciej@odra.dev** or [join Discord]().

---
<div align="center">
by <a href="https://odra.dev">odra.dev<a>
</dev>