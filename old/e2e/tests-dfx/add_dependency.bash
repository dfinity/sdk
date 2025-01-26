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

# Check that attempt to compile before correcting dependencies does not break further compilation.
@test "compiles after correcting a dependency" {
  install_asset base

  dfx_start

  assert_command_fail dfx deploy

  cp dfx_corrected.json dfx.json

  assert_command dfx deploy
}
