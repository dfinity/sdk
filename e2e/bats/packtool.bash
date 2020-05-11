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

@test "build fails without dependencies" {
    install_asset packtool_mo

    assert_command_fail dfx build
    assert_match 'import error, package "(rate|describe)" not defined'
}

@test "build succeeds with dependencies" {
    install_asset packtool_mo
    install_asset packtool_dependencies

    dfx build
}

@test "project calls dependencies" {
    install_asset packtool_mo
    install_asset packtool_dependencies

    dfx_start
    dfx build
    dfx canister install e2e_project

    assert_command dfx canister call e2e_project rate '("rust")'
    assert_eq '("rust: So hot right now.")'

    assert_command dfx canister call e2e_project rate '("php")'
    assert_eq '("php: No comment.")'
}

@test "failure to invoke the package tool reports the command line and reason" {
    install_asset packtool_mo
    install_asset packtool_error_invocation

    assert_command_fail dfx build
    assert_match 'Failed to invoke Package Tool Command "ec567ho" "that command cannot be invoked"'
    assert_match 'due to error: No such file or directory \(os error 2\)'
}

@test "failure in execution reports the command line and exit code" {
    install_asset packtool_mo
    install_asset packtool_error_execution

    assert_command_fail dfx build
    assert_match 'Package Tool Command "sh" "-c" "exit 3" failed with status: exit code: 3'
}
