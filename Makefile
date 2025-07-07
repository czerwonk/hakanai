# Hakanai Makefile
# Build system for Rust workspace and TypeScript compilation

# Variables
CARGO := cargo
TSC := tsc

# TypeScript files
TS_SRC := server/src/includes/hakanai-client.ts
JS_OUT := server/src/includes/hakanai-client.js

# Default target
.PHONY: all
all: build

# Build everything
.PHONY: build
build: build-rust build-ts

# Build Rust workspace
.PHONY: build-rust
build-rust:
	$(CARGO) build --workspace --verbose

# Build TypeScript
.PHONY: build-ts
build-ts: $(JS_OUT)

$(JS_OUT): $(TS_SRC) tsconfig.json
	$(TSC)

# Release builds
.PHONY: release
release: build-rust-release build-ts

.PHONY: build-rust-release
build-rust-release:
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
clean: clean-rust clean-ts

.PHONY: clean-rust
clean-rust:
	$(CARGO) clean

.PHONY: clean-ts
clean-ts:
	rm -f $(JS_OUT)
	rm -rf tests/node_modules tests/coverage
