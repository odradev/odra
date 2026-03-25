# Storage Key Redesign

## Problem

The current storage key creation mechanism limits each Odra module to 15 fields. The `ContractEnv::child(index)` method packs field indices into a `u32` using 4-bit left shifts: `(parent << 4) + child`. With 4 bits per nesting level, field indices are capped at 1-15, and nesting depth at 8 levels. This is becoming a practical limitation for contracts that need more fields.

## Goals

- Increase the per-module field limit to at least 254
- Preserve full backward compatibility with deployed contracts (identical storage keys for existing field layouts)
- Maintain at least 6 nesting levels (current practical maximum)

## Design

### Encoding Modes

Two encoding modes, selected per-module:

**Legacy mode (default):** Fields 1-15 use the current 4-bit-shift `u32` formula. Fields 16+ use a new path encoding with a `0xFF` prefix. Existing contracts are unaffected.

**V2 mode (opt-in via `#[odra::module(keys = "v2")]`):** All fields use the path encoding regardless of field count. For brand new contracts only — breaks storage compatibility by design.

### Key Encoding

The `current_key()` method hashes `index_bytes ++ mapping_data` with blake2b, then hex-encodes to a 64-byte `StorageKey`. The change is in how `index_bytes` are produced.

**Legacy encoding (fields 1-15, all path indices <= 15):**
```
index_bytes = u32::to_be_bytes()
```
Where `u32` is computed as `path.iter().fold(0u32, |acc, &idx| (acc << 4) + idx as u32)`.

This produces byte-identical output to the current implementation.

**Path encoding (fields 16+, or V2 mode):**
```
index_bytes = [0xFF, path_len, path[0], path[1], ..., path[path_len-1]]
```

The `0xFF` prefix byte acts as a version tag. It cannot collide with legacy keys because the first byte of a legacy `u32` in big-endian never exceeds `0x0F` (worst case: 8 levels of field 15 = `0x0FFFFFFF`).

The `path_len` byte makes the encoding self-delimiting, preventing collisions between path bytes and appended `mapping_data`. Without it, a `Var` at path `[3, 16]` and a `Mapping` at path `[3]` with key `[0x10]` would produce identical hash inputs.

### Mode Selection

In legacy mode, `current_key()` checks whether all indices in the path are <= 15 (`can_use_legacy()`). If yes, legacy encoding. If any index exceeds 15, path encoding is used automatically. This means:

- Existing modules with <= 15 fields: identical storage keys
- Upgraded modules with new fields 16+: fields 1-15 keep old keys, fields 16+ get path-encoded keys
- Nested modules where a parent exceeds 15 fields: children use path encoding (correct, since this combination never existed in deployed contracts)

### Data Model

```rust
const MAX_PATH_LEN: usize = 8;

enum KeyEncoding {
    Legacy,  // default
    V2,      // opt-in via #[odra::module(keys = "v2")]
}

struct ContractEnv {
    path: [u8; MAX_PATH_LEN],              // field indices from root to current node
    path_len: u8,                           // number of valid entries in path
    encoding: KeyEncoding,
    mapping_data: Vec<u8>,
    backend: Rc<RefCell<dyn ContractContext>>,
}
```

Using a fixed-size `[u8; 8]` array avoids heap allocation for the path. With max 6 practical nesting levels, 8 slots provide comfortable headroom.

**`child()` implementation:**
```rust
fn child(&self, index: u8) -> Self {
    let mut new_path = self.path;
    new_path[self.path_len as usize] = index;
    Self {
        path: new_path,
        path_len: self.path_len + 1,
        encoding: self.encoding,
        mapping_data: self.mapping_data.clone(),
        backend: self.backend.clone(),
    }
}
```

**`current_key()` implementation:**
```rust
fn current_key(&self) -> StorageKey {
    let path = &self.path[..self.path_len as usize];
    let index_bytes = match self.encoding {
        KeyEncoding::Legacy if path.iter().all(|&idx| idx <= 15) => {
            let index: u32 = path.iter().fold(0u32, |acc, &idx| (acc << 4) + idx as u32);
            index.to_be_bytes().to_vec()
        }
        _ => {
            let mut bytes = Vec::with_capacity(2 + path.len());
            bytes.push(0xFF);
            bytes.push(path.len() as u8);
            bytes.extend_from_slice(path);
            bytes
        }
    };

    let mut key = Vec::with_capacity(index_bytes.len() + self.mapping_data.len());
    key.extend_from_slice(&index_bytes);
    key.extend_from_slice(&self.mapping_data);

    let mut result = [0u8; KEY_LEN];
    let hashed_key = self.backend.borrow().hash(&key);
    utils::hex_to_slice(&hashed_key, &mut result);
    result
}
```

### Macro Changes

**Remove field limit:** In `odra-macros/src/ast/module_def.rs`, `MAX_FIELDS` (currently 15) is raised to 255. The `u8` index type naturally caps field indices at 255. Index 0 is valid but reserved for internal component use (e.g., `List` uses index 0 for its internal `Mapping` and index 1 for its length `Var`). The macro assigns user fields starting at index 1 via `enumerate() + 1`.

