#!/usr/bin/env bash
# Multisig check on Casper TESTNET: a native transfer from a 2-of-N account, signed by two keys
# on their own (as two machines would), merged with odra-casper-signing and sent, once as a
# TransactionV1 and once as a legacy Deploy. First each one is sent with one signature only,
# which the node must reject.
#
#   KEY_1=path/to/key1.pem KEY_2=path/to/key2.pem ./examples/multisig-testnet.sh
#
# KEY_1 is the account's own key; KEY_2 is one of its associated keys. The account needs
# deployment >= 2, with KEY_1 and KEY_2 weighing 1 each. The script only checks this and prints
# how to set it up. Optional: NODE (default https://node.testnet.casper.network/rpc),
# CSPR_CLOUD_AUTH_TOKEN for node.testnet.cspr.cloud, TARGET (default KEY_2's account),
# AMOUNT in motes (default 2500000000, the 2.5 CSPR minimum transfer).
#
# Safety: it reads the keys only through the example binary, never prints them, and stops
# unless the node says its chain is casper-test. Run from odra-casper/signing.
set -euo pipefail

NODE="${NODE:-https://node.testnet.casper.network/rpc}"
AMOUNT="${AMOUNT:-2500000000}"
: "${KEY_1:?set KEY_1 to the account key file}" "${KEY_2:?set KEY_2 to an associated key file}"
for tool in curl jq cargo; do command -v "$tool" >/dev/null || { echo "needs $tool"; exit 1; }; done

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
cargo build -q --example multisig_testnet
BIN="$(cargo metadata --format-version 1 --no-deps | jq -r .target_directory)/debug/examples/multisig_testnet"

rpc() { # method, params JSON
    local body
    body="$(jq -nc --arg m "$1" --argjson p "$2" '{jsonrpc:"2.0",id:1,method:$m,params:$p}')"
    if [ -n "${CSPR_CLOUD_AUTH_TOKEN:-}" ]; then
        # The token goes in from a file descriptor, so it is not on the command line either.
        curl -sS -H 'Content-Type: application/json' \
            -H @<(printf 'Authorization: %s\n' "$CSPR_CLOUD_AUTH_TOKEN") --data "$body" "$NODE"
    else
        curl -sS -H 'Content-Type: application/json' --data "$body" "$NODE"
    fi
}

CHAIN="$(rpc info_get_status '[]' | jq -r '.result.chainspec_name')"
if [ "$CHAIN" != "casper-test" ]; then
    echo "The node at $NODE says its chain is '$CHAIN', not casper-test. Stopping."; exit 1
fi
echo "node: $NODE (casper-test)"

PK1="$("$BIN" public-key "$KEY_1")"; PK2="$("$BIN" public-key "$KEY_2")"
ACCOUNT_JSON="$WORK/account.json"
rpc state_get_account_info "$(jq -nc --arg k "$PK1" '{account_identifier:$k,block_identifier:null}')" \
    | jq '.result.account' > "$ACCOUNT_JSON"
echo "account:"; jq '{account_hash, associated_keys, action_thresholds}' "$ACCOUNT_JSON"
H2="$(rpc state_get_account_info "$(jq -nc --arg k "$PK2" '{account_identifier:$k,block_identifier:null}')" | jq -r '.result.account.account_hash // empty')"
TARGET="${TARGET:-$H2}"
[ -n "$TARGET" ] || { echo "KEY_2 has no account on testnet; set TARGET=account-hash-..."; exit 1; }

DEPLOYMENT="$(jq -r '.action_thresholds.deployment' "$ACCOUNT_JSON")"
if [ "$DEPLOYMENT" -lt 2 ]; then
    echo "The account needs deployment >= 2 for this check. For example, with KEY_1 configured:"
    echo "  cargo odra account configure --set-key $PK2:1 --deployment 2 --key-management 2 \\"
    echo "    --chain-name casper-test --node-address $NODE --secret-key \$KEY_1"
    echo "(After that, KEY_1 alone can no longer send from or change this account.)"
    exit 1
fi

wait_for() { # transaction hash JSON
    for _ in $(seq 1 60); do
        local result
        result="$(rpc info_get_transaction "$(jq -nc --argjson h "$1" '{transaction_hash:$h,finalized_approvals:false}')")"
        if [ "$(jq -r '.result.execution_info.execution_result // empty' <<<"$result")" != "" ]; then
            jq '.result.execution_info.execution_result | to_entries[0].value | {error_message, consumed, cost}' <<<"$result"
            return
        fi
        sleep 5
    done
    echo "not executed within 5 minutes"
}

for KIND in v1 deploy; do
    echo; echo "=== $KIND ==="
    TX="$WORK/$KIND.json"
    "$BIN" build-transfer "$KIND" "$PK1" "$TARGET" "$AMOUNT" "$TX"
    "$BIN" sign "$TX" "$KEY_1" > "$WORK/$KIND.sig1"   # machine 1
    "$BIN" sign "$TX" "$KEY_2" > "$WORK/$KIND.sig2"   # machine 2

    echo "--- one signature: inspect, then the node should reject it"
    "$BIN" merge "$TX" "$WORK/$KIND.one.json" "$WORK/$KIND.sig1"
    "$BIN" inspect "$WORK/$KIND.one.json" "$ACCOUNT_JSON"
    rpc account_put_transaction "$(jq -c '{transaction: .}' "$WORK/$KIND.one.json")" | jq '.error // .result'

    echo "--- both signatures: inspect, send, wait"
    "$BIN" merge "$TX" "$WORK/$KIND.both.json" "$WORK/$KIND.sig1" "$WORK/$KIND.sig2"
    "$BIN" inspect "$WORK/$KIND.both.json" "$ACCOUNT_JSON"
    SENT="$(rpc account_put_transaction "$(jq -c '{transaction: .}' "$WORK/$KIND.both.json")")"
    jq '.error // .result' <<<"$SENT"
    HASH="$(jq -c '.result.transaction_hash // empty' <<<"$SENT")"
    [ -n "$HASH" ] && wait_for "$HASH"
done
