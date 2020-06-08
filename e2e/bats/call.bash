#!/usr/bin/env bats

load utils/_

setup() {
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    dfx_new hello
}

teardown() {
    dfx_stop
}

@test "call subcommand accepts canister identifier as canister name" {
    install_asset greet
    dfx_start
    dfx build
    dfx canister install hello
    assert_command dfx canister call $(dfx canister id hello) greet '("Names are difficult")'
    assert_eq 'cannot find method type, dfx will send message with inferred type\n("Hello, Names are difficult!")'
}
