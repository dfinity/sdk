#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)

    dfx_new
}

@test "dfx config -- read/write" {
    assert_command_fail dfx config defaults/build/output

    assert_command dfx config networks.tungsten.providers '[ "http://127.0.0.1:8000" ]'
    assert_eq ""

    assert_command dfx config networks.tungsten.providers
    assert_eq '[ "http://127.0.0.1:8000" ]'

    assert_command_fail dfx config non_existent

    # We don't allow to change values that are non existent.
    assert_command_fail dfx config non_existent 123
}
