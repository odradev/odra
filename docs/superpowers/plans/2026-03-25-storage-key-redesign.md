# Storage Key Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the 4-bit-shift storage key encoding with a path-based encoding that supports up to 255 fields per module while preserving backward compatibility with deployed contracts.

**Architecture:** `ContractEnv.index: u32` is replaced with `path: [u8; 8]` + `path_len: u8` + `encoding: KeyEncoding`. Legacy encoding (4-bit shift into u32) is used when all path indices are <= 15. Path encoding (`[0xFF, len, path...]`) is used for indices > 15 or when V2 mode is opted in. The `#[odra::module]` macro's field limit is raised from 15 to 255 and gains a `keys = "v2"` attribute.

**Tech Stack:** Rust, syn/quote proc macros, blake2b hashing

**Spec:** `docs/superpowers/specs/2026-03-25-storage-key-redesign.md`

---

### Task 1: Add `KeyEncoding` enum, update `ContractEnv` struct, and re-export

**Files:**
- Modify: `core/src/contract_env.rs:16-18` (constants), `core/src/contract_env.rs:39-44` (struct), `core/src/contract_env.rs:52-60` (constructor)
- Modify: `core/src/lib.rs:46` — add `KeyEncoding` to re-exports
- Modify: `odra/src/lib.rs:51-54` — add `KeyEncoding` to re-exports

- [ ] **Step 1: Add `KeyEncoding` enum and `MAX_PATH_LEN` constant**

In `core/src/contract_env.rs`, add after the existing constants (line 18):

```rust
/// Maximum nesting depth for module paths.
pub(crate) const MAX_PATH_LEN: usize = 8;

/// Determines how storage keys are encoded.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KeyEncoding {
    /// Default: fields 1-15 use legacy 4-bit u32 encoding, fields 16+ use path encoding.
    Legacy,
    /// All fields use path encoding. For new contracts only.
    V2,
}
```

- [ ] **Step 2: Re-export `KeyEncoding` from `odra-core` and `odra` facade**

In `core/src/lib.rs:46`, add `KeyEncoding`:

```rust
pub use contract_env::{ContractEnv, ContractRef, ExecutionEnv, KeyEncoding};
```

In `odra/src/lib.rs:51-54`, add `KeyEncoding`:

```rust
pub use odra_core::{
    AddressError, CallDef, CollectionError, ContractCallResult, ContractContext, ContractEnv,
    ContractRef, DeployReport, EventError, ExecutionEnv, GasReport, KeyEncoding, OdraContract, VmError
};
```

- [ ] **Step 3: Replace `index: u32` with path-based fields in `ContractEnv`**

Change the struct (currently at line 40):

```rust
#[derive(Clone)]
pub struct ContractEnv {
    path: [u8; MAX_PATH_LEN],
    path_len: u8,
    encoding: KeyEncoding,
    mapping_data: Vec<u8>,
    backend: Rc<RefCell<dyn ContractContext>>
}
```

- [ ] **Step 3: Update the constructor**

Replace `new()` at line 54:

```rust
pub const fn new(encoding: KeyEncoding, backend: Rc<RefCell<dyn ContractContext>>) -> Self {
    Self {
        path: [0u8; MAX_PATH_LEN],
        path_len: 0,
        encoding,
        mapping_data: Vec::new(),
        backend
    }
}
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo check -p odra-core 2>&1 | head -30`
Expected: Compilation errors in `child()`, `current_key()`, and callers of `new()` — these are addressed in subsequent tasks.

- [ ] **Step 6: Commit**

```bash
git add core/src/contract_env.rs core/src/lib.rs odra/src/lib.rs
git commit -m "refactor: replace ContractEnv index with path-based fields"
```

---

### Task 2: Implement `child()` and `current_key()` with dual encoding

**Files:**
- Modify: `core/src/contract_env.rs:62-85` (`current_key`, `child`)

- [ ] **Step 1: Write a test for legacy encoding compatibility**

