#!/bin/bash

set -ex

export

# Enter temporary directory.
pushd /tmp

# Install Bats.
sudo apt-get install --yes bats

# Install Bats support.
version=0.3.0
wget https://github.com/ztombol/bats-support/archive/v$version.tar.gz
sudo mkdir /usr/local/lib/bats-support
sudo tar --directory /usr/local/lib/bats-support --extract --file v$version.tar.gz --strip-components 1
rm v$version.tar.gz

# Packages needed for some tests
if [ "$E2E_TEST" = "tests-dfx/certificate.bash" ]; then
    sudo apt-get install --yes mitmproxy
fi
if [ "$E2E_TEST" = "tests-dfx/bitcoin.bash" ]; then
    BITCOIN_CORE_VERSION=22.0
    (
        cd "$(mktemp -d)"
        BTC_FILENAME="bitcoin-core-$BITCOIN_CORE_VERSION/bitcoin-$BITCOIN_CORE_VERSION-x86_64-linux-gnu.tar.gz"
        wget "https://bitcoin.org/bin/$BTC_FILENAME"
        tar xzf "$BTC_FILENAME"
        cd "bitcoin-$BITCOIN_CORE_VERSION/bin"
        sudo install -m 0755 -o root -g root -t /usr/local/bin *
    )
fi

# Set environment variables.
BATS_SUPPORT="/usr/local/lib/bats-support"
echo "BATSLIB=${BATS_SUPPORT}" >> "$GITHUB_ENV"
echo "$HOME/bin" >> "$GITHUB_PATH"

# Exit temporary directory.
popd
