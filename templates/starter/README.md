# {{project-name}}

An Odra smart contract project.

## Prerequisites

Install [cargo-odra](https://github.com/odradev/cargo-odra) and its dependencies.

If you have Claude Code, run `/check-env` to verify your setup.

## Getting Started

### With Claude Code (recommended)

- **New to Odra?** Run `/onboard` for a guided walkthrough.
- **Know Odra?** Use skills directly: `/new-contract`, `/new-entrypoint`, `/new-version`, etc.

### Manual

#### Build

```
cargo odra build -b casper
```

#### Test

```
cargo odra test
```

#### Test against Casper VM

```
cargo odra test -b casper
```
