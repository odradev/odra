# Changelog

Changelog for `odra`.

## [0.8.0] - 2024-XX-XX
- Removed `#[odra::init]` macro - now only one method can be a constructor and it has to be named `init` and env needs to be passed as a first parameter.
- Replaced `contract_env::` with `self.env.` in contract context. For example, instead of `contract_env::caller()` use `self.env.caller()`.
- Removed `emit()` method from events. Use `self.env.emit_event(event)` instead.
- It is no longer possible to put a Mapping inside of a Mapping. Use tuple keys instead, for example, instead of
```rust
allowances: Mapping<Address, Mapping<Address, U256>>
```
use
```rust
allowances: Mapping<(Address, Address), U256>
```
- `Ref` has been divided into two: `HostRef` and `ContractRef` depending on the context.
- Removed `assert_events!` macro. Use `env.emitted_event` and similar methods instead.
- Storage keys are constructed differently, keep that in mind when querying the storage outside the contract.

## [0.7.0] - 2023-11-07
## Changed
- `no_std` is now the default mode for WASM.
- `odra-types` are the only types crate.

## Removed
- Removed `odra-casper-types` and `odra-mock-vm-types`.
- Removed `OdraType` in favor of `ToBytes` and `FromBytes` traits.

## [0.6.0] - 2023-09-20
### Added
- `std` feature for odra crate.
- `odra::prelude` reexports basic types and traits based on std feature.

### Changed
- Replace `WeeAlloc` with `ink!`'s `BumpAllocator`.

## [0.5.0] - 2023-07-19
### Added
- Signature verification in contract_envs.
- Signature creation in test_envs.
- StaticInstance and DynamicInstance traits.
- Node trait to build static keys at the compilation time.
- `List` now can contain modules.

### Removed
- Crypto module from odra-modules.
- Instance trait.
- derive PartialEq, Eq, Debug from storage building block (Variable, Mapping, List).

### Changed
- utilize `KeyMaker` to create storage keys in `casper` and `casper-livenet`.
- Only `payable` entrypoints checks incoming Casper purse and it is optional.
- Casper contract's `cargo_purse` is now created just once and reused.

## [0.4.0] - 2023-06-26
### Added
- `odra-casper-livenet` has new resource: `proxy_caller.wasm`.
- `odra-casper-shared`, `odra-casper-types` and `odra-types` are now `no_std`.
- Modules can have arguments that are references.
- `odra-casper-test-env` has new `gas_report()` function, that shows gas usages.

### Fixed
- Casper gas counting.

### Changed
- `test-env/getter-proxy` is now `proxy-caller`. 
- `proxy_getter.wasm` is now `proxy_caller_with_result.wasm`.
- Casper's `call` methods for wasm build has a new set of arguments including upgradable contracts.
- Support for the Casper 1.5.1 version.

### Removed
- `casper-node` is no longer a dependency for `odra-casper-livenet`.
- `Composer`

## [0.3.1] - 2023-06-01
### Added
- `Composer` - a tool for composing Odra modules' instances.
- Casper's `chainspec.yaml` added to the repository.
- Improvements to numeric conversions.
- `Typed` trait is implemented automatically for `OdraType` types.
- `test_env` exposes `total_gas_used()` function to allow better native token usage tracking.
- `RUST_BACKTRACE=-1` will show the backtrace of the contract calls that panicked.
- Panicked contract calls will print the information about the call.

### Fixed
- Better handling of call stack in MockVM.
- Better handling of native transfers in MockVM.
- Genesis in the MockVM generates more addresses with more tokens.
- Improved `assert_exception!` macro. It no longer needs new instance of Ref.

### Changed
- `transfer_tokens()` now reverts if the transfer failed instead of returning false.
- `advance_block_time_by()` now uses milliseconds instead of seconds.

## [0.3.0] - 2022-05-22
### Added
- new module - `Sequence` - for storing single value in the storage that can be incremented.
- Casper contracts now generate schemas automatically.
containing contract structure and Events schemas following the Casper Event Standard.
- Modules and Mappings now can be a part of a Mapping allowing to nest them.
- Namespaces can be generated manually for a finer control over nesting.
- `#[odra::module(events = [Event1, Event2, ...])]` attribute for modules to generate events.
- `delegate!` macro for delegating calls to other modules.
- `#[odra(non_reeentrant)]` attribute for modules to enable reentrancy guard.
- `#[derive(OdraType)]` for deriving `OdraType` trait for custom types.
- Unit-type Enums can now be OdraTypes.
- New Odra Modules:
  - `Erc721`,
  - `Erc1155`,
  - `Ownable`,
  - `Ownable2Step`,
  - `AccessControl`.
- Experimental `odra-casper-livenet` crate for deploying contracts to Casper Livenet.

### Changed
- `last_call_contract_cost()` is now `last_call_contract_gas_used()`.
- It is no longer possible to build zero address in casper.

### Removed
- `new` method for Address.

## [0.2.0] - 2022-11-23
### Added
- Global `CHANGELOG.md`.
- `odra`
  - `List` collection.
  - Add tests for `Variable` and `Mapping`.
  - Contract refs (`*Ref` struct) has `with_tokens()` function to attach tokens to the call.
- `odra-mock-vm`
  - Add `AccountBalance` to store the native token amount associated with the `Address`.
  - Add `CallstackElement` to stack contract execution along with the attached value.
- New crate `odra-mock-vm-types` - encapsulates `mock-vm`-specific types.
- New crate `odra-casper-codegen` - generates Casper contracts, moved from `odra-casper-backend`. 
- New crate `odra-casper-types` - encapsulates `Casper`-specific types.
- `odra-casper-test-env`
  - Add `dummy_contract_env` module.
- New crate `odra-examples`
   - `Ownable` - simple storage, errors, events.
   - `Erc20` - erc-20 standard implementation.
   - `OwnedToken` - modules reuse.
   - `BalanceChecker` - external contract calls.
   - `TimeLockWallet` - payable.
- New crate `odra-modules` a set of reusable modules.
  - `Erc20` - erc-20 token.
  - `WrappedNativeToken` - erc20-based wrapped token.
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
  - Add `Type` enum describing all supported types.
  - Rename `Event` to `OdraEvent`.
  - Remove all dependencies.
  - Update `VmError`s.
  - Bring `contract_def` module from `odra` crate.
  - Remove `casper` dependencies from `contract_def`.
- `odra-utils`
  - Remove `odra-types` dependency.
  - Change `event_absolute_position` signature - the function returns `Option<usize>`.
- `odra-codegen`
  - Handle `odra(payable)` attribute.
  - Adjust generated code to framework changes.
- `odra-mock-vm`
  - `MockVM` refactor.
  - `test_env` is a module not a struct.
  - Change `call_contract()` signature.
  - Add `advance_block_time_by()`, `token_balance()`, `one_token()` functions.
- `odra-casper-backend`, `odra-casper-shared`, `odra-casper-test-env` moved from a separate repository.
- `odra-casper-backend`
    - `contract_env` is a module not a struct.
    - Add `one_token()`, `self_balance()`, `attached_value()`, `transfer_tokens()` functions.
    - Change `call_contract()` signature
  - `odra-casper-test-env`
    - `test_env` is a module not a struct.
    - Change `call_contract()` signature.
    - Add `advance_block_time_by()`, `token_balance()`, `one_token()` functions.

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
  - `unwrap_or_revert` module that provides a trait for unwrapping values in the context of smart contract.
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
