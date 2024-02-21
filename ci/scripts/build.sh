#!/usr/bin/env bash

set -euxo pipefail

echo "Installing rust toolchain for $TARGET..."
rustup target add $TARGET

echo "Building shared..."
cd gandiva_rust_udf_shared  
cargo build --workspace --lib --target $TARGET --release

echo "Building macro..."
cd ../gandiva_rust_udf_macro
cargo build --workspace --lib --target $TARGET --release

