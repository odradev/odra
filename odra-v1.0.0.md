# Odra v1.0.0

## Introduction
Odra is a operating system for the WASM environment and Linux environment.  

## Features
- Memory and storage management.
- System of Modules.
- Pluggable Contexts for different environments.
- Runnable Scripts.
- WASM build system.
- Testing system.
- Type system.
- Error handling.
- Gherkin scripts.
- Remote execution.

## Host interaction.
The only way Odra can communicate and be useful is through the host.
Host is your environment.

Potential hosts are:
    - browser,
    - server,
    - mobile app,
    - blockchain,
    - testing environment,
    - off-chain clients,
    - build script (possible interaction with syn macros),
    - cargo odra,
    - Gherkin scripts runner,

## Modules and States
Module and States is the main components of Odra.
State is a representation of the data that is stored in the host.
Every struct marked as `#[odra::state]` or `#[derive(State)]` is a state.
Every `impl` marked as `#[odra::module]` is a module.
Most of the code that smart contract developers should build are modules
and storages.
The whole business logic should be implemented here.

To access state each entrypoint can do it via context.
```rust

#[odra::storage]
struct Token {
    balance: Variable<u64>,
}

// type Result<T> = std::result::Result<T, E: Error>;

#[odra::module]
impl Token {
    type Error = TokenError;

    fn balance_of(&self, address: Address) -> Result<u64> {
        Ok(self.balance.get(address).unwrap_or_default())
    }
}

#[odra::test(MockVM, CasperWasmTest)]
fn it_works() {
    let ctx = odra::ctx();
    let token = Token::new(ctx)?;
}
```

Odra Modules provides a few modules:
    - variable
    - mapping
    - list
    - ownable

Entrypoints returns a `Result` type.

### Result type


## Remote module
Remote module is a representation of a smart contract.
In future it might be different thing like a P2P Network Node.
The call can be synchronous or asynchronous.

### Interacting with the host.

## Contexts
Context is the bridge interface between the host and the module.
Most of the backend code is written as a Context implementations.
Context is a granular interface for different modes execution.

ContextInfo:
- context_name
- caller_address
- block_number
- time
- self_address

HashContext:
- hash

SignatureContext:
- verify_signature

StorageReaderContext:
- get_key

BaseContext:
    - ContextInfo
    - HashContext
    - SignatureContext
    - StorageReaderContext

WriterContext:
- set_key
- emit_event

EventReaderContext:
- read_event

ContractCallerContext:
- lock
- unlock
- call_contract
- call_contract_mut

ContractDeployerContext:
- new_contract

ContractUpgraderContext:
- upgrade_contract

WalletContext:
- get_address_at
- set_caller
- set_gas_price

BuilderContext:
- write_contract_schema

OnChainContext:
    BaseContext
    StorageWriterContext
    EventWriterContext
    ContractCallerContext

OffChainContext:
    BaseContext
    ContractCallerContext
    WalletContext

## Testing
At Odra we believe that testing is a crucial part of the development process.
It proves to the developer that hers code is working as expected.
We provide tools to write unit tests for modules and integration tests for
the whole project.
BDD tests with Gherkin are also supported.

## Gherkin scripts
Based on the contract definition we generate `cucumber` bindings.
This allows writing large tests scripts in a human readable language.
We also proved that these scripts can be used as a documentation
and are a great way to communicate with non-technical people.
Gherkin scripts are usable in tests and in the production environment.

## Local execution of remote state
In `odra-casper-client` we provided a way of running Odra Module
locally against the remote state of the smart contract deployed
on the blockchain.
It only supports read-only entry points.

It is also possible to have similar mode of execution, but when
a state write happens it is persisted locally and now-on used as a state.
This mode basically uses the blockchain as a source of data,
but later uses local state for the execution.
It allows for simulating the execution of the smart contract locally
against the remote state.

## Errors handling
We go full Rust-mode with errors.
We require every module to provide a Error type.
All entry points return an `Result` type.
Including the `init` entry point and read-only entry points.

## Odra Backends
Backend should provide implementation for some of those traits:
- OnChainContext
- OffChainContext

### Casper Example
CasperOnchainWasmContext:
    OnChainContext

CasperLocalTestContext:
    OffChainContext

CasperLivenetContext:
    OffChainContext

CasperLivenetSimulationContext:
    OffChainContext

CasperBuilder:
    BuilderContext
