#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)

    dfx_new
}

@test "dfx config -- read/write" {
    assert_command dfx config defaults/build/output
    assert_eq '"canisters/"'

    assert_command dfx config defaults.build.output
    assert_eq '"canisters/"'

    assert_command dfx config defaults/build/output "other/"
    assert_eq ""

    assert_command dfx config defaults/build/output
    assert_eq '"other/"'

    assert_command dfx config --format json
    assert_match '^{ '
    assert_match ': "other/"'

    assert_command_fail dfx config non_existent

    # We don't allow to change values that are non existent.
    assert_command_fail dfx config non_existent 123
}
