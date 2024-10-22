#!/usr/bin/env bats

load ../utils/_
load ../utils/cycles-ledger

setup() {
  standard_setup

  dfx_new_assets
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "create with reserved cycles limit" {
    dfx_start

    assert_command_fail dfx canister create e2e_project_backend --reserved-cycles-limit 470000
    assert_contains "Cannot create a canister using a wallet if the reserved_cycles_limit is set. Please create with --no-wallet or use dfx canister update-settings instead."

    assert_command dfx canister create e2e_project_frontend --no-wallet
    assert_command dfx canister status e2e_project_frontend
    assert_contains "Reserved cycles limit: 5_000_000_000_000 Cycles"

    assert_command dfx canister create e2e_project_backend --reserved-cycles-limit 470000 --no-wallet
    assert_command dfx canister status e2e_project_backend
    assert_contains "Reserved cycles limit: 470_000 Cycles"
}

@test "create succeeds on default project" {
  dfx_start
  assert_command dfx canister create --all
}

@test "create succeeds with --specified-id" {
  dfx_start
  assert_command dfx canister create e2e_project_backend --specified-id n5n4y-3aaaa-aaaaa-p777q-cai
  assert_command dfx canister id e2e_project_backend
  assert_match n5n4y-3aaaa-aaaaa-p777q-cai
}

@test "create succeeds when specify large canister ID" {
  [[ "$USE_POCKETIC" ]] && skip "skipped for pocketic: subnet range"
  dfx_start
  # hhn2s-5l777-77777-7777q-cai is the canister ID of (u64::MAX / 2)
  assert_command dfx canister create e2e_project_backend --specified-id hhn2s-5l777-77777-7777q-cai
  assert_command dfx canister id e2e_project_backend
  assert_match hhn2s-5l777-77777-7777q-cai
}

@test "create fails when specify out of range canister ID" {
  dfx_start
  # nojwb-ieaaa-aaaaa-aaaaa-cai is the canister ID of (u64::MAX / 2 + 1)
  assert_command_fail dfx canister create e2e_project_backend --specified-id nojwb-ieaaa-aaaaa-aaaaa-cai

  if [[ "$USE_POCKETIC" ]]; then
    assert_match "The effective canister ID nojwb-ieaaa-aaaaa-aaaaa-cai does not belong to an existing subnet and it is not a mainnet canister ID."
  else
    assert_match "Specified CanisterId nojwb-ieaaa-aaaaa-aaaaa-cai is not hosted by subnet"
  fi
}

@test "create fails if set both --all and --specified-id" {
  dfx_start
  assert_command_fail dfx canister create --all --specified-id xbgkv-fyaaa-aaaaa-aaava-cai
  assert_match "error: the argument '--all' cannot be used with '--specified-id <PRINCIPAL>'"
}

@test "create succeeds when specify canister ID in dfx.json" {
  dfx_start
  jq '.canisters.e2e_project_backend.specified_id="n5n4y-3aaaa-aaaaa-p777q-cai"' dfx.json | sponge dfx.json
  assert_command dfx canister create e2e_project_backend
  assert_command dfx canister id e2e_project_backend
  assert_match n5n4y-3aaaa-aaaaa-p777q-cai
}

@test "create succeeds when specify canister ID both in dfx.json and cli; warning if different; cli value takes effect" {
  dfx_start
  jq '.canisters.e2e_project_backend.specified_id="n5n4y-3aaaa-aaaaa-p777q-cai"' dfx.json | sponge dfx.json
  assert_command dfx canister create e2e_project_backend --specified-id n2m2m-wyaaa-aaaaa-p777a-cai
  assert_contains "WARN: Canister 'e2e_project_backend' has a specified ID in dfx.json: n5n4y-3aaaa-aaaaa-p777q-cai,"
  assert_contains "which is different from the one specified in the command line: n2m2m-wyaaa-aaaaa-p777a-cai."
  assert_contains "The command line value will be used."

  assert_command dfx canister id e2e_project_backend
  assert_match n2m2m-wyaaa-aaaaa-p777a-cai
}

@test "create generates the canister_ids.json" {
  dfx_start
  assert_command dfx canister create --all
  [[ -f .dfx/local/canister_ids.json ]]
}

@test "create without parameters sets wallet and self as controller" {
  dfx_start
  PRINCIPAL=$(dfx identity get-principal)
  WALLET=$(dfx identity get-wallet)
  assert_command dfx canister create --all
  assert_command dfx canister info e2e_project_backend
  assert_match "Controllers: ($PRINCIPAL $WALLET|$WALLET $PRINCIPAL)"
}

@test "create with --no-wallet sets only self as controller" {
  dfx_start
  PRINCIPAL=$(dfx identity get-principal)
  WALLET=$(dfx identity get-wallet)
  assert_command dfx canister create --all --no-wallet
  assert_command dfx canister info e2e_project_backend
  assert_not_match "Controllers: ($PRINCIPAL $WALLET|$WALLET $PRINCIPAL)"
  assert_match "Controllers: $PRINCIPAL"
}

@test "build fails without create" {
  dfx_start
  assert_command_fail dfx build
  assert_match "Cannot find canister id."
}

@test "build fails if all canisters in project are not created" {
  dfx_start
  assert_command dfx canister create e2e_project_backend
  assert_command_fail dfx build
  assert_match "Cannot find canister id. Please issue 'dfx canister create e2e_project_frontend'"
}

@test "create succeeds with network parameter" {
  dfx_start
  assert_command dfx canister create --all --network local
}

@test "create fails with incorrect network" {
  dfx_start
  assert_command_fail dfx canister create --all --network nosuch
  assert_match "Network not found"
}

@test "create succeeds when requested network is configured" {
  dfx_start

  setup_actuallylocal_shared_network
  assert_command dfx canister create --all --network actuallylocal
}

@test "create with wallet succeeds when requested network is configured" {
  dfx_start
  setup_actuallylocal_shared_network

  assert_command dfx_set_wallet
  assert_command dfx canister create --all --network actuallylocal
}

@test "create fails if selected network exists but has no providers" {
  dfx_start

  jq '.networks.actuallylocal.providers=[]' dfx.json | sponge dfx.json
  assert_command_fail dfx canister create --all --network actuallylocal
  assert_match "Did not find any providers for network 'actuallylocal'"
}

@test "create fails with network parameter when network does not exist" {
  dfx_start
  jq '.networks.actuallylocal.providers=["http://not-real.nowhere.test."]' dfx.json | sponge dfx.json
  assert_command_fail dfx canister create --all --network actuallylocal
  assert_contains "error sending request for url (http://not-real.nowhere.test./api/v2/status)"
}

@test "create accepts --controller <controller> named parameter, with controller by identity name" {
  dfx_start
  dfx identity new --storage-mode plaintext alice
  ALICE_PRINCIPAL=$(dfx identity get-principal --identity alice)


  assert_command dfx canister create --all --controller alice
  assert_command dfx canister info e2e_project_backend
  assert_match "Controllers: $ALICE_PRINCIPAL"

  assert_command_fail dfx deploy
  assert_command dfx deploy --identity alice
}

@test "create accepts --controller <controller> named parameter, with controller by identity principal" {
  dfx_start
  dfx identity new --storage-mode plaintext alice
  ALICE_PRINCIPAL=$(dfx identity get-principal --identity alice)
  ALICE_WALLET=$(dfx identity get-wallet --identity alice)

  assert_command dfx canister create --all --controller "${ALICE_PRINCIPAL}"
  assert_command dfx canister info e2e_project_backend
  assert_not_match "Controllers: ($ALICE_WALLET $ALICE_PRINCIPAL|$ALICE_PRINCIPAL $ALICE_WALLET)"
  assert_match "Controllers: $ALICE_PRINCIPAL"

  assert_command_fail dfx deploy
  assert_command dfx deploy --identity alice
}

@test "create accepts --controller <controller> named parameter, with controller by wallet principal" {
  dfx_start
  dfx identity new --storage-mode plaintext alice
  ALICE_WALLET=$(dfx identity get-wallet --identity alice)

  assert_command dfx canister create --all --controller "${ALICE_WALLET}"
  assert_command dfx canister info e2e_project_backend
  assert_match "Controllers: $ALICE_WALLET"

  assert_command_fail dfx deploy
  assert_command_fail dfx deploy --identity alice
  assert_command dfx deploy --identity alice --wallet alice
}

@test "create accepts --controller <controller> named parameter, with controller by name of selected identity" {
  # there is a different code path if the specified controller happens to be
  # the currently selected identity.
  dfx_start
  dfx identity new --storage-mode plaintext alice
  dfx identity new --storage-mode plaintext bob
  BOB_PRINCIPAL=$(dfx identity get-principal --identity bob)

  dfx identity use bob

  assert_command dfx canister create --all --controller bob

  dfx identity use alice
  assert_command dfx canister info e2e_project_backend
  assert_match "Controllers: $BOB_PRINCIPAL"

  assert_command_fail dfx deploy
  assert_command dfx deploy --identity bob
}

@test "create single controller accepts --controller <controller> named parameter, with controller by identity name" {
  dfx_start
  dfx identity new --storage-mode plaintext alice
  dfx identity new --storage-mode plaintext bob
  ALICE_PRINCIPAL=$(dfx identity get-principal --identity alice)
  BOB_PRINCIPAL=$(dfx identity get-principal --identity bob)

  assert_command dfx canister create --controller alice e2e_project_backend
  assert_command dfx canister create --controller bob e2e_project_frontend

  assert_command dfx canister info e2e_project_backend
  assert_match "Controllers: $ALICE_PRINCIPAL"

  assert_command dfx canister info e2e_project_frontend
  assert_match "Controllers: $BOB_PRINCIPAL"

  # check this first, because alice will deploy e2e_project in the next step
  assert_command_fail dfx deploy e2e_project_backend --identity bob
  # this actually deploys e2e_project before failing, because it is a dependency
  assert_command_fail dfx deploy e2e_project_frontend --identity alice

  assert_command dfx deploy e2e_project_backend --identity alice
  assert_command dfx deploy e2e_project_frontend --identity bob
}

@test "create canister with multiple controllers" {
  dfx_start
  dfx identity new --storage-mode plaintext alice
  dfx identity new --storage-mode plaintext bob
  ALICE_PRINCIPAL=$(dfx identity get-principal --identity alice)
  BOB_PRINCIPAL=$(dfx identity get-principal --identity bob)
  # awk step is to avoid trailing space
  PRINCIPALS_SORTED=$(echo "$ALICE_PRINCIPAL" "$BOB_PRINCIPAL" | tr " " "\n" | sort | tr "\n" " " | awk '{printf "%s %s",$1,$2}' )

  assert_command dfx canister create --all --controller alice --controller bob --identity alice
  assert_command dfx canister info e2e_project_backend
  assert_match "Controllers: ${PRINCIPALS_SORTED}"

  assert_command dfx deploy --identity alice
  assert_command_fail dfx deploy --identity bob

  # The certified assets canister will have added alice as an authorized user, because she was the caller
  # at initialization time.  Bob has to be added separately.  BUT, the canister has to be deployed first
  # in order to call the authorize method.
  assert_command dfx canister call e2e_project_frontend authorize "(principal \"$BOB_PRINCIPAL\")" --identity alice

  assert_command dfx deploy --identity bob
}

@test "reports wallet must be upgraded if attempting to create a canister with multiple controllers through an old wallet" {
  use_wallet_wasm 0.7.2

  dfx_start
  dfx identity new --storage-mode plaintext alice
  dfx identity new --storage-mode plaintext bob

  assert_command_fail dfx canister create --all --controller alice --controller bob --identity alice
  assert_match "The wallet canister must be upgraded: The installed wallet does not support multiple controllers."
  assert_match "To upgrade, run dfx wallet upgrade"

  use_wallet_wasm 0.8.2
  assert_command dfx wallet upgrade --identity alice
  assert_command dfx canister create --all --controller alice --controller bob --identity alice
}

@test "create canister - subnet targetting" {
  # fake cmc setup
  cd ..
  dfx_new fake_cmc
  install_asset fake_cmc
  install_cycles_ledger_canisters
  dfx_start
  assert_command dfx deploy fake-cmc --specified-id "rkp4c-7iaaa-aaaaa-aaaca-cai" # CMC canister id
  cd ../e2e_project

  # use --subnet <principal>
  SUBNET_ID="5kdm2-62fc6-fwnja-hutkz-ycsnm-4z33i-woh43-4cenu-ev7mi-gii6t-4ae" # a random, valid principal
  assert_command dfx canister create e2e_project_backend --subnet "$SUBNET_ID"
  cd ../fake_cmc
  assert_command dfx canister call fake-cmc last_create_canister_args
  assert_contains "subnet = principal \"$SUBNET_ID\";"
  
  # use --subnet-type
  cd ../e2e_project
  assert_command dfx canister create e2e_project_frontend --subnet-type custom_subnet_type
  cd ../fake_cmc
  assert_command dfx canister call fake-cmc last_create_canister_args
  assert_contains 'subnet_type = opt "custom_subnet_type"'
}

@test "create with dfx.json settings" {
  jq '.canisters.e2e_project_backend.initialization_values={
    "compute_allocation": 5,
    "freezing_threshold": "7days",
    "memory_allocation": "2 GiB",
    "reserved_cycles_limit": 1000000000000,
    "wasm_memory_limit": "1 GiB",
    "log_visibility": "public",
  }' dfx.json | sponge dfx.json
  dfx_start
  assert_command dfx deploy e2e_project_backend --no-wallet
  assert_command dfx canister status e2e_project_backend
  assert_contains 'Memory allocation: 2_147_483_648'
  assert_contains 'Compute allocation: 5'
  assert_contains 'Reserved cycles limit: 1_000_000_000_000'
  assert_contains 'Wasm memory limit: 1_073_741_824'
  assert_contains 'Freezing threshold: 604_800'
  assert_contains 'Log visibility: public'
}

