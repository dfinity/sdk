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

@test "identity get-principal (anonymous): the get-principal is the same as sender id" {
    install_asset identity
    dfx_start
    assert_command dfx identity new jose

    ANONYMOUS_PRINCIPAL_ID="2vxsx-fae"

    PRINCIPAL_ID=$(dfx --identity anonymous identity get-principal)

    if [ "$PRINCIPAL_ID" -ne "$ANONYMOUS_PRINCIPAL_ID" ]; then
      echo "IDs did not match: Principal '${ANONYMOUS_PRINCIPAL_ID}' != Sender '${PRINCIPAL_ID}'..." | fail
    fi

    dfx --identity jose canister create e2e_project
    dfx --identity jose build e2e_project
    dfx --identity jose canister install e2e_project

    SENDER_ID=$(dfx --identity anonymous canister call e2e_project fromCall)

    if [ "$ANONYMOUS_PRINCIPAL_ID" -ne "$SENDER_ID" ]; then
      echo "IDs did not match: Principal '${ANONYMOUS_PRINCIPAL_ID}' != Sender '${SENDER_ID}'..." | fail
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

@test "after using a specific identity while creating a canister, that wallet is the initializer" {
    install_asset identity
    dfx_start
    assert_command dfx identity new alice
    assert_command dfx identity new bob

    dfx --identity alice canister create --all
    assert_command dfx --identity alice build
    assert_command dfx --identity alice canister install --all

    # The wallet is the initializer
    assert_command dfx --identity alice canister --wallet="$(dfx --identity alice identity get-wallet)" call e2e_project amInitializer
    assert_eq '(true)'

    # The user Identity's principal is not the initializer
    assert_command dfx --identity alice canister call e2e_project amInitializer
    assert_eq '(false)'

    assert_command dfx --identity alice canister --no-wallet call \
      "$(dfx --identity alice identity get-wallet)" wallet_call \
      "(record { canister = principal \"$(dfx canister id e2e_project)\"; method_name = \"amInitializer\"; args = blob \"DIDL\00\00\"; cycles = (0:nat64)})"
    assert_eq '(variant { 17_724 = record { 153_986_224 = blob "DIDL\00\01~\01" } })'  # True in DIDL.

    assert_command dfx --identity bob canister --no-wallet call e2e_project amInitializer
    assert_eq '(false)'

    # these all fail (other identities are not initializer; cannot store assets):
    assert_command_fail dfx --identity bob canister --no-wallet call e2e_project_assets store '(record{key="B"; content_type="application/octet-stream"; content_encoding="identity"; content=vec { 88; 87; 86; }})'
    assert_command_fail dfx --identity default canister --no-wallet call e2e_project_assets store '(record{key="B"; content_type="application/octet-stream"; content_encoding="identity"; content=vec { 88; 87; 86; }})'
    assert_command_fail dfx canister --no-wallet call e2e_project_assets store '(record{key="B"; content_type="application/octet-stream"; content_encoding="identity"; content=vec { 88; 87; 86; }})'
    assert_command_fail dfx canister --no-wallet call e2e_project_assets retrieve '("B")'

    # but alice, the initializer, can store assets:
    assert_command dfx --identity alice canister call e2e_project_assets store '(record{key="B"; content_type="application/octet-stream"; content_encoding="identity"; content=vec { 88; 87; 86; }})'
    assert_eq '()'
    assert_command dfx canister --no-wallet call --output idl e2e_project_assets retrieve '("B")'
    assert_eq '(blob "XWV")'
}

@test "after renaming an identity, the renamed identity's wallet is still initializer" {
    install_asset identity
    dfx_start
    assert_command dfx identity new alice

    dfx --identity alice canister create --all
    assert_command dfx --identity alice build
    assert_command dfx --identity alice canister install --all
    assert_command dfx --identity alice canister --no-wallet call \
      "$(dfx --identity alice identity get-wallet)" wallet_call \
      "(record { canister = principal \"$(dfx canister id e2e_project)\"; method_name = \"amInitializer\"; args = blob \"DIDL\00\00\"; cycles = (0:nat64)})"
    assert_eq '(variant { 17_724 = record { 153_986_224 = blob "DIDL\00\01~\01" } })'  # True in DIDL.
    assert_command dfx canister --no-wallet call e2e_project amInitializer
    assert_eq '(false)'

    assert_command dfx identity rename alice bob

    assert_command dfx identity whoami
    assert_eq 'default'
    assert_command dfx --identity bob canister --no-wallet call \
      "$(dfx --identity bob identity get-wallet)" wallet_call \
      "(record { canister = principal \"$(dfx canister id e2e_project)\"; method_name = \"amInitializer\"; args = blob \"DIDL\00\00\"; cycles = (0:nat64)})"
    assert_eq '(variant { 17_724 = record { 153_986_224 = blob "DIDL\00\01~\01" } })'  # True in DIDL.

    assert_command dfx --identity bob canister call e2e_project_assets store '(record{key="B"; content_type="application/octet-stream"; content_encoding="identity"; content=blob "hello"})'
    assert_eq '()'
    assert_command dfx canister --no-wallet call --output idl e2e_project_assets retrieve '("B")'
    assert_eq '(blob "hello")'
}
