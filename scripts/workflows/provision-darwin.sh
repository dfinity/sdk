#!/bin/bash

set -ex

export

# Install Bats + moreutils + parallel
brew fetch --retry coreutils moreutils parallel
brew install coreutils moreutils
# moreutils also has a command named parallel, so on homebrew you have to force it
brew unlink moreutils
brew install parallel
brew unlink parallel
brew link moreutils
brew link parallel --overwrite

# Install Bats.
if [ "$(uname -r)" = "19.6.0" ]; then
    brew unlink bats
fi
brew fetch --retry bats-core
brew install bats-core

# Modifications needed for some tests
if [ "$E2E_TEST" = "tests-dfx/bitcoin.bash" ]; then
     brew fetch --retry bitcoin
     brew install bitcoin
fi
if [ "$E2E_TEST" = "tests-dfx/build_rust.bash" ] && command -v cargo-audit &>/dev/null; then
    cargo uninstall cargo-audit
fi
if [ "$E2E_TEST" = "tests-dfx/certificate.bash" ]; then
     HOMEBREW_CURL_OPTS="--http1.1" brew install --cask mitmproxy --no-quarantine
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
