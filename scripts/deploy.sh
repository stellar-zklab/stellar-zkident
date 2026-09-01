#!/usr/bin/env bash
set -euo pipefail

NETWORK="${STELLAR_NETWORK:-testnet}"
echo "🪪 Deploying stellar-zkident contracts to Stellar $NETWORK..."

cargo build --release --target wasm32v1-none

echo "🚀 Deploying did_registry..."
DID_ID=$(stellar contract deploy --wasm target/wasm32v1-none/release/did_registry.wasm --source deployer --network "$NETWORK")

echo "🚀 Deploying credential_verifier..."
VERIFIER_ID=$(stellar contract deploy --wasm target/wasm32v1-none/release/credential_verifier.wasm --source deployer --network "$NETWORK")

echo ""
echo "═══════════════════════════════════════════════════"
echo "🎉 stellar-zkident deployed successfully to $NETWORK!"
echo "  did_registry Contract ID         : $DID_ID"
echo "  credential_verifier Contract ID : $VERIFIER_ID"
echo "═══════════════════════════════════════════════════"
