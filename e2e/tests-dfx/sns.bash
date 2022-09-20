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
    assert_command dfx sns config create
    assert_match "Created SNS configuration at: .*/sns.yml"
    : "Check that the file exists..."
    test -e sns.yml
}

@test "sns config validate approves a valid configuration" {
    dfx_new
    install_asset sns/valid
    assert_command dfx sns config validate
    assert_match 'SNS config file is valid'
}

@test "sns config validate identifies a missing key" {
    dfx_new
    install_asset sns/valid
    grep -v token_name "${SNS_CONFIG_FILE_NAME}" | sponge "$SNS_CONFIG_FILE_NAME"
    assert_command_fail dfx sns config validate
    assert_match token.name
}

@test "sns deploy exists" {
    dfx sns deploy --help
}

@test "sns deploy fails without config file" {
    dfx_new
    rm -f sns.yml # Is not expected to be present anyway
    assert_command_fail dfx sns deploy
    assert_match 
}

@test "sns deploy succeeds" {
    dfx_new
    install_shared_asset subnet_type/shared_network_settings/system
    dfx start --clean --background --host 127.0.0.1:8080
    sleep 1
    dfx nns install
    # TODO: The IC commit currently used by the sdk doesn't have all the canister IDs yet.
    #       When it does, remove this DFX_IC_SRC override.
    export DFX_IC_SRC="https://raw.githubusercontent.com/dfinity/ic/master"
    dfx nns import --network-mapping local
    ls candid
    cat dfx.json
    install_asset sns/valid
    dfx sns deploy
}
