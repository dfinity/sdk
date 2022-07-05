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
    # shellcheck disable=SC2154
    assert_eq '"motoko"' "$stdout"
    # shellcheck disable=SC2094
    cat <<<"$(jq '.canisters.e2e_project.candid="/dev/null" | .canisters.e2e_project.package="e2e_project"' dfx.json)" >dfx.json
    assert_command dfx config canisters.e2e_project.type "rust"
    # shellcheck disable=SC2154
    assert_eq "" "$stdout"

    assert_command dfx config canisters.e2e_project.type
    # shellcheck disable=SC2154
    assert_eq '"rust"' "$stdout"

    assert_command_fail dfx config non_existent

    # We don't allow to change values that are non existent.
    assert_command_fail dfx config non_existent 123
}
