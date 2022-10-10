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



@test "create succeeds on default project" {
    dfx_start
    assert_command dfx canister create --all
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
    assert_match "ComputeNetworkNotFound"
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
    assert_match "Cannot find providers for network"
}

@test "create fails with network parameter when network does not exist" {
    dfx_start
    jq '.networks.actuallylocal.providers=["http://not-real.nowhere.test."]' dfx.json | sponge dfx.json
    assert_command_fail dfx canister create --all --network actuallylocal
    assert_match "dns error: failed to lookup address information"
}

@test "create accepts --controller <controller> named parameter, with controller by identity name" {
    dfx_start
    dfx identity new --disable-encryption alice
    ALICE_PRINCIPAL=$(dfx identity get-principal --identity alice)
    
    
    assert_command dfx canister create --all --controller alice
    assert_command dfx canister info e2e_project_backend
    assert_match "Controllers: $ALICE_PRINCIPAL"

    assert_command_fail dfx deploy
    assert_command dfx deploy --identity alice
}

@test "create accepts --controller <controller> named parameter, with controller by identity principal" {
    dfx_start
    dfx identity new --disable-encryption alice
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
    dfx identity new --disable-encryption alice
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
    dfx identity new --disable-encryption alice
    dfx identity new --disable-encryption bob
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
    dfx identity new --disable-encryption alice
    dfx identity new --disable-encryption bob
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
    dfx identity new --disable-encryption alice
    dfx identity new --disable-encryption bob
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
    dfx identity new --disable-encryption alice
    dfx identity new --disable-encryption bob

    assert_command_fail dfx canister create --all --controller alice --controller bob --identity alice
    assert_match "The wallet canister must be upgraded: The installed wallet does not support multiple controllers."
    assert_match "To upgrade, run dfx wallet upgrade"

    use_wallet_wasm 0.8.2
    assert_command dfx wallet upgrade --identity alice
    assert_command dfx canister create --all --controller alice --controller bob --identity alice
}

@test "canister-create on mainnet without wallet does not propagate the 404" {
    assert_command_fail dfx deploy --network ic --no-wallet
    assert_match 'dfx ledger create-canister'
}
