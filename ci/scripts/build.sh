#!/usr/bin/env bash

set -euxo pipefail

VERSION=${REF#"refs/tags/"}

echo "Packaging $VERSION for $TARGET..."

echo "Installing rust toolchain for $TARGET..."
rustup target add $TARGET

echo "Building shared..."
cd gandiva_rust_udf_shared  
cargo build --workspace --lib --target $TARGET --release

echo "Building macro..."
cd ../gandiva_rust_udf_macro
cargo build --workspace --lib --target $TARGET --release

