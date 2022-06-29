#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "deploy from a fresh project" {
    dfx_new hello
    dfx_start
    install_asset greet
    assert_command dfx deploy

    assert_command dfx canister call hello_backend greet '("Banzai")'
    assert_eq '("Hello, Banzai!")'
}

@test "deploy a canister without dependencies" {
    dfx_new hello
    dfx_start
    install_asset greet
    assert_command dfx deploy hello_backend
    assert_match 'Deploying: hello_backend'
    assert_not_match 'hello_frontend'
}

@test "deploy a canister with dependencies" {
    dfx_new hello
    dfx_start
    install_asset greet
    assert_command dfx deploy hello_frontend
    assert_match 'Deploying: hello_backend hello_frontend'
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

@test "deploy with InstallMode::Install on an empty canister" {
    dfx_new hello
    install_asset greet
    dfx_start
    assert_command dfx canister create --all

    assert_command dfx deploy
    assert_match 'Installing code for canister'
}

@test "dfx deploy supports arguments" {
    dfx_new hello
    install_asset greet_arg
    dfx_start
    assert_command dfx canister create --all

    assert_command dfx deploy --argument '("World")'

    assert_command dfx canister call hello_backend greet
    assert_match 'Hello, World'
}

@test "dfx deploy with InstallMode::Install on first invocation, InstallMode::Upgrade on second" {
    dfx_new hello
    install_asset greet
    dfx_start

    # In the normal case, whether for an initial install or a subsequent install,
    # dfx deploy does the right thing, so it doesn't need to retry.
    # Therefore, there is no "attempting (install|upgrade)" message.

    assert_command dfx deploy hello_backend
    assert_match 'Installing code for canister'

    assert_command dfx canister call hello_backend greet '("First")'
    assert_eq '("Hello, First!")'

    assert_command dfx deploy hello_backend --upgrade-unchanged
    assert_match 'Upgrading code for canister'

    assert_command dfx canister call hello_backend greet '("Second")'
    assert_eq '("Hello, Second!")'
}
