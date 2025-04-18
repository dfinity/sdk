#!/bin/bash

set -e

cargo build --package=plugin --target=wasm32-wasip2 --release
cargo test --package=host
cargo run --package=host -- identity new aaaa
