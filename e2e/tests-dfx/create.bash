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

@test "build fails without create" {
    dfx_start
    assert_command_fail dfx build
    assert_match "Cannot find canister id."
}

@test "build fails if all canisters in project are not created" {
    dfx_start
    assert_command dfx canister create e2e_project
    assert_command_fail dfx build
    assert_match "Cannot find canister id. Please issue 'dfx canister create e2e_project_assets'"
}

@test "create succeeds with network parameter" {
    dfx_start
    assert_command dfx canister --network local create --all
}

@test "create fails with incorrect network" {
    dfx_start
    assert_command_fail dfx canister --network nosuch create --all
    assert_match "ComputeNetworkNotFound"
}

@test "create succeeds when requested network is configured" {
    dfx_start

    setup_actuallylocal_network
    assert_command dfx canister --network actuallylocal create --all
}

@test "create with wallet succeeds when requested network is configured" {
    dfx_start
    setup_actuallylocal_network

    assert_command dfx_set_wallet
    assert_command dfx canister --network actuallylocal create --all
}

@test "create fails if selected network exists but has no providers" {
    dfx_start

    # shellcheck disable=SC2094
    cat <<<"$(jq '.networks.actuallylocal.providers=[]' dfx.json)" >dfx.json
    assert_command_fail dfx canister --network actuallylocal create --all
    assert_match "Cannot find providers for network"
}

@test "create fails with network parameter when network does not exist" {
    dfx_start
    # shellcheck disable=SC2094
    cat <<<"$(jq '.networks.actuallylocal.providers=["http://not-real.nowhere.test."]' dfx.json)" >dfx.json
    assert_command_fail dfx canister --network actuallylocal create --all
    assert_match "dns error: failed to lookup address information"
}


@test "create accepts --controller <controller> named parameter, with controller by identity name" {
    dfx_start
    dfx identity new alice
    ALICE_PRINCIPAL=$(dfx --identity alice identity get-principal)

    assert_command dfx canister create --all --controller alice
    assert_command dfx canister info e2e_project
    assert_match "Controllers: $ALICE_PRINCIPAL"

    assert_command_fail dfx deploy --no-wallet
    assert_command_fail dfx deploy
    assert_command dfx --identity alice deploy --no-wallet
}

@test "create accepts --controller <controller> named parameter, with controller by principal" {
    dfx_start
    dfx identity new alice
    ALICE_PRINCIPAL=$(dfx --identity alice identity get-principal)

    assert_command dfx canister create --all --controller "${ALICE_PRINCIPAL}"
    assert_command dfx canister info e2e_project
    assert_match "Controllers: $ALICE_PRINCIPAL"

    assert_command_fail dfx deploy --no-wallet
    assert_command_fail dfx deploy
    assert_command dfx --identity alice deploy --no-wallet
}

@test "create accepts --controller <controller> named parameter, with controller by name of selected identity" {
    # there is a different code path if the specified controller happens to be
    # the currently selected identity.
    dfx_start
    dfx identity new alice
    dfx identity new bob
    BOB_PRINCIPAL=$(dfx --identity bob identity get-principal)

    dfx identity use bob

    assert_command dfx canister create --all --controller bob

    dfx identity use alice
    assert_command dfx canister info e2e_project
    assert_match "Controllers: $BOB_PRINCIPAL"

    assert_command_fail dfx deploy --no-wallet
    assert_command_fail dfx deploy
    assert_command dfx --identity bob deploy --no-wallet
}

@test "create single controller accepts --controller <controller> named parameter, with controller by identity name" {
    dfx_start
    dfx identity new alice
    dfx identity new bob
    ALICE_PRINCIPAL=$(dfx --identity alice identity get-principal)
    BOB_PRINCIPAL=$(dfx --identity bob identity get-principal)

    assert_command dfx canister create --controller alice e2e_project
    assert_command dfx canister create --controller bob e2e_project_assets

    assert_command dfx canister info e2e_project
    assert_match "Controllers: $ALICE_PRINCIPAL"

    assert_command dfx canister info e2e_project_assets
    assert_match "Controllers: $BOB_PRINCIPAL"

    assert_command_fail dfx --identity alice deploy e2e_project_assets
    assert_command_fail dfx --identity bob deploy e2e_project
    # check this first, because alice will deploy e2e_project in the next step
    assert_command_fail dfx --identity bob deploy --no-wallet e2e_project
    # this actually deploys e2e_project before failing, because it is a dependency
    assert_command_fail dfx --identity alice deploy --no-wallet e2e_project_assets

    assert_command dfx --identity alice deploy --no-wallet e2e_project
    assert_command dfx --identity bob deploy --no-wallet e2e_project_assets
}

@test "create canister with multiple controllers" {
    dfx_start
    dfx identity new alice
    dfx identity new bob
    ALICE_PRINCIPAL=$(dfx --identity alice identity get-principal)
    BOB_PRINCIPAL=$(dfx --identity bob identity get-principal)
    # awk step is to avoid trailing space
    WALLETS_SORTED=$(echo "$ALICE_PRINCIPAL" "$BOB_PRINCIPAL" | tr " " "\n" | sort | tr "\n" " " | awk '{printf "%s %s",$1,$2}' )

    assert_command dfx --identity alice canister create --all --controller alice --controller bob
    assert_command dfx canister info e2e_project
    assert_match "Controllers: ${WALLETS_SORTED}"

    assert_command dfx --identity alice deploy --no-wallet
    assert_command_fail dfx --identity bob deploy --no-wallet

    # The certified assets canister will have added alice as an authorized user, because she was the caller
    # at initialization time.  Bob has to be added separately.  BUT, the canister has to be deployed first
    # in order to call the authorize method.
    assert_command dfx --identity alice canister call e2e_project_assets authorize "(principal \"$BOB_PRINCIPAL\")"

    assert_command dfx --identity bob deploy --no-wallet
}

@test "reports wallet must be upgraded if attempting to create a canister with multiple controllers through an old wallet" {
    use_wallet_wasm 0.7.2

    dfx_start
    dfx identity new alice
    dfx identity new bob
    ALICE_PRINCIPAL=$(dfx --identity alice identity get-principal)
    BOB_PRINCIPAL=$(dfx --identity bob identity get-principal)
    # awk step is to avoid trailing space
    WALLETS_SORTED=$(echo "$ALICE_PRINCIPAL" "$BOB_PRINCIPAL" | tr " " "\n" | sort | tr "\n" " " | awk '{printf "%s %s",$1,$2}' )

    assert_command_fail dfx --identity alice canister create --all --controller alice --controller bob
    assert_match "The wallet canister must be upgraded: The installed wallet does not support multiple controllers."
    assert_match "To upgrade, run dfx wallet upgrade"

    use_wallet_wasm 0.8.2
    assert_command dfx --identity alice wallet upgrade
    assert_command dfx --identity alice canister create --all --controller alice --controller bob
}

