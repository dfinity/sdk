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
