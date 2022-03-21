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

echo "What is happening with pyparsing"
apt-cache policy mitmproxy
apt list -a mitmproxy
apt-cache policy python3-pyparsing
apt-cache policy python-pyparsing
python3 --version
python3 -c "
import pyparsing as pp
print(pp.__version__)
"

# Packages needed for some tests
if [ "$E2E_TEST" = "tests-dfx/certificate.bash" ]; then
    sudo apt-get install --yes mitmproxy=7.0.4
    echo "mitmproxy version: $(mitmproxy --version)"
fi
if [ "$E2E_TEST" = "tests-dfx/bitcoin.bash" ]; then
    BITCOIN_CORE_VERSION=22.0
    BITCOIN_CORE_FILENAME="bitcoin-$BITCOIN_CORE_VERSION-x86_64-linux-gnu.tar.gz"
    BITCOIN_CORE_TARBALL_SHA="59ebd25dd82a51638b7a6bb914586201e67db67b919b2a1ff08925a7936d1b16"
    (
        cd "$(mktemp -d)"
        wget "https://bitcoin.org/bin/bitcoin-core-$BITCOIN_CORE_VERSION/$BITCOIN_CORE_FILENAME"
        echo "$BITCOIN_CORE_TARBALL_SHA  $BITCOIN_CORE_FILENAME" | shasum -c
        tar xzf "$BITCOIN_CORE_FILENAME"
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
