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

@test "deploy causes wrong ids" {
    dfx_start

    assert_command dfx deploy

    assert_not_contains "panicked at"
}