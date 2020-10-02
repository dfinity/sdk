#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)
    export RUST_BACKTRACE=1
}

teardown() {
  dfx_stop
}

@test "deploy from a fresh project" {
    dfx_new hello
    dfx_start
    install_asset greet
    assert_command dfx deploy

    assert_command dfx canister call hello greet '("Banzai")'
    assert_eq '("Hello, Banzai!")'
}

@test "deploy a canister without dependencies" {
    dfx_new hello
    dfx_start
    install_asset greet
    assert_command dfx deploy hello
    assert_match 'Deploying: hello'
    assert_not_match 'hello_assets'
}

@test "deploy a canister with dependencies" {
    dfx_new hello
    dfx_start
    install_asset greet
    assert_command dfx deploy hello_assets
    assert_match 'Deploying: hello hello_assets'
}

@test "deploy a canister with non-circular shared dependencies" {
    install_asset transitive_deps_canisters
    dfx_start
    assert_command dfx deploy canister_f
    assert_match 'Deploying: canister_a canister_f canister_g canister_h'
}

@test "report an error on attempt to deploy a canister with circular dependencies" {
    install_asset transitive_deps_canisters
    dfx_start
    assert_command_fail dfx deploy canister_d
    assert_match 'canister_d -> canister_e -> canister_d'
}

@test "if already registered, try to upgrade then install" {
    dfx_new hello
    install_asset greet
    dfx_start
    assert_command dfx canister create --all

    assert_command dfx deploy
    assert_match 'attempting install'
}

@test "dfx deploy supports arguments" {
    dfx_new hello
    install_asset greet_arg
    dfx_start
    assert_command dfx canister create --all

    assert_command dfx deploy --argument World
    assert_match 'attempting install'

    assert_command dfx canister call hello greet
    assert_match 'Hello, World'
}