Add at the bottom of `core/src/contract_env.rs` (or in a new test module):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract_context::MockContractContext;

    fn make_env(encoding: KeyEncoding) -> ContractEnv {
        let mut ctx = MockContractContext::new();
        ctx.expect_hash().returning(|input| {
            // Simple identity-ish hash for testing: just copy input into [u8; 32]
            let mut result = [0u8; 32];
            for (i, byte) in input.iter().enumerate() {
                if i < 32 {
                    result[i] = *byte;
                }
            }
            result
        });
        ContractEnv::new(encoding, Rc::new(RefCell::new(ctx)))
    }

    /// Helper: compute the legacy u32 index the old way for a given path.
    fn legacy_u32_for_path(path: &[u8]) -> u32 {
        path.iter().fold(0u32, |acc, &idx| (acc << 4) + idx as u32)
    }

    #[test]
    fn legacy_encoding_matches_old_u32_formula() {
        let env = make_env(KeyEncoding::Legacy);
        // path [3] → u32 = 3
        let child = env.child(3);
        assert_eq!(child.index_bytes(), legacy_u32_for_path(&[3]).to_be_bytes().to_vec());

        // path [3, 15] → u32 = (3 << 4) + 15 = 63
        let grandchild = child.child(15);
        assert_eq!(grandchild.index_bytes(), legacy_u32_for_path(&[3, 15]).to_be_bytes().to_vec());

        // path [1, 2, 3, 4] → nested
        let deep = env.child(1).child(2).child(3).child(4);
        assert_eq!(deep.index_bytes(), legacy_u32_for_path(&[1, 2, 3, 4]).to_be_bytes().to_vec());
    }

    #[test]
    fn path_encoding_used_for_indices_above_15() {
        let env = make_env(KeyEncoding::Legacy);
        let child = env.child(3).child(16);
        let bytes = child.index_bytes();
        assert_eq!(bytes[0], 0xFF);
        assert_eq!(bytes[1], 2); // path_len
        assert_eq!(bytes[2], 3);
        assert_eq!(bytes[3], 16);
    }

    #[test]
    fn v2_encoding_always_uses_path() {
        let env = make_env(KeyEncoding::V2);
        let child = env.child(3);
        let bytes = child.index_bytes();
        assert_eq!(bytes[0], 0xFF);
        assert_eq!(bytes[1], 1); // path_len
        assert_eq!(bytes[2], 3);
    }

    #[test]
    fn v2_and_legacy_produce_different_keys_for_same_path() {
        let legacy_env = make_env(KeyEncoding::Legacy);
        let v2_env = make_env(KeyEncoding::V2);
        let legacy_key = legacy_env.child(3).child(5).current_key();
        let v2_key = v2_env.child(3).child(5).current_key();
        assert_ne!(legacy_key, v2_key);
    }

    #[test]
    fn no_collision_between_var_and_mapping() {
        let env = make_env(KeyEncoding::Legacy);

        // Case 1: Var at path [3, 16] vs Mapping at path [3] with key [16]
        let var_key = env.child(3).child(16).current_key();
        let mut map_env = env.child(3);
        map_env.add_to_mapping_data(&[16]);
        let map_key = map_env.current_key();
        assert_ne!(var_key, map_key);

        // Case 2: Different path lengths that could alias without length prefix
        let var_key2 = env.child(3).child(1).current_key();
        let mut map_env2 = env.child(3);
        map_env2.add_to_mapping_data(&[0xFF, 2, 3, 1]);
        let map_key2 = map_env2.current_key();
        assert_ne!(var_key2, map_key2);
    }

    #[test]
    fn no_collision_between_legacy_and_path_encoding() {
        let env = make_env(KeyEncoding::Legacy);
        // Legacy path [1, 2] → u32 = 0x12 → bytes [0, 0, 0, 18]
        let legacy_key = env.child(1).child(2).current_key();
        // Path with index > 15 forces path encoding
        let path_key = env.child(1).child(20).current_key();
        assert_ne!(legacy_key, path_key);
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p odra-core tests::legacy_encoding -- 2>&1 | head -20`
Expected: FAIL — `index_bytes()` method does not exist yet.

- [ ] **Step 3: Implement `index_bytes()`, update `child()` and `current_key()`**

Replace `current_key()` and `child()` in `core/src/contract_env.rs`:

```rust
/// Returns the index bytes for the current path, using the appropriate encoding.
pub(crate) fn index_bytes(&self) -> Vec<u8> {
    let path = &self.path[..self.path_len as usize];
    match self.encoding {
        KeyEncoding::Legacy if path.iter().all(|&idx| idx <= 15) => {
            let index: u32 = path.iter().fold(0u32, |acc, &idx| (acc << 4) + idx as u32);
            index.to_be_bytes().to_vec()
        }
        _ => {
            let mut bytes = Vec::with_capacity(2 + path.len());
            bytes.push(0xFF);
            bytes.push(self.path_len);
            bytes.extend_from_slice(path);
            bytes
        }
    }
}

/// Returns the current storage key for the contract environment.
pub(crate) fn current_key(&self) -> StorageKey {
    let mut result = [0u8; KEY_LEN];
    let index_bytes = self.index_bytes();
    let mut key = Vec::with_capacity(index_bytes.len() + self.mapping_data.len());
    key.extend_from_slice(&index_bytes);
    key.extend_from_slice(&self.mapping_data);
    let hashed_key = self.backend.borrow().hash(key.as_slice());
    utils::hex_to_slice(&hashed_key, &mut result);
    result
}

/// Returns a child contract environment with the specified index.
pub(crate) fn child(&self, index: u8) -> Self {
    assert!(
        (self.path_len as usize) < MAX_PATH_LEN,
        "Module nesting depth exceeds maximum of {}",
        MAX_PATH_LEN
    );
    let mut new_path = self.path;
    new_path[self.path_len as usize] = index;
    Self {
        path: new_path,
        path_len: self.path_len + 1,
        encoding: self.encoding,
        mapping_data: self.mapping_data.clone(),
        backend: self.backend.clone()
    }
}
```

Also remove the now-unused `INDEX_SIZE` constant (line 16: `const INDEX_SIZE: usize = 4;`).

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p odra-core contract_env::tests -- 2>&1 | tail -20`
Expected: All 4 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add core/src/contract_env.rs
git commit -m "feat: implement dual-mode storage key encoding (legacy + path)"
```

---

### Task 3: Update all `ContractEnv::new()` callers

**Files:**
- Modify: `odra-vm/src/odra_vm_host.rs:213`
- Modify: `odra-casper/wasm-env/src/wasm_contract_env.rs:137`
- Modify: `odra-casper/livenet-env/src/livenet_host.rs:46`
- Modify: `core/src/args.rs:219,237`
- Modify: `core/src/contract_container.rs:125`

All callers currently pass `ContractEnv::new(0, backend)`. Change to `ContractEnv::new(KeyEncoding::Legacy, backend)`.

- [ ] **Step 1: Update `odra-vm/src/odra_vm_host.rs:213`**

```rust
// Before:
let contract_env = Rc::new(ContractEnv::new(0, OdraVmContractEnv::new(vm.clone())));
// After:
let contract_env = Rc::new(ContractEnv::new(KeyEncoding::Legacy, OdraVmContractEnv::new(vm.clone())));
```

Add the import at the top of the file: `use odra_core::KeyEncoding;`

- [ ] **Step 2: Update `odra-casper/wasm-env/src/wasm_contract_env.rs:137`**

```rust
// Before:
ContractEnv::new(0, Rc::new(RefCell::new(WasmContractEnv)))
// After:
ContractEnv::new(KeyEncoding::Legacy, Rc::new(RefCell::new(WasmContractEnv)))
```

Add the import: `use odra_core::KeyEncoding;`

- [ ] **Step 3: Update `odra-casper/livenet-env/src/livenet_host.rs:46`**

```rust
// Before:
let contract_env = Rc::new(ContractEnv::new(0, livenet_contract_env));
// After:
let contract_env = Rc::new(ContractEnv::new(KeyEncoding::Legacy, livenet_contract_env));
```

Add the import: `use odra_core::KeyEncoding;`

- [ ] **Step 4: Update test callers in `core/src/args.rs` (lines 219, 237) and `core/src/contract_container.rs` (line 125)**

All three call sites: `ContractEnv::new(0, ...)` → `ContractEnv::new(KeyEncoding::Legacy, ...)`

- [ ] **Step 5: Verify compilation**

Run: `cargo check 2>&1 | tail -20`
Expected: Clean compilation (or only macro-related errors from downstream crates, addressed in Task 5).

- [ ] **Step 6: Commit**

```bash
git add odra-vm/src/odra_vm_host.rs odra-casper/wasm-env/src/wasm_contract_env.rs odra-casper/livenet-env/src/livenet_host.rs core/src/args.rs core/src/contract_container.rs
git commit -m "refactor: update ContractEnv::new callers to use KeyEncoding"
```

---

### Task 4: Raise `MAX_FIELDS` and add `keys` attribute parsing in macros

**Files:**
- Modify: `odra-macros/src/ast/module_def.rs:4` — raise `MAX_FIELDS`
- Modify: `odra-macros/src/ir/config.rs:24-30,33-39,42-89` — add `keys` keyword and field
- Modify: `odra-macros/src/test_utils.rs:224-247` — update invalid test mock

- [ ] **Step 1: Raise `MAX_FIELDS` in `odra-macros/src/ast/module_def.rs`**

```rust
// Before:
const MAX_FIELDS: usize = 15;
// After:
const MAX_FIELDS: usize = 255;
```

- [ ] **Step 2: Add `keys` keyword and field to `ModuleConfiguration` in `odra-macros/src/ir/config.rs`**

Add the custom keyword:

```rust
mod kw {
    syn::custom_keyword!(name);
    syn::custom_keyword!(version);
    syn::custom_keyword!(events);
    syn::custom_keyword!(errors);
    syn::custom_keyword!(factory);
    syn::custom_keyword!(keys);
}
```

Add the field to `ModuleConfiguration`:

```rust
#[derive(Default, Clone)]
pub struct ModuleConfiguration {
    pub events: ModuleEvents,
    pub errors: ModuleErrors,
    pub name: ModuleName,
    pub version: ModuleVersion,
    pub factory: Factory,
    pub keys: ModuleKeys,
}
```

Add the `ModuleKeys` type:

```rust
#[derive(Default, Clone, Debug)]
pub struct ModuleKeys(bool); // true = V2, false = Legacy

impl Deref for ModuleKeys {
    type Target = bool;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Parse for ModuleKeys {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<kw::keys>()?;
        input.parse::<Token![=]>()?;
        let value = input.parse::<syn::LitStr>()?;
        match value.value().as_str() {
            "v2" => Ok(Self(true)),
            other => Err(syn::Error::new_spanned(
                value,
                format!("Unknown keys mode '{}'. Expected 'v2'.", other)
            ))
        }
    }
}
```

Add parsing in the `while !input.is_empty()` loop in `ModuleConfiguration::parse`:

```rust
if keys.is_none() && input.peek(kw::keys) {
    keys = Some(input.parse::<ModuleKeys>()?);
    let _ = input.parse::<Token![,]>(); // optional comma
    continue;
}
```

And in the `Ok(Self { ... })` block: `keys: keys.unwrap_or_default()`

- [ ] **Step 3: Update `invalid_module_definition` test in `odra-macros/src/test_utils.rs`**

The current mock has 16 fields and is expected to fail. With `MAX_FIELDS = 255`, it will succeed. Update the test to use 256 fields or remove it and add a new test that validates the 255 limit.

Simplest approach — change the test in `odra-macros/src/ast/module_def.rs:82-86`:

```rust
#[test]
fn test_invalid_module_definition() {
    let ir = mock::invalid_module_definition();
    let def = ModuleDefItem::try_from(&ir);
    // With MAX_FIELDS = 255, 16 fields is valid
    assert!(def.is_ok());
}
```

And update `invalid_module_definition()` in `test_utils.rs` to produce 256 fields for a separate overflow test if desired, or simply remove the assertion and rename the test.

- [ ] **Step 4: Verify macro tests pass**

Run: `cargo test -p odra-macros 2>&1 | tail -20`
Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add odra-macros/src/ast/module_def.rs odra-macros/src/ir/config.rs odra-macros/src/test_utils.rs
git commit -m "feat: raise MAX_FIELDS to 255 and add keys attribute parsing"
```

---

### Task 5: Propagate encoding mode from macro to `ContractEnv`

**Files:**
- Modify: `odra-macros/src/utils/expr.rs:73-75` — parameterize `new_wasm_contract_env`
- Modify: `odra-macros/src/ast/wasm_parts.rs` — all `WasmContractEnv::new_env()` call sites (both via utility and hard-coded in `quote!`/`parse_quote!` blocks)
- Modify: `odra-macros/src/ast/factory/parts/wasm_parts.rs` — same (many hard-coded sites)
- Modify: `odra-macros/src/ir/mod.rs` — expose `keys` config on the IR
- Modify: `odra-casper/wasm-env/src/wasm_contract_env.rs:134-139` — add V2 constructor

This is the most macro-heavy task. The goal: when `#[odra::module(keys = "v2")]` is used, the macro-generated WASM entry points create `ContractEnv` with `KeyEncoding::V2` instead of `KeyEncoding::Legacy`.

**Important:** `WasmContractEnv::new_env()` appears in ~20+ locations across the macro crate. Some use the `utils::expr::new_wasm_contract_env()` helper, but most are hard-coded inside `quote!` and `parse_quote!` blocks in `wasm_parts.rs` (lines 131, 357, 398, 403, 408, etc.) and `factory/parts/wasm_parts.rs` (lines 307, 374, 455, 495, 623, 642, 679, 719, 750, 755, 765, 770). All sites must be updated.

- [ ] **Step 1: Add V2 constructor to `WasmContractEnv`**

In `odra-casper/wasm-env/src/wasm_contract_env.rs`:

```rust
impl WasmContractEnv {
    pub fn new_env() -> ContractEnv {
        ContractEnv::new(KeyEncoding::Legacy, Rc::new(RefCell::new(WasmContractEnv)))
    }

    pub fn new_env_v2() -> ContractEnv {
        ContractEnv::new(KeyEncoding::V2, Rc::new(RefCell::new(WasmContractEnv)))
    }
}
```

- [ ] **Step 2: Expose `is_v2_keys()` on the IR**

In `odra-macros/src/ir/mod.rs`, add a method on `ModuleStructIR` (or the relevant IR type):

```rust
pub fn is_v2_keys(&self) -> bool {
    if let ConfigItem::Module(cfg) = &self.config {
        *cfg.keys
    } else {
        false
    }
}
```

- [ ] **Step 3: Update `new_wasm_contract_env` utility to accept a flag**

In `odra-macros/src/utils/expr.rs`, replace the current function:

```rust
pub fn new_wasm_contract_env(v2: bool) -> syn::Expr {
    if v2 {
        parse_quote!(odra::odra_casper_wasm_env::WasmContractEnv::new_env_v2())
    } else {
        parse_quote!(odra::odra_casper_wasm_env::WasmContractEnv::new_env())
    }
}
```

- [ ] **Step 4: Update ALL `WasmContractEnv::new_env()` call sites in wasm_parts**

Grep for all occurrences of `WasmContractEnv::new_env()` in the macro crate:

```bash
grep -rn "WasmContractEnv::new_env()" odra-macros/src/
```

For each site:
1. If it uses `utils::expr::new_wasm_contract_env()` — update to pass `ir.is_v2_keys()`.
2. If it's hard-coded in a `quote!`/`parse_quote!` block — refactor to use the utility function, or extract the env expression into a variable before the `quote!` block and interpolate it.

The pattern for hard-coded sites:

```rust
// Before (hard-coded in quote!):
let env = odra::odra_casper_wasm_env::WasmContractEnv::new_env();

// After (extract and interpolate):
let env_expr = utils::expr::new_wasm_contract_env(ir.is_v2_keys());
// then in quote!:
let env = #env_expr;
```

Thread the IR (or just the `v2: bool` flag) to each struct that generates WASM code. This may require adding a field to `CallFnItem`, `NoMangleFnItem`, and their factory counterparts.

- [ ] **Step 5: Update snapshot tests in wasm_parts**

The snapshot tests in `odra-macros/src/ast/wasm_parts.rs` (from line 286) and `odra-macros/src/ast/factory/parts/wasm_parts.rs` (from line 524) compare generated code. Since the default is Legacy (`new_env()`), existing snapshot expectations should remain unchanged. Verify by running tests.

- [ ] **Step 6: Verify macro tests pass**

Run: `cargo test -p odra-macros 2>&1 | tail -20`
Expected: All tests pass.

- [ ] **Step 7: Commit**

```bash
git add odra-macros/ odra-casper/wasm-env/src/wasm_contract_env.rs
git commit -m "feat: propagate KeyEncoding from module attribute to ContractEnv"
```

---

### Task 6: Run the full test suite

**Files:** None (verification only)

- [ ] **Step 1: Run core tests**

Run: `cargo test -p odra-core 2>&1 | tail -20`
Expected: All tests pass.

- [ ] **Step 2: Run macro tests**

Run: `cargo test -p odra-macros 2>&1 | tail -20`
Expected: All tests pass.

- [ ] **Step 3: Run VM tests**

Run: `cargo test -p odra-vm 2>&1 | tail -20`
Expected: All tests pass.

- [ ] **Step 4: Run full workspace test**

Run: `cargo test 2>&1 | tail -30`
Expected: All tests pass.

- [ ] **Step 5: Run clippy**

Run: `just clippy 2>&1 | tail -20`
Expected: No warnings or errors.

---

### Task 7: Add integration test with >15 fields

**Files:**
- Create or modify: an example contract in `examples/src/features/` with 20+ fields to demonstrate the new capability

- [ ] **Step 1: Create a module with 20 fields**

Add a new file `examples/src/features/many_fields.rs` (or extend an existing test file):

```rust
use odra::prelude::*;
use odra::Var;

#[odra::module]
pub struct ManyFieldsContract {
    f1: Var<u32>,
    f2: Var<u32>,
    f3: Var<u32>,
    f4: Var<u32>,
    f5: Var<u32>,
    f6: Var<u32>,
    f7: Var<u32>,
    f8: Var<u32>,
    f9: Var<u32>,
    f10: Var<u32>,
    f11: Var<u32>,
    f12: Var<u32>,
    f13: Var<u32>,
    f14: Var<u32>,
    f15: Var<u32>,
    f16: Var<u32>,
    f17: Var<u32>,
    f18: Var<u32>,
    f19: Var<u32>,
    f20: Var<u32>,
}

#[odra::module]
impl ManyFieldsContract {
    pub fn set_values(&mut self) {
        self.f1.set(1);
        self.f15.set(15);
        self.f16.set(16);
        self.f20.set(20);
    }

    pub fn get_f1(&self) -> u32 {
        self.f1.get_or_default()
    }

    pub fn get_f15(&self) -> u32 {
        self.f15.get_or_default()
    }

    pub fn get_f16(&self) -> u32 {
        self.f16.get_or_default()
    }

    pub fn get_f20(&self) -> u32 {
        self.f20.get_or_default()
    }
}
```

- [ ] **Step 2: Add a test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use odra::host::Deployer;

    #[test]
    fn many_fields_storage_works() {
        let env = odra_test::env();
        let mut contract = ManyFieldsContractHostRef::deploy(&env, NoArgs);
        contract.set_values();
        assert_eq!(contract.get_f1(), 1);
        assert_eq!(contract.get_f15(), 15);
        assert_eq!(contract.get_f16(), 16);
        assert_eq!(contract.get_f20(), 20);
    }
}
```

- [ ] **Step 3: Register the module**

Add `pub mod many_fields;` to `examples/src/features/mod.rs`.

- [ ] **Step 4: Run the test**

Run: `cd examples && cargo odra test -t many_fields 2>&1 | tail -20`
Expected: Test passes — all 20 fields independently store and retrieve values.

- [ ] **Step 5: Commit**

```bash
git add examples/src/features/many_fields.rs examples/src/features/mod.rs
git commit -m "test: add integration test with 20-field contract"
```

---

### Task 8: Add upgrade scenario test

**Files:**
- Modify: `examples/src/features/many_fields.rs` — add upgrade simulation test

This test verifies spec test #9: deploy with 15 fields, write data, upgrade to 16+, verify original fields are still readable.

- [ ] **Step 1: Add upgrade scenario test**

In `examples/src/features/many_fields.rs`, add a second contract that simulates an "upgrade" by having the first 15 fields in the same order plus additional ones:

```rust
/// Simulates the original contract before upgrade (15 fields).
#[odra::module]
pub struct OriginalContract {
    f1: Var<u32>,
    f2: Var<u32>,
    f3: Var<u32>,
    f4: Var<u32>,
    f5: Var<u32>,
    f6: Var<u32>,
    f7: Var<u32>,
    f8: Var<u32>,
    f9: Var<u32>,
    f10: Var<u32>,
    f11: Var<u32>,
    f12: Var<u32>,
    f13: Var<u32>,
    f14: Var<u32>,
    f15: Var<u32>,
}

