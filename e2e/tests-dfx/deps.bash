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

# only execute in project root (deps)
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
  dfx canister install c --argument "(opt 3)"

  # copy wasm files to web server dir
  cp .dfx/local/canisters/a/a.wasm ../www/a.wasm
  cp .dfx/local/canisters/b/b.wasm.gz ../www/b.wasm.gz
  cp .dfx/local/canisters/c/c.wasm ../www/c.wasm

  cd .. || exit
}

# only execute in project root (deps)
cleanup_onchain() {
  cd onchain || exit
  dfx canister stop a
  dfx canister delete a --no-withdrawal
  dfx canister stop b
  dfx canister delete b --no-withdrawal
  dfx canister stop c
  dfx canister delete c --no-withdrawal
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
  assert_eq "http://httpbin.org/status/404" "$output"
  assert_command jq -r '.pullable.dependencies | length' c_dfx.json
  assert_eq 1 "$output"
  assert_command jq -r '.pullable.dependencies | first' c_dfx.json
  assert_eq "$CANISTER_ID_A" "$output"
  assert_command jq -r '.pullable.init_guide' c_dfx.json
  assert_eq "An optional natural number, e.g. \"(opt 20)\"." "$output"
}

@test "dfx deps pull can resolve dependencies from on-chain canister metadata" {
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
  dfx deploy c

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
Failed to download from url: http://httpbin.org/status/404."
  assert_contains "Pulling canister $CANISTER_ID_B...
ERROR: Failed to pull canister $CANISTER_ID_B.
Failed to download from url: http://httpbin.org/status/404."
  assert_contains "Pulling canister $CANISTER_ID_C...
ERROR: Failed to pull canister $CANISTER_ID_C.
Failed to download from url: http://httpbin.org/status/404."

  # 3. sad path: if the canister is not present on-chain
  cd ../onchain
  dfx build c
  dfx canister install c --mode=reinstall --yes # reinstall the correct canister c
  dfx canister uninstall-code a

  cd ../app
  assert_command_fail dfx deps pull --network local
  assert_contains "Failed to get dependencies of canister $CANISTER_ID_A."

  cd ../onchain
  dfx canister stop a
  dfx canister delete a --no-withdrawal

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

  # hash mismatch is ok
  # the expected hash is written to pulled.json wasm_hash field
  # the hash of downloaded wasm is written to pulled.json wasm_hash_download field
  rm -r "${PULLED_DIR:?}/"
  cd ../onchain
  cp .dfx/local/canisters/c/c.wasm ../www/a.wasm # we will get wasm of canister_c when pulling canister_a
  WASM_HASH_A="$(sha256sum .dfx/local/canisters/a/a.wasm | cut -d " " -f 1)"
  WASM_HASH_DOWNLOAD_A="$(sha256sum .dfx/local/canisters/c/c.wasm | cut -d " " -f 1)"

  cd ../app
  assert_command dfx deps pull --network local
  assert_not_contains "WARN"
  assert_command jq -r '.canisters."'"$CANISTER_ID_A"'".wasm_hash' deps/pulled.json
  assert_match "$WASM_HASH_A" "$output"
  assert_command jq -r '.canisters."'"$CANISTER_ID_A"'".wasm_hash_download' deps/pulled.json
  assert_match "$WASM_HASH_DOWNLOAD_A" "$output"

  # sad path: url server doesn't have the file
  rm -r "${PULLED_DIR:?}/"
  rm ../www/a.wasm

  assert_command_fail dfx deps pull --network local
  assert_contains "Failed to pull canister $CANISTER_ID_A."
  assert_contains "Failed to download from url:"
}

