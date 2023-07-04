# Odra v1.0.0

## Introduction
Odra is a operating system for the WASM environment.  

## Features
- Basic memory management.
- System of Modules.

## Host interaction.
The only way Odra can communicate and be useful is through the host.
Host is your environment.

Potential hosts are:
    - browser,
    - server,
    - mobile app,
    - blockchain,
    - testing environment,
    - off-chain clients,
    - build script (possible interaction with syn macros).
    - cargo odra

## Modules
Module is the main component of Odra.
Every `struct` marked as `#[odra::module]` is a module.
Most of the code that smart contract developers should build are modules.
The whole business logic should be implemented in modules.

Odra Modules provides a few modules:
    - variable
    - mapping
    - list
    - ownable

### Interacting with the host.

