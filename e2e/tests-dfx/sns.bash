#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

}

teardown() {
    dfx_stop

    standard_teardown
}

# The location of the SNS init config.
SNS_CONFIG_FILE_NAME="sns.yml"

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
    cp "${BATS_TEST_DIRNAME}/../assets/sns/valid_sns_init_config.yaml" "$SNS_CONFIG_FILE_NAME"
    cp "${BATS_TEST_DIRNAME}/../assets/sns/logo.svg" .
    assert_command dfx sns config validate
    assert_match 'SNS config file is valid'
}

@test "sns config validate identifies a missing key" {
    dfx_new
    grep -v token_name "${BATS_TEST_DIRNAME}/../assets/sns/valid_sns_init_config.yaml" > "$SNS_CONFIG_FILE_NAME"
    cp "${BATS_TEST_DIRNAME}/../assets/sns/logo.svg" .
    assert_command_fail dfx sns config validate
    assert_match token.name
}
