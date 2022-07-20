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

@test "schema in docs is current" {
    assert_command dfx schema --outfile out.json
    run diff "out.json" "${BATS_TEST_DIRNAME}/../../docs/dfx-json-schema.json"
    if [ "$output" != "" ]; then
        echo "docs/dfx-json-schema.json is not up to date. Run 'dfx-wip schema --outfile docs/dfx-json-schema.json' to update it." && exit 1
    fi;
}
