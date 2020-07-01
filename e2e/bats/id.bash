#!/usr/bin/env bats

load utils/_

setup() {
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    dfx_new
}

teardown() {
    dfx_stop
}

@test "id subcommand prints valid canister identifier" {
    install_asset id
    dfx_start
    dfx canister create --all
    dfx build
    assert_command dfx canister id e2e_project
    assert_match $(cat canisters/canister_manifest.json | jq -r .canisters.e2e_project.canister_id)
}
