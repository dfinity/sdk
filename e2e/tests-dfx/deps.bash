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

    jq '.canisters.a.pullable.wasm_url="'"http://localhost:$E2E_WEB_SERVER_PORT/a.wasm"'"' dfx.json | sponge dfx.json
    jq '.canisters.b.pullable.wasm_url="'"http://localhost:$E2E_WEB_SERVER_PORT/b.wasm.gz"'"' dfx.json | sponge dfx.json
    jq '.canisters.c.pullable.wasm_url="'"http://localhost:$E2E_WEB_SERVER_PORT/c.wasm"'"' dfx.json | sponge dfx.json

    dfx canister create a --specified-id "$CANISTER_ID_A"
    dfx canister create b --specified-id "$CANISTER_ID_B"
    dfx canister create c --specified-id "$CANISTER_ID_C"
    dfx build

    dfx canister install a --argument 1
    dfx canister install b
    dfx canister install c --argument 3

    # copy wasm files to web server dir
    cp .dfx/local/canisters/a/a.wasm ../www/a.wasm
    cp .dfx/local/canisters/b/b.wasm.gz ../www/b.wasm.gz
    cp .dfx/local/canisters/c/c.wasm ../www/c.wasm

    cd .. || exit
}

@test "dfx build can write required metadata for pullable" {
    dfx_start

    install_asset deps

    cd onchain
    assert_command dfx canister create --all
    assert_command dfx build
    assert_command ic-wasm .dfx/local/canisters/c/c.wasm metadata
    assert_match "icp:public candid:service"
    assert_match "icp:public dfx"

    ic-wasm .dfx/local/canisters/c/c.wasm metadata dfx > c_dfx.json
    assert_command jq -r '.pullable.wasm_url' c_dfx.json
    assert_eq "http://example.com/c.wasm" "$output"
    assert_command jq -r '.pullable.dependencies | length' c_dfx.json
    assert_eq 1 "$output"
    assert_command jq -r '.pullable.dependencies | first' c_dfx.json
    assert_eq "$CANISTER_ID_A" "$output"
    assert_command jq -r '.pullable.init_guide' c_dfx.json
    assert_eq "A natural number, e.g. 20." "$output"
}

@test "dfx build can handle dynamic_wasm_url" {
    dfx_start

    install_asset deps

    cd onchain
    jq '.canisters.a.pullable.dynamic_wasm_url.generate="bash dynamic_wasm_url_a.sh"' dfx.json | sponge dfx.json
    jq '.canisters.a.pullable.dynamic_wasm_url.path="dynamic_wasm_url.txt"' dfx.json | sponge dfx.json

    echo "echo \"http:/example.com/dynamic_wasm_url/a.wasm\" > dynamic_wasm_url.txt" > dynamic_wasm_url_a.sh

    assert_command dfx canister create --all
    assert_command_fail dfx build a
    assert_contains "Cannot define \`wasm_url\` and \`dynamic_wasm_url\` at the same time."

    jq 'del(.canisters.a.pullable.wasm_url)' dfx.json | sponge dfx.json
    assert_command dfx build a

    ic-wasm .dfx/local/canisters/a/a.wasm metadata dfx > a_dfx.json
    assert_command jq -r '.pullable.wasm_url' a_dfx.json
    assert_eq "http:/example.com/dynamic_wasm_url/a.wasm" "$output"  
}

