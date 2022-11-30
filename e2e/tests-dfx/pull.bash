#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "dfx build can write required metadata for pull" {
    dfx_new
    install_asset pull

    dfx_start
    
    dfx canister create --all
    assert_command dfx build
    assert_command ic-wasm .dfx/local/canisters/e2e_project_backend/e2e_project_backend.wasm metadata
    assert_match "icp:public candid:service"
    assert_match "icp:public dfx:deps"
    assert_match "icp:public dfx:init"
    assert_match "icp:public dfx:wasm_url"

    assert_command ic-wasm .dfx/local/canisters/e2e_project_backend/e2e_project_backend.wasm metadata dfx:deps
    assert_match "dep1:rrkah-fqaaa-aaaaa-aaaaq-cai;dep2:ryjl3-tyaaa-aaaaa-aaaba-cai;"

    assert_command ic-wasm .dfx/local/canisters/e2e_project_backend/e2e_project_backend.wasm metadata dfx:init
    assert_match "NA"

    assert_command ic-wasm .dfx/local/canisters/e2e_project_backend/e2e_project_backend.wasm metadata dfx:wasm_url
    assert_match "https://example.com/e2e_project.wasm"
}

@test "dfx pull can resolve dependencies from on-chain canister metadata" {
    # system-wide local replica
    dfx_start

    install_asset pullable

    # prepare onchain canisters
    cd onchain

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_a/main.wasm

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_b/empty.wasm
    cd src/onchain_b
    ic-wasm empty.wasm -o main.wasm metadata "dfx:deps" -d "onchain_a:rrkah-fqaaa-aaaaa-aaaaq-cai;" -v public
    cd ../../ # back to root of onchain project
    dfx deploy

    assert_command dfx canister metadata ryjl3-tyaaa-aaaaa-aaaba-cai dfx:deps
    assert_match "onchain_a:rrkah-fqaaa-aaaaa-aaaaq-cai;"

    # app project to pull onchain canisters
    cd ../app
}
