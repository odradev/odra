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
