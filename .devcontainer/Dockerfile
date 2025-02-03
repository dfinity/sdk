FROM debian:12

RUN \
    apt update && apt install -y \
        build-essential \
        ccache \
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

ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=clang
ENV RUSTFLAGS="-C link-arg=-fuse-ld=mold"

ENV CC="ccache clang"
ENV CXX="ccache clang++"
