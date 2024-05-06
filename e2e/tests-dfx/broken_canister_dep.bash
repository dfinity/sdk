#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup

  install_asset broken_canister_dep
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "crash on a broken canister dependency" {
    dfx_start

    assert_command_fail dfx deploy

    assert_not_contains "panicked at"
}