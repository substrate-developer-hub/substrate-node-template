.PHONY: init
init:
	./scripts/init.sh

.PHONY: check
check:
	WASM_BUILD_TOOLCHAIN=nightly-2021-01-10 cargo check

.PHONY: test
test:
	WASM_BUILD_TOOLCHAIN=nightly-2021-01-10 cargo test --all

.PHONY: run
run:
	WASM_BUILD_TOOLCHAIN=nightly-2021-01-10 cargo run --release -- --dev --tmp

.PHONY: build
build:
	WASM_BUILD_TOOLCHAIN=nightly-2021-01-10 cargo build --release