@test "create with default settings" {
  dfx_start
  assert_command dfx deploy e2e_project_backend
  assert_command dfx canister status e2e_project_backend
  assert_contains 'Memory allocation: 0'
  assert_contains 'Compute allocation: 0'
  assert_contains 'Reserved cycles limit: 5_000_000_000_000'
  assert_contains 'Wasm memory limit: 3_221_225_472'
  assert_contains 'Freezing threshold: 2_592_000'
  assert_contains 'Log visibility: controllers'
}

@test "create with multiple log allowed viewer list in dfx.json" {
  # Create two identities
  assert_command dfx identity new --storage-mode plaintext alice
  assert_command dfx identity new --storage-mode plaintext bob
  ALICE_PRINCIPAL=$(dfx identity get-principal --identity alice)
  BOB_PRINCIPAL=$(dfx identity get-principal --identity bob)

  jq '.canisters.e2e_project_backend.initialization_values={
    "compute_allocation": 5,
    "freezing_threshold": "7days",
    "memory_allocation": "2 GiB",
    "reserved_cycles_limit": 1000000000000,
    "wasm_memory_limit": "1 GiB",
    "log_visibility": {
      "allowed_viewers" :
       ['\""$ALICE_PRINCIPAL"\"', '\""$BOB_PRINCIPAL"\"']
    }
  }' dfx.json | sponge dfx.json
  dfx_start
  assert_command dfx deploy e2e_project_backend --no-wallet
  assert_command dfx canister status e2e_project_backend
  assert_contains 'Memory allocation: 2_147_483_648'
  assert_contains 'Compute allocation: 5'
  assert_contains 'Reserved cycles limit: 1_000_000_000_000'
  assert_contains 'Wasm memory limit: 1_073_741_824'
  assert_contains 'Freezing threshold: 604_800'
  assert_contains "${ALICE_PRINCIPAL}"
  assert_contains "${BOB_PRINCIPAL}"
}

