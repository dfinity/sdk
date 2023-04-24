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

CANISTER_ID_A="yofga-2qaaa-aaaaa-aabsq-cai"
CANISTER_ID_B="yhgn4-myaaa-aaaaa-aabta-cai"
CANISTER_ID_C="yahli-baaaa-aaaaa-aabtq-cai"

setup_onchain() {
    install_asset deps

    # start a webserver to host wasm files
    mkdir www
    start_webserver --directory www

    cd onchain || exit

    dfx canister create a --specified-id "$CANISTER_ID_A"
    dfx canister create b --specified-id "$CANISTER_ID_B"
    dfx canister create c --specified-id "$CANISTER_ID_C"
    dfx build

    cd .dfx/local/canisters || exit
    ic-wasm a/a.wasm -o a/a.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/a.wasm" -v public
    ic-wasm b/b.wasm -o b/b.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/b.wasm" -v public
    ic-wasm c/c.wasm -o c/c.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/c.wasm" -v public

    cd ../../../ || exit
    dfx canister install a --argument 1
    dfx canister install b
    dfx canister install c --argument 3

    # copy wasm files to web server dir
    cp .dfx/local/canisters/a/a.wasm ../www/a.wasm
    cp .dfx/local/canisters/b/b.wasm ../www/b.wasm
    cp .dfx/local/canisters/c/c.wasm ../www/c.wasm

    cd .. || exit
}

@test "dfx build can write required metadata for pull" {
    dfx_start

    install_asset deps

    cd onchain
    dfx canister create --all
    assert_command dfx build
    assert_command ic-wasm .dfx/local/canisters/c/c.wasm metadata
    assert_match "icp:public candid:service"
    assert_match "icp:public dfx:deps"
    assert_match "icp:public dfx:init"
    assert_match "icp:public dfx:wasm_url"

    assert_command ic-wasm .dfx/local/canisters/c/c.wasm metadata dfx:deps
    assert_match "yofga-2qaaa-aaaaa-aabsq-cai;"

    assert_command ic-wasm .dfx/local/canisters/c/c.wasm metadata dfx:init
    assert_match "Nat"

    assert_command ic-wasm .dfx/local/canisters/c/c.wasm metadata dfx:wasm_url
    assert_match "http://example.com/c.wasm"
}

