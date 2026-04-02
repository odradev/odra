---
name: deploy-to-livenet
description: >
  Deploy contracts to a Casper network (nctl, testnet, or mainnet).
  Use when the user says "deploy", "deploy to livenet", "deploy to nctl",
  "deploy to testnet", "deploy to mainnet", "run on livenet", or "deploy-to-livenet".
allowed-tools: Bash(docker ps:*),Bash(curl *),Bash(jq *),Bash(casper-client *),Bash(set -a && source *),Bash(cargo run --bin * --features=livenet),Bash(wc *)
---

# Deploy Contracts to Livenet

## Context

Read these before proceeding:

- `.claude/context/reference/deployment.md`

Deploys contracts using the CLI binary against a Casper network.

---

## Step 1 — Identify Target Network

If not stated, ask the user:
- **nctl** — local Docker node
- **testnet** — Casper test network
- **mainnet** — Casper main network

---

## Step 2 — Ensure Environment is Ready

### For nctl

Check if NCTL is running:

```bash
docker ps --filter name=mynctl --format '{{.Names}}'
```

If not running, tell the user to run `/start-nctl` first.

Check `.env` exists and has nctl values. If not, suggest running `/start-nctl` which creates it.

### For testnet

Check if `.env.testnet` exists. If not, create it:

1. Discover a responsive node:
   ```bash
   curl -s 'https://node.testnet.cspr.cloud/rpc' \
     -H 'content-type: application/json' \
     --data-raw '{"jsonrpc":"2.0","id":"1","method":"info_get_peers"}' \
     | jq -r '.result.peers[:10][] | .address' | head -5
   ```

2. Test each node until one responds:
   ```bash
   casper-client list-rpcs -n http://<node_ip>:7777/rpc
   ```

3. Ask the user for their secret key path (must be a `.pem` file on disk). Verify it exists.

4. Write `.env.testnet`:
   ```env
   ODRA_CASPER_LIVENET_SECRET_KEY_PATH=<user-provided-path>
   ODRA_CASPER_LIVENET_NODE_ADDRESS=http://<responsive-node-ip>:7777
   ODRA_CASPER_LIVENET_EVENTS_URL=http://<responsive-node-ip>:9999/events
   ODRA_CASPER_LIVENET_CHAIN_NAME=casper-test
   ```

### For mainnet

Same flow as testnet but:
- Use `https://node.cspr.cloud/rpc` for peer discovery
- Chain name: `casper`
- Write to `.env.mainnet`

---

## Step 3 — Identify the CLI Binary

```bash
grep -A2 '\[\[bin\]\]' cli/Cargo.toml | grep 'name'
```

The CLI binary is typically `{{project-name}}_cli`.

---

## Step 4 — Run Deployment

Load the env file and run:

```bash
set -a && source .env && set +a  # or .env.testnet / .env.mainnet
cargo run --bin <CLI_BINARY> --features=livenet 2>&1
```

**Important:**
- The `--features=livenet` flag is mandatory
- Deployments can take 30-90 seconds per contract
- Use a generous timeout (10 minutes)

---

## Step 5 — Report Results

After execution, report:

1. **Exit status**: success or failure with exit code
2. **Key events**: lines containing `Deploying`, `Deploy hash`, `Contract`, `Success`, `Done`
3. **Errors**: lines containing `panicked`, `assertion failed`, `error[`, `Error`
4. **Verdict**: one-line summary

```
### Deployment Summary
**Network**: nctl/testnet/mainnet   **Exit code**: 0

#### Key Events
- Deployed MyContract at hash abc123...
- ...

#### Errors
None

#### Verdict
All contracts deployed successfully to nctl.
```
