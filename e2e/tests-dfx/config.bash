#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new
}

teardown() {
    standard_teardown
}

@test "dfx config -- read/write" {
    assert_command_fail dfx config defaults/build/output

    assert_command dfx config networks.local.bind "192.168.0.1:8000"
    assert_eq ""

    assert_command dfx config networks.local.bind
    assert_eq '"192.168.0.1:8000"'

    assert_command_fail dfx config non_existent

    # We don't allow to change values that are non existent.
    assert_command_fail dfx config non_existent 123
}
