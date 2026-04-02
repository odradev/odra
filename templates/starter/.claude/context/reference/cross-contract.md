# Cross-Contract Calls

## Two Patterns

Odra supports two cross-contract patterns:

1. **`External<T>` field** — stores a contract address as a field, calls it like a sub-field
2. **`ContractRef::new()`** — constructs a ref on the fly from an address (no storage field)

Both produce the same on-chain behavior. `External<T>` is cleaner when the address is
stored long-term; `ContractRef::new()` is useful for one-off or injected addresses.

## Pattern 1: `External<T>` Field

Define the external contract's interface with `#[odra::external_contract]`:

```rust
#[odra::external_contract]
pub trait Token {
    fn balance_of(&self, owner: &Address) -> U256;
    fn transfer(&mut self, to: &Address, amount: &U256);
}
```

This generates a `TokenContractRef` type. Store it in a contract field:

```rust
#[odra::module]
pub struct Wallet {
    token: External<TokenContractRef>,
}

#[odra::module]
impl Wallet {
    pub fn init(&mut self, token_address: Address) {
        self.token.set(token_address);
    }

    pub fn my_balance(&self) -> U256 {
        self.token.balance_of(&self.env().caller())
    }
}
```

## Pattern 2: `ContractRef::new()`

When you have the address but don't store it permanently:

```rust
use odra::ContractRef;

pub fn query_balance(&self, token_address: Address, owner: &Address) -> U256 {
    TokenContractRef::new(self.env(), token_address).balance_of(owner)
}
```

## Using Cross-Contract in Tests

Deploy both contracts, wire the address:

```rust
#[test]
fn test_wallet() {
    let env = odra_test::env();

    // Deploy the token first
    let token = MyToken::deploy(&env, MyTokenInitArgs {
        initial_supply: 1000.into(),
    });

    // Deploy wallet with token address
    let wallet = Wallet::deploy(&env, WalletInitArgs {
        token_address: token.address(),
    });

    assert_eq!(wallet.my_balance(), 1000.into());
}
```

## Important: Mutable vs Immutable Calls

- Entry points that mutate state (`&mut self`) require the `ContractRef` to be `mut`
- Read-only entry points (`&self`) work on an immutable ref

```rust
// Mutable cross-contract call
TokenContractRef::new(self.env(), addr).transfer(&recipient, &amount);

// Read-only cross-contract call
let supply = TokenContractRef::new(self.env(), addr).total_supply();
```
