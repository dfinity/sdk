#!/usr/bin/env bats

load ../utils/_

setup() {
    cd "$(mktemp -d -t dfx-e2e-XXXXXXXX)" || exit
    dfx_new hello
}

teardown() {
    dfx_stop
}

@test "get certified-info" {
    dfx_start
    dfx canister create hello
    assert_command dfx canister info "$(dfx canister id hello)"
    assert_match "Controller: $(dfx identity get-wallet) Module hash: None"

    dfx build hello
    RESULT="0x$(shasum -a 256 .dfx/local/canisters/hello/hello.wasm)"
    # shellcheck disable=SC2034
    HASH=$(echo "${RESULT}" | cut -d' ' -f 1)

    dfx canister install hello    
    assert_command dfx canister info "$(dfx canister id hello)"
    assert_match "Controller: $(dfx identity get-wallet) Module hash: $(HASH)"
}
