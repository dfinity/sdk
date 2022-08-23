#!/usr/bin/env bash

set -e

#   $1 not set   ||   not running in repo root
if [ -z ${1+x} ] || [ ! -f ./scripts/write-dfx-asset-sources.sh ]; then
    echo "Usage: run ./scripts/update-replica.sh <SHA-to-update-to> in repo root"
    exit 1
fi

SHA=$1
echo "Updating sources to rev ${SHA}"
niv update ic-admin-x86_64-darwin -a rev=$SHA
niv update ic-admin-x86_64-linux -a rev=$SHA
niv update ic-btc-adapter-x86_64-darwin -a rev=$SHA
niv update ic-btc-adapter-x86_64-linux -a rev=$SHA
niv update ic-canister-http-adapter-x86_64-darwin -a rev=$SHA
niv update ic-canister-http-adapter-x86_64-linux -a rev=$SHA
niv update ic-nns-init-x86_64-darwin -a rev=$SHA
niv update ic-nns-init-x86_64-linux -a rev=$SHA
niv update ic-starter-x86_64-darwin -a rev=$SHA
niv update ic-starter-x86_64-linux -a rev=$SHA
niv update replica-x86_64-darwin -a rev=$SHA
niv update replica-x86_64-linux -a rev=$SHA
niv update canister-sandbox-x86_64-darwin -a rev=$SHA
niv update canister-sandbox-x86_64-linux -a rev=$SHA
niv update sandbox-launcher-x86_64-darwin -a rev=$SHA
niv update sandbox-launcher-x86_64-linux -a rev=$SHA
niv update sns-x86_64-darwin -a rev=$SHA
niv update sns-x86_64-linux -a rev=$SHA

echo "Writing asset sources"
./scripts/write-dfx-asset-sources.sh

for arg in "$@"; do
    if [ "$arg" = '--update-nns' ]; then
        ./scripts/update-nns.bash "$SHA"
    fi
done

echo "Done. Don't forget to update CHANGELOG.adoc"
