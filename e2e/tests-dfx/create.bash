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

@test "create with reserved cycles limit" {
    dfx_start

    assert_command_fail dfx canister create e2e_project_backend --reserved-cycles-limit 470000
    assert_contains "Cannot create a canister using a wallet if the reserved_cycles_limit is set. Please create with --no-wallet or use dfx canister update-settings instead."

    assert_command dfx canister create e2e_project_frontend --no-wallet
    assert_command dfx canister status e2e_project_frontend
    assert_contains "Reserved Cycles Limit: 5_000_000_000_000 Cycles"

    assert_command dfx canister create e2e_project_backend --reserved-cycles-limit 470000 --no-wallet
    assert_command dfx canister status e2e_project_backend
    assert_contains "Reserved Cycles Limit: 470_000 Cycles"
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

  assert_match "Specified CanisterId nojwb-ieaaa-aaaaa-aaaaa-cai is not hosted by subnet"
}

@test "create fails if set both --all and --specified-id" {
  dfx_start
  assert_command_fail dfx canister create --all --specified-id xbgkv-fyaaa-aaaaa-aaava-cai
  assert_match "error: the argument '--all' cannot be used with '--specified-id <PRINCIPAL>'"
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
  assert_match "dns error: failed to lookup address information"
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
  assert_command dfx deploy --identity alice --wallet "${ALICE_WALLET}"
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
