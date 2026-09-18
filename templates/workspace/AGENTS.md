# Instructions for AI agents

This is an [Odra](https://odra.dev) smart contract project for the Casper Network. Odra has its
own module system, storage types, testing backends and CLI (`cargo odra`) that differ from
plain Rust and from other blockchain frameworks - do not guess them.

## Claude Code: install the Odra plugin

The plugin carries the skills for writing, testing and deploying Odra contracts. Ask the user
to run these commands once, then use the skills:

```
/plugin marketplace add odradev/odradev-plugins
/plugin install odra-plugin@odradev-plugins
```

## Other agents

Read [https://odra.dev/llms.txt](https://odra.dev/llms.txt) first - it indexes the whole
documentation set. Then follow the docs at [https://odra.dev/docs](https://odra.dev/docs).

## Commands

- `cargo odra build` - compile the contracts listed in `Odra.toml` to wasm
- `cargo odra test` - run tests on the in-memory OdraVM (fast, no wasm needed)
- `cargo odra test -b casper` - build wasm and run the same tests on the Casper execution engine
