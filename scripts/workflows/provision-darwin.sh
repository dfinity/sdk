#!/bin/bash

set -ex

export

# Enter temporary directory.
pushd /tmp

brew install coreutils moreutils

# Install Bats.
if [ "$(uname -r)" = "19.6.0" ]; then
    brew unlink bats
fi
brew install bats-core

# Install Bats support.
version=0.3.0
curl --location --output bats-support.tar.gz https://github.com/ztombol/bats-support/archive/v$version.tar.gz
mkdir /usr/local/lib/bats-support
tar --directory /usr/local/lib/bats-support --extract --file bats-support.tar.gz --strip-components 1
rm bats-support.tar.gz

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

# Set environment variables.
BATS_SUPPORT="/usr/local/lib/bats-support"
echo "BATSLIB=${BATS_SUPPORT}" >> "$GITHUB_ENV"

# Exit temporary directory.
popd

if [ "$E2E_TEST" = "tests-icx-asset/icx-asset.bash" ]; then
    cargo build -p icx-asset
    ICX_ASSET="$(pwd)/target/debug/icx-asset"
    echo "ICX_ASSET=$ICX_ASSET" >> "$GITHUB_ENV"
fi
