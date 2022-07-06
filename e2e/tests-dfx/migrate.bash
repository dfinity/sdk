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

@test "detects the wallet being outdated" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref because uploading wallet.wasm as data takes too long"

    use_wallet_wasm 0.7.2
    dfx_start
    WALLET=$(dfx identity get-wallet)
    use_wallet_wasm 0.10.0
    assert_command dfx diagnose
    assert_match "dfx wallet upgrade"
    assert_command_fail dfx canister call "$WALLET" wallet_balance128
    assert_command dfx fix
    assert_command dfx canister call "$WALLET" wallet_balance128
}

@test "detects the wallet being the sole controller" {
    dfx_start
    dfx canister create e2e_project_backend --controller "$(dfx identity get-wallet)" --no-wallet
    dfx build e2e_project_backend
    assert_command dfx diagnose
    assert_match "dfx canister update-settings"
    assert_command_fail dfx canister install e2e_project_backend
    assert_command dfx fix
    assert_command dfx canister install e2e_project_backend
}
