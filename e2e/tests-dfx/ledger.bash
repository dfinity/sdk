#!/usr/bin/env bats

load ../utils/_

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

setup() {
    standard_setup
    install_asset ledger

    dfx identity import --disable-encryption alice alice.pem
    dfx identity import --disable-encryption bob bob.pem

    dfx_start

    # Setup NNS
    IC_COMMIT="b90edb9897718730f65e92eb4ff6057b1b25f766"

    if [[ -z "${DOWNLOAD_DIR}" ]]; then
      DOWNLOAD_DIR=$(mktemp -d -t dfx-e2e-xxxxxxxx)
      export DOWNLOAD_DIR
    else
      echo "DOWNLOAD DIR is ${DOWNLOAD_DIR}."
    fi

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

    local NNS_URL 
    NNS_URL="http://localhost:$(cat .dfx/replica-configuration/replica-1.port)"

    "${DOWNLOAD_DIR}/ic-nns-init" \
      --url "$NNS_URL" \
      --initialize-ledger-with-test-accounts 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752 22ca7edac648b814e81d7946e8bacea99280e07c5f51a04ba7a38009d8ad8e89\
      --wasm-dir "$DOWNLOAD_DIR"
}

teardown() {
    dfx_stop

    standard_teardown
    rm -rf "$DOWNLOAD_DIR" || rm -rf "$DOWNLOAD_DIR"
}

@test "ledger balance & transfer" {
    dfx identity use alice
    assert_command dfx ledger account-id
    assert_match 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752

    assert_command dfx ledger balance
    assert_match "10000.00000000 ICP"

    assert_command dfx ledger transfer --amount 100 --memo 1 22ca7edac648b814e81d7946e8bacea99280e07c5f51a04ba7a38009d8ad8e89 # to bob
    assert_match "Transfer sent at BlockHeight:"

    # The sender(alice) paid transaction fee which is 0.0001 ICP
    assert_command dfx ledger balance
    assert_match "9899.99990000 ICP"

    dfx identity use bob
    assert_command dfx ledger account-id
    assert_match 22ca7edac648b814e81d7946e8bacea99280e07c5f51a04ba7a38009d8ad8e89

    assert_command dfx ledger balance
    assert_match "10100.00000000 ICP"

    assert_command dfx ledger transfer --icp 100 --e8s 1 --memo 2 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752 # to bob
    assert_match "Transfer sent at BlockHeight:"

    # The sender(bob) paid transaction fee which is 0.0001 ICP
    # 10100 - 100 - 0.0001 - 0.00000001 = 9999.99989999
    assert_command dfx ledger balance
    assert_match "9999.99989999 ICP"
}
