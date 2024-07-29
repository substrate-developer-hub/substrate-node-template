#!/usr/bin/env bash

DEFAULT_BIN_PATH='./target/production/node-subtensor'
BIN_PATH=$DEFAULT_BIN_PATH
TMP_SPEC='temp.json'
OUTPUT_FILE='benchmarking.txt'

# Getting arguments from user
while [[ $# -gt 0 ]]; do
    case $1 in
        -p | --bin-path)
            BIN_PATH="$2"
            shift
            shift
        ;;
        -* | --*)
            echo "Unknown option $1"
            exit 1
        ;;
        *)
            POSITIONAL_ARGS+=("$1")
            shift
        ;;
    esac
done

# Ensure binary exists before node-subtensor executions
if [ ! -f $BIN_PATH ]; then
    if [[ "$DEFAULT_BIN_PATH" == "$BIN_PATH" ]]; then
        cargo build --profile production --features runtime-benchmarks
    else
        echo "Binary '$BIN_PATH' does not exist. You can use -p or --bin-path to specify a different location."
        exit 1
    fi
fi

# Build Temporary Spec
$BIN_PATH build-spec --disable-default-bootnode --raw --chain local >$TMP_SPEC

# Run benchmark
$BIN_PATH benchmark pallet \
--chain=$TMP_SPEC \
--pallet pallet-subtensor --extrinsic 'schedule_coldkey_swap' \
--output $OUTPUT_FILE

rm $TMP_SPEC
