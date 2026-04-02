# Odra Testing Model

## Three Backends

### OdraVM (default — use for unit tests)

Fast in-memory VM. No blockchain, no WASM. Runs in milliseconds.

```bash
cargo odra test           # from the contracts/ or project root
```

Use OdraVM for: all contract logic tests, event assertions, error assertions,
caller-switching tests, balance tests.

### CasperVM (use for integration tests)

Full Casper execution engine. Compiles contracts to WASM and runs them through
the real Casper runtime. Much slower than OdraVM but catches WASM-specific issues.

```bash
cargo odra test -b casper
```

Use CasperVM for: pre-deployment validation, testing contracts that interact
with Casper-specific types, verifying WASM compilation succeeds.

### Livenet (use for deployment)

Real Casper blockchain node. Requires a running node (NCTL locally or testnet/mainnet).
Not used for automated tests — used via the CLI binary in `cli/`.

```bash
# from cli/ directory
cargo run --bin cli --features=livenet -- deploy
```

Use livenet for: deploying to nctl/testnet/mainnet, running scenarios against a live node.

## Switching Backends

The backend is selected via the `ODRA_BACKEND` environment variable:

```bash
ODRA_BACKEND=casper cargo odra test    # force CasperVM
cargo odra test                         # OdraVM (default)
```

In test code, `odra_test::env()` returns the correct backend automatically.

## Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::{MyContract, MyContractInitArgs};
    use odra::host::Deployer;

    #[test]
    fn my_test() {
        let env = odra_test::env();
        let mut contract = MyContract::deploy(&env, MyContractInitArgs { /* ... */ });
        // call entry points on contract
        // assert using env.emitted_event, env.balance_of, etc.
    }
}
```

For contracts with no constructor args, import and use `NoArgs`:

```rust
use odra::host::{Deployer, NoArgs};
let contract = MyContract::deploy(&env, NoArgs);
```

## Key Testing Utilities

See `reference/testing.md` for full API details.

- `env.set_caller(address)` — change the active caller
- `env.advance_block_time(ms)` — move block time forward
- `env.balance_of(&contract_ref)` — check native token balance
- `env.emitted_event(&contract, Event { .. })` — assert event was emitted
- `contract.try_method()` — call that returns Result instead of panicking
