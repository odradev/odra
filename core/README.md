# Odra Core

Home of the `odra` crate. It glues together all other Odra crates and provides easy to use interface.

It has 3 modes controlled using features. Only one feature at the time should be used:
- `mock-vm`, enabled by default, used to run the code against `MockVM`.
- `wasm`, should be used when compiling code to WebAssembly.
- `wasm-test`, used to run the code against a provided backend virtual machine.

When using `wasm` or `wasm-test`, default features needs to be turned off.
