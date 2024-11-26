#!/bin/bash

# Exit on error
set -e

echo "Building WebAssembly module..."

# Navigate to the datalog_wasm directory
cd "$(dirname "$0")/datalog_wasm"

# Build the wasm package
wasm-pack build --target web

# Create dist directory if it doesn't exist
mkdir -p ../dist

# Copy the built files to dist
cp pkg/*.js ../dist/
cp pkg/*.wasm ../dist/

echo "Build complete! Files have been copied to dist/"

cd ../dist

python3 -m http.server 