@test "create with multiple log allowed viewer list" {
  # Create two identities
  assert_command dfx identity new --storage-mode plaintext alice
  assert_command dfx identity new --storage-mode plaintext bob
  ALICE_PRINCIPAL=$(dfx identity get-principal --identity alice)
  BOB_PRINCIPAL=$(dfx identity get-principal --identity bob)

  dfx_start
  assert_command dfx canister create --all --log-viewer "${ALICE_PRINCIPAL}" --log-viewer "${BOB_PRINCIPAL}"  --no-wallet
  assert_command dfx deploy e2e_project_backend --no-wallet
  assert_command dfx canister status e2e_project_backend
  assert_contains "${ALICE_PRINCIPAL}"
  assert_contains "${BOB_PRINCIPAL}"
}

# The following function decodes a canister id in the textual form into its binary form
# and is taken from the [IC Interface Specification](https://internetcomputer.org/docs/current/references/ic-interface-spec#principal).
function textual_decode() {
  echo -n "$1" | tr -d - | tr "[:lower:]" "[:upper:]" |
  fold -w 8 | xargs -n1 printf '%-8s' | tr ' ' = |
  base32 -d | xxd -p | tr -d '\n' | cut -b9- | tr "[:lower:]" "[:upper:]"
}

@test "create targets application subnet in PocketIC" {
  [[ ! "$USE_POCKETIC" ]] && skip "skipped for replica: no support for multiple subnets"
  dfx_start
  # create the backend canister without a wallet canister so that the backend canister is the only canister ever created
  assert_command dfx canister create e2e_project_backend --no-wallet
  # actual canister id
  CANISTER_ID="$(dfx canister id e2e_project_backend)"
  # base64 encode the actual canister id
  CANISTER_ID_BASE64="$(textual_decode "${CANISTER_ID}" | xxd -r -p | base64)"
  # fetch topology from PocketIC server
  TOPOLOGY="$(curl "http://127.0.0.1:$(dfx info replica-port)/instances/0/read/topology")"
  echo "${TOPOLOGY}"
  # find application subnet id in the topology
  for subnet_id in $(echo "${TOPOLOGY}" | jq '.subnet_configs | keys[]')
  do
    SUBNET_KIND="$(echo "$TOPOLOGY" | jq -r ".subnet_configs.${subnet_id}.\"subnet_kind\"")"
    if [ "${SUBNET_KIND}" == "Application" ]
    then
      # find the expected canister id as the beginning of the first canister range of the app subnet
      EXPECTED_CANISTER_ID_BASE64="$(echo "$TOPOLOGY" | jq -r ".subnet_configs.${subnet_id}.\"canister_ranges\"[0].\"start\".\"canister_id\"")"
    fi
  done
  # check if the actual canister id matches the expected canister id
  if [ "${CANISTER_ID_BASE64}" != "${EXPECTED_CANISTER_ID_BASE64}" ]
  then
    echo "Canister id ${CANISTER_ID_BASE64} does not match expected canister id ${EXPECTED_CANISTER_ID_BASE64}"
    exit 1
  fi
}
