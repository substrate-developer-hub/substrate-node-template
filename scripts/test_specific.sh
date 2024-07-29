pallet="${3:-pallet-subtensor}"
features="${4:-pow-faucet}"

# RUST_LOG="pallet_subtensor=info" cargo test --release --features=$features -p $pallet --test $1 -- $2 --nocapture --exact

RUST_LOG=INFO cargo test --release --features=$features -p $pallet --test $1 -- $2 --nocapture --exact