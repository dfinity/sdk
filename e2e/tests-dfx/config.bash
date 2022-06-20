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


    assert_command dfx config canisters.e2e_project.type
    assert_eq '"motoko"'

    assert_command dfx config canisters.e2e_project.type "rust"
    assert_eq ""

    assert_command dfx config canisters.e2e_project.type
    assert_eq '"rust"'

    assert_command_fail dfx config non_existent

    # We don't allow to change values that are non existent.
    assert_command_fail dfx config non_existent 123
}
