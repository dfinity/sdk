FROM debian:12

RUN \
    apt update && apt install -y \
        build-essential \
        clang \
        cmake \
        curl \
        git \
        libssl-dev \
        mold \
        pkg-config

RUN \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- \
        -y \
        --default-toolchain 1.78.0

RUN \
    curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | sh

RUN \
    $HOME/.cargo/bin/cargo \
        binstall --locked sccache

ENV SCCACHE_DIRECT=true
ENV SCCACHE_LOG=info

ENV RUSTC_WRAPPER=sccache
ENV CARGO_INCREMENTAL=false
