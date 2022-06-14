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
    assert_command dfx config non_existent 123
    assert_command dfx config non_existent  
    assert_eq '"123"'
}
