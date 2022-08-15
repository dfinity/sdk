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
    assert_command dfx identity new --disable-encryption jose

    PRINCPAL_ID=$(dfx identity get-principal --identity jose)

    dfx canister create e2e_project_backend --identity jose
    dfx build e2e_project_backend --identity jose
    dfx canister install e2e_project_backend --identity jose

    assert_command dfx canister call e2e_project_backend amInitializer --identity jose

    SENDER_ID=$(dfx canister call e2e_project_backend fromCall --identity jose)

    if [ "$PRINCPAL_ID" -ne "$SENDER_ID" ]; then
      echo "IDs did not match: Principal '${PRINCPAL_ID}' != Sender '${SENDER_ID}'..." | fail
    fi
}

@test "identity get-principal (anonymous): the get-principal is the same as sender id" {
    install_asset identity
    dfx_start
    assert_command dfx identity new --disable-encryption jose

    ANONYMOUS_PRINCIPAL_ID="2vxsx-fae"

    PRINCIPAL_ID=$(dfx identity get-principal --identity anonymous)

    if [ "$PRINCIPAL_ID" -ne "$ANONYMOUS_PRINCIPAL_ID" ]; then
      echo "IDs did not match: Principal '${ANONYMOUS_PRINCIPAL_ID}' != Sender '${PRINCIPAL_ID}'..." | fail
    fi

    dfx canister create e2e_project_backend --identity jose
    dfx build e2e_project_backend --identity jose
    dfx canister install e2e_project_backend --identity jose

    SENDER_ID=$(dfx canister call e2e_project_backend fromCall --identity anonymous)

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

    ID_CALL=$(dfx canister call e2e_project_backend fromCall)
    ID_QUERY=$(dfx canister call e2e_project_backend fromQuery)
    if [ "$ID_CALL" -ne "$ID_QUERY" ]; then
      echo "IDs did not match: call '${ID_CALL}' != query '${ID_QUERY}'..." | fail
    fi

    ID=$(dfx canister call e2e_project_backend getCanisterId)
    assert_command dfx canister call e2e_project_backend isMyself "$ID"
    assert_eq '(true)'
    assert_command dfx canister call e2e_project_backend isMyself "$ID_CALL"
    assert_eq '(false)'
}

@test "dfx ping does not create a default identity" {
    dfx_start

    assert_file_not_exists "$DFX_CONFIG_ROOT/.config/dfx/identity.json"
    assert_file_not_exists "$DFX_CONFIG_ROOT/.config/dfx/identity/default/identity.pem"

    assert_command dfx ping

    assert_file_not_exists "$DFX_CONFIG_ROOT/.config/dfx/identity.json"
    assert_file_not_exists "$DFX_CONFIG_ROOT/.config/dfx/identity/default/identity.pem"

    # shellcheck disable=SC2154
    assert_not_match 'Creating' "$stderr"
    # shellcheck disable=SC2154
    assert_not_match '(default.*identity|identity.*default)' "$stderr"
    # shellcheck disable=SC2154
    assert_match "ic_api_version" "$stdout"
}

@test "dfx canister: creates the default identity on first run" {
    install_asset identity
    dfx_start
    assert_command dfx canister create e2e_project_backend
    assert_match 'Creating the "default" identity.' "$stderr"
}

@test "after using a specific identity while creating a canister, that user is the initializer" {
    install_asset identity
    dfx_start
    assert_command dfx identity new --disable-encryption alice
    assert_command dfx identity new --disable-encryption bob

    dfx canister create --all --identity alice
    assert_command dfx build --identity alice
    assert_command dfx canister install --all --identity alice

    # The user Identity's principal is the initializer
    assert_command dfx canister call e2e_project_backend amInitializer --identity alice
    assert_eq '(true)'

    assert_command dfx canister call e2e_project_backend amInitializer --identity bob
    assert_eq '(false)'

    # these all fail (other identities are not initializer; cannot store assets):
    assert_command_fail dfx canister call e2e_project_frontend store '(record{key="B"; content_type="application/octet-stream"; content_encoding="identity"; content=vec { 88; 87; 86; }})' --identity bob
    assert_command_fail dfx canister call e2e_project_frontend store '(record{key="B"; content_type="application/octet-stream"; content_encoding="identity"; content=vec { 88; 87; 86; }})' --identity default
    assert_command_fail dfx canister call e2e_project_frontend store '(record{key="B"; content_type="application/octet-stream"; content_encoding="identity"; content=vec { 88; 87; 86; }})'
    assert_command_fail dfx canister call e2e_project_frontend retrieve '("B")'

    # but alice, the initializer, can store assets:
    assert_command dfx canister call e2e_project_frontend store '(record{key="B"; content_type="application/octet-stream"; content_encoding="identity"; content=vec { 88; 87; 86; }})' --identity alice
    assert_eq '()'
    assert_command dfx canister call --output idl e2e_project_frontend retrieve '("B")'
    assert_eq '(blob "XWV")'
}

@test "after renaming an identity, the renamed identity is still initializer" {
    install_asset identity
    dfx_start
    assert_command dfx identity new --disable-encryption alice

    dfx canister create --all --identity alice
    assert_command dfx build --identity alice
    assert_command dfx canister install --all --identity alice
    assert_command dfx canister call e2e_project_backend amInitializer --identity alice
    assert_eq '(true)'
    assert_command dfx canister call e2e_project_backend amInitializer
    assert_eq '(false)'

    assert_command dfx identity rename alice bob

    assert_command dfx identity whoami
    assert_eq 'default'
    assert_command dfx canister call e2e_project_backend amInitializer --identity bob
    assert_eq '(true)'

    assert_command dfx canister call e2e_project_frontend store '(record{key="B"; content_type="application/octet-stream"; content_encoding="identity"; content=blob "hello"})' --identity bob
    assert_eq '()'
    assert_command dfx canister call --output idl e2e_project_frontend retrieve '("B")'
    assert_eq '(blob "hello")'
}
