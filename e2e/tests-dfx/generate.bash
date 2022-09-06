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

@test "dfx generate creates files" {
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install --all

    dfx --version
    dfx generate

    assert_command ls src/declarations/hello_backend
    assert_match "hello_backend.did"
    assert_match "hello_backend.did.js"
    assert_match "hello_backend.did.d.ts"
    assert_match "index.js"
    assert_match "index.d.ts"
}
