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

@test "update controller" {
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
    assert_command dfx canister update-settings hello --controller juana
    assert_match "Updated \"juana\" as controller of \"hello\"."

    # Juana is controller, Jose cannot reinstall
    assert_command_fail dfx canister install hello -m reinstall
    if [ "$USE_IC_REF" ]
    then
        assert_match "${JOSE_PRINCIPAL} is not authorized to manage canister ${ID}"
    else
        assert_match "Only the controller of canister ${ID} can control it."
    fi

    # Juana can reinstall
    assert_command dfx --identity juana canister install hello -m reinstall

    assert_command dfx identity use juana
    # Set controller using canister id and principal
    assert_command dfx canister update-settings ${ID} --controller ${JOSE_PRINCIPAL}
    assert_match "Updated \"${JOSE_PRINCIPAL}\" as controller of \"${ID}\"."
    assert_command_fail dfx canister install hello -m reinstall

    # Set controller using combination of name/id and identity/principal
    assert_command dfx --identity jose canister update-settings hello --controller ${JUANA_PRINCIPAL}
    assert_match "Updated \"${JUANA_PRINCIPAL}\" as controller of \"hello\"."

    assert_command dfx --identity juana canister update-settings ${ID} --controller jose
    assert_match "Updated \"jose\" as controller of \"${ID}\"."

    # Set controller using invalid principal/identity fails
    assert_command_fail dfx --identity jose canister update-settings hello --controller bob
    assert_match "Identity bob does not exist"

    # Set controller using invalid canister name/id fails
    assert_command_fail dfx --identity jose canister update-settings hello_assets --controller juana
    assert_match "Cannot find canister id. Please issue 'dfx canister create hello_assets'."
}
