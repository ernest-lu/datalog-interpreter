#!/bin/bash

# Build the WebAssembly module
wasm-pack build --target web

# Create pkg directory if it doesn't exist
mkdir -p pkg

# Copy the generated files to the web directory
cp pkg/datalog_ptr_web_bg.wasm pkg/
cp pkg/datalog_ptr_web.js pkg/

# Start a local server (optional)
echo "To start a local server, run: python3 -m http.server"