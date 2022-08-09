# Changelog

Changelog for `odra-types`.

## [0.0.1] - 2022-07-20
### Added
- `address` module defines a blockchain-agnostic address struct that can be used for storing contract and account addresses.
- `arithmetic` module defines traits for safe (revertable), overflowing addition, and subtraction.
- `event` module that defines an event interface and companion errors.
- `error` module encapsulates errors that may occur during smart contract execution.
- `ToBytes` and `FromBytes` traits for structs serialization and deserialization, e.g. events.
- `CHANGELOG.md` and `README.md` files.
- Re-export `casper-types` as `odra-types`.
