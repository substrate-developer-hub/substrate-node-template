#!/usr/bin/env bash

set -TeE

## Find true directory this script resides in
__SOURCE__="${BASH_SOURCE[0]}"
while [[ -h "${__SOURCE__}" ]]; do
    __SOURCE__="$(find "${__SOURCE__}" -type l -ls | sed -n 's@^.* -> \(.*\)@\1@p')"
done
__DIR__="$(cd -P "$(dirname "${__SOURCE__}")" && pwd)"
__G_DIR__="$(dirname "${__DIR__}")"

## Sub-directory name under: ../target/
_target_dir_name='tarpaulin'

_tarpaulin_options=(
	--skip-clean
	--no-fail-fast
	--ignore-tests
	--exclude-files "${__G_DIR__}/target/*"
)

if (( VERBOSE )); then
	_tarpaulin_options+=( --verbose )
fi

##
# Do not fool around with contents of: ../target/debug
# - https://lib.rs/crates/cargo-tarpaulin#readme-recompilation
_tarpaulin_options+=(
	--target-dir "${__G_DIR__}/target/${_target_dir_name}"
)

##
# Allow additional CLI parameters too
_extra_arguments=("${@}")
if ((${#_extra_arguments[@]})); then
	_tarpaulin_options+=( "${_extra_arguments[@]}" )
fi

SKIP_WASM_BUILD=1 cargo +nightly tarpaulin "${_tarpaulin_options[@]}" |
	grep -vE '^\|\|\s+(target/debug)'

