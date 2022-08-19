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
    assert_command_fail dfx sns config create
    # todo
    assert_match 'not yet implemented'
}

@test "sns config validate approves a valid configuration" {
    dfx_new
    assert_command_fail dfx sns config validate
    # todo
    assert_match 'not yet implemented'
}
