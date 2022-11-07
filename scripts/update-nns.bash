#! /bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."

# Download NNS artifacts

IC_COMMIT=${1:-"$(jq -r '."replica-x86_64-linux".rev' nix/sources.json)"}

NNS_ARTIFACTS=${NNS_ARTIFACTS:-"$PWD/e2e/assets/ledger"}
mkdir -p $NNS_ARTIFACTS
export NNS_ARTIFACTS

get_binary() {
  local FILENAME
  FILENAME="$1"
  if [ -e "$NNS_ARTIFACTS/${FILENAME}_linux" ] && [ -e "$NNS_ARTIFACTS/${FILENAME}_macos" ] && [ -n "${NO_CLOBBER:-}" ]; then
    return
  fi
  local TMP_FILE
  TMP_FILE="$(mktemp)"
  curl -s "https://download.dfinity.systems/ic/${IC_COMMIT}/nix-release/x86_64-darwin/${FILENAME}.gz" | gunzip >"$TMP_FILE"
  install -m 755 "$TMP_FILE" "$NNS_ARTIFACTS/${FILENAME}_macos"
  curl -s "https://download.dfinity.systems/ic/${IC_COMMIT}/release/${FILENAME}.gz" | gunzip >"$TMP_FILE"
  install -m 755 "$TMP_FILE" "$NNS_ARTIFACTS/${FILENAME}_linux"
  
  rm "$TMP_FILE"
}

get_wasm() {
  local FILENAME
  FILENAME="$1"
  if test -e "$NNS_ARTIFACTS/$FILENAME" && test -n "${NO_CLOBBER:-}"; then
    return
  fi
  local TMP_FILE
  TMP_FILE="$(mktemp)"
  curl -s "https://download.dfinity.systems/ic/${IC_COMMIT}/canisters/${FILENAME}.gz" | gunzip >"$TMP_FILE"
  install -m 644 "$TMP_FILE" "$NNS_ARTIFACTS/$FILENAME"
  rm "$TMP_FILE"
}

get_binary ic-nns-init
get_wasm registry-canister.wasm
get_wasm governance-canister.wasm
get_wasm governance-canister_test.wasm
get_wasm ic-ckbtc-minter.wasm
get_wasm ledger-canister_notify-method.wasm
get_wasm root-canister.wasm
get_wasm cycles-minting-canister.wasm
get_wasm lifeline.wasm
get_wasm genesis-token-canister.wasm
get_wasm identity-canister.wasm
get_wasm nns-ui-canister.wasm
get_wasm sns-wasm-canister.wasm
get_wasm ic-icrc1-ledger.wasm

# Patch NNS canisters so the conversion rate can be set in the CMC

pushd "$(dirname "$0")"
DOCKER_BUILDKIT=1 docker build \
  --target scratch \
  -t "ledger" \
  -f nns-canister.Dockerfile \
  --build-arg=IC_COMMIT="$IC_COMMIT" \
  -o "$NNS_ARTIFACTS" .
popd
