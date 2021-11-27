# Run all test configurations for WASM. Requires `wasm-pack`

#!/bin/bash

# Error out on the first `wasm-pack test --node` error
trap 'exit 1' ERR

# TODO: --firefox --chrome --safari --headless
wasm-pack test --node
wasm-pack test --node --no-default-features --features futures-lock,std
wasm-pack test --node --no-default-features --features tokio-lock,std
wasm-pack test --node --no-default-features --features async-std-lock,std
wasm-pack test --node --no-default-features --features futures-lock
wasm-pack test --node --no-default-features --features tokio-lock
wasm-pack test --node --no-default-features --features async-std-lock
