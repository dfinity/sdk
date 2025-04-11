#!/bin/bash

cargo build --package=plugin --target=wasm32-wasip2 --release
cargo run --package=host
