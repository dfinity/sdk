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

@test "id subcommand prints valid canister identifier" {
    install_asset id
    dfx_start
    dfx canister create --all
    dfx build
    assert_command dfx canister id e2e_project_backend
    assert_match "$(jq -r .e2e_project_backend.local < .dfx/local/canister_ids.json)"
}

@test "id subcommand works from a subdirectory of the project - ephemeral id" {
    install_asset id
    dfx_start
    dfx canister create --all
    ID=$(dfx canister id e2e_project_backend)
    echo "canister id is $ID"

    (
        cd src
        dfx canister id e2e_project_backend
        assert_command dfx canister id e2e_project_backend
        assert_eq "$ID"
    )
}

@test "id subcommand works from a subdirectory of the project - persistent id" {
    install_asset id

    jq '.networks.local.type="persistent"' dfx.json | sponge dfx.json
    dfx_start
    dfx canister create --all
    ID=$(dfx canister id e2e_project_backend)
    echo "canister id is $ID"
    (
        cd src
        dfx canister id e2e_project_backend
        assert_command dfx canister id e2e_project_backend
        assert_eq "$ID"
    )
}

@test "id subcommand uses default network for remotes only" {
    install_asset id
    dfx_start
    dfx canister create --all
    dfx build
    jq '.canisters.e2e_project_backend.remote.id.__default = "rkp4c-7iaaa-aaaaa-aaaca-cai"' dfx.json | sponge dfx.json
    jq '.canisters.e2e_project_backend.remote.id.namedremote = "va76m-bqaaa-aaaaa-aaayq-cai"' dfx.json | sponge dfx.json
    jq '.somethingelse= {"type": "persistent", "providers": ["http://somethingelse"]}' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
    # The local dfx canister ID should not be affected:
    assert_command dfx canister id e2e_project_backend
    assert_match "$(jq -r .e2e_project_backend.local < .dfx/local/canister_ids.json)"
    # Named remotes should be unaffected:
    assert_command dfx canister --network namedremote id e2e_project_backend
    assert_match "va76m-bqaaa-aaaaa-aaayq-cai"
    # Other remotes should use the default entry:
    assert_command dfx canister --network somethingelse id e2e_project_backend
    assert_match "rkp4c-7iaaa-aaaaa-aaaca-cai"
}
