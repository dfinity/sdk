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
    dfx build
    assert_command dfx canister id e2e_project
    assert_match $(python id.py canisters/e2e_project/_canister.id)
}
