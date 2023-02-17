#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    stop_webserver

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

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_b/main.wasm
    ic-wasm src/onchain_b/main.wasm -o src/onchain_b/main.wasm metadata "dfx:deps" -d "onchain_a:rrkah-fqaaa-aaaaa-aaaaq-cai;" -v public

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_c/main.wasm
    ic-wasm src/onchain_c/main.wasm -o src/onchain_c/main.wasm metadata "dfx:deps" -d "onchain_a:rrkah-fqaaa-aaaaa-aaaaq-cai;" -v public

    dfx deploy

    assert_command dfx canister metadata ryjl3-tyaaa-aaaaa-aaaba-cai dfx:deps
    assert_match "onchain_a:rrkah-fqaaa-aaaaa-aaaaq-cai;"

    ## 1.2. pull onchain canisters in "app" project
    cd ../app
    assert_command_fail dfx pull dep1
    assert_contains "Pulling canister ryjl3-tyaaa-aaaaa-aaaba-cai...
Pulling canister rrkah-fqaaa-aaaaa-aaaaq-cai...
WARN: \`dfx:deps\` metadata not found in canister rrkah-fqaaa-aaaaa-aaaaq-cai."

    assert_command_fail dfx pull # if not specify canister name, all pull type canisters (dep1, dep2) will be pulled
    assert_contains "Pulling canister ryjl3-tyaaa-aaaaa-aaaba-cai...
Pulling canister r7inp-6aaaa-aaaaa-aaabq-cai...
Pulling canister rrkah-fqaaa-aaaaa-aaaaq-cai...
WARN: \`dfx:deps\` metadata not found in canister rrkah-fqaaa-aaaaa-aaaaq-cai."
    assert_occurs 1 "Pulling canister rrkah-fqaaa-aaaaa-aaaaq-cai..." # common dependency onchain_a is pulled only once

    # 2. sad path: if the canister is not present on-chain
    cd ../onchain
    dfx canister uninstall-code onchain_a

    cd ../app
    assert_command_fail dfx pull
    assert_contains "Failed to fetch and parse \`dfx:deps\` metadata from canister rrkah-fqaaa-aaaaa-aaaaq-cai."
    assert_contains "Canister rrkah-fqaaa-aaaaa-aaaaq-cai has no module."

    cd ../onchain
    dfx canister stop onchain_a
    dfx canister delete onchain_a

    cd ../app
    assert_command_fail dfx pull
    assert_contains "Failed to fetch and parse \`dfx:deps\` metadata from canister rrkah-fqaaa-aaaaa-aaaaq-cai."
    assert_contains "Canister rrkah-fqaaa-aaaaa-aaaaq-cai not found."

    # 3. sad path: if dependency metadata cannot be read (wrong format)
    cd ../onchain
    cd src/onchain_b
    ic-wasm main.wasm -o main.wasm metadata "dfx:deps" -d "rrkah-fqaaa-aaaaa-aaaaq-cai;onchain_a" -v public
    cd ../../ # go back to root of "onchain" project
    dfx deploy

    cd ../app
    assert_command_fail dfx pull
    assert_contains "Failed to fetch and parse \`dfx:deps\` metadata from canister ryjl3-tyaaa-aaaaa-aaaba-cai."
    assert_contains "Failed to parse \`dfx:deps\` entry: rrkah-fqaaa-aaaaa-aaaaq-cai. Expected \`name:Principal\`."
}

@test "dfx pull can download wasm" {
    # When ran with ic-ref, got following error:
    # Certificate is not authorized to respond to queries for this canister. While developing: Did you forget to set effective_canister_id?
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"
    use_test_specific_cache_root # dfx pull will download files to cache

    WASM_CACHE="$DFX_CACHE_ROOT/.cache/dfinity/wasms/"

    assert_file_not_exists "$WASM_CACHE/ryjl3-tyaaa-aaaaa-aaaba-cai/canister.wasm"
    assert_file_not_exists "$WASM_CACHE/rrkah-fqaaa-aaaaa-aaaaq-cai/canister.wasm"
    assert_file_not_exists "$WASM_CACHE/r7inp-6aaaa-aaaaa-aaabq-cai/canister.wasm"

    # system-wide local replica
    dfx_start

    install_asset pullable

    # start a webserver to host wasm files
    mkdir www
    start_webserver --directory www

    # prepare "onchain" canisters
    cd onchain

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_a/main.wasm
    ic-wasm src/onchain_a/main.wasm -o src/onchain_a/main.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/a.wasm"

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_b/main.wasm
    ic-wasm src/onchain_b/main.wasm -o src/onchain_b/main.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/b.wasm"
    ic-wasm src/onchain_b/main.wasm -o src/onchain_b/main.wasm metadata "dfx:deps" -d "onchain_a:rrkah-fqaaa-aaaaa-aaaaq-cai;" -v public

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_c/main.wasm
    ic-wasm src/onchain_c/main.wasm -o src/onchain_c/main.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/c.wasm"
    ic-wasm src/onchain_c/main.wasm -o src/onchain_c/main.wasm metadata "dfx:deps" -d "onchain_a:rrkah-fqaaa-aaaaa-aaaaq-cai;" -v public

    dfx deploy

    assert_command dfx canister metadata ryjl3-tyaaa-aaaaa-aaaba-cai dfx:deps
    assert_match "onchain_a:rrkah-fqaaa-aaaaa-aaaaq-cai;"

    # copy wasm files to web server dir
    cp src/onchain_a/main.wasm ../www/a.wasm
    cp src/onchain_b/main.wasm ../www/b.wasm
    cp src/onchain_c/main.wasm ../www/c.wasm

    # pull canisters in app project
    cd ../app
    assert_command dfx pull dep1
    
    assert_file_exists "$WASM_CACHE/ryjl3-tyaaa-aaaaa-aaaba-cai/canister.wasm"
    assert_file_exists "$WASM_CACHE/rrkah-fqaaa-aaaaa-aaaaq-cai/canister.wasm"
    assert_file_not_exists "$WASM_CACHE/r7inp-6aaaa-aaaaa-aaabq-cai/canister.wasm"

    assert_command dfx pull # if not specify canister name, all pull type canisters (dep1, dep2) will be pulled
    assert_contains "The canister wasm found in cache." # a, b were downloaded before
    assert_file_exists "$WASM_CACHE/r7inp-6aaaa-aaaaa-aaabq-cai/canister.wasm"

    # sad path 1: wasm hash doesn't match on chain
    rm -r "${WASM_CACHE:?}/"
    cd ../onchain
    cp src/onchain_b/main.wasm ../www/a.wasm 

    cd ../app
    assert_command_fail dfx pull dep1
    assert_contains "Failed to download wasm of canister rrkah-fqaaa-aaaaa-aaaaq-cai."
    assert_file_exists "$WASM_CACHE/ryjl3-tyaaa-aaaaa-aaaba-cai/canister.wasm"

    # sad path 2: url server doesn't have the file
    rm -r "${WASM_CACHE:?}/"
    rm ../www/a.wasm

    assert_command_fail dfx pull dep1
    assert_contains "Failed to download wasm of canister rrkah-fqaaa-aaaaa-aaaaq-cai."
}
