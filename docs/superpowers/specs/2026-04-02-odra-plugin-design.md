# Odra Claude Code Plugin Design

**Date**: 2026-04-02
**Status**: Approved

## Goal

Bundle the starter template's `.claude/skills/` into a standalone Claude Code plugin (`odra`) that can be installed independently. Add a coding agent for implementation tasks.

---

## Design Principles

- **Thin packaging** — existing skills move as-is with no content rewriting
- **Project-dependent context** — plugin reads context docs from the project's `.claude/context/`, not self-contained
- **Agent-augmented** — a coding agent handles implementation tasks with Odra awareness
- **Tiered context loading** — agent loads overviews upfront, references on-demand

---

## Plugin Structure

```
odra/                                     # separate repo (e.g. github.com/odradev/odra-plugin)
  .claude-plugin/
    plugin.json                           # name: "odra"
  skills/
    onboard/SKILL.md                      # primary entry point — guided walkthrough
    check-env/SKILL.md
    new-contract/SKILL.md
    new-factory-contract/SKILL.md
    new-entrypoint/SKILL.md
    new-version/SKILL.md
    new-scenario/SKILL.md
    start-nctl/
      SKILL.md
      scripts/extract-keys.sh
      scripts/wait-for-nctl.sh
    deploy-to-livenet/SKILL.md
  agents/
    odra-coder.md                         # coding agent for implementation tasks
  README.md
```

All skills invoked as `/odra:onboard`, `/odra:new-contract`, etc.
The coding agent appears as `odra:odra-coder` in the agent list.

---

## Plugin Manifest

```json
{
  "name": "odra",
  "description": "Odra smart contract development toolkit — onboarding, scaffolding, and coding agent for Casper Network contracts",
  "version": "0.1.0",
  "author": {
    "name": "Odra Dev"
  },
  "repository": "https://github.com/odradev/odra-plugin",
  "license": "MIT",
  "keywords": [
    "odra",
    "casper",
    "smart-contracts",
    "blockchain",
    "wasm"
  ]
}
```

---

## Skill Migration

Existing skills from `templates/starter/.claude/skills/` move into the plugin's `skills/` directory with no content changes. Context section paths (`.claude/context/overview/...`, `.claude/context/reference/...`) remain the same since they point to the project's `.claude/context/` directory.

Skills migrated as-is:
- `onboard/SKILL.md` — primary entry point, guided walkthrough
- `check-env/SKILL.md` — environment validation
- `new-contract/SKILL.md` — scaffold a contract module
- `new-factory-contract/SKILL.md` — scaffold a factory contract
- `new-entrypoint/SKILL.md` — add entry point to existing contract
- `new-version/SKILL.md` — create contract upgrade version
- `new-scenario/SKILL.md` — add CLI scenario
- `start-nctl/SKILL.md` + `scripts/` — start local Casper node
- `deploy-to-livenet/SKILL.md` — deploy to a live network

No plugin CLAUDE.md is needed — all knowledge lives in the project's context docs.

---

## Coding Agent: `odra-coder`

### Purpose

An Odra-aware implementation agent that writes contracts, tests, and CLI wiring. Used when the user asks to implement, write, build, fix, or add contract code.

### Frontmatter

```yaml
---
name: odra-coder
description: |
  Use this agent for Odra smart contract implementation tasks: writing contracts,
  adding entry points, fixing bugs, writing tests, wiring up CLI scenarios.
  Use when the user asks to "implement", "write", "build", "fix", or "add"
  contract code.
model: inherit
---
```

### Behavior

**Context loading — tiered:**

1. On start, read the project's overview context:
   - `.claude/context/overview/architecture.md`
   - `.claude/context/overview/contract-model.md`
   - `.claude/context/overview/testing-model.md`

2. On-demand, based on the task, read relevant reference files:
   - Writing storage fields → `.claude/context/reference/storage.md`
   - Writing entry points → `.claude/context/reference/entry-points.md`
   - Adding events → `.claude/context/reference/events.md`
   - Adding errors → `.claude/context/reference/errors.md`
   - Cross-contract calls → `.claude/context/reference/cross-contract.md`
   - Writing tests → `.claude/context/reference/testing.md`
   - Deployment/scenarios → `.claude/context/reference/deployment.md`

**Workflow — TDD-oriented:**

1. Read existing contract code to understand the codebase
2. Write a failing test first
3. Implement the minimal code to pass
4. Register in `Odra.toml` and `lib.rs` if new contract
5. Wire up CLI if needed
6. Run `cargo odra test` to verify
7. Commit when done

**The agent does not invoke skills.** It is the implementation worker — skills orchestrate higher-level workflows and may reference the agent, but the agent works directly with code.

---

## Context Doc Dependency

The plugin depends on the project having context docs in `.claude/context/`. These docs are part of the starter template (`templates/starter/.claude/context/`) and ship with scaffolded projects via `cargo odra new --template starter`.

The context docs will be moved from the starter template into the plugin at a later stage by the user.

---

## Starter Template Changes

After the plugin is created, the starter template's `.claude/` directory is simplified:

```
templates/starter/.claude/
  CLAUDE.md                              # project-specific context (kept)
  context/                               # reference docs (kept for now, moved later)
    overview/
      architecture.md
      contract-model.md
      testing-model.md
    reference/
      storage.md
      entry-points.md
      events.md
      errors.md
      cross-contract.md
      testing.md
      deployment.md
```

The `skills/` directory is removed from the template — skills now live in the plugin.
