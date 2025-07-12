# Hakanai Makefile
# Build system for Rust workspace with integrated TypeScript compilation

# Variables
CARGO := cargo
TSC := tsc

# TypeScript source files (for reference and manual compilation if needed)
TS_SRC := server/src/typescript/hakanai-client.ts server/src/typescript/common-utils.ts server/src/typescript/i18n.ts server/src/typescript/get-secret.ts server/src/typescript/create-secret.ts server/src/typescript/types.ts
# Generated JavaScript files (output to includes directory)
JS_OUT := server/src/includes/hakanai-client.js server/src/includes/common-utils.js server/src/includes/i18n.js server/src/includes/get-secret.js server/src/includes/create-secret.js server/src/includes/types.js

# Default target
.PHONY: all
all: build

# Build everything (TypeScript compilation now integrated in build.rs)
.PHONY: build
build:
	$(CARGO) build --workspace --verbose

# Manual TypeScript compilation (optional - normally handled by build.rs)
.PHONY: build-ts
build-ts: clean-ts $(JS_OUT)

$(JS_OUT): $(TS_SRC) tsconfig.json
	$(TSC)

# Release builds (TypeScript compilation handled by build.rs)
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
clean: clean-rust clean-ts

.PHONY: clean-rust
clean-rust:
	$(CARGO) clean

.PHONY: clean-ts
clean-ts:
	rm -f $(JS_OUT)
	rm -rf tests/node_modules tests/coverage
