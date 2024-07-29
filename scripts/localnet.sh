#!/bin/bash

# Determine the directory this script resides in. This allows invoking it from any location.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

# The base directory of the subtensor project
BASE_DIR="$SCRIPT_DIR/.."

# get parameters
# Get the value of fast_blocks from the first argument
fast_blocks=${1:-"True"}

# Check the value of fast_blocks
if [ "$fast_blocks" == "False" ]; then
  # Block of code to execute if fast_blocks is False
  echo "fast_blocks is Off"
  : "${CHAIN:=local}"
  : "${BUILD_BINARY:=1}"
  : "${FEATURES:="pow-faucet runtime-benchmarks"}"
else
  # Block of code to execute if fast_blocks is not False
  echo "fast_blocks is On"
  : "${CHAIN:=local}"
  : "${BUILD_BINARY:=1}"
  : "${FEATURES:="pow-faucet runtime-benchmarks fast-blocks"}"
fi

SPEC_PATH="${SCRIPT_DIR}/specs/"
FULL_PATH="$SPEC_PATH$CHAIN.json"

# Kill any existing nodes which may have not exited correctly after a previous
# run.
pkill -9 'node-subtensor'

if [ ! -d "$SPEC_PATH" ]; then
  echo "*** Creating directory ${SPEC_PATH}..."
  mkdir $SPEC_PATH
fi

if [[ $BUILD_BINARY == "1" ]]; then
  echo "*** Building substrate binary..."
  cargo build --release --features "$FEATURES" --manifest-path "$BASE_DIR/Cargo.toml"
  echo "*** Binary compiled"
fi

echo "*** Building chainspec..."
"$BASE_DIR/target/release/node-subtensor" build-spec --disable-default-bootnode --raw --chain $CHAIN >$FULL_PATH
echo "*** Chainspec built and output to file"

echo "*** Purging previous state..."
"$BASE_DIR/target/release/node-subtensor" purge-chain -y --base-path /tmp/bob --chain="$FULL_PATH" >/dev/null 2>&1
"$BASE_DIR/target/release/node-subtensor" purge-chain -y --base-path /tmp/alice --chain="$FULL_PATH" >/dev/null 2>&1
echo "*** Previous chainstate purged"

echo "*** Starting localnet nodes..."
alice_start=(
  "$BASE_DIR/target/release/node-subtensor"
  --base-path /tmp/alice
  --chain="$FULL_PATH"
  --alice
  --port 30334
  --rpc-port 9946
  --validator
  --rpc-cors=all
  --allow-private-ipv4
  --discover-local
)

bob_start=(
  "$BASE_DIR"/target/release/node-subtensor
  --base-path /tmp/bob
  --chain="$FULL_PATH"
  --bob
  --port 30335
  --rpc-port 9945
  --validator
  --allow-private-ipv4
  --discover-local
)

trap 'pkill -P $$' EXIT SIGINT SIGTERM

(
  ("${alice_start[@]}" 2>&1) &
  ("${bob_start[@]}" 2>&1)
  wait
)
