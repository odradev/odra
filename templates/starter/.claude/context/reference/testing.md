# Testing Reference

See `overview/testing-model.md` for the big picture on backends (OdraVM, CasperVM, livenet).
This file covers the test API.

## Setting Up a Test

```rust
#[cfg(test)]
mod tests {
    use super::{MyContract, MyContractInitArgs};
    use odra::host::Deployer;

    #[test]
    fn my_test() {
        let env = odra_test::env();
        let mut contract = MyContract::deploy(&env, MyContractInitArgs {
            initial_value: 42,
        });
        assert_eq!(contract.get_value(), 42);
    }
}
```

For contracts with no constructor, use `NoArgs`:

```rust
use odra::host::{Deployer, NoArgs};
let contract = MyContract::deploy(&env, NoArgs);
```

## `HostEnv` — Test Environment API

```rust
let env = odra_test::env();
```

| Method | Description |
|---|---|
| `env.get_account(n)` | Returns the n-th test account (`Address`). Account 0 is the default caller. |
| `env.set_caller(address)` | Change the active caller for subsequent calls |
| `env.advance_block_time(ms)` | Advance the block time by `ms` milliseconds |
| `env.balance_of(&contract_ref)` | Native token balance of a contract or address (`U512`) |
| `env.emitted_event(&contract, event)` | Returns `true` if the event was emitted |
| `env.emitted_native_event(&contract, event)` | Same for native events |
| `env.emitted(&contract, "EventName")` | Returns `true` if any event with that name was emitted |
| `env.events_count(&contract)` | Number of events emitted since last check |

## Calling Entry Points

```rust
// Normal call — panics on revert
contract.increment();
let value = contract.get();

// Try call — returns Result, does not panic on revert
let result = contract.try_increment();
let err = contract.try_restricted_action().unwrap_err();
```

## Switching Callers

```rust
let alice = env.get_account(0);
let bob = env.get_account(1);

env.set_caller(alice);
let mut contract = MyContract::deploy(&env, NoArgs);

env.set_caller(bob);
// subsequent calls are made as bob
contract.some_action();
```

## Native Token Transfers (Payable Calls)

```rust
// Send 100 motes to a payable entry point
contract.with_tokens(U512::from(100)).deposit();

// Check contract balance
let balance = env.balance_of(&contract);
assert_eq!(balance, U512::from(100));
```

## Advancing Time

```rust
env.advance_block_time(1000); // advance 1 second (1000 ms)
let time = contract.get_timestamp(); // should reflect new block time
```

## Asserting Events

See `reference/events.md` for the full events API.

```rust
assert!(env.emitted_event(&contract, MyEvent {
    field: expected_value,
}));
```

## Asserting Errors

See `reference/errors.md` for error definition. Use `.try_method()`:

```rust
let err = contract.try_restricted_action().unwrap_err();
assert_eq!(err, MyError::Unauthorized.into());
```

## Getting a Contract's Env Reference

If you need the env from a deployed contract ref:

```rust
let env = contract.env();
```
