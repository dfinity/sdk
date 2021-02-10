#!/usr/bin/env bats

load ./utils/_

setup() {
    cd "$(mktemp -d -t dfx-e2e-XXXXXXXX)" || exit
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
    assert_match "$(jq -r .e2e_project.local < .dfx/local/canister_ids.json)"
}
