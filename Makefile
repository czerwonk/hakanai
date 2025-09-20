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
build-wasm: check-wasm-pack clean-wasm
	cd wasm && ./build.sh

.PHONY: check-npm
check-npm:
	@which npm > /dev/null 2>&1 || (echo "Error: npm is not installed" && exit 1)
	@which npx > /dev/null 2>&1 || (echo "Error: npx is not installed" && exit 1)

.PHONY: check-wasm-pack
check-wasm-pack:
	@which wasm-pack > /dev/null 2>&1 || (echo "Warning: wasm-pack not installed. Install with: cargo install wasm-pack" && echo "Continuing without WASM support...")

.PHONY: build-ts
build-ts: clean-ts build-wasm
	cd typescript && npm run build

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
	cd typescript/tests && npm test

# Clean builds
.PHONY: clean
clean: clean-rust clean-ts clean-wasm

.PHONY: clean-rust
clean-rust:
	$(CARGO) clean

.PHONY: clean-ts
clean-ts:
	rm -rf server/includes/*.js server/includes/core server/includes/components server/includes/*.min.css
	rm -rf typescript/tests/node_modules typescript/tests/coverage

.PHONY: clean-wasm
clean-wasm:
	rm -rf server/includes/*.wasm
