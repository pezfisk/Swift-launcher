#!/usr/bin/env bash
cargo component build --release

BINARY_NAME=$(cargo pkgid | cut -d'#' -f2 | cut -d'@' -f1)

mv ./target/wasm32-wasip1/release/$BINARY_NAME.wasm ~/.config/swift/plugins/$BINARY_NAME.wasm
rm ~/.config/swift/plugins/$BINARY_NAME.cwasm
