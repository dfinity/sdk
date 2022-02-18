#!/usr/bin/env bats

load ../utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    x=$(mktemp -d -t dfx-e2e-XXXXXXXX)
    export DFX_CONFIG_ROOT="$x"
    cd "$x" || exit
    export RUST_BACKTRACE=1

    dfx_new hello
}

teardown() {
    dfx_stop
    rm -rf "$DFX_CONFIG_ROOT"
}

@test "add optional field in stable variable upgrade" {
  install_asset upgrade
  dfx_start
  dfx deploy
  dfx canister call hello add '()'
  dfx config canisters/hello/main database_v2.mo
  dfx deploy
  dfx canister call hello dump '()'
  assert_match "hjkhgjkh"
}

@test "add non-optional field in stable variable upgrade" {
    install_asset upgrade
    dfx_start
    dfx deploy
    dfx canister call hello add '()'
    dfx config canisters/hello/main database_v2_bad.mo
    echo yes | (
      dfx deploy
      assert_match "Stable interface compatibility check failed"
    )
    dfx canister call hello dump '()'
    assert_match "[]"
}