@test "dfx build can handle custom_wasm" {
    dfx_start

    install_asset deps

    cd onchain

    echo -n -e "\x00asm\x01\x00\x00\x00" > main.wasm
    WASM_HASH="$(sha256sum main.wasm | cut -d " " -f 1)"
    echo "$WASM_HASH" > wasm_hash.txt
    
    assert_command dfx canister create --all

    # Cannot define multiple fields for wasm_hash
    jq '.canisters.a.pullable.wasm_hash="'"$WASM_HASH"'"' dfx.json | sponge dfx.json
    jq '.canisters.a.pullable.wasm_hash_file="wasm_hash.txt"' dfx.json | sponge dfx.json
    assert_command_fail dfx build a
    assert_contains "Pullable canister can only define one of \`wasm_hash\`, \`wasm_hash_file\`, \`custom_wasm\`."

    # Read wasm_hash from a file
    jq 'del(.canisters.a.pullable.wasm_hash)' dfx.json | sponge dfx.json
    assert_command dfx build a
    ic-wasm .dfx/local/canisters/a/a.wasm metadata dfx > a_dfx.json
    assert_command jq -r '.pullable.wasm_hash' a_dfx.json
    assert_eq "$WASM_HASH" "$output"

    # Generate the custom wasm, post-process it and calculate wasm_hash
    jq 'del(.canisters.a.pullable.wasm_hash_file)' dfx.json | sponge dfx.json
    jq '.canisters.a.pullable.custom_wasm.generate="cp main.wasm custom.wasm"' dfx.json | sponge dfx.json
    jq '.canisters.a.pullable.custom_wasm.path="custom.wasm"' dfx.json | sponge dfx.json

    assert_command dfx build a
    assert_contains "Custom wasm for pullable at:"
    assert_contains ".dfx/local/canisters/a/a_custom.wasm"
    PROCESSED_WASM_HASH="$(sha256sum .dfx/local/canisters/a/a_custom.wasm | cut -d " " -f 1)"
    assert_contains "wasm_hash: $PROCESSED_WASM_HASH"

    ic-wasm .dfx/local/canisters/a/a.wasm metadata dfx > a_dfx.json
    assert_command jq -r '.pullable.wasm_hash' a_dfx.json
    assert_eq "$PROCESSED_WASM_HASH" "$output"

    assert_command ic-wasm .dfx/local/canisters/a/a.wasm metadata
    assert_contains "icp:public dfx"
    assert_contains "icp:public candid:service"
    assert_contains "icp:public candid:args"
}

@test "dfx deps pull can resolve dependencies from on-chain canister metadata" {
    # ic-ref has different behavior than the replica:
    #   it doesn't differ whether the canister not exist or the metadata not exist
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

    ## 1.2. pull onchain canisters in "app" project
    cd ../app

    assert_command_fail dfx deps pull --network local # the overall pull fail but succeed to fetch and parse `dfx:deps` recursively
    assert_contains "Fetching dependencies of canister $CANISTER_ID_B...
Fetching dependencies of canister $CANISTER_ID_C...
Fetching dependencies of canister $CANISTER_ID_A...
Found 3 dependencies:
$CANISTER_ID_A
$CANISTER_ID_B
$CANISTER_ID_C"
    assert_occurs 1 "Fetching dependencies of canister $CANISTER_ID_A..." # common dependency onchain_a is pulled only once
    assert_contains "Pulling canister $CANISTER_ID_A...
ERROR: Failed to pull canister $CANISTER_ID_A.
Failed to download from url: http://example.com/a.wasm."
    assert_contains "Pulling canister $CANISTER_ID_B...
ERROR: Failed to pull canister $CANISTER_ID_B.
Failed to download from url: http://example.com/b.wasm.gz."
    assert_contains "Pulling canister $CANISTER_ID_C...
ERROR: Failed to pull canister $CANISTER_ID_C.
Failed to download from url: http://example.com/c.wasm."

    # 3. sad path: if the canister is not present on-chain
    cd ../onchain
    dfx build c
    dfx canister install c --argument 3 --mode=reinstall --yes # reinstall the correct canister c
    dfx canister uninstall-code a

    cd ../app
    assert_command_fail dfx deps pull --network local
    assert_contains "Failed to get dependencies of canister $CANISTER_ID_A."

    cd ../onchain
    dfx canister stop a
    dfx canister delete a

    cd ../app
    assert_command_fail dfx deps pull --network local
    assert_contains "Failed to get dependencies of canister $CANISTER_ID_A."
}

