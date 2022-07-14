framework crates:
    - odra
      |-- pub use odra-core::* 
      |-- pub use odra-macros::* 
    - odra-core
      |-- types
        |-- U256
        |-- U512
        |-- Address
      |-- macro-utils
        |-- module-ast
      |-- variable
      |-- mapping
      |-- backend-trait
    - odra-macros
      |-- Instance
      |-- module
      |-- external_contract
    - odra-mock-vm
    - odra-modules
      |-- erc20
      |-- Ownable
backend crates:
    - odra-casper
        |-- pub use odra-casper-macros::*
    - odra-casper-macros 

## Getting Started

### Init odra module.

```bash
cargo odra new mycontract
```

### Build wasm
```bash
cargo build --release --no-default-features --features "wasm"
cargo odra build-wasm -p mycontract
```

### Run tests against MockVM

```bash
cargo test -p mycontract
```

### Run tests against CasperVM
```bash
cargo odra test-with-backend -p mycontract
```

## Modes

### Build WASM using Backend

### Run tests using MockVM

### Run tests using BackendTestVM
