# odra-casper-account-manager

Changes a Casper account's multisig configuration: its associated keys, their weights and the
`deployment` and `key_management` action thresholds.

- `ACCOUNT_MANAGER_WASM`: session code that applies an `AccountChange` in one transaction. It
  installs nothing. Built from `odra-casper/account-manager-wasm` with `just build-proxy-callers`.
- `AccountChange::to_runtime_args`: the arguments for that wasm.
- `plan`: checks a change against the current `AccountState` before anything is sent, and
  returns the state after the change plus warnings, or the error the node would reject it with.
