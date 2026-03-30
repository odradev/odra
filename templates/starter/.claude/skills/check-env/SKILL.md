---
name: check-env
description: >
  Validate the development environment for Odra smart contract development.
  Use when the user says "check env", "check setup", "check prerequisites",
  "verify environment", or "check-env".
allowed-tools: Bash(rustc *),Bash(rustup *),Bash(cargo odra *),Bash(wasm-opt *),Bash(wasm-strip *),Bash(docker *)
---

# Check Development Environment

Validates that all prerequisites for Odra development are installed.

---

## Step 1 — Check Each Prerequisite

Run these checks and collect results:

### Rust nightly toolchain

```bash
rustc --version
```

Compare against the version in `rust-toolchain` file. If the required nightly is not installed:
- Install: `rustup toolchain install <version>`

### wasm32-unknown-unknown target

```bash
rustup target list --installed | grep wasm32-unknown-unknown
```

If missing:
- Install: `rustup target add wasm32-unknown-unknown`

### cargo-odra

```bash
cargo odra --version
```

If missing:
- Install: `cargo install cargo-odra --git https://github.com/odradev/cargo-odra --locked`

### wasm-opt (binaryen)

```bash
wasm-opt --version
```

If missing:
- macOS: `brew install binaryen`
- Linux: download from https://github.com/WebAssembly/binaryen/releases

### wasm-strip (wabt)

```bash
wasm-strip --version
```

If missing:
- macOS: `brew install wabt`
- Linux: `sudo apt install wabt`

### Docker (optional)

```bash
docker --version
```

If missing, note it is optional — only needed for local NCTL node testing.
- Install from https://docs.docker.com/get-docker/

---

## Step 2 — Report Results

Present a summary table:

```
| Prerequisite             | Status | Action needed         |
|--------------------------|--------|-----------------------|
| Rust nightly (YYYY-MM-DD)| OK/MISSING | install command    |
| wasm32-unknown-unknown   | OK/MISSING | install command    |
| cargo-odra               | OK/MISSING | install command    |
| wasm-opt (binaryen)      | OK/MISSING | install command    |
| wasm-strip (wabt)        | OK/MISSING | install command    |
| Docker (optional)        | OK/MISSING | install link       |
```

If everything is OK, report: "Environment is ready for Odra development."

If items are missing, list the install commands and explain what each tool is for:
- **Rust nightly**: required compiler for Odra contracts
- **wasm32-unknown-unknown**: WebAssembly compilation target for smart contracts
- **cargo-odra**: Odra's build/test tool — wraps cargo with WASM compilation steps
- **wasm-opt**: optimizes WASM binaries for smaller contract size
- **wasm-strip**: strips debug info from WASM binaries
- **Docker**: runs a local Casper blockchain node for testing deployments
