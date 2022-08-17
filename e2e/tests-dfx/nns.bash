#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "ic-nns-init binary exists and is executable" {
    dfx cache install

    # it panics, but still shows help
    assert_command_fail "$(dfx cache show)/ic-nns-init" --help
    assert_match "thread 'main' panicked at 'Illegal arguments:"
    assert_match "ic-nns-init \[OPTIONS\]"
    assert_match "-h, --help.*Print help information"
    assert_match '--version.*Print version information'

    # --version fails too
    assert_command_fail "$(dfx cache show)/ic-nns-init" --version
}

@test "dfx nns install command exists" {
    dfx_start

    # it just calls ic-nns-init, which panics
    assert_command_fail dfx nns install
    assert_match "thread 'main' panicked at 'Illegal arguments:"
    assert_match "ic-nns-init \[OPTIONS\]"
    assert_match "ic-nns-init call failed"
}