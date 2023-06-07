# Odra Casper Proxy Caller

This crate provides:
- `proxy_caller_with_return.wasm` that can be used to call any Casper contract and save result into caller's named key. It bypasses Casper's limitation to return value from contract call.
- `proxy_caller.wasm` that is used to bypass Casper's limitation to call contract and pass purse with tokens to it. 
