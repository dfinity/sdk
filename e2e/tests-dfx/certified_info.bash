#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new hello
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "get certified-info" {
    dfx_start
    dfx canister create hello_backend
    assert_command dfx canister info "$(dfx canister id hello_backend)"
    WALLET_ID=$(dfx identity get-wallet)
    SELF_ID=$(dfx identity get-principal)
    assert_match \
"Controllers: ($WALLET_ID $SELF_ID|$SELF_ID $WALLET_ID)
Module hash: None"

    dfx build hello_backend
    RESULT="$(openssl dgst -sha256 .dfx/local/canisters/hello_backend/hello_backend.wasm)"
    # shellcheck disable=SC2034
    HASH="0x"
    HASH+=$(echo "${RESULT}" | cut -d' ' -f 2)


    dfx canister install hello_backend  
    assert_command dfx canister info "$(dfx canister id hello_backend)"
    assert_match \
"Controllers: ($WALLET_ID $SELF_ID|$SELF_ID $WALLET_ID)
Module hash: $(HASH)"
}
