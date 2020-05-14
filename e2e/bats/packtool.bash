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
    install_asset packtool_mo

    assert_command_fail dfx build
    assert_match 'import error, package "(rate|describe)" not defined'
}

@test "build succeeds if packtool is configured" {
    install_asset packtool_mo
    source configure_packtool.bash

    dfx build
}

@test "project calls dependencies made available by packtool" {
    install_asset packtool_mo
    source configure_packtool.bash

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
    dfx config defaults/build/packtool '["./no-such-command", "that command cannot be invoked"]'

    assert_command_fail dfx build
    assert_match 'FailedToInvokePackageTool'
    assert_match './no-such-command.*that command cannot be invoked'
    assert_match 'No such file or directory \(os error 2\)'
}

@test "failure in execution reports the command line and exit code" {
    install_asset packtool_mo
    dfx config defaults/build/packtool '["sh", "-c", "exit 3"]'

    assert_command_fail dfx build
    assert_match 'PackageToolReportedError'
    assert_match 'sh.*-c.*exit 3'
    assert_match 'exit code: 3'
}
