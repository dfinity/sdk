#!/usr/bin/env bash

set -e

SCRIPT_PARENT_DIR="$( cd -- "$(dirname -- "$( dirname -- "${BASH_SOURCE[0]}" )" )" &> /dev/null && pwd )"

(
    cd "$SCRIPT_PARENT_DIR"

    nix-shell --command 'ln -sfn $DFX_ASSETS .dfx-assets'

    echo "Export this environment variable:"
    echo "  $ export DFX_ASSETS=\"$(pwd)/.dfx-assets\""
)
