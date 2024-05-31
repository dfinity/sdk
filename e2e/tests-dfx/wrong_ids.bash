#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup

  install_asset wrong_ids
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "deploy causes wrong ids" {
    dfx_start

    assert_command dfx deploy -vv pst
    assert_command ls .dfx/local/lsp
    assert_contains '-cai.did'
}