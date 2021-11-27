# Run all test configurations

#!/bin/bash

# Error out on the first `cargo test` error
trap 'exit 1' ERR

cargo test
cargo test --no-default-features --features futures-lock,std
cargo test --no-default-features --features tokio-lock,std
cargo test --no-default-features --features async-std-lock,std
cargo test --no-default-features --features futures-lock
cargo test --no-default-features --features tokio-lock
cargo test --no-default-features --features async-std-lock
