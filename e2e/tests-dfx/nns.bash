#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "ic-nns-init binary exists and is executable" {
    dfx cache install

    # it panics, but still shows help
    assert_command_fail "$(dfx cache show)/ic-nns-init" --help
    assert_match "thread 'main' panicked at 'Illegal arguments:"
    assert_match "ic-nns-init \[OPTIONS\]"
    assert_match "-h, --help.*Print help information"
    assert_match '--version.*Print version information'

    # --version fails too
    assert_command_fail "$(dfx cache show)/ic-nns-init" --version
}

@test "ic-admin binary exists and is executable" {
    dfx cache install

    assert_command "$(dfx cache show)/ic-admin" --help
    assert_match "Common command-line options for \`ic-admin\`"
}

@test "sns binary exists and is executable" {
    dfx cache install

    assert_command_fail "$(dfx cache show)/sns" --help
    assert_match "Initialize, deploy and interact with an SNS."
}

@test "dfx nns install command exists" {
    assert_command dfx nns install --help
}

@test "dfx nns install runs" {
    install_asset subnet_type/shared_network_settings/system

    assert_command dfx start --clean --background
    assert_match "subnet_type: System"

    assert_command dfx nns install

    # Checking that the install worked:
    # ... Canisters should exist:
    # ... ... Backend canisters:
    assert_command dfx canister id nns-registry
    assert_command dfx canister id nns-governance
    assert_command dfx canister id nns-ledger
    assert_command dfx canister id nns-root
    assert_command dfx canister id nns-cycles-minting
    assert_command dfx canister id nns-lifeline
    assert_command dfx canister id nns-genesis-token
    assert_command dfx canister id nns-sns-wasm
    # ... ... Frontend canisters:
    assert_command dfx canister id nns-identity
    assert_command dfx canister id nns-ui
    # ... Just to be sure that the existence check does not always pass:
    assert_command_fail dfx canister id i-always-return-true
    # ... Pages should be accessible for the front end canisters:
    BOUNDARY_ORIGIN="localhost:8080"
    canister_url() {
      echo "http://$(dfx canister id "$1").${BOUNDARY_ORIGIN}"
    }
    assert_command curl --fail -sSL "$(canister_url nns-identity)"
    assert_command curl --fail -sSL "$(canister_url nns-ui)"
}
