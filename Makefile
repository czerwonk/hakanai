# Hakanai Makefile
# Build system for Rust workspace and TypeScript compilation

# Variables
CARGO := cargo
TSC := tsc

# TypeScript files
TS_SRC := server/src/includes/hakanai-client.ts server/src/includes/common-utils.ts server/src/includes/i18n.ts
JS_OUT := server/src/includes/hakanai-client.js server/src/includes/common-utils.js server/src/includes/i18n.js

# Default target
.PHONY: all
all: build

# Build everything
.PHONY: build
build: build-ts build-rust

# Build Rust workspace (depends on TypeScript)
.PHONY: build-rust
build-rust: build-ts
	$(CARGO) build --workspace --verbose

# Build TypeScript
.PHONY: build-ts
build-ts: $(JS_OUT)

$(JS_OUT): $(TS_SRC) tsconfig.json
	$(TSC)

# Release builds
.PHONY: release
release: build-ts build-rust-release

.PHONY: build-rust-release
build-rust-release: build-ts
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
