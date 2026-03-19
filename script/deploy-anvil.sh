#!/bin/bash
# scripts/deploy-anvil.sh

echo "Starting Anvil fork..."
# Kill any existing anvil process
pkill -f anvil || true

# Start anvil in background (fork Polygon for lower gas costs)
anvil --fork-url https://polygon-mainnet.g.alchemy.com/v2/$ALCHEMY_KEY \
      --chain-id 137 \
      --accounts 10 \
      --balance 10000 \
      --block-time 2 &

# Wait for anvil to start
sleep 3

echo "Deploying contracts..."
forge script script/Deploy.s.sol:DeployScript \
    --rpc-url http://127.0.0.1:8545 \
    --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
    --broadcast \
    -vvvv

echo "Deployment complete!"