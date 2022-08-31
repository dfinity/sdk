#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "can call a canister using an ed25519 identity" {
    install_asset ed25519
    assert_command dfx identity import --disable-encryption ed25519 identity.pem
    dfx_new # This installs replica and other binaries
    dfx identity use ed25519
    install_asset whoami
    dfx_start
    dfx canister create whoami
    dfx build
    dfx canister install whoami
    assert_command dfx canister call whoami whoami
    assert_eq '(principal "2nor3-keehi-duuup-d7jcn-onggn-3atzm-gejtl-5tlzn-k4g6c-nnbf7-7qe")'
    assert_command dfx identity get-principal
    assert_eq "2nor3-keehi-duuup-d7jcn-onggn-3atzm-gejtl-5tlzn-k4g6c-nnbf7-7qe"
}
