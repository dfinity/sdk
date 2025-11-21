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

@test "provides core package location by default" {
  install_asset core

  dfx_start
  dfx canister create --all
  dfx build
  dfx canister install e2e_project_backend

  assert_command dfx canister call --query e2e_project_backend test_core
  assert_eq '(true)'
}

@test "does not provide core package if there is a packtool" {
  install_asset core
  jq '.defaults.build.packtool="echo"' dfx.json | sponge dfx.json

  dfx_start
  dfx canister create --all
  assert_command_fail dfx build
  assert_match 'import error \[M0010\], package "core" not defined'
}
