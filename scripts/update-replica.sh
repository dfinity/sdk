#!/usr/bin/env bash

set -euo pipefail

#   $1 not set   ||   not running in repo root
if [ -z ${1+x} ] || [ ! "$0" -ef ./scripts/update-replica.sh ]; then
    echo "Usage: run ./scripts/update-replica.sh <SHA-to-update-to> in repo root"
    exit 1
fi

for cmd in curl jq sponge; do
    if ! command -v "$cmd" &>/dev/null; then
        echo "'$cmd' was not found."
        echo "This script requires curl, jq, and moreutils to be installed"
        exit 1
    fi
done

sources="src/dfx/assets/dfx-asset-sources.json"

rev=$1
echo "Updating sources to rev ${rev}"
jq '."replica-rev" = $rev' --arg rev "$rev" "$sources" | sponge "$sources"

echo "Done. Don't forget to update CHANGELOG.md"
