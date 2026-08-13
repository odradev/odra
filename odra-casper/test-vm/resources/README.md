# Test resources

- `proxy_caller_with_return.wasm` - Odra's proxy caller used by `CasperVm` to
  capture call results.
- `cep18.wasm`, `cep18_test_contract.wasm` - the upstream CEP-18 reference
  implementation and its test helper contract, used by the addressable-entity
  migration tests. Built from https://github.com/casper-ecosystem/cep18
  at commit 7fa83c559d04b3516e06b4d7202aba912c14c235 (v2.0.0) with the repo's
  pinned toolchain (nightly-2025-02-04, `-C target-cpu=mvp`,
  `-Z build-std=std,panic_abort`).