@test "dfx deps pull works when wasm_hash or wasm_hash_url specified" {
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

  # A: set dfx:wasm_hash
  CUSTOM_HASH_A="$(sha256sum .dfx/local/canisters/a/a.wasm | cut -d " " -f 1)"
  jq '.canisters.a.pullable.wasm_hash="'"$CUSTOM_HASH_A"'"' dfx.json | sponge dfx.json
  # B: set dfx:wasm_hash_url with output of sha256sum
  echo -n "$(sha256sum .dfx/local/canisters/b/b.wasm.gz)" > ../www/b.wasm.gz.sha256
  jq '.canisters.b.pullable.wasm_hash_url="'"http://localhost:$E2E_WEB_SERVER_PORT/b.wasm.gz.sha256"'"' dfx.json | sponge dfx.json
  # C: set dfx:wasm_hash_url with the hash only
  CUSTOM_HASH_C="$(sha256sum .dfx/local/canisters/c/c.wasm | cut -d " " -f 1)"
  echo -n "$CUSTOM_HASH_C" > ../www/c.wasm.sha256
  jq '.canisters.c.pullable.wasm_hash_url="'"http://localhost:$E2E_WEB_SERVER_PORT/c.wasm.sha256"'"' dfx.json | sponge dfx.json

  dfx build

  dfx canister install a --argument 1
  dfx canister install b
  dfx canister install c

  # pull canisters in app project
  cd ../app
  assert_file_not_exists "deps/pulled.json"

  assert_command dfx deps pull --network local -vvv
  assert_contains "Canister $CANISTER_ID_A specified a custom hash:"
  assert_contains "Canister $CANISTER_ID_B specified a custom hash via url:"
  assert_contains "Canister $CANISTER_ID_C specified a custom hash via url:"
 
  # warning: specified both `wasm_hash` and `wasm_hash_url`. Providers should avoid this.
  PULLED_DIR="$DFX_CACHE_ROOT/.cache/dfinity/pulled/"
  rm -r "${PULLED_DIR:?}/"
  cd ../onchain
  jq '.canisters.c.pullable.wasm_hash="'"$CUSTOM_HASH_C"'"' dfx.json | sponge dfx.json
  dfx build
  dfx canister install c --mode=reinstall --yes

  cd ../app
  assert_command dfx deps pull --network local -vvv
  assert_contains "WARN: Canister $CANISTER_ID_C specified both \`wasm_hash\` and \`wasm_hash_url\`. \`wasm_hash\` will be used."

  # hash mismatch is ok
  rm -r "${PULLED_DIR:?}/"
  cd ../onchain
  cp .dfx/local/canisters/a/a.wasm ../www/a.wasm # now the webserver has the onchain version of canister_a which won't match wasm_hash

  cd ../app
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
  assert_contains "Canister $CANISTER_ID_C (dep_c) set init argument with \"(null)\"."
  assert_contains "WARN: The following canister(s) require an init argument. Please run \`dfx deps init <NAME/PRINCIPAL>\` to set them individually:
$CANISTER_ID_A"

  assert_command dfx deps init "$CANISTER_ID_A" --argument 11

  # dep_c requires an init argument with top-level opt
  # without --argument, it will try to set "(null)"
  assert_command dfx deps init dep_c
  
  # overwrite the empty argument with a valid one
  assert_command dfx deps init dep_c --argument "(opt 33)"

  # can also set with --argument-file
  echo "(opt 44)" > arg.txt
  assert_command dfx deps init dep_c --argument-file arg.txt

  # The argument is the hex string of '("abc")' which doesn't type check
  # However, passing raw argument will bypass the type check so following command succeed
  assert_command dfx deps init "$CANISTER_ID_A" --argument "4449444c00017103616263" --argument-type raw
  assert_command jq -r '.canisters."'"$CANISTER_ID_A"'".arg_raw' deps/init.json
  assert_eq "4449444c00017103616263" "$output"

  # Canister A has been set, set again without --argument will prompt a info message
  assert_command dfx deps init "$CANISTER_ID_A"
  assert_contains "Canister $CANISTER_ID_A already set init argument."
  
  # error cases
  rm deps/init.json
  ## require init arguments but not provide
  assert_command_fail dfx deps init "$CANISTER_ID_A"
  assert_contains "Canister $CANISTER_ID_A requires an init argument. The following info might be helpful:
init_guide => A natural number, e.g. 10.
candid:args => (nat)"

  ## wrong type
  assert_command_fail dfx deps init "$CANISTER_ID_A" --argument '("abc")'
  assert_contains "Invalid data: Unable to serialize Candid values: type mismatch: \"abc\" cannot be of type nat"

  ## require no init argument but provide
  assert_command_fail dfx deps init dep_b --argument 1
  assert_contains "Canister $CANISTER_ID_B (dep_b) takes no init argument. Please rerun without \`--argument\`"

  ## canister ID not in pulled.json
  assert_command_fail dfx deps init aaaaa-aa
  assert_contains "Could not find aaaaa-aa in pulled.json"
}

