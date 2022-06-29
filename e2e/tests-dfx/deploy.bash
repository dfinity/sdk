#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new hello
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "deploy --upgrade-unchanged upgrades even if the .wasm did not change" {
    dfx_start
    assert_command dfx deploy

    assert_command dfx deploy
    assert_match "Module hash.*is already installed"

    assert_command dfx deploy --upgrade-unchanged
    assert_not_match "Module hash.*is already installed"
}

@test "deploy without arguments sets wallet and self as the controllers" {
    dfx_start
    WALLET=$(dfx identity get-wallet)
    PRINCIPAL=$(dfx identity get-principal)
    assert_command dfx deploy hello
    assert_command dfx canister info hello
    assert_match "Controllers: ($WALLET $PRINCIPAL|$PRINCIPAL $WALLET)"
}

@test "deploy --no-wallet sets only self as the controller" {
    dfx_start
    WALLET=$(dfx identity get-wallet)
    PRINCIPAL=$(dfx identity get-principal)
    assert_command dfx deploy hello --no-wallet
    assert_command dfx canister info hello
    assert_not_match "Controllers: ($WALLET $PRINCIPAL|$PRINCIPAL $WALLET)"
    assert_match "Controllers: $PRINCIPAL"
}

@test "deploy from a subdirectory" {
    dfx_new hello
    dfx_start
    install_asset greet

    (
        cd src
        assert_command dfx deploy
        assert_match "Installing code for"
    )

    assert_command dfx canister call hello greet '("Banzai")'
    assert_eq '("Hello, Banzai!")'

    assert_command dfx deploy
    assert_not_match "Installing code for"
    assert_match "is already installed"
}
