#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new
}

teardown() {
    dfx_stop

    standard_teardown
}


@test "can read canister metadata from replica" {
    dfx_new hello
    dfx_start

    assert_command dfx deploy

    dfx canister metadata hello_backend candid:service >metadata.txt
    assert_command diff .dfx/local/canisters/hello_backend/hello_backend.did ./metadata.txt
}

@test "asset canister provides candid:service metadata" {
    dfx_new hello
    dfx_start

    assert_command dfx deploy
    REPO_ROOT=${BATS_TEST_DIRNAME}/../../

    dfx canister metadata hello_frontend candid:service >candid_service_metadata.txt
    assert_command diff "$REPO_ROOT/src/distributed/assetstorage.did" ./candid_service_metadata.txt
}
