#!/usr/bin/env bats

# This file is for tests that must run in serial but are semantically part of other test files which could otherwise run in parallel.

load ../utils/_

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
