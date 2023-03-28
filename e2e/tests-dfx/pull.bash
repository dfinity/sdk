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

export_canister_ids() {
    local a b c
    a=$(dfx canister id onchain_a)
    b=$(dfx canister id onchain_b)
    c=$(dfx canister id onchain_c)
    export CANISTER_ID_A="$a"
    export CANISTER_ID_B="$b"
    export CANISTER_ID_C="$c"
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

    dfx canister create --all
    export_canister_ids

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_a/main.wasm

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_b/main.wasm
    ic-wasm src/onchain_b/main.wasm -o src/onchain_b/main.wasm metadata "dfx:deps" -d "onchain_a:$CANISTER_ID_A;" -v public

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_c/main.wasm
    ic-wasm src/onchain_c/main.wasm -o src/onchain_c/main.wasm metadata "dfx:deps" -d "onchain_a:$CANISTER_ID_A;" -v public

    dfx deploy

    assert_command dfx canister metadata "$CANISTER_ID_B" dfx:deps
    assert_match "onchain_a:$CANISTER_ID_A;"

    ## 1.2. pull onchain canisters in "app" project
    cd ../app
    jq '.canisters.dep1.id="'"$CANISTER_ID_B"'"' dfx.json | sponge dfx.json
    jq '.canisters.dep2.id="'"$CANISTER_ID_C"'"' dfx.json | sponge dfx.json

    assert_command_fail dfx pull dep1 # the overall pull fail but succeed to fetch and parse `dfx:deps` recursively
    assert_contains "Pulling canister $CANISTER_ID_B...
Pulling canister $CANISTER_ID_A...
WARN: \`dfx:deps\` metadata not found in canister $CANISTER_ID_A."
    assert_contains "ERROR: Failed to download wasm of canister $CANISTER_ID_B.
\`dfx:wasm_url\` metadata not found in canister $CANISTER_ID_B."
    assert_contains "ERROR: Failed to download wasm of canister $CANISTER_ID_A.
\`dfx:wasm_url\` metadata not found in canister $CANISTER_ID_A."

    ## 1.3. if not specify canister name, all pull type canisters (dep1, dep2) will be pulled
    assert_command_fail dfx pull # the overall pull fail but succeed to fetch and parse `dfx:deps` recursively
    assert_contains "Pulling canister $CANISTER_ID_B...
Pulling canister $CANISTER_ID_C...
Pulling canister $CANISTER_ID_A...
WARN: \`dfx:deps\` metadata not found in canister $CANISTER_ID_A."
    assert_occurs 1 "Pulling canister $CANISTER_ID_A..." # common dependency onchain_a is pulled only once
    assert_contains "ERROR: Failed to download wasm of canister $CANISTER_ID_B.
\`dfx:wasm_url\` metadata not found in canister $CANISTER_ID_B."
    assert_contains "ERROR: Failed to download wasm of canister $CANISTER_ID_C.
\`dfx:wasm_url\` metadata not found in canister $CANISTER_ID_C."
    assert_contains "ERROR: Failed to download wasm of canister $CANISTER_ID_A.
\`dfx:wasm_url\` metadata not found in canister $CANISTER_ID_A."

    # 2. sad path: if the canister is not present on-chain
    cd ../onchain
    dfx canister uninstall-code onchain_a

    cd ../app
    assert_command_fail dfx pull
    assert_contains "Failed to fetch and parse \`dfx:deps\` metadata from canister $CANISTER_ID_A."
    assert_contains "Canister $CANISTER_ID_A has no module."

    cd ../onchain
    dfx canister stop onchain_a
    dfx canister delete onchain_a

    cd ../app
    assert_command_fail dfx pull
    assert_contains "Failed to fetch and parse \`dfx:deps\` metadata from canister $CANISTER_ID_A."
    assert_contains "Canister $CANISTER_ID_A not found."

    # 3. sad path: if dependency metadata cannot be read (wrong format)
    cd ../onchain
    cd src/onchain_b
    ic-wasm main.wasm -o main.wasm metadata "dfx:deps" -d "$CANISTER_ID_A;onchain_a" -v public
    cd ../../ # go back to root of "onchain" project
    dfx deploy

    cd ../app
    assert_command_fail dfx pull
    assert_contains "Failed to fetch and parse \`dfx:deps\` metadata from canister $CANISTER_ID_B."
    assert_contains "Failed to parse \`dfx:deps\` entry: $CANISTER_ID_A. Expected \`name:Principal\`."
}

@test "dfx pull can download wasm" {
    # When ran with ic-ref, got following error:
    # Certificate is not authorized to respond to queries for this canister. While developing: Did you forget to set effective_canister_id?
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"
    use_test_specific_cache_root # dfx pull will download files to cache

    WASM_CACHE="$DFX_CACHE_ROOT/.cache/dfinity/wasms/"

    assert_file_not_exists "$WASM_CACHE/$CANISTER_ID_B/canister.wasm"
    assert_file_not_exists "$WASM_CACHE/$CANISTER_ID_A/canister.wasm"
    assert_file_not_exists "$WASM_CACHE/$CANISTER_ID_C/canister.wasm"

    # system-wide local replica
    dfx_start

    install_asset pullable

    # start a webserver to host wasm files
    mkdir www
    start_webserver --directory www

    # prepare "onchain" canisters
    cd onchain

    dfx canister create --all
    export_canister_ids

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_a/main.wasm
    ic-wasm src/onchain_a/main.wasm -o src/onchain_a/main.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/a.wasm" -v public

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_b/main.wasm
    ic-wasm src/onchain_b/main.wasm -o src/onchain_b/main.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/b.wasm" -v public
    ic-wasm src/onchain_b/main.wasm -o src/onchain_b/main.wasm metadata "dfx:deps" -d "onchain_a:$CANISTER_ID_A;" -v public

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_c/main.wasm
    ic-wasm src/onchain_c/main.wasm -o src/onchain_c/main.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/c.wasm" -v public
    ic-wasm src/onchain_c/main.wasm -o src/onchain_c/main.wasm metadata "dfx:deps" -d "onchain_a:$CANISTER_ID_A;" -v public

    dfx deploy

    # copy wasm files to web server dir
    cp src/onchain_a/main.wasm ../www/a.wasm
    cp src/onchain_b/main.wasm ../www/b.wasm
    cp src/onchain_c/main.wasm ../www/c.wasm

    # pull canisters in app project
    cd ../app
    jq '.canisters.dep1.id="'"$CANISTER_ID_B"'"' dfx.json | sponge dfx.json
    jq '.canisters.dep2.id="'"$CANISTER_ID_C"'"' dfx.json | sponge dfx.json

    assert_command dfx pull dep1
    
    assert_file_exists "$WASM_CACHE/$CANISTER_ID_B/canister.wasm"
    assert_file_exists "$WASM_CACHE/$CANISTER_ID_A/canister.wasm"
    assert_file_not_exists "$WASM_CACHE/$CANISTER_ID_C/canister.wasm"

    assert_command dfx pull # if not specify canister name, all pull type canisters (dep1, dep2) will be pulled
    assert_contains "The canister wasm was found in the cache." # a, b were downloaded before
    assert_file_exists "$WASM_CACHE/$CANISTER_ID_C/canister.wasm"

    # sad path 1: wasm hash doesn't match on chain
    rm -r "${WASM_CACHE:?}/"
    cd ../onchain
    cp src/onchain_b/main.wasm ../www/a.wasm 

    cd ../app
    assert_command_fail dfx pull dep1
    assert_contains "Failed to download wasm of canister $CANISTER_ID_A."
    assert_contains "Hash mismatch."
    assert_file_exists "$WASM_CACHE/$CANISTER_ID_B/canister.wasm"

    # sad path 2: url server doesn't have the file
    rm -r "${WASM_CACHE:?}/"
    rm ../www/a.wasm

    assert_command_fail dfx pull dep1
    assert_contains "Failed to download wasm of canister $CANISTER_ID_A."
    assert_contains "Failed to download wasm from url:"
}


@test "dfx pull can check hash when dfx:wasm_hash specified" {
    # When ran with ic-ref, got following error:
    # Certificate is not authorized to respond to queries for this canister. While developing: Did you forget to set effective_canister_id?
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"
    use_test_specific_cache_root # dfx pull will download files to cache

    WASM_CACHE="$DFX_CACHE_ROOT/.cache/dfinity/wasms/"

    assert_file_not_exists "$WASM_CACHE/$CANISTER_ID_B/canister.wasm"
    assert_file_not_exists "$WASM_CACHE/$CANISTER_ID_A/canister.wasm"
    assert_file_not_exists "$WASM_CACHE/$CANISTER_ID_C/canister.wasm"

    # system-wide local replica
    dfx_start

    install_asset pullable

    # start a webserver to host wasm files
    mkdir www
    start_webserver --directory www

    # prepare "onchain" canisters
    cd onchain

    dfx canister create --all
    export_canister_ids

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_a/main.wasm # to be deployed
    ic-wasm src/onchain_a/main.wasm -o src/onchain_a/main.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/a.wasm" -v public
    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_a/custom.wasm # to be download
    ic-wasm src/onchain_a/main.wasm -o src/onchain_a/main.wasm metadata "dfx:wasm_hash" -d "$(sha256sum src/onchain_a/custom.wasm | cut -d " " -f 1)" -v public

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_b/main.wasm
    ic-wasm src/onchain_b/main.wasm -o src/onchain_b/main.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/b.wasm" -v public
    ic-wasm src/onchain_b/main.wasm -o src/onchain_b/main.wasm metadata "dfx:deps" -d "onchain_a:$CANISTER_ID_A;" -v public

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_c/main.wasm
    ic-wasm src/onchain_c/main.wasm -o src/onchain_c/main.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/c.wasm" -v public
    ic-wasm src/onchain_c/main.wasm -o src/onchain_c/main.wasm metadata "dfx:deps" -d "onchain_a:$CANISTER_ID_A;" -v public

    dfx deploy

    # copy wasm files to web server dir
    cp src/onchain_a/custom.wasm ../www/a.wasm
    cp src/onchain_b/main.wasm ../www/b.wasm
    cp src/onchain_c/main.wasm ../www/c.wasm

    # pull canisters in app project
    cd ../app
    jq '.canisters.dep1.id="'"$CANISTER_ID_B"'"' dfx.json | sponge dfx.json
    jq '.canisters.dep2.id="'"$CANISTER_ID_C"'"' dfx.json | sponge dfx.json

    assert_command dfx pull
    assert_contains "Canister $CANISTER_ID_A specified a custom hash:"
    
    assert_file_exists "$WASM_CACHE/$CANISTER_ID_B/canister.wasm"
    assert_file_exists "$WASM_CACHE/$CANISTER_ID_A/canister.wasm"
    assert_file_exists "$WASM_CACHE/$CANISTER_ID_C/canister.wasm"
}
