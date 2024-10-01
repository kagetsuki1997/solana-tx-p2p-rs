#!/usr/bin/env bash

set -o errexit
set -o errtrace
set -o nounset
set -o pipefail
# Uncomment for debugging purpose
# set -o xtrace

export SOLANA_TX_P2P_RELAY_LEADER_DURATION=10s
export SOLANA_TX_P2P_SIGNING_LEADER_DURATION=15s
export SOLANA_TX_P2P_SOLANA_PROGRAM_ID=C4zHy4qLZsqTk7jSRTfD2riHdRUJzaWwBfceovJiiBwR
export SOLANA_TX_P2P_SOLANA_RPC_URL=http://127.0.0.1:8899

cargo run --bin solana-tx-p2p node
