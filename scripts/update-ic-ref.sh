#!/usr/bin/env bash

set -e

#   $1 not set   ||   not running in repo root
if [ -z ${1+x} ] || [ ! -f ./scripts/write-dfx-asset-sources.sh ]; then
    echo "Usage: run ./scripts/update-ic-ref.sh <first 8 digits of commit SHA> in repo root"
    exit 1
fi

VERSION="0.0.1-$1"
echo "Updating sources to version ${VERSION}"
niv update ic-ref-x86_64-linux -a version=$VERSION
niv update ic-ref-x86_64-darwin -a version=$VERSION

echo "Writing asset sources"
./scripts/write-dfx-asset-sources.sh

echo "Done. Don't forget to update CHANGELOG.md"
