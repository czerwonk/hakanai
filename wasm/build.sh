#!/usr/bin/env bash
set -e

# Build the WASM module
wasm-pack build --target web --out-dir pkg --release

# Copy the generated files to the server's includes directory
mkdir -p ../server/includes
cp pkg/hakanai_wasm_bg.wasm ../server/includes/
cp pkg/hakanai_wasm.js ../server/includes/

echo "WASM module built and copied to server/includes/wasm/"
