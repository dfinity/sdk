#!/bin/bash

set -ex

export

# Enter temporary directory.
pushd /tmp

# Install Homebrew
curl --location --output install-brew.sh "https://raw.githubusercontent.com/Homebrew/install/master/install.sh"
bash install-brew.sh
rm install-brew.sh

# Install Node.
version=14.15.4
curl --location --output node.pkg "https://nodejs.org/dist/v$version/node-v$version.pkg"
sudo installer -pkg node.pkg -store -target /
rm node.pkg

# Install Bats.
brew unlink bats
brew install bats-core

# Install Bats support.
version=0.3.0
curl --location --output bats-support.tar.gz https://github.com/ztombol/bats-support/archive/v$version.tar.gz
mkdir /usr/local/lib/bats-support
tar --directory /usr/local/lib/bats-support --extract --file bats-support.tar.gz --strip-components 1
rm bats-support.tar.gz

# Install DFINITY SDK.
curl --location --output install-dfx.sh "https://sdk.dfinity.org/install.sh"
bash install-dfx.sh < <(yes Y)
rm install-dfx.sh

# Set environment variables.
BATS_SUPPORT="/usr/local/lib/bats-support"
echo "BATS_SUPPORT=${BATS_SUPPORT}" >> "$GITHUB_ENV"

# Exit temporary directory.
popd
