#!/usr/bin/env bats

load ../utils/_
# load ../utils/cycles-ledger

setup() {
  standard_setup

  install_asset make_like
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "trying to break dependency compiling" {
    dfx_start
    assert_command dfx deploy -vv dependent
    assert_contains '"dependent.mo"'
    assert_contains '"dependency.mo"'

    touch dependent.mo
    assert_contains '"dependent.mo"'
    assert_not_contains '"dependency.mo"'

    # TODO
    # touch dependency.mo
    # assert_contains '"dependent.mo"'
    # assert_contains '"dependency.mo"'
}