**Support `keys = "v2"` attribute:** The `#[odra::module]` macro gains an optional parameter:
```rust
#[odra::module]                    // Legacy mode (default)
#[odra::module(keys = "v2")]       // V2 mode
```

The macro parses the `keys` attribute and propagates the encoding mode when constructing the root `ContractEnv`.

**Field index assignment** stays unchanged: `enumerate() + 1`, but indices can now exceed 15.

### V2 Mode and Nesting

The `keys = "v2"` attribute applies only when the annotated module is the **root contract**. The encoding mode is stored in `ContractEnv` and propagated to children via `child()`. A `SubModule` inside a legacy root inherits legacy encoding regardless of its own `keys` attribute — the root determines the mode for the entire contract tree. Conversely, a V2 root passes V2 encoding to all its children.

### Constructor Change

The current public constructor:
```rust
pub const fn new(index: u32, backend: Rc<RefCell<dyn ContractContext>>) -> Self
```

Changes to:
```rust
pub const fn new(encoding: KeyEncoding, backend: Rc<RefCell<dyn ContractContext>>) -> Self
```

The root `ContractEnv` starts with an empty path (`path_len = 0`). The `index` parameter is no longer needed since path building happens via `child()`. All callers of `ContractEnv::new()` must be updated — primarily the macro-generated code and test utilities.

### Backward Compatibility

| Scenario | Behavior |
|----------|----------|
| Existing contract, <= 15 fields per module | Identical storage keys (legacy encoding) |
| Upgrade: add field 16+ to existing contract | Fields 1-15 unchanged, field 16+ path-encoded |
| Nested: parent <= 15, child <= 15 | Legacy encoding at all levels |
| Nested: parent has field 16+, child <= 15 | Path encoding (correct: this combination never existed before) |
| Brand new contract with `keys = "v2"` | Path encoding for all fields |

### Collision Safety

1. **Legacy vs path encoding:** Legacy first byte maxes at `0x0F`. Path encoding starts with `0xFF`. No overlap.
2. **Path encoding vs mapping data:** The `path_len` byte makes the boundary unambiguous. `[0xFF, 2, 3, 16] ++ mapping_data` cannot be confused with `[0xFF, 1, 3] ++ [16, ...mapping_data]`.
3. **Within same encoding:** Different paths produce different byte sequences. Same path with different mapping data produces different byte sequences.

## Testing Strategy

### Unit tests (odra-core)

1. **Legacy compatibility:** Build paths with indices <= 15 at various depths, assert `current_key()` matches the current `(parent << 4) + child` u32 formula.
2. **Path encoding:** Paths with indices > 15, verify pre-hash bytes are `[0xFF, len, path...]`.
3. **No-collision:** Comprehensive set of (path, mapping_data) pairs, verify unique pre-hash inputs.
4. **Mixed mode:** Path `[3, 16]` uses path encoding, not legacy.
5. **V2 mode:** Same path `[3, 5]` with V2 produces different keys than legacy.

### Macro tests (odra-macros)

6. **>15 fields compiles:** Module with 20+ fields assigns indices 1-20.
7. **V2 attribute parsing:** `#[odra::module(keys = "v2")]` parses and propagates encoding mode.

### Integration tests (examples)

8. **Large module:** Contract with ~20 fields, verify storage read/write on OdraVM.
9. **Upgrade scenario:** Deploy with 15 fields, write data, upgrade to 16+, verify original fields readable.

## Files to Modify

### Code changes required
- `core/src/contract_env.rs` — `ContractEnv` struct (`index: u32` → `path`/`path_len`/`encoding`), `child()`, `current_key()`, `new()` signature
- `odra-macros/src/ast/module_def.rs` — raise `MAX_FIELDS` from 15 to 255
- `odra-macros/src/lib.rs` or IR layer — parse `keys = "v2"` attribute
- `odra-macros/src/ast/module_item.rs` — propagate encoding mode to `ContractEnv::new()` call
- `odra-macros/src/test_utils.rs` — update `invalid_module_definition()` test (currently expects error at 16 fields)

### Verified unchanged (depend on `ContractEnv` but need no code changes)
- `core/src/module.rs` — `ModuleComponent::instance` signature unchanged, `SubModule::module()` calls `self.env.child()` which has the same API
- `core/src/var.rs` — stores `index: u8`, calls `self.env.child(self.index)` — unchanged API
- `core/src/mapping.rs` — clones `ContractEnv` and calls `add_to_mapping_data()` — unchanged API
- `core/src/list.rs` — uses `child()` with indices 0 and 1 internally — unchanged API, both indices <= 15 so legacy encoding applies
- `core/src/sequence.rs` — same pattern as `Var`, unchanged API
- `odra-macros/src/ir/mod.rs` — field index assignment already uses `enumerate() + 1`, no change needed
