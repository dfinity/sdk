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

@test "canister snapshots" {
    dfx_start
    install_asset counter
    dfx deploy

    assert_command dfx canister call hello_backend inc_read
    assert_contains '(1 : nat)'

    dfx canister stop hello_backend
    assert_command dfx canister snapshot create hello_backend
    assert_match 'Snapshot ID: ([0-9a-f]+)'
    snapshot=${BASH_REMATCH[1]}
    dfx canister start hello_backend

    assert_command dfx canister call hello_backend inc_read
    assert_contains '(2 : nat)'

    dfx canister stop hello_backend
    assert_command dfx canister snapshot load hello_backend "$snapshot"
    dfx canister start hello_backend
    assert_command dfx canister call hello_backend read
    assert_contains '(1 : nat)'

    assert_command dfx canister snapshot list hello_backend
    assert_match "^${snapshot}:"
    assert_command dfx canister snapshot delete hello_backend "$snapshot"
    assert_command dfx canister snapshot list hello_backend
    assert_contains 'No snapshots found in canister hello_backend'

    assert_command_fail dfx canister snapshot create hello_backend
    assert_contains 'Canister hello_backend is running and snapshots should not be taken of running canisters'
    assert_command dfx canister snapshot create hello_backend -f
}