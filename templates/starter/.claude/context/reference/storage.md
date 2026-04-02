# Odra Storage Types

All storage fields are declared on the contract struct. They read from and write to
the blockchain's key-value store — not in-memory. Access them via `self.field_name`.

## `Var<T>` — single value

```rust
name: Var<String>,
count: Var<u32>,
owner: Var<Address>,
```

| Method | Description |
|---|---|
| `self.name.set(value)` | Write a value |
| `self.name.get()` | Returns `Option<T>` |
| `self.name.get_or_default()` | Returns `T`, uses `Default::default()` if unset |
| `self.name.get_or_revert_with(err)` | Returns `T`, reverts if unset |
| `self.name.is_none()` | Returns `true` if no value has been set |

**Common pattern — initialize in `init`, update in entry points:**

```rust
pub fn init(&mut self, name: String) {
    self.name.set(name);
}

pub fn rename(&mut self, new_name: String) {
    self.name.set(new_name);
}

pub fn get_name(&self) -> String {
    self.name.get_or_default()
}
```

## `Mapping<K, V>` — key-value store

```rust
balances: Mapping<Address, U256>,
visits: Mapping<String, u32>,
```

| Method | Description |
|---|---|
| `self.balances.set(&key, value)` | Write a value for a key |
| `self.balances.get(&key)` | Returns `Option<V>` |
| `self.balances.get_or_default(&key)` | Returns `V`, uses `Default::default()` if unset |

**Common pattern:**

```rust
pub fn add_visit(&mut self, name: &String) {
    let count = self.visits.get_or_default(name);
    self.visits.set(name, count + 1);
}

pub fn visit_count(&self, name: &String) -> u32 {
    self.visits.get_or_default(name)
}
```

## `List<T>` — append-only list

```rust
walks: List<u32>,
items: List<Address>,
```

| Method | Description |
|---|---|
| `self.walks.push(value)` | Append a value |
| `self.walks.len()` | Number of elements (`u32`) |
| `self.walks.iter()` | Iterator over values |
| `self.walks.pop()` | Remove and return last element (`Option<T>`) |

**Common pattern:**

```rust
pub fn add_walk(&mut self, distance: u32) {
    self.walks.push(distance);
}

pub fn total_distance(&self) -> u32 {
    self.walks.iter().sum()
}
```

## `Sequence<T>` — auto-incrementing counter

```rust
next_id: Sequence<u32>,
```

| Method | Description |
|---|---|
| `self.next_id.get_current_value()` | Read without incrementing |
| `self.next_id.next_value()` | Increment and return new value |

**Common pattern — generating unique IDs:**

```rust
pub fn create_item(&mut self) -> u32 {
    self.next_id.next_value()
}
```

## Storing Contract References: `External<T>`

To store a reference to another contract (for cross-contract calls):

```rust
token: External<TokenContractRef>,
```

| Method | Description |
|---|---|
| `self.token.set(address)` | Store a contract address |
| `self.token.method(args)` | Call a method on the referenced contract |

See `reference/cross-contract.md` for the full cross-contract pattern.
