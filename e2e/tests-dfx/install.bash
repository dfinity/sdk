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

@test "canister install --upgrade-unchanged upgrades even if the .wasm did not change" {
    dfx_start
    dfx canister create --all
    dfx build

    assert_command dfx canister install --all

    assert_command dfx canister install --all --mode upgrade
    assert_match "Module hash.*is already installed"

    assert_command dfx canister install --all --mode upgrade --upgrade-unchanged
    assert_not_match "Module hash.*is already installed"
}

@test "install fails if no argument is provided" {
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    dfx_start
    assert_command_fail dfx canister install
    assert_match "required arguments were not provided"
    assert_match "--all"
}

@test "install succeeds when --all is provided" {
    dfx_start
    dfx canister create --all
    dfx build

    assert_command dfx canister install --all

    assert_match "Installing code for canister e2e_project_backend"
}

@test "install succeeds with network name" {
    dfx_start
    dfx canister create --all
    dfx build

    assert_command dfx canister install --all --network local

    assert_match "Installing code for canister e2e_project_backend"
}

@test "install fails with network name that is not in dfx.json" {
    dfx_start
    dfx canister create --all
    dfx build

    assert_command_fail dfx canister install --all --network nosuch

    assert_match "ComputeNetworkNotFound.*nosuch"
}

@test "install succeeds with arbitrary wasm" {
    dfx_start
    dfx canister create --all
    wallet="${archive:?}/wallet/0.10.0/wallet.wasm"
    assert_command dfx canister install e2e_project_backend --wasm "$wallet"
    assert_command dfx canister info e2e_project_backend
    assert_match "Module hash: 0x$(sha2sum "$wallet" | head -c 64)"
}

@test "install succeeds with canisterid" {
    dfx_start
    dfx canister create --all
    wallet="${archive:?}/wallet/0.10.0/wallet.wasm"
    CANISTER_ID=$(dfx canister id e2e_project_backend)
    assert_command dfx canister install "$CANISTER_ID" --wasm "$wallet"
    assert_command dfx canister info "$CANISTER_ID"
    assert_match "Module hash: 0x$(sha2sum "$wallet" | head -c 64)"
}

@test "install --all fails with arbitrary wasm" {
    dfx_start
    dfx canister create --all
    assert_command_fail dfx canister install --all --wasm "${archive:?}/wallet/0.10.0/wallet.wasm"
}

@test "install runs post-install tasks" {
    install_asset post_install
    dfx_start

    assert_command dfx canister create --all
    assert_command dfx build

    assert_command dfx canister install postinstall
    assert_match 'hello-file'

    assert_command dfx canister install postinstall_script
    assert_match 'hello-script'
    
    echo 'return 1' >> postinstall.sh
    assert_command_fail dfx canister install postinstall_script --mode upgrade
    assert_match 'hello-script'
}

@test "post-install tasks receive environment variables" {
    install_asset post_install
    dfx_start
    echo "echo hello \$CANISTER_ID" >> postinstall.sh

    assert_command dfx canister create --all
    assert_command dfx build
    id=$(dfx canister id postinstall_script)

    assert_command dfx canister install --all
    assert_match "hello $id"
    assert_command dfx canister install postinstall_script --mode upgrade
    assert_match "hello $id"

    assert_command dfx deploy
    assert_match "hello $id"
    assert_command dfx deploy postinstall_script
    assert_match "hello $id"
}

@test "post-install tasks discover dependencies" {
    install_asset post_install
    dfx_start
    echo "echo hello \$CANISTER_ID_postinstall" >> postinstall.sh

    assert_command dfx canister create --all
    assert_command dfx build
    id=$(dfx canister id postinstall)
    
    assert_command dfx canister install postinstall_script
    assert_match "hello $id"
}

@test "can install wasm.gz canisters" {
    install_asset gzip
    install_asset wasm/identity
    dfx_start
    assert_command dfx canister create --all 
    assert_command dfx build
    assert_command dfx canister install --all
    assert_command dfx canister call gzipped fromQuery '()'
    assert_match "$(dfx identity get-principal)"
}

@test "--mode=auto selects install or upgrade automatically" {
    dfx_start
    assert_command dfx canister create e2e_project_backend
    assert_command dfx build e2e_project_backend
    assert_command dfx canister install e2e_project_backend --mode auto
    assert_command dfx canister call e2e_project_backend greet dfx
    assert_command dfx canister install e2e_project_backend --mode auto --upgrade-unchanged
    assert_command dfx canister call e2e_project_backend greet dfx
}
