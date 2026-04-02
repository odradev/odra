# Odra Claude Code Plugin Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a standalone Claude Code plugin (`odra`) from the starter template's `.claude/skills/`, with a coding agent for implementation tasks.

**Architecture:** New repo at `/Users/kpob/workspace/odra-plugin/`. Skills are copied from `templates/starter/.claude/skills/` with cross-references namespaced to `/odra:*`. The `start-nctl` scripts are inlined since plugin paths differ from project paths. A new `odra-coder` agent handles code-writing tasks with tiered context loading.

**Tech Stack:** Claude Code plugin system (`.claude-plugin/plugin.json`), Markdown skills, Markdown agent definitions

---

## File Structure

```
/Users/kpob/workspace/odra-plugin/
  .claude-plugin/
    plugin.json
  skills/
    onboard/SKILL.md
    check-env/SKILL.md
    new-contract/SKILL.md
    new-factory-contract/SKILL.md
    new-entrypoint/SKILL.md
    new-version/SKILL.md
    new-scenario/SKILL.md
    start-nctl/SKILL.md
    deploy-to-livenet/SKILL.md
  agents/
    odra-coder.md
  README.md
```

Changes to existing repo:
- Remove: `templates/starter/.claude/skills/` (entire directory)
- Modify: `templates/starter/.claude/CLAUDE.md` (update skill references to `/odra:*`)

---

## Task 1: Initialize plugin repo and create plugin.json

**Files:**
- Create: `/Users/kpob/workspace/odra-plugin/.claude-plugin/plugin.json`

- [ ] **Step 1: Create the plugin directory and manifest**

```bash
mkdir -p /Users/kpob/workspace/odra-plugin/.claude-plugin
```

Write `/Users/kpob/workspace/odra-plugin/.claude-plugin/plugin.json`:

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

- [ ] **Step 2: Initialize git**

```bash
cd /Users/kpob/workspace/odra-plugin && git init && git add .claude-plugin/plugin.json && git commit -m "feat: initialize odra plugin with manifest"
```

---

## Task 2: Copy simple skills (no cross-references)

These skills have no references to other skills and need no modification beyond the file copy.

**Files:**
- Create: `/Users/kpob/workspace/odra-plugin/skills/check-env/SKILL.md`
- Create: `/Users/kpob/workspace/odra-plugin/skills/new-contract/SKILL.md`
- Create: `/Users/kpob/workspace/odra-plugin/skills/new-factory-contract/SKILL.md`
- Create: `/Users/kpob/workspace/odra-plugin/skills/new-entrypoint/SKILL.md`
- Create: `/Users/kpob/workspace/odra-plugin/skills/new-version/SKILL.md`
- Create: `/Users/kpob/workspace/odra-plugin/skills/new-scenario/SKILL.md`

- [ ] **Step 1: Create skills directory and copy files**

```bash
mkdir -p /Users/kpob/workspace/odra-plugin/skills/{check-env,new-contract,new-factory-contract,new-entrypoint,new-version,new-scenario}
```

Copy each file from `/Users/kpob/workspace/odra/templates/starter/.claude/skills/` to `/Users/kpob/workspace/odra-plugin/skills/`:

```bash
cp /Users/kpob/workspace/odra/templates/starter/.claude/skills/check-env/SKILL.md /Users/kpob/workspace/odra-plugin/skills/check-env/SKILL.md
cp /Users/kpob/workspace/odra/templates/starter/.claude/skills/new-contract/SKILL.md /Users/kpob/workspace/odra-plugin/skills/new-contract/SKILL.md
cp /Users/kpob/workspace/odra/templates/starter/.claude/skills/new-factory-contract/SKILL.md /Users/kpob/workspace/odra-plugin/skills/new-factory-contract/SKILL.md
cp /Users/kpob/workspace/odra/templates/starter/.claude/skills/new-entrypoint/SKILL.md /Users/kpob/workspace/odra-plugin/skills/new-entrypoint/SKILL.md
cp /Users/kpob/workspace/odra/templates/starter/.claude/skills/new-version/SKILL.md /Users/kpob/workspace/odra-plugin/skills/new-version/SKILL.md
cp /Users/kpob/workspace/odra/templates/starter/.claude/skills/new-scenario/SKILL.md /Users/kpob/workspace/odra-plugin/skills/new-scenario/SKILL.md
```

- [ ] **Step 2: Verify all 6 files exist**

```bash
ls /Users/kpob/workspace/odra-plugin/skills/*/SKILL.md
```

Expected: 6 files listed.

