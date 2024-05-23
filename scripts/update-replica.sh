#!/usr/bin/env bash

set -e

#   $1 not set   ||   not running in repo root
if [ -z ${1+x} ] || [ ! -f ./scripts/write-dfx-asset-sources.sh ]; then
    echo "Usage: run ./scripts/update-replica.sh <SHA-to-update-to> in repo root"
    exit 1
fi

SHA=$1
echo "Updating sources to rev ${SHA}"
niv update ic-admin-x86_64-darwin -a rev="$SHA"
niv update ic-admin-x86_64-linux -a rev="$SHA"
niv update ic-btc-adapter-x86_64-darwin -a rev="$SHA"
niv update ic-btc-adapter-x86_64-linux -a rev="$SHA"
niv update ic-https-outcalls-adapter-x86_64-darwin -a rev="$SHA"
niv update ic-https-outcalls-adapter-x86_64-linux -a rev="$SHA"
niv update ic-nns-init-x86_64-darwin -a rev="$SHA"
niv update ic-nns-init-x86_64-linux -a rev="$SHA"
niv update ic-starter-x86_64-darwin -a rev="$SHA"
niv update ic-starter-x86_64-linux -a rev="$SHA"

# icx-proxy is pinned at f2cd225b621674b19f3d601ebb42555d0c6f614d
# niv update icx-proxy-x86_64-darwin -a rev="$SHA"
# niv update icx-proxy-x86_64-linux -a rev="$SHA"

niv update replica-x86_64-darwin -a rev="$SHA"
niv update replica-x86_64-linux -a rev="$SHA"
niv update canister_sandbox-x86_64-darwin -a rev="$SHA"
niv update canister_sandbox-x86_64-linux -a rev="$SHA"
niv update sandbox_launcher-x86_64-darwin -a rev="$SHA"
niv update sandbox_launcher-x86_64-linux -a rev="$SHA"
niv update sns-x86_64-darwin -a rev="$SHA"
niv update sns-x86_64-linux -a rev="$SHA"

echo "Writing asset sources"
./scripts/write-dfx-asset-sources.sh

echo "Done. Don't forget to update CHANGELOG.md"
