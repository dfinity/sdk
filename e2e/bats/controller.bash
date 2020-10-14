#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)

    # Each test gets its own home directory in order to have its own identities.
    mkdir $(pwd)/home-for-test
    export HOME=$(pwd)/home-for-test

    dfx_new hello
}

teardown() {
    dfx_stop
    rm -rf $(pwd)/home-for-test
}

@test "set controller" {
    # Create two identities and get their Principals
    assert_command dfx identity new alice
    assert_command dfx identity new bob
    ALICE_PRINCIPAL=$(dfx --identity alice identity get-principal)
    BOB_PRINCIPAL=$(dfx --identity bob identity get-principal)
    ALICE_WALLET=$(dfx --identity alice identity get-wallet)
    BOB_WALLET=$(dfx --identity bob identity get-wallet)

    assert_command dfx identity use alice

    dfx_start
    dfx canister create hello
    dfx build hello
    dfx canister install hello
    ID=$(dfx canister id hello)

    # Set controller using canister name and identity name
    assert_command dfx canister set-controller hello bob
    assert_match "Set \"bob\" as controller of \"hello\"."

    # Juana is controller, Jose cannot reinstall
    assert_command_fail dfx canister install hello -m reinstall
    if [ "$USE_IC_REF" ]
    then
        assert_match "${ALICE_PRINCIPAL} is not authorized to manage canister ${ID}"
    else
        assert_match "Only the controller of canister ${ID} can control it."
    fi

    # Juana can reinstall
    assert_command dfx --identity bob canister install hello -m reinstall

    assert_command dfx identity use bob
    # Set controller using canister id and principal
    assert_command dfx canister set-controller ${ID} ${ALICE_PRINCIPAL}
    assert_match "Set \"${ALICE_PRINCIPAL}\" as controller of \"${ID}\"."
    assert_command_fail dfx canister install hello -m reinstall

    # Set controller using combination of name/id and identity/principal
    assert_command dfx --identity alice canister set-controller hello ${BOB_PRINCIPAL}
    assert_match "Set \"${BOB_PRINCIPAL}\" as controller of \"hello\"."

    assert_command dfx --identity bob canister set-controller ${ID} alice
    assert_match "Set \"alice\" as controller of \"${ID}\"."

    # Set controller using invalid principal/identity fails
    assert_command_fail dfx --identity alice canister set-controller hello bob
    assert_match "Identity bob does not exist"

    # Set controller using invalid canister name/id fails
    assert_command_fail dfx --identity alice canister set-controller hello_assets bob
    assert_match "Cannot find canister id. Please issue 'dfx canister create hello_assets'."
}