@test "dfx deps pull can resolve dependencies from on-chain canister metadata" {
    # ic-ref has different behavior than the replica:
    #   it doesn't differ whether the canister not exist or the metadata not exist
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"
    dfx_start

    install_asset deps

    # 1. success path
    ## 1.1. prepare "onchain" canisters
    # a -> []
    # b -> [a]
    # c -> [a]
    # app -> [a, b]

    cd onchain

    dfx canister create a --specified-id "$CANISTER_ID_A"
    dfx canister create b --specified-id "$CANISTER_ID_B"
    dfx canister create c --specified-id "$CANISTER_ID_C"

    dfx deploy a --argument 1
    dfx deploy b
    dfx deploy c --argument 3

    assert_command dfx canister metadata b dfx:deps
    assert_match "$CANISTER_ID_A;"

    ## 1.2. pull onchain canisters in "app" project
    cd ../app

    assert_command_fail dfx deps pull --network local # the overall pull fail but succeed to fetch and parse `dfx:deps` recursively
    assert_contains "Resolving dependencies of canister $CANISTER_ID_B...
Resolving dependencies of canister $CANISTER_ID_C...
Resolving dependencies of canister $CANISTER_ID_A...
WARN: \`dfx:deps\` metadata not found in canister $CANISTER_ID_A.
Found 3 dependencies:
yofga-2qaaa-aaaaa-aabsq-cai
yhgn4-myaaa-aaaaa-aabta-cai
yahli-baaaa-aaaaa-aabtq-cai"
    assert_occurs 1 "Resolving dependencies of canister $CANISTER_ID_A..." # common dependency onchain_a is pulled only once
    assert_contains "Pulling canister $CANISTER_ID_A...
ERROR: Failed to pull canister $CANISTER_ID_A.
Failed to download wasm from url: http://example.com/a.wasm."
    assert_contains "Pulling canister $CANISTER_ID_B...
ERROR: Failed to pull canister $CANISTER_ID_B.
Failed to download wasm from url: http://example.com/b.wasm."
    assert_contains "Pulling canister $CANISTER_ID_C...
ERROR: Failed to pull canister $CANISTER_ID_C.
Failed to download wasm from url: http://example.com/c.wasm."

    # 2. sad path: if dependency metadata cannot be read (wrong format)
    cd ../onchain
    cd .dfx/local/canisters
    ic-wasm c/c.wasm -o c/c.wasm metadata "dfx:deps" -d "not_a_principal;" -v public
    cd ../../../ # go back to root of "onchain" project
    dfx canister install c --argument 3 --mode=reinstall --yes

    cd ../app
    assert_command_fail dfx deps pull --network local
    assert_contains "Failed to get dependencies of canister $CANISTER_ID_C."
    assert_contains "Found invalid entry in \`dfx:deps\`: \"not_a_principal\". Expected a Principal."


    # 3. sad path: if the canister is not present on-chain
    cd ../onchain
    dfx build c
    dfx canister install c --argument 3 --mode=reinstall --yes # reinstall the correct canister c
    dfx canister uninstall-code a

    cd ../app
    assert_command_fail dfx deps pull --network local
    assert_contains "Failed to get dependencies of canister $CANISTER_ID_A."
    assert_contains "Canister $CANISTER_ID_A has no module."

    cd ../onchain
    dfx canister stop a
    dfx canister delete a

    cd ../app
    assert_command_fail dfx deps pull --network local
    assert_contains "Failed to get dependencies of canister $CANISTER_ID_A."
    assert_contains "Canister $CANISTER_ID_A not found."
}

@test "dfx deps pull can download wasm and candids to shared cache and generate pulled.json" {
    use_test_specific_cache_root # dfx deps pull will download files to cache

    PULLED_DIR="$DFX_CACHE_ROOT/.cache/dfinity/pulled/"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_B/canister.wasm"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_A/canister.wasm"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_C/canister.wasm"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_B/service.did"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_A/service.did"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_C/service.did"

    # start a "mainnet" replica which host the onchain canisters
    dfx_start

    setup_onchain

    # pull canisters in app project
    cd app
    assert_file_not_exists "deps/pulled.json"

    assert_command dfx deps pull --network local
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_B/canister.wasm"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_A/canister.wasm"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_C/canister.wasm"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_B/service.did"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_A/service.did"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_C/service.did"

    cd deps
    assert_file_exists "pulled.json"
    assert_file_exists "$CANISTER_ID_B.did"
    assert_file_exists "$CANISTER_ID_C.did"
    assert_eq 5 "$(jq -r '.canisters | keys' pulled.json | wc -l | tr -d ' ')" # 3 canisters + 2 lines of '[' and ']'
    assert_command jq -r '.canisters."'"$CANISTER_ID_A"'".dfx_init' pulled.json
    assert_match "Nat"
    assert_command jq -r '.canisters."'"$CANISTER_ID_B"'".name' pulled.json
    assert_match "dep_b"
    assert_command jq -r '.canisters."'"$CANISTER_ID_C"'".name' pulled.json
    assert_match "dep_c"
    cd ../

    assert_command dfx deps pull --network local -vvv
    assert_contains "The canister wasm was found in the cache." # cache hit

    # sad path 1: wasm hash doesn't match on chain
    rm -r "${PULLED_DIR:?}/"
    cd ../onchain
    cp .dfx/local/canisters/b/b.wasm ../www/a.wasm 

    cd ../app
    assert_command_fail dfx deps pull --network local
    assert_contains "Failed to pull canister $CANISTER_ID_A."
    assert_contains "Hash mismatch."

    # sad path 2: url server doesn't have the file
    rm -r "${PULLED_DIR:?}/"
    rm ../www/a.wasm

    assert_command_fail dfx deps pull --network local
    assert_contains "Failed to pull canister $CANISTER_ID_A."
    assert_contains "Failed to download wasm from url:"
}


@test "dfx deps pull can check hash when dfx:wasm_hash specified" {
    use_test_specific_cache_root # dfx deps pull will download files to cache

    # start a "mainnet" replica which host the onchain canisters
    dfx_start

    install_asset deps

    # start a webserver to host wasm files
    mkdir www
    start_webserver --directory www

    cd onchain

    dfx canister create a --specified-id "$CANISTER_ID_A"
    dfx canister create b --specified-id "$CANISTER_ID_B"
    dfx canister create c --specified-id "$CANISTER_ID_C"
    dfx build

    cd .dfx/local/canisters
    ic-wasm a/a.wasm -o a/a.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/a.wasm" -v public
    ic-wasm b/b.wasm -o b/b.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/b.wasm" -v public
    ic-wasm c/c.wasm -o c/c.wasm metadata "dfx:wasm_url" -d "http://localhost:$E2E_WEB_SERVER_PORT/c.wasm" -v public
    ic-wasm a/a.wasm -o a/a_custom.wasm metadata "hello" -d "world" -v public
    ic-wasm a/a.wasm -o a/a.wasm metadata "dfx:wasm_hash" -d "$(sha256sum a/a_custom.wasm | cut -d " " -f 1)" -v public

    cd ../../../
    dfx canister install a --argument 1
    dfx canister install b
    dfx canister install c --argument 3

    # copy wasm files to web server dir
    cp .dfx/local/canisters/a/a_custom.wasm ../www/a.wasm
    cp .dfx/local/canisters/b/b.wasm ../www/b.wasm
    cp .dfx/local/canisters/c/c.wasm ../www/c.wasm

    # pull canisters in app project
    cd ../app
    assert_file_not_exists "deps/pulled.json"

    assert_command dfx deps pull --network local -vvv
    assert_contains "Canister $CANISTER_ID_A specified a custom hash:"
}

@test "dfx deps init works" {
    use_test_specific_cache_root # dfx deps pull will download files to cache

    # start a "mainnet" replica which host the onchain canisters
    dfx_start

    setup_onchain

    # pull canisters in app project
    cd app
    assert_command dfx deps pull --network local

    # stop the "mainnet" replica
    dfx_stop

    assert_command dfx deps init
    assert_contains "The following canister(s) require an init argument. Please run \`dfx deps init <NAME/PRINCIPAL>\` to set them individually:"
    assert_contains "$CANISTER_ID_A"
    assert_contains "$CANISTER_ID_C (dep_c)"

    assert_command dfx deps init "$CANISTER_ID_A" --argument 11
    assert_command dfx deps init dep_c --argument 33

    # The argument is the hex string of '("abc")' which doesn't type check
    # However, passing raw argument will bypass the type check so following command succeed
    assert_command dfx deps init "$CANISTER_ID_A" --argument "4449444c00017103616263" --argument-type raw
    assert_command jq -r '.canisters."'"$CANISTER_ID_A"'".arg_raw' deps/init.json
    assert_match "4449444c00017103616263"

    # error cases
    assert_command_fail dfx deps init "$CANISTER_ID_A"
    assert_contains "Canister $CANISTER_ID_A requires an init argument"

    assert_command_fail dfx deps init "$CANISTER_ID_A" --argument '("abc")'
    assert_contains "Invalid data: Unable to serialize Candid values: type mismatch: \"abc\" cannot be of type nat"

    assert_command_fail dfx deps init dep_b --argument 1
    assert_contains "Canister $CANISTER_ID_B (dep_b) takes no init argument. Please rerun without \`--argument\`"

    assert_command_fail dfx deps init "$CANISTER_ID_C"
    assert_contains "Canister $CANISTER_ID_C (dep_c) requires an init argument. The following info might be helpful:
dfx:init => Nat
candid:args => (nat)"
}

@test "dfx deps deploy works" {
    # ic-ref have a different behavior than the repilca:
    #    once a canister has been deleted, it cannot be created again.
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    use_test_specific_cache_root # dfx deps pull will download files to cache

    # start a "mainnet" replica which host the onchain canisters
    dfx_start

    setup_onchain

    # pull canisters in app project
    cd app
    assert_command dfx deps pull --network local

    # delete onchain canisters so that the replica has no canisters as a clean local replica
    cd ../onchain
    dfx canister stop a
    dfx canister delete a
    dfx canister stop b
    dfx canister delete b
    dfx canister stop c
    dfx canister delete c

    cd ../app
    assert_command dfx deps init # b is set here
    assert_command dfx deps init "$CANISTER_ID_A" --argument 11
    assert_command dfx deps init "$CANISTER_ID_C" --argument 33

    # deploy all
    assert_command dfx deps deploy
    assert_contains "Creating canister: $CANISTER_ID_A
Installing canister: $CANISTER_ID_A"
    assert_contains "Creating canister: $CANISTER_ID_B (dep_b)
Installing canister: $CANISTER_ID_B (dep_b)"
    assert_contains "Creating canister: $CANISTER_ID_C (dep_c)
Installing canister: $CANISTER_ID_C (dep_c)"

    # by name in dfx.json
    assert_command dfx deps deploy dep_b
    assert_contains "Creating canister: $CANISTER_ID_B (dep_b)
Installing canister: $CANISTER_ID_B (dep_b)"

    # by canister id
    assert_command dfx deps deploy $CANISTER_ID_A
    assert_contains "Creating canister: $CANISTER_ID_A
Installing canister: $CANISTER_ID_A"

    # deployed pulleds dependencies can be stopped and deleted
    assert_command dfx canister stop dep_b --identity anonymous
    assert_command dfx canister delete dep_b --identity anonymous

    assert_command dfx canister stop $CANISTER_ID_A --identity anonymous
    assert_command dfx canister delete $CANISTER_ID_A --identity anonymous

    # error cases
    ## set wrong init argument
    assert_command dfx deps init "$CANISTER_ID_A" --argument "4449444c00017103616263" --argument-type raw
    assert_command_fail dfx deps deploy
    assert_contains "Failed to install canister $CANISTER_ID_A"

    ## no init.json
    rm deps/init.json
    assert_command_fail dfx deps deploy
    assert_contains "Failed to read init.json"

    ## forgot to set init argument for some dependencies
    assert_command dfx deps init # b is set here
    assert_command_fail dfx deps deploy "$CANISTER_ID_A"
    assert_contains "Failed to create and install canister $CANISTER_ID_A"
    assert_contains "Failed to find $CANISTER_ID_A entry in init.json. Please run \`dfx deps init $CANISTER_ID_A\`."
}

@test "dfx deps pulled dependencies work with app canister" {
    # ic-ref have a different behavior than the repilca:
    #    once a canister has been deleted, it cannot be created again.
    [ "$USE_IC_REF" ] && skip "skipped for ic-ref"

    use_test_specific_cache_root # dfx deps pull will download files to cache

    # start a "mainnet" replica which host the onchain canisters
    dfx_start

    setup_onchain

    # pull canisters in app project
    cd app
    assert_command dfx deps pull --network local

    # delete onchain canisters so that the replica has no canisters as a clean local replica
    cd ../onchain
    dfx canister stop a
    dfx canister delete a
    dfx canister stop b
    dfx canister delete b
    dfx canister stop c
    dfx canister delete c

    cd ../app
    assert_command_fail dfx canister create dep_b
    assert_contains "dep_b is a pull dependency. Please deploy it using \`dfx deps deploy dep_b\`"
    assert_command dfx canister create app

    assert_command dfx canister create --all
    assert_contains "There are pull dependencies defined in dfx.json. Please deploy them using \`dfx deps deploy\`."

    assert_command dfx build app
    assert_command dfx canister install app

    # pulled dependency dep_b hasn't been deployed on local replica
    assert_command_fail dfx canister call app get_b
    assert_contains "Canister $CANISTER_ID_B not found" 

    assert_command dfx deps init
    assert_command dfx deps init "$CANISTER_ID_A" --argument 11
    assert_command dfx deps init "$CANISTER_ID_C" --argument 33
    assert_command dfx deps deploy

    assert_command dfx canister call app get_b
    assert_match "(2 : nat)"
    assert_command dfx canister call app get_c
    assert_match "(33 : nat)" # corresponding to "--argument 33" above

    # start a clean local replica
    dfx canister stop app
    dfx canister delete app
    assert_command dfx deploy # only deploy app canister
}

@test "dfx deps do nothing in a project has no pull dependencies" {
    dfx_new empty

    # verify the help message
    assert_command dfx deps pull -h
    assert_contains "Pull canisters upon which the project depends. This command connects to the \"ic\" mainnet by default.
You can still choose other network by setting \`--network\`"

    assert_command dfx deps pull
    assert_match "There are no pull dependencies defined in dfx.json"
    assert_command dfx deps init
    assert_match "There are no pull dependencies defined in dfx.json"
    assert_command dfx deps deploy
    assert_match "There are no pull dependencies defined in dfx.json"
    
    assert_directory_not_exists "deps"
}