@test "dfx deps pull can download wasm and candids to shared cache and generate pulled.json" {
    use_test_specific_cache_root # dfx deps pull will download files to cache

    PULLED_DIR="$DFX_CACHE_ROOT/.cache/dfinity/pulled/"
    assert_file_not_exists "$PULLED_DIR/$CANISTER_ID_B/canister.wasm.gz"
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
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_B/canister.wasm.gz"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_A/canister.wasm"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_C/canister.wasm"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_B/service.did"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_A/service.did"
    assert_file_exists "$PULLED_DIR/$CANISTER_ID_C/service.did"

    cd deps
    assert_file_exists "pulled.json"
    assert_file_exists "candid/$CANISTER_ID_B.did"
    assert_file_exists "candid/$CANISTER_ID_C.did"
    assert_eq 5 "$(jq -r '.canisters | keys' pulled.json | wc -l | tr -d ' ')" # 3 canisters + 2 lines of '[' and ']'
    assert_command jq -r '.canisters."'"$CANISTER_ID_A"'".init_guide' pulled.json
    assert_eq "A natural number, e.g. 10." "$output"
    assert_command jq -r '.canisters."'"$CANISTER_ID_B"'".name' pulled.json
    assert_eq "dep_b" "$output"
    assert_command jq -r '.canisters."'"$CANISTER_ID_C"'".name' pulled.json
    assert_eq "dep_c" "$output"
    cd ../

    assert_command dfx deps pull --network local -vvv
    assert_contains "The canister wasm was found in the cache." # cache hit

    # sad path 1: wasm hash doesn't match on chain
    rm -r "${PULLED_DIR:?}/"
    cd ../onchain
    cp .dfx/local/canisters/c/c.wasm ../www/a.wasm 

    cd ../app
    assert_command_fail dfx deps pull --network local
    assert_contains "Failed to pull canister $CANISTER_ID_A."
    assert_contains "Hash mismatch."

    # sad path 2: url server doesn't have the file
    rm -r "${PULLED_DIR:?}/"
    rm ../www/a.wasm

    assert_command_fail dfx deps pull --network local
    assert_contains "Failed to pull canister $CANISTER_ID_A."
    assert_contains "Failed to download from url:"
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

    jq '.canisters.a.pullable.wasm_url="'"http://localhost:$E2E_WEB_SERVER_PORT/a.wasm"'"' dfx.json | sponge dfx.json
    jq '.canisters.b.pullable.wasm_url="'"http://localhost:$E2E_WEB_SERVER_PORT/b.wasm.gz"'"' dfx.json | sponge dfx.json
    jq '.canisters.c.pullable.wasm_url="'"http://localhost:$E2E_WEB_SERVER_PORT/c.wasm"'"' dfx.json | sponge dfx.json

    dfx canister create a --specified-id "$CANISTER_ID_A"
    dfx canister create b --specified-id "$CANISTER_ID_B"
    dfx canister create c --specified-id "$CANISTER_ID_C"
    dfx build

    # copy wasm files to web server dir
    cp .dfx/local/canisters/a/a.wasm ../www/a.wasm
    cp .dfx/local/canisters/b/b.wasm.gz ../www/b.wasm.gz
    cp .dfx/local/canisters/c/c.wasm ../www/c.wasm

    CUSTOM_HASH="$(sha256sum .dfx/local/canisters/a/a.wasm | cut -d " " -f 1)"
    jq '.canisters.a.pullable.wasm_hash="'"$CUSTOM_HASH"'"' dfx.json | sponge dfx.json
    dfx build a # .dfx/local/canisters/a/a.wasm is replaced. The new wasm has wasm_hash defined and will be installed.

    # cd ../../../
    dfx canister install a --argument 1
    dfx canister install b
    dfx canister install c --argument 3

    # pull canisters in app project
    cd ../app
    assert_file_not_exists "deps/pulled.json"

    assert_command dfx deps pull --network local -vvv
    assert_contains "Canister $CANISTER_ID_A specified a custom hash:"

    # error case: hash mismatch
    PULLED_DIR="$DFX_CACHE_ROOT/.cache/dfinity/pulled/"
    rm -r "${PULLED_DIR:?}/"
    cd ../onchain
    cp .dfx/local/canisters/a/a.wasm ../www/a.wasm # now the webserver has the onchain version of canister_a which won't match wasm_hash

    cd ../app
    assert_command_fail dfx deps pull --network local -vvv
    assert_contains "Canister $CANISTER_ID_A specified a custom hash:"
    assert_contains "Failed to pull canister $CANISTER_ID_A."
    assert_contains "Hash mismatch."
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
    assert_eq "4449444c00017103616263" "$output"

    # error cases
    ## require init arguments but not provide
    assert_command_fail dfx deps init "$CANISTER_ID_A"
    assert_contains "Canister $CANISTER_ID_A requires an init argument"

    ## wrong type
    assert_command_fail dfx deps init "$CANISTER_ID_A" --argument '("abc")'
    assert_contains "Invalid data: Unable to serialize Candid values: type mismatch: \"abc\" cannot be of type nat"

    ## require no init argument but provide
    assert_command_fail dfx deps init dep_b --argument 1
    assert_contains "Canister $CANISTER_ID_B (dep_b) takes no init argument. Please rerun without \`--argument\`"

    ## require init arguments but not provide
    assert_command_fail dfx deps init dep_c
    assert_contains "Canister $CANISTER_ID_C (dep_c) requires an init argument. The following info might be helpful:
init => A natural number, e.g. 20.
candid:args => (nat)"

    ## canister ID not in pulled.json
    assert_command_fail dfx deps init aaaaa-aa
    assert_contains "Could not find aaaaa-aa in pulled.json"
}

