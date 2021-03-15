#!/usr/bin/env bats

load ../utils/_

setup() {
    cd "$(mktemp -d -t dfx-e2e-XXXXXXXX)" || exit
}

teardown() {
    :
}

@test "can deploy twice" {
    dfx_new
    dfx_start
    dfx canister create --all
    dfx build
    dfx deploy
    dfx deploy
    dfx canister call e2e_project greet world
    dfx_stop
}
