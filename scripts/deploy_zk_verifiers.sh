#!/usr/bin/env bash
# Deploys three zk_verifier instances (one per real Circom circuit: age_proof,
# kyc_tier_proof, membership_proof), each initialized with its own real verification key
# from circuits/build/. Does NOT touch did_registry/credential_verifier/asp_registry/
# reputation_nft — those are unrelated to this real, separate ZK capability and keep their
# existing addresses.
#
# This is a genuinely different identity-proof mechanism from credential_verifier's
# classical Merkle inclusion check: these three verify real Groth16 zero-knowledge proofs
# (age >= 18, KYC tier >= N, Merkle membership — all without revealing the private data),
# using the same real Soroban BN254 host crypto stellar-zkstream's verifier already proved
# out. See circuits/README.md for why Circom/Groth16 instead of this repo's original Noir
# circuits (Noir's UltraHonk proving system needs curve support — Grumpkin — Soroban
# doesn't have).
set -euo pipefail

NETWORK="${STELLAR_NETWORK:-testnet}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEPLOYMENTS_FILE="$REPO_ROOT/deployments/$NETWORK.json"

if [ ! -f "$DEPLOYMENTS_FILE" ]; then
  echo "Expected an existing $DEPLOYMENTS_FILE — run scripts/deploy.sh first if this is a fresh environment." >&2
  exit 1
fi
if ! command -v jq &> /dev/null; then
  echo "This script needs 'jq' to safely merge new addresses into $DEPLOYMENTS_FILE (apt install jq / brew install jq)." >&2
  exit 1
fi

for c in age_proof kyc_tier_proof membership_proof; do
  if [ ! -f "$REPO_ROOT/circuits/build/$c/${c}_vk.hex" ]; then
    echo "Missing circuits/build/$c/${c}_vk.hex — run the pipeline in circuits/README.md first." >&2
    exit 1
  fi
done

echo "Deploying 3 zk_verifier instances to Stellar $NETWORK..."

if ! stellar keys address deployer >/dev/null 2>&1; then
  echo "No 'deployer' identity found. This script expects the SAME deployer used for the rest of this workspace's deployment." >&2
  exit 1
fi
stellar keys fund deployer --network "$NETWORK" || true
DEPLOYER_ADDR=$(stellar keys address deployer)

cd "$REPO_ROOT"
cargo build --release --target wasm32v1-none -p zk-verifier
WASM_DIR="target/wasm32v1-none/release"

deploy_verifier() {
  local circuit="$1"
  local vk_hex
  vk_hex=$(cat "circuits/build/$circuit/${circuit}_vk.hex")
  local id
  id=$(stellar contract deploy --wasm "$WASM_DIR/zk_verifier.wasm" --source deployer --network "$NETWORK")
  stellar contract invoke --id "$id" --source deployer --network "$NETWORK" \
    -- initialize --admin "$DEPLOYER_ADDR" --verification_key "$vk_hex" >&2
  echo "$id"
}

echo "Deploying zk_verifier for age_proof..."
AGE_PROOF_VERIFIER_ID=$(deploy_verifier age_proof)

echo "Deploying zk_verifier for kyc_tier_proof..."
KYC_TIER_VERIFIER_ID=$(deploy_verifier kyc_tier_proof)

echo "Deploying zk_verifier for membership_proof..."
MEMBERSHIP_VERIFIER_ID=$(deploy_verifier membership_proof)

TMP_FILE="$(mktemp)"
jq \
  --arg age "$AGE_PROOF_VERIFIER_ID" \
  --arg kyc "$KYC_TIER_VERIFIER_ID" \
  --arg mem "$MEMBERSHIP_VERIFIER_ID" \
  --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  '.contracts.age_proof_verifier = $age
   | .contracts.kyc_tier_verifier = $kyc
   | .contracts.membership_verifier = $mem
   | .notes.zk_verifiers = ("Deployed " + $ts + ". Three real Groth16 BN254 verifier instances, one per real Circom circuit (see circuits/README.md), each initialized with its own real verification key. This is separate from credential_verifier'"'"'s classical Merkle-inclusion system — these verify actual zero-knowledge proofs.")' \
  "$DEPLOYMENTS_FILE" > "$TMP_FILE"
mv "$TMP_FILE" "$DEPLOYMENTS_FILE"

echo ""
echo "Deployed:"
echo "  age_proof_verifier:       $AGE_PROOF_VERIFIER_ID"
echo "  kyc_tier_verifier:        $KYC_TIER_VERIFIER_ID"
echo "  membership_verifier:      $MEMBERSHIP_VERIFIER_ID"
echo "Updated: $DEPLOYMENTS_FILE"
echo ""
echo "Next steps (not done by this script):"
echo "  - review: git -C \"$REPO_ROOT\" diff"
echo "  - update sdk/README.md and README.md with these real addresses"
echo "  - git add -A && git commit"
