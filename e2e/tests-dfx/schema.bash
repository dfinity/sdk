#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    dfx_stop
    standard_teardown
}

@test "dfx schema prints valid json" {
    assert_command dfx schema --outfile out.json
    # make sure out.json contains exactly one json object
    assert_command jq type out.json
    assert_eq '"object"'
}

@test "dfx schema still works with broken dfx.json" {
    jq '.broken_key="blahblahblah"' dfx.json | sponge dfx.json
    assert_command dfx schema
}
