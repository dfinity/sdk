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

@test "Candid UI" {
  dfx_start
  dfx deploy
  ID=$(dfx canister id __Candid_UI)
  PORT=$(get_webserver_port)
  assert_command curl http://raw.localhost:"$PORT"/?canisterId="$ID"
  assert_match "Candid UI"
}
