#!/usr/bin/env bats

load ../utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd "$(mktemp -d -t dfx-e2e-XXXXXXXX)" || exit

    # Each test gets its own home directory in order to have its own identities.
    x=$(pwd)/home-for-test
    mkdir "$x"
    export HOME="$x"
}

teardown() {
    dfx_stop
    x=$(pwd)/home-for-test
    rm -rf "$x"
}

@test "deploy wallet" {
    dfx_new hello
    dfx_start
    setup_actuallylocal_network

    # get Canister IDs to install the wasm onto
    dfx canister --network actuallylocal create dummy_canister1
    ID=$(dfx canister --network actuallylocal id dummy_canister1)
    dfx canister --network actuallylocal create dummy_canister2
    ID_TWO=$(dfx canister --network actuallylocal id dummy_canister2)

    # set controller to user
    dfx canister --network actuallylocal set-controller dummy_canister1 "$(dfx identity get-principal)"
    dfx canister --network actuallylocal set-controller dummy_canister2 "$(dfx identity get-principal)"

    # We're testing on a local network so the create command actually creates a wallet
    # Delete this file to force associate wallet created by deploy-wallet to identity
    rm "$HOME"/.config/dfx/identity/default/wallets.json

    assert_command dfx identity --network actuallylocal deploy-wallet "${ID}"
    GET_WALLET_RES=$(dfx identity --network actuallylocal get-wallet)
    assert_eq "$ID" "$GET_WALLET_RES"

    assert_command dfx identity --network actuallylocal deploy-wallet "${ID_TWO}"
    assert_match "The wallet canister \"${ID}\"\ already exists for user \"default\" on \"actuallylocal\" network."
}

@test "wallet create wallet" {
    dfx_start
    WALLET_ID=$(dfx identity get-wallet)
    CREATE_RES=$(dfx canister call "${WALLET_ID}" wallet_create_wallet "(record { cycles = (2000000000000:nat64); controller = opt principal \"$(dfx identity get-principal)\";})")
    CHILD_ID=$(echo "${CREATE_RES}" | cut -d'"' -f 2)
    assert_command dfx canister call "${CHILD_ID}" wallet_balance '()'
}

@test "bypass wallet call as user" {
    dfx_new
    install_asset identity
    dfx_start
    assert_command dfx canister --no-wallet create --all
    assert_command dfx build
    assert_command dfx canister --no-wallet install --all

    CALL_RES=$(dfx canister --no-wallet call e2e_project fromCall)
    CALLER=$(echo "${CALL_RES}" | cut -d'"' -f 2)
    ID=$(dfx identity get-principal)
    assert_eq "$CALLER" "$ID"

    assert_command dfx canister --no-wallet call e2e_project amInitializer
    assert_eq '(true)'
}

@test "bypass wallet call as user: deploy" {
    dfx_new
    install_asset identity
    dfx_start
    assert_command dfx deploy --no-wallet

    CALL_RES=$(dfx canister --no-wallet call e2e_project fromCall)
    CALLER=$(echo "${CALL_RES}" | cut -d'"' -f 2)
    ID=$(dfx identity get-principal)
    assert_eq "$CALLER" "$ID"

    assert_command dfx canister --no-wallet call e2e_project amInitializer
    assert_eq '(true)'
}
