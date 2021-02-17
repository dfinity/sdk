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


@test "identity get-principal: the get-principal is the same as sender id" {
    install_asset identity
    dfx_start
    assert_command dfx identity new jose

    PRINCPAL_ID=$(dfx --identity jose identity get-principal)

    dfx --identity jose canister create e2e_project
    dfx --identity jose build e2e_project
    dfx --identity jose canister install e2e_project

    assert_command dfx --identity jose canister call e2e_project amInitializer

    SENDER_ID=$(dfx --identity jose canister call e2e_project fromCall)

    if [ "$PRINCPAL_ID" -ne "$SENDER_ID" ]; then
      echo "IDs did not match: Principal '${PRINCPAL_ID}' != Sender '${SENDER_ID}'..." | fail
    fi
}

@test "calls and query receive the same principal from dfx" {
    install_asset identity
    dfx_start
    dfx canister create --all
    assert_command dfx build
    assert_command dfx canister install --all

    ID_CALL=$(dfx canister call e2e_project fromCall)
    ID_QUERY=$(dfx canister call e2e_project fromQuery)
    if [ "$ID_CALL" -ne "$ID_QUERY" ]; then
      echo "IDs did not match: call '${ID_CALL}' != query '${ID_QUERY}'..." | fail
    fi

    ID=$(dfx canister call e2e_project getCanisterId)
    assert_command dfx canister call e2e_project isMyself "$ID"
    assert_eq '(true)'
    assert_command dfx canister call e2e_project isMyself "$ID_CALL"
    assert_eq '(false)'
}

@test "dfx ping creates the default identity on first run" {
    install_asset identity
    dfx_start
    assert_command dfx ping
    # shellcheck disable=SC2154
    assert_match 'Creating the "default" identity.' "$stderr"
    # shellcheck disable=SC2154
    assert_match "ic_api_version" "$stdout"
}

@test "dfx canister: creates the default identity on first run" {
    install_asset identity
    dfx_start
    assert_command dfx canister create e2e_project
    assert_match 'Creating the "default" identity.' "$stderr"
}

@test "after using a specific identity while creating a canister, that identity is the initializer" {
    [ "$USE_IC_REF" ] && skip "Skip for ic-ref as its ic_api_version > 0.14.0, test with set controller with wallet"
    install_asset identity
    dfx_start
    assert_command dfx identity new alice
    assert_command dfx identity new bob

    dfx --identity alice canister create --all
    assert_command dfx --identity alice build
    assert_command dfx --identity alice canister install --all

    assert_command dfx --identity alice canister call e2e_project amInitializer
    assert_eq '(true)'

    assert_command dfx --identity bob canister call e2e_project amInitializer
    assert_eq '(false)'

    # these all fail (other identities are not initializer; cannot store assets):
    assert_command_fail dfx --identity bob canister call e2e_project_assets store '("B", vec { 88; 87; 86 })'
    assert_command_fail dfx --identity default canister call e2e_project_assets store '("B", vec { 88; 87; 86 })'
    assert_command_fail dfx canister call e2e_project_assets store '("B", vec { 88; 87; 86 })'
    assert_command_fail dfx canister call e2e_project_assets retrieve '("B")'

    # but alice, the initializer, can store assets:
    assert_command dfx --identity alice canister call e2e_project_assets store '("B", vec { 88; 87; 86 })'
    assert_eq '()'
    assert_command dfx canister call --output idl e2e_project_assets retrieve '("B")'
    assert_eq '(blob "XWV")'
}

@test "after using a specific identity while creating a canister, that wallet is the initializer" {
    [ ! "$USE_IC_REF" ] && skip "Skip until updating to Replica with ic_api_version > 0.14.0"
    install_asset identity
    dfx_start
    assert_command dfx identity new alice
    assert_command dfx identity new bob

    dfx --identity alice canister create --all
    assert_command dfx --identity alice build
    assert_command dfx --identity alice canister install --all

    assert_command dfx --identity alice canister call e2e_project amInitializer
    assert_eq '(false)'

    assert_command dfx --identity alice canister call \
      "$(dfx --identity alice identity get-wallet)" wallet_call \
      "(record { canister = principal \"$(dfx canister id e2e_project)\"; method_name = \"amInitializer\"; args = blob \"DIDL\00\00\"; cycles = (0:nat64)})"
    assert_eq '(record { 153_986_224 = blob "DIDL\00\01~\01" })'  # True in DIDL.

    assert_command dfx --identity bob canister call e2e_project amInitializer
    assert_eq '(false)'

    # these all fail (other identities are not initializer; cannot store assets):
    assert_command_fail dfx --identity bob canister call e2e_project_assets store '("B", vec { 88; 87; 86 })'
    assert_command_fail dfx --identity default canister call e2e_project_assets store '("B", vec { 88; 87; 86 })'
    assert_command_fail dfx canister call e2e_project_assets store '("B", vec { 88; 87; 86 })'
    assert_command_fail dfx canister call e2e_project_assets retrieve '("B")'

    # but alice, the initializer, can store assets:
    assert_command dfx --identity alice canister call e2e_project_assets store '("B", vec { 88; 87; 86 })'
    assert_eq '()'
    assert_command dfx canister call --output idl e2e_project_assets retrieve '("B")'
    assert_eq '(blob "XWV")'
}

