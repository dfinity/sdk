#!/usr/bin/env bats

load ../utils/_

setup() {
    # We want to work from a different temporary directory for every test.
    x=$(mktemp -d -t dfx-identity-home-XXXXXXXX)
    export TEMPORARY_HOME="$x"
    export HOME="$TEMPORARY_HOME"
}

teardown() {
    dfx_stop
    rm -rf "$TEMPORARY_HOME"
}

@test "can call a canister using a secp256k1 identity" {
    openssl ecparam -name secp256k1 -genkey -out identity.pem
    assert_command dfx identity import secp256k1 identity.pem
    dfx identity use secp256k1
    install_asset whoami
    dfx_start
    dfx canister create whoami
    dfx build
    dfx canister install whoami
    assert_command dfx canister whoami whoami
    assert_match "$(dfx identity get-principal)"
}
