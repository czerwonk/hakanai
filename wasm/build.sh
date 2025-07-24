#!/usr/bin/env bash
set -e

# Build the WASM module
wasm-pack build --target web --out-dir pkg --release

# Copy the generated files to the server's includes directory
mkdir -p ../server/src/includes
cp pkg/hakanai_wasm_bg.wasm ../server/src/includes/
cp pkg/hakanai_wasm.js ../server/src/includes/

echo "WASM module built and copied to server/src/includes/wasm/"
