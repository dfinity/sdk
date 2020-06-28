#!/usr/bin/env bats

load utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    cd $(mktemp -d -t dfx-e2e-XXXXXXXX)

    dfx_new
}

teardown() {
    dfx_stop
}

@test "build fails if packtool is not configured" {
    install_asset packtool

    dfx_start
    dfx canister create --all
    assert_command_fail dfx build
    assert_match 'import error, package "(rate|describe)" not defined'
}

@test "build succeeds if packtool is configured" {
    install_asset packtool
    source configure_packtool.bash

    dfx_start
    dfx canister create --all
    dfx build
}

@test "project calls dependencies made available by packtool" {
    install_asset packtool
    source configure_packtool.bash

    dfx_start
    dfx canister create --all
    dfx build
    dfx canister install e2e_project

    assert_command dfx canister call e2e_project rate '("rust")'
    assert_eq '("rust: So hot right now.")'

    assert_command dfx canister call e2e_project rate '("php")'
    assert_eq '("php: No comment.")'
}

@test "failure to invoke the package tool reports the command line and reason" {
    install_asset packtool
    dfx config defaults/build/packtool "./no-such-command that command cannot be invoked"

    dfx_start
    dfx canister create --all
    assert_command_fail dfx build
    assert_match 'Failed to invoke the package tool'
    assert_match 'no-such-command.*that.*command.*cannot.*be.*invoked'
    assert_match 'No such file or directory \(os error 2\)'
}

@test "failure in execution reports the command line and exit code" {
    install_asset packtool
    dfx config defaults/build/packtool "sh ./command-that-fails.bash"

    dfx_start
    dfx canister create --all
    assert_command_fail dfx build
    assert_match 'Package tool.*reported an error'
    assert_match 'sh.*command-that-fails.bash'
    assert_match 'exit code: 3'
}