@test "dfx deps init can handle init_arg in pullable metadata" {
  use_test_specific_cache_root # dfx deps pull will download files to cache

  # start a "mainnet" replica which host the onchain canisters
  dfx_start

  setup_onchain
  cd onchain
  # Canister A: set init_arg in pullable metadata then redeploy and copy wasm file to web server dir
  jq '.canisters.a.pullable.init_arg="42"' dfx.json | sponge dfx.json
  dfx build a
  dfx canister install a --argument 1 --mode=reinstall --yes
  cp .dfx/local/canisters/a/a.wasm ../www/a.wasm

  # pull canisters in app project
  cd ../app
  assert_command dfx deps pull --network local

  # stop the "mainnet" replica
  dfx_stop

  assert_command dfx deps init
  assert_command jq -r '.canisters."'"$CANISTER_ID_A"'".arg_str' deps/init.json
  assert_match "42" "$output" # This matches the init_arg which was set above

  # Explicitly set with --argument can overwrite
  assert_command dfx deps init "$CANISTER_ID_A" --argument 37
  assert_command jq -r '.canisters."'"$CANISTER_ID_A"'".arg_str' deps/init.json
  assert_match "37" "$output"
}

@test "dfx deps init errors when init_arg in pullable metadata has wrong type" {
  use_test_specific_cache_root # dfx deps pull will download files to cache

  # start a "mainnet" replica which host the onchain canisters
  dfx_start

  setup_onchain
  cd onchain
  # Canister A: set init_arg in pullable metadata then redeploy and copy wasm file to web server dir
  jq '.canisters.a.pullable.init_arg="(\"abc\")"' dfx.json | sponge dfx.json
  dfx build a
  dfx canister install a --argument 1 --mode=reinstall --yes
  cp .dfx/local/canisters/a/a.wasm ../www/a.wasm

  # pull canisters in app project
  cd ../app
  assert_command dfx deps pull --network local

  # stop the "mainnet" replica
  dfx_stop

  assert_command_fail dfx deps init "$CANISTER_ID_A"
  assert_contains "Pulled canister $CANISTER_ID_A provided an invalid \`init_arg\`.
Please try to set an init argument with \`--argument\` option.
The following info might be helpful:
init_guide => A natural number, e.g. 10.
candid:args => (nat)"

  # Consumer set correct init_arg
  assert_command dfx deps init "$CANISTER_ID_A" --argument 10
  assert_command jq -r '.canisters."'"$CANISTER_ID_A"'".arg_str' deps/init.json
  assert_match "10" "$output"
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
  cd ../
  cleanup_onchain

  cd app
  assert_command dfx deps init # b is set here
  assert_command dfx deps init "$CANISTER_ID_A" --argument 11
  assert_command dfx deps init "$CANISTER_ID_C" --argument "(opt 33)"

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
  assert_command dfx canister delete dep_b --identity anonymous --no-withdrawal

  assert_command dfx canister stop $CANISTER_ID_A --identity anonymous
  assert_command dfx canister delete $CANISTER_ID_A --identity anonymous --no-withdrawal

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

@test "dfx deps init/deploy works when hash mismatch" {
  use_test_specific_cache_root # dfx deps pull will download files to cache

  # start a "mainnet" replica which host the onchain canisters
  dfx_start

  setup_onchain
  cd onchain
  cp .dfx/local/canisters/c/c.wasm ../www/a.wasm # we will get wasm of canister_c when pulling canister_a

  # pull canisters in app project
  cd ../app
  assert_command dfx deps pull --network local

  # delete onchain canisters so that the replica has no canisters as a clean local replica
  cd ../
  cleanup_onchain

  cd app
  assert_command dfx deps init # b is set here
  assert_command dfx deps init "$CANISTER_ID_A" --argument "(opt 11)" # the downloaded wasm need argument type as canister_c
  assert_command dfx deps init "$CANISTER_ID_C" --argument "(opt 33)"  

  # deploy all
  assert_command dfx deps deploy
}

@test "dfx deps init/deploy abort when pulled.json and cache are invalid" {
  use_test_specific_cache_root # dfx deps pull will download files to cache

  # start a "mainnet" replica which host the onchain canisters
  dfx_start

  setup_onchain

  # pull canisters in app project
  cd app
  assert_command dfx deps pull --network local
  cp deps/pulled.json deps/pulled.json.bak

  # 1. `pulled.json` is not consistent with `dfx.json`

  ## 1.1. missing pull dependency in pulled.json
  jq 'del(.canisters."'"$CANISTER_ID_B"'")' deps/pulled.json.bak > deps/pulled.json
  assert_command_fail dfx deps init
  assert_contains "Failed to find dep_b:$CANISTER_ID_B in pulled.json."
  assert_contains "Please rerun \`dfx deps pull\`."
  assert_command_fail dfx deps deploy
  assert_contains "Failed to find dep_b:$CANISTER_ID_B in pulled.json."
  assert_contains "Please rerun \`dfx deps pull\`."

  ## 1.2. name mismatch in pulled.json and dfx.json
  jq '.canisters."'"$CANISTER_ID_B"'".name="not_dep_b"' deps/pulled.json.bak > deps/pulled.json
  assert_command_fail dfx deps init
  assert_contains "$CANISTER_ID_B is \"dep_b\" in dfx.json, but it has name \"not_dep_b\" in pulled.json."
  assert_command_fail dfx deps deploy
  assert_contains "$CANISTER_ID_B is \"dep_b\" in dfx.json, but it has name \"not_dep_b\" in pulled.json."

  ## 1.3. no name in pulled.json
  jq 'del(.canisters."'"$CANISTER_ID_B"'".name)' deps/pulled.json.bak > deps/pulled.json
  assert_command_fail dfx deps init
  assert_contains "$CANISTER_ID_B is \"dep_b\" in dfx.json, but it doesn't have name in pulled.json."
  assert_command_fail dfx deps deploy
  assert_contains "$CANISTER_ID_B is \"dep_b\" in dfx.json, but it doesn't have name in pulled.json."

  cp deps/pulled.json.bak deps/pulled.json

  # 2. the wasm modules in pulled cache are not consistent with `pulled.json`

  ## 2.1. missing wasm in cache
  WASM_PATH_A="$DFX_CACHE_ROOT/.cache/dfinity/pulled/$CANISTER_ID_A/canister.wasm"
  mv "$WASM_PATH_A" "$WASM_PATH_A.bak"
  assert_command_fail dfx deps init
  assert_contains "failed to read from $WASM_PATH_A"
  assert_command_fail dfx deps deploy
  assert_contains "failed to read from $WASM_PATH_A"
  mv "$WASM_PATH_A.bak" "$WASM_PATH_A"

  ## 2.2. wasm_hash_download is not valid hex string
  jq '.canisters."'"$CANISTER_ID_B"'".wasm_hash_download="xyz"' deps/pulled.json.bak > deps/pulled.json
  assert_command_fail dfx deps init
  assert_contains "In pulled.json, the \`wasm_hash_download\` field of $CANISTER_ID_B is invalid."
  assert_command_fail dfx deps deploy
  assert_contains "In pulled.json, the \`wasm_hash_download\` field of $CANISTER_ID_B is invalid."

  ## 2.3. hash mismatch
  jq '.canisters."'"$CANISTER_ID_A"'".wasm_hash_download="0123456789abcdef"' deps/pulled.json.bak > deps/pulled.json
  assert_command_fail dfx deps init
  assert_contains "The wasm of $CANISTER_ID_A in pulled cache has different hash than in pulled.json:"
  assert_contains "The pulled cache is at \"$WASM_PATH_A\". Its hash is:"
  assert_contains "The hash (wasm_hash_download) in pulled.json is:"
  assert_contains "The pulled cache may be modified manually or the same canister was pulled in different projects."
  assert_command_fail dfx deps deploy
  assert_contains "The wasm of $CANISTER_ID_A in pulled cache has different hash than in pulled.json:"
  assert_contains "The pulled cache is at \"$WASM_PATH_A\". Its hash is:"
  assert_contains "The hash (wasm_hash_download) in pulled.json is:"
  assert_contains "The pulled cache may be modified manually or the same canister was pulled in different projects."
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
  cd ../
  cleanup_onchain

  cd app
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
  assert_command dfx deps init "$CANISTER_ID_C" --argument "(opt 33)"
  assert_command dfx deps deploy

  assert_command dfx canister call app get_b
  assert_eq "(2 : nat)" "$output"
  assert_command dfx canister call app get_c
  assert_eq "(33 : nat)" "$output" # corresponding to --argument "(opt 33)" above
  assert_command dfx canister call app get_b_times_a
  assert_eq "(22 : nat)" "$output" # 2 * 11
  assert_command dfx canister call app get_c_times_a
  assert_eq "(363 : nat)" "$output" # 33 * 11

  # start a clean local replica
  dfx canister stop app
  dfx canister delete app --no-withdrawal
  assert_command dfx deploy # only deploy app canister
}

@test "dfx deps does nothing in a project has no pull dependencies" {
  dfx_new empty

  # verify the help message
  assert_command dfx deps pull -h
  assert_contains "Pull canisters upon which the project depends. This command connects to the \"ic\" mainnet by default."
  assert_contains "You can still choose other network by setting \`--network\`"

  assert_command dfx deps pull
  assert_contains "There are no pull dependencies defined in dfx.json"
  assert_command dfx deps init
  assert_contains "There are no pull dependencies defined in dfx.json"
  assert_command dfx deps deploy
  assert_contains "There are no pull dependencies defined in dfx.json"

  assert_directory_not_exists "deps"
}

@test "dfx deps pull can set correct pulled.json when the dependency is already in cache" {
  use_test_specific_cache_root # dfx deps pull will download files to cache

  # start a "mainnet" replica which host the onchain canisters
  dfx_start

  setup_onchain

  # pull canisters in app project and the dependencies are cached
  cd app
  assert_command dfx deps pull --network local

  # the second pull should be able to set the correct pulled.json
  assert_command dfx deps pull --network local

  # this command will fail if the pulled.json is not correct
  assert_command dfx deps init
}

@test "dfx deps can facade pull ICP ledger" {
  use_test_specific_cache_root # dfx deps pull will download files to cache

  dfx_new
  jq '.canisters.e2e_project_backend.dependencies=["icp_ledger"]' dfx.json | sponge dfx.json
  jq '.canisters.icp_ledger.type="pull"' dfx.json | sponge dfx.json
  jq '.canisters.icp_ledger.id="ryjl3-tyaaa-aaaaa-aaaba-cai"' dfx.json | sponge dfx.json

  dfx_start
  assert_command dfx deps pull --network local
  assert_contains "Using facade dependencies for canister ryjl3-tyaaa-aaaaa-aaaba-cai."

  dfx identity new --storage-mode plaintext minter
  assert_command_fail dfx deps init icp_ledger
  assert_contains "1. Create a 'minter' identity: dfx identity new minter
2. Run the following multi-line command:"

  assert_command dfx deps init ryjl3-tyaaa-aaaaa-aaaba-cai --argument "(variant { 
    Init = record {
        minting_account = \"$(dfx --identity minter ledger account-id)\";
        initial_values = vec {};
        send_whitelist = vec {};
        transfer_fee = opt record { e8s = 10_000 : nat64; };
        token_symbol = opt \"LICP\";
        token_name = opt \"Local ICP\"; 
    }
})"

  assert_command dfx deps deploy

  # Can mint tokens (transfer from minting_account)
  assert_command dfx --identity minter canister call icp_ledger icrc1_transfer "(
  record {
    to = record {
      owner = principal \"$(dfx --identity default identity get-principal)\";
    };
    amount = 1_000_000 : nat;
  },
)"

  assert_command dfx canister call icp_ledger icrc1_balance_of "(
  record {
    owner = principal \"$(dfx --identity default identity get-principal)\";
  },
)"
  assert_eq "(1_000_000 : nat)"
}

@test "dfx deps can facade pull ckBTC ledger" {
  [[ "$USE_POCKETIC" ]] && skip "skipped for pocketic which doesn't have ckBTC subnet"

  use_test_specific_cache_root # dfx deps pull will download files to cache

  dfx_new
  jq '.canisters.e2e_project_backend.dependencies=["ckbtc_ledger"]' dfx.json | sponge dfx.json
  jq '.canisters.ckbtc_ledger.type="pull"' dfx.json | sponge dfx.json
  jq '.canisters.ckbtc_ledger.id="mxzaz-hqaaa-aaaar-qaada-cai"' dfx.json | sponge dfx.json

  dfx_start
  assert_command dfx deps pull --network local
  assert_contains "Using facade dependencies for canister mxzaz-hqaaa-aaaar-qaada-cai."

  dfx identity new --storage-mode plaintext minter
  assert_command_fail dfx deps init ckbtc_ledger
  assert_contains "1. Create a 'minter' identity: dfx identity new minter
2. Run the following multi-line command:"

  assert_command dfx deps init mxzaz-hqaaa-aaaar-qaada-cai --argument "(variant {
    Init = record {
        minting_account = record { owner = principal \"$(dfx --identity minter identity get-principal)\"; };
        transfer_fee = 10;
        token_symbol = \"ckBTC\";
        token_name = \"ckBTC\";
        metadata = vec {};
        initial_balances = vec {};
        max_memo_length = opt 80;
        archive_options = record {
            num_blocks_to_archive = 1000;
            trigger_threshold = 2000;
            max_message_size_bytes = null;
            cycles_for_archive_creation = opt 100_000_000_000_000;
            node_max_memory_size_bytes = opt 3_221_225_472;
            controller_id = principal \"2vxsx-fae\"
        }
    }
})"

  assert_command dfx deps deploy

  # Can mint tokens (transfer from minting_account)
  assert_command dfx --identity minter canister call ckbtc_ledger icrc1_transfer "(
  record {
    to = record {
      owner = principal \"$(dfx --identity default identity get-principal)\";
    };
    amount = 1_000_000 : nat;
  },
)"

  assert_command dfx canister call ckbtc_ledger icrc1_balance_of "(
  record {
    owner = principal \"$(dfx --identity default identity get-principal)\";
  },
)"
  assert_eq "(1_000_000 : nat)"
}


@test "dfx deps can facade pull ckETH ledger" {
  [[ "$USE_POCKETIC" ]] && skip "skipped for pocketic which doesn't have ckETH subnet"

  use_test_specific_cache_root # dfx deps pull will download files to cache

  dfx_new
  jq '.canisters.e2e_project_backend.dependencies=["cketh_ledger"]' dfx.json | sponge dfx.json
  jq '.canisters.cketh_ledger.type="pull"' dfx.json | sponge dfx.json
  jq '.canisters.cketh_ledger.id="ss2fx-dyaaa-aaaar-qacoq-cai"' dfx.json | sponge dfx.json

  dfx_start
  assert_command dfx deps pull --network local
  assert_contains "Using facade dependencies for canister ss2fx-dyaaa-aaaar-qacoq-cai."

  dfx identity new --storage-mode plaintext minter
  assert_command_fail dfx deps init cketh_ledger
  assert_contains "1. Create a 'minter' identity: dfx identity new minter
2. Run the following multi-line command:"

  assert_command dfx deps init ss2fx-dyaaa-aaaar-qacoq-cai --argument "(variant {
    Init = record {
        minting_account = record { owner = principal \"$(dfx --identity minter identity get-principal)\"; };
        decimals = opt 18;
        max_memo_length = opt 80;
        transfer_fee = 2_000_000_000_000;
        token_symbol = \"ckETH\";
        token_name = \"ckETH\";
        feature_flags = opt record { icrc2 = true };
        metadata = vec {};
        initial_balances = vec {};
        archive_options = record {
            num_blocks_to_archive = 1000;
            trigger_threshold = 2000;
            max_message_size_bytes = null;
            cycles_for_archive_creation = opt 100_000_000_000_000;
            node_max_memory_size_bytes = opt 3_221_225_472;
            controller_id = principal \"2vxsx-fae\"
        }
    }
})"

  assert_command dfx deps deploy

  # Can mint tokens (transfer from minting_account)
  assert_command dfx --identity minter canister call cketh_ledger icrc1_transfer "(
  record {
    to = record {
      owner = principal \"$(dfx --identity default identity get-principal)\";
    };
    amount = 1_000_000 : nat;
  },
)"

  assert_command dfx canister call cketh_ledger icrc1_balance_of "(
  record {
    owner = principal \"$(dfx --identity default identity get-principal)\";
  },
)"
  assert_eq "(1_000_000 : nat)"
}
