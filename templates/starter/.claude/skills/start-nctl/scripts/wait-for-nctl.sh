#!/bin/bash
# Wait for the NCTL node to be ready by polling the RPC endpoint.
# Usage: ./wait-for-nctl.sh [timeout_seconds]
# Default timeout: 120 seconds

TIMEOUT=${1:-120}
ELAPSED=0
INTERVAL=3
RPC_URL="http://localhost:11101/rpc"

echo "Waiting for NCTL node at $RPC_URL (timeout: ${TIMEOUT}s)..."

while [ $ELAPSED -lt $TIMEOUT ]; do
  if curl -s -m 2 "$RPC_URL" \
    -H 'content-type: application/json' \
    --data-raw '{"jsonrpc":"2.0","id":"1","method":"info_get_status"}' \
    2>/dev/null | grep -q '"jsonrpc"'; then
    echo "NCTL node is ready (after ${ELAPSED}s)."
    exit 0
  fi
  sleep $INTERVAL
  ELAPSED=$((ELAPSED + INTERVAL))
done

echo "ERROR: NCTL node did not become ready within ${TIMEOUT}s."
exit 1
