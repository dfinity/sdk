#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
    install_asset ledger

    dfx identity import --disable-encryption alice alice.pem
    dfx identity import --disable-encryption bob bob.pem

    dfx_start

    # The nns has been init with two accounts corresponding with identities above
    # Each has 10000 ICP
    NO_CLOBBER="1" load $BATS_TEST_DIRNAME/../utils/setup_nns.bash
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "ledger balance & transfer" {
    dfx identity use alice
    assert_command dfx ledger account-id
    assert_match 345f723e9e619934daac6ae0f4be13a7b0ba57d6a608e511a00fd0ded5866752

    assert_command dfx ledger balance
    assert_match "10000.00000000 ICP"

    assert_command dfx ledger transfer --amount 100 --memo 1 22ca7edac648b814e81d7946e8bacea99280e07c5f51a04ba7a38009d8ad8e89 # to bob
    assert_match "Transfer sent at BlockHeight:"

    # The sender paid transaction fee which is 0.0001 
    assert_command dfx ledger balance
    assert_match "9899.99990000 ICP"

    dfx identity use bob
    assert_command dfx ledger balance
    assert_match "10100.00000000 ICP"
}