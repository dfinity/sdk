#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup

  install_asset add_dependency
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "compiles after correcting a dependency" {
  install_asset base

  dfx_start

  # fails
  dfx deploy || true

  cp dfx_corrected.json dfx.json

  assert_command dfx deploy
}
