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

@test "ledger fabricate-cycles works with default amount" {
    install_asset greet
    dfx_start
    dfx deploy
    # default amount is 10 trillion cycles, which results in an amount like 13_899_071_239_420
    assert_command dfx ledger fabricate-cycles --canister "$(dfx canister id hello)"
    # bash does not accept \d, use [0-9] instead
    assert_match 'updated balance: [0-9]{2}(_[0-9]{3}){4} cycles'
    assert_command dfx ledger fabricate-cycles --all
    assert_match 'updated balance: [0-9]{2}(_[0-9]{3}){4} cycles'
}

@test "ledger fabricate-cycles works with specific amount" {
    install_asset greet
    dfx_start
    dfx deploy
    # adding 100 trillion cycles, which results in an amount like 103_899_071_239_420
    assert_command dfx ledger fabricate-cycles --canister "$(dfx canister id hello)" --cycles 100000000000000
    assert_match 'updated balance: [0-9]{3}(_[0-9]{3}){4} cycles'
    assert_command dfx ledger fabricate-cycles --canister hello --t 100
    assert_match 'updated balance: [0-9]{3}(_[0-9]{3}){4} cycles'
}

@test "ledger fabricate-cycles fails on real IC" {
    install_asset greet
    assert_command_fail dfx ledger --network ic fabricate-cycles --all
    assert_match "Cannot run this on the real IC."
}

@test "ledger fabricate-cycles fails with wrong option combinations" {
    install_asset greet
    assert_command_fail dfx ledger --network ic fabricate-cycles --all --cycles 1 --icp 1
    assert_command_fail dfx ledger --network ic fabricate-cycles --all --icp 1 --t 1
    assert_command_fail dfx ledger --network ic fabricate-cycles --all --t 1 --cycles 1
    assert_command_fail dfx ledger --network ic fabricate-cycles --all --e8s 1 --amount 1
    assert_command_fail dfx ledger --network ic fabricate-cycles --all --amount 1 --cycles 1
}
