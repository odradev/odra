# Changelog

Changelog for `odra`.

## [1.2.0] - 2024-07-30
### Added
- `Extern` type for simple external calls.

### Changed
- Allow for reminting tokens in `CEP78`.
- Include custom types into schema.

## [1.1.0] - 2024-06-17
### Added
- Support for CSPR.cloud's auth token.
- `new` method for `Address` to create an address from a string.
- `transfer` function in the `HostEnv` allowing native token transfers between accounts.
- Reverse lookup feature for `CEP78`.
- `Addressable` trait for types that can be converted to `Address`.

### Updated
- Fix `get_block_time` in `casper-rpc-client`.
- Calling `try_` functions in the livenet context now returns a `Result` instead of panicking.
- Improve error messages in the livenet context.
- Update `HostEnv` functions accept `Addressable` instead of `Address`.

## [1.0.0] - 2024-05-23
### Added
- Add `try_deploy` function to the `Deployer` trait, which allows deploying a contract and returning an error if the deployment fails.
- Add `Cep18` to `odra-modules` - a standard for fungible tokens.
- Add `Cep78` to `odra-modules` - a standard for non-fungible tokens.
- Add `cep18` and `cep78` templates.

### Changed
- Update casper crates to the latest versions.
- Fix minor bugs in generated code.
- Fix events collection in submodules.

## [0.9.1] - 2024-04-15
### Changed
- `#[odra::odra_type]` attribute can be applied to all flavors of enums (before applicable to unit-only enums).

## [0.9.0] - 2024-04-02
### Added
- `Maybe<T>` - a type that represents an entrypoint arg that may or may not be present.
- `EntrypointArgument` - a trait for types that can be used as entrypoint arguments.
- an example of using `Maybe<T>` in `odra-examples` crate.
- `get_opt_named_arg_bytes(&str)` in `ContractEnv`
- `odra::contract_def::Argument` has a new `is_required` field.
- `odra-schema` crate for generating contract schema that aligns with Casper Contract Schema.
- `odra::event` proc macro that combines Casper Event Standard and Casper Contract Schema boilerplate code.
- `odra::module` accepts `error` attribute to generate custom error in the schema.
  
### Changed
- update modules and examples to reflect the changes.
- `OdraType` derive macro is now an attribute macro `[odra::odra_type]`.
- `OdraError` derive macro is now an attribute macro `[odra::odra_error]`.

### Fixed
- https://github.com/odradev/odra/issues/391 - compile error when using a type reference in the constructor signature.

## [0.8.1] - 2024-03-01
### Added
- `ContractRef` trait with `new` and `address` functions. All contract references now implement it.
- `disable-allocator` feature for `odra` crate. It allows to disable the allocator used by Odra Framework in
wasm build.
- `odra-build` crate for including the code in contract's build.rs file.

### Changed
- Traits implemented by modules are now also implemented by their `ContractRefs` and `HostRefs`.

## [0.8.0] - 2024-02-06

### Changed
- Replaced `contract_env::` with `self.env()` in the contract context (of type `ContractEnv`).
For example, instead of `contract_env::caller()` use `self.env.caller()`.
- Replaced `test_env` with `odra_test::env()` in the test context (of type `HostEnv`).
- It is no longer possible to put a Mapping inside a Mapping. Use tuple keys instead, for example, instead of
```rust
allowances: Mapping<Address, Mapping<Address, U256>>
```
use
```rust
allowances: Mapping<(Address, Address), U256>
```
- `Ref` has been divided into two: `HostRef` and `ContractRef` depending on the context.
- Storage keys are constructed differently, keep that in mind when querying the storage outside the contract.
- `token_balance(address: Address)` in test context is now `balance_of(address: &Address)`.
- Modules needs to be wrapped in SubModule<> when used in another module.
- `HostRef` (formerly `Ref`) methods parameters accept values even if endpoints expect references.
- `unwrap_or_revert` and `unwrap_or_revert_with` now require `&HostEnv` as a first parameter.
- `Variable` has been renamed to `Var`.
- Spent Gas is taken into account when checking an account balance.
This means that all calls appear to cost 0 gas to the test accounts,
but it is still possible to check the costs of the calls.
- Various changes of method signatures in `odra-modules`, e.g. `&[U256]` changed to `Vec<U256>`
in `on_erc1155_batch_received` method. 
- `get_block_time()` now returns `u64` instead of wrapped type `Blocktime(u64)`.
- The `name` property in `Odra.toml` is not required anymore.
- `odra-casper-backend` crate is now `odra-casper-wasm-env`
- `odra-mock-vm` crate is now `odra-vm`
- `odra-proc-macros` is the only code generation point.

### Removed
- Removed `#[odra::init]` macro - now only one method can be a constructor,
and it has to be named `init`.
- Removed `emit()` method from events. Use `self.env.emit_event(event)` instead.
- Removed `assert_events!` macro. Use `env.emitted_event` and similar methods instead.
- Removed `native_token_metadata()` and its respective structs.
- Removed `assert_exception!` macro. Use `try_` prefix for endpoints instead. It returns a `Result`
which can be used with a regular `assert_eq!` macro.`
- Removed `odra(using)` macro.
- Removed `default()` method from `Deployer`. Use `init()` instead.
- Removed `StaticInstance` and `DynamicInstance` traits. Use `Module` instead.
- Removed `Node` trait.
- Removed `odra-types`, `odra-ir`, `odra-codegen`, `odra-casper-backend`, `odra-casper-shared`, `odra-utils` crates.
- Removed `OdraEvent` trait, `odra` reexports `casper_event_standard` crate instead.

### Added
- `last_call()` in test env with `ContractCallResult` struct which holds all information about the last call.
- `odra::prelude::*` module which reexports all basic types and traits. 
It is recommended to use it in the module code.
- `odra-test` crate providing `HostEnv` for testing purposes.
- `odra-core` crate providing framework core functionalities.
- `odra-casper-rcp-client` crate providing `CasperClient` for interacting with Casper node. 
- `odra-casper-livenet-env` crate providing a host environment fo for the Casper livenet.
- `HostRef` trait - a host side reference to a contract.
- `HostRefLoader` trait for loading a contract from the host environment.
- `EntryPointsCallerProvider` trait.
- `Deployer` trait - a struct implementing that trait can deploy a contract.
- `InitArgs` trait - a struct implementing that trait can be used as a constructor argument.
- `NoArgs` struct, an implementation of `InitArgs` for empty/none (no constructor) arguments.

### Very Brief Upgrade Path
- Copy `bin` folder and `build.rs` files from a freshly generated project
using `cargo-odra` at version at least`0.1.0`.
- Update `Cargo.toml` by removing features and adding [[bin]] sections (See `Cargo.toml` from generated project).
- Update the code to reflect changes in the framework.

## [0.7.0] - 2023-11-07
### Changed
- `no_std` is now the default mode for WASM.
- `odra-types` are the only types crate.

### Removed
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
- derive PartialEq, Eq, Debug from storage building block (Var, Mapping, List).

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
  - Add tests for `Var` and `Mapping`.
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
    - `Var`'s functions: `set()`, `add()`, `subtract()`.
    - `Var` and `Mapping` are implemented for `OdraType`.
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
