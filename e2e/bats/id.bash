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
    dfx build --all
    assert_command dfx canister id e2e_project
    assert_match $(cat .dfx/local/canister_ids.json | jq -r .e2e_project.local)
}
