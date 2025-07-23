# Hakanai Makefile
# Build system for Rust workspace with integrated TypeScript compilation

# Variables
CARGO := cargo

# Default target
.PHONY: all
all: build

# Build everything (TypeScript compilation now integrated in build.rs)
.PHONY: build
build:
	$(CARGO) build --workspace --verbose

# Manual TypeScript compilation (optional - normally handled by build.rs)
.PHONY: build-ts
build-ts: clean-ts
	npm run build

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
	rm -rf server/src/includes/*.js server/src/includes/core server/src/includes/components
	rm -rf tests/node_modules tests/coverage