- [ ] **Step 3: Commit**

```bash
cd /Users/kpob/workspace/odra-plugin && git add skills/ && git commit -m "feat: add check-env, new-contract, new-factory-contract, new-entrypoint, new-version, new-scenario skills"
```

---

## Task 3: Copy and update deploy-to-livenet skill

This skill references `/start-nctl` which needs to become `/odra:start-nctl`.

**Files:**
- Create: `/Users/kpob/workspace/odra-plugin/skills/deploy-to-livenet/SKILL.md`

- [ ] **Step 1: Copy the file**

```bash
mkdir -p /Users/kpob/workspace/odra-plugin/skills/deploy-to-livenet
cp /Users/kpob/workspace/odra/templates/starter/.claude/skills/deploy-to-livenet/SKILL.md /Users/kpob/workspace/odra-plugin/skills/deploy-to-livenet/SKILL.md
```

- [ ] **Step 2: Update cross-references**

In `/Users/kpob/workspace/odra-plugin/skills/deploy-to-livenet/SKILL.md`, replace all occurrences of `/start-nctl` with `/odra:start-nctl`.

There are two occurrences (lines 41 and 43 in the original):
- `tell the user to run /start-nctl first` → `tell the user to run /odra:start-nctl first`
- `suggest running /start-nctl which creates it` → `suggest running /odra:start-nctl which creates it`

- [ ] **Step 3: Commit**

```bash
cd /Users/kpob/workspace/odra-plugin && git add skills/deploy-to-livenet/ && git commit -m "feat: add deploy-to-livenet skill with namespaced references"
```

---

## Task 4: Create start-nctl skill with inlined scripts

The original `start-nctl` has separate shell scripts referenced via project-relative paths (`.claude/skills/start-nctl/scripts/*`). In a plugin, these paths won't resolve. Inline the script content directly into the SKILL.md as heredoc bash commands.

**Files:**
- Create: `/Users/kpob/workspace/odra-plugin/skills/start-nctl/SKILL.md`

- [ ] **Step 1: Create the skill directory**

```bash
mkdir -p /Users/kpob/workspace/odra-plugin/skills/start-nctl
```

- [ ] **Step 2: Write the updated SKILL.md**

Write `/Users/kpob/workspace/odra-plugin/skills/start-nctl/SKILL.md` with this content:

````markdown
---
name: start-nctl
description: >
  Start a local Casper NCTL node using Docker, wait for readiness, and extract keys.
  Use when the user says "start nctl", "start node", "start local node",
  "run nctl", or "start-nctl".
allowed-tools: Bash(docker *),Bash(chmod *),Bash(curl *),Bash(wc *),Bash(bash -c *)
---

# Start Local Casper Node (NCTL)

## Context

Read these before proceeding:

- `.claude/context/reference/deployment.md`

Starts a local Casper blockchain node via Docker for testing contract deployments.

---

## Step 1 — Check Docker

```bash
docker --version
```

If Docker is not installed or not running, tell the user and stop.

---

## Step 2 — Check if NCTL is Already Running

```bash
docker ps --filter name=mynctl --format '{{.Names}}'
```

If `mynctl` is listed, skip to Step 4.

---

## Step 3 — Start NCTL

```bash
docker run --rm -it --cpus=1 --name mynctl -d -p 11101:11101 -p 14101:14101 -p 18101:18101 -p 25101:25101 makesoftware/casper-nctl:v203
```

Wait for readiness:

```bash
bash -c '
TIMEOUT=120
ELAPSED=0
INTERVAL=3
RPC_URL="http://localhost:11101/rpc"

echo "Waiting for NCTL node at $RPC_URL (timeout: ${TIMEOUT}s)..."

while [ $ELAPSED -lt $TIMEOUT ]; do
  if curl -s -m 2 "$RPC_URL" \
    -H "content-type: application/json" \
    --data-raw "{\"jsonrpc\":\"2.0\",\"id\":\"1\",\"method\":\"info_get_status\"}" \
    2>/dev/null | grep -q "jsonrpc"; then
    echo "NCTL node is ready (after ${ELAPSED}s)."
    exit 0
  fi
  sleep $INTERVAL
  ELAPSED=$((ELAPSED + INTERVAL))
done

echo "ERROR: NCTL node did not become ready within ${TIMEOUT}s."
exit 1
'
```

If the wait command exits non-zero, report failure and stop.

---

## Step 4 — Extract Keys

