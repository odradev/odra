---
name: start-nctl
description: >
  Start a local Casper NCTL node using Docker, wait for readiness, and extract keys.
  Use when the user says "start nctl", "start node", "start local node",
  "run nctl", or "start-nctl".
allowed-tools: Bash(docker *),Bash(chmod *),Bash(curl *),Bash(wc *),Bash(.claude/skills/start-nctl/scripts/*)
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
chmod +x .claude/skills/start-nctl/scripts/wait-for-nctl.sh
.claude/skills/start-nctl/scripts/wait-for-nctl.sh 120
```

If the wait script exits non-zero, report failure and stop.

---

## Step 4 — Extract Keys

```bash
chmod +x .claude/skills/start-nctl/scripts/extract-keys.sh
.claude/skills/start-nctl/scripts/extract-keys.sh
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
