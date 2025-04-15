#!/bin/bash

set -e

cargo build --package=plugin --target=wasm32-wasip2 --release
cargo run --package=host
cargo test --package=host
