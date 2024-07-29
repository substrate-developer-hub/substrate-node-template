#!/bin/bash

set -e

echo "*** Building node..."
cargo build

echo "*** Building new chainspecs..."

finney_genesis_temp=$(mktemp)
testfinney_genesis_temp=$(mktemp)
raw_spec_finney_temp=$(mktemp)
raw_spec_testfinney_temp=$(mktemp)

# Save old genesis state before doing anything
jq -r ".genesis" raw_spec_finney.json >"$finney_genesis_temp"
jq -r ".genesis" raw_spec_testfinney.json >"$testfinney_genesis_temp"

# Build new chainspecs
./target/debug/node-subtensor build-spec --raw --chain finney >"$raw_spec_finney_temp"
./target/debug/node-subtensor build-spec --chain finney >plain_spec_finney.json

./target/debug/node-subtensor build-spec --raw --chain test_finney >"$raw_spec_testfinney_temp"
./target/debug/node-subtensor build-spec --chain test_finney >plain_spec_testfinney.json

echo "*** Updating genesis..."

# The genesis is not allowed to change. Since the wasm genesis will change depending on the system
# architecture used, we need to extract the genesis from the old chain specs and insert them into
# the new chain specs to ensure there are no genesis mismatch issues.

# Update genesis in new chainspecs using the extracted genesis data from the temporary files
jq --slurpfile genesis "$finney_genesis_temp" '.genesis = $genesis[0]' "$raw_spec_finney_temp" >raw_spec_finney.json
jq --slurpfile genesis "$testfinney_genesis_temp" '.genesis = $genesis[0]' "$raw_spec_testfinney_temp" >raw_spec_testfinney.json

# Cleanup
rm -f "$finney_genesis_temp" "$testfinney_genesis_temp" "$raw_spec_finney_temp" "$raw_spec_testfinney_temp"

echo "*** Done!"