```bash
bash -c '
project_root=$(git rev-parse --show-toplevel)
cd "$project_root"

mkdir -p .node-keys

docker exec mynctl /bin/bash -c \
  "cat /home/casper/casper-nctl/assets/net-1/users/user-1/secret_key.pem" \
  > .node-keys/secret_key.pem

docker exec mynctl /bin/bash -c \
  "cat /home/casper/casper-nctl/assets/net-1/users/user-2/secret_key.pem" \
  > .node-keys/secret_key_1.pem
'
```

Verify keys are non-empty:

```bash
wc -c .node-keys/secret_key.pem .node-keys/secret_key_1.pem
```

If either file is 0 bytes, report error and stop.

---

## Step 5 — Create .env

Create `.env` from `.env.sample` with nctl defaults:

```env
ODRA_CASPER_LIVENET_SECRET_KEY_PATH=.node-keys/secret_key.pem
ODRA_CASPER_LIVENET_NODE_ADDRESS=http://localhost:11101
ODRA_CASPER_LIVENET_EVENTS_URL=http://localhost:18101/events
ODRA_CASPER_LIVENET_CHAIN_NAME=casper-net-1
```

---

## Step 6 — Report

```
NCTL node is running.
- RPC: http://localhost:11101
- Events: http://localhost:18101/events
- Chain: casper-net-1
- Keys: .node-keys/secret_key.pem, .node-keys/secret_key_1.pem
- .env: configured for nctl

To stop: docker stop mynctl
```
````

- [ ] **Step 3: Commit**

```bash
cd /Users/kpob/workspace/odra-plugin && git add skills/start-nctl/ && git commit -m "feat: add start-nctl skill with inlined scripts"
```

---

## Task 5: Copy and update onboard skill

The onboard skill has the most cross-references — it invokes other skills and lists them all in the "What's Next" section. All `/skill-name` references need to become `/odra:skill-name`.

**Files:**
- Create: `/Users/kpob/workspace/odra-plugin/skills/onboard/SKILL.md`

- [ ] **Step 1: Copy the file**

```bash
mkdir -p /Users/kpob/workspace/odra-plugin/skills/onboard
cp /Users/kpob/workspace/odra/templates/starter/.claude/skills/onboard/SKILL.md /Users/kpob/workspace/odra-plugin/skills/onboard/SKILL.md
```

- [ ] **Step 2: Update all cross-references**

In `/Users/kpob/workspace/odra-plugin/skills/onboard/SKILL.md`, make these replacements:

| Original | Replacement |
|---|---|
| `Invoke \`/check-env\`` | `Invoke \`/odra:check-env\`` |
| `Invoke \`/new-contract\`` | `Invoke \`/odra:new-contract\`` |
| `Invoke \`/start-nctl\`` | `Invoke \`/odra:start-nctl\`` |
| `Invoke \`/deploy-to-livenet\`` | `Invoke \`/odra:deploy-to-livenet\`` |
| `` `/new-contract` — add more contracts`` | `` `/odra:new-contract` — add more contracts`` |
| `` `/new-entrypoint` — add methods`` | `` `/odra:new-entrypoint` — add methods`` |
| `` `/new-version` — create an upgraded`` | `` `/odra:new-version` — create an upgraded`` |
| `` `/new-factory-contract` — create a factory`` | `` `/odra:new-factory-contract` — create a factory`` |
| `` `/new-scenario` — add CLI scenarios`` | `` `/odra:new-scenario` — add CLI scenarios`` |
| `` `/deploy-to-livenet` — deploy to testnet`` | `` `/odra:deploy-to-livenet` — deploy to testnet`` |

