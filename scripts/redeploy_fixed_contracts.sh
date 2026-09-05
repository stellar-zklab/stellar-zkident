#!/usr/bin/env bash
# Targeted redeploy of asp_registry, credential_verifier, and reputation_nft, after
# fixing the admin-impersonation holes in register_asp()/mint() and the re-init hijack in
# credential_verifier.initialize(). did_registry did NOT change and keeps its existing
# testnet address — this script does not touch it, and other repos (swfix's dashboard and
# SDK e2e example) depend on that address staying stable.
#
# All three redeployed contracts depend on each other's addresses at init time
# (credential_verifier needs asp_registry's ID, reputation_nft needs credential_verifier's
# ID), so they must be redeployed together in this order, not independently.
#
# This also re-registers the same demo ASP credential the pre-fix asp_registry had
# (deployer acting as its own attestation service for a 'kyc_tier_2' credential), using
# the SAME Merkle root already recorded in deployments/testnet.json's notes — so the
# frontend's hardcoded demo Merkle proof (frontend/src/soroban.ts's DEMO_MERKLE_PROOF_HEX)
# keeps verifying against the new asp_registry without any frontend proof-data changes.
set -euo pipefail

NETWORK="${STELLAR_NETWORK:-testnet}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEPLOYMENTS_FILE="$REPO_ROOT/deployments/$NETWORK.json"

if [ ! -f "$DEPLOYMENTS_FILE" ]; then
  echo "Expected an existing $DEPLOYMENTS_FILE to merge into — run scripts/deploy.sh first if this is a fresh environment." >&2
  exit 1
fi
if ! command -v jq &> /dev/null; then
  echo "This script needs 'jq' to safely merge new addresses into $DEPLOYMENTS_FILE (apt install jq / brew install jq)." >&2
  exit 1
fi

DEMO_ROOT=$(jq -r '.notes.asp_registry' "$DEPLOYMENTS_FILE" | grep -oE '[0-9a-f]{64}' | head -1)
if [ -z "$DEMO_ROOT" ]; then
  echo "Could not find the existing demo Merkle root in $DEPLOYMENTS_FILE's notes.asp_registry — cannot safely re-register the demo credential without it. Aborting rather than guessing." >&2
  exit 1
fi
echo "Found existing demo Merkle root to re-register: $DEMO_ROOT"

echo "Redeploying asp_registry, credential_verifier, reputation_nft to Stellar $NETWORK (did_registry untouched)..."

if ! stellar keys address deployer >/dev/null 2>&1; then
  echo "No 'deployer' identity found. This script expects the SAME deployer used for the original deployment." >&2
  exit 1
fi
stellar keys fund deployer --network "$NETWORK" || true
DEPLOYER_ADDR=$(stellar keys address deployer)
EXPECTED_DEPLOYER=$(jq -r '.deployer' "$DEPLOYMENTS_FILE")
if [ "$DEPLOYER_ADDR" != "$EXPECTED_DEPLOYER" ]; then
  echo "WARNING: local 'deployer' identity ($DEPLOYER_ADDR) does not match the deployer recorded in $DEPLOYMENTS_FILE ($EXPECTED_DEPLOYER). The demo ASP credential is registered as the deployer acting as its own attestation service, so a different deployer here changes who that demo credential is 'about'." >&2
fi

OLD_ASP_REGISTRY=$(jq -r '.contracts.asp_registry' "$DEPLOYMENTS_FILE")
OLD_CREDENTIAL_VERIFIER=$(jq -r '.contracts.credential_verifier' "$DEPLOYMENTS_FILE")
OLD_REPUTATION_NFT=$(jq -r '.contracts.reputation_nft' "$DEPLOYMENTS_FILE")
DID_REGISTRY_ID=$(jq -r '.contracts.did_registry' "$DEPLOYMENTS_FILE")

cd "$REPO_ROOT"
cargo build --release --target wasm32v1-none --package asp-registry --package credential-verifier --package reputation-nft
WASM_DIR="target/wasm32v1-none/release"

echo "Deploying asp_registry..."
NEW_ASP_REGISTRY=$(stellar contract deploy --wasm "$WASM_DIR/asp_registry.wasm" --source deployer --network "$NETWORK")
stellar contract invoke --id "$NEW_ASP_REGISTRY" --source deployer --network "$NETWORK" \
  -- initialize --admin "$DEPLOYER_ADDR"

