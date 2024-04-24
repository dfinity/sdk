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

@test "trying to break dependency compiling: deploy" {
    dfx_start

    assert_command dfx deploy -vv dependent
    assert_contains '"dependent.mo"'
    assert_contains '"dependency.mo"'

    touch dependent.mo
    assert_command dfx deploy -vv dependent
    assert_contains '"dependent.mo"'
    assert_not_contains '"dependency.mo"'

    touch dependency.mo
    assert_command dfx deploy -vv dependent
    assert_contains '"dependent.mo"'
    assert_contains '"dependency.mo"'

    touch dependency.mo
    assert_command dfx deploy -vv dependency
    assert_not_contains '"dependent.mo"'
    assert_contains '"dependency.mo"'

    assert_command dfx deploy -vv dependent
    assert_contains '"dependent.mo"'
    assert_not_contains '"dependency.mo"'

    touch lib.mo
    assert_command dfx deploy -vv dependent
    assert_contains '"dependent.mo"'
    assert_contains '"dependency.mo"'

    touch lib.mo
    assert_command dfx deploy -vv dependency
    assert_contains '"dependency.mo"'
    assert_not_contains '"dependent.mo"'
}

@test "trying to break dependency compiling: build" {
    dfx_start

    assert_command dfx canister create dependency
    assert_command dfx canister create dependent
    assert_command dfx build -vv dependent
    assert_contains '"dependent.mo"'
    assert_contains '"dependency.mo"'

    touch dependent.mo
    assert_command dfx build -vv dependent
    assert_contains '"dependent.mo"'
    assert_not_contains '"dependency.mo"'

    touch dependency.mo
    assert_command dfx build -vv dependent
    assert_contains '"dependent.mo"'
    assert_contains '"dependency.mo"'

    touch dependency.mo
    assert_command dfx build -vv dependency
    assert_not_contains '"dependent.mo"'
    assert_contains '"dependency.mo"'

    assert_command dfx build -vv dependent
    assert_contains '"dependent.mo"'
    assert_not_contains '"dependency.mo"'

    touch lib.mo
    assert_command dfx build -vv dependent
    assert_contains '"dependent.mo"'
    assert_contains '"dependency.mo"'

    touch lib.mo
    assert_command dfx build -vv dependency
    assert_contains '"dependency.mo"'
    assert_not_contains '"dependent.mo"'
}
