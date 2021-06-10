.PHONY: init
init:
	./scripts/init.sh

.PHONY: check
check:
	SKIP_WASM_BUILD=1 cargo check --release

.PHONY: test
test:
	SKIP_WASM_BUILD=1 cargo test --release --all

.PHONY: build
build:
	 cargo build --release

.PHONY: run
run:
	./target/release/node-template --dev --tmp

.PHONY: debug
debug:
	./target/release/node-template --dev --tmp -lruntime=debug