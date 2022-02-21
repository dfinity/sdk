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
  dfx canister call hello inc '()'
  dfx config canisters/hello/main v2.mo
  dfx deploy
  dfx canister call hello read '()'
  assert_match "(1 : nat)"
}

@test "add non-optional field in stable variable upgrade" {
    install_asset upgrade
    dfx_start
    dfx deploy
    dfx canister call hello inc '()'
    dfx config canisters/hello/main v2_bad.mo
    echo yes | (
      dfx deploy
      assert_match "Stable interface compatibility check failed for canister 'hello'"
    )
    dfx canister call hello read '()'
    assert_match "(0 : nat)"
}
