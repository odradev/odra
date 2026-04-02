# Odra Events

## Defining an Event

Apply `#[odra::event]` to a public struct. All fields must implement `CLTyped`:

```rust
#[odra::event]
pub struct Transfer {
    pub from: Address,
    pub to: Address,
    pub amount: U256,
}

#[odra::event]
pub struct Approval {
    pub owner: Address,
    pub spender: Address,
    pub amount: U256,
}
```

## Registering Events on the Module

List emitted events in the module attribute for schema generation:

```rust
#[odra::module(events = [Transfer, Approval])]
pub struct Token { ... }
```

## Emitting Events

Inside an entry point, use `self.env().emit_event(...)`:

```rust
pub fn transfer(&mut self, to: Address, amount: U256) {
    // ... update balances ...
    self.env().emit_event(Transfer {
        from: self.env().caller(),
        to,
        amount,
    });
}
```

## Native Events (Casper-specific)

For events that need to be recorded in Casper's native event schema:

```rust
self.env().emit_native_event(Transfer { from, to, amount });
```

Most contracts use `emit_event`. Use `emit_native_event` only when targeting
Casper's native CEP-47/CEP-78 event standards.

## Asserting Events in Tests

Use `env.emitted_event(&contract_ref, event_value)` to assert a specific event
was emitted. Struct fields can be partially matched:

```rust
assert!(env.emitted_event(
    &token,
    Transfer {
        from: env.get_account(0),
        to: env.get_account(1),
        amount: 100.into(),
    }
));
```

Check if any event with a given name was emitted (without checking fields):

```rust
assert!(env.emitted(&token, "Transfer"));
```

Count total events emitted by a contract since last check:

```rust
assert_eq!(env.events_count(&token), 1);
```

For native events:

```rust
assert!(env.emitted_native_event(&token, Transfer { ... }));
assert!(env.emitted_native(&token, "Transfer"));
assert_eq!(env.native_events_count(&token), 1);
```

## Complete Example

```rust
#[odra::event]
pub struct NameChanged {
    pub old_name: String,
    pub new_name: String,
}

#[odra::module(events = [NameChanged])]
pub struct Registry {
    name: Var<String>,
}

#[odra::module]
impl Registry {
    pub fn init(&mut self, name: String) {
        self.name.set(name);
    }

    pub fn rename(&mut self, new_name: String) {
        let old_name = self.name.get_or_default();
        self.name.set(new_name.clone());
        self.env().emit_event(NameChanged { old_name, new_name });
    }
}

#[cfg(test)]
mod tests {
    use super::{NameChanged, Registry, RegistryInitArgs};
    use odra::host::Deployer;

    #[test]
    fn test_rename_emits_event() {
        let env = odra_test::env();
        let mut registry = Registry::deploy(&env, RegistryInitArgs {
            name: "Alice".to_string(),
        });
        registry.rename("Bob".to_string());
        assert!(env.emitted_event(&registry, NameChanged {
            old_name: "Alice".to_string(),
            new_name: "Bob".to_string(),
        }));
    }
}
```
