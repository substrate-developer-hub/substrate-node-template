cargo build --release --features runtime-benchmarks
./target/production/node-subtensor benchmark pallet \
  --chain=local \
  --pallet=pallet_registry \
  --extrinsic="*" \
  --output=pallets/registry/src/weights.rs \
  --template=./.maintain/frame-weight-template.hbs
