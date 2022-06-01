#! /bin/bash

IC_COMMIT="b90edb9897718730f65e92eb4ff6057b1b25f766"

if [[ -z "${DOWNLOAD_DIR}" ]]; then
  export DOWNLOAD_DIR=$(mktemp -d -t dfx-e2e-download)
else
  echo "DOWNLOAD DIR is ${DOWNLOAD_DIR}."
fi

get_binary() {
  local FILENAME
  FILENAME="$1"
  if test -e "$DOWNLOAD_DIR/$FILENAME" && test -n "${NO_CLOBBER:-}"; then
    return
  fi
  local TMP_FILE
  TMP_FILE="$(mktemp)"
  local OS
  OS="$(uname)"
  case "$OS" in
  Darwin)
    echo "fetching from: https://download.dfinity.systems/ic/${IC_COMMIT}/nix-release/x86_64-darwin/${FILENAME}.gz"
    curl "https://download.dfinity.systems/ic/${IC_COMMIT}/nix-release/x86_64-darwin/${FILENAME}.gz" | gunzip >"$TMP_FILE"
    ;;
  Linux)
    curl "https://download.dfinity.systems/ic/${IC_COMMIT}/release/${FILENAME}.gz" | gunzip >"$TMP_FILE"
    ;;
  *)
    printf "ERROR: %s '%s'\n" \
      "Cannot download binary" "$FILENAME" \
      "Unsupported platform:" "$OS" \
      >&2
    exit 1
    ;;
  esac
  install -m 755 "$TMP_FILE" "$DOWNLOAD_DIR/$FILENAME"
  rm "$TMP_FILE"
}

get_wasm() {
  local FILENAME
  FILENAME="$1"
  if test -e "$DOWNLOAD_DIR/$FILENAME" && test -n "${NO_CLOBBER:-}"; then
    return
  fi
  local TMP_FILE
  TMP_FILE="$(mktemp)"
  curl "https://download.dfinity.systems/ic/${IC_COMMIT}/canisters/${FILENAME}.gz" | gunzip >"$TMP_FILE"
  install -m 644 "$TMP_FILE" "$DOWNLOAD_DIR/$FILENAME"
  rm "$TMP_FILE"
}

get_binary ic-nns-init
get_wasm registry-canister.wasm
get_wasm governance-canister.wasm
get_wasm governance-canister_test.wasm
get_wasm ledger-canister_notify-method.wasm
get_wasm root-canister.wasm
get_wasm cycles-minting-canister.wasm
get_wasm lifeline.wasm
get_wasm genesis-token-canister.wasm
get_wasm identity-canister.wasm
get_wasm nns-ui-canister.wasm

export NNS_URL="http://localhost:$(cat .dfx/replica-configuration/replica-1.port)"

"${DOWNLOAD_DIR}/ic-nns-init" \
  --url "$NNS_URL" \
  --initialize-ledger-with-test-accounts 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752 22ca7edac648b814e81d7946e8bacea99280e07c5f51a04ba7a38009d8ad8e89\
  --wasm-dir "$DOWNLOAD_DIR"
