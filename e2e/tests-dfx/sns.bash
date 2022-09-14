#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

}

teardown() {
    dfx_stop

    standard_teardown
}

@test "sns config create and validate fail outside of a project" {
    assert_command_fail dfx sns config create
    assert_match 'Cannot find dfx configuration file in the current working directory'

    assert_command_fail dfx sns config validate
    assert_match 'Cannot find dfx configuration file in the current working directory'
}

@test "sns config create creates a default configuration" {
    dfx_new
    assert_command dfx sns config create
    assert_match "Created SNS configuration at: .*/sns.yml"
    : "Check that the file exists..."
    test -e sns.yml
}

@test "sns config validate approves a valid configuration" {
    dfx_new
    assert_command_fail dfx sns config validate
    # todo
    assert_match 'not yet implemented'
}