#[odra::module]
impl OriginalContract {
    pub fn set_all(&mut self) {
        self.f1.set(100);
        self.f10.set(1000);
        self.f15.set(1500);
    }

    pub fn get_f1(&self) -> u32 { self.f1.get_or_default() }
    pub fn get_f10(&self) -> u32 { self.f10.get_or_default() }
    pub fn get_f15(&self) -> u32 { self.f15.get_or_default() }
}

/// Simulates the upgraded contract (same first 15 fields + 5 new ones).
#[odra::module]
pub struct UpgradedContract {
    f1: Var<u32>,
    f2: Var<u32>,
    f3: Var<u32>,
    f4: Var<u32>,
    f5: Var<u32>,
    f6: Var<u32>,
    f7: Var<u32>,
    f8: Var<u32>,
    f9: Var<u32>,
    f10: Var<u32>,
    f11: Var<u32>,
    f12: Var<u32>,
    f13: Var<u32>,
    f14: Var<u32>,
    f15: Var<u32>,
    f16: Var<u32>,
    f17: Var<u32>,
    f18: Var<u32>,
    f19: Var<u32>,
    f20: Var<u32>,
}

#[odra::module]
impl UpgradedContract {
    pub fn get_f1(&self) -> u32 { self.f1.get_or_default() }
    pub fn get_f10(&self) -> u32 { self.f10.get_or_default() }
    pub fn get_f15(&self) -> u32 { self.f15.get_or_default() }
    pub fn get_f16(&self) -> u32 { self.f16.get_or_default() }
    pub fn set_f16(&mut self, v: u32) { self.f16.set(v); }
}
```

- [ ] **Step 2: Write the upgrade test**

```rust
#[test]
fn upgrade_preserves_existing_fields() {
    let env = odra_test::env();

    // Deploy "original" contract and write data
    let mut original = OriginalContractHostRef::deploy(&env, NoArgs);
    original.set_all();
    assert_eq!(original.get_f1(), 100);
    assert_eq!(original.get_f10(), 1000);
    assert_eq!(original.get_f15(), 1500);

    // "Upgrade": load the same address as the upgraded contract
    let addr = *original.address();
    let mut upgraded = UpgradedContractHostRef::new(env.clone(), addr);

    // Original fields must still be readable with same values
    assert_eq!(upgraded.get_f1(), 100);
    assert_eq!(upgraded.get_f10(), 1000);
    assert_eq!(upgraded.get_f15(), 1500);

    // New fields start empty
    assert_eq!(upgraded.get_f16(), 0);

    // Can write to new fields without affecting old ones
    upgraded.set_f16(1600);
    assert_eq!(upgraded.get_f16(), 1600);
    assert_eq!(upgraded.get_f1(), 100);
}
```

Note: The exact mechanism to load an existing contract address as a different type depends on the test framework's API. The implementer should check how `HostRef::new` or contract loading works in the test environment and adapt accordingly.

- [ ] **Step 3: Run the test**

Run: `cd examples && cargo odra test -t upgrade_preserves 2>&1 | tail -20`
Expected: Test passes.

- [ ] **Step 4: Commit**

```bash
git add examples/src/features/many_fields.rs
git commit -m "test: add upgrade scenario test for storage key compatibility"
```

---

### Task 9: Final verification and cleanup

**Files:** None new

- [ ] **Step 1: Run `just check-lint`**

Run: `just check-lint 2>&1 | tail -20`
Expected: Clean.

- [ ] **Step 2: Run examples test suite**

Run: `cd examples && cargo odra test 2>&1 | tail -30`
Expected: All existing tests pass alongside the new one.

- [ ] **Step 3: Run modules test suite**

Run: `cd modules && cargo odra test 2>&1 | tail -30`
Expected: All tests pass.

- [ ] **Step 4: Verify no leftover `INDEX_SIZE` references**

Run: `grep -r "INDEX_SIZE" core/src/`
Expected: No results (constant was removed in Task 2).

- [ ] **Step 5: Create final commit if any cleanup was needed**

```bash
git add -A
git commit -m "chore: final cleanup for storage key redesign"
```
