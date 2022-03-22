#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "can call a canister using a secp256k1 identity" {
    openssl ecparam -name secp256k1 -genkey -out identity.pem
    assert_command dfx identity import --disable-encryption secp256k1 identity.pem
    dfx_new # This installs replica and other binaries
    dfx identity use secp256k1
    install_asset whoami
    dfx_start
    dfx canister create whoami
    dfx build
    dfx canister install whoami
    assert_command dfx canister call whoami whoami
    assert_match "$(dfx identity get-principal)"
}
