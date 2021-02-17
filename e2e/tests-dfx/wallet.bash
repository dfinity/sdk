#!/usr/bin/env bats

load ./utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd "$(mktemp -d -t dfx-e2e-XXXXXXXX)" || exit

    # Each test gets its own home directory in order to have its own identities.
    x=$(pwd)/home-for-test
    mkdir "$x"
    export HOME="$x"

    dfx_new
}

teardown() {
    dfx_stop
    x=$(pwd)/home-for-test
    rm -rf "$x"
}

@test "deploy wallet" {
    [ ! "$USE_IC_REF" ] && skip "Skip until updating to Replica with ic_api_version > 0.14.0"

    dfx_start
    webserver_port=$(cat .dfx/webserver-port)
    # shellcheck disable=SC2094
    cat <<<"$(jq '.networks.actuallylocal.providers=["http://127.0.0.1:'"$webserver_port"'"]' dfx.json)" >dfx.json

    # get a Canister IDs to install the wasm onto
    dfx canister --network actuallylocal create dummy_canister1
    ID=$(dfx canister --network actuallylocal id dummy_canister1)
    dfx canister --network actuallylocal create dummy_canister2
    ID_TWO=$(dfx canister --network actuallylocal id dummy_canister2)

    # set controller to user
    dfx canister --network actuallylocal set-controller dummy_canister1 $(dfx identity get-principal)
    dfx canister --network actuallylocal set-controller dummy_canister2 $(dfx identity get-principal)

    # We're testing on a local network so the create command actually creates a wallet
    # Delete this file to force associate wallet created by deploy-wallet to identity
    rm "$HOME"/.config/dfx/identity/default/wallets.json

    assert_command dfx identity --network actuallylocal deploy-wallet "${ID}"
    GET_WALLET_RES=$(dfx identity --network actuallylocal get-wallet)
    assert_eq "$ID" "$GET_WALLET_RES"

    assert_command dfx identity --network actuallylocal deploy-wallet "${ID_TWO}"
    assert_match "The wallet canister \"${ID}\"\ already exists for user \"default\" on \"actuallylocal\" network."
}
