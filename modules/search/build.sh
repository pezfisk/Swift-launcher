#!/usr/bin/env bash

cargo component build --release

BINARY_NAME='web_search'

cp ./target/wasm32-wasip1/release/$BINARY_NAME.wasm ~/.config/swift/plugins/$BINARY_NAME.wasm
rm ~/.config/swift/plugins/$BINARY_NAME.cwasm

./../../target/release/Swift-launcher
