ARG RUST_VERSION
FROM rust:${RUST_VERSION} AS builder
RUN cargo install ic-wasm --version 0.2.0
COPY . /build
WORKDIR /build
RUN export RUSTFLAGS="--remap-path-prefix $CARGO_HOME=/cargo" && \
    cargo build -p ic-frontend-canister --release --target wasm32-unknown-unknown

RUN export BUILD_DIR=target/wasm32-unknown-unknown/release && \
    ic-wasm --output $BUILD_DIR/ic_frontend_canister.wasm $BUILD_DIR/ic_frontend_canister.wasm metadata --file src/canisters/frontend/ic-certified-assets/assets.did --visibility public candid:service && \
    ic-wasm --output $BUILD_DIR/ic_frontend_canister.wasm $BUILD_DIR/ic_frontend_canister.wasm shrink && \
    gzip --best --keep --force --no-name $BUILD_DIR/ic_frontend_canister.wasm

FROM scratch AS scratch
COPY --from=builder /build/target/wasm32-unknown-unknown/release/ic_frontend_canister.wasm.gz /assetstorage.wasm.gz
COPY --from=builder /build/src/canisters/frontend/ic-certified-assets/assets.did /assetstorage.did
