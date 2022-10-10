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

@test "call --candid <path to candid file>" {
    install_asset call
    cat dfx.json

    dfx_start
    dfx deploy
    assert_command dfx canister call hello_backend make_struct '("A", "B")'
    assert_eq '(record { c = "A"; d = "B" })'

    CANISTER_ID=$(dfx canister id hello_backend)
    rm .dfx/local/canister_ids.json

    # if no candid file known, then no field names
    assert_command dfx canister call "$CANISTER_ID" make_struct '("A", "B")'
    assert_eq '(record { 99 = "A"; 100 = "B" })'

    # if passing the candid file, field names available
    assert_command dfx canister call --candid .dfx/local/canisters/hello_backend/hello_backend.did "$CANISTER_ID" make_struct '("A", "B")'
    assert_eq '(record { c = "A"; d = "B" })'
}

@test "call subcommand accepts canister identifier as canister name" {
    install_asset greet
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install hello_backend
    assert_command dfx canister call "$(dfx canister id hello_backend)" greet '("Names are difficult")'
    assert_match '("Hello, Names are difficult!")'
}

@test "call subcommand accepts argument from a file" {
    install_asset greet
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install hello_backend
    TMP_NAME_FILE="$(mktemp)"
    printf '("Names can be very long")' > "$TMP_NAME_FILE"
    assert_command dfx canister call --argument-file "$TMP_NAME_FILE" hello_backend greet
    assert_match '("Hello, Names can be very long!")'
    rm "$TMP_NAME_FILE"
}

@test "call subcommand accepts argument from stdin" {
    install_asset greet
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install hello_backend
    TMP_NAME_FILE="$(mktemp)"
    printf '("stdin")' > "$TMP_NAME_FILE"
    assert_command dfx canister call --argument-file - hello_backend greet < "$TMP_NAME_FILE"
    assert_match '("Hello, stdin!")'
    rm "$TMP_NAME_FILE"
}

@test "call random value (pattern)" {
    install_asset greet
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install hello_backend
    assert_command dfx canister call hello_backend greet --random '{ value = Some ["\"DFINITY\""] }'
    assert_match '("Hello, DFINITY!")'
}

@test "error on empty arguments when the method requires some" {
    install_asset greet
    dfx_start
    dfx deploy
    assert_command_fail dfx canister call hello_backend greet
}

@test "call random value (empty)" {
    install_asset greet
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install hello_backend
    assert_command dfx canister call hello_backend greet --random ''
    assert_match '("Hello, .*!")'
}

@test "long call" {
    install_asset recurse
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install hello_backend
    assert_command dfx canister call hello_backend recurse 100
}

@test "call with cycles" {
    dfx_start
    dfx deploy
    assert_command_fail dfx canister call hello_backend greet '' --with-cycles 100
    assert_command dfx canister call hello_backend greet '' --with-cycles 100 --wallet "$(dfx identity get-wallet)"
}

@test "call by canister id outside of a project" {
    install_asset greet
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install hello_backend
    ID="$(dfx canister id hello_backend)"
    NETWORK="http://localhost:$(get_webserver_port)"
    (
        cd "$E2E_TEMP_DIR"
        mkdir "not-a-project-dir"
        cd "not-a-project-dir"
        assert_command dfx canister call "$ID" greet '("you")' --network "$NETWORK"
        assert_match '("Hello, you!")'
    )
}
