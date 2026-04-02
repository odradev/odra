# Odra Errors

## Defining an Error Enum

Apply `#[odra::odra_error]` to an enum. Each variant needs a unique `u16` discriminant:

```rust
#[odra::odra_error]
pub enum Error {
    OwnerNotSet = 1,
    NotAnOwner = 2,
    InsufficientBalance = 3,
    AlreadyInitialized = 4,
}
```

Discriminants must be unique within the project across all error enums to avoid
collisions in the final WASM binary.

## Registering Errors on the Module

```rust
#[odra::module(errors = Error)]
pub struct OwnedContract { ... }
```

## Reverting with an Error

Use `self.env().revert(error)` — it never returns (the transaction is rolled back):

```rust
pub fn change_name(&mut self, name: String) {
    let owner = self.owner.get_or_revert_with(Error::OwnerNotSet);
    if self.env().caller() != owner {
        self.env().revert(Error::NotAnOwner);
    }
    self.name.set(name);
}
```

Alternatively, implement `Revertible` (via `SubModule<...>` composition) and call
`self.revert(Error::Variant)` directly.

## Convenience Methods on Storage Types

`Var<T>` and `Mapping<K, V>` provide revert shortcuts:

```rust
// Reverts with Error::OwnerNotSet if value is None
let owner = self.owner.get_or_revert_with(Error::OwnerNotSet);
```

## `unwrap_or_revert`

The `UnwrapOrRevert` trait adds `.unwrap_or_revert(self)` to `Option<T>` and `Result<T, E>`.
Import it via `use odra::prelude::*`:

```rust
use odra::prelude::*;

// Option — reverts with a generic ExecutionError if None
let value = self.some_field.get().unwrap_or_revert(self);

// Result — reverts with the error if Err
let parsed = some_result.unwrap_or_revert(self);
```

## Asserting Errors in Tests

Use `.try_method()` to get a `Result` instead of panicking. The error type is `OdraError`:

```rust
let err = contract.try_change_name("Bob".to_string()).unwrap_err();
assert_eq!(err, Error::NotAnOwner.into());
```

## Complete Example

```rust
#[odra::module(errors = Error)]
pub struct Vault {
    owner: Var<Address>,
    balance: Var<U256>,
}

#[odra::odra_error]
pub enum Error {
    NotOwner = 1,
    InsufficientFunds = 2,
}

#[odra::module]
impl Vault {
    pub fn init(&mut self) {
        self.owner.set(self.env().caller());
        self.balance.set(U256::zero());
    }

    pub fn withdraw(&mut self, amount: U256) {
        let owner = self.owner.get_or_revert_with(Error::NotOwner);
        if self.env().caller() != owner {
            self.env().revert(Error::NotOwner);
        }
        let bal = self.balance.get_or_default();
        if bal < amount {
            self.env().revert(Error::InsufficientFunds);
        }
        self.balance.set(bal - amount);
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, Vault};
    use odra::host::{Deployer, NoArgs};

    #[test]
    fn non_owner_cannot_withdraw() {
        let env = odra_test::env();
        env.set_caller(env.get_account(0));
        let mut vault = Vault::deploy(&env, NoArgs);

        env.set_caller(env.get_account(1));
        let err = vault.try_withdraw(100.into()).unwrap_err();
        assert_eq!(err, Error::NotOwner.into());
    }
}
```
