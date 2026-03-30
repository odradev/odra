#!/bin/bash
# Extract Casper user keys from the Docker NCTL container.

project_root=$(git rev-parse --show-toplevel)
cd "$project_root"

mkdir -p .node-keys

docker exec mynctl /bin/bash -c \
  "cat /home/casper/casper-nctl/assets/net-1/users/user-1/secret_key.pem" \
  > .node-keys/secret_key.pem

docker exec mynctl /bin/bash -c \
  "cat /home/casper/casper-nctl/assets/net-1/users/user-2/secret_key.pem" \
  > .node-keys/secret_key_1.pem
