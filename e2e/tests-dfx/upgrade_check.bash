#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    dfx_new hello
}

teardown() {
    dfx_stop

    standard_teardown
}

@test "safe upgrade by adding a new stable variable" {
  install_asset upgrade
  dfx_start
  dfx deploy
  dfx canister call hello_backend inc '()'
  jq '.canisters.hello_backend.main="v2.mo"' dfx.json | sponge dfx.json
  dfx deploy
  assert_command dfx canister call hello_backend read '()'
  assert_match "(1 : nat)"
}

@test "changing stable variable from Int to Nat is not allowed" {
    install_asset upgrade
    dfx_start
    dfx deploy
    dfx canister call hello_backend inc '()'
    jq '.canisters.hello_backend.main="v2_bad.mo"' dfx.json | sponge dfx.json
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
    jq '.canisters.hello_backend.main="v2_bad.mo"' dfx.json | sponge dfx.json
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
    jq '.canisters.hello_backend.main="v3_bad.mo"' dfx.json | sponge dfx.json
    echo yes | (
      assert_command dfx deploy
      assert_match "Candid interface compatibility check failed"
    )
    assert_command dfx canister call hello_backend read2 '()'
    assert_match "(1 : int)"
}
