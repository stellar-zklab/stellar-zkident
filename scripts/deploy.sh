#!/usr/bin/env bash
# Deploys every real contract in this workspace to Stellar testnet, wires up the real
# cross-contract dependencies (credential_verifier needs asp_registry's address,
# reputation_nft needs credential_verifier's address), and records the resulting
# contract IDs in deployments/<network>.json. Requires the `stellar` CLI already
# installed and on PATH.
set -euo pipefail

NETWORK="${STELLAR_NETWORK:-testnet}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$REPO_ROOT/deployments"
OUT_FILE="$OUT_DIR/$NETWORK.json"
mkdir -p "$OUT_DIR"

echo "Deploying stellar-zkident to Stellar $NETWORK..."

if ! stellar keys address deployer >/dev/null 2>&1; then
  echo "Generating deployer key..."
  stellar keys generate deployer
fi
stellar keys fund deployer --network "$NETWORK" || true
DEPLOYER_ADDR=$(stellar keys address deployer)

cd "$REPO_ROOT"
cargo build --release --target wasm32v1-none

WASM_DIR="target/wasm32v1-none/release"

deploy() {
  local wasm_name="$1"
  stellar contract deploy \
    --wasm "$WASM_DIR/$wasm_name.wasm" \
    --source deployer \
    --network "$NETWORK"
}

echo "Deploying asp_registry..."
ASP_REGISTRY_ID=$(deploy asp_registry)
stellar contract invoke --id "$ASP_REGISTRY_ID" --source deployer --network "$NETWORK" \
  -- initialize --admin "$DEPLOYER_ADDR"

echo "Deploying credential_verifier..."
VERIFIER_ID=$(deploy credential_verifier)
stellar contract invoke --id "$VERIFIER_ID" --source deployer --network "$NETWORK" \
  -- initialize --admin "$DEPLOYER_ADDR" --asp_registry "$ASP_REGISTRY_ID"

echo "Deploying did_registry..."
DID_ID=$(deploy did_registry)
stellar contract invoke --id "$DID_ID" --source deployer --network "$NETWORK" \
  -- initialize --admin "$DEPLOYER_ADDR"

echo "Deploying reputation_nft..."
REPUTATION_ID=$(deploy reputation_nft)
stellar contract invoke --id "$REPUTATION_ID" --source deployer --network "$NETWORK" \
  -- initialize --admin "$DEPLOYER_ADDR" --cv "$VERIFIER_ID"

cat > "$OUT_FILE" <<EOF
{
  "network": "$NETWORK",
  "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "deployer": "$DEPLOYER_ADDR",
  "contracts": {
    "asp_registry": "$ASP_REGISTRY_ID",
    "credential_verifier": "$VERIFIER_ID",
    "did_registry": "$DID_ID",
    "reputation_nft": "$REPUTATION_ID"
  }
}
EOF

echo ""
echo "Deployed to $NETWORK — recorded in $OUT_FILE"
cat "$OUT_FILE"
