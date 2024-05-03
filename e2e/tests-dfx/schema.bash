#!/usr/bin/env bats

load ../utils/_

setup() {
  standard_setup
}

teardown() {
  dfx_stop
  standard_teardown
}

@test "dfx schema prints valid json" {
  assert_command dfx schema --outfile out.json
  # make sure out.json contains exactly one json object
  assert_command jq type out.json
  assert_eq '"object"'
}

@test "dfx schema still works with broken dfx.json" {
  echo '{}' | jq '.broken_key="blahblahblah"' > dfx.json
  assert_command dfx schema
}

@test "dfx schema can display for networks" {
  assert_command dfx schema --for networks --outfile out.json
  # make sure out.json contains exactly one json object
  assert_command jq type out.json
  assert_eq '"object"'
}

@test "dfx schema can display for dfx-metadata" {
  assert_command dfx schema --for dfx-metadata --outfile out.json
  # make sure out.json contains exactly one json object
  assert_command jq type out.json
  assert_eq '"object"'
}