echo "Re-registering the demo ASP credential (same root as before: $DEMO_ROOT)..."
stellar contract invoke --id "$NEW_ASP_REGISTRY" --source deployer --network "$NETWORK" \
  -- register_asp --admin "$DEPLOYER_ADDR" --asp "$DEPLOYER_ADDR" --merkle_root "$DEMO_ROOT"

echo "Deploying credential_verifier..."
NEW_CREDENTIAL_VERIFIER=$(stellar contract deploy --wasm "$WASM_DIR/credential_verifier.wasm" --source deployer --network "$NETWORK")
stellar contract invoke --id "$NEW_CREDENTIAL_VERIFIER" --source deployer --network "$NETWORK" \
  -- initialize --admin "$DEPLOYER_ADDR" --asp_registry "$NEW_ASP_REGISTRY"

echo "Deploying reputation_nft..."
NEW_REPUTATION_NFT=$(stellar contract deploy --wasm "$WASM_DIR/reputation_nft.wasm" --source deployer --network "$NETWORK")
stellar contract invoke --id "$NEW_REPUTATION_NFT" --source deployer --network "$NETWORK" \
  -- initialize --admin "$DEPLOYER_ADDR" --cv "$NEW_CREDENTIAL_VERIFIER"

TMP_FILE="$(mktemp)"
jq \
  --arg new_asp "$NEW_ASP_REGISTRY" --arg old_asp "$OLD_ASP_REGISTRY" \
  --arg new_cv "$NEW_CREDENTIAL_VERIFIER" --arg old_cv "$OLD_CREDENTIAL_VERIFIER" \
  --arg new_rep "$NEW_REPUTATION_NFT" --arg old_rep "$OLD_REPUTATION_NFT" \
  --arg root "$DEMO_ROOT" \
  --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  '.contracts.asp_registry = $new_asp
   | .contracts.credential_verifier = $new_cv
   | .contracts.reputation_nft = $new_rep
   | .notes.asp_registry = ("Redeployed " + $ts + " after fixing register_asp() to check the caller is actually the stored admin, not just any address passing itself off as one — " + $old_asp + " was the pre-fix instance and is stale. Re-registered the same demo ASP credential with the same root: " + $root + ".")
   | .notes.credential_verifier = ("Redeployed " + $ts + " after adding a re-init guard to initialize() (previously callable twice) — " + $old_cv + " was the pre-fix instance and is stale. Points at the new asp_registry (" + $new_asp + ").")
   | .notes.reputation_nft = ("Redeployed " + $ts + " after fixing mint() to check the caller is actually the stored admin — " + $old_rep + " was the pre-fix instance and is stale. Points at the new credential_verifier (" + $new_cv + ").")' \
  "$DEPLOYMENTS_FILE" > "$TMP_FILE"
mv "$TMP_FILE" "$DEPLOYMENTS_FILE"

for f in README.md sdk/README.md frontend/src/soroban.ts; do
  path="$REPO_ROOT/$f"
  [ -f "$path" ] || continue
  changed=0
  for pair in "$OLD_ASP_REGISTRY:$NEW_ASP_REGISTRY" "$OLD_CREDENTIAL_VERIFIER:$NEW_CREDENTIAL_VERIFIER" "$OLD_REPUTATION_NFT:$NEW_REPUTATION_NFT"; do
    old="${pair%%:*}"; new="${pair##*:}"
    if grep -q "$old" "$path"; then
      sed -i "s/$old/$new/g" "$path"
      changed=1
    fi
  done
  [ "$changed" = "1" ] && echo "Patched $f"
done

echo ""
echo "Redeployed:"
echo "  asp_registry:         $OLD_ASP_REGISTRY -> $NEW_ASP_REGISTRY"
echo "  credential_verifier:  $OLD_CREDENTIAL_VERIFIER -> $NEW_CREDENTIAL_VERIFIER"
echo "  reputation_nft:       $OLD_REPUTATION_NFT -> $NEW_REPUTATION_NFT"
echo "  did_registry (unchanged): $DID_REGISTRY_ID"
echo "Updated: $DEPLOYMENTS_FILE"
echo ""
echo "Next steps (not done by this script):"
echo "  - review: git -C \"$REPO_ROOT\" diff"
echo "  - git add -A && git commit"
