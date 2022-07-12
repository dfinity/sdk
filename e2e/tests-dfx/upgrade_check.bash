#!/usr/bin/env bats

load ../utils/_

setup() {
    # We want to work from a temporary directory, different for every test.
    x=$(mktemp -d -t dfx-e2e-XXXXXXXX)
    export DFX_CACHE_ROOT="$x"
    export DFX_CONFIG_ROOT="$x"
    cd "$x" || exit
    export RUST_BACKTRACE=1

    dfx_new hello
}

teardown() {
    dfx_stop
    rm -rf "$DFX_CACHE_ROOT"
    rm -rf "$DFX_CONFIG_ROOT"
}

@test "safe upgrade by adding a new stable variable" {
  install_asset upgrade
  dfx_start
  dfx deploy
  dfx canister call hello_backend inc '()'
    # shellcheck disable=SC2094
  cat <<<"$(jq '.canisters.hello_backend.main="v2.mo"' dfx.json)" >dfx.json
  dfx deploy
  assert_command dfx canister call hello_backend read '()'
  assert_match "(1 : nat)"
}

@test "changing stable variable from Int to Nat is not allowed" {
    install_asset upgrade
    dfx_start
    dfx deploy
    dfx canister call hello_backend inc '()'
    # shellcheck disable=SC2094
    cat <<<"$(jq '.canisters.hello_backend.main="v2_bad.mo"' dfx.json)" >dfx.json
    echo yes | (
      assert_command dfx deploy
      assert_match "Stable interface compatibility check failed"
    )
    assert_command dfx canister call hello_backend read '()'
    assert_match "(0 : nat)"
}

@test "changing stable variable from Int to Nat with reinstall is allowed" {
    install_asset upgrade
    dfx_start
    dfx deploy
    dfx canister call hello_backend inc '()'
    # shellcheck disable=SC2094
    cat <<<"$(jq '.canisters.hello_backend.main="v2_bad.mo"' dfx.json)" >dfx.json
    dfx build
    echo yes | dfx canister install hello_backend --mode=reinstall
    assert_command dfx canister call hello_backend read '()'
    assert_match "(0 : nat)"
}

@test "warning for changing method name" {
    install_asset upgrade
    dfx_start
    dfx deploy
    dfx canister call hello_backend inc '()'
    # shellcheck disable=SC2094
    cat <<<"$(jq '.canisters.hello_backend.main="v3_bad.mo"' dfx.json)" >dfx.json
    echo yes | (
      assert_command dfx deploy
      assert_match "Candid interface compatibility check failed"
    )
    assert_command dfx canister call hello_backend read2 '()'
    assert_match "(1 : int)"
}
