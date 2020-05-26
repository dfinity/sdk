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
    install_asset greet
    dfx_start
    dfx build
    dfx canister install e2e_project
    assert_command dfx canister call $(dfx canister id e2e_project) greet '("Names are difficult")'
    assert_eq '("Hello, Names are difficult!")'
}
