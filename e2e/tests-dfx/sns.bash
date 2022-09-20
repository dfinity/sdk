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
    assert_match "Error encountered when generating the SnsInitPayload: Couldn't open initial parameters file"
}

@test "sns deploy succeeds" {
    dfx_new
    install_shared_asset subnet_type/shared_network_settings/system
    dfx start --clean --background --host 127.0.0.1:8080
    sleep 1
    dfx nns install
    # There are no entries for "local" upstream yet, so we need a network mapping.
    dfx nns import --network-mapping local=mainnet
    # This canister ID is not included upstream .. yet.
    jq '.canisters["nns-sns-wasm"].remote.id.local="qaa6y-5yaaa-aaaaa-aaafa-cai"' dfx.json | sponge dfx.json
    ls candid
    cat dfx.json
    dfx nns import --network-mapping local
    ls candid
    cat dfx.json
    install_asset sns/valid
    dfx sns config validate
    dfx sns deploy
}