There are 10 replacements total (4 invocations + 6 in the What's Next list).

- [ ] **Step 3: Verify no un-namespaced references remain**

```bash
grep -n "Invoke \`/" /Users/kpob/workspace/odra-plugin/skills/onboard/SKILL.md
grep -n "> - \`/" /Users/kpob/workspace/odra-plugin/skills/onboard/SKILL.md
```

Every match should contain `/odra:`.

- [ ] **Step 4: Commit**

```bash
cd /Users/kpob/workspace/odra-plugin && git add skills/onboard/ && git commit -m "feat: add onboard skill with namespaced cross-references"
```

---

## Task 6: Create the odra-coder agent

**Files:**
- Create: `/Users/kpob/workspace/odra-plugin/agents/odra-coder.md`

- [ ] **Step 1: Create the agents directory**

```bash
mkdir -p /Users/kpob/workspace/odra-plugin/agents
```

- [ ] **Step 2: Write the agent definition**

Write `/Users/kpob/workspace/odra-plugin/agents/odra-coder.md`:

````markdown
---
name: odra-coder
description: |
  Use this agent for Odra smart contract implementation tasks: writing contracts,
  adding entry points, fixing bugs, writing tests, wiring up CLI scenarios.
  Use when the user asks to "implement", "write", "build", "fix", or "add"
  contract code.
model: inherit
---

You are an Odra smart contract developer. You implement contracts, write tests, and wire up CLI tooling for the Odra framework on the Casper Network.

## On Start — Load Overview Context

Before doing any work, read these files from the project to understand the framework:

1. `.claude/context/overview/architecture.md` — crate map, execution flow, context layers
2. `.claude/context/overview/contract-model.md` — `#[odra::module]`, HostRef, Deployer, init convention
3. `.claude/context/overview/testing-model.md` — OdraVM vs CasperVM vs livenet

If any of these files don't exist, inform the caller that this project needs Odra context docs (installed via the starter template).

## On-Demand — Load Reference Context

Based on the task, read the relevant reference files before writing code:

| Task involves... | Read this file |
|---|---|
| Storage fields (`Var`, `Mapping`, `List`, `Sequence`, `External`) | `.claude/context/reference/storage.md` |
| Entry points, constructors, payable, `self.env()` | `.claude/context/reference/entry-points.md` |
| Defining or emitting events | `.claude/context/reference/events.md` |
| Error enums, reverting, `unwrap_or_revert` | `.claude/context/reference/errors.md` |
| Cross-contract calls, `#[odra::external_contract]` | `.claude/context/reference/cross-contract.md` |
| Writing or fixing tests | `.claude/context/reference/testing.md` |
| CLI deploy scripts, scenarios, `load_or_deploy` | `.claude/context/reference/deployment.md` |

Read only the files relevant to the current task. Do not load all reference files upfront.

## Workflow

Follow this process for every implementation task:

1. **Understand the codebase** — read existing contracts in `contracts/src/`, `Odra.toml`, and `cli/cli.rs` to understand what exists
2. **Write a failing test first** — in the contract's `#[cfg(test)] mod tests` block
3. **Run the test to confirm it fails** — `cargo odra test`
4. **Implement the minimal code to pass** — follow patterns from the context docs
5. **Run tests to confirm they pass** — `cargo odra test`
6. **Wire up registrations if needed:**
   - New contract → add `[[contracts]]` entry in `Odra.toml`, add `pub mod` in `contracts/src/lib.rs`
   - New CLI contract → add `.contract::<MyContract>()` in `cli/cli.rs`
   - New scenario → add `.scenario::<MyScenario>(MyScenario)` in `cli/cli.rs`
7. **Commit** — descriptive commit message

## Code Quality Rules

- Follow existing patterns in the codebase
- Use `&self` for read-only entry points, `&mut self` for state-changing ones
- Register events in the module attribute: `#[odra::module(events = [...])]`
- Register errors in the module attribute: `#[odra::module(errors = Error)]`
- Error discriminants must be unique across the project
- Use `NoArgs` for contracts without a constructor
- Use `self.env()` for on-chain context, never global state

## What You Don't Do

- You do not invoke skills — you write code directly
- You do not modify `.claude/context/` files — those are reference docs
- You do not make architectural decisions — ask if unsure
- You do not deploy to livenet — that's what `/odra:deploy-to-livenet` is for

## Report Format

When done, report:
- **Status:** DONE | DONE_WITH_CONCERNS | BLOCKED | NEEDS_CONTEXT
- What you implemented
- Test results (`cargo odra test` output summary)
- Files changed
- Any concerns
````

- [ ] **Step 3: Commit**

```bash
cd /Users/kpob/workspace/odra-plugin && git add agents/ && git commit -m "feat: add odra-coder agent for implementation tasks"
```

---

## Task 7: Create README.md

**Files:**
- Create: `/Users/kpob/workspace/odra-plugin/README.md`

- [ ] **Step 1: Write the README**

Write `/Users/kpob/workspace/odra-plugin/README.md`:

````markdown
# Odra Plugin for Claude Code

A Claude Code plugin for developing smart contracts with the [Odra framework](https://github.com/odradev/odra) on the Casper Network.

## Installation

```bash
# In Claude Code
/install-plugin odradev/odra-plugin
```

Or during development:

```bash
claude --plugin-dir /path/to/odra-plugin
```

## Skills

| Skill | Description |
|---|---|
| `/odra:onboard` | Guided walkthrough — write, test, and deploy your first contract |
| `/odra:check-env` | Verify your development environment |
| `/odra:new-contract` | Scaffold a new contract module |
| `/odra:new-factory-contract` | Scaffold a factory contract |
| `/odra:new-entrypoint` | Add an entry point to an existing contract |
| `/odra:new-version` | Create an upgraded contract version |
| `/odra:new-scenario` | Add a CLI deployment/interaction scenario |
| `/odra:start-nctl` | Start a local Casper node via Docker |
| `/odra:deploy-to-livenet` | Deploy contracts to nctl, testnet, or mainnet |

## Agents

| Agent | Description |
|---|---|
| `odra:odra-coder` | Odra-aware coding agent for writing contracts, tests, and CLI wiring |

## Requirements

This plugin expects the project to have Odra context docs in `.claude/context/`. Projects scaffolded with `cargo odra new --template starter` include these automatically.

## License

MIT
````

- [ ] **Step 2: Commit**

```bash
cd /Users/kpob/workspace/odra-plugin && git add README.md && git commit -m "docs: add README"
```

---

## Task 8: Remove skills from starter template

Now that skills live in the plugin, remove the `skills/` directory from the starter template.

**Files:**
- Remove: `templates/starter/.claude/skills/` (entire directory)

- [ ] **Step 1: Remove the skills directory**

```bash
cd /Users/kpob/workspace/odra && git rm -rf templates/starter/.claude/skills/
```

- [ ] **Step 2: Verify only context/ and CLAUDE.md remain**

```bash
find templates/starter/.claude/ -type f | sort
```

Expected: `CLAUDE.md` and `context/overview/*.md` and `context/reference/*.md` only.

- [ ] **Step 3: Commit**

```bash
cd /Users/kpob/workspace/odra && git add -A templates/starter/.claude/skills/ && git commit -m "feat(starter): remove skills — now in odra plugin"
```

---

## Task 9: Update starter template CLAUDE.md

Update the skill references in `templates/starter/.claude/CLAUDE.md` to use the namespaced `/odra:*` commands.

**Files:**
- Modify: `/Users/kpob/workspace/odra/templates/starter/.claude/CLAUDE.md`

- [ ] **Step 1: Read the current CLAUDE.md**

Read `templates/starter/.claude/CLAUDE.md`.

- [ ] **Step 2: Update skill references**

Replace the "Getting Started" section. Change:

```markdown
**New to Odra?** Run `/onboard` for a guided walkthrough — from writing your first contract to deploying it.
```

To:

```markdown
**New to Odra?** Run `/odra:onboard` for a guided walkthrough — from writing your first contract to deploying it.
```

Replace the skills table. Change every `/skill-name` to `/odra:skill-name`:

```markdown
| Skill | What it does |
|---|---|
| `/odra:check-env` | Verify your development environment |
| `/odra:new-contract` | Scaffold a new contract module |
| `/odra:new-factory-contract` | Scaffold a factory contract |
| `/odra:new-entrypoint` | Add an entry point to an existing contract |
| `/odra:new-version` | Create an upgraded contract version |
| `/odra:new-scenario` | Add a CLI deployment/interaction scenario |
| `/odra:start-nctl` | Start a local Casper node via Docker |
| `/odra:deploy-to-livenet` | Deploy contracts to nctl, testnet, or mainnet |
```

- [ ] **Step 3: Commit**

```bash
cd /Users/kpob/workspace/odra && git add templates/starter/.claude/CLAUDE.md && git commit -m "feat(starter): update skill references to odra plugin namespace"
```

---

## Self-Review Checklist

- [x] **Spec coverage:**
  - Plugin structure (`.claude-plugin/plugin.json`, `skills/`, `agents/`) → Tasks 1-6
  - Skill migration (all 9 skills) → Tasks 2-5
  - Namespace updates (`/skill` → `/odra:skill`) → Tasks 3, 5, 9
  - Coding agent (`odra-coder`) → Task 6
  - README → Task 7
  - Starter template cleanup → Tasks 8-9
- [x] **Placeholder scan:** No TBDs, TODOs, or incomplete sections. All file content is complete.
- [x] **Type consistency:** All skill names and paths are consistent across tasks. Namespace prefix is `odra:` everywhere.
- [x] **Script inlining:** `start-nctl` scripts inlined as `bash -c` heredocs to avoid plugin path resolution issues.
- [x] **Cross-reference audit:** All 12 cross-skill references (10 in onboard, 2 in deploy-to-livenet) updated to `/odra:*` namespace.
