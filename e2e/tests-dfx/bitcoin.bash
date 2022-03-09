#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    bitcoind -regtest -daemonwait
}

teardown() {
    bitcoin-cli -regtest stop

    standard_teardown
}

@test "noop" {
    assert_command bitcoin-cli -regtest createwallet "test"
    ADDRESS="$(bitcoin-cli -regtest getnewaddress)"
    assert_command bitcoin-cli -regtest generatetoaddress 101 "$ADDRESS"
}
