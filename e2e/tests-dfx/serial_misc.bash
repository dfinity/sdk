#!/usr/bin/env bats

# This file is for tests that must run in serial but are semantically part of other test files which could otherwise run in parallel.
export BATS_NO_PARALLELIZE_WITHIN_FILE=true

setup() {
    standard_setup

    dfx_new
}

teardown() {
    dfx_stop
    
    standard_teardown
}

@test "dfx-started processes can be killed with dfx killall" {
    dfx_start
    dfx killall
    assert_command_fail pgrep dfx replica pocket-ic
}