@test "after renaming an identity, the renamed identity is still initializer" {
    [ "$USE_IC_REF" ] && skip "Skip for ic-ref as its ic_api_version > 0.14.0, test with set controller with wallet"
    install_asset identity
    dfx_start
    assert_command dfx identity new alice

    dfx --identity alice canister create --all
    assert_command dfx --identity alice build
    assert_command dfx --identity alice canister install --all
    assert_command dfx --identity alice canister call e2e_project amInitializer
    assert_eq '(true)'
    assert_command dfx canister call e2e_project amInitializer
    assert_eq '(false)'

    assert_command dfx identity rename alice bob

    assert_command dfx identity whoami
    assert_eq 'default'
    assert_command dfx --identity bob canister call e2e_project amInitializer
    assert_eq '(true)'

    assert_command dfx --identity bob canister call e2e_project_assets store '("B", blob "hello")'
    assert_eq '()'
    assert_command dfx canister call --output idl e2e_project_assets retrieve '("B")'
    assert_eq '(blob "hello")'
}

@test "after renaming an identity, the renamed identity's wallet is still initializer" {
    [ ! "$USE_IC_REF" ] && skip "Skip until updating to Replica with ic_api_version > 0.14.0"
    install_asset identity
    dfx_start
    assert_command dfx identity new alice

    dfx --identity alice canister create --all
    assert_command dfx --identity alice build
    assert_command dfx --identity alice canister install --all
    assert_command dfx --identity alice canister call \
      "$(dfx --identity alice identity get-wallet)" wallet_call \
      "(record { canister = principal \"$(dfx canister id e2e_project)\"; method_name = \"amInitializer\"; args = blob \"DIDL\00\00\"; cycles = (0:nat64)})"
    assert_eq '(record { 153_986_224 = blob "DIDL\00\01~\01" })'  # True in DIDL.
    assert_command dfx canister call e2e_project amInitializer
    assert_eq '(false)'

    assert_command dfx identity rename alice bob

    assert_command dfx identity whoami
    assert_eq 'default'
    assert_command dfx --identity bob canister call \
      "$(dfx --identity bob identity get-wallet)" wallet_call \
      "(record { canister = principal \"$(dfx canister id e2e_project)\"; method_name = \"amInitializer\"; args = blob \"DIDL\00\00\"; cycles = (0:nat64)})"
    assert_eq '(record { 153_986_224 = blob "DIDL\00\01~\01" })'  # True in DIDL.

    assert_command dfx --identity bob canister call e2e_project_assets store '("B", blob "hello")'
    assert_eq '()'
    assert_command dfx canister call --output idl e2e_project_assets retrieve '("B")'
    assert_eq '(blob "hello")'
}

@test "deploy wallet" {
    [ ! "$USE_IC_REF" ] && skip "Skip until updating to Replica with ic_api_version > 0.14.0"

    dfx_start
    webserver_port=$(cat .dfx/webserver-port)
    # shellcheck disable=SC2094
    cat <<<"$(jq '.networks.actuallylocal.providers=["http://127.0.0.1:'"$webserver_port"'"]' dfx.json)" >dfx.json

    # get a Canister ID to install the wasm onto
    dfx canister --network actuallylocal create abc
    # set controller to user
    dfx canister set-controller abc default

    # We're testing on a local network so the create command actually creates a wallet
    # Delete this file to force associate wallet created by deploy-wallet to identity
    rm "$HOME"/.config/dfx/identity/default/wallets.json

    ID=$(dfx canister --network actuallylocal id abc)
    assert_command dfx identity --network actuallylocal deploy-wallet "${ID}"
    GET_WALLET_RES=$(dfx identity --network actuallylocal get-wallet)
    assert_eq "$ID" "$GET_WALLET_RES"

    dfx canister --network actuallylocal create def
    ID_TWO=$(dfx canister --network actuallylocal id def)
    assert_command_fail dfx identity --network actuallylocal deploy-wallet "${ID_TWO}"
    assert_match "The wallet canister \"${ID}\" already exists for user \"default\" on \"local\" network."
}
