# Changelog

Changelog for `odra`.

## [0.2.0] - xxxx-xx-xx
### Added
- Global `CHANGELOG.md`
- `odra`
  - `List` collection.
  - Add tests for `Variable` and `Mapping`.
  - Contract refs (`*Ref` struct) has `with_tokens()` function to attach tokens to the call.
- `odra-types`
  - `std` feature, disabled by default.
- `odra-utils`
  - `no_std` feature, enabled by default.
  - `std` feature, disabled by default.
- `odra-mock-vm-types`
- `odra-casper-codegen`
- `odra-casper-types`
- `odra-examples`
   - `Ownable` - simple storage, errors, events.
   - `Erc20` - erc-20 standard implementation.
   - `OwnedToken` - modules reuse.
   - `BalanceChecker` - external contract calls.
   - `TimeLockWallet` - payable.
- `odra-modules`
- `CONTRIBUTING.md` and `SECURITY.md`.
- `justfile`
  
### Changed
- `odra`
    - Features update: rename `wasm` to `casper`, remove `wasm-test`.
    - Features check: setting `casper` and `mock-vm` causes compile error.
    - `Mapping`'s functions: `set()`, `add()`, `subtract()`.
    - `Variable`'s functions: `set()`, `add()`, `subtract()`.
    - `Variable` and `Mapping` are implemented for `OdraType`.
    - Add `amount` parameter to `call_contract`.
    - `contract_env` and `test_env` are modules not structs.
    - To deploy contract in test, structs no longer have `deploy_*` function, `*Deployer` structs are generated.
    - The default contract constructor is called `*Deployer::default`.
- `odra-types`
- `odra-utils`
  - Remove `odra-types` dependency.
  - Change `event_absolute_position` signature - the function returns `Option<usize>`.
- `odra-test-env-wrapper`
- `odra-codegen`
  - Handle `odra(payable)` attribute.
  - Adjust generated code to framework changes.
- `odra-test-env-wrapper`
- `odra-mock-vm`
- `odra-casper-backend`, `odra-casper-shared`, `odra-casper-test-env` moved from a separate repository.

### Removed
- `odra` extern bindings.
- `odra-test-env-wrapper` crate.
- Separate changelog files per crate.

## [0.1.0] - 2022-08-10
### Added
- `odra-casper-backend`, `odra-casper-shared`, `odra-casper-test-env` documentation.

## [0.0.1] - 2022-07-20
### Added
- `odra`
  - `contract_def` provides an abstract definition of a smart contract.
  - `external_api` module that defines the api to be implemented by a backed blockchain.
  - `instance` module exposing a trait used to instantiate a module.
  - `variable` and `mapping` modules that allow to read and write single values and collections in smart contracts.
  - `test_utils` module that defines functions and macros to utilize smart contracts testing.
  - `unwrap_or_revert` module that provides a trait for unwrapping values in the context of smart contract
  - `CHANGELOG.md` and `README.md` files.
- `odra-types`
  - `address` module defines a blockchain-agnostic address struct that can be used for storing contract and account addresses.
  - `arithmetic` module defines traits for safe (revertable), overflowing addition, and subtraction.
  - `event` module that defines an event interface and companion errors.
  - `error` module encapsulates errors that may occur during smart contract execution.
  - `ToBytes` and `FromBytes` traits for structs serialization and deserialization, e.g. events.
  - `CHANGELOG.md` and `README.md` files.
  - Re-export `casper-types` as `odra-types`.
- `odra-utils`
  - `camel_to_snake` function for the contract name conversion.
  - `event_absolute_position` function to calculate the event absolute position.
  - `CHANGELOG.md` and `README.md` files.
- `odra-test-env-wrapper`
  - `TestBackend` defines the api of `libodra_test_env.so`.
  - `on_backend` function to interact with `libodra_test_env.so`.
  - `CHANGELOG.md` and `README.md` files.
- `odra-mock-vm`
  - `TestEnv` and `ContractEnv` implementations used to run tests.
  - `MockVM`, a lightweight blockchain virtual machine.
  - `CHANGELOG.md` and `README.md` files.
- `odra-proc-macros`
  - Procedural macro `odra::module`.
  - Procedural macro `odra::instance`.
  - Procedural macro `odra::external_contract`.
  - Procedural macro `odra::execution_error`.
  - Procedural macro `odra::odra_error`.
  - Derive macro `Event`.
  - `CHANGELOG.md` and `README.md` files.
- `odra-ir`
  - structs responsible for capturing code used to generate code for `odra::module`, `odra::instance`, `odra::external_contract`, `odra::execution_error`, `odra::odra_error`, and `Event` macros.
  - `CHANGELOG.md` and `README.md` files.
- `odra-codegen`
  - code generators for `odra::module`, `odra::instance`, `odra::external_contract`, `odra::execution_error`, `odra::odra_error`, and `Event` macros.
  - `CHANGELOG.md` and `README.md` files.
- `odra-casper-test-env`
  - `getter_proxy` child crate.
  - `CHANGELOG.md` and `README.md` files.
  - `env::CasperTestEnv` that wraps Casper's `InMemoryWasmTestBuilder`.
  - `#[no_mangle]` functions used to communicate with the `libodra_test_env.so`
- `odra-casper-shared`
  - `casper_address::CasperAddress` struct.
  - `CHANGELOG.md` and `README.md` files.
- `odra-casper-backend`
  - `codegen` module that is used to generate wasm file.
  - `backend` module that exports all required `#[no_mangle]` functions and interacts with the Casper Host.
  - `CHANGELOG.md` and `README.md` files.
- `odra-casper-getter-proxy`
  - `getter_proxy.rs` binary file.
