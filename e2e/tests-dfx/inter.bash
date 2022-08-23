#!/usr/bin/env bash

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    dfx_stop
    stop_dfx_replica
    stop_dfx_bootstrap
    standard_teardown
}

@test "inter-canister calls" {
    dfx_new_rust inter
    install_asset inter
    dfx_start
    dfx deploy

    # calling motoko canister from rust canister
    assert_command dfx canister call inter_rs read
    assert_match '(0 : nat)'
    assert_command dfx canister call inter_rs inc
    assert_command dfx canister call inter_rs read
    assert_match '(1 : nat)'
    assert_command dfx canister call inter_rs write '(5)'
    assert_command dfx canister call inter_rs read
    assert_match '(5 : nat)'

    # calling rust canister from motoko canister
    assert_command dfx canister call inter_mo write '(0)'
    assert_command dfx canister call inter_mo read
    assert_match '(0 : nat)'
    assert_command dfx canister call inter_mo inc
    assert_command dfx canister call inter_mo read
    assert_match '(1 : nat)'
    assert_command dfx canister call inter_mo write '(6)'
    assert_command dfx canister call inter_mo read
    assert_match '(6 : nat)'

    # calling rust canister from rust canister, trough motoko canisters
    assert_command dfx canister call inter2_rs write '(0)'
    assert_command dfx canister call inter2_rs read
    assert_match '(0 : nat)'
    assert_command dfx canister call inter2_rs inc
    assert_command dfx canister call inter2_rs read
    assert_match '(1 : nat)'
    assert_command dfx canister call inter2_rs write '(7)'
    assert_command dfx canister call inter2_rs read
    assert_match '(7 : nat)'

    # calling motoko canister from motoko canister, trough rust canisters
    assert_command dfx canister call inter2_mo write '(0)'
    assert_command dfx canister call inter2_mo read
    assert_match '(0 : nat)'
    assert_command dfx canister call inter2_mo inc
    assert_command dfx canister call inter2_mo read
    assert_match '(1 : nat)'
    assert_command dfx canister call inter2_mo write '(8)'
    assert_command dfx canister call inter2_mo read
    assert_match '(8 : nat)'
}
