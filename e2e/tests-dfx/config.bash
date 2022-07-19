#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new_rust
}

teardown() {
    standard_teardown
}

@test "dfx config -- read/write" {
    assert_command_fail dfx config defaults/build/output


    assert_command dfx config canisters.e2e_project_backend.type
    # shellcheck disable=SC2154
    assert_eq '"rust"' "$stdout"

    assert_command dfx config canisters.e2e_project_backend.type "motoko"
    # shellcheck disable=SC2154
    assert_eq "" "$stdout"

    assert_command dfx config canisters.e2e_project_backend.type
    # shellcheck disable=SC2154
    assert_eq '"motoko"' "$stdout"

    assert_command_fail dfx config non_existent

    # We don't allow to change values that are non existent.
    assert_command_fail dfx config non_existent 123
}
