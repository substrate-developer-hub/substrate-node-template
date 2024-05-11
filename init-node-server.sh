#!/bin/bash

PASSWORD="Pl3n17udN0d3"

plenitud key insert --base-path /data/node01 --chain dev --scheme Ecdsa --suri "vacuum mandate digital voice play control another milk attitude install diary shield" --password $PASSWORD --key-type aura
# plenitud key insert --base-path /data/node01 --chain spec.json --scheme Ecdsa --suri "vacuum mandate digital voice play control another milk attitude install diary shield" --password $PASSWORD --key-type aura

plenitud --base-path /data/node01 \
  --chain dev \
  --port 30333 \
  --rpc-port 9945 \
  --telemetry-url "wss://telemetry.polkadot.io/submit/ 0" \
  --validator \
  --rpc-methods Unsafe \
  --name plenitud-node \
  --rpc-external --rpc-cors all \
  --ws-external --no-mdns \
  --node-key 2980169f85d6d5e7f82b62ff01ce679bfd3d8dedc0d244c556336da97f03fc8b \
  --pruning archive \
  --offchain-worker always \
  --password $PASSWORD
