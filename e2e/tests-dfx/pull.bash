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
    # When ran with ic-ref, got following error:
    # Certificate is not authorized to respond to queries for this canister. While developing: Did you forget to set effective_canister_id?
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"
    # system-wide local replica
    dfx_start

    install_asset pullable

    # 1. success path
    ## 1.1. prepare "onchain" canisters
    # a -> []
    # b -> [a]
    # c -> [a]
    # app -> [a, b]
    cd onchain

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_a/main.wasm

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_b/empty.wasm
    ic-wasm src/onchain_b/empty.wasm -o src/onchain_b/main.wasm metadata "dfx:deps" -d "onchain_a:rrkah-fqaaa-aaaaa-aaaaq-cai;" -v public

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_c/empty.wasm
    ic-wasm src/onchain_c/empty.wasm -o src/onchain_c/main.wasm metadata "dfx:deps" -d "onchain_a:rrkah-fqaaa-aaaaa-aaaaq-cai;" -v public

    dfx deploy

    assert_command dfx canister metadata ryjl3-tyaaa-aaaaa-aaaba-cai dfx:deps
    assert_match "onchain_a:rrkah-fqaaa-aaaaa-aaaaq-cai;"

    ## 1.2. pull onchain canisters in "app" project
    cd ../app
    assert_command dfx pull dep1
    assert_match "Pulling canister ryjl3-tyaaa-aaaaa-aaaba-cai...
Pulling canister rrkah-fqaaa-aaaaa-aaaaq-cai...
WARN: \`dfx:deps\` metadata not found in canister rrkah-fqaaa-aaaaa-aaaaq-cai."

    assert_command dfx pull # if not specify canister name, all pull type canisters (dep1, dep2) will be pulled
    assert_match "Pulling canister ryjl3-tyaaa-aaaaa-aaaba-cai...
Pulling canister r7inp-6aaaa-aaaaa-aaabq-cai...
Pulling canister rrkah-fqaaa-aaaaa-aaaaq-cai...
WARN: \`dfx:deps\` metadata not found in canister rrkah-fqaaa-aaaaa-aaaaq-cai."
    assert_occurs 1 "Pulling canister rrkah-fqaaa-aaaaa-aaaaq-cai..." # common dependency onchain_a is pulled only once

    # 2. sad path: if the canister is not present on-chain
    cd ../onchain
    dfx canister uninstall-code onchain_a

    cd ../app
    assert_command_fail dfx pull
    assert_contains "Failed while fetch and parse \`dfx:deps\` metadata from canister rrkah-fqaaa-aaaaa-aaaaq-cai."
    assert_contains "Canister rrkah-fqaaa-aaaaa-aaaaq-cai has no module."

    cd ../onchain
    dfx canister stop onchain_a
    dfx canister delete onchain_a

    cd ../app
    assert_command_fail dfx pull
    assert_contains "Failed while fetch and parse \`dfx:deps\` metadata from canister rrkah-fqaaa-aaaaa-aaaaq-cai."
    assert_contains "Canister rrkah-fqaaa-aaaaa-aaaaq-cai not found."

    # 3. sad path: if dependency metadata cannot be read (wrong format)
    cd ../onchain
    cd src/onchain_b
    ic-wasm empty.wasm -o main.wasm metadata "dfx:deps" -d "rrkah-fqaaa-aaaaa-aaaaq-cai;onchain_a" -v public
    cd ../../ # go back to root of "onchain" project
    dfx deploy

    cd ../app
    assert_command_fail dfx pull
    assert_contains "Failed while fetch and parse \`dfx:deps\` metadata from canister ryjl3-tyaaa-aaaaa-aaaba-cai."
    assert_contains "Failed to parse \`dfx:deps\` entry: rrkah-fqaaa-aaaaa-aaaaq-cai. Expected \`name:Principal\`."
}
