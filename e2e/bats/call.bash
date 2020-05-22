#!/usr/bin/env bats

load utils/_

setup() {
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    dfx_new
}

teardown() {
    dfx_stop
}

@test "call subcommand accepts canister identifier as canister name" {
    install_asset print
    dfx_start
    dfx build
    dfx canister install e2e_project
    dfx canister call $(dfx canister id e2e_project) hello
}
