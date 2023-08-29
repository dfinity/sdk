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
    assert_command dfx deploy hello_backend
    assert_command dfx canister info hello_backend
    assert_match "Controllers: ($WALLET $PRINCIPAL|$PRINCIPAL $WALLET)"
}

@test "deploy --no-wallet sets only self as the controller" {
    dfx_start
    WALLET=$(dfx identity get-wallet)
    PRINCIPAL=$(dfx identity get-principal)
    assert_command dfx deploy hello_backend --no-wallet
    assert_command dfx canister info hello_backend
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

    assert_command dfx canister call hello_backend greet '("Banzai")'
    assert_eq '("Hello, Banzai!")'

    assert_command dfx deploy
    assert_not_match "Installing code for"
    assert_match "is already installed"
}

@test "deploying a dependent doesn't require already-installed dependencies to take args" {
    install_asset deploy_deps
    dfx_start
    assert_command dfx deploy dependency --argument '("dfx")'
    touch dependency.mo
    assert_command dfx deploy dependent
    assert_command dfx canister call dependency greet
    assert_match "Hello, dfx!"
}

@test "reinstalling a single Motoko canister with imported dependency works" {
    install_asset import_canister
    dfx_start
    assert_command dfx deploy
    assert_command dfx deploy importer --mode reinstall --yes
}

@test "deploy succeeds with --specified-id" {
    dfx_start
    assert_command dfx deploy hello_backend --specified-id n5n4y-3aaaa-aaaaa-p777q-cai
    assert_command dfx canister id hello_backend
    assert_match n5n4y-3aaaa-aaaaa-p777q-cai
}

@test "deploy fails if --specified-id without canister_name" {
    dfx_start
    assert_command_fail dfx deploy --specified-id n5n4y-3aaaa-aaaaa-p777q-cai
    assert_match \
"error: the following required arguments were not provided:
  <CANISTER_NAME>"
}

@test "deploy does not require wallet if all canisters are created" {
    dfx_start
    dfx canister create --all --no-wallet
    assert_command dfx deploy
    assert_not_contains "Creating a wallet canister"
    assert_command dfx identity get-wallet
    assert_contains "Creating a wallet canister"
}

@test "deploying multiple canisters with arguments fails" {
    assert_command_fail dfx deploy --argument hello
    assert_contains \
"error: the following required arguments were not provided:
  <CANISTER_NAME>"
}

@test "can deploy gzip wasm" {
    jq '.canisters.hello_backend.gzip=true' dfx.json | sponge dfx.json
    dfx_start
    assert_command dfx deploy
    BUILD_HASH="0x$(sha256sum .dfx/local/canisters/hello_backend/hello_backend.wasm.gz | cut -d " " -f 1)"
    ONCHAIN_HASH="$(dfx canister info hello_backend | tail -n 1 | cut -d " " -f 3)"
    assert_eq "$BUILD_HASH" "$ONCHAIN_HASH"
}

@test "prints the frontend url after deploy" {
    dfx_new_frontend hello
    dfx_start
    assert_command dfx deploy
    assert_contains "hello_frontend: http://127.0.0.1"
}

@test "prints the frontend url if 'frontend' section is not present in dfx.json" {
    dfx_new_frontend hello
    jq 'del(.canisters.hello_frontend.frontend)' dfx.json | sponge dfx.json
    dfx_start
    assert_command dfx deploy
    assert_contains "hello_frontend: http://127.0.0.1"
}

@test "prints the frontend url if the frontend section has been removed after initial deployment" {
    dfx_new_frontend hello
    dfx_start
    assert_command dfx deploy
    assert_contains "hello_frontend: http://127.0.0.1"
    jq 'del(.canisters.hello_frontend.frontend)' dfx.json | sponge dfx.json
    assert_command dfx deploy
    assert_contains "hello_frontend: http://127.0.0.1"
}
