#!/usr/bin/env bash
# Deploys sybil_resistant_faucet to Stellar testnet, wired to the already-deployed
# credential_verifier (real cross-contract dependency, same pattern reputation_nft uses),
# funds it with real native XLM, and records the result in deployments/<network>.json.
# Does NOT touch did_registry/credential_verifier/asp_registry/reputation_nft/zk_verifier —
# this is an additive, standalone consumer of credential_verifier, not a redeploy of it.
set -euo pipefail

NETWORK="${STELLAR_NETWORK:-testnet}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEPLOYMENTS_FILE="$REPO_ROOT/deployments/$NETWORK.json"
CLAIM_AMOUNT="${CLAIM_AMOUNT:-5000000}"       # 0.5 XLM per claim, stroops
FUND_AMOUNT="${FUND_AMOUNT:-50000000}"        # 5 XLM funded into the faucet, stroops
CREDENTIAL_TYPE="${CREDENTIAL_TYPE:-kyc_tier_2}"

if [ ! -f "$DEPLOYMENTS_FILE" ]; then
  echo "Expected an existing $DEPLOYMENTS_FILE — run scripts/deploy.sh first if this is a fresh environment." >&2
  exit 1
fi
if ! command -v jq &> /dev/null; then
  echo "This script needs 'jq' to safely merge the new address into $DEPLOYMENTS_FILE (apt install jq / brew install jq)." >&2
  exit 1
fi

CREDENTIAL_VERIFIER_ID=$(jq -r '.contracts.credential_verifier' "$DEPLOYMENTS_FILE")
if [ -z "$CREDENTIAL_VERIFIER_ID" ] || [ "$CREDENTIAL_VERIFIER_ID" == "null" ]; then
  echo "No credential_verifier address found in $DEPLOYMENTS_FILE — run scripts/deploy.sh first." >&2
  exit 1
fi

if ! stellar keys address deployer >/dev/null 2>&1; then
  echo "No 'deployer' identity found. This script expects the SAME deployer used for the rest of this workspace's deployment." >&2
  exit 1
fi
stellar keys fund deployer --network "$NETWORK" || true
DEPLOYER_ADDR=$(stellar keys address deployer)

cd "$REPO_ROOT"
cargo build --release --target wasm32v1-none -p sybil-resistant-faucet
WASM_DIR="target/wasm32v1-none/release"

echo "Deploying sybil_resistant_faucet..."
FAUCET_ID=$(stellar contract deploy --wasm "$WASM_DIR/sybil_resistant_faucet.wasm" --source deployer --network "$NETWORK")

NATIVE_SAC_ID=$(stellar contract id asset --asset native --network "$NETWORK")

echo "Initializing (credential_verifier=$CREDENTIAL_VERIFIER_ID, token=$NATIVE_SAC_ID native XLM SAC, claim_amount=$CLAIM_AMOUNT, required_credential_type=$CREDENTIAL_TYPE)..."
stellar contract invoke --id "$FAUCET_ID" --source deployer --network "$NETWORK" \
  -- initialize \
  --admin "$DEPLOYER_ADDR" \
  --credential_verifier "$CREDENTIAL_VERIFIER_ID" \
  --token "$NATIVE_SAC_ID" \
  --claim_amount "$CLAIM_AMOUNT" \
  --required_credential_type "$CREDENTIAL_TYPE"

echo "Funding the faucet with $FUND_AMOUNT stroops of real native XLM from deployer..."
stellar contract invoke --id "$NATIVE_SAC_ID" --source deployer --network "$NETWORK" \
  -- transfer --from "$DEPLOYER_ADDR" --to "$FAUCET_ID" --amount "$FUND_AMOUNT"

TMP_FILE="$(mktemp)"
jq \
  --arg id "$FAUCET_ID" \
  --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg cv "$CREDENTIAL_VERIFIER_ID" \
  --arg native "$NATIVE_SAC_ID" \
  --arg amt "$CLAIM_AMOUNT" \
  --arg fund "$FUND_AMOUNT" \
  --arg ct "$CREDENTIAL_TYPE" \
  '.contracts.sybil_resistant_faucet = $id
   | .notes.sybil_resistant_faucet = ("Deployed " + $ts + ". Pays " + $amt + " stroops of native XLM (" + $native + ") per address, at most once, gated on a real cross-contract has_credential(user, \"" + $ct + "\") check against credential_verifier (" + $cv + ") — the same real gate reputation_nft::mint already uses. Funded with " + $fund + " stroops of real testnet XLM from the deployer.")' \
  "$DEPLOYMENTS_FILE" > "$TMP_FILE"
mv "$TMP_FILE" "$DEPLOYMENTS_FILE"

echo ""
echo "Deployed:"
echo "  sybil_resistant_faucet: $FAUCET_ID"
echo "Updated: $DEPLOYMENTS_FILE"
echo ""
echo "Next steps (not done by this script):"
echo "  - review: git -C \"$REPO_ROOT\" diff"
echo "  - update frontend/src/soroban.ts and README.md with this real address"
echo "  - git add -A && git commit"
