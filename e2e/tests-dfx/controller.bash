#!/usr/bin/env bats

load ./utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd "$(mktemp -d -t dfx-e2e-XXXXXXXX)" || exit

    # Each test gets its own home directory in order to have its own identities.
    x=$(pwd)/home-for-test
    mkdir "$x"
    export HOME="$x"

    dfx_new hello
}

teardown() {
    dfx_stop
    rm -rf "$(pwd)/home-for-test"
}

@test "set controller" {
    [ "$USE_IC_REF" ] && skip "Skip for ic-ref as its ic_api_version > 0.14.0, test with set controller with wallet"
    # Create two identities and get their Principals
    assert_command dfx identity new jose
    assert_command dfx identity new juana
    JOSE_PRINCIPAL=$(dfx --identity jose identity get-principal)
    JUANA_PRINCIPAL=$(dfx --identity juana identity get-principal)

    assert_command dfx identity use jose

    dfx_start
    dfx canister create hello
    dfx build hello
    dfx canister install hello
    ID=$(dfx canister id hello)

    # Set controller using canister name and identity name
    assert_command dfx canister set-controller hello juana
    assert_match "Set \"juana\" as controller of \"hello\"."

    # Juana is controller, Jose cannot reinstall
    assert_command_fail dfx canister install hello -m reinstall

    # Juana can reinstall
    assert_command dfx --identity juana canister install hello -m reinstall

    assert_command dfx identity use juana
    # Set controller using canister id and principal
    assert_command dfx canister set-controller "${ID}" "${JOSE_PRINCIPAL}"
    assert_match "Set \"${JOSE_PRINCIPAL}\" as controller of \"${ID}\"."
    assert_command_fail dfx canister install hello -m reinstall

    # Set controller using combination of name/id and identity/principal
    assert_command dfx --identity jose canister set-controller hello "${JUANA_PRINCIPAL}"
    assert_match "Set \"${JUANA_PRINCIPAL}\" as controller of \"hello\"."

    assert_command dfx --identity juana canister set-controller "${ID}" jose
    assert_match "Set \"jose\" as controller of \"${ID}\"."

    # Set controller using invalid principal/identity fails
    assert_command_fail dfx --identity jose canister set-controller hello bob
    assert_match "Identity bob does not exist"

    # Set controller using invalid canister name/id fails
    assert_command_fail dfx --identity jose canister set-controller hello_assets juana
    assert_match "Cannot find canister id. Please issue 'dfx canister create hello_assets'."
}

@test "set controller with wallet" {
    [ ! "$USE_IC_REF" ] && skip "Skip until updating to Replica with ic_api_version > 0.14.0"
    # Create two identities
    assert_command dfx identity new alice
    assert_command dfx identity new bob

    assert_command dfx identity use alice

    dfx_start
    ALICE_WALLET=$(dfx --identity alice identity get-wallet)
    BOB_WALLET=$(dfx --identity bob identity get-wallet)

    dfx canister create hello
    dfx build hello
    dfx canister install hello
    ID=$(dfx canister id hello)

    # Set controller using canister name and identity name
    assert_command dfx canister set-controller hello bob
    assert_match "Set \"bob\" as controller of \"hello\"."

    # Juana is controller, Jose cannot reinstall
    assert_command_fail dfx canister install hello -m reinstall

    # Juana can reinstall
    assert_command dfx --identity bob canister install hello -m reinstall

    assert_command dfx identity use bob
    # Set controller using canister id and principal
    assert_command dfx canister set-controller "${ID}" "${ALICE_WALLET}"
    assert_match "Set \"${ALICE_WALLET}\" as controller of \"${ID}\"."
    assert_command_fail dfx canister install hello -m reinstall

    # Set controller using combination of name/id and identity/principal
    assert_command dfx --identity alice canister set-controller hello "${BOB_WALLET}"
    assert_match "Set \"${BOB_WALLET}\" as controller of \"hello\"."

    assert_command dfx --identity bob canister set-controller "${ID}" alice
    assert_match "Set \"alice\" as controller of \"${ID}\"."

    # Set controller using invalid principal/identity fails
    assert_command_fail dfx --identity alice canister set-controller hello charlie
    assert_match "Identity charlie does not exist"

    # Set controller using invalid canister name/id fails
    assert_command_fail dfx --identity alice canister set-controller hello_assets bob
    assert_match "Cannot find canister id. Please issue 'dfx canister create hello_assets'."
}
