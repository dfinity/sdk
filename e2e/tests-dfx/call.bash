#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new hello
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "call subcommand accepts canister identifier as canister name" {
    install_asset greet
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install hello
    assert_command dfx canister call "$(dfx canister id hello)" greet '("Names are difficult")'
    assert_match '("Hello, Names are difficult!")'
}

@test "call random value (pattern)" {
    install_asset greet
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install hello
    assert_command dfx canister call hello greet --random '{ value = Some ["\"DFINITY\""] }'
    assert_match '("Hello, DFINITY!")'
}

@test "error on empty arguments when the method requires some" {
    install_asset greet
    dfx_start
    dfx deploy
    assert_command_fail dfx canister call hello greet
}

@test "call random value (empty)" {
    install_asset greet
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install hello
    assert_command dfx canister call hello greet --random ''
    assert_match '("Hello, .*!")'
}

@test "long call" {
    install_asset recurse
    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install hello
    assert_command dfx canister call hello recurse 100
}
