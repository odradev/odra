# Odra Core

This is the crate README.
To see framework documentation goto [Github](https://github.com/odradev/odra).

Home of the `odra` crate.
It glues together all other Odra crates and provides easy to use interface.

It has 2 modes controlled using features. Only one feature at the time should be used:
- `mock-vm`, enabled by default, used to run the code against `MockVM`.
- `casper`, should be used when compiling code to Casper WebAssembly and to
run the code against a provided Casper virtual machine. When using `casper`,
default features needs to be turned off.
