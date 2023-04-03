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

@test "dfx deps pull can resolve dependencies from on-chain canister metadata" {
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
    ic-wasm src/onchain_a/main.wasm -o src/onchain_a/main.wasm metadata "dfx:deps" -d "" -v public
    ic-wasm src/onchain_a/main.wasm -o src/onchain_a/main.wasm metadata "candid:service" -d "service : {}" -v public

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_b/main.wasm
    ic-wasm src/onchain_b/main.wasm -o src/onchain_b/main.wasm metadata "dfx:deps" -d "onchain_a:$CANISTER_ID_A;" -v public
    ic-wasm src/onchain_b/main.wasm -o src/onchain_b/main.wasm metadata "candid:service" -d "service : {}" -v public

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_c/main.wasm
    ic-wasm src/onchain_c/main.wasm -o src/onchain_c/main.wasm metadata "dfx:deps" -d "onchain_a:$CANISTER_ID_A;" -v public
    ic-wasm src/onchain_c/main.wasm -o src/onchain_c/main.wasm metadata "candid:service" -d "service : {}" -v public

    dfx deploy

    assert_command dfx canister metadata "$CANISTER_ID_B" dfx:deps
    assert_match "onchain_a:$CANISTER_ID_A;"

    ## 1.2. pull onchain canisters in "app" project
    cd ../app
    jq '.canisters.dep1.id="'"$CANISTER_ID_B"'"' dfx.json | sponge dfx.json
    jq '.canisters.dep2.id="'"$CANISTER_ID_C"'"' dfx.json | sponge dfx.json

    assert_command_fail dfx deps pull # the overall pull fail but succeed to fetch and parse `dfx:deps` recursively
    assert_contains "Resolving dependencies of canister $CANISTER_ID_B...
Resolving dependencies of canister $CANISTER_ID_C...
Resolving dependencies of canister $CANISTER_ID_A...
Found 3 dependencies:"
    assert_occurs 1 "Resolving dependencies of canister $CANISTER_ID_A..." # common dependency onchain_a is pulled only once
    assert_contains "ERROR: Failed to pull canister $CANISTER_ID_B.
\`dfx:wasm_url\` metadata not found in canister $CANISTER_ID_B."
    assert_contains "ERROR: Failed to pull canister $CANISTER_ID_C.
\`dfx:wasm_url\` metadata not found in canister $CANISTER_ID_C."
    assert_contains "ERROR: Failed to pull canister $CANISTER_ID_A.
\`dfx:wasm_url\` metadata not found in canister $CANISTER_ID_A."

    # 2. sad path: if the canister is not present on-chain
    cd ../onchain
    dfx canister uninstall-code onchain_a

    cd ../app
    assert_command_fail dfx deps pull
    assert_contains "Failed to fetch and parse \`dfx:deps\` metadata from canister $CANISTER_ID_A."
    assert_contains "Canister $CANISTER_ID_A has no module."

    cd ../onchain
    dfx canister stop onchain_a
    dfx canister delete onchain_a

    cd ../app
    assert_command_fail dfx deps pull
    assert_contains "Failed to fetch and parse \`dfx:deps\` metadata from canister $CANISTER_ID_A."
    assert_contains "Canister $CANISTER_ID_A not found."

    # 3. sad path: if dependency metadata cannot be read (wrong format)
    cd ../onchain
    cd src/onchain_b
    ic-wasm main.wasm -o main.wasm metadata "dfx:deps" -d "$CANISTER_ID_A;onchain_a" -v public
    cd ../../ # go back to root of "onchain" project
    dfx deploy

    cd ../app
    assert_command_fail dfx deps pull
    assert_contains "Failed to fetch and parse \`dfx:deps\` metadata from canister $CANISTER_ID_B."
    assert_contains "Failed to parse \`dfx:deps\` entry: $CANISTER_ID_A. Expected \`name:Principal\`."
}

@test "dfx deps pull can download wasm and candid to shared cache and generate pulled.json" {
    # When ran with ic-ref, got following error:
    # Certificate is not authorized to respond to queries for this canister. While developing: Did you forget to set effective_canister_id?
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    use_test_specific_cache_root # dfx deps pull will download files to cache

    PULLED_DIR="$DFX_CACHE_ROOT/.cache/dfinity/pulled/"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_B/canister.wasm"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_A/canister.wasm"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_C/canister.wasm"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_B/canister.did"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_A/canister.did"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_C/canister.did"

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
    ic-wasm src/onchain_a/main.wasm -o src/onchain_a/main.wasm metadata "candid:service" -d "service : {}" -v public
    ic-wasm src/onchain_a/main.wasm -o src/onchain_a/main.wasm metadata "dfx:init" -d "onchain_a needs no init inputs" -v public

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_b/main.wasm
    ic-wasm src/onchain_b/main.wasm -o src/onchain_b/main.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/b.wasm" -v public
    ic-wasm src/onchain_b/main.wasm -o src/onchain_b/main.wasm metadata "dfx:deps" -d "onchain_a:$CANISTER_ID_A;" -v public
    ic-wasm src/onchain_b/main.wasm -o src/onchain_b/main.wasm metadata "candid:service" -d "service : {}" -v public

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_c/main.wasm
    ic-wasm src/onchain_c/main.wasm -o src/onchain_c/main.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/c.wasm" -v public
    ic-wasm src/onchain_c/main.wasm -o src/onchain_c/main.wasm metadata "dfx:deps" -d "onchain_a:$CANISTER_ID_A;" -v public
    ic-wasm src/onchain_c/main.wasm -o src/onchain_c/main.wasm metadata "candid:service" -d "service : {}" -v public

    dfx deploy

    # copy wasm files to web server dir
    cp src/onchain_a/main.wasm ../www/a.wasm
    cp src/onchain_b/main.wasm ../www/b.wasm
    cp src/onchain_c/main.wasm ../www/c.wasm

    # pull canisters in app project
    cd ../app
    jq '.canisters.dep1.id="'"$CANISTER_ID_B"'"' dfx.json | sponge dfx.json
    jq '.canisters.dep2.id="'"$CANISTER_ID_C"'"' dfx.json | sponge dfx.json
    assert_file_not_exists "pulled.json"

    assert_command dfx deps pull
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_B/canister.wasm"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_A/canister.wasm"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_C/canister.wasm"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_B/canister.did"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_A/canister.did"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_C/canister.did"

    assert_file_exists "pulled.json"
    assert_eq $CANISTER_ID_B $(jq -r '.named.dep1' pulled.json)
    assert_eq $CANISTER_ID_C $(jq -r '.named.dep2' pulled.json)
    assert_eq 5 $(jq -r '.canisters | keys' pulled.json | wc -l | tr -d ' ') # 3 canisters + 2 lines of '[' and ']'
    assert_command jq -r '.canisters."'"$CANISTER_ID_A"'".init' pulled.json
    assert_match "onchain_a needs no init inputs"

    assert_command dfx deps pull
    assert_contains "The canister wasm was found in the cache." # cache hit

    # sad path 1: wasm hash doesn't match on chain
    rm -r "${PULLED_DIR:?}/"
    cd ../onchain
    cp src/onchain_b/main.wasm ../www/a.wasm 

    cd ../app
    assert_command_fail dfx deps pull
    assert_contains "Failed to pull canister $CANISTER_ID_A."
    assert_contains "Hash mismatch."
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_B/canister.wasm"

    # sad path 2: url server doesn't have the file
    rm -r "${PULLED_DIR:?}/"
    rm ../www/a.wasm

    assert_command_fail dfx deps pull
    assert_contains "Failed to pull canister $CANISTER_ID_A."
    assert_contains "Failed to download wasm from url:"
}


@test "dfx deps pull can check hash when dfx:wasm_hash specified" {
    # When ran with ic-ref, got following error:
    # Certificate is not authorized to respond to queries for this canister. While developing: Did you forget to set effective_canister_id?
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    use_test_specific_cache_root # dfx deps pull will download files to cache

    PULLED_DIR="$DFX_CACHE_ROOT/.cache/dfinity/pulled/"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_B/canister.wasm"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_A/canister.wasm"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_C/canister.wasm"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_B/canister.did"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_A/canister.did"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_C/canister.did"

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
    ic-wasm src/onchain_a/main.wasm -o src/onchain_a/main.wasm metadata "candid:service" -d "service : {}" -v public

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_b/main.wasm
    ic-wasm src/onchain_b/main.wasm -o src/onchain_b/main.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/b.wasm" -v public
    ic-wasm src/onchain_b/main.wasm -o src/onchain_b/main.wasm metadata "dfx:deps" -d "onchain_a:$CANISTER_ID_A;" -v public
    ic-wasm src/onchain_b/main.wasm -o src/onchain_b/main.wasm metadata "candid:service" -d "service : {}" -v public

    echo -n -e \\x00asm\\x01\\x00\\x00\\x00 > src/onchain_c/main.wasm
    ic-wasm src/onchain_c/main.wasm -o src/onchain_c/main.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/c.wasm" -v public
    ic-wasm src/onchain_c/main.wasm -o src/onchain_c/main.wasm metadata "dfx:deps" -d "onchain_a:$CANISTER_ID_A;" -v public
    ic-wasm src/onchain_c/main.wasm -o src/onchain_c/main.wasm metadata "candid:service" -d "service : {}" -v public

    dfx deploy

    # copy wasm files to web server dir
    cp src/onchain_a/custom.wasm ../www/a.wasm
    cp src/onchain_b/main.wasm ../www/b.wasm
    cp src/onchain_c/main.wasm ../www/c.wasm

    # pull canisters in app project
    cd ../app
    jq '.canisters.dep1.id="'"$CANISTER_ID_B"'"' dfx.json | sponge dfx.json
    jq '.canisters.dep2.id="'"$CANISTER_ID_C"'"' dfx.json | sponge dfx.json
    assert_file_not_exists "pulled.json"

    assert_command dfx deps pull
    assert_contains "Canister $CANISTER_ID_A specified a custom hash:"
    
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_B/canister.wasm"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_A/canister.wasm"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_C/canister.wasm"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_B/canister.did"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_A/canister.did"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_C/canister.did"
    assert_file_exists "pulled.json"
}
