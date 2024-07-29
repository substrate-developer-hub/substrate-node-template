cargo build --profile production --features runtime-benchmarks
./target/production/node-subtensor benchmark pallet \
  --chain=local \
  --pallet=pallet_admin_utils \
  --extrinsic="*" \
  --steps 50 \
  --repeat 20 \
  --output=pallets/admin-utils/src/weights.rs \
  --template=./.maintain/frame-weight-template.hbs
