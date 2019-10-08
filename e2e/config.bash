#!/usr/bin/env bats

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
}

@test "dfx config -- read/write" {
    dfx new e2e-project
    cd e2e-project

    run dfx config defaults/build/output
    [[ $status == 0 ]]
    [[ "$output" == "\"canisters/\"" ]]

    run dfx config defaults.build.output
    [[ $status == 0 ]]
    [[ "$output" == "\"canisters/\"" ]]

    run dfx config defaults/build/output "other/"
    [[ $status == 0 ]]

    run dfx config defaults/build/output
    [[ $status == 0 ]]
    [[ "$output" == "\"other/\"" ]]

    run dfx config non_existent
    [[ $status == 255 ]]

    # We don't allow to change values that are non existent.
    run dfx config non_existent 123
    [[ $status != 0 ]]
}
