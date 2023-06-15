#!/bin/bash

set -ex

export

# Enter temporary directory.
pushd /tmp

# Install Bats + moreutils.
brew install coreutils moreutils

# Install Bats.
if [ "$(uname -r)" = "19.6.0" ]; then
    brew unlink bats
fi
brew install bats-core

# Modifications needed for some tests
if [ "$E2E_TEST" = "tests-dfx/bitcoin.bash" ]; then
     brew install bitcoin
fi
if [ "$E2E_TEST" = "tests-dfx/build_rust.bash" ]; then
    cargo uninstall cargo-audit
fi
if [ "$E2E_TEST" = "tests-dfx/certificate.bash" ]; then
     brew install mitmproxy
fi
if [ "$E2E_TEST" = "tests-dfx/deps.bash" ]; then
     cargo install ic-wasm
fi

# Exit temporary directory.
popd

if [ "$E2E_TEST" = "tests-icx-asset/icx-asset.bash" ]; then
    cargo build -p icx-asset
    ICX_ASSET="$(pwd)/target/debug/icx-asset"
    echo "ICX_ASSET=$ICX_ASSET" >> "$GITHUB_ENV"
fi
