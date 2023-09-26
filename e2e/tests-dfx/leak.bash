#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup

  dfx_new
}

teardown() {
  dfx_stop

  standard_teardown
}

@test "repeated install wasm" {
  install_asset custom_canister
  install_asset wasm/identity
  dfx_start
  dfx deploy
  for _ in {1..50}
  do
  echo yes | dfx canister install --mode=reinstall custom
  done
}
