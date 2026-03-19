#!/usr/bin/env bash
set -euo pipefail

# -----------------------------------------------------------------------------
# Setup Environment
# -----------------------------------------------------------------------------
# Default to development if not explicitly set
RUST_ENV="${RUST_ENV:-development}"

echo "[INFO] Starting upgradeable deployment sequence for '$RUST_ENV'..."

# Ensure base .env for this environment exists
if [[ ! -f ".env.${RUST_ENV}" ]]; then
  echo "[ERROR] Missing .env.${RUST_ENV}"
  exit 1
fi

# Load environment variables (RPC URL, private key, etc.)
set -a
source ".env.${RUST_ENV}"
set +a

# -----------------------------------------------------------------------------
# Run Foundry Deployment
# -----------------------------------------------------------------------------
DEPLOY_LOG="deployment.${RUST_ENV}.log"
DEPLOY_ENV="deployed.${RUST_ENV}.env"

echo "[INFO] Running forge deployment script..."
forge script script/Deploy.s.sol \
  --rpc-url "$BLOCKCHAIN_RPC_URL" \
  --private-key "$BLOCKCHAIN_PRIVATE_KEY" \
  --broadcast \
  -vvvv \
  | tee "$DEPLOY_LOG"

# -----------------------------------------------------------------------------
# Parse & Save Contract Addresses
# -----------------------------------------------------------------------------
echo "[INFO] Extracting deployed contract addresses..."
grep -E "CONTRACT_" "$DEPLOY_LOG" > "$DEPLOY_ENV" || true

if [[ ! -s "$DEPLOY_ENV" ]]; then
  echo "[ERROR] No contract addresses were captured!"
  exit 1
fi

echo "[INFO] Deployment complete."
echo "[INFO] Contract addresses written to $DEPLOY_ENV"
echo "----------------------------------------------------------------"
cat "$DEPLOY_ENV"
echo "----------------------------------------------------------------"
