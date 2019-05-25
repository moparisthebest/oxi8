#!/usr/bin/env bash
cargo web build --release --target wasm32-unknown-unknown -p oxi8_kiss3d
cargo web build --release --target wasm32-unknown-unknown -p oxi8_quicksilver
cargo build --release

# for dev, in subdir:
# cargo web start --target wasm32-unknown-unknown --open --auto-reload
