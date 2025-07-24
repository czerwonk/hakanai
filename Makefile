# Hakanai Makefile
# Build system for Rust workspace with integrated TypeScript compilation

# Variables
CARGO := cargo

.PHONY: all
all: build

.PHONY: build
build:
	$(CARGO) build --workspace --verbose

.PHONY: build-wasm
build-ts: clean-wasm
	cd wasm && ./build.sh

.PHONY: build-ts
build-ts: clean-ts build-wasm
	npm run build

.PHONY: release
release:
	$(CARGO) build --release --workspace --verbose

# Test targets
.PHONY: test
test: test-rust test-ts

.PHONY: test-rust
test-rust:
	$(CARGO) test --verbose

.PHONY: test-ts
test-ts: build-ts
	cd tests && npm test

# Clean builds
.PHONY: clean
clean: clean-rust clean-ts clean-wasm

.PHONY: clean-rust
clean-rust:
	$(CARGO) clean

.PHONY: clean-ts
clean-ts:
	rm -rf server/src/includes/*.js server/src/includes/core server/src/includes/components
	rm -rf tests/node_modules tests/coverage

.PHONY: clean-wasm
clean-ts:
	rm -rf server/src/includes/*.wasm
