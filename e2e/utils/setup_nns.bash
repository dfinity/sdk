#! /bin/bash

# The commit hash was get from ic release note
IC_COMMIT="3b5d893c0857c47715fc339112e5dd1dbfff77a8"

if test -z "${NNS_ARTIFACTS}"
then
  NNS_ARTIFACTS=$(mktemp -d -t dfx-e2e-nns-artifacts-XXXX) # Linux mktemp requires Xs in suffix
  export NNS_ARTIFACTS
else
  echo "NNS_ARTIFACTS is ${NNS_ARTIFACTS}."
fi

get_binary() {
  local FILENAME
  FILENAME="$1"
  if test -e "$NNS_ARTIFACTS/$FILENAME" && test -n "${NO_CLOBBER:-}"; then
    return
  fi
  local TMP_FILE
  TMP_FILE="$(mktemp)"
  local OS
  OS="$(uname)"
  case "$OS" in
  Darwin)
    curl -s "https://download.dfinity.systems/ic/${IC_COMMIT}/nix-release/x86_64-darwin/${FILENAME}.gz" | gunzip >"$TMP_FILE"
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
  install -m 755 "$TMP_FILE" "$NNS_ARTIFACTS/$FILENAME"
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
get_wasm ledger-canister_notify-method.wasm
get_wasm root-canister.wasm
get_wasm cycles-minting-canister.wasm
get_wasm lifeline.wasm
get_wasm genesis-token-canister.wasm
get_wasm identity-canister.wasm
get_wasm nns-ui-canister.wasm
