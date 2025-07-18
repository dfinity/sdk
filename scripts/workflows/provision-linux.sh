#!/bin/bash

set -ex

export

# Enter temporary directory.
pushd /tmp

# Install Bats + moreutils + parallel.
sudo apt-get install --yes bats parallel moreutils

# Modifications needed for some tests
if [ "$E2E_TEST" = "tests-dfx/bitcoin.bash" ]; then
    BITCOIN_CORE_VERSION=22.0

    # Check architecture and set filename and sha
    ARCH=$(uname -m)
    if [ "$ARCH" = "x86_64" ]; then
        BITCOIN_CORE_FILENAME="bitcoin-$BITCOIN_CORE_VERSION-x86_64-linux-gnu.tar.gz"
        BITCOIN_CORE_TARBALL_SHA="59ebd25dd82a51638b7a6bb914586201e67db67b919b2a1ff08925a7936d1b16"
    elif [ "$ARCH" = "aarch64" ]; then
        BITCOIN_CORE_FILENAME="bitcoin-$BITCOIN_CORE_VERSION-aarch64-linux-gnu.tar.gz"
        BITCOIN_CORE_TARBALL_SHA="ac718fed08570a81b3587587872ad85a25173afa5f9fbbd0c03ba4d1714cfa3e"
    else
        echo "Unsupported architecture: $ARCH"
        exit 1
    fi

    (
        cd "$(mktemp -d)"
        wget "https://bitcoin.org/bin/bitcoin-core-$BITCOIN_CORE_VERSION/$BITCOIN_CORE_FILENAME"
        echo "$BITCOIN_CORE_TARBALL_SHA  $BITCOIN_CORE_FILENAME" | shasum -c
        tar xzf "$BITCOIN_CORE_FILENAME"
        cd "bitcoin-$BITCOIN_CORE_VERSION/bin"
        sudo install -m 0755 -o root -g root -t /usr/local/bin *
    )
fi
if [ "$E2E_TEST" = "tests-dfx/certificate.bash" ]; then
    wget -O mitmproxy.tar.gz https://snapshots.mitmproxy.org/7.0.4/mitmproxy-7.0.4-linux.tar.gz
    sudo tar --directory /usr/local/bin --extract --file mitmproxy.tar.gz
    echo "mitmproxy version: $(mitmproxy --version)"
fi
if [ "$E2E_TEST" = "tests-dfx/identity_encryption.bash" ] \
    || [ "$E2E_TEST" = "tests-dfx/identity.bash" ] \
    || [ "$E2E_TEST" = "tests-dfx/generate.bash" ] \
    || [ "$E2E_TEST" = "tests-dfx/start.bash" ] \
    || [ "$E2E_TEST" = "tests-dfx/new.bash" ] \
    || [ "$E2E_TEST" = "tests-dfx/call.bash" ] \
    || [ "$E2E_TEST" = "tests-dfx/build.bash" ]
then
    sudo apt-get install --yes expect
fi
if [ "$E2E_TEST" = "tests-dfx/info.bash" ]; then
    sudo apt-get install --yes libarchive-zip-perl
fi

# Set environment variables.
echo "$HOME/bin" >> "$GITHUB_PATH"

# Exit temporary directory.
popd

if [ "$E2E_TEST" = "tests-dfx/build_rust.bash" ]; then
    cargo uninstall cargo-audit
fi

if [ "$E2E_TEST" = "tests-dfx/deps.bash" ]; then
     cargo install cargo-binstall@1.6.9 --locked
     cargo binstall -y ic-wasm --locked
fi

if [ "$E2E_TEST" = "tests-icx-asset/icx-asset.bash" ]; then
    cargo build -p icx-asset
    ICX_ASSET="$(pwd)/target/debug/icx-asset"
    echo "ICX_ASSET=$ICX_ASSET" >> "$GITHUB_ENV"
fi