@test "dfx deps deploy works" {
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
    assert_contains "Installing canister: $CANISTER_ID_B (dep_b)" # dep_p has been created before, so we only see "Installing ..." here

    # by canister id
    assert_command dfx deps deploy $CANISTER_ID_A
    assert_contains "Installing canister: $CANISTER_ID_A"

    # deployed pull dependencies can be stopped and deleted
    assert_command dfx canister stop dep_b --identity anonymous
    assert_command dfx canister delete dep_b --identity anonymous

    assert_command dfx canister stop $CANISTER_ID_A --identity anonymous
    assert_command dfx canister delete $CANISTER_ID_A --identity anonymous

    # error cases
    ## set wrong init argument
    assert_command dfx deps init "$CANISTER_ID_A" --argument "4449444c00017103616263" --argument-type raw
    assert_command_fail dfx deps deploy
    assert_contains "Failed to install canister $CANISTER_ID_A"

    ## canister ID not in pulled.json
    assert_command_fail dfx deps deploy aaaaa-aa
    assert_contains "Could not find aaaaa-aa in pulled.json"

    ## no init.json
    rm deps/init.json
    assert_command_fail dfx deps deploy
    assert_contains "Failed to read init.json. Please run \`dfx deps init\`."

    ## forgot to set init argument for some dependencies
    assert_command dfx deps init # b is set here
    assert_command_fail dfx deps deploy "$CANISTER_ID_A"
    assert_contains "Failed to create and install canister $CANISTER_ID_A"
    assert_contains "Failed to find $CANISTER_ID_A entry in init.json. Please run \`dfx deps init $CANISTER_ID_A\`."
}

@test "dfx deps pulled dependencies work with app canister" {
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
    assert_eq "(2 : nat)" "$output"
    assert_command dfx canister call app get_c
    assert_eq "(33 : nat)" "$output" # corresponding to "--argument 33" above
    assert_command dfx canister call app get_b_times_a
    assert_eq "(22 : nat)" "$output" # 2 * 11
    assert_command dfx canister call app get_c_times_a
    assert_eq "(363 : nat)" "$output" # 33 * 11

    # start a clean local replica
    dfx canister stop app
    dfx canister delete app
    assert_command dfx deploy # only deploy app canister
}

@test "dfx deps does nothing in a project has no pull dependencies" {
    dfx_new empty

    # verify the help message
    assert_command dfx deps pull -h
    assert_contains "Pull canisters upon which the project depends. This command connects to the \"ic\" mainnet by default.
You can still choose other network by setting \`--network\`"

    assert_command dfx deps pull
    assert_contains "There are no pull dependencies defined in dfx.json"
    assert_command dfx deps init
    assert_contains "There are no pull dependencies defined in dfx.json"
    assert_command dfx deps deploy
    assert_contains "There are no pull dependencies defined in dfx.json"
    
    assert_directory_not_exists "deps"
}
