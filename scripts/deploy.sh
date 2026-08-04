#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/../.env"
NETWORK="${STELLAR_NETWORK:-testnet}"
echo "Deploying stellar-zkident to $NETWORK..."
cargo build --release --target wasm32v1-none

for contract in did_registry credential_verifier reputation_nft asp_registry; do
    ID=$(stellar contract deploy \
        --wasm "target/wasm32v1-none/release/${contract//-/_}.wasm" \
        --source "$STELLAR_ACCOUNT" --network "$NETWORK")
    echo "✅ $contract: $ID"
done
